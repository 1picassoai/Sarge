# Loading a local model from Rust, in-process

**Sat 13 September 2026.** A Rust program loaded Qwen3-8B and read the model's own
parameters out of it, with no server, no HTTP, and no subprocess.

This is a small result and it is worth being precise about why it matters. Everything
in this project up to now spoke to `llama.cpp` over its HTTP server: the handshake
selected rules in one process, serialised them, and posted them to another. That
boundary is the reason the rules could only ever be *text at the front of a prompt*.
Once the model is loaded inside the same program that holds the rulebook, the rules and
the context live in one address space, and what can be done with them stops being
limited by what fits in a POST body.

Nothing here generates a token. It loads a model, reports what the model says about
itself, and creates a context. That is the whole claim.

---

## What ran

```rust
use std::num::NonZeroU32;
use std::path::PathBuf;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model_path = PathBuf::from(r"C:\llama-b9213\models\Qwen3-8B-Q4_K_M.gguf");

    let backend = LlamaBackend::init()?;
    println!("backend initialised");

    let model_params = LlamaModelParams::default().with_n_gpu_layers(0);
    let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)?;
    println!("model loaded: {}", model_path.display());

    println!("n_vocab      = {}", model.n_vocab());
    println!("n_ctx_train  = {}", model.n_ctx_train());
    println!("n_embd       = {}", model.n_embd());

    let ctx_params =
        LlamaContextParams::default().with_n_ctx(Some(NonZeroU32::new(512).unwrap()));
    let ctx = model.new_context(&backend, ctx_params)?;
    println!("context created, n_ctx = {}", ctx.n_ctx());

    Ok(())
}
```

Twenty-eight lines, and one dependency:

```toml
[dependencies]
llama-cpp-2 = "0.1.156"
```

The load reports what the GGUF says about itself. No timing claim is made from it.

---

## What it printed

```
backend initialised
model loaded: C:\llama-b9213\models\Qwen3-8B-Q4_K_M.gguf
n_vocab      = 151936
n_ctx_train  = 40960
n_embd       = 4096
context created, n_ctx = 512
```

Those four numbers came out of the GGUF file, not out of a config we wrote. `llama.cpp`
printed its own view of the same model in the same run, and it agrees:

```
print_info: file format = GGUF V3 (latest)
print_info: file type   = Q4_K - Medium
print_info: file size   = 4.68 GiB (4.90 BPW)
print_info: arch                  = qwen3
print_info: n_ctx_train           = 40960
print_info: n_embd                = 4096
print_info: n_layer               = 36
print_info: n_head                = 32
print_info: n_head_kv             = 8
print_info: model params          = 8.19 B
print_info: n_vocab               = 151936
```

`n_head_kv = 8` against `n_head = 32` is grouped-query attention with a factor of four —
which is why the KV cache for this model is a quarter the size a reader might estimate
from the head count alone.

"Q4_K_M" is a mix, not a uniform precision, and the loader says so — of 399 tensors,
217 are `q4_K`, 37 are `q6_K`, and 145 remain `f32`. The parts of the network most
sensitive to rounding keep more bits. That is what gets a 8.19-billion-parameter model
into 4.68 GiB at an average 4.90 bits per weight without it falling over.

Tensors were loaded with `load_mode = mmap`, so the 4.68 GiB is mapped from disk rather
than copied into the process up front, and every one of the 36 layers was assigned to
the CPU as instructed.

The context was asked for at 512 tokens, well under the 40,960 the model was trained
for. `llama.cpp` notes the mismatch, correctly; it was a deliberate choice, because
nothing here generates and a full-size context would only have cost memory to prove
the same point.

---

## Reproducing it

| | |
|---|---|
| Model | `Qwen3-8B-Q4_K_M.gguf`, 5,027,783,488 bytes |
| SHA-256 | `d98cdcbd03e17ce47681435b5150e34c1417f50b5c0019dd560e4882c5745785` |
| Crate | `llama-cpp-2` 0.1.156 (with `llama-cpp-sys-2` 0.1.156, `bindgen` 0.72.1) |
| Rust | `rustc` 1.93.0 (254b59607 2026-01-19), `cargo` 1.93.0 |
| Toolchain | MSVC, `x86_64-pc-windows-msvc` |
| LLVM | 22.1.8, `x86_64-pc-windows-msvc` |
| CPU | Intel Core i9-14900HX, 24 physical / 32 logical cores |
| RAM | 31.6 GB |
| OS | Windows 11 Home 10.0.26200 |

The crate's CUDA feature is enabled, so llama.cpp's kernels are compiled from source and
the CUDA toolkit is a build-time requirement.

---

## The one thing that actually blocked this

An earlier attempt at exactly this code failed, and it failed in a way worth writing
down because the error does not name its own cause clearly:

```
Unable to find libclang
```

`llama-cpp-sys-2` generates its Rust bindings from llama.cpp's C headers using
`bindgen`, and `bindgen` needs `libclang` at *build* time. It is not a Rust dependency,
so `cargo` cannot fetch it — it is a system install of LLVM. On Windows that means
installing LLVM and pointing the build at it:

```bat
set "LIBCLANG_PATH=C:\Program Files\LLVM\bin"
```

One detail that matters and is easy to get wrong: the LLVM build must target the same
ABI as the Rust toolchain. Both here are `x86_64-pc-windows-msvc`. An MSVC Rust
toolchain with a MinGW LLVM will fail later and less legibly.

With LLVM installed, the build was clean:

```
=== BUILD START  8:41:48.98 ===
   Compiling bindgen v0.72.1
   Compiling llama-cpp-sys-2 v0.1.156
   Compiling llama-cpp-2 v0.1.156
   Compiling llamarust v0.1.0 (C:\tmp\llamarust)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 53s
=== BUILD EXIT 0 AT  8:43:43.04 ===
```

**That 1 m 53 s figure should not be quoted as a cold build time.** The previous attempt
died at the bindgen stage, *after* CMake had already compiled llama.cpp's C++ into
`target/`. Those artefacts were still cached, so this build skipped the long leg. A
genuine first build on a clean machine compiles llama.cpp from source and takes
considerably longer. We have not measured that here, so we are not going to state it.

The run itself, including loading 4.68 GiB off disk, took under ten seconds
(08:44:01 → 08:44:11), exit 0.

---

## What this does and does not prove

First, what is *not* in question. The handshake itself works and has done all week: rule
selection is five green tests in Rust, and on a five-lane agent workflow it measurably
changed what the models wrote — a research lane went from an offer to quantify to
`1,154 posts · 66% · 63%`, and a PR lane dropped a promise it had no evidence for.
Generation already happens today, over HTTP, with the handshake feeding it.

What this page is about is the *plumbing underneath that*.

**It proves** the binding is real on this platform, that the toolchain problem is solved
and understood, and that a Rust program can hold a local model directly.

**It does not prove** anything about generation, throughput, or quality *through this
route*. The 28-line program above deliberately loads and stops — it produces no token
because producing one was not its job. It is a scratch crate at `C:\tmp\llamarust`, not
yet joined to the handshake library.

So there are two working pieces here, not one working and one broken: rule selection that
changes output, and an in-process model loader. They are not yet in the same binary.

Merging them buys speed and reach, not correctness — no process boundary per call, and
Rust gets its hands on the context rather than only on the prompt string. The sequence
`select → render → context → generate` then runs without crossing a boundary. That is
where the interesting claims start, and none of them are made yet.
