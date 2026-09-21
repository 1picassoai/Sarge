@echo off
REM The KV cache POC. Same build environment as compile.cmd - see that file for why
REM each variable is here. The one that bites: CUDA 13 keeps cudart64_13/cublas64_13
REM in bin\x64, and without it the exe builds and then dies at launch with
REM 0xC0000135, STATUS_DLL_NOT_FOUND.
REM
REM   kv.cmd --build              prefill the vetted rules once, save the state
REM   kv.cmd --task "..."         load that state, then answer
REM   kv.cmd --task "..." --cold  same task with no state, for the comparison
call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
set "CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4"
set "CUDA_PATH_V13_4=%CUDA_PATH%"
set "CUDAToolkit_ROOT=%CUDA_PATH%"
set "PATH=%CUDA_PATH%\bin\x64;%CUDA_PATH%\bin;%PATH%"
set "LIBCLANG_PATH=C:\Program Files\LLVM\bin"
set "CMAKE_BUILD_PARALLEL_LEVEL=4"
set "CUDAARCHS=89"
cd /d "%~dp0"
echo === START %TIME% ===
cargo run --release --features model,cuda --bin kv -- %*
echo === EXIT %ERRORLEVEL% AT %TIME% ===
