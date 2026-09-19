//! Reading VETTED.md, and noticing when it changes.
//!
//! VETTED.md is the artefact: a closed, portable document that says which rules are
//! safe to write into the model's KV cache and which are only safe to show it. The
//! manager writes it, a human can read it and strike a line, and this module reads it.
//!
//! It is markdown on purpose. A human has to be able to open the thing that decides
//! what a model cannot argue with.
//!
//! The delta watch is a hash. Boot, hash the file, keep the digest. On each check,
//! hash again - if it moved, the injected state is stale and the engine reloads. That
//! is what makes the loop run 24/7 rather than being a build step.

use std::path::{Path, PathBuf};

/// Where a rule is allowed to go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Into the KV cache. The model cannot argue with it.
    Inject,
    /// System message only. The model reads it and may still push back.
    Ground,
    /// Never injected. It fights the training prior and loses.
    Rewrite,
}

impl Tier {
    fn parse(s: &str) -> Option<Tier> {
        match s {
            "INJECT" => Some(Tier::Inject),
            "GROUND" => Some(Tier::Ground),
            "REWRITE" => Some(Tier::Rewrite),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VettedRule {
    pub id: String,
    pub tier: Tier,
    pub shape: String,
}

#[derive(Debug, Clone)]
pub struct Vetted {
    pub path: PathBuf,
    pub digest: u64,
    pub rules: Vec<VettedRule>,
}

impl Vetted {
    /// Rules cleared for the KV cache.
    pub fn inject(&self) -> impl Iterator<Item = &VettedRule> {
        self.rules.iter().filter(|r| r.tier == Tier::Inject)
    }

    /// Rules that may only be shown, never built in.
    pub fn ground(&self) -> impl Iterator<Item = &VettedRule> {
        self.rules.iter().filter(|r| r.tier == Tier::Ground)
    }

    /// Has the file changed since this was read? Cheap enough to call on every turn.
    pub fn is_stale(&self) -> bool {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => hash(&text) != self.digest,
            // Unreadable is not stale. A file being rewritten mid-read would otherwise
            // trigger a reload of nothing, and a deleted file must not silently empty
            // the ruleset - keep what we have and let the caller notice.
            Err(_) => false,
        }
    }

    pub fn reload(&mut self) -> std::io::Result<bool> {
        let fresh = load(&self.path)?;
        let changed = fresh.digest != self.digest;
        *self = fresh;
        Ok(changed)
    }
}

/// FNV-1a. Not cryptographic - this only has to notice that a file moved.
fn hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// Parse VETTED.md. The structure it relies on is written by `tools/vet.py`:
///
/// ```text
/// ## INJECT — into the blood
/// ### `rule-id` — 9/9 (100%)
/// > the shape, one line
/// ```
///
/// A malformed section is skipped rather than fatal. One bad heading must not cost
/// you the other forty rules - the same rule the JSONL loader follows.
pub fn load(path: &Path) -> std::io::Result<Vetted> {
    let text = std::fs::read_to_string(path)?;
    Ok(parse(&text, path))
}

/// Same, from text already in hand. The compiled-in core goes through here: the
/// VETTED.md is embedded at `cargo build` and there is no file to read at runtime.
pub fn parse(text: &str, path: &Path) -> Vetted {
    let mut rules = Vec::new();
    let mut tier: Option<Tier> = None;
    let mut pending: Option<String> = None;

    for line in text.lines() {
        let t = line.trim();

        // harness.py core.md: "- **[3/4]** the law". Every line in that file has
        // already survived the compaction count, so it is INJECT by construction.
        if let Some(rest) = t.strip_prefix("- **[") {
            if let Some((count, shape)) = rest.split_once("]** ") {
                let shape = shape.trim();
                if !shape.is_empty() {
                    rules.push(VettedRule {
                        id: format!("core-{}", count.replace('/', "of")),
                        tier: Tier::Inject,
                        shape: shape.to_string(),
                    });
                }
            }
            continue;
        }

        if let Some(rest) = t.strip_prefix("## ") {
            // "## INJECT — into the blood" -> INJECT
            tier = rest.split_whitespace().next().and_then(Tier::parse);
            pending = None;
            continue;
        }

        if let Some(rest) = t.strip_prefix("### ") {
            // "### `rule-id` — 9/9 (100%)"
            pending = rest
                .split('`')
                .nth(1)
                .map(str::to_string)
                .filter(|s| !s.is_empty());
            continue;
        }

        if let (Some(id), Some(tier), Some(shape)) = (&pending, tier, t.strip_prefix("> ")) {
            if !shape.trim().is_empty() {
                rules.push(VettedRule {
                    id: id.clone(),
                    tier,
                    shape: shape.trim().to_string(),
                });
                pending = None;
            }
        }
    }

    Vetted {
        path: path.to_path_buf(),
        digest: hash(text),
        rules,
    }
}

/// The block written into the context for INJECT rules. Separate from `render` in the
/// library because these are not advice - they are stated as fact about the codebase.
pub fn render_injected(v: &Vetted) -> String {
    let injected: Vec<&VettedRule> = v.inject().collect();
    if injected.is_empty() {
        return String::new();
    }
    let mut out = String::from("## HOW THIS CODEBASE WORKS\n\n");
    out.push_str("These are not preferences. They are how the code in front of you is written.\n\n");
    for r in injected {
        out.push_str("- ");
        out.push_str(&r.shape);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write(name: &str, body: &str) -> PathBuf {
        let p = std::env::temp_dir().join(name);
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        p
    }

    const DOC: &str = "\
# VETTED.md

## INJECT — into the blood

### `archive-not-delete` — 3/3 (100%)

> A customer is never deleted. Set IsArchived = true.

- **Why:** held 3/3 across 3 models

### `no-controllers` — 9/9 (100%)

> Map endpoints with app.MapGet. Never write an [ApiController] class.

## GROUND — system message only

### `no-console-write` — 8/9 (89%)

> Never call Console.WriteLine. Use the injected ILogger.

## REWRITE — never inject

### `no-var` — 8/18 (44%)

> Declare locals with their explicit type. Never use var.
";

    #[test]
    fn reads_the_three_tiers() {
        let p = write("vetted-tiers.md", DOC);
        let v = load(&p).unwrap();
        assert_eq!(v.rules.len(), 4);
        assert_eq!(v.inject().count(), 2, "two rules cleared for the cache");
        assert_eq!(v.ground().count(), 1);
        assert!(v.rules.iter().any(|r| r.id == "no-var" && r.tier == Tier::Rewrite));
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn a_rewrite_rule_never_reaches_the_blood() {
        // The whole point of the file. no-var broke in all three models; if it can
        // reach render_injected the vetting was decorative.
        let p = write("vetted-blood.md", DOC);
        let v = load(&p).unwrap();
        let block = render_injected(&v);
        assert!(block.contains("IsArchived"), "an INJECT rule must be there");
        assert!(!block.contains("Never use var"), "a REWRITE rule must never be injected");
        assert!(!block.contains("Console.WriteLine"), "a GROUND rule is not injected either");
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn notices_the_file_changing() {
        let p = write("vetted-delta.md", DOC);
        let v = load(&p).unwrap();
        assert!(!v.is_stale(), "unchanged file is not stale");

        let edited = DOC.replace("### `no-var` — 8/18 (44%)", "### `no-var` — 18/18 (100%)");
        std::fs::write(&p, edited).unwrap();
        assert!(v.is_stale(), "a changed file must be seen");

        let mut v2 = v.clone();
        assert!(v2.reload().unwrap(), "reload reports that it changed");
        assert!(!v2.is_stale());
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn a_missing_file_does_not_empty_the_ruleset() {
        // Deleting VETTED.md must not silently unlock the model. Better to keep the
        // last known rules and let the caller decide.
        let p = write("vetted-gone.md", DOC);
        let v = load(&p).unwrap();
        std::fs::remove_file(&p).ok();
        assert!(!v.is_stale(), "an unreadable file is not a change");
        assert_eq!(v.inject().count(), 2, "the rules we had are still held");
    }
}
