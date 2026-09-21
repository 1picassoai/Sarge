@echo off
REM Build the compile binary without running it. Same environment as compile.cmd -
REM see the notes there for why each line exists. Four cores, sm_89 only.
call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
set "CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4"
set "CUDA_PATH_V13_4=%CUDA_PATH%"
set "CUDAToolkit_ROOT=%CUDA_PATH%"
set "PATH=%CUDA_PATH%\bin\x64;%CUDA_PATH%\bin;%PATH%"
set "LIBCLANG_PATH=C:\Program Files\LLVM\bin"
set "CMAKE_BUILD_PARALLEL_LEVEL=4"
set "CMAKE_CUDA_ARCHITECTURES=89"
set "CUDAARCHS=89"
cd /d "%~dp0"
echo === START %TIME% ===
REM -j 4 is what actually caps the C++ side: the cmake crate reads cargo's NUM_JOBS and
REM passed --parallel 32 when only CMAKE_BUILD_PARALLEL_LEVEL was set.
REM BOTH binaries. handshake.exe is the one the AGENT shells for --check, --learn and
REM --ask; compile.exe is the standalone loop. Building only `compile` left the agent
REM talking to a stale handshake that rejected a flag added minutes earlier.
cargo build -j 4 --release --features model,cuda --bin compile --bin handshake --bin point
echo === EXIT %ERRORLEVEL% AT %TIME% ===
