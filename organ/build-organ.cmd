@echo off
REM Build llama-server with the CompilerGPT handshake compiled in. One organ.
REM
REM Vinn's constraints, all from Sun 13 Sep: 4 cores or the laptop goes down; CUDA 13.4
REM pinned ahead of 12.0 on PATH; only the 4070's architecture (sm_89). And no HTTP
REM between handshake and llama.cpp - which this build makes structurally true, since
REM the handshake is a static library linked into server-context.
call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
set "CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4"
set "CUDA_PATH_V13_4=%CUDA_PATH%"
set "PATH=%CUDA_PATH%\bin\x64;%CUDA_PATH%\bin;%PATH%"
set "CMAKE_BUILD_PARALLEL_LEVEL=4"
set "CUDAARCHS=89"
cd /d "%~dp0"
echo === CONFIGURE %TIME% ===
cmake -B build -G "Visual Studio 18 2026" -A x64 ^
  -DGGML_CUDA=ON ^
  -DLLAMA_BUILD_SERVER=ON ^
  -DLLAMA_BUILD_EXAMPLES=OFF ^
  -DLLAMA_BUILD_TESTS=OFF ^
  -DLLAMA_CURL=OFF ^
  -DHANDSHAKE_LIB=%~dp0..\..\rust\target\release\handshake.lib
if errorlevel 1 (echo CONFIGURE FAILED & exit /b 1)
echo === BUILD %TIME% ===
cmake --build build --config Release --target llama-server --parallel 4
echo === EXIT %ERRORLEVEL% AT %TIME% ===
