//! KV cache POC — pay for the rules once, not every turn.
//!
//! The rules that pass VET are the same on every call. They do not change between
//! tasks, and yet today they are re-tokenised and re-attended on every single turn,
//! because a system message is just text at the front of a prompt.
//!
//! A KV cache is the model's attention state for tokens it has already read. Causal
//! attention means token 7's keys and values can never change once computed - so they
//! can be computed once and kept. llama.cpp can write that state to a file and load it
//! back into a fresh context.
//!
//! So: prefill the VETTED rules once, save the state, and on every later run load it
//! instead of re-reading them. The rules stop being something the model reads and
//! become something it already knows.
//!
//!   kv --build                    prefill the vetted rules, save the state
//!   kv --task "..."               load that state, then answer
//!   kv --task "..." --cold        same task with no state, for the comparison
//!
//! What this measures is PREFILL. Generation speed is llama.cpp's and is untouched.

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Instant;

use handshake::vetted;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

const MODELS_DIR: &str = r"C:\llama-b9213\models";
const DEFAULT_MODEL: &str = r"C:\llama-b9213\models\Qwen3-8B-Q4_K_M.gguf";
const N_CTX: u32 = 4096;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut task: Option<String> = None;
    let mut build = false;
    let mut cold = false;
    let mut budget: i32 = 200;
    let mut model_path = PathBuf::from(DEFAULT_MODEL);
    let mut vetted_path = PathBuf::from("../harness/VETTED.md");
    let mut state_path = PathBuf::from("../harness/vetted.kv");

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--build" => { build = true; i += 1; }
            "--cold" => { cold = true; i += 1; }
            "--task" => { task = args.get(i + 1).cloned(); i += 2; }
            "--n" => { budget = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(budget); i += 2; }
            "--vetted" => { vetted_path = args.get(i + 1).map(PathBuf::from).unwrap_or(vetted_path); i += 2; }
            "--state" => { state_path = args.get(i + 1).map(PathBuf::from).unwrap_or(state_path); i += 2; }
            "--model" => {
                model_path = args.get(i + 1).map(|s| {
                    let p = PathBuf::from(s);
                    if p.is_absolute() { p } else { PathBuf::from(MODELS_DIR).join(s) }
                }).unwrap_or(model_path);
                i += 2;
            }
            other => { eprintln!("unknown argument: {other}"); std::process::exit(2); }
        }
    }

    if !build && task.is_none() {
        eprintln!("usage: kv --build | kv --task \"...\" [--cold] [--n 200]");
        std::process::exit(2);
    }

    let v = vetted::load(&vetted_path)?;
    let injected = vetted::render_injected(&v);
    if injected.is_empty() {
        eprintln!("nothing to inject - {} has no INJECT rules", vetted_path.display());
        std::process::exit(2);
    }

    let mut backend = LlamaBackend::init()?;
    backend.void_logs();
    let model_params = LlamaModelParams::default().with_n_gpu_layers(99);
    let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)?;
    let template = model.chat_template(None)?;

    // The prefix must be byte-identical between build and load, or the saved state
    // describes tokens the context never saw. Building it the same way both times is
    // the whole contract.
    let prefix_chat = vec![LlamaChatMessage::new("system".into(), injected.clone())?];
    let prefix_text = model.apply_chat_template(&template, &prefix_chat, false)?;
    let prefix_tokens = model.str_to_token(&prefix_text, AddBos::Always)?;

    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(Some(NonZeroU32::new(N_CTX).unwrap()));

    // ── BUILD ────────────────────────────────────────────────────────────────
    if build {
        let mut ctx = model.new_context(&backend, ctx_params)?;
        let mut batch = LlamaBatch::new(prefix_tokens.len().max(1), 1);
        for (pos, tok) in prefix_tokens.iter().enumerate() {
            batch.add(*tok, pos as i32, &[0], pos == prefix_tokens.len() - 1)?;
        }
        let t = Instant::now();
        ctx.decode(&mut batch)?;
        let ms = t.elapsed().as_millis();

        ctx.state_save_file(&state_path, &prefix_tokens)?;
        let bytes = std::fs::metadata(&state_path).map(|m| m.len()).unwrap_or(0);
        println!("built      {} rule(s), {} tokens, prefilled in {ms} ms",
                 v.inject().count(), prefix_tokens.len());
        println!("saved      {}  ({:.1} MB)", state_path.display(), bytes as f64 / 1_048_576.0);
        println!("\nThat prefill is now paid. Every later run loads it instead.");
        return Ok(());
    }

    // ── ANSWER ───────────────────────────────────────────────────────────────
    let task = task.unwrap();
    let mut ctx = model.new_context(&backend, ctx_params)?;

    let t_total = Instant::now();
    let mut loaded = 0usize;
    let mut load_ms = 0u128;

    if !cold {
        let t = Instant::now();
        let toks = ctx.state_load_file(&state_path, N_CTX as usize)?;
        load_ms = t.elapsed().as_millis();
        loaded = toks.len();
        if toks != prefix_tokens {
            eprintln!("warning    the saved state does not match the current VETTED.md");
            eprintln!("           re-run --build; the rules have changed since it was saved");
        }
    }

    // The user turn, appended after the prefix. Cold runs pay for the prefix here;
    // warm runs already have it in the cache and only pay for these tokens.
    let full_chat = vec![
        LlamaChatMessage::new("system".into(), injected)?,
        LlamaChatMessage::new("user".into(), format!("{task} /no_think"))?,
    ];
    let full_text = model.apply_chat_template(&template, &full_chat, true)?;
    let full_tokens = model.str_to_token(&full_text, AddBos::Always)?;

    let start = if cold { 0 } else { loaded.min(full_tokens.len().saturating_sub(1)) };
    let todo = &full_tokens[start..];

    let mut batch = LlamaBatch::new(todo.len().max(1), 1);
    for (n, tok) in todo.iter().enumerate() {
        batch.add(*tok, (start + n) as i32, &[0], n == todo.len() - 1)?;
    }
    let t = Instant::now();
    ctx.decode(&mut batch)?;
    let prefill_ms = t.elapsed().as_millis();

    eprintln!("{:<10} {} of {} tokens prefilled in {prefill_ms} ms{}",
              if cold { "COLD" } else { "WARM" },
              todo.len(), full_tokens.len(),
              if cold { String::new() } else { format!("  (state loaded in {load_ms} ms)") });

    let mut sampler = LlamaSampler::chain_simple([LlamaSampler::greedy()]);
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut n = 0;
    let mut pos = full_tokens.len() as i32;

    println!("\n────────────────────────────────────────────────────────────");
    while n < budget {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);
        if model.is_eog_token(token) {
            break;
        }
        print!("{}", model.token_to_piece(token, &mut decoder, false, None)?);
        use std::io::Write;
        std::io::stdout().flush().ok();
        batch.clear();
        batch.add(token, pos, &[0], true)?;
        ctx.decode(&mut batch)?;
        pos += 1;
        n += 1;
    }
    println!("\n────────────────────────────────────────────────────────────");

    eprintln!("\n{:<10} prefill {prefill_ms} ms, {n} tokens generated, {} ms total",
              if cold { "COLD" } else { "WARM" }, t_total.elapsed().as_millis());
    Ok(())
}
