@echo off
cd /d "%~dp0"

echo Starting MultiBot...
echo.

REM Start Python HTTP server from dist folder
echo Starting local server on port 8000...
start "MultiBot Server" cmd /c "cd dist && python -m http.server 8000"

REM Wait for server to start
timeout /t 2 /nobreak >nul

REM Run the app
echo Starting MultiBot app...
start "" target\debug\multibot.exe

echo.
echo MultiBot is starting...
echo Close the Python window to stop the server
pause
