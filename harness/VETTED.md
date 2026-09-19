# VETTED.md — the rules, and where each one is allowed to go

*Generated 2026-09-14 by `tools/vet.py` from `bench/stick/results/findings.json`.*
*Do not hand-edit the verdicts — re-run the matrix. You may strike a whole rule.*

**The handshake reads this file at boot and on change. It is the only thing that
gets injected.**

| | |
|---|---|
| **INJECT** | 9 rules — safe for the KV cache, the model cannot argue |
| **GROUND** | 1 rules — system message only, the model may push back |
| **REWRITE** | 2 rules — never inject; name the replacement, re-test |

---

## INJECT — into the blood

Held everywhere it was shown, across every model. Safe to write into the
KV cache, where it stops being advice and becomes part of how the model
thinks for the length of that context.

### `archive-not-delete` — 3/3 (100%)

> A customer is never deleted. DELETE means load it, set IsArchived = true, SaveChangesAsync, return 204. Never call Customers.Remove(...).

- **Why:** held 3/3 across 3 models
- **Per model:** Qwen3-8B 1/1 · Qwen2.5-Coder-7B 1/1 · Qwen3-0.6B 1/1
- **Check:** `Customers\.Remove\(` over `**/*.cs`

### `no-controllers` — 9/9 (100%)

> This is a minimal-API codebase. Map endpoints with app.MapGet / app.MapPost / app.MapDelete. Never write a [ApiController] class.

- **Why:** held 9/9 across 3 models
- **Per model:** Qwen3-8B 3/3 · Qwen2.5-Coder-7B 3/3 · Qwen3-0.6B 3/3
- **Check:** `\[ApiController\]|:\s*ControllerBase` over `**/*.cs`

### `async-suffix` — 8/8 (100%)

> Every async method name ends in Async. Write DeleteCustomerAsync, never DeleteCustomer.

- **Why:** held 8/8 across 4 models
- **Per model:** Qwen3-8B 2/2 · Qwen2.5-Coder-7B 2/2 · Qwen3-0.6B 2/2 · MiniCPM5-2B 2/2
- **Check:** `async\s+Task[^\n]*\s(?!\w*Async)\w+\s*\(` over `**/*.cs`

### `no-datetime-now` — 8/8 (100%)

> Never call DateTime.Now. Use DateTime.UtcNow so stored times are unambiguous.

- **Why:** held 8/8 across 4 models
- **Per model:** Qwen3-8B 2/2 · Qwen2.5-Coder-7B 2/2 · Qwen3-0.6B 2/2 · MiniCPM5-2B 2/2
- **Check:** `DateTime\.Now` over `**/*.cs`

### `no-magic-status` — 9/9 (100%)

> Return typed results - Results.NoContent(), Results.NotFound(). Never return a bare StatusCode(204) or StatusCode(404).

- **Why:** held 9/9 across 3 models
- **Per model:** Qwen3-8B 3/3 · Qwen2.5-Coder-7B 3/3 · Qwen3-0.6B 3/3
- **Check:** `StatusCode\(\s*\d{3}\s*\)` over `**/*.cs`

### `no-sync-over-async` — 8/8 (100%)

> Never block on a task with .Result or .Wait(). Always await it.

- **Why:** held 8/8 across 4 models
- **Per model:** Qwen3-8B 2/2 · Qwen2.5-Coder-7B 2/2 · Qwen3-0.6B 2/2 · MiniCPM5-2B 2/2
- **Check:** `\.Result\b|\.Wait\(\)` over `**/*.cs`

### `no-string-concat-sql` — 3/3 (100%)

> Never build SQL by string concatenation. Use FromSqlInterpolated or LINQ.

- **Why:** held 3/3 across 3 models
- **Per model:** Qwen3-8B 1/1 · Qwen2.5-Coder-7B 1/1 · Qwen3-0.6B 1/1
- **Check:** `FromSqlRaw\(|ExecuteSqlRaw\(` over `**/*.cs`

### `no-public-fields` — 3/3 (100%)

> Never expose a public field. Use a property with { get; set; }.

- **Why:** held 3/3 across 3 models
- **Per model:** Qwen3-8B 1/1 · Qwen2.5-Coder-7B 1/1 · Qwen3-0.6B 1/1
- **Check:** `public\s+(?!class|record|interface|enum|static|async|override|virtual|sealed|abstract|partial|readonly\s+struct)\w+(<[^>]+>)?\s+\w+\s*;` over `**/*.cs`

### `no-hardcoded-conn` — 3/3 (100%)

> Never hardcode a connection string. Read it from IConfiguration.

- **Why:** held 3/3 across 3 models
- **Per model:** Qwen3-8B 1/1 · Qwen2.5-Coder-7B 1/1 · Qwen3-0.6B 1/1
- **Check:** `Server=|Data Source=.*Password=` over `**/*.cs`

---

## GROUND — system message only

Read by the model, not built into it. It can still push back, and CHECK
catches it when it does. Promote by re-running the matrix on more tasks.

### `no-console-write` — 9/10 (90%)

> Never call Console.WriteLine. Use the injected ILogger.

- **Why:** held 9/10 - one failure, not safe to make unarguable
- **Per model:** Qwen3-8B 3/3 · Qwen2.5-Coder-7B 3/3 · Qwen3-0.6B 2/3 · MiniCPM5-2B 1/1
- **Check:** `Console\.WriteLine` over `**/*.cs`

---

## REWRITE — never inject

These fight the model's training prior and lose. **The fix is not a
stronger prohibition — it is naming the replacement.** `no-controllers`
held 9/9 despite minimal APIs being the *less* common pattern, because it
says what to write instead. `no-var` only says what not to write.

### `no-catch-all` — 5/11 (45%)

> Never write catch (Exception). Catch the specific type you can handle.

- **Why:** broke 6 of 11 - fights the training prior
- **Per model:** Qwen3-8B 0/3 · Qwen2.5-Coder-7B 1/3 · Qwen3-0.6B 3/3 · MiniCPM5-2B 1/2
- **Check:** `catch\s*\(\s*Exception` over `**/*.cs`

### `no-var` — 9/20 (45%)

> Declare locals with their explicit type. Never use var.

- **Why:** broke 11 of 20 - fights the training prior
- **Per model:** Qwen3-8B 2/6 · Qwen2.5-Coder-7B 3/6 · Qwen3-0.6B 4/6 · MiniCPM5-2B 0/2
- **Check:** `(?<![\w.])var\s+\w+\s*=` over `**/*.cs`

---

## How a rule moves between tiers

Only by evidence. Re-run `python bench/stick/run.py`, then `python tools/vet.py`.
A rule earns INJECT by holding 100% across at least three models and three runs.
Nothing is promoted by argument.
