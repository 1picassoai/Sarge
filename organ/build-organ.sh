#!/usr/bin/env bash
# Build llama-server with the Sarge handshake compiled in. One organ. macOS / Apple silicon.
#
# The twin of build-organ.cmd, with one difference that matters: this script does the WHOLE
# job from nothing - clone, checkout, patch, build. The .cmd assumes llama-src is already
# sitting there because a human followed organ/README.md once by hand. CI starts from an
# empty machine every time, so the setup steps live here rather than in a README.
#
# Same law as Windows: no HTTP between the handshake and llama.cpp, because the handshake is
# a static library linked into server-context. Structural on both platforms, not a setting.
#
# Metal replaces CUDA and needs nothing installed - it ships with the OS. clang replaces the
# Visual Studio generator. The parallel cap is kept low on purpose: it exists because 32
# cores put the Captain's laptop at 88% memory, and a 3-core CI runner does not want more
# than it has either.
set -euo pipefail

cd "$(dirname "$0")"
ORGAN="$PWD"

: "${CMAKE_BUILD_PARALLEL_LEVEL:=3}"
export CMAKE_BUILD_PARALLEL_LEVEL
JOBS="$CMAKE_BUILD_PARALLEL_LEVEL"

HANDSHAKE_LIB="$ORGAN/../rust/target/release/libhandshake.a"
if [ ! -f "$HANDSHAKE_LIB" ]; then
  echo "handshake static library not found: $HANDSHAKE_LIB"
  echo "build it first:  cd ../rust && cargo build --release --features model,metal --lib --bin handshake"
  exit 1
fi
HANDSHAKE_LIB="$(cd "$(dirname "$HANDSHAKE_LIB")" && pwd)/$(basename "$HANDSHAKE_LIB")"

# LLAMA_CPP_COMMIT is UTF-8 with a BOM, and the BOM travels into the ref if it is not
# stripped: run 3 died on "couldn't find remote ref <BOM>89fe242...". sed removes the BOM
# as a byte sequence; tr -d with an octal escape did not.
COMMIT="$(sed -e '1s/^\xEF\xBB\xBF//' LLAMA_CPP_COMMIT | tr -cd '0-9a-fA-F')"
echo "=== llama.cpp pinned at $COMMIT ==="

# The patch was cut against exactly this commit. A moving checkout is a broken patch, so the
# clone is pinned and never tracks upstream.
if [ ! -d llama-src/.git ]; then
  git clone --filter=blob:none https://github.com/ggml-org/llama.cpp.git llama-src
fi
cd llama-src
git fetch --depth 1 origin "$COMMIT"
git checkout -q "$COMMIT"

# Idempotent: --check first so re-running on a machine that already patched is not an error.
if git apply --check "$ORGAN/sarge-organ.patch" 2>/dev/null; then
  git apply "$ORGAN/sarge-organ.patch"
  echo "patch applied"
else
  echo "patch already applied (or does not apply) - continuing"
fi
cp "$ORGAN/handshake.h" tools/server/

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
cmake --build build --config Release --target llama-server --parallel "$JOBS"
echo "=== DONE $(date +%T) ==="
ls -la build/bin/llama-server
