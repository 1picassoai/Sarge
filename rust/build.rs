//! Compile the core in.
//!
//! Core and costume: the core is the standing law - stable, compiled once, never
//! re-sent. The costume is the task, sent with each request. So the core is not a file
//! llama-server opens at startup; it is part of the build. `COMPILERGPT_CORE` names the
//! VETTED.md to embed (default: harness/VETTED.md beside this crate). New rules mean a
//! new build, and the tag on that build is the receipt.
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let core = env::var("COMPILERGPT_CORE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../harness/VETTED.md"));
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("core.md");

    println!("cargo:rerun-if-env-changed=COMPILERGPT_CORE");
    println!("cargo:rerun-if-changed={}", core.display());

    // A build with no core is a stock server. That is allowed - it must just say so.
    let text = fs::read_to_string(&core).unwrap_or_default();
    fs::write(&out, text).expect("write core.md into OUT_DIR");
    println!("cargo:warning=Sarge core: {}", core.display());
}
