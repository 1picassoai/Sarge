//! The sandbox: the one folder the model may write into.
//!
//! The model states a path on a comment line above a fenced block, and that is the
//! whole protocol - no tool calls, no schema for a small model to get wrong. The path is
//! untrusted input: `..`, an absolute path, a drive letter or a symlink out all fail
//! closed. Anything outside the sandbox is refused and reported, never written.
//!
//! Format parsing only. The fence and the `File:` line are formats, not meanings.

use std::path::{Component, Path, PathBuf};

use serde::Serialize;

pub const CODE_EXT: &[&str] = &["cs", "csproj", "json", "md", "sln", "props",
                                 "js", "mjs", "cjs", "jsx", "ts", "tsx", "html", "css"];   // Node series, 16 Sep

#[derive(Debug, Clone, Serialize)]
pub struct Written {
    pub path: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub path: String,
    pub bytes: u64,
}

/// Resolve a model-supplied path inside the sandbox, or refuse it.
pub fn safe_target(root: &Path, rel: &str) -> Option<PathBuf> {
    let rel = rel.trim().replace('\\', "/");
    let rel = rel.trim_start_matches('/');
    if rel.is_empty() || rel.starts_with("http") || rel.starts_with('~') || rel.contains(':') {
        return None;
    }
    // The escape check runs on the RELATIVE part, never on the joined path. The joined
    // path always carries the root's own `C:` prefix and root directory, so checking it
    // refused every legitimate file - measured: three good paths refused on the first
    // smoke of the Rust port.
    if Path::new(rel).components().any(|c| {
        matches!(c, Component::ParentDir | Component::Prefix(_) | Component::RootDir)
    }) {
        return None;
    }
    let p = root.join(rel);
    let ext = p.extension()?.to_string_lossy().to_lowercase();
    if !CODE_EXT.contains(&ext.as_str()) {
        return None;
    }
    // A symlink inside the box pointing out: the nearest existing parent must still
    // resolve under the box.
    let root_c = root.canonicalize().ok()?;
    let mut probe = p.parent();
    while let Some(dir) = probe {
        if dir.exists() {
            let dc = dir.canonicalize().ok()?;
            if !dc.starts_with(&root_c) {
                return None;
            }
            break;
        }
        probe = dir.parent();
    }
    Some(p)
}

/// Pull `// File: path` blocks (or a markdown heading naming a code file) out of the
/// answer and write them. Two accepted forms, as two alternatives, because merging them
/// into one clever pattern once broke the first and both arms wrote nothing.
pub fn write_files(root: &Path, text: &str) -> Vec<Written> {
    let re = regex::Regex::new(
        r"(?s)(?:^|\n)[ \t]*(?:(?://|#|<!--)[ \t]*File:[ \t]*[`*]{0,2}([^\s`*<>|]+?)[`*]{0,2}[ \t]*(?:-->)?|#{1,6}[ \t]*[^\n`]*?[`*]{1,2}([^\s`*<>|]+?\.(?:cs|csproj|json|sln|props))[`*]{1,2}[^\n]*)[ \t]*\n+```[a-zA-Z#+]*[ \t]*\n(.*?)```",
    )
    .expect("file-block pattern");
    let _ = std::fs::create_dir_all(root);
    let mut out = Vec::new();
    for m in re.captures_iter(text) {
        let rel = m.get(1).or_else(|| m.get(2)).map(|g| g.as_str()).unwrap_or("");
        let body = m.get(3).map(|g| g.as_str()).unwrap_or("");
        match safe_target(root, rel) {
            None => out.push(Written {
                path: rel.to_string(), ok: false, bytes: None, action: None,
                why: Some("refused - outside the sandbox or not a code file".into()),
            }),
            Some(target) => {
                if let Some(parent) = target.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let existed = target.exists();
                match std::fs::write(&target, body) {
                    Ok(()) => out.push(Written {
                        path: target.strip_prefix(root).unwrap_or(&target).to_string_lossy().replace('\\', "/"),
                        ok: true, bytes: Some(body.len()),
                        action: Some(if existed { "updated" } else { "created" }), why: None,
                    }),
                    Err(e) => out.push(Written {
                        path: rel.to_string(), ok: false, bytes: None, action: None, why: Some(e.to_string()),
                    }),
                }
            }
        }
    }
    out
}

/// What the build looks like now. Names and sizes, never contents.
pub fn tree(root: &Path) -> Vec<Entry> {
    let mut out = Vec::new();
    fn walk(dir: &Path, root: &Path, out: &mut Vec<Entry>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if p.is_dir() {
                if !matches!(name.as_str(), "bin" | "obj" | ".git" | "node_modules") {
                    walk(&p, root, out);
                }
            } else if let Some(ext) = p.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                if matches!(ext.as_str(), "cs" | "csproj" | "json" | "sln") {
                    let bytes = e.metadata().map(|m| m.len()).unwrap_or(0);
                    out.push(Entry {
                        path: p.strip_prefix(root).unwrap_or(&p).to_string_lossy().replace('\\', "/"),
                        bytes,
                    });
                }
            }
        }
    }
    walk(root, root, &mut out);
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}
