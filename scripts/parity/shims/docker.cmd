@echo off
rem DARE parity shim for `docker` - records argv, never runs the real tool. See _shim.mjs.
node "%~dp0_shim.mjs" docker %*
