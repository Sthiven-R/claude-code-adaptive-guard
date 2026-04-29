@echo off
REM kill-port.bat - Free port 1420 if a previous dev run left it held.
REM Useful when `npm run tauri dev` fails with "Port 1420 is already in use".

setlocal enabledelayedexpansion

set "PORT=1420"
if not "%1"=="" set "PORT=%1"

echo Looking for processes on port %PORT%...
for /f "tokens=5" %%a in ('netstat -ano ^| findstr ":%PORT%"') do (
    if not "%%a"=="0" (
        echo Killing PID %%a
        taskkill /F /PID %%a 2>nul
    )
)
echo Done. Port %PORT% should now be free.
endlocal
