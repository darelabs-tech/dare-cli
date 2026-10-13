# frozen_string_literal: true

# DARE v3.0 — HomeController (full-stack MVC example)
#
# Thin controller: HTTP concerns only. It renders a server-rendered view.
# Business logic belongs in app/services/, data access in app/repositories/ —
# never inline here.
class HomeController < ApplicationController
  def index; end
end
