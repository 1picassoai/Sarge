//! POINTING. Where a rule sits decides whether the model attends to it.
//!
//! The same rule, the same task, the same model, greedy - and one variable: the rule's
//! position relative to the token being written. Design in docs/KV-POINTING.md, from the
//! Captain's question after the attention slide: softmax(QK^T / sqrt(d_k)) V.
//!
//!   none   no rule at all                                  (control)
//!   head   rule as a system message at the top             (today's arm A)
//!   tail   rule as the last thing in the user turn         (lever 2, by prompt order)
//!   kv     rule prefilled as its OWN sequence, then its K and V position-shifted and
//!          copied into the main sequence right before the assistant turn  (lever 2, by cache)
//!   all    every arm above, in sequence, model loaded once - ONE JSON result
//!
//! Everything that decides anything lives here: the arms, the cache surgery, the verdict.
//! The console is a page; it shells this once and shows what comes back. The Captain's
//! ruling, 14 Sep: no logic outside the Rust binary.
//!
//!   point --task "..." --rule-id scoped-context-in-handler --repo myshop --arm all

use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Instant;

use handshake::{for_scope, load, render};

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

const MODELS_DIR: &str = r"C:\llama-b9213\models";
const DEFAULT_MODEL: &str = r"C:\llama-b9213\models\Qwen3-4B-Instruct-2507-Q4_K_M.gguf";
//   kvs    the rule rendered THROUGH THE CHAT TEMPLATE as a system turn, cached, attached
//          before the assistant header - the "framed block" hypothesis (17 Sep)
//   kvr    the rule block cached at the ROOT - position 0, before the core, the main
//          prompt shifted after it - where the compiled core already lives
const ARMS: [&str; 6] = ["none", "head", "tail", "kv", "kvs", "kvr"];
fn is_kv(arm: &str) -> bool { matches!(arm, "kv" | "kvs" | "kvr") }

struct ArmResult {
    arm: String,
    handlers: usize,
    clean: usize,
    faulty: usize,
    details: Vec<String>,
    result: String,
    held: Option<bool>,
    prompt_tokens: usize,
    rule_tokens: usize,
    gen_tokens: i32,
    prefill_ms: u128,
    gen_s: f32,
    output: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut task: Option<String> = None;
    let mut rule_id: Option<String> = None;
    let mut scope: Option<String> = None;
    let mut arm = String::from("all");
    let mut rules_path = PathBuf::from("../harness/rules.jsonl");
    let mut core_path = PathBuf::from("../harness/CHARACTER.md");
    let mut model_path = PathBuf::from(DEFAULT_MODEL);
    let mut budget: i32 = 900;
    let mut json = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--task" => { task = args.get(i + 1).cloned(); i += 2; }
            "--rule-id" => { rule_id = args.get(i + 1).cloned(); i += 2; }
            "--repo" => { scope = args.get(i + 1).cloned(); i += 2; }
            "--arm" => { arm = args.get(i + 1).cloned().unwrap_or(arm); i += 2; }
            "--rules" => { rules_path = args.get(i + 1).map(PathBuf::from).unwrap_or(rules_path); i += 2; }
            "--core" => { core_path = args.get(i + 1).map(PathBuf::from).unwrap_or(core_path); i += 2; }
            "--model" => {
                model_path = args.get(i + 1).map(|s| {
                    let p = PathBuf::from(s);
                    if p.is_absolute() { p } else { PathBuf::from(MODELS_DIR).join(s) }
                }).unwrap_or(model_path);
                i += 2;
            }
            "--n" => { budget = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(budget); i += 2; }
            "--json" => { json = true; i += 1; }
            other => { eprintln!("unknown argument: {other}"); std::process::exit(2); }
        }
    }
    let (Some(task), Some(rule_id)) = (task, rule_id) else {
        eprintln!("usage: point --task \"...\" --rule-id ID [--repo X] [--arm none|head|tail|kv|all] [--n 900] [--json]");
        std::process::exit(2);
    };
    let arms: Vec<&str> = if arm == "all" { ARMS.to_vec() }
        else if ARMS.contains(&arm.as_str()) { vec![arm.as_str()] }
        else { eprintln!("--arm must be none, head, tail, kv, kvs, kvr or all"); std::process::exit(2); };

    // ── THE ONE RULE ─────────────────────────────────────────────────────────
    let (rules, _) = load(&rules_path)?;
    let scoped = for_scope(&rules, scope.as_deref());
    let Some(rule) = scoped.iter().find(|r| r.id == rule_id) else {
        eprintln!("rule {rule_id} not found in {} for scope {:?}", rules_path.display(), scope);
        std::process::exit(2);
    };
    let rule_block = render(&[*rule]);
    let core_text = std::fs::read_to_string(&core_path).unwrap_or_default().trim().to_string();

    // ── THE MODEL, loaded once for every arm ─────────────────────────────────
    let mut backend = LlamaBackend::init()?;
    backend.void_logs();
    let model_params = LlamaModelParams::default().with_n_gpu_layers(99);
    let t_load = Instant::now();
    let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)?;
    eprintln!("model      {} in {:.1}s", model_path.file_name().unwrap_or_default().to_string_lossy(),
              t_load.elapsed().as_secs_f32());
    let template = model.chat_template(None)?;

    let mut results = Vec::new();
    for a in arms {
        eprintln!("\n===== ARM {a} =====");
        let r = run_arm(&backend, &model, &template, a, &core_text, &rule_block, &task, budget)?;
        if !json {
            println!("\n────────────────────────────────────────────────────────────");
            println!("{}", r.output);
            println!("────────────────────────────────────────────────────────────");
        }
        println!("ARM {:<5} rule {rule_id}", r.arm);
        for d in &r.details { println!("  {d}"); }
        println!("  handlers {} · clean {} · faulty {}", r.handlers, r.clean, r.faulty);
        println!("RESULT {}", r.result);
        results.push(r);
    }

    // ── ONE RESULT, for the page ─────────────────────────────────────────────
    if json {
        let items: Vec<String> = results.iter().map(|r| format!(
            "{{\"arm\":\"{}\",\"handlers\":{},\"clean\":{},\"faulty\":{},\"result\":\"{}\",\"held\":{},\
             \"details\":[{}],\"prompt_tokens\":{},\"rule_tokens\":{},\"gen_tokens\":{},\"prefill_ms\":{},\"gen_s\":{:.1}}}",
            r.arm, r.handlers, r.clean, r.faulty, r.result,
            match r.held { Some(true) => "true", Some(false) => "false", None => "null" },
            r.details.iter().map(|d| format!("\"{}\"", d.replace('"', "'"))).collect::<Vec<_>>().join(","),
            r.prompt_tokens, r.rule_tokens, r.gen_tokens, r.prefill_ms, r.gen_s)).collect();
        println!("<<<JSON>>>{{\"rule\":\"{rule_id}\",\"arms\":[{}]}}", items.join(","));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_arm(backend: &LlamaBackend, model: &LlamaModel, template: &llama_cpp_2::model::LlamaChatTemplate,
           arm: &str, core_text: &str, rule_block: &str, task: &str, budget: i32)
           -> Result<ArmResult, Box<dyn std::error::Error>> {
    // ── THE PROMPT, PER ARM ──────────────────────────────────────────────────
    // Every arm ends with the assistant header. What differs is only where the rule is.
    let sys = |s: &str| LlamaChatMessage::new("system".into(), s.to_string());
    let usr = |s: &str| LlamaChatMessage::new("user".into(), s.to_string());

    let mut main_chat: Vec<LlamaChatMessage> = Vec::new();
    if !core_text.is_empty() { main_chat.push(sys(core_text)?); }
    match arm {
        "head" => { main_chat.push(sys(rule_block)?); main_chat.push(usr(task)?); }
        "tail" => { main_chat.push(usr(&format!("{task}\n\n{rule_block}"))?); }
        _      => { main_chat.push(usr(task)?); }   // none and kv: rule not in the text
    }

    // `kv` splits the prompt: everything up to the user turn, THEN the rule as cached KV,
    // THEN the assistant header. So the main prompt is rendered WITHOUT the assistant
    // header and the header is decoded separately after the rule has been attached.
    let with_ass = model.apply_chat_template(template, &main_chat, true)?;
    let without_ass = model.apply_chat_template(template, &main_chat, false)?;
    // Templates do not always append cleanly to the non-generation render, so take
    // everything after the longest common prefix. An empty header here is what produced
    // NTokensZero on the first smoke.
    let common = with_ass.bytes().zip(without_ass.bytes()).take_while(|(a, b)| a == b).count();
    let ass_header = with_ass[common..].to_string();

    // What the cached block IS, per arm: kv = the framed rule block as bare text; kvs = the
    // same block rendered through the chat template as a system turn, so its K/V carry
    // "this is an instruction" and not "these are words"; kvr = the bare block, at the root.
    let cached_text: String = match arm {
        "kvs" => model.apply_chat_template(template, &[sys(rule_block)?], false)?,
        _ => rule_block.to_string(),
    };
    let (prompt_text, rule_tokens): (String, Vec<_>) = if is_kv(arm) {
        (without_ass, model.str_to_token(&cached_text, AddBos::Never)?)
    } else {
        (with_ass, Vec::new())
    };
    let prompt_tokens = model.str_to_token(&prompt_text, AddBos::Never)?;
    let header_tokens = if is_kv(arm) { model.str_to_token(&ass_header, AddBos::Never)? } else { Vec::new() };

    let p = prompt_tokens.len();
    let r = rule_tokens.len();
    let h = header_tokens.len();

    // Two lessons, both measured on the first smokes of this file:
    //   - by default llama.cpp DIVIDES the context between sequences, so a context sized
    //     for one sequence left the prompt no room (NoKvCacheSlot). Sized for both.
    //   - by default each sequence has its own cache STREAM, and copying a position range
    //     between streams is refused with a hard abort (exit 9, no message). Unified mode
    //     is one shared cache, where a copy just adds a sequence id to the same cells -
    //     exactly what "attach this cached block here" means.
    let seqs: usize = if is_kv(arm) { 2 } else { 1 };
    let n_ctx = ((p + r + h + budget as usize + 64) * seqs).max(512) as u32;
    eprintln!("tokens     prompt {p} · rule {r} · header {h} · ctx {n_ctx} · seqs {seqs}");
    if p == 0 || (is_kv(arm) && (r == 0 || h == 0)) {
        return Err("a batch would be empty - refusing to decode nothing".into());
    }
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(Some(NonZeroU32::new(n_ctx).unwrap()))
        .with_n_seq_max(seqs as u32)
        .with_kv_unified(true);
    let mut ctx = model.new_context(backend, ctx_params)?;

    // ── DECODE ───────────────────────────────────────────────────────────────
    let t_prefill = Instant::now();
    // kvr: the rule takes positions 0..r at the root, so the main prompt starts at r.
    let base: i32 = if arm == "kvr" { r as i32 } else { 0 };
    let mut batch = LlamaBatch::new(p.max(1), 1);
    for (n, tok) in prompt_tokens.iter().enumerate() {
        batch.add(*tok, base + n as i32, &[0], n == p - 1)?;
    }
    ctx.decode(&mut batch)?;
    let mut pos = base + p as i32;

    if arm == "kvr" {
        // THE RULE AT THE ROOT. Prefilled as its own sequence at 0..r - the same positions
        // it will occupy - so no shift is needed: copy the cells into seq 0 and the main
        // prompt, already decoded from position r onward, sits after it. The compiled core
        // lives here in the organ; this arm asks whether a rule would bind from there.
        let mut rb = LlamaBatch::new(r.max(1), 1);
        for (n, tok) in rule_tokens.iter().enumerate() {
            rb.add(*tok, n as i32, &[1], n == r - 1)?;
        }
        ctx.decode(&mut rb)?;
        ctx.kv_cache_seq_cp(1, 0, Some(0), Some(r as u32))?;
        ctx.kv_cache_seq_rm(1, None, None)?;
        eprintln!("attached   rule K/V ({r} tokens) at the root, positions 0..{r} in seq 0; prompt at {r}..{}", r + p);
        let mut hb = LlamaBatch::new(h.max(1), 1);
        for (n, tok) in header_tokens.iter().enumerate() {
            hb.add(*tok, pos + n as i32, &[0], n == h - 1)?;
        }
        ctx.decode(&mut hb)?;
        pos += h as i32;
        batch = hb;
    } else if is_kv(arm) {
        // THE RULE AS ITS OWN SEQUENCE. Prefilled at positions 0..r in sequence 1, so its
        // K and V are computed in isolation - attending only to itself, exactly as a
        // cached block would be. Then position-shifted to sit at p..p+r and copied into
        // sequence 0. No token of the rule is recomputed; only the RoPE shift is applied,
        // lazily, on the next decode - the same mechanism llama.cpp uses for context
        // shifting.
        let mut rb = LlamaBatch::new(r.max(1), 1);
        for (n, tok) in rule_tokens.iter().enumerate() {
            rb.add(*tok, n as i32, &[1], n == r - 1)?;
        }
        ctx.decode(&mut rb)?;
        // Shift FIRST, then copy. Shifting after the copy would move the prompt's own
        // cells at positions 0..r as well, since they share those positions in seq 0.
        ctx.kv_cache_seq_add(1, None, None, p as i32)?;
        ctx.kv_cache_seq_cp(1, 0, Some(p as u32), Some((p + r) as u32))?;
        ctx.kv_cache_seq_rm(1, None, None)?;
        pos = (p + r) as i32;
        eprintln!("attached   rule K/V ({r} tokens) shifted to positions {p}..{} in seq 0", p + r);

        // Now the assistant header, after the rule - so the very next thing the model
        // attends back to, at distance one, is the rule.
        let mut hb = LlamaBatch::new(h.max(1), 1);
        for (n, tok) in header_tokens.iter().enumerate() {
            hb.add(*tok, pos + n as i32, &[0], n == h - 1)?;
        }
        ctx.decode(&mut hb)?;
        pos += h as i32;
        batch = hb;
    }
    let prefill_ms = t_prefill.elapsed().as_millis();
    eprintln!("prefill    {prefill_ms} ms");

    // ── GENERATE, greedy ─────────────────────────────────────────────────────
    let mut sampler = LlamaSampler::chain_simple([LlamaSampler::greedy()]);
    let mut decoder = encoding_rs::UTF_8.new_decoder();
    let mut out = String::new();
    let mut n = 0;
    let t_gen = Instant::now();
    while n < budget {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        sampler.accept(token);
        if model.is_eog_token(token) { break; }
        let piece = model.token_to_piece(token, &mut decoder, false, None)?;
        out.push_str(&piece);
        batch.clear();
        batch.add(token, pos, &[0], true)?;
        ctx.decode(&mut batch)?;
        pos += 1;
        n += 1;
    }
    let gen_s = t_gen.elapsed().as_secs_f32();
    eprintln!("generated  {n} tokens in {gen_s:.1}s ({:.1} tok/s)", n as f32 / gen_s.max(0.001));

    // ── THE VERDICT: did the rule hold, handler by handler ───────────────────
    // No authored pattern judges MEANING here. This reads a FORMAT: the C# minimal-API
    // lambda header between `app.MapX(` and `=>`, and asks whether the context is a
    // parameter of it. That is the rule's own shape, checked where it applies.
    let mut handlers = 0;
    let mut clean = 0;
    let mut faulty = 0;
    let mut details = Vec::new();
    for verb in ["MapGet", "MapPost", "MapPut", "MapDelete"] {
        let needle = format!("app.{verb}(");
        let mut from = 0;
        while let Some(at) = out[from..].find(&needle) {
            let start = from + at;
            let rest = &out[start..];
            let Some(arrow) = rest.find("=>") else { break };
            let header = &rest[..arrow];
            let body_end = rest[arrow..].find("});").map(|e| arrow + e).unwrap_or(rest.len().min(arrow + 600));
            let body = &rest[arrow..body_end];
            handlers += 1;
            let injected = header.contains("ToolContext");
            let root = body.contains("app.Services") || body.contains("new ToolContext(");
            if injected && !root { clean += 1; details.push(format!("{verb}: CLEAN (context injected)")); }
            else if root { faulty += 1; details.push(format!("{verb}: FAULT ({})",
                if body.contains("app.Services") { "app.Services" } else { "new ToolContext()" })); }
            else { details.push(format!("{verb}: no context used")); }
            from = start + needle.len();
        }
    }
    let (result, held) = if handlers == 0 { ("NO HANDLERS WRITTEN".to_string(), None) }
        else if faulty == 0 { ("RULE HELD".to_string(), Some(true)) }
        else if clean == 0 { ("RULE BROKEN IN EVERY HANDLER".to_string(), Some(false)) }
        else { ("RULE BROKEN IN SOME HANDLERS".to_string(), Some(false)) };

    Ok(ArmResult { arm: arm.to_string(), handlers, clean, faulty, details, result, held,
                   prompt_tokens: p, rule_tokens: r, gen_tokens: n, prefill_ms, gen_s, output: out })
}
