//! The circle closed: rules and the model in one process.
//!
//! Until now the handshake selected rules here and posted them to a llama.cpp server
//! over HTTP. Two processes, and the only thing that could cross between them was a
//! string. This binary loads the model in-process, so selection and generation share
//! an address space.
//!
//! What it does, in order:
//!
//!   SELECT    the rules this task touches (the library, unchanged)
//!   RENDER    them into the block, allowance travelling with its prohibition
//!   TEMPLATE  the model's OWN chat template, read out of the GGUF - an instruct
//!             model fed a raw string produces rubbish, and that would look like the
//!             handshake failing when it is the prompt that is wrong
//!   DECODE    prefill, then sample a token at a time until EOG or the budget
//!
//! The rules go in the system message. That is the honest first step - it proves the
//! circle closes without a process boundary. Writing them straight into the KV cache
//! is the next thing, and it is not claimed here.
//!
//!   compile --task "..." [--lane pr] [--repo X] [--model path.gguf] [--n 200] [--raw]

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Instant;

use handshake::{for_scope, load, render, select, MAX_RULES};

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

// The context size a KV state is built at. A state file carries its context size, and
// loading it into a smaller context fails - so build and inject must agree. Must match
// N_CTX in kv.rs.
const KV_CTX: u32 = 4096;

const MODELS_DIR: &str = r"C:\llama-b9213\models";
const DEFAULT_MODEL: &str = r"C:\llama-b9213\models\Qwen3-8B-Q4_K_M.gguf";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut task: Option<String> = None;
    let mut lane: Option<String> = None;
    let mut scope: Option<String> = None;
    let mut rules_path = PathBuf::from("../harness/rules.jsonl");
    let mut model_path = PathBuf::from(DEFAULT_MODEL);
    let mut budget: i32 = 200;
    let mut raw = false;
    let mut no_think = false;
    let mut all = false;
    let mut vetted_path = PathBuf::from("../harness/VETTED.md");
    // CORE: who the model is and what its job is, plus the laws that hold in every
    // codebase. Injected first, once, and never changed by a rule update - rules are
    // costumes, the core is the character. Vinn's ruling, 14 Sep 2026.
    let mut core_path = PathBuf::from("../harness/CHARACTER.md");
    // --inject loads the vetted rules from a saved KV state instead of re-sending them
    // as a system message. The whole point of the project: a rule the model READS can
    // be weighed against its own prior and lose - measured, `minimal-api` was stated in
    // the prompt and it wrote controllers anyway. A rule already in the attention state
    // was never up for discussion.
    let mut inject: Option<PathBuf> = None;
    let mut run_loop = false;
    let mut sandbox: Option<PathBuf> = None;
    let mut ledger: Option<PathBuf> = None;
    let mut json_out = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--task" => { task = args.get(i + 1).cloned(); i += 2; }
            "--lane" => { lane = args.get(i + 1).cloned(); i += 2; }
            "--repo" => { scope = args.get(i + 1).cloned(); i += 2; }
            "--rules" => { rules_path = args.get(i + 1).map(PathBuf::from).unwrap_or(rules_path); i += 2; }
            // A bare name resolves against the models directory, so a sweep across
            // models reads `--model Qwen3-0.6B-Q4_K_M.gguf` rather than a full path.
            "--model" => {
                model_path = args.get(i + 1).map(|s| {
                    let p = PathBuf::from(s);
                    if p.is_absolute() { p } else { PathBuf::from(MODELS_DIR).join(s) }
                }).unwrap_or(model_path);
                i += 2;
            }
            "--n" => { budget = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(budget); i += 2; }
            // The control arm: same model, same task, no rules. Without it a claim that
            // the rules changed the output is an assertion, not a measurement.
            "--raw" => { raw = true; i += 1; }
            // Family-agnostic: closes the think block rather than sending a Qwen-only
            // `/no_think` suffix. See where the prompt is built.
            "--no-think" => { no_think = true; i += 1; }
            "--all" => { all = true; i += 1; }
            "--vetted" => { vetted_path = args.get(i + 1).map(PathBuf::from).unwrap_or(vetted_path); i += 2; }
            "--core" => { core_path = args.get(i + 1).map(PathBuf::from).unwrap_or(core_path); i += 2; }
            "--inject" => {
                inject = Some(args.get(i + 1).map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("../harness/vetted.kv")));
                i += if args.get(i + 1).map_or(false, |s| !s.starts_with("--")) { 2 } else { 1 };
            }
            // THE LOOP, in this process. Write the files the model named, build them,
            // and on a red build call the tutor once and put its rule in the book.
            // Vinn's ruling, 14 Sep: no logic outside this binary.
            "--loop" => { run_loop = true; i += 1; }
            "--sandbox" => { sandbox = args.get(i + 1).map(PathBuf::from); i += 2; }
            "--ledger" => { ledger = args.get(i + 1).map(PathBuf::from); i += 2; }
            // Machine-readable result on stdout, so a page can render it without
            // parsing our status lines.
            "--json" => { json_out = true; i += 1; }
            // 99 means "all of them" - llama.cpp clamps to the layer count.
            other => { eprintln!("unknown argument: {other}"); std::process::exit(2); }
        }
    }

    let Some(task) = task else {
        eprintln!("usage: compile --task \"...\" [--lane pr] [--repo X] [--n 200] [--raw] [--no-think] [--all] [--vetted PATH] [--core PATH]\n\
                   \x20      [--loop] [--sandbox DIR] [--ledger PATH] [--json]");
        std::process::exit(2);
    };

    // ── SELECT and RENDER ────────────────────────────────────────────────────
    // Same library the CLI and the C# agents call. No second implementation.
    let t_select = Instant::now();
    let (rules, problems) = load(&rules_path)?;
    for p in &problems {
        eprintln!("skipped a malformed rule, {p}");
    }
    let scoped = for_scope(&rules, scope.as_deref());
    // --all renders every rule in the file, bypassing BM25. Without it a benchmark that
    // varies the rule COUNT varies only the pool: rules whose topic words miss the task
    // score zero and are dropped before the model sees them. Measured - a 12-rule file
    // and a 3-rule file both rendered 2 rules, and the "collapse curve" was an artefact.
    let chosen = if raw {
        Vec::new()
    } else if all {
        scoped.clone()
    } else {
        select(&scoped, &task, lane.as_deref(), MAX_RULES)
    };
    let block = render(&chosen);
    let select_us = t_select.elapsed().as_micros();

    eprintln!("handshake  {} of {} rule(s) selected in {select_us} us{}",
              chosen.len(), scoped.len(), if raw { "  [RAW - rules withheld]" } else { "" });
    for r in &chosen {
        eprintln!("           - {}", r.id);
    }

    // ── the model, in this process ───────────────────────────────────────────
    let mut backend = LlamaBackend::init()?;
    // llama.cpp logs to stdout by default - loader chatter, and on CUDA a line per
    // graph reuse, interleaved with the model's own tokens. The benchmarks parse
    // that stream, so the logging goes to nowhere. Our own status lines are on
    // stderr and unaffected.
    backend.void_logs();
    // Every layer on the GPU, always. CPU inference for an 8B is not a deployment and
    // was never a baseline worth keeping - it only made the code branch.
    let model_params = LlamaModelParams::default().with_n_gpu_layers(99);
    let t_load = Instant::now();
    let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)?;
    eprintln!("model      {} in {:.1}s",
              model_path.file_name().unwrap_or_default().to_string_lossy(),
              t_load.elapsed().as_secs_f32());

    // ── TEMPLATE ─────────────────────────────────────────────────────────────
    // Read out of the GGUF, not written by us. Qwen3 has its own and using the wrong
    // one gives "really unexpected responses" - the crate's own warning, and true.
    let template = model.chat_template(None)?;
    let mut chat = Vec::new();

    // ── CORE ─────────────────────────────────────────────────────────────────
    // The first message, always, unless RAW. Character and job; the model boots as
    // this and every costume below is worn on top of it. When a KV state is injected
    // the core has to be part of that saved prefix too, or the prefix check below
    // reports STALE and falls back to a full prefill - honest, but paid for.
    let core_text = std::fs::read_to_string(&core_path).unwrap_or_default();
    if !core_text.trim().is_empty() && !raw {
        chat.push(LlamaChatMessage::new("system".into(), core_text.trim().to_string())?);
        eprintln!("core       {} ({} chars)",
                  core_path.file_name().unwrap_or_default().to_string_lossy(), core_text.len());
    } else if raw {
        eprintln!("core       withheld [RAW]");
    } else {
        eprintln!("core       none at {}", core_path.display());
    }

    // ── VETTED ───────────────────────────────────────────────────────────────
    // Rules that earned it go in first, stated as fact about the codebase rather than
    // as instruction. This is the system-message stand-in for KV-cache injection: the
    // position is what a cache write would take, so what lands here is what will
    // eventually be built in rather than read. Nothing below INJECT gets in.
    let vetted = handshake::vetted::load(&vetted_path).ok();
    let mut injected_text = String::new();
    if let Some(v) = &vetted {
        injected_text = handshake::vetted::render_injected(v);
        eprintln!("vetted     {} inject, {} ground, from {}",
                  v.inject().count(), v.ground().count(),
                  vetted_path.file_name().unwrap_or_default().to_string_lossy());
        if !injected_text.is_empty() && !raw {
            // Always the first message, injected or not. When --inject is on, these
            // exact tokens are the ones already in the saved state, so the prefix must
            // be built identically here or the cache describes tokens this context
            // never saw.
            chat.push(LlamaChatMessage::new("system".into(), injected_text.clone())?);
        }
    }

    if !block.is_empty() {
        chat.push(LlamaChatMessage::new("system".into(), block.clone())?);
    }
    // LOOP: the model is told what already exists and how to name a file. The file list
    // is not coaching - a developer on task 8 can see what task 1 left behind, and
    // withholding it would measure amnesia rather than drift.
    // Default box: the shopfloor book beside this binary's repo - rust/target/release/
    // compile.exe -> ../../../book/shopfloor. No absolute path, so the repo can be
    // renamed or moved without touching this.
    let box_dir = sandbox.clone().unwrap_or_else(|| {
        std::env::current_exe().ok()
            .and_then(|e| e.ancestors().nth(4).map(|r| r.join("book").join("shopfloor")))
            .unwrap_or_else(|| PathBuf::from("book/shopfloor"))
    });
    let sent = if run_loop {
        let tree = handshake::sandbox::tree(&box_dir);
        let listing = if tree.is_empty() {
            "  (empty - this is the first task)".to_string()
        } else {
            tree.iter().map(|f| format!("  {}", f.path)).collect::<Vec<_>>().join("\n")
        };
        format!("{task}\n\nFiles that already exist in the project:\n{listing}\n\n\
                 Write each file as a fenced code block with the path on the line above it, \
                 exactly like this:\n\n// File: Models/Product.cs\n```csharp\n...\n```")
    } else {
        task.clone()
    };
    chat.push(LlamaChatMessage::new("user".into(), sent)?);
    let mut prompt = model.apply_chat_template(&template, &chat, true)?;

    // Disable reasoning by closing the think block before the model can open it.
    //
    // The first version appended `/no_think` to the user turn. That is a QWEN
    // convention and did nothing for anything else - measured: MiniCPM5-2B reasoned
    // straight through a 900-token budget and wrote no code in five of six tasks,
    // which made it useless as a control and nearly produced a finding about a model
    // that had never been asked the question properly.
    //
    // Both families gate it on an `enable_thinking` template variable, and this crate
    // cannot pass template variables. But both emit the SAME thing when it is false:
    // an already-closed think block after the assistant header. Checked in the GGUF
    // templates of both models rather than assumed. So append it directly - it works
    // for any model following the ChatML thinking convention, not just the two here.
    if no_think {
        prompt.push_str("<think>\n\n</think>\n\n");
    }

    let tokens = model.str_to_token(&prompt, AddBos::Never)?;
    eprintln!("prompt     {} tokens ({} of them rules)",
              tokens.len(),
              if block.is_empty() { 0 } else { model.str_to_token(&block, AddBos::Never)?.len() });

    // The context has to be big enough for the whole cached prefix, not just this
    // prompt - a state file holds the context size it was built at, and loading into a
    // smaller one fails. So when injecting, size to the state's context.
    let n_ctx = if inject.is_some() {
        KV_CTX
    } else {
        (tokens.len() as u32 + budget as u32 + 64).max(512)
    };
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(Some(NonZeroU32::new(n_ctx).unwrap()));
    let mut ctx = model.new_context(&backend, ctx_params)?;

    // ── INJECT ───────────────────────────────────────────────────────────────
    // Load the rules' attention state straight into the context. They are not read
    // from a prompt - they are already there, computed, before the task arrives.
    //
    // `start` is how much of this prompt the cache already covers. Everything before
    // it is skipped: not re-tokenised, not re-attended, not paid for.
    let mut start = 0usize;
    if let Some(state_path) = &inject {
        if !state_path.exists() {
            eprintln!("inject     no state at {} - build it with kv.cmd --build",
                      state_path.display());
            std::process::exit(2);
        }
        let t = Instant::now();
        let cached = ctx.state_load_file(state_path, KV_CTX as usize)?;
        let load_ms = t.elapsed().as_millis();

        // The cached tokens must be a genuine prefix of this prompt. If VETTED.md
        // changed since the state was built they will not be, and silently serving a
        // stale cache means the model is grounded in rules that no longer exist - the
        // exact pollution VET exists to prevent, arriving by the back door.
        let is_prefix = cached.len() <= tokens.len()
            && cached.iter().zip(tokens.iter()).all(|(a, b)| a == b);
        if is_prefix {
            start = cached.len();
            eprintln!("inject     {} tokens from cache in {load_ms} ms, {} left to prefill",
                      cached.len(), tokens.len() - start);
        } else {
            eprintln!("inject     STALE - the state does not prefix this prompt.");
            eprintln!("           VETTED.md has changed. Re-run kv.cmd --build.");
            eprintln!("           Falling back to a full prefill so the answer is honest.");
            ctx = model.new_context(&backend, LlamaContextParams::default()
                .with_n_ctx(Some(NonZeroU32::new(n_ctx).unwrap())))?;
        }
    }

    // ── DECODE ───────────────────────────────────────────────────────────────
    let todo = &tokens[start..];
    let mut batch = LlamaBatch::new(todo.len().max(1), 1);
    let last = todo.len() - 1;
    for (n, tok) in todo.iter().enumerate() {
        // Only the final token needs logits - it is the one we sample from.
        batch.add(*tok, (start + n) as i32, &[0], n == last)?;
    }
    let t_prefill = Instant::now();
    ctx.decode(&mut batch)?;
    let prefill_ms = t_prefill.elapsed().as_millis();

    // Greedy, deliberately. A temperature would make two runs differ for reasons that
    // have nothing to do with the rules, and the comparison is the whole point.
    let mut sampler = LlamaSampler::chain_simple([LlamaSampler::greedy()]);

    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut out = String::new();
    let mut n = 0;
    let mut pos = tokens.len() as i32;
    let t_gen = Instant::now();

    println!("\n────────────────────────────────────────────────────────────");
    while n < budget {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);
        if model.is_eog_token(token) {
            break;
        }
        let piece = model.token_to_piece(token, &mut decoder, false, None)?;
        print!("{piece}");
        use std::io::Write;
        std::io::stdout().flush().ok();
        out.push_str(&piece);

        batch.clear();
        batch.add(token, pos, &[0], true)?;
        ctx.decode(&mut batch)?;
        pos += 1;
        n += 1;
    }
    println!("\n────────────────────────────────────────────────────────────");

    let gen_s = t_gen.elapsed().as_secs_f32();
    eprintln!("\ngenerated  {n} tokens in {gen_s:.1}s ({:.1} tok/s), prefill {prefill_ms} ms",
              n as f32 / gen_s.max(0.001));
    eprintln!("in-process no HTTP, no server, no subprocess");

    // ── THE LOOP ─────────────────────────────────────────────────────────────
    // Everything from here is what used to live in Python. Write, judge, teach, learn -
    // one process, one binary. The only thing that leaves the box is the tutor call,
    // and only when the build is red.
    if !run_loop {
        return Ok(());
    }

    let written = handshake::sandbox::write_files(&box_dir, &out);
    for w in &written {
        match (w.ok, &w.why) {
            (true, _) => eprintln!("wrote      {} ({} bytes, {})", w.path,
                                   w.bytes.unwrap_or(0), w.action.unwrap_or("")),
            (false, Some(why)) => eprintln!("refused    {} - {why}", w.path),
            (false, None) => eprintln!("refused    {}", w.path),
        }
    }

    let mut build_result = None;
    let mut lesson: Option<handshake::tutor::Lesson> = None;
    let mut lesson_error: Option<String> = None;

    if written.iter().any(|w| w.ok) {
        let b = handshake::judge::build(&box_dir, std::time::Duration::from_secs(300));
        eprintln!("build      {} in {:.1}s", if b.ok { "GREEN" } else { "RED" }, b.wall_s);
        // Green: the frontier is never called. That is the saving, and it is the point.
        if !b.ok && !raw {
            let shapes: Vec<String> = chosen.iter().map(|r| r.shape.clone()).collect();
            let files: Vec<String> = handshake::sandbox::tree(&box_dir)
                .into_iter().map(|f| f.path).collect();
            match handshake::tutor::teach(&task, &out, &shapes,
                                          &format!("build failed:\n{}", b.errors),
                                          &files, scope.as_deref(), &rules_path) {
                Ok(l) => {
                    eprintln!("tutor      {} - {}", l.status,
                              l.rule.as_ref().map_or("no rule".into(), |r| r.id.clone()));
                    lesson = Some(l);
                }
                Err(e) => {
                    eprintln!("tutor      unreachable: {e}");
                    lesson_error = Some(e);
                }
            }
        }
        build_result = Some(b);
    }

    // The ledger: one row per run, the same shape the console has always written, so a
    // day's drift can be reconstructed from the ledger alone.
    if let Some(path) = &ledger {
        let now = std::time::SystemTime::now();
        let row = serde_json::json!({
            "day": chrono_day(now),
            "at": chrono_time(now),
            "task": task,
            "model": model_path.file_name().unwrap_or_default().to_string_lossy()
                        .replace("-Q4_K_M.gguf", ""),
            "lane": lane,
            "arm": if raw { "raw" } else { "harness" },
            "prompt_tokens": tokens.len(),
            "gen_tokens": n,
            "wall_s": (t_load.elapsed().as_secs_f32() * 10.0).round() / 10.0,
            "rules_shown": chosen.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
            "core": core_path.file_name().unwrap_or_default().to_string_lossy(),
            "written": written,
            "build": true,
            "build_ok": build_result.as_ref().map(|b| b.ok),
            "lesson": lesson.as_ref().and_then(|l| l.rule.as_ref()).map(|r| r.id.clone()),
            "lesson_status": lesson.as_ref().map(|l| l.status),
        });
        if let Some(p) = path.parent() { let _ = std::fs::create_dir_all(p); }
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            use std::io::Write;
            let _ = writeln!(f, "{row}");
        }
    }

    if json_out {
        let result = serde_json::json!({
            "text": out,
            "rules": chosen.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
            "core": core_path.file_name().unwrap_or_default().to_string_lossy(),
            "written": written,
            "tree": handshake::sandbox::tree(&box_dir),
            "sandbox": box_dir.to_string_lossy(),
            "book": rules_path.to_string_lossy(),
            "build_result": build_result,
            "lesson": lesson,
            "lesson_error": lesson_error,
            "stats": {
                "prompt_tokens": tokens.len(),
                "gen_tokens": n,
                "select_us": select_us,
                "tok_s": (n as f32 / gen_s.max(0.001) * 10.0).round() / 10.0,
                "prefill_ms": prefill_ms,
            },
        });
        println!("\n<<<JSON>>>{result}");
    }

    Ok(())
}

/// Local date, without pulling in a date crate for two strings.
fn chrono_day(t: std::time::SystemTime) -> String {
    let secs = t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = secs / 86_400;
    let (mut y, mut d) = (1970i64, days as i64);
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let len = if leap { 366 } else { 365 };
        if d < len { break }
        d -= len;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let months = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0;
    while d >= months[m] { d -= months[m]; m += 1; }
    format!("{y:04}-{:02}-{:02}", m + 1, d + 1)
}

fn chrono_time(t: std::time::SystemTime) -> String {
    let secs = t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) % 86_400;
    format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
}
