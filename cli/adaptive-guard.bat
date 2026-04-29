@echo off
REM adaptive-guard.bat - Global CLI dispatcher for Windows cmd.
REM
REM This script is installed to a directory on the user's PATH by
REM scripts\setup-global.bat. It reads the repo location from
REM %USERPROFILE%\.adaptive-guard\config and dispatches subcommands
REM to the corresponding script in the repo.

setlocal enabledelayedexpansion

set "CONFIG_FILE=%USERPROFILE%\.adaptive-guard\config"

if not exist "%CONFIG_FILE%" (
    echo Error: adaptive-guard is not set up on this machine.
    echo Run setup-global.bat from the repo first:
    echo    cd ^<repo-path^> ^&^& scripts\setup-global.bat
    exit /b 1
)

REM Read REPO_ROOT from config file (first line is KEY=VALUE)
for /f "tokens=1,* delims==" %%A in ('type "%CONFIG_FILE%"') do (
    if "%%A"=="REPO_ROOT" set "REPO_ROOT=%%B"
)

if "%REPO_ROOT%"=="" (
    echo Error: REPO_ROOT not found in %CONFIG_FILE%.
    echo Re-run setup-global.bat to fix.
    exit /b 1
)

if not exist "%REPO_ROOT%" (
    echo Error: repo path does not exist: %REPO_ROOT%
    echo The repo may have been moved. Re-run setup-global.bat from the new location.
    exit /b 1
)

REM Dispatch
if "%1"=="" goto :help
if "%1"=="help" goto :help
if "%1"=="--help" goto :help
if "%1"=="-h" goto :help
if "%1"=="version" goto :version
if "%1"=="--version" goto :version

set "SUBCOMMAND=%1"
shift

REM Shift args so %* gives only the remaining args
set "ARGS="
:collect_args
if "%1"=="" goto :dispatch
set "ARGS=%ARGS% %1"
shift
goto :collect_args

:dispatch
set "SCRIPT=%REPO_ROOT%\scripts\%SUBCOMMAND%.bat"
if not exist "%SCRIPT%" (
    echo Error: unknown subcommand: %SUBCOMMAND%
    echo Run 'adaptive-guard help' for the list.
    exit /b 1
)

call "%SCRIPT%"%ARGS%
exit /b %ERRORLEVEL%

:help
echo adaptive-guard - quality control layer for Claude Code
echo.
echo USAGE:
echo   adaptive-guard ^<command^> [args...]
echo.
echo COMMANDS:
echo   install [--profile balanced^|strict^|lenient]
echo                        Install the Stop hook into ~/.claude/settings.json.
echo   uninstall            Remove the hook.
echo   stats [--last N] [--today] [--recent N] [--session ID]
echo                        Inspect decision history.
echo   explain              Score any prompt/response interactively.
echo   dashboard            Launch the desktop dashboard window.
echo   version              Print version.
echo   help                 This message.
echo.
echo REPO:   %REPO_ROOT%
echo.
exit /b 0

:version
if exist "%REPO_ROOT%\VERSION" (
    for /f %%V in ('type "%REPO_ROOT%\VERSION"') do echo adaptive-guard %%V
) else (
    echo adaptive-guard (version unknown)
)
exit /b 0
