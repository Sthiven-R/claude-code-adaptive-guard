@echo off
REM dashboard.bat — Windows cmd wrapper for dashboard.sh.
REM Forwards all args to the bash script.

setlocal

REM Locate the repo root (one level up from scripts/)
set "REPO_ROOT=%~dp0.."

REM Find bash. Prefer Git Bash at usr\bin (matches what install.rs and
REM install.sh emit into the hook command — keep one source of truth).
set "BASH_EXE="
if exist "C:\Program Files\Git\usr\bin\bash.exe" (
    set "BASH_EXE=C:\Program Files\Git\usr\bin\bash.exe"
) else if exist "%ProgramFiles%\Git\usr\bin\bash.exe" (
    set "BASH_EXE=%ProgramFiles%\Git\usr\bin\bash.exe"
) else (
    where bash >nul 2>nul && set "BASH_EXE=bash"
)

if "%BASH_EXE%"=="" (
    echo Error: bash not found.
    echo Install Git for Windows or add bash to PATH.
    exit /b 1
)

"%BASH_EXE%" "%~dp0dashboard.sh" %*
