# Sarge

**Your agent writes the code. Sarge decides if it runs.**

Every file your coding agent writes gets checked against your repo's rules — by a small
model on your own machine (Qwen3-4B, 2.3 GB). Break a rule and the code doesn't run until
it's fixed. Break the same rule twice and Sarge writes it down, so next time it's already
there.

![Sarge catching a bad write and refusing the run](docs/run-animation.gif)

## What actually happens

**Your agent writes a bad file.** Sarge catches it and tells the agent what's wrong. The
agent fixes it. You didn't have to say anything.

**The agent tries to run the app anyway.** It can't. Nothing runs while a rule is broken —
that's the whole point.

**The file gets fixed.** Sarge steps out of the way and the app runs.

**Sarge can't check for some reason?** It says so, loudly, every single time. It will never
quietly tell you everything's fine when it hasn't looked.

![how it fits together](docs/architecture.png)

## How this is different

| | AI coding agents | a linter | Sarge |
|---|---|---|---|
| writes the code | yes | no | yes |
| where the rules live | in the prompt | in a config file | in your repo |
| what reads them | the model writing the code | a pattern matcher | a second model, on your machine |
| can it know your repo's conventions | told each session | only if you write the rule | yes, that's the book |
| if a rule is broken | nothing | a warning | the code doesn't run |
| do the rules improve | only when you edit them | only when you edit them | they grow from what broke |
| where your code goes | to their servers | nowhere | nowhere |

**We measured it rather than claiming it.** We took a file a local model wrote with no
help, and gave it to both:

| the fault | ESLint | Sarge |
|---|---|---|
| a syntax error | **caught** | missed |
| used the `sqlite3` package where the repo uses `node:sqlite` | missed | **caught** |
| hardcoded the database path | missed | missed |
| opened a new database connection per request | missed | missed |

ESLint found the syntax error — which `node --check` catches anyway. Once that was fixed,
it passed the file clean. It has no way to know your repo uses `node:sqlite`: that's a
convention, not a language rule, and nobody has written an ESLint rule for it.

Sarge caught the convention and missed two others, so this is not a clean sweep and we are
not going to pretend otherwise. One file, one model, one run. **They're not the same tool
and you'd sensibly run both.**

## Install

**macOS**, Apple silicon:

```bash
curl -fsSL https://raw.githubusercontent.com/1picassoai/Sarge/main/install.sh | bash
```

It clones this repo, downloads the engine for your machine, and fetches the model. Then
start it and leave it running:

```bash
./start-organ.sh
```

**Switch it on for a repo.** Copy `hooks/settings.example.json` into
`.claude/settings.json`, and put a `.sarge` file at the repo root — copy
`book/universal.sarge` if you haven't got one. That's it.

**Already running a local model?** Point Sarge at it instead — set `SARGE_ORGAN` to its
address and skip the 2.3 GB download. Qwen3-4B is what we test on and what we'd recommend,
but nothing here is tied to it.

That address has to be on your own machine. If it isn't, Sarge says so and uses the local
one anyway — your code doesn't leave by accident because of a pasted URL.

***Linux is coming.*** The installer is in the tree and refuses to run until a Linux organ
has been built and walked on a real machine.

## The rules

Sarge comes with **33 rules for Node and Express**, plus 3 that apply to any language. Each
one is written as real code — a wrong line and a right line — so the model can see the
difference rather than read about it:

```
rule config-not-code
  do     read the database name from configuration
  never  write a filename or connection string into source
  wrong  | const db = new DatabaseSync('tools.db');
  right  | const db = new DatabaseSync(config.database);
```

Your own rules go in `.sarge` at your repo root. Yours always win.

## The numbers

| | |
|---|---|
| what you need | 8 GB RAM, 4 cores. A GPU makes it quicker; it isn't required |
| download | 2.3 GB, almost all of it the model |
| checking one file | a few seconds on a GPU, about a minute on 4 cores |
| rules it ships with | 33 for Node, 3 universal |
| what leaves your machine | nothing. The check only talks to a model on your own machine — it refuses any other address unless you explicitly allow it. The tutor is the one exception, and only if you give it a key |

MIT. Something it got wrong? [Discussions](../../discussions).
