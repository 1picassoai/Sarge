@echo off
REM THE POINTING EXPERIMENT. Same rule, same task, same model, greedy. One variable: where
REM the rule sits. Runs the four arms one after another on the one GPU, never together.
REM
REM   point.cmd            all four arms
REM   point.cmd kv         one arm
REM
REM Design: docs\KV-POINTING.md. Criteria fixed before running, as with the A/B.
set "CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4"
set "PATH=%CUDA_PATH%\bin\x64;%CUDA_PATH%\bin;%PATH%"
cd /d "%~dp0"

set "TASK=Write a single file Program.cs for an ASP.NET minimal API named WorkshopTools, target net8.0, namespace WorkshopTools. Register Entity Framework Core with SQLite reading the connection string from configuration. Define a Tool entity with Id, Name and Price and a ToolContext for it, both in this file. Add two endpoints: GET /tools returns all tools, POST /tools adds one tool. Output only the C# file."
set "RULE=scoped-context-in-handler"
set "BOOK=%USERPROFILE%\source\repos\myshop\.sarge"
set "OUT=%~dp0..\runs\point"
if not exist "%OUT%" mkdir "%OUT%"

if "%~1"=="" (
  call :arm none
  call :arm head
  call :arm tail
  call :arm kv
  call :arm kvs
  call :arm kvr
) else (
  call :arm %~1
)
echo.
echo === RESULTS ===
for %%a in (none head tail kv kvs kvr) do (
  if exist "%OUT%\%%a.txt" (
    echo --- %%a ---
    findstr /B /C:"  Map" /C:"  handlers" /C:"RESULT" "%OUT%\%%a.txt"
  )
)
goto :eof

:arm
echo.
echo ===== ARM %1  %TIME% =====
target\release\point.exe --task "%TASK%" --rule-id %RULE% --repo myshop --rules "%BOOK%" --arm %1 --n 900 > "%OUT%\%1.txt" 2> "%OUT%\%1.err"
type "%OUT%\%1.err"
findstr /B /C:"RESULT" "%OUT%\%1.txt"
goto :eof
