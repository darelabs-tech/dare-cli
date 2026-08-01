---
name: skill-rails-api
description: Padrões DARE para APIs em Ruby on Rails 8 — API mode, ActiveRecord, Solid Queue, Solid Cable, Action Cable, strong parameters, services (interactors), serializers (Blueprinter/Alba), Devise/JWT, rack-attack, rswag/grape-swagger.
---

# DARE Rails API Skill

Você é um desenvolvedor sênior Ruby on Rails 8.x especializado em APIs. Seu objetivo é gerar código **idiomático Rails, com Layered Design, Solid Queue/Cable, serializers explícitos e auth/autz robustos**.

## Quando usar

- Projeto Rails 8 API novo via DARE
- Adicionar feature em API Rails existente
- Migrar de Rails 7 → 8 (incluindo Solid Queue + Solid Cable)
- Auditar projeto Rails para conformidade DARE

## Stack canônica

- **Ruby 3.3+**
- **Rails 8.x** modo API (`rails new --api`)
- **PostgreSQL 16**
- **Solid Queue** (substitui Sidekiq/DelayedJob — built-in Rails 8)
- **Solid Cable** (substitui Redis para Action Cable — built-in Rails 8)
- **Devise** ou **JWT puro** (devise-jwt) para auth
- **Pundit** ou **CanCanCan** para autorização
- **Blueprinter** ou **Alba** para serializers (substituem Jbuilder em APIs)
- **rack-attack** para rate limit
- **rswag** para OpenAPI/Swagger
- **RSpec** + **FactoryBot** + **Faker**
- **Rubocop** + **rubocop-rails-omakase** (estilo oficial Rails)
- **bundler-audit** para CVEs

## Layered Design em Rails

Mapeamento DARE → Rails:

| Camada DARE | Pasta Rails |
|---|---|
| Handler | `app/controllers/api/v1/` |
| Service | `app/services/` (ou `app/interactors/`) |
| Repository | `app/repositories/` (opcional — Rails usa AR direto frequentemente) |
| Model | `app/models/` |
| Presenter | `app/serializers/` ou `app/blueprints/` |

> Em Rails 8 API, Repositories são opcionais — ActiveRecord queries em Services é comum quando bem encapsulado.

## Controllers (Handler)

```ruby
# app/controllers/api/v1/users_controller.rb
module Api
  module V1
    class UsersController < ApplicationController
      before_action :authenticate_user!

      def create
        result = RegisterUser.call(params: user_params)
        if result.success?
          render json: UserBlueprint.render(result.user), status: :created
        else
          render json: { error: result.error_code }, status: :conflict
        end
      end

      private

      def user_params
        params.require(:user).permit(:email, :name, :password)
      end
    end
  end
end
```

Regras:
- Apenas: autenticar → strong params → chamar Service → renderizar Serializer
- NUNCA: query AR no controller, lógica de negócio, validação manual

## Services (Interactors)

Padrão Interactor (com a gem `interactor` ou manual):

```ruby
# app/services/register_user.rb
class RegisterUser
  include Interactor

  def call
    user = User.find_by(email: context.params[:email])
    if user
      context.fail!(error_code: 'USER_EXISTS')
    else
      context.user = User.create!(
        email: context.params[:email],
        name: context.params[:name],
        password: context.params[:password],
      )
    end
  end
end
```

Ou padrão "Service Object" puro:

```ruby
class RegisterUser
  Result = Struct.new(:success?, :user, :error_code)

  def initialize(params)
    @params = params
  end

  def call
    return Result.new(false, nil, 'USER_EXISTS') if User.exists?(email: @params[:email])
    user = User.create!(@params)
    Result.new(true, user, nil)
  end
end

# Uso: RegisterUser.new(params).call
```

## Models

```ruby
class User < ApplicationRecord
  has_secure_password

  validates :email, presence: true, uniqueness: true, format: URI::MailTo::EMAIL_REGEXP
  validates :name, presence: true
  validates :password, length: { minimum: 12 }, if: :password_required?

  has_many :sessions, dependent: :destroy

  scope :active, -> { where(deleted_at: nil) }
end
```

Use `has_secure_password` (Rails built-in) com Argon2 ou Bcrypt.

## Serializers (Blueprinter)

```ruby
# app/blueprints/user_blueprint.rb
class UserBlueprint < Blueprinter::Base
  identifier :id

  fields :email, :name

  field :created_at do |user|
    user.created_at.iso8601
  end
end

# Uso:
UserBlueprint.render(user)
UserBlueprint.render(users, view: :extended)
```

## Auth (JWT com devise-jwt)

```ruby
# Gemfile
gem 'devise'
gem 'devise-jwt'

# config/initializers/devise.rb
config.jwt do |jwt|
  jwt.secret = Rails.application.credentials.devise_jwt_secret_key!
  jwt.dispatch_requests = [['POST', %r{^/api/v1/login$}]]
  jwt.revocation_requests = [['DELETE', %r{^/api/v1/logout$}]]
  jwt.expiration_time = 15.minutes.to_i
end
```

Refresh token com rotação em uma tabela separada (não confie em logout só no client).

## Rate limit (rack-attack)

```ruby
# config/initializers/rack_attack.rb
class Rack::Attack
  throttle('login/ip', limit: 5, period: 15.minutes) do |req|
    req.ip if req.path == '/api/v1/login' && req.post?
  end

  throttle('api/ip', limit: 100, period: 1.minute) do |req|
    req.ip if req.path.start_with?('/api/')
  end

  throttle('login/email', limit: 5, period: 15.minutes) do |req|
    if req.path == '/api/v1/login' && req.post?
      req.params['email'].to_s.downcase.gsub(/\s+/, '')
    end
  end
end
```

## Solid Queue (jobs)

```ruby
# app/jobs/send_welcome_email_job.rb
class SendWelcomeEmailJob < ApplicationJob
  queue_as :default

  def perform(user_id)
    user = User.find(user_id)
    UserMailer.welcome(user).deliver_now
  end
end

# config/queue.yml configurado para Solid Queue (default em Rails 8)
```

## Solid Cable + Action Cable

```ruby
# app/channels/notifications_channel.rb
class NotificationsChannel < ApplicationCable::Channel
  def subscribed
    stream_for current_user
  end
end

# Broadcast
NotificationsChannel.broadcast_to(user, type: 'message.sent.v1', payload: {...})

# Authorization
class ApplicationCable::Connection < ActionCable::Connection::Base
  identified_by :current_user

  def connect
    self.current_user = find_verified_user
  end

  private

  def find_verified_user
    if (user = decode_jwt(request.params[:token]))
      user
    else
      reject_unauthorized_connection
    end
  end
end
```

## OpenAPI (rswag)

```ruby
# Gemfile
group :development, :test do
  gem 'rswag-specs'
end

# spec/integration/users_spec.rb
require 'swagger_helper'

RSpec.describe 'Users API', type: :request do
  path '/api/v1/users' do
    post 'Creates a user' do
      tags 'Users'
      consumes 'application/json'
      parameter name: :user, in: :body, schema: {
        type: :object,
        properties: {
          email: { type: :string },
          name: { type: :string },
          password: { type: :string, minLength: 12 },
        },
        required: %w[email name password],
      }
      response '201', 'user created' do ... end
      response '409', 'duplicate email' do ... end
    end
  end
end

# Gerar OpenAPI:
# rake rswag:specs:swaggerize
# Saída: swagger/v1/swagger.yaml (export para JSON via tool)
```

## Testes (RSpec)

```ruby
# spec/services/register_user_spec.rb
RSpec.describe RegisterUser do
  let(:params) { { email: 'jane@example.com', name: 'Jane', password: 'longsecret123' } }

  it 'cria usuário com sucesso' do
    result = described_class.new(params).call
    expect(result.success?).to be true
    expect(result.user.email).to eq 'jane@example.com'
  end

  it 'falha se email já existe' do
    User.create!(params)
    result = described_class.new(params).call
    expect(result.success?).to be false
    expect(result.error_code).to eq 'USER_EXISTS'
  end
end

# spec/requests/api/v1/users_spec.rb
RSpec.describe 'POST /api/v1/users', type: :request do
  let(:admin) { create(:user, :admin) }

  it 'cria com sucesso' do
    post '/api/v1/users',
         params: { user: { email: 'jane@example.com', name: 'Jane', password: 'longsecret123' } },
         headers: { 'Authorization' => "Bearer #{token_for(admin)}" }
    expect(response).to have_http_status(:created)
  end

  it 'retorna 409 em duplicate' do
    create(:user, email: 'taken@example.com')
    post '/api/v1/users',
         params: { user: { email: 'taken@example.com', name: 'X', password: 'longsecret123' } },
         headers: { 'Authorization' => "Bearer #{token_for(admin)}" }
    expect(response).to have_http_status(:conflict)
  end
end
```

## Antipatterns

| AP | Antipattern | Correção |
|---|---|---|
| AP-01 | Query AR no controller | Service ou Repository |
| AP-02 | Lógica no controller | Service object / Interactor |
| AP-03 | Retornar `render json: user` | Blueprinter/Alba/serializer |
| AP-04 | Fat Model (Model com 500+ linhas) | Concerns + Services |
| AP-05 | Skip strong parameters | sempre `params.require(...).permit(...)` |
| AP-06 | Sem `rack-attack` em login | rate limit obrigatório |
| AP-07 | JWT secret em código | `Rails.application.credentials` |
| AP-08 | Logs com PII (senha, token) | `filter_parameters` configurado |
| AP-09 | N+1 sem `includes` | usar `bullet` gem em dev |
| AP-10 | Renderizar Model direto na response | Blueprinter |

## Segurança

- `has_secure_password` com BCrypt cost ≥ 12 (default Rails OK)
- `force_ssl = true` em produção
- `secure_headers` gem ou middleware nativo Rails 8
- CORS com `rack-cors` (origens explícitas)
- `filter_parameters` para `password`, `token`, `secret`
- bundler-audit no CI

## CI

```bash
bundle exec rubocop
bundle exec rspec
bundle exec bundler-audit check --update
bundle exec rails db:schema:dump   # garante schema sync
```

## Como aplicar

### Passo 1: Audit

```bash
grep -rn "\.where\|\.find_by\|\.create\|\.update" app/controllers/  # AP-01
grep -rn "render json: @user\b" app/controllers/                   # AP-03
grep -rn "params\[:" app/controllers/ | grep -v "permit"            # AP-05
```

### Passo 2: Extrair Services

Lógica em controller > 10 linhas → Service object com `.call`.

### Passo 3: Adicionar Blueprints/Alba

Substituir `render json: object` por `render json: ObjectBlueprint.render(object)`.

### Passo 4: rack-attack + filtros

Configurar throttles para `/login`, `/api/*`. `filter_parameters` para campos sensíveis.

### Passo 5: rswag para OpenAPI

Documente endpoints em `spec/integration/`, rode `rswag:specs:swaggerize`.

## Dicas

- **Combine** com `dare-docker` (Ruby 3.3-alpine multi-stage)
- **Combine** com `dare-realtime` para Action Cable + Solid Cable
- **Use** `dare-security` para auditoria OWASP + bundler-audit

---

Esta skill é parte do DARE Method e está sob licença MIT.
