@echo off
rem DARE parity shim for `gem` - records argv, never runs the real tool. See _shim.mjs.
node "%~dp0_shim.mjs" gem %*
