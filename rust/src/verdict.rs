//! THE CHECK. The organ judges the file the model just wrote against the rules it was
//! given. No regex - regex was banned on 14 Sep and the old check, regex-only, has said
//! CHECKS PASSED over every broken rule since. The Captain's GO, 17 Sep 07:00: "that is
//! the correct way, it is part of the loop."
//!
//! One question per file, all the rules in it, answered by the same local model that
//! wrote the code - greedy, short, over the organ's own endpoint on this machine. A YES
//! names the rule and the line; the agent turns that into a refusal, as it always did.
//! Every answer is checked against the file before it is believed: a rule id that is not
//! in the book, or a line number that is not in the file, is thrown away.

use std::path::Path;

use crate::{Hit, Rule};

const ORGAN_DEFAULT: &str = "http://127.0.0.1:8421";

/// Where the judge lives. SARGE_ORGAN points it at a model you are already running -
/// llama.cpp, Ollama, anything that speaks the OpenAI chat shape - instead of the one the
/// installer fetches. The Captain's point, 23 Sep: a developer with a local model already
/// on the box should not be made to download a second one.
///
/// The check still needs a model that ANSWERS rather than thinks; see the note on
/// enable_thinking below.
/// LOOPBACK ONLY, unless the user says otherwise in as many words.
///
/// @Galahad's finding, 23 Sep, and he is right: the first version of this took any URL.
/// The hook is the product now, so SARGE_ORGAN=https://anywhere/v1 would have POSTed every
/// file Claude Code writes to that host - while README.md said "what leaves your machine:
/// nothing" and the hook's own header said "No key, no network". Proven off-box before
/// this fix: SARGE_ORGAN=https://example.com returned a 405 from a remote server, which
/// means the request left.
///
/// So: 127.0.0.1, ::1 and localhost are allowed. Anything else needs
/// SARGE_ORGAN_ALLOW_REMOTE=1 as well - a second, deliberate act. A developer who has a
/// model on another box in their own network can still have it; a developer who pastes a
/// URL from somewhere cannot lose their source to it by accident.
fn is_loopback(url: &str) -> bool {
    let host = url
        .split("://").last().unwrap_or(url)
        .split('/').next().unwrap_or("")
        .rsplit_once(':').map(|(h, _)| h).unwrap_or_else(|| url.split("://").last().unwrap_or(url).split('/').next().unwrap_or(""))
        .trim_start_matches('[').trim_end_matches(']');
    matches!(host, "127.0.0.1" | "localhost" | "::1") || host.starts_with("127.")
}

pub fn organ() -> String {
    let want = std::env::var("SARGE_ORGAN")
        .ok()
        .map(|s| s.trim().trim_end_matches('/').trim_end_matches("/v1").to_string())
        .filter(|s| !s.is_empty());
    let Some(want) = want else { return ORGAN_DEFAULT.to_string() };
    if is_loopback(&want) || std::env::var("SARGE_ORGAN_ALLOW_REMOTE").as_deref() == Ok("1") {
        return want;
    }
    // Loud, and NOT a silent fall back to the default - a check that quietly judged
    // against a different model than the user asked for would be worse than refusing.
    eprintln!("SARGE_ORGAN={want} is not on this machine, and your code would be sent to it.
               Sarge judges locally. If you really mean it, set SARGE_ORGAN_ALLOW_REMOTE=1 as well.
               Using {ORGAN_DEFAULT} instead.");
    ORGAN_DEFAULT.to_string()
}

/// The prompt, in the organ's own chat form (Qwen3 speaks ChatML). The rules are given
/// with their `wrong` and `right` lines - the demonstration is the question.
fn prompt(file_name: &str, source: &str, rules: &[&Rule]) -> String {
    let mut rules_block = String::new();
    for (i, r) in rules.iter().enumerate() {
        rules_block.push_str(&format!("{}. {}\n   rule: {}\n", i + 1, r.id, r.shape));
        for w in &r.wrong { rules_block.push_str(&format!("   wrong: {w}\n")); }
        for w in &r.right { rules_block.push_str(&format!("   right: {w}\n")); }
        if let Some(a) = &r.allow { rules_block.push_str(&format!("   allowed exception: {a}\n")); }
    }
    let numbered: String = source.lines().take(400).enumerate()
        .map(|(i, l)| format!("{:4}  {}\n", i + 1, l)).collect();
    // system \x1f user - the organ's chat endpoint renders the model's own template.
    format!(
        "You are a strict, literal code reviewer. You answer ONLY in the exact format asked. \
         You never invent a line that is not in the file. You report a rule as broken only when a line in the \
         file does what the rule says never to do.\x1f\
         RULES OF THIS CODEBASE\n{rules_block}\n\
         FILE {file_name} (line numbers on the left)\n{numbered}\n\
         For every rule above that THIS FILE breaks, write exactly one line:\n\
         HIT <rule-id> line <number>: <the offending line, copied>\n\
         If the file breaks none of the rules, write exactly:\n\
         NONE\n\
         Nothing else."
    )
}

/// Ask the organ through its chat endpoint, so the model's OWN template is applied -
/// Qwen speaks ChatML, DeepSeek Coder does not, and the check must judge under either.
/// 17 Sep, for the second student. `prompt` is "<system>\n\x1f\n<user>".
fn ask_organ(prompt: &str) -> Result<String, String> {
    ask_organ_n(prompt, 240)
}

fn ask_organ_n(prompt: &str, n_predict: u32) -> Result<String, String> {
    let (system, user) = prompt.split_once('\x1f').unwrap_or(("", prompt));
    let resp: serde_json::Value = ureq::post(&format!("{}/v1/chat/completions", organ()))
        .send_json(serde_json::json!({
            "messages": [
                { "role": "system", "content": system.trim() },
                { "role": "user", "content": user.trim() }
            ],
            "max_tokens": n_predict, "temperature": 0, "cache_prompt": true,
            // A thinking model (Qwen3-1.7B, 0.6B) spends the whole budget on <think> and
            // answers nothing, so the check read a blank as NONE and caught 0 of 7 (19 Sep).
            // llama-server passes this through to the chat template; a non-thinking model
            // ignores it. The judge must speak, not think.
            // NOT adding a stop sequence here, and the reason is recorded because it was
            // tried and reverted on 20 Sep. The release review found a single call running
            // to the 240-token cap at ~4 tok/s on CPU (~59s) and that IS real. But on the
            // fixtures measured here the cost was spread evenly: 7 calls, ~10s each, 70s
            // total, and stop sequences changed nothing. So the driver is the NUMBER of
            // calls as much as the length of any one, and a stop sequence risks truncating
            // a legitimate list of HITs to buy a saving that did not appear. The real fix
            // is fewer or cheaper calls, which is a design change, not a flag.
            //
            // Deleted by accident on 23 Sep - verdict.rs was overwritten wholesale from a
            // working clone that never had it - and restored the same day on @Galahad's
            // finding. The condition has not changed. It is ALSO in
            // docs/FINDING-STOP-SEQUENCE.md now, so a file overwrite cannot take it again.
            "chat_template_kwargs": { "enable_thinking": false }
        }))
        .map_err(|e| format!("organ not reachable at {}: {e}", organ()))?
        .into_json()
        .map_err(|e| e.to_string())?;
    let answer = resp["choices"][0]["message"]["content"].as_str().map(|s| s.to_string())
        .ok_or_else(|| "empty answer from the organ".to_string())?;
    // A blank answer is not NONE. A judge that said nothing has not judged; say so
    // loudly (CHECKS INCOMPLETE) rather than pass the file (the 0-of-7 of 19 Sep).
    if answer.trim().is_empty() {
        return Err("the organ answered nothing - a judge that says nothing has not judged".to_string());
    }
    // SARGE_TRACE=1 shows what the organ was asked and what it said - the only way to
    // tell a student that cannot judge from a prompt it cannot read (DeepSeek, 17 Sep).
    if std::env::var_os("SARGE_TRACE").is_some() {
        eprintln!("--- organ asked ---\n{}\n--- organ said ---\n{}\n---", user.trim(), answer.trim());
    }
    Ok(answer)
}

/// Parse `HIT <id> line <n>: <text>` lines, and believe only the ones the file bears out.
fn parse(answer: &str, file_name: &str, source: &str, rules: &[&Rule]) -> Vec<Hit> {
    let lines: Vec<&str> = source.lines().collect();
    let mut hits = Vec::new();
    for l in answer.lines() {
        let t = l.trim();
        let Some(rest) = t.strip_prefix("HIT ") else { continue };
        let mut parts = rest.splitn(2, " line ");
        let who = parts.next().unwrap_or("").trim();
        // The organ names the rule by its list number ("HIT 1"), by its id, or both
        // ("HIT 1 use-config-for-connection") - measured 17 Sep. Take any of them.
        let r = who.split_whitespace().find_map(|w| {
            if let Ok(n) = w.parse::<usize>() { return (n >= 1 && n <= rules.len()).then(|| rules[n - 1]); }
            rules.iter().copied().find(|r| r.id == w)
        });
        let Some(r) = r else { continue };
        let after = parts.next().unwrap_or("");
        let (num, quoted) = match after.split_once(':') {
            Some((n, q)) => (n.trim(), q.trim()),
            None => (after.trim(), ""),
        };
        let Ok(n) = num.parse::<usize>() else { continue };
        if n == 0 || n > lines.len() { continue }
        let actual = lines[n - 1].trim();
        // The quoted line must be the real one, or at least share its substance. A model
        // that names line 12 and quotes line 40 is guessing.
        let bears_out = quoted.is_empty()
            || actual.contains(quoted.trim_end_matches(';'))
            || quoted.contains(actual.trim_end_matches(';'))
            || overlap(actual, quoted) >= 0.6;
        if !bears_out { continue }
        if hits.iter().any(|h: &Hit| h.rule == r.id && h.line == n) { continue }
        hits.push(Hit { rule: r.id.clone(), file: file_name.to_string(), line: n,
                        matched: actual.to_string(), shape: r.shape.clone() });
    }
    hits
}

/// Words too common to tell a wrong line from a right one: SQL, keywords, boilerplate.
const NOISE: &[&str] = &[
    "select", "from", "where", "set", "into", "values", "table", "create", "update", "insert",
    "const", "let", "var", "return", "using", "new", "async", "function", "public", "private",
    "app", "res", "req", "tools", "tool", "context", "result", "services", "builder", "options",
    "name", "price", "true", "false", "null", "undefined", "string", "int", "json", "status",
    "get", "post", "put", "delete", "exec", "prepare", "run", "all", "then", "catch", "error",
];

/// Words of a line: alphanumeric runs of three or more, plus quoted literals whole.
fn tokens(s: &str) -> std::collections::HashSet<String> {
    let mut out: std::collections::HashSet<String> = s
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.len() >= 3 && !NOISE.contains(&w.to_lowercase().as_str()))
        .map(|w| w.to_string())
        .collect();
    // A quoted literal is one token - 'tools.db' is the point of the hardcoded-name rule.
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\'' || c == '"' {
            let lit: String = chars.by_ref().take_while(|&x| x != c).collect();
            if lit.len() >= 3 && lit.len() <= 80 { out.insert(lit); }
        }
    }
    out
}

/// Tokens the wrong examples carry that no right example does. Empty when a rule has
/// no right examples to subtract - then every line is fair game for the second look.
fn discriminating(r: &Rule) -> std::collections::HashSet<String> {
    if r.right.is_empty() { return std::collections::HashSet::new() }
    let mut wrong = std::collections::HashSet::new();
    for w in &r.wrong { wrong.extend(tokens(w)); }
    let mut right = std::collections::HashSet::new();
    for w in &r.right { right.extend(tokens(w)); }
    wrong.difference(&right).cloned().collect()
}

fn overlap(a: &str, b: &str) -> f32 {
    let wa: std::collections::HashSet<&str> = a.split(|c: char| !c.is_alphanumeric()).filter(|w| w.len() > 2).collect();
    let wb: std::collections::HashSet<&str> = b.split(|c: char| !c.is_alphanumeric()).filter(|w| w.len() > 2).collect();
    if wa.is_empty() || wb.is_empty() { return 0.0 }
    wa.intersection(&wb).count() as f32 / wa.len().max(wb.len()) as f32
}

/// How many of a rule set the check can enforce - the demonstrated ones.
pub fn enforced_count(rules: &[&Rule]) -> usize {
    rules.iter().filter(|r| !r.wrong.is_empty()).count()
}

/// Judge one file against the rules. `Err` means the organ could not be asked - the
/// caller decides what an unanswered check is worth (it is never a pass).
pub fn judge_file(path: &Path, rel: &str, all_rules: &[&Rule]) -> Result<Vec<Hit>, String> {
    // ENFORCED means demonstrated. A rule with no `wrong` line is a slogan - delivered to
    // the model, never judged. 17 Sep: every false hit in the replay came from a rule that
    // had no example to compare against. The language says it; the check now means it.
    let enforced: Vec<&Rule> = all_rules.iter().copied().filter(|r| !r.wrong.is_empty()).collect();
    let rules = &enforced[..];
    if rules.is_empty() { return Ok(Vec::new()) }
    let source = std::fs::read_to_string(path).map_err(|e| format!("cannot read {rel}: {e}"))?;
    let source = source.trim_start_matches('\u{feff}');
    if source.trim().is_empty() { return Ok(Vec::new()) }
    let answer = ask_organ(&prompt(rel, source, rules))?;
    let mut candidates = parse(&answer, rel, source, rules);
    // THE DISCRIMINATING WORD. What the wrong examples say that the right ones never do -
    // `await`, `AddControllers`, `EnsureDeleted`, `'tools.db'`, `<td>`. Two uses: a line
    // that carries one becomes a candidate even if the organ's first pass missed it (it
    // missed EnsureDeleted once); and a candidate that carries none is dropped before the
    // second look - the organ said WRONG to a clean SELECT because it looked like the
    // example minus the one word that mattered. Token sets, not patterns.
    // (Generating candidates FROM the tokens was tried and flooded the check - `Services`
    // and `FROM` lit up every line. The organ finds candidates by meaning; the tokens only
    // gate them.)
    candidates.retain(|h| {
        let Some(r) = rules.iter().find(|r| r.id == h.rule) else { return false };
        let disc = discriminating(r);
        disc.is_empty() || tokens(&h.matched).iter().any(|t| disc.contains(t))
    });
    // THE SECOND LOOK. The first pass over-reports - 17 Sep it flagged two correct
    // handlers beside the one real fault. A false YES blocks a correct run, which is
    // worse than a miss, so every candidate is put to a narrow question on its own:
    // this rule, this one line, yes or no. Only a YES survives.
    let mut hits = Vec::new();
    let all_lines: Vec<&str> = source.lines().collect();
    for h in candidates {
        let Some(r) = rules.iter().find(|r| r.id == h.rule) else { continue };
        // The line with its neighbours: the startup scope sits on the line above, the
        // `changes === 0` check BELOW. One line alone is a blind spot - and so was one
        // line below: 18 Sep, the Captain's first run, the model wrote the UPDATE, a blank
        // line, a comment, then `if (result.changes === 0) return 404` three lines down,
        // and the check flagged the write five times with the proof just out of frame.
        // Two above, five below: a write is followed by what checks it.
        let from = h.line.saturating_sub(2); let to = (h.line + 5).min(all_lines.len());
        let window: String = (from..to).map(|i| format!("{}{:4}  {}\n", if i + 1 == h.line { ">" } else { " " }, i + 1, all_lines[i])).collect();
        match confirm(r, &h.matched, &window) {
            Ok(true) => hits.push(h),
            Ok(false) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(hits)
}

fn confirm(r: &Rule, line: &str, window: &str) -> Result<bool, String> {
    let mut demo = String::new();
    for w in &r.wrong { demo.push_str(&format!("WRONG example: {w}\n")); }
    for w in &r.right { demo.push_str(&format!("RIGHT example: {w}\n")); }
    if let Some(a) = &r.allow { demo.push_str(&format!("ALLOWED exception: {a}\n")); }
    // A likeness question, not a judgement: is the line doing what the WRONG examples do,
    // or what the RIGHT examples do? Small models answer likeness well and abstract rules
    // badly - 17 Sep, "quote the forbidden part" flagged five clean lines because a model
    // can always find a substring to quote. Only WRONG is a hit.
    // `allow` is a word the model may answer with, not only a line it may read. 17 Sep,
    // the first run through the bridge: the rule ALLOWED "resolving inside an explicit
    // scope created with app.Services.CreateScope() at startup", the line above the hit
    // was exactly that scope, and the check flagged it five times because the only words
    // on offer were WRONG, RIGHT and NEITHER. The model argued the exception correctly in
    // prose and the check could not hear it. The language has `allow`; the check now does.
    let allowed_word = if r.allow.is_some() { ", or ALLOWED if it is the ALLOWED exception" } else { "" };
    let p = format!(
        "You are a strict, literal code reviewer. Answer with one word.\x1f\
         RULE: {}\n{demo}\nTHE LINE, marked >, with the lines around it:\n{window}\n\
         Is the marked line doing what the WRONG examples do, or what the RIGHT examples do{allowed_word}? \
         Read the lines around it before answering. Answer exactly one word: WRONG, RIGHT, NEITHER{}.",
        r.shape, if r.allow.is_some() { ", or ALLOWED" } else { "" });
    let in_context = ask_organ_n(&p, 6)?.trim().to_uppercase();
    if in_context.starts_with("ALLOWED") { return Ok(false) }
    // Two looks, and both must agree it is not fine. The line alone catches the handler
    // that resolves from app.Services (the window let it pass); the window catches the
    // startup scope and the checked UPDATE that the line alone flagged. WRONG alone,
    // and not RIGHT in context, is a hit.
    let alone = format!(
        "You are a strict, literal code reviewer. Answer with one word.\x1f\
         RULE: {}\n{demo}\nLINE OF CODE:\n{line}\n\n\
         Is this line doing what the WRONG examples do, or what the RIGHT examples do? \
         Answer exactly one word: WRONG, RIGHT, or NEITHER.", r.shape);
    let by_itself = ask_organ_n(&alone, 6)?.trim().to_uppercase();
    Ok(by_itself.starts_with("WRONG") && !in_context.starts_with("RIGHT"))
}


/// Source files worth judging under a folder. Never node_modules, bin, obj, dist.
pub fn source_files(root: &Path) -> Vec<std::path::PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if p.is_dir() {
                if matches!(name.as_str(), "node_modules" | "bin" | "obj" | "dist" | ".git" | ".vs") { continue }
                walk(&p, out);
            } else if let Some(ext) = p.extension().and_then(|x| x.to_str()) {
                if matches!(ext, "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx") { out.push(p) }
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(id: &str, shape: &str) -> Rule {
        Rule { id: id.into(), kind: "prohibit".into(), topic: String::new(), shape: shape.into(),
               allow: None, applies: None, lane: None, check: None, never: None, instead: None,
               wrong: vec![], right: vec![], since: None }
    }

    #[test]
    fn believes_only_lines_the_file_bears_out() {
        let r = rule("no-x", "never call x()");
        let rs = vec![&r];
        let src = "let a = 1;\nx();\nlet b = 2;\n";
        let good = parse("HIT no-x line 2: x();", "f.js", src, &rs);
        assert_eq!(good.len(), 1); assert_eq!(good[0].line, 2);
        let wrong_line = parse("HIT no-x line 9: x();", "f.js", src, &rs);
        assert!(wrong_line.is_empty());
        let wrong_quote = parse("HIT no-x line 1: x();", "f.js", src, &rs);
        assert!(wrong_quote.is_empty());
        let unknown = parse("HIT no-y line 2: x();", "f.js", src, &rs);
        assert!(unknown.is_empty());
        assert!(parse("NONE", "f.js", src, &rs).is_empty());
    }
}
