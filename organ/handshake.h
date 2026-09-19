// CompilerGPT handshake - the C face of rust/src/ffi.rs.
//
// llama.cpp and the handshake are one organ. The Rust is compiled in as a static
// library, and the core - VETTED.md - is compiled into the Rust. No path, no file, no
// flag: the rules are in the binary. Agents reach them through the normal OpenAI
// endpoint and never know the difference.
//
// Every char* returned is owned by Rust. Release it with handshake_free, never free()
// or delete[] - different allocators across the boundary.
#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// The INJECT rules of the compiled-in core, rendered as fact about the codebase.
// Empty string if the build carried no core.
char *   handshake_core(void);

// How many INJECT rules the core holds.
int32_t  handshake_core_count(void);

// Content hash of the core. Same digest, same law.
uint64_t handshake_core_digest(void);

// Release a string returned by this library.
void     handshake_free(char * s);

#ifdef __cplusplus
}
#endif
