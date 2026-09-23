//! The judge: the build. Green or red, no pattern. Errors come back as the toolchain
//! wrote them. This is the only subprocess in the loop, and it is the build.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Build {
    pub ok: bool,
    pub errors: String,
    pub wall_s: f32,
    pub ran: bool,
}

/// The build, for a Node project. Was `dotnet_build` until 22 Sep, when the Captain ruled
/// Sarge is Node and TypeScript only: *"we only support Node.js for the minute and
/// TypeScript"*. The machinery below is the same - spawn, read both pipes on a thread,
/// kill on timeout, keep the lines that say error - only the command changed.
pub fn build(dir: &Path, timeout: Duration) -> Build {
    let t0 = Instant::now();
    // npm on Windows is a .cmd, which CreateProcess will not spawn directly.
    let (prog, pre): (&str, &[&str]) = if cfg!(windows) { ("cmd", &["/c", "npm"]) } else { ("npm", &[]) };
    let mut child = match Command::new(prog)
        .args(pre)
        .args(["run", "--if-present", "build"])
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return Build { ok: false, errors: format!("npm could not start: {e}"), wall_s: 0.0, ran: false },
    };
    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();
    let reader = std::thread::spawn(move || {
        let mut out = String::new();
        if let Some(s) = stdout.as_mut() { let _ = s.read_to_string(&mut out); }
        let mut err = String::new();
        if let Some(s) = stderr.as_mut() { let _ = s.read_to_string(&mut err); }
        out + &err
    });
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break Some(s),
            Ok(None) if t0.elapsed() > timeout => {
                let _ = child.kill();
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(200)),
            Err(_) => break None,
        }
    };
    let text = reader.join().unwrap_or_default();
    let wall_s = t0.elapsed().as_secs_f32();
    let Some(status) = status else {
        return Build { ok: false, errors: "build took longer than the limit and was stopped".into(), wall_s, ran: true };
    };
    let errors: Vec<&str> = text.lines().map(str::trim).filter(|l| l.to_lowercase().contains("error")).collect();
    let mut errors = errors.join("\n");
    if errors.is_empty() {
        errors = text.chars().rev().take(2000).collect::<Vec<_>>().into_iter().rev().collect();
    }
    errors.truncate(4000);
    Build { ok: status.success(), errors, wall_s: (wall_s * 10.0).round() / 10.0, ran: true }
}
