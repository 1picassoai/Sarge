@echo off
REM Type-check the compile binary in the same environment build-compile.cmd builds it in.
REM Same notes apply - see compile.cmd. Four cores, sm_89, CUDA 13.4, libclang for bindgen.
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
cargo check -j 4 --release --features model,cuda --bin compile
echo === EXIT %ERRORLEVEL% ===
