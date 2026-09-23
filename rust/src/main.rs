//! A thin CLI over the handshake library, for testing it against the Python one.
//!
//! The library is the product - it is meant to be linked into llama.cpp so the rules
//! go straight into the context with no process boundary. This binary exists so the
//! same logic can be run and compared before that binding lands.

use handshake::{check, for_scope, load, render, select, MAX_RULES};
use std::path::{Path, PathBuf};

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut task: Option<String> = None;
    let mut scope: Option<String> = None;
    let mut lane: Option<String> = None;
    let mut check_dir: Option<PathBuf> = None;
    let mut check_file: Option<String> = None;   // --file: the one file just written
    let mut rules_path = PathBuf::from("../harness/rules.jsonl");
    // --learn: the loop back into the book, and the ONLY door to the tutor. The agent
    // hands over the red build; this decides whether the fault is one the book already
    // knows, and only then is a token spent. Vinn's ruling, 14 Sep 2026: if the tutor
    // spends on a fault the rules already cover, the design is wrong.
    let mut learn_errors: Option<String> = None;
    let mut answer_path: Option<PathBuf> = None;
    let mut ask_question: Option<String> = None;
    let mut ask_failure: Option<String> = None;
    let mut all = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--task" => { task = args.get(i + 1).cloned(); i += 2; }
            "--repo" => { scope = args.get(i + 1).cloned(); i += 2; }
            "--lane" => { lane = args.get(i + 1).cloned(); i += 2; }
            "--check" => { check_dir = args.get(i + 1).map(PathBuf::from); i += 2; }
            "--file" => { check_file = args.get(i + 1).cloned(); i += 2; }
            "--rules" => { rules_path = args.get(i + 1).map(PathBuf::from).unwrap_or(rules_path); i += 2; }
            // The WHOLE book for this repo, not a BM25 top-five. compile.exe had this and
            // handshake.exe did not - so the agent, which calls handshake, was sending the
            // model two or three rules out of fifteen. 15 Sep 2026.
            "--all" => { all = true; i += 1; }
            "--learn" => { learn_errors = args.get(i + 1).cloned(); i += 2; }
            "--answer" => { answer_path = args.get(i + 1).map(PathBuf::from); i += 2; }
            // THE STUDENT ASKS. The other direction of the loop: the model asks a
            // specific question about a failure it is looking at, and gets an answer for
            // NOW rather than a rule for later. Vinn's design, 15 Sep 2026.
            "--ask" => { ask_question = args.get(i + 1).cloned(); i += 2; }
            "--failure" => { ask_failure = args.get(i + 1).cloned(); i += 2; }
            "--export" => { i += 2; }
            other => { eprintln!("unknown argument: {other}"); return std::process::ExitCode::from(2); }
        }
    }

    let (mut rules, problems) = match load(&rules_path) {
        Ok(r) => r,
        Err(e) => { eprintln!("cannot read {}: {e}", rules_path.display()); return std::process::ExitCode::from(2); }
    };
    for p in &problems {
        eprintln!("skipped a malformed rule, {p}");
    }
    // The repo's OWN rules are the first `own_rules` entries - everything merged in below
    // is a shipped book. The check needs to tell them apart: own rules were written from
    // this repo's failures and are always judged; shipped ones are selected per file.
    let own_rules = rules.len();
    // The universal laws travel with the binary: book/universal.sarge beside this repo,
    // loaded under every book that is not itself the universal one. Where a file sits is
    // its scope - the Captain's ruling, 16 Sep. A .sarge in a repo applies to that repo.
    // book/node.sarge loads above it, under the same merge, when the repo looks like Node
    // (22 Sep) - so a repo in another language is never handed rules about express.
    let repo_dir = check_dir.clone()
        .or_else(|| rules_path.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    for u in [stack_book(&repo_dir, task.as_deref().unwrap_or("")), universal_path()].into_iter().flatten() {
        if u.exists() && std::fs::canonicalize(&u).ok() != std::fs::canonicalize(&rules_path).ok() {
            if let Ok((ur, up)) = load(&u) {
                for p in &up { eprintln!("universal: skipped a malformed rule, {p}"); }
                // Same id in both: the DEMONSTRATED one wins - a rule with wrong/right lines
                // can be enforced, a bare one cannot. 17 Sep: myshop's example-less copy of
                // archive-not-delete shadowed the universal one and EnsureDeleted walked past.
                for r in ur {
                    match rules.iter().position(|x| x.id == r.id) {
                        Some(i) if rules[i].wrong.is_empty() && !r.wrong.is_empty() => rules[i] = r,
                        Some(_) => {}
                        None => rules.push(r),
                    }
                }
            }
        }
    }

    // --export <path>: write the loaded book out in the language. The way a JSONL book
    // becomes a .sarge one.
    if let Some(pos) = args.iter().position(|a| a == "--export") {
        if let Some(out) = args.get(pos + 1) {
            // Scoped when --repo is given: a repo's .sarge holds that repo's rules and
            // the unscoped ones, never another repo's.
            let refs: Vec<&handshake::Rule> = match scope.as_deref() {
                Some(s) => for_scope(&rules, Some(s)),
                None => rules.iter().collect(),
            };
            if let Err(e) = std::fs::write(out, handshake::lang::write_book(&refs)) {
                eprintln!("cannot write {out}: {e}"); return std::process::ExitCode::from(2);
            }
            println!("wrote {} rule(s) to {out}", refs.len());
            return std::process::ExitCode::SUCCESS;
        }
    }

    let scoped = for_scope(&rules, scope.as_deref());

    // ── ASK ──────────────────────────────────────────────────────────────────
    // The student's question. The other direction of the loop, and the cheaper one: an
    // answer to the exact question beats a rule inferred from wreckage. Costs one call
    // and the cost is printed, so it lands on the ledger like everything else.
    if let Some(q) = ask_question {
        let t = task.clone().unwrap_or_default();
        let f = ask_failure.unwrap_or_default();
        match handshake::tutor::answer(&t, &f, &q) {
            Ok((text, pt, ct)) => {
                println!("{text}");
                let (inr, outr) = handshake::tutor::rates();
                let usd = pt as f64 / 1_000_000.0 * inr + ct as f64 / 1_000_000.0 * outr;
                println!("\nCOST {pt} in, {ct} out, ${usd:.6}");
                return std::process::ExitCode::SUCCESS;
            }
            Err(e) => { eprintln!("tutor unreachable: {e}"); return std::process::ExitCode::from(3); }
        }
    }

    // ── LEARN ────────────────────────────────────────────────────────────────
    // The loop back. One door in, one door out: nothing else may call the tutor and
    // nothing else may write the book.
    if let Some(errors) = learn_errors {
        let t = task.clone().unwrap_or_default();
        // Is this fault one the book already covers? Cosine between the compiler's own
        // words and the shapes we hold. No authored pattern, no model call to decide it.
        match handshake::book::embed(&errors) {
            Ok(v) => {
                let mut best = 0.0f32;
                let mut near = String::new();
                for r in &rules {
                    if let Ok(rv) = handshake::book::embed(&r.shape) {
                        let c = handshake::book::cosine(&v, &rv);
                        if c > best { best = c; near = r.id.clone(); }
                    }
                }
                if best >= handshake::book::same_law_threshold() {
                    println!("KNOWN FAULT  the book already covers this ({near})");
                    println!("  no tutor call, no token spent - the rule exists and did not hold");
                    return std::process::ExitCode::SUCCESS;
                }
            }
            Err(e) => eprintln!("could not weigh the fault against the book: {e}"),
        }

        let answer = answer_path.as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_default();
        let shapes: Vec<String> = scoped.iter().map(|r| r.shape.clone()).collect();
        let files: Vec<String> = Vec::new();
        match handshake::tutor::teach(&t, &answer, &shapes, &errors, &files,
                                      scope.as_deref(), &rules_path) {
            Ok(l) => {
                match (&l.rule, l.status) {
                    (Some(r), "learned") => println!("LEARNED  {}\n  {}", r.id, r.shape),
                    (Some(r), "known") => println!("ALREADY HELD  {}", r.id),
                    (Some(_), "same-law") => println!("SAME LAW  as {}",
                                                     l.near.clone().unwrap_or_default()),
                    _ => println!("NOTHING TO TEACH"),
                }
                // The provider's own count, so the figure on the page is the figure on
                // the bill. Parsed by the console; never re-estimated anywhere.
                println!("COST {} in, {} out, ${:.6}",
                         l.prompt_tokens, l.completion_tokens, l.usd);
                if !l.correction.is_empty() {
                    println!("\n{}", l.correction);
                }
                return std::process::ExitCode::SUCCESS;
            }
            Err(e) => { eprintln!("tutor unreachable: {e}"); return std::process::ExitCode::from(3); }
        }
    }

    // ── CHECK ────────────────────────────────────────────────────────────────
    // The organ judges the written file against EVERY rule in scope - not a BM25 five,
    // and not a regex. `--file` names the file the agent just wrote; without it every
    // source file under the folder is judged (capped). An organ that cannot be asked
    // is reported as UNAVAILABLE, never as a pass.
    if let Some(dir) = check_dir {
        let _ = &check;   // the regex check stays for its tests; the loop uses the organ
        // NOT the whole book. 22 Sep: with 80 shipped rules in one prompt the organ stopped
        // reading the file and recited the rule list back ("HIT 1 wrong: ... HIT 2 wrong:
        // ..." down the list, naming rules that appear nowhere in the file) - one real
        // fault missed, a clean file flagged twice. Every other call site selects; the
        // check did not. Now, per file: the repo's own rules always, plus the shipped rules
        // that share words with THIS file, capped. The replay was perfect at nine rules.
        let (own, shipped): (Vec<&handshake::Rule>, Vec<&handshake::Rule>) = scoped.iter().copied()
            .partition(|r| rules.iter().position(|x| std::ptr::eq(x, *r)).is_some_and(|i| i < own_rules));
        // Only SOURCE is judged. 17 Sep, first live run: the check flagged config.json
        // itself for "hardcoding the database name" - the config file is where the name
        // is supposed to live - and the model emptied it and moved the name to an env
        // var in a loop. A .json, .md or lockfile is data, not code.
        if let Some(f) = &check_file {
            let ext = std::path::Path::new(f).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            if !matches!(ext.as_str(), "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx") {
                println!("CHECKS PASSED  (not a source file - data and config are not judged)");
                return std::process::ExitCode::SUCCESS;
            }
        }
        let files: Vec<std::path::PathBuf> = match &check_file {
            Some(f) => vec![dir.join(f)],
            None => handshake::verdict::source_files(&dir).into_iter().take(12).collect(),
        };
        const CHECK_RULES: usize = 10;
        let mut hits = Vec::new();
        let mut judged = 0usize;
        let mut enforced_total = 0usize;
        let mut advisory_total = 0usize;
        for f in &files {
            let rel = f.strip_prefix(&dir).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_else(|_| f.to_string_lossy().to_string());
            // The file's own words are the query: a file that mentions express.json,
            // DatabaseSync or readFileSync pulls the rules about them and no others.
            let text = std::fs::read_to_string(f).unwrap_or_default();
            let mut chosen: Vec<&handshake::Rule> = own.clone();
            for r in select(&shipped, &text, None, CHECK_RULES) {
                if !chosen.iter().any(|c| c.id == r.id) { chosen.push(r); }
            }
            let enforced = handshake::verdict::enforced_count(&chosen);
            enforced_total += enforced;
            advisory_total += chosen.len() - enforced;
            match handshake::verdict::judge_file(f, &rel, &chosen) {
                Ok(mut h) => { judged += 1; hits.append(&mut h); }
                Err(e) => {
                    println!("CHECKS UNAVAILABLE  ({e})");
                    return std::process::ExitCode::from(2);
                }
            }
        }
        if hits.is_empty() {
            println!("CHECKS PASSED  ({enforced_total} demonstrated rule(s) judged by the organ over {judged} file(s); {advisory_total} advisory, not judged)");
            return std::process::ExitCode::SUCCESS;
        }
        println!("CHECKS FAILED - {} hit(s)", hits.len());
        for h in &hits {
            println!("  [{}] {}:{}  matched `{}`", h.rule, h.file, h.line, h.matched);
            println!("      rule: {}", h.shape);
        }
        return std::process::ExitCode::from(1);
    }

    let Some(t) = task else {
        println!("{} rule(s) loaded{}", scoped.len(),
                 scope.map(|s| format!(" for {s}")).unwrap_or_default());
        for r in &scoped {
            let mark = if r.is_enforceable() { " " } else { "!" };
            println!("  {mark} {:<24} {:<9} {}", r.id, r.kind,
                     r.shape.chars().take(64).collect::<String>());
        }
        if scoped.iter().any(|r| !r.is_enforceable()) {
            println!("\n  ! = no check. Guidance only - nothing stops the model breaking it.");
        }
        return std::process::ExitCode::SUCCESS;
    };

    let chosen = if all { scoped.clone() } else { select(&scoped, &t, lane.as_deref(), MAX_RULES) };
    print!("{}", render(&chosen));
    println!("\n<!-- {} of {} rules selected -->", chosen.len(), scoped.len());
    std::process::ExitCode::SUCCESS
}

#[allow(dead_code)]
fn default_rules() -> &'static Path {
    Path::new("../harness/rules.jsonl")
}

/// book/universal.sarge, found from the binary: rust/target/release/handshake.exe ->
/// ../../../book/universal.sarge. No absolute path, so the repo can move.
fn universal_path() -> Option<PathBuf> {
    std::env::current_exe().ok()
        .and_then(|e| e.ancestors().nth(4).map(|r| r.join("book").join("universal.sarge")))
}

/// The book for the language this repo is written in, found the same way as the universal
/// one. 22 Sep: `book/node.sarge`, the Node and Express laws taken from the documentation.
/// It loads only when the repo LOOKS like Node - a package.json, or a .js/.ts file at the
/// root - so nothing hands express rules to a repo that has no express in it.
/// THE STACK'S BOOK. "Books are per-stack, the engine isn't" - the Captain's ruling,
/// 22 Sep. Nothing below knows what Node is: a book is `book/<stack>.sarge`, a stack is
/// named by MARKERS declared in that book's own `stack` line, and whichever book's markers
/// match this repo is the one that loads. Drop `book/python.sarge` in with its own markers
/// and it works with no change here.
///
/// Two ways a repo names its stack, because a NEW project has no files. The Captain's
/// first test was a fresh folder - nothing but `.sarge` - and a file-sniffing version gave
/// it five rules instead of sixty. So: the repo's own files when it has them, and
/// otherwise the words of the task, which say "a Node.js Express API" before any file
/// exists.
fn stack_book(repo: &Path, task: &str) -> Option<PathBuf> {
    let books = std::env::current_exe().ok()
        .and_then(|e| e.ancestors().nth(4).map(|r| r.join("book")))?;

    // What this repo shows of itself: file names and extensions, one lowercase haystack.
    let mut seen = String::new();
    if let Ok(d) = std::fs::read_dir(repo) {
        for e in d.flatten() {
            seen.push_str(&e.file_name().to_string_lossy().to_lowercase());
            seen.push(' ');
        }
    }
    let task = task.to_lowercase();

    let mut best: Option<(usize, PathBuf)> = None;
    for e in std::fs::read_dir(&books).ok()?.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("sarge") { continue }
        if p.file_stem().and_then(|x| x.to_str()) == Some("universal") { continue }
        let Ok(text) = std::fs::read_to_string(&p) else { continue };
        // `stack  package.json .js .ts node express` - outside any rule, so the parser
        // reports it and moves on; a book without one is never auto-loaded.
        let Some(line) = text.lines().find_map(|l| l.trim().strip_prefix("stack ")) else { continue };
        let markers: Vec<String> = line.split_whitespace().map(|m| m.to_lowercase()).collect();
        let n = markers.iter().filter(|m| seen.contains(m.as_str()) || task.contains(m.as_str())).count();
        if n > 0 && best.as_ref().is_none_or(|(b, _)| n > *b) { best = Some((n, p)); }
    }
    best.map(|(_, p)| p)
}
