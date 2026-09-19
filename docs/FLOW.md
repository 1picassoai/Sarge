# Sarge — the flow

*Two diagrams, in Mermaid, so GitHub renders them in the README with no image file.
Paste the fenced blocks as they are.*

## The loop — one task, start to finish

```mermaid
flowchart LR
  T([you type a task]) --> A[the agent<br/>list · read · write · build · run]
  A -- every write --> C{the check<br/>against the book}
  C -- hit --> R[refused<br/>fix it first — the run will not start]
  R --> A
  C -- clean --> B[build → run → did it answer?]
  B -- same failure twice --> Q[ask the tutor<br/>on the model's behalf]
  Q -- the answer, not a lecture --> A
  B -- WORKS --> L[the tutor reviews the code<br/>writes ONE rule]
  L --> S[(.sarge<br/>in the repo)]
  S -. next task: the rules sit at the tail,<br/>where a small model obeys them .-> A
```

**Read it left to right.** You give one task. The agent writes and builds and runs. Every
file it writes is checked against the book the moment it lands; a hit comes back as the
tool result and the app will not run until it is fixed. When the same failure comes back
twice, the loop asks the tutor for the model and hands it the answer. When the task ends
green, the tutor writes what it learned as one rule — with the wrong line and the right
line as code — into `.sarge` in the repo. The next task starts with it.

## Where things live

```mermaid
flowchart TB
  subgraph M[your machine — nothing leaves it]
    direction TB
    subgraph O[the organ · llama-server]
      H[the handshake · Rust, linked in<br/>SELECT · CHECK · LEARN · ASK]
      Q4[the model · Qwen3-4B on your GPU]
    end
    subgraph K[the book]
      U[book/universal.sarge<br/>laws true in any language]
      RS[repo/.sarge<br/>learned in this repo, travels with the clone]
    end
    AG[a client<br/>the Sarge agent · a Claude Code hook · a LangChain or CrewAI tool]
  end
  TU[the tutor<br/>your own frontier session — never sees the repo]
  AG -- task --> O
  H -- reads --> U
  H -- reads & writes --> RS
  H -- a question, a failure, file names --> TU
  TU -- one rule, in the language --> H
```

**The engine is the organ and the book.** Everything else is a client. Today the client
is the Sarge agent; the same engine sits behind a Claude Code hook on the write, and
behind the router when a subscription runs out and the model goes local — *when your
subscription runs out, your rules don't.*

## The rule, as the model reads it

```
=== sarge ===

rule use-config-for-connection
  do     read database filenames and connection strings from configuration or the environment
  never  hardcode database filenames or connection strings in source code
  wrong  | const db = new DatabaseSync(path.join(__dirname, 'tools.db'));
  right  | const db = new DatabaseSync(path.join(__dirname, config.database));
  since  2026-09-16 tutor
end

=== end sarge ===
```

Nine words. `do` first. Real code after the bar. Written by the tutor, never by a person —
though a person can, in the same nine words. This one was learned on 16 Sep 2026 from the
first task on a Node repo, and the second task got it at the tail.
