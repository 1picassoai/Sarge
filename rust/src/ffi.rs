//! The C face of the handshake, so llama-server can compile it in.
//!
//! Vinn's ruling: the handshake and llama.cpp are one organ, and agents reach it
//! through llama-server's own OpenAI endpoint. So the Rust does not become a server
//! and it does not shim one - it becomes a static library that server-context.cpp
//! links, and calls at the root, before any slot has taken a request.
//!
//! Core and costume. The core - VETTED.md - is embedded here at `cargo build` (see
//! build.rs). There is no path to pass and no file to open: the rules are in the
//! binary. The costume is whatever the agent sends per request.
//!
//! Every returned string is owned by Rust and freed with `handshake_free`. Passing it
//! to free() or delete[] is undefined - different allocators across the boundary.

use std::ffi::CString;
use std::os::raw::c_char;
use std::path::Path;

use crate::vetted;

const CORE: &str = include_str!(concat!(env!("OUT_DIR"), "/core.md"));

fn core() -> vetted::Vetted {
    vetted::parse(CORE, Path::new("<compiled-in>"))
}

/// The block at the root of every prompt: the INJECT rules of the compiled-in core,
/// rendered as fact about the codebase. Empty if the build carried no core - the
/// server still starts, with nothing in the blood, and says so.
///
/// # Safety
/// The result must be released with `handshake_free`.
#[no_mangle]
pub unsafe extern "C" fn handshake_core() -> *mut c_char {
    CString::new(vetted::render_injected(&core()))
        .unwrap_or_default()
        .into_raw()
}

/// How many INJECT rules the compiled-in core holds.
#[no_mangle]
pub extern "C" fn handshake_core_count() -> i32 {
    core().inject().count() as i32
}

/// Content hash of the compiled-in core. Two builds with the same digest carry the
/// same law; the tag says which.
#[no_mangle]
pub extern "C" fn handshake_core_digest() -> u64 {
    core().digest
}

/// Release a string returned by this library.
///
/// # Safety
/// `s` must have come from this library and not have been freed already.
#[no_mangle]
pub unsafe extern "C" fn handshake_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
