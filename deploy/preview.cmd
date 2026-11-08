@echo off
rem Windows launcher for preview.sh (PowerShell or cmd): build this working tree's site on
rem quidnug-docker and serve it at http://127.0.0.1:5320 through an SSH tunnel, until Ctrl+C.
rem Uses Git for Windows' bash on purpose: a bare `bash` on Windows is often WSL's,
rem which has no ssh setup for quidnug-docker.
setlocal
set "GITBASH=%ProgramFiles%\Git\bin\bash.exe"
if not exist "%GITBASH%" for /f "delims=" %%G in ('where git.exe 2^>nul') do set "GITBASH=%%~dpG..\bin\bash.exe"
if not exist "%GITBASH%" (echo Git for Windows' bash.exe not found 1>&2 & exit /b 1)
set "SCRIPT=%~dp0preview.sh"
"%GITBASH%" "%SCRIPT:\=/%" %*
