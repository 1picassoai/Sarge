#!/usr/bin/env bash
# Build llama-server with the Sarge handshake compiled in. One organ. macOS / Apple silicon.
#
# The twin of build-organ.cmd. Same law, different toolchain: no HTTP between the handshake
# and llama.cpp, because the handshake is a static library linked into server-context. That
# is structural on both platforms, not a setting.
#
# Metal replaces CUDA here and needs nothing installed - it ships with the OS. clang replaces
# the Visual Studio generator. The 4-core parallel cap is kept: it exists because the
# Captain's laptop went to 88% memory at 32 cores, and a CI runner with 3 cores does not
# want 32 either.
set -euo pipefail

cd "$(dirname "$0")"

export CMAKE_BUILD_PARALLEL_LEVEL=4

HANDSHAKE_LIB="$(cd ../rust/target/release && pwd)/libhandshake.a"
if [ ! -f "$HANDSHAKE_LIB" ]; then
  echo "handshake static library not found: $HANDSHAKE_LIB"
  echo "build it first:  cd ../rust && cargo build --release --features model,metal"
  exit 1
fi

echo "=== CONFIGURE $(date +%T) ==="
cmake -B build -G "Unix Makefiles" \
  -DCMAKE_BUILD_TYPE=Release \
  -DGGML_METAL=ON \
  -DLLAMA_BUILD_SERVER=ON \
  -DLLAMA_BUILD_EXAMPLES=OFF \
  -DLLAMA_BUILD_TESTS=OFF \
  -DLLAMA_CURL=OFF \
  -DHANDSHAKE_LIB="$HANDSHAKE_LIB"

echo "=== BUILD $(date +%T) ==="
cmake --build build --config Release --target llama-server --parallel 4
echo "=== DONE $(date +%T) ==="
