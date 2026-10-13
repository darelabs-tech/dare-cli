@echo off
rem DARE parity shim for `rails` - records argv, never runs the real tool. See _shim.mjs.
node "%~dp0_shim.mjs" rails %*
