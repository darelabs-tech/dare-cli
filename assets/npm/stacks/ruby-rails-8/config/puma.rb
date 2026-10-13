# frozen_string_literal: true

# Puma configuration — https://github.com/puma/puma

threads_count = ENV.fetch("RAILS_MAX_THREADS", 5)
threads threads_count, threads_count

port ENV.fetch("PORT", 3000)

environment ENV.fetch("RAILS_ENV", "development")

# Allow puma to be restarted by `bin/rails restart`.
plugin :tmp_restart

# Run the Solid Queue supervisor inside Puma in single-process mode (opt-in).
# plugin :solid_queue if ENV["SOLID_QUEUE_IN_PUMA"]

pidfile ENV.fetch("PIDFILE", "tmp/pids/server.pid")
