@echo off
REM Build and run the merged binary: rules and the model in one process, on the GPU.
REM
REM Two build-time system dependencies, neither of them cargo's to fetch:
REM   libclang  - llama-cpp-sys-2 generates its bindings with bindgen
REM   CUDA      - llama.cpp's kernels are compiled from source
REM
REM CUDA 13.4 is required. 12.0 is also installed here and the machine's CUDA_PATH
REM still points at it, but 12.0 predates Visual Studio 18: its MSBuild integration
REM was never placed there, and its host-compiler guard refuses MSVC 14.4x.
REM
REM The failure to know about: MSBuild resolved an empty CudaToolkitDir and died with
REM "The CUDA Toolkit directory '' does not exist". The CUDA 13.4.props file fills that
REM from CudaToolkitVersionedPath, which comes from CUDA_PATH_V13_4 - and the 13.4
REM installer left that variable unset while the machine's CUDA_PATH still pointed at
REM 12.0. Setting it here is the fix.
REM
REM Do NOT set CMAKE_GENERATOR_TOOLSET=cuda=<path>. The cmake crate passes its own
REM -Thost=x64, CMake sees two different toolsets across configures and refuses:
REM "generator toolset: host=x64 does not match the toolset used previously".
call "C:\Program Files (x86)\Microsoft Visual Studio\18\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul
set "CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4"
set "CUDA_PATH_V13_4=%CUDA_PATH%"
set "CUDAToolkit_ROOT=%CUDA_PATH%"
REM CUDA 13 moved the runtime DLLs (cudart64_13, cublas64_13) into bin\x64. With only
REM bin on PATH the exe builds fine and then dies at launch with 0xC0000135, DLL not
REM found. Both folders go on: bin for nvcc, bin\x64 for the runtime.
set "PATH=%CUDA_PATH%\bin\x64;%CUDA_PATH%\bin;%PATH%"
set "LIBCLANG_PATH=C:\Program Files\LLVM\bin"

REM Keep the one-off compile from taking the laptop down. The crate defaults to every
REM core (32 here) and lets nvcc build kernels for every GPU generation - that pushed
REM memory to 88% and the fans to full. Four cores, and only the 4070's architecture
REM (sm_89), so it is slow but the machine stays usable.
set "CMAKE_BUILD_PARALLEL_LEVEL=4"
set "CMAKE_CUDA_ARCHITECTURES=89"
set "CUDAARCHS=89"
cd /d "%~dp0"
echo === START %TIME% ===
nvcc --version | findstr release
cargo run --release --features model,cuda --bin compile -- %*
echo === EXIT %ERRORLEVEL% AT %TIME% ===
