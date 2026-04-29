@echo off
REM setup-global.bat - One-time setup: install 'adaptive-guard' as a global
REM command in Windows cmd. Safe, idempotent.
REM
REM Steps:
REM   1. Resolve repo root (parent of this script).
REM   2. Create %USERPROFILE%\bin\ if missing.
REM   3. Create %USERPROFILE%\.adaptive-guard\config with REPO_ROOT.
REM   4. Copy cli\adaptive-guard.bat -> %USERPROFILE%\bin\adaptive-guard.bat.
REM   5. If %USERPROFILE%\bin is NOT on user PATH, add it via setx.
REM
REM Run from the repo root:
REM   scripts\setup-global.bat

setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "REPO_ROOT=%SCRIPT_DIR%.."
pushd "%REPO_ROOT%" >nul
set "REPO_ROOT=%CD%"
popd >nul

echo Setting up adaptive-guard as a global command on this machine.
echo.
echo   Repo: %REPO_ROOT%
echo.

REM 1. Bin dir
set "BIN_DIR=%USERPROFILE%\bin"
if not exist "%BIN_DIR%" (
    mkdir "%BIN_DIR%"
    echo Created %BIN_DIR%
)

REM 2. Config dir and file
set "CONF_DIR=%USERPROFILE%\.adaptive-guard"
if not exist "%CONF_DIR%" (
    mkdir "%CONF_DIR%"
)
set "CONF_FILE=%CONF_DIR%\config"
(
    echo REPO_ROOT=%REPO_ROOT%
) > "%CONF_FILE%"
echo Wrote config: %CONF_FILE%

REM 3. Copy dispatcher
set "SRC=%REPO_ROOT%\cli\adaptive-guard.bat"
set "DST=%BIN_DIR%\adaptive-guard.bat"
if not exist "%SRC%" (
    echo Error: dispatcher not found at %SRC%
    exit /b 1
)
copy /Y "%SRC%" "%DST%" >nul
if errorlevel 1 (
    echo Error: failed to copy dispatcher to %DST%
    exit /b 1
)
echo Installed dispatcher: %DST%

REM 4. Check PATH and add if missing
echo %PATH% | findstr /I /C:"%BIN_DIR%" >nul
if errorlevel 1 (
    echo.
    echo %BIN_DIR% is not on your PATH. Adding it now via setx.
    REM Get current user PATH ^(not the combined PATH^)
    for /f "usebackq tokens=2,*" %%A in (`reg query "HKCU\Environment" /v PATH 2^>nul ^| findstr /I "REG_"`) do set "USER_PATH=%%B"
    if "!USER_PATH!"=="" (
        setx PATH "%BIN_DIR%" >nul
    ) else (
        setx PATH "!USER_PATH!;%BIN_DIR%" >nul
    )
    echo.
    echo   IMPORTANT: open a NEW cmd window for the change to take effect.
    echo   Then run:  adaptive-guard help
) else (
    echo.
    echo %BIN_DIR% is already on PATH. You can use 'adaptive-guard' right away.
)

echo.
echo Setup complete.
echo.
echo Try:
echo   adaptive-guard help
echo   adaptive-guard version
echo   adaptive-guard stats --last
echo.

endlocal
exit /b 0
