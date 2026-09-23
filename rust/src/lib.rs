//! The handshake: what a small model is held to, decided before it runs.
//!
//! Four steps, no model call:
//!
//!   SELECT   only the rules this task touches. Instruction following in small models
//!            collapses past about five simultaneous constraints (arXiv 2608.12426).
//!   SHAPE    rules are stored already shaped - a checkable sentence, not a slogan.
//!            "archive-never-delete" was received and misread; "DELETE means set
//!            IsArchived = true, never call Remove" was followed.
//!   LINK     an allowance is printed WITH the prohibition it softens, never alone.
//!            An allowance standing on its own overrode a different rule's prohibition.
//!   CHECK    a rule may carry a regex over what the model wrote. A rule that cannot be
//!            checked is guidance, not a rule - and it says so.
//!
//! This is a library first and a binary second, because the point is to link it into
//! llama.cpp rather than shell out to it.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub mod vetted;
pub mod ffi;
pub mod sandbox;
pub mod judge;
pub mod book;
pub mod tutor;
pub mod lang;
pub mod verdict;

/// How many rules reach the model in one delivery. Five was right when a book held three
/// rules of its own; with a shipped stack book under it, five leaves out the rule that
/// decides the task (22 Sep: `no-await-on-a-sync-call` missed the cut on a node:sqlite
/// task, which is the exact fault that had cancelled a run that morning). Twelve is room
/// for the stack's rules AND the repo's own without the wall of thirty-eight that stalled
/// a model for 120 turns. The check has its own, larger cap - it judges one file, not a
/// whole task.
pub const MAX_RULES: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Check {
    pub files: String,
    pub must_not_match: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub topic: String,
    pub shape: String,
    #[serde(default)]
    pub allow: Option<String>,
    #[serde(default)]
    pub applies: Option<String>,
    /// A workflow lane. A rule tagged with one belongs to that lane and no other -
    /// topic matching cannot decide that, and got it wrong: a `pr` rule scored onto
    /// the `product` lane because its topic said "product adaptability".
    #[serde(default)]
    pub lane: Option<String>,
    #[serde(default)]
    pub check: Option<Check>,
    /// The tutor's form. A law that names no replacement does not hold - measured twice:
    /// bare "never" rules broke at every rule count, `instead` rules held. The shape is
    /// what the model reads; these two are what the tutor was made to write.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub never: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instead: Option<String>,
    /// The language's demonstrations (lang.rs): real lines that break the rule and real
    /// lines that obey it. What a next-token predictor follows; what the check asks about.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wrong: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub right: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
}

impl Rule {
    /// A rule with no check is guidance. Nothing will stop the model breaking it.
    pub fn is_enforceable(&self) -> bool {
        self.check.is_some()
    }
}

const STOP: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "if", "then", "of", "to", "in", "on", "at", "by",
    "for", "with", "from", "as", "is", "are", "was", "were", "be", "been", "it", "its", "do",
    "does", "did", "not", "no", "so", "such", "can", "could", "will", "would", "should",
    "have", "has", "had", "what", "how", "when", "where", "which", "who", "why", "this",
    "that", "these", "those", "you", "your", "i", "my", "we", "our", "add", "write",
    "create", "make", "new",
];

fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.len() > 1 && !STOP.contains(w))
        .map(|w| w.to_string())
        .collect()
}

/// Load a rulebook. One JSON object per line; a malformed line is reported, not fatal -
/// one bad rule must not cost you the other forty.
pub fn load(path: &Path) -> std::io::Result<(Vec<Rule>, Vec<String>)> {
    let text = std::fs::read_to_string(path)?;
    // A byte-order mark on line 1 made the first rule "malformed" and it was silently
    // skipped for days. 16 Sep. Editors on Windows add one without asking.
    let text = text.trim_start_matches('\u{feff}');
    // The language (lang.rs) if the file opens with its frame; JSONL otherwise, for the
    // books that predate it. Both load into the same Rule.
    if lang::is_sarge(text) {
        return Ok(lang::parse(text));
    }
    let mut rules = Vec::new();
    let mut problems = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Rule>(line) {
            Ok(r) => rules.push(r),
            Err(e) => problems.push(format!("line {}: {}", i + 1, e)),
        }
    }
    Ok((rules, problems))
}

pub fn for_scope<'a>(rules: &'a [Rule], scope: Option<&str>) -> Vec<&'a Rule> {
    rules
        .iter()
        .filter(|r| match (&r.applies, scope) {
            (Some(a), Some(s)) => a.eq_ignore_ascii_case(s),
            (Some(_), None) => false,
            (None, _) => true,
        })
        .collect()
}

/// SELECT. BM25 over each rule's topic words, capped at `k`, under a priority order.
///
/// Lanes converge - two lanes can work the same subject matter, and that is the normal
/// case rather than the edge case. Vinn's ruling (13 Sep): treat it like a roundabout.
/// You do not ban traffic from the other roads, you say who has right of way.
///
///   1. the lane's OWN rules   - written from that lane's own mistakes, always first
///   2. UNTAGGED rules         - shared defaults, everyone gets them
///   3. another lane's rules   - never. That is the bleed.
///
/// Under the cap, a lane's own rules fill the slots before any default can, so a `pr`
/// rule can never reach the `product` lane and the `pr` lane is never left with nothing.
pub fn select<'a>(rules: &[&'a Rule], task: &str, lane: Option<&str>, k: usize) -> Vec<&'a Rule> {
    // Priority 3 is enforced here, before scoring: a foreign lane's rules never enter
    // the pool at all. Measured before this existed - a `pr` rule whose topic read
    // "product adaptability" scored onto the product lane, and pr got nothing.
    let pool: Vec<&Rule> = match lane {
        Some(l) => rules
            .iter()
            .filter(|r| r.lane.as_deref().map_or(true, |rl| rl == l))
            .copied()
            .collect(),
        None => rules.to_vec(),
    };
    let rules: &[&Rule] = &pool;

    let docs: Vec<Vec<String>> = rules.iter().map(|r| words(&r.topic)).collect();
    let n = docs.len() as f64;
    if n == 0.0 {
        return Vec::new();
    }
    let avgdl = docs.iter().map(|d| d.len()).sum::<usize>() as f64 / n;

    let mut df: HashMap<&str, f64> = HashMap::new();
    for d in &docs {
        let mut seen: Vec<&str> = d.iter().map(|s| s.as_str()).collect();
        seen.sort_unstable();
        seen.dedup();
        for w in seen {
            *df.entry(w).or_insert(0.0) += 1.0;
        }
    }

    let q = words(task);
    let (k1, b) = (1.5_f64, 0.75_f64);
    let mut scored: Vec<(f64, &'a Rule)> = rules
        .iter()
        .zip(&docs)
        .map(|(r, d)| {
            let dl = d.len() as f64;
            let mut s = 0.0;
            for w in &q {
                let f = d.iter().filter(|x| *x == w).count() as f64;
                if f > 0.0 {
                    let c = df.get(w.as_str()).copied().unwrap_or(0.0);
                    let idf = (1.0 + (n - c + 0.5) / (c + 0.5)).ln();
                    s += idf * (f * (k1 + 1.0)) / (f + k1 * (1.0 - b + b * dl / avgdl));
                }
            }
            (s, *r)
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Priorities 1 and 2. Own-lane rules take the slots first whatever they scored -
    // they came from this lane's own mistake, so a topic-word miss is not evidence
    // against them. Defaults then fill what is left, best-scoring first.
    let mut chosen: Vec<&Rule> = Vec::new();

    if let Some(l) = lane {
        // Own lane, in score order, all of them before any default.
        for (_, r) in scored.iter().filter(|(_, r)| r.lane.as_deref() == Some(l)) {
            if chosen.len() < k {
                chosen.push(*r);
            }
        }
    }

    // Untagged defaults, best score first, only what the lane's own rules left free.
    for (s, r) in scored.iter() {
        if chosen.len() >= k {
            break;
        }
        if r.lane.is_none() && *s > 0.0 && !chosen.iter().any(|c| c.id == r.id) {
            chosen.push(*r);
        }
    }

    chosen
}

/// SHAPE + LINK. The block that goes in front of the model. An allowance is printed on
/// the same line as its prohibition, never as a separate rule.
pub fn render(chosen: &[&Rule]) -> String {
    if chosen.is_empty() {
        return String::new();
    }
    // The block the model reads IS the language, framed - the Captain's ruling, 16 Sep:
    // the rules live inside the syntax. Delivered as text at the tail of the task today
    // (measured to bind there); the same block is what the organ will cache.
    let mut out = String::from("## RULES FOR THIS TASK\n\n");
    out.push_str("Every rule below is checked on what you write before you may say done. ");
    out.push_str("`wrong` is a line you must never write; `right` is the line to write instead.\n\n");
    out.push_str(&lang::write_book(chosen));
    out
}

#[derive(Debug)]
pub struct Hit {
    pub rule: String,
    pub file: String,
    pub line: usize,
    pub matched: String,
    pub shape: String,
}

/// CHECK. Run each chosen rule's regex over the files it names. A rule with no check is
/// skipped and counted, so "checks passed" never quietly means "nothing was checked".
///
/// Returns (hits, unenforceable, unchecked). `unchecked` counts rules whose glob matched
/// no file at all - measured: a rule globbed `**/*.fsharp` reported "CHECKS PASSED
/// (1 rule checked, 0 unenforceable)" over a file that plainly violated it. The
/// unenforceable counter catches a missing check or a bad regex, not a glob typo, and a
/// rule that opened no files has not been checked whatever its regex says.
pub fn check(root: &Path, chosen: &[&Rule]) -> (Vec<Hit>, usize, Vec<String>) {
    let mut hits = Vec::new();
    let mut unenforceable = 0;
    let mut unchecked = Vec::new();

    for r in chosen {
        let Some(c) = &r.check else {
            unenforceable += 1;
            continue;
        };
        let Ok(re) = regex::Regex::new(&c.must_not_match) else {
            unenforceable += 1;
            continue;
        };
        let mut examined = 0usize;
        let pattern = c.files.trim_start_matches("**/");
        for entry in walk(root) {
            let rel = entry
                .strip_prefix(root)
                .unwrap_or(&entry)
                .to_string_lossy()
                .replace('\\', "/");
            if rel.split('/').any(|p| matches!(p, "bin" | "obj" | ".git" | "node_modules")) {
                continue;
            }
            let name = rel.rsplit('/').next().unwrap_or(&rel);
            let matches_glob = glob::Pattern::new(pattern)
                .map(|g| g.matches(name) || g.matches(&rel))
                .unwrap_or(false);
            if !matches_glob {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&entry) else { continue };
            examined += 1;
            if let Some(m) = re.find(&text) {
                let line = text[..m.start()].matches('\n').count() + 1;
                hits.push(Hit {
                    rule: r.id.clone(),
                    file: rel,
                    line,
                    matched: m.as_str().chars().take(80).collect(),
                    shape: r.shape.clone(),
                });
            }
        }
        if examined == 0 {
            unchecked.push(r.id.clone());
        }
    }
    (hits, unenforceable, unchecked)
}

fn walk(root: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push(p);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(id: &str, topic: &str, lane: Option<&str>) -> Rule {
        Rule {
            id: id.into(),
            kind: "prohibit".into(),
            topic: topic.into(),
            shape: format!("shape for {id}"),
            allow: None,
            applies: None,
            lane: lane.map(|s| s.into()),
            check: None,
            never: None, instead: None, wrong: Vec::new(), right: Vec::new(), since: None,
        }
    }

    #[test]
    fn selects_by_topic() {
        let rs = vec![
            rule("delete", "delete remove customer archive", None),
            rule("orders", "order line quantity price", None),
        ];
        let refs: Vec<&Rule> = rs.iter().collect();
        let got = select(&refs, "add a delete customer endpoint", None, 5);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].id, "delete");
    }

    #[test]
    fn a_lanes_own_rule_always_applies() {
        // The bug this exists to prevent: a `pr` rule whose topic said "product
        // adaptability" scored onto the `product` lane, and the pr lane got nothing.
        let rs = vec![rule("pr-promise", "product adaptability promises", Some("pr"))];
        let refs: Vec<&Rule> = rs.iter().collect();
        let got = select(&refs, "pr AI developer tooling", Some("pr"), 5);
        assert_eq!(got.len(), 1, "the pr lane must receive its own rule");
        let other = select(&refs, "product AI developer tooling", Some("product"), 5);
        assert!(
            other.iter().all(|r| r.lane.as_deref() != Some("pr")),
            "a pr rule must not reach the product lane"
        );
    }

    #[test]
    fn the_roundabout_has_a_priority_order() {
        // Vinn's ruling, 13 Sep: lanes converge, so this is not a ban, it is right of
        // way. Own lane first, untagged defaults second, another lane's rules never -
        // and the order must hold even when the foreign rule scores higher on topic.
        let rs = vec![
            rule("mine", "unrelated words entirely", Some("alpha")),
            rule("theirs", "delete customer archive remove", Some("beta")),
            rule("shared", "delete customer archive", None),
        ];
        let refs: Vec<&Rule> = rs.iter().collect();
        let got = select(&refs, "delete a customer", Some("alpha"), 5);
        let ids: Vec<&str> = got.iter().map(|r| r.id.as_str()).collect();

        assert_eq!(ids.first(), Some(&"mine"),
                   "the lane's own rule comes first even scoring zero on the task");
        assert!(ids.contains(&"shared"), "untagged defaults still apply to the lane");
        assert!(!ids.contains(&"theirs"),
                "another lane's rule must never enter, however well it scores");
    }

    #[test]
    fn own_lane_rules_take_the_slots_before_defaults() {
        // Under the cap, priority 1 must crowd out priority 2 rather than compete
        // with it on score. Three own-lane rules and a cap of 3 leaves no room for
        // a default, however strongly the default matches the task.
        let rs = vec![
            rule("own-a", "nothing relevant here", Some("alpha")),
            rule("own-b", "nothing relevant either", Some("alpha")),
            rule("own-c", "still nothing relevant", Some("alpha")),
            rule("default", "delete customer archive remove", None),
        ];
        let refs: Vec<&Rule> = rs.iter().collect();
        let got = select(&refs, "delete a customer", Some("alpha"), 3);
        assert_eq!(got.len(), 3);
        assert!(got.iter().all(|r| r.lane.as_deref() == Some("alpha")),
                "own-lane rules fill the cap before any default is considered");
    }

    #[test]
    fn caps_at_five() {
        let rs: Vec<Rule> = (0..10)
            .map(|i| rule(&format!("r{i}"), "delete customer archive remove", None))
            .collect();
        let refs: Vec<&Rule> = rs.iter().collect();
        assert!(select(&refs, "delete customer", None, MAX_RULES).len() <= MAX_RULES);
    }

    #[test]
    fn allowance_travels_with_its_prohibition() {
        let mut r = rule("offbox", "network send remote", None);
        r.shape = "Do not send data off this machine.".into();
        r.allow = Some("Calling localhost is fine.".into());
        let rs = vec![r];
        let refs: Vec<&Rule> = rs.iter().collect();
        let out = render(&refs);
        let line = out.lines().find(|l| l.starts_with("- ")).unwrap();
        assert!(line.contains("off this machine") && line.contains("localhost"));
    }

    #[test]
    fn a_rule_without_a_check_is_not_enforceable() {
        assert!(!rule("x", "y", None).is_enforceable());
    }

    #[test]
    fn a_glob_that_matches_nothing_is_not_a_pass() {
        // The hole this exists to close: a rule globbed `**/*.fsharp` reported
        // "CHECKS PASSED (1 rule checked, 0 unenforceable)" over a file containing the
        // exact text it prohibits. No file was opened, so nothing was checked - but the
        // run reported success, which is worse than reporting a failure.
        let dir = std::env::temp_dir().join("handshake-glob-test");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Program.cs"), "db.Customers.Remove(c);\n").unwrap();

        let mut r = rule("ghost", "delete customer", None);
        r.check = Some(Check {
            files: "**/*.fsharp".into(),
            must_not_match: "Customers\\.Remove\\(".into(),
        });
        let rs = vec![r];
        let refs: Vec<&Rule> = rs.iter().collect();
        let (hits, unenforceable, unchecked) = check(&dir, &refs);

        assert!(hits.is_empty(), "the glob matches no file, so there is nothing to hit");
        assert_eq!(unenforceable, 0, "the rule has a check and a valid regex");
        assert_eq!(unchecked, vec!["ghost".to_string()],
                   "a rule that opened no file must be reported, never folded into a pass");

        // And the opposite direction, so the counter cannot simply always fire.
        let mut ok = rule("real", "delete customer", None);
        ok.check = Some(Check {
            files: "**/*.cs".into(),
            must_not_match: "Customers\\.Remove\\(".into(),
        });
        let rs2 = vec![ok];
        let refs2: Vec<&Rule> = rs2.iter().collect();
        let (hits2, _, unchecked2) = check(&dir, &refs2);
        assert_eq!(hits2.len(), 1, "the same file must fail a rule whose glob matches it");
        assert!(unchecked2.is_empty(), "a rule that opened a file has been checked");

        std::fs::remove_dir_all(&dir).ok();
    }
}
