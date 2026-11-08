@echo off
rem Windows launcher for check.sh (PowerShell or cmd):
rem   deploy\check.cmd          bun run build + bun run test of this working tree, on quidnug-docker
rem   deploy\check.cmd build    the build only
rem Uses Git for Windows' bash on purpose: a bare `bash` on Windows is often WSL's,
rem which has no ssh setup for quidnug-docker.
setlocal
set "GITBASH=%ProgramFiles%\Git\bin\bash.exe"
if not exist "%GITBASH%" for /f "delims=" %%G in ('where git.exe 2^>nul') do set "GITBASH=%%~dpG..\bin\bash.exe"
if not exist "%GITBASH%" (echo Git for Windows' bash.exe not found 1>&2 & exit /b 1)
set "SCRIPT=%~dp0check.sh"
"%GITBASH%" "%SCRIPT:\=/%" %*
