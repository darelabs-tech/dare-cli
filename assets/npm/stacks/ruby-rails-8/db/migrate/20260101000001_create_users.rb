# frozen_string_literal: true

# Creates the users table backing app/models/user.rb (email, name, active, admin).
class CreateUsers < ActiveRecord::Migration[8.0]
  def change
    create_table :users do |t|
      t.string  :email,  null: false
      t.string  :name,   null: false
      t.boolean :active, null: false, default: true
      t.boolean :admin,  null: false, default: false

      t.timestamps
    end

    add_index :users, :email, unique: true
  end
end
