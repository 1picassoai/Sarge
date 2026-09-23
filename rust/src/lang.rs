//! The rule language. Its name is Sarge - the Captain's ruling, 17 Sep: the language and
//! the engine share one name, the way SQL is both the language and what every engine
//! speaks. (AIML was the working name for a day; it is a 2001 chatbot format, binned.)
//! Files are `.sarge`. The parser does not care what the file is called.
//!
//! Universal like SQL: the same words in every repo and every programming language.
//! What a file applies to is where it sits. No scope in the syntax.
//!
//! One word belongs to the BOOK rather than to a rule: `stack <markers>`, between the
//! frame and the first rule, saying what the book is FOR. The engine names no
//! language - a book declares its own markers and the matching one loads.
//!
//!     === sarge ===
//!
//!     rule scoped-context-in-handler
//!       do     take the DbContext as a handler parameter
//!       never  create or resolve the DbContext inside a handler
//!       wrong  | using var db = new ToolContext(options);
//!       right  | app.MapGet("/tools", async (ToolContext db) => ...);
//!       allow  a project that uses MVC controllers
//!       since  2026-09-15 tutor
//!     end
//!
//!     === end sarge ===
//!
//! Three jobs, one mark each: the frame (`=== sarge ===`) is what a cached block carries
//! so the model reads an order, not words; `wrong |` / `right |` are real code, because a
//! next-token predictor follows a demonstration better than a description; and `wrong`
//! is what the check asks about. `do` comes first because positives hold across a session
//! and prohibitions decay. `|` keeps code unparsed - no braces, no quotes, no escaping,
//! nothing a BOM can hide in: a block that does not open with the frame is not a block.

use crate::Rule;

pub const OPEN: &str = "=== sarge ===";
pub const CLOSE: &str = "=== end sarge ===";

/// Does this text look like the language rather than JSONL? The frame decides.
pub fn is_sarge(text: &str) -> bool {
    text.trim_start().starts_with("=== sarge")
}

/// Parse a book. Malformed rules are reported, never fatal - one bad rule must not cost
/// the other forty. `applies` is set by the caller from where the file sits.
pub fn parse(text: &str) -> (Vec<Rule>, Vec<String>) {
    let mut rules = Vec::new();
    let mut problems = Vec::new();
    let mut cur: Option<(usize, Rule)> = None;
    let mut framed = false;

    for (i, raw) in text.lines().enumerate() {
        let n = i + 1;
        let line = raw.trim_end();
        let t = line.trim();
        if t.is_empty() { continue; }
        if t == OPEN { framed = true; continue; }
        if t == CLOSE { framed = false; continue; }
        // `stack <markers>` - a book saying what it is FOR, between the frame and the
        // first rule. The engine names no language; a book declares its own markers and
        // whichever book matches the repo or the task is the one that loads. It belongs to
        // the BOOK, not to a rule, so it sits outside them.
        //
        // It was written into book/node.sarge before the parser knew the word, so every
        // user running against the shipped book saw "skipped a malformed rule, line 3" on
        // a book WE ship - which reads as a broken install on first contact. @Galahad
        // found it on the RC walk, 23 Sep. Nothing was lost (all 33 rules still loaded);
        // what was lost was confidence, and on a first run that is the whole of it.
        if t.starts_with("stack ") && cur.is_none() {
            continue;
        }
        if !framed && cur.is_none() {
            problems.push(format!("line {n}: outside the frame ({OPEN} … {CLOSE}): `{t}`"));
            continue;
        }

        if let Some(id) = t.strip_prefix("rule ") {
            if let Some((start, r)) = cur.take() {
                problems.push(format!("line {start}: rule `{}` has no `end`", r.id));
            }
            cur = Some((n, blank(id.trim())));
            continue;
        }
        let Some((start, r)) = cur.as_mut() else {
            problems.push(format!("line {n}: `{t}` is not inside a rule"));
            continue;
        };
        if t == "end" {
            let (start, r) = cur.take().unwrap();
            match finish(r) {
                Ok(r) => rules.push(r),
                Err(e) => problems.push(format!("line {start}: {e}")),
            }
            continue;
        }
        let (key, rest) = match t.split_once(char::is_whitespace) {
            Some((k, v)) => (k, v.trim()),
            None => (t, ""),
        };
        match key {
            "do"    => r.instead = Some(rest.to_string()),
            "never" => r.never = Some(rest.to_string()),
            "allow" => r.allow = Some(rest.to_string()),
            "since" => r.since = Some(rest.to_string()),
            "wrong" | "right" => {
                let code = rest.strip_prefix('|').map(|c| c.trim_start()).unwrap_or(rest).to_string();
                if key == "wrong" { r.wrong.push(code) } else { r.right.push(code) }
            }
            other => problems.push(format!("line {n}: `{other}` is not a word in the language (rule `{}`)", r.id)),
        }
        let _ = start;
    }
    if let Some((start, r)) = cur {
        problems.push(format!("line {start}: rule `{}` has no `end`", r.id));
    }
    (rules, problems)
}

fn blank(id: &str) -> Rule {
    Rule { id: id.to_string(), kind: "prohibit".into(), topic: String::new(), shape: String::new(),
           allow: None, applies: None, lane: None, check: None, never: None, instead: None,
           wrong: Vec::new(), right: Vec::new(), since: None }
}

/// A rule is whole when it has a `do`. `never` alone is a slogan - measured: bare
/// prohibitions broke at every rule count, `instead` rules held.
fn finish(mut r: Rule) -> Result<Rule, String> {
    if r.id.is_empty() { return Err("rule has no id".into()); }
    // The tutor once copied the template's placeholder id verbatim (16 Sep). A placeholder
    // is not a name.
    if r.id.contains('<') || r.id == "short-kebab-case-id" || r.id.contains("your-own-id") {
        return Err(format!("rule id `{}` is the template placeholder, not a name", r.id));
    }
    let Some(d) = r.instead.clone() else {
        return Err(format!("rule `{}` has no `do` - a rule that names no replacement does not hold", r.id));
    };
    r.shape = match &r.never {
        Some(n) => format!("{}; never {}.", cap(&d), n.trim_end_matches('.')),
        None => format!("{}.", cap(&d)),
    };
    if r.never.is_none() { r.kind = "style".into(); }
    // Topic words for SELECT come from the sentences and the code, so a task that names
    // the thing finds the rule.
    let mut topic = String::new();
    for s in [&d].into_iter().chain(r.never.iter()).chain(r.wrong.iter()).chain(r.right.iter()) {
        topic.push_str(s); topic.push(' ');
    }
    r.topic = topic.trim().to_string();
    Ok(r)
}

fn cap(s: &str) -> String {
    let mut c = s.trim().chars();
    match c.next() { Some(f) => f.to_uppercase().collect::<String>() + c.as_str(), None => String::new() }
}

/// One rule as a block. A JSON-born rule (no `do`) is rendered from its shape.
pub fn write_rule(r: &Rule) -> String {
    let mut s = format!("rule {}\n", r.id);
    let d = r.instead.clone().unwrap_or_else(|| r.shape.split(';').next().unwrap_or("").trim().to_string());
    s.push_str(&format!("  do     {}\n", d.trim_end_matches('.')));
    if let Some(n) = &r.never { s.push_str(&format!("  never  {}\n", n.trim_end_matches('.'))); }
    for w in &r.wrong { s.push_str(&format!("  wrong  | {w}\n")); }
    for w in &r.right { s.push_str(&format!("  right  | {w}\n")); }
    if let Some(a) = &r.allow { s.push_str(&format!("  allow  {}\n", a.trim_end_matches('.'))); }
    if let Some(w) = &r.since { s.push_str(&format!("  since  {w}\n")); }
    s.push_str("end\n");
    s
}

/// A whole book, framed.
pub fn write_book(rules: &[&Rule]) -> String {
    let mut s = format!("{OPEN}\n\n");
    for r in rules { s.push_str(&write_rule(r)); s.push('\n'); }
    s.push_str(CLOSE);
    s.push('\n');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOOK: &str = "=== sarge ===\n\nrule a\n  do     take it as a parameter\n  never  new it up\n  wrong  | var x = new T();\n  right  | (T x) => x\n  since  2026-09-16 tutor\nend\n\n=== end sarge ===\n";

    #[test]
    fn round_trips() {
        let (rules, problems) = parse(BOOK);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].shape, "Take it as a parameter; never new it up.");
        assert_eq!(rules[0].wrong, vec!["var x = new T();"]);
        let again = write_book(&rules.iter().collect::<Vec<_>>());
        let (r2, p2) = parse(&again);
        assert!(p2.is_empty());
        assert_eq!(r2[0].shape, rules[0].shape);
    }

    #[test]
    fn a_rule_without_do_is_reported_not_kept() {
        let (rules, problems) = parse("=== sarge ===\nrule b\n  never  do the thing\nend\n=== end sarge ===\n");
        assert!(rules.is_empty());
        assert_eq!(problems.len(), 1);
    }

    #[test]
    fn a_stray_word_is_reported() {
        let (rules, problems) = parse("=== sarge ===\nrule c\n  do     x\n  maybe  y\nend\n=== end sarge ===\n");
        assert_eq!(rules.len(), 1);
        assert!(problems[0].contains("not a word in the language"));
    }
}
