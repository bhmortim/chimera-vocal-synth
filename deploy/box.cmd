@echo off
rem Windows launcher for box.sh (PowerShell or cmd): run a command against this working
rem tree on quidnug-docker, in the pipeline's toolchain. Quote the command:
rem   deploy\box.cmd "cargo test -p chimera --test engine"
rem Uses Git for Windows' bash on purpose: a bare `bash` on Windows is often WSL's,
rem which has no ssh setup for quidnug-docker.
setlocal
set "GITBASH=%ProgramFiles%\Git\bin\bash.exe"
if not exist "%GITBASH%" for /f "delims=" %%G in ('where git.exe 2^>nul') do set "GITBASH=%%~dpG..\bin\bash.exe"
if not exist "%GITBASH%" (echo Git for Windows' bash.exe not found 1>&2 & exit /b 1)
set "SCRIPT=%~dp0box.sh"
"%GITBASH%" "%SCRIPT:\=/%" %*
