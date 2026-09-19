# Rules and the model in one process

**Sat 13 September 2026.** The handshake now selects rules and generates text inside a
single binary. No HTTP, no server, no subprocess.

Until this morning the two halves lived apart. Rule selection ran in one process,
serialised its output, and posted it to a `llama.cpp` server. That boundary meant the
only thing that could ever cross between them was a string.

---

## What one command does now

```
compile --task "add a delete customer endpoint" --repo HarrogatePortal --n 500 --no-think
```

```
handshake  1 of 1 rule(s) selected in 292 us
           - archive-not-delete
model      Qwen3-8B-Q4_K_M.gguf in 6.4s
prompt     123 tokens (101 of them rules)
generated  158 tokens
in-process no HTTP, no server, no subprocess
```

Selection takes **292 microseconds**. Of the 123-token prompt, **101 tokens are the rule
block** — the rules are not a footnote on the prompt, they are most of it.

---

## The comparison

Same model, same task, same greedy sampling. The only variable is whether the rules were
selected or withheld (`--raw`). Greedy is deliberate: a temperature would make two runs
differ for reasons that have nothing to do with the rules.

### With the rule

> I'll add a DELETE endpoint for deleting a customer. This will mark the customer as
> archived instead of actually removing it from the database.

```csharp
[HttpDelete("{id}")]
public async Task<IActionResult> DeleteCustomer(int id)
{
    var customer = await _context.Customers.FindAsync(id);
    if (customer == null)
    {
        return NotFound();
    }

    customer.IsArchived = true;
    await _context.SaveChangesAsync();

    return NoContent();
}
```

### Without it

> First, I need to recall the structure of a typical REST API... That involves finding
> the customer in the database by their ID and then **removing them**... In a backend
> framework like Express.js, I can use a route with a parameter. For example,
> `app.delete('/api/customers/:id', ...)`.

The unguided model deletes the row, and reaches for Express.js on a C# repository. It has
no way to know better — nothing in "add a delete customer endpoint" says otherwise.

---

## CHECK, proven in both directions

This is the part that had never been tested in-process, and it is the part most easily
faked. A check that cannot fail is not a check, and "CHECKS PASSED" over files the glob
never matched is a lie that looks like a pass. So both cases were run.

The rule's check:

```json
{ "files": "**/Program.cs", "must_not_match": "Customers\\.Remove\\(" }
```

**On what the model actually wrote:**

```
CHECKS PASSED  (1 rule(s) checked, 0 unenforceable)
exit 0
```

**On a deliberately bad fixture** containing `_context.Customers.Remove(customer);`:

```
CHECKS FAILED - 1 hit(s)
  [archive-not-delete] Program.cs:7  matched `Customers.Remove(`
      rule: In this codebase a customer is never deleted. DELETE on a customer means:
            load it, set IsArchived = true, SaveChangesAsync, return 204. Never call
            db.Customers.Remove(...). PortalDbContext has HasQueryFilter(c => !c.IsArchived)
            so archived rows are already hidden - no extra IsArchived guard is needed.
exit 1
```

The rule count is reported alongside an `unenforceable` count, so a pass can never
quietly mean nothing was checked.

---

## Two things that had to be got right

**The model's own chat template.** Qwen3 is an instruct model, and the template is read
out of the GGUF rather than written by us. Feeding an instruct model a raw string
produces rubbish — which would look like the handshake failing when it is the prompt that
is wrong.

**Thinking, disabled for coding tasks.** Qwen3 reasons by default. On the first run it
spent all 160 tokens inside `<think>` and never wrote a line of code. The reasoning was
correct — it restated the archive rule in its own words — but CHECK needs written files
to run over. Qwen3 honours a `/no_think` suffix, and `--no-think` sets it.

---

## What is not claimed

**No throughput claim is made.** This run measured whether the circle closes, not how
fast it turns. Generation speed is llama.cpp's, unchanged by anything here.

**The rules still enter as a system message.** They are text at the front of the context,
same as they were over HTTP. What changed is that no process boundary is crossed and the
program holds the context directly. Writing rules into the KV cache is the next step and
is not done here.

**The library is unchanged.** Its five tests still pass, and the model sits behind a
`model` feature flag so the library and CLI still build on a machine with no LLVM.
