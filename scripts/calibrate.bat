@echo off
REM calibrate.bat - Windows wrapper that runs calibrate.sh through Git Bash.
REM Mirrors the pattern used by stats.bat / explain.bat.

setlocal

set "BASH_EXE=C:\Program Files\Git\usr\bin\bash.exe"
if not exist "%BASH_EXE%" (
  echo Error: Git Bash not found at %BASH_EXE%.
  echo Install Git for Windows ^(https://git-scm.com/download/win^) and try again.
  exit /b 1
)

set "SCRIPT_DIR=%~dp0"
"%BASH_EXE%" "%SCRIPT_DIR%calibrate.sh" %*

endlocal
