//! The tutor: the frontier, called once per failure, never per task.
//!
//! It reads the task, the rules the model was given, the names of the files already in
//! the project, what the model wrote this turn, and what the compiler said. Never the
//! repo. It writes a correction addressed to the model and a rule as a form:
//!
//!     never    what the model reached for
//!     instead  the replacement - always, or it is not a rule
//!     allow    the one case where the never does not apply, or "none"
//!     applies  the repo or lane this was learned in
//!     topic    the words a task about this would contain
//!
//! No regex, no check field. What judges the rule afterwards is the next run.

use std::path::Path;

use serde::Serialize;

use crate::Rule;

pub const INSTRUCTIONS: &str = r#"You are the tutor for a small local coding model. You are a senior engineer.
You are shown a task, the rules the model was given, the files already in the project,
what the model wrote this turn, and what went wrong. You never see the repository. Do two things.

0. FIRST, before anything else: WHAT CAN THIS CODE DESTROY? Read what was written for any
   line that drops, deletes, truncates, recreates or overwrites data, or that would do so
   in production - an unguarded EnsureDeleted, a DROP TABLE, a delete without an archive,
   a migration that loses a column. A green build with such a line is not done. If you
   find one, it is the correction AND the rule, ahead of any style point. (16 Sep: a run
   went green dropping its database on every start in every environment, and the review
   asked only what convention it broke. Theo's finding.)

1. CORRECTION - addressed to the local model, in as few lines as the fault needs. Say what
   it did, why that is wrong, and exactly what to do instead. No praise, no padding. If a
   file the model needs already exists in the project, say so by name rather than asking
   for it to be written again.

2. RULE - one lesson, written in the rule language inside a ```sarge fence, exactly this shape:

```sarge
rule <your-own-id-here>
  do     <the replacement, one sentence - required, never empty>
  never  <what the model reached for, one sentence>
  wrong  | <one real line of code that breaks the rule, taken from what was written>
  right  | <one real line of code that obeys it>
  allow  <the one case where the never does not apply - omit the line if none>
end
```

Replace every <...> with your own words - the id is a short kebab-case name YOU choose
that names the lesson, for example `json-body-parser-before-routes`. A rule whose id is
copied from this template is garbage and is thrown away. The nine words are the whole
language: rule do never wrong right allow since end, and the frame. `do` comes first.
`wrong` and `right` are REAL CODE after the bar, one line each, at least one of each - a
rule without a demonstration is a slogan. Two spaces of indent inside the rule. Nothing
else in the fence.

If nothing went wrong, say so in the correction and write no rule.
Never include file contents, secrets, customer names or company names in a rule."#;

/// Today as YYYY-MM-DD, for `since`. No chrono: seconds since the epoch, civil date by
/// the usual arithmetic.
pub fn today() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs()).unwrap_or(0) as i64;
    let days = secs / 86_400;
    // Howard Hinnant's days-to-civil.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// ONE tutor. The Captain's ruling, 16 Sep 19:55: it is Claude or it is ChatGPT, never
/// both. Claude - because on 16 Sep gpt-4.1 taught the student a JSON-import syntax Node
/// had removed, and the student obeyed. The teacher's cutoff was the ceiling of the loop.
/// Sonnet for the tutor (called rarely, quality is the game); Haiku is for the check later.
pub fn model() -> String {
    std::env::var("TUTOR_MODEL").unwrap_or_else(|_| "claude-sonnet-5".to_string())
}

pub fn is_claude() -> bool {
    model().starts_with("claude")
}

/// USD per million tokens (in, out) for the tutor model, so the COST line on the page is
/// auditable. Overridable by TUTOR_IN_PER_M / TUTOR_OUT_PER_M. The Claude defaults are the
/// Sonnet-class list price as last seen - CONFIRM against the provider's page before any
/// figure goes outward; that is Gareth's number, not this file's.
pub fn rates() -> (f64, f64) {
    let env = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<f64>().ok());
    let (i, o) = if is_claude() { (3.00, 15.00) } else { (IN_PER_M, OUT_PER_M) };
    (env("TUTOR_IN_PER_M").unwrap_or(i), env("TUTOR_OUT_PER_M").unwrap_or(o))
}

pub fn anthropic_key() -> Result<String, String> {
    if let Ok(k) = std::env::var("ANTHROPIC_API_KEY") {
        if !k.trim().is_empty() {
            return Ok(k.trim().to_string());
        }
    }
    // Outside the repo, always: %USERPROFILE%\.sarge\anthropic-key.txt unless ANTHROPIC_KEY_FILE says otherwise.
    let file = std::env::var("ANTHROPIC_KEY_FILE").unwrap_or_else(|_| {
        let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_default();
        format!("{home}/.sarge/anthropic-key.txt")
    });
    std::fs::read_to_string(&file)
        .map(|s| s.trim().to_string())
        .map_err(|_| format!("no Anthropic key: set ANTHROPIC_API_KEY or put it in {file}"))
}

pub fn key() -> Result<String, String> {
    if let Ok(k) = std::env::var("OPENAI_API_KEY") {
        if !k.trim().is_empty() {
            return Ok(k.trim().to_string());
        }
    }
    let file = std::env::var("OPENAI_KEY_FILE").unwrap_or_else(|_| {
        let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_default();
        format!("{home}/.sarge/openai-key.txt")
    });
    std::fs::read_to_string(&file)
        .map(|s| s.trim().to_string())
        .map_err(|_| format!("no OpenAI key: set OPENAI_API_KEY or put it in {file}"))
}

#[derive(Debug, Clone, Serialize)]
pub struct Lesson {
    pub correction: String,
    pub rule: Option<Rule>,
    pub status: &'static str,
    pub near: Option<String>,
    pub cosine: f32,
    pub model: String,
    pub backend: &'static str,
    /// What this lesson actually cost, from the provider's own usage block. Not an
    /// estimate - the number on the screen has to be the number on the bill.
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub usd: f64,
}

/// gpt-4.1, USD per million tokens. Stated here so the cost on the page is auditable
/// rather than a figure that appears from nowhere.
const IN_PER_M: f64 = 2.00;
const OUT_PER_M: f64 = 8.00;

pub fn prompt(task: &str, answer: &str, rules: &[String], wrong: &str, applies: Option<&str>, files: &[String]) -> String {
    let listing = if files.is_empty() {
        "  (none - this is the first task)".to_string()
    } else {
        files.iter().map(|f| format!("  {f}")).collect::<Vec<_>>().join("\n")
    };
    let shown = if rules.is_empty() {
        "(none)".to_string()
    } else {
        rules.iter().map(|r| format!("- {r}")).collect::<Vec<_>>().join("\n")
    };
    let answer: String = answer.chars().take(6000).collect();
    format!(
        "TASK\n{task}\n\nAPPLIES TO\n{}\n\nFILES ALREADY IN THE PROJECT (from earlier turns; the model wrote them, \
         and it only re-emits the ones it changes)\n{listing}\n\nRULES THE MODEL WAS GIVEN\n{shown}\n\n\
         WHAT THE MODEL WROTE THIS TURN\n{answer}\n\nWHAT WENT WRONG\n{}",
        applies.unwrap_or("unknown"),
        if wrong.is_empty() { "(not stated - judge it yourself)" } else { wrong }
    )
}

pub fn ask(text: &str) -> Result<(String, u64, u64), String> {
    ask_with(INSTRUCTIONS, text)
}

/// One call, one system prompt. Teaching and answering are different jobs with different
/// instructions, but they are the same request and the same usage accounting.
/// VPN SILENCE - the Captain's law, now in the code. Nothing leaves this machine while
/// the work VPN is up. 16 Sep: a run attempted two tutor calls while GlobalProtect was
/// connected; they failed at TLS, but they were attempted. The adapter is named in
/// ipconfig; if it is there, the tutor does not exist.
pub fn vpn_up() -> bool {
    std::process::Command::new("ipconfig").arg("/all").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("PANGP"))
        .unwrap_or(false)
}

pub fn ask_with(system: &str, text: &str) -> Result<(String, u64, u64), String> {
    if vpn_up() {
        return Err("VPN SILENCE: the work VPN is up, so nothing leaves this machine - the tutor is not called".into());
    }
    if is_claude() {
        // The Anthropic Messages API. Same job, same accounting: the usage block is the
        // provider's own count, never our estimate.
        let key = anthropic_key()?;
        let resp: serde_json::Value = ureq::post("https://api.anthropic.com/v1/messages")
            .set("x-api-key", &key)
            .set("anthropic-version", "2023-06-01")
            .set("content-type", "application/json")
            .send_json(serde_json::json!({
                // No temperature: the Claude 5 models reject it as deprecated (16 Sep, first call).
                // 6000, not 1200: Sonnet 5 thinks before it answers, and twice on 17 Sep it spent
                // the whole 1200 on the thinking block and returned no text - two lessons lost.
                // Only the text parts are read; the thinking is the model's own.
                "model": model(), "max_tokens": 6000,
                "system": system,
                "messages": [ { "role": "user", "content": text } ]
            }))
            .map_err(|e| match e {
                ureq::Error::Status(code, r) => format!("anthropic {code}: {}", r.into_string().unwrap_or_default().chars().take(300).collect::<String>()),
                other => other.to_string(),
            })?
            .into_json()
            .map_err(|e| e.to_string())?;
        let pt = resp["usage"]["input_tokens"].as_u64().unwrap_or(0);
        let ct = resp["usage"]["output_tokens"].as_u64().unwrap_or(0);
        let content = resp["content"].as_array()
            .map(|parts| parts.iter().filter_map(|p| p["text"].as_str()).collect::<Vec<_>>().join(""))
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                // Twice on 17 Sep the teach came back with no text and the lesson was lost.
                // Say what the provider actually sent, so the next one is diagnosable.
                let raw = resp.to_string();
                format!("no content in the tutor's reply (stop_reason={}, raw={})",
                        resp["stop_reason"].as_str().unwrap_or("?"),
                        raw.chars().take(400).collect::<String>())
            })?;
        return Ok((content, pt, ct));
    }
    let key = key()?;
    let resp: serde_json::Value = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Authorization", &format!("Bearer {key}"))
        .send_json(serde_json::json!({
            "model": model(), "temperature": 0,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": text }
            ]
        }))
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;
    // The usage block is the provider's own count. Never our estimate of it.
    let pt = resp["usage"]["prompt_tokens"].as_u64().unwrap_or(0);
    let ct = resp["usage"]["completion_tokens"].as_u64().unwrap_or(0);
    let content = resp["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "no content in the tutor's reply".to_string())?;
    Ok((content, pt, ct))
}

/// The rule is the first fenced JSON object in the reply. The fence's label is not
/// trusted - the tutor wrote ```rule once and ``` the next time, same rule inside.
pub fn parse_rule(reply: &str) -> (String, Option<Rule>) {
    // The language first: a fenced block that opens with `rule `. 16 Sep.
    let lang_re = regex::Regex::new(r"(?s)```[a-zA-Z]*[ \t]*\n(\s*rule .*?\n\s*end)\s*\n?```").expect("sarge fence");
    if let Some(m) = lang_re.captures(reply) {
        let raw = m.get(1).map_or("", |g| g.as_str());
        let framed = format!("{}\n{}\n{}\n", crate::lang::OPEN, raw, crate::lang::CLOSE);
        let (mut rules, problems) = crate::lang::parse(&framed);
        for p in &problems { eprintln!("tutor      rule block: {p}"); }
        if let Some(mut r) = rules.pop() {
            r.since = Some(format!("{} tutor", today()));
            let end = m.get(0).map_or(reply.len(), |g| g.start());
            let mut correction = reply[..end].trim().to_string();
            if let Some(s) = correction.strip_suffix("RULE") { correction = s.trim_end().to_string(); }
            return (correction, Some(r));
        }
        eprintln!("tutor      a rule block was written but did not parse - the lesson was LOST");
    }
    let re = regex::Regex::new(r"(?s)```[a-zA-Z]*[ \t]*\n(\s*\{.*?\})\s*```").expect("rule fence");
    let mut correction_end = reply.len();
    let mut rule = None;
    for m in re.captures_iter(reply) {
        let raw = m.get(1).map_or("", |g| g.as_str());
        // A rule that will not parse is a LOST LESSON, and reporting it as "nothing to
        // teach" is a lie. One missing comma threw away a correct rule on 15 Sep.
        if serde_json::from_str::<serde_json::Value>(raw).is_err() {
            eprintln!("tutor      a rule block was written but is not valid JSON - the lesson was LOST");
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) else { continue };
        correction_end = m.get(0).map_or(reply.len(), |g| g.start());
        let field = |k: &str| v[k].as_str().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
        // An empty required field is a format error, not a judgement.
        let (Some(id), Some(kind), Some(topic), Some(never), Some(instead), Some(shape)) =
            (field("id"), field("kind"), field("topic"), field("never"), field("instead"), field("shape"))
        else { break };
        let allow = field("allow").filter(|a| a.to_lowercase() != "none");
        let applies = field("applies").filter(|a| a.to_lowercase() != "any");
        rule = Some(Rule { id, kind, topic, shape, allow, applies, lane: None, check: None,
                           never: Some(never), instead: Some(instead),
                           wrong: Vec::new(), right: Vec::new(), since: None });
        break;
    }
    let mut correction = reply[..correction_end].trim().to_string();
    if let Some(stripped) = correction.strip_suffix("RULE") {
        correction = stripped.trim_end().to_string();
    }
    (correction, rule)
}

/// THE STUDENT'S QUESTION. Vinn's design, 15 Sep 2026. The tutor answers what was asked
/// instead of inferring it from wreckage - and the answer is for the student NOW, not a
/// rule for later. Short, direct, no lecture: a small model cannot use an essay.
pub const ANSWER_INSTRUCTIONS: &str = r#"You are a senior engineer answering a junior developer
who is stuck on a specific error. You are shown the task, the error they are looking at, and
their question.

Answer in at most six lines. Say what is wrong and exactly what to write instead. Show the
corrected line or lines if that is the clearest answer. No preamble, no encouragement, no
explanation of things they did not ask about. They are a small model with a short memory -
give them the fix, not a lesson."#;

pub fn answer(task: &str, failure: &str, question: &str) -> Result<(String, u64, u64), String> {
    let text = format!("TASK\n{task}\n\nTHE ERROR THEY ARE LOOKING AT\n{failure}\n\nTHEIR QUESTION\n{question}");
    ask_with(ANSWER_INSTRUCTIONS, &text)
}

pub fn teach(task: &str, answer: &str, rules: &[String], wrong: &str, files: &[String],
             applies: Option<&str>, book: &Path) -> Result<Lesson, String> {
    let (reply, pt, ct) = ask(&prompt(task, answer, rules, wrong, applies, files))?;
    let (correction, rule) = parse_rule(&reply);
    let (status, near, cosine) = match &rule {
        Some(r) => crate::book::learn(r, book)?,
        None => ("no-rule", None, 0.0),
    };
    let (inr, outr) = rates();
    let usd = pt as f64 / 1_000_000.0 * inr + ct as f64 / 1_000_000.0 * outr;
    Ok(Lesson { correction, rule, status, near, cosine, model: model(), backend: "https",
                prompt_tokens: pt, completion_tokens: ct, usd })
}
