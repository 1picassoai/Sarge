# The organ — llama.cpp with Sarge compiled in

The organ is a stock `llama-server` plus one small patch: the Sarge handshake is linked in
as a static library, and the compiled-in core (the vetted universal laws) is put at the root
of every prompt inside the server. Agents talk to it on the normal OpenAI-shaped endpoint
and never see the difference. Everything in the organ runs on your GPU; nothing in it
calls out. (The tutor, elsewhere in Sarge, is the one thing that does - the README says
exactly what it is handed.)

You build it once. It takes a while the first time (llama.cpp's CUDA kernels compile from
source); after that it is a binary you start with one command.

## What you need

- An NVIDIA card with 8 GB (tested floor: an RTX 4070 laptop card).
- CUDA toolkit 13.4 or later, Visual Studio 2026 Build Tools with the C++ workload, CMake.
- Rust (stable, MSVC toolchain).
- One model file: `Qwen3-4B-Instruct-2507-Q4_K_M.gguf` (Hugging Face, ~2.5 GB). Any
  llama.cpp-served model works in principle; this is the one every result here was
  measured on.

## Build

```bat
:: 1. the handshake, as a static library (no CUDA or LLVM needed for this step)
cd rust
cargo build --release
::    -> rust\target\release\handshake.lib  and  handshake.exe

:: 2. llama.cpp at the commit this patch was cut against: 89fe242 (the full hash is in organ\LLAMA_CPP_COMMIT)
cd ..\organ
git clone https://github.com/ggml-org/llama.cpp.git llama-src
cd llama-src
git checkout 89fe242
git apply ..\sarge-organ.patch
copy ..\handshake.h tools\server\
copy ..\build-organ.cmd .
copy ..\organ.cmd .

:: 3. build llama-server with the handshake linked in
build-organ.cmd
::    reads HANDSHAKE_LIB from ..\..\rust\target\release\handshake.lib
```

`build-organ.cmd` pins four cores (`CMAKE_BUILD_PARALLEL_LEVEL=4`) and one GPU
architecture (`CUDAARCHS=89`, the 4070). Change the architecture to your card's; leave the
cores unless you enjoy a frozen laptop.

## Start

```bat
organ.cmd
```

Starts `llama-server` on `http://127.0.0.1:8421` with one slot, a 32k context and the KV
cache in q8 so it fits beside the model on 8 GB. Edit the model path at the top of the
file. The server log says, at boot, how many rules the core holds and how many tokens
they cost — that line is the proof the handshake is in.

## What the patch does (71 lines, read it)

- `tools/server/CMakeLists.txt`: link `HANDSHAKE_LIB` if set; otherwise build stock and say so.
- `tools/server/server-context.cpp`: tokenize the core once at boot; prepend it to every
  text prompt after BOS. Byte-identical prefix on every request, so the slot's own prompt
  cache carries it after the first decode.
- `tools/server/handshake.h`: the four C functions the Rust library exports.

The patch is deliberately small so it can follow llama.cpp forward. It was cut against the
commit in `LLAMA_CPP_COMMIT`; later commits may need a trivial re-apply.

## A second model

`organ-deepseek.cmd` is not shipped, but the recipe is one line: the same flags with
`-m <your gguf>` and, for a model whose chat template llama.cpp does not know,
`--jinja --chat-template-file <template>`. `deepseek-coder.jinja` is the one we used for
`deepseek-coder-6.7b-instruct`. Note what the docs say about it: the check is only as good
as the loaded model's ability to judge a line, and that model scored 0 of 7 on the replay.
