//! The book: a repo's rules, and the compare that runs before anything is added.
//!
//! Reloading the model's state is costly, so a new rule is compared with what the
//! book already holds before it lands:
//!
//!     exact     the id is already held                → known
//!     reworded  cosine to a held shape >= SAME_LAW    → same-law, nothing added
//!     new       appended to the book                  → learned
//!
//! Cosine is arithmetic, not a pattern. The threshold is a knob until the stickiness
//! matrix gives the number.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{load, Rule};

pub const EMBED_MODEL: &str = "text-embedding-3-small";

pub fn same_law_threshold() -> f32 {
    std::env::var("TUTOR_SAME_LAW").ok().and_then(|s| s.parse().ok()).unwrap_or(0.90)
}

pub fn held(path: &Path) -> Vec<Rule> {
    load(path).map(|(r, _)| r).unwrap_or_default()
}

fn vec_path(book: &Path) -> PathBuf {
    PathBuf::from(format!("{}.vec", book.display()))
}

#[derive(Deserialize)]
struct Stored { id: String, v: Vec<f32> }

pub fn embed(text: &str) -> Result<Vec<f32>, String> {
    if crate::tutor::vpn_up() {
        return Err("VPN SILENCE: the work VPN is up, so nothing leaves this machine - no embedding".into());
    }
    let key = crate::tutor::key()?;
    let resp: serde_json::Value = ureq::post("https://api.openai.com/v1/embeddings")
        .set("Authorization", &format!("Bearer {key}"))
        .send_json(serde_json::json!({ "model": EMBED_MODEL, "input": text }))
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;
    resp["data"][0]["embedding"]
        .as_array()
        .map(|a| a.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect())
        .ok_or_else(|| "no embedding in the reply".to_string())
}

pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb = b.iter().map(|y| y * y).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

/// One vector per held rule, kept beside the book so nothing is re-embedded.
fn vectors(book: &Path, rules: &[Rule]) -> Result<std::collections::HashMap<String, Vec<f32>>, String> {
    let vp = vec_path(book);
    let mut have = std::collections::HashMap::new();
    if let Ok(text) = std::fs::read_to_string(&vp) {
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            if let Ok(s) = serde_json::from_str::<Stored>(line) {
                have.insert(s.id, s.v);
            }
        }
    }
    let missing: Vec<&Rule> = rules.iter().filter(|r| !have.contains_key(&r.id)).collect();
    if !missing.is_empty() {
        let mut out = String::new();
        for r in missing {
            let v = embed(&r.shape)?;
            out.push_str(&serde_json::json!({ "id": r.id, "v": v }).to_string());
            out.push('\n');
            have.insert(r.id.clone(), v);
        }
        append(&vp, &out)?;
    }
    Ok(have)
}

fn append(path: &Path, text: &str) -> Result<(), String> {
    use std::io::Write;
    if let Some(p) = path.parent() {
        let _ = std::fs::create_dir_all(p);
    }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path).map_err(|e| e.to_string())?;
    f.write_all(text.as_bytes()).map_err(|e| e.to_string())
}

/// Compare, then add. Returns (status, nearest held id, best cosine seen).
pub fn learn(rule: &Rule, book: &Path) -> Result<(&'static str, Option<String>, f32), String> {
    let rules = held(book);
    if rules.iter().any(|r| r.id == rule.id) {
        return Ok(("known", Some(rule.id.clone()), 1.0));
    }
    let mine = embed(&rule.shape)?;
    let (mut best, mut near) = (0.0f32, None);
    if !rules.is_empty() {
        let vec = vectors(book, &rules)?;
        for r in &rules {
            if let Some(v) = vec.get(&r.id) {
                let c = cosine(&mine, v);
                if c > best {
                    best = c;
                    near = Some(r.id.clone());
                }
            }
        }
        if best >= same_law_threshold() {
            return Ok(("same-law", near, best));
        }
    }
    // A book in the language gets the rule as a block, inside the frame; a JSONL book
    // gets a line. The file decides, not the caller.
    let existing = std::fs::read_to_string(book).unwrap_or_default();
    let is_lang = crate::lang::is_sarge(existing.trim_start_matches('\u{feff}'))
        || book.extension().map_or(false, |e| e == "sarge" || e == "aiml");
    if is_lang {
        let body = existing.trim_start_matches('\u{feff}');
        let block = crate::lang::write_rule(rule);
        let new_text = match body.rfind(crate::lang::CLOSE) {
            Some(i) => format!("{}{}\n{}", &body[..i], block, &body[i..]),
            None => format!("{}\n\n{}\n{}\n", crate::lang::OPEN, block, crate::lang::CLOSE),
        };
        std::fs::write(book, new_text).map_err(|e| e.to_string())?;
    } else {
        append(book, &format!("{}\n", serde_json::to_string(rule).map_err(|e| e.to_string())?))?;
    }
    append(&vec_path(book), &format!("{}\n", serde_json::json!({ "id": rule.id, "v": mine })))?;
    Ok(("learned", near, best))
}
