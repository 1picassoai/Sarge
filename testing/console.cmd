@echo off
REM The console. Start it, then open http://127.0.0.1:8420
REM The page holds no logic - it shells the agent, which shells the handshake.
cd /d "%~dp0"
python tools\console.py %*
