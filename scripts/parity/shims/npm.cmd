@echo off
rem DARE parity shim for `npm` - records argv, never runs the real tool. See _shim.mjs.
node "%~dp0_shim.mjs" npm %*
