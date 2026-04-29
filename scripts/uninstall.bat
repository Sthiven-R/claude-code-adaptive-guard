@echo off
REM uninstall.bat — Windows cmd wrapper for uninstall.sh.

setlocal

set "BASH_EXE="
if exist "C:\Program Files\Git\usr\bin\bash.exe" (
    set "BASH_EXE=C:\Program Files\Git\usr\bin\bash.exe"
) else if exist "%ProgramFiles%\Git\usr\bin\bash.exe" (
    set "BASH_EXE=%ProgramFiles%\Git\usr\bin\bash.exe"
) else (
    where bash >nul 2>nul && set "BASH_EXE=bash"
)

if "%BASH_EXE%"=="" (
    echo Error: bash not found. Install Git for Windows.
    exit /b 1
)

"%BASH_EXE%" "%~dp0uninstall.sh" %*
