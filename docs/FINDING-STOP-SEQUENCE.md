# A stop sequence does not make the check faster — tried, measured, reverted

**Sun 20 Sep 2026, in the release review. Restored to docs 23 Sep on @Galahad's finding,
after the comment carrying it was deleted by accident.**

## The finding

The check is slow on CPU and the obvious fix is a stop sequence — cut the model off as
soon as it has said what it needs to. It was tried. **It changed nothing**, and the reason
matters more than the result.

The release review found a single call running to the 240-token cap at ~4 tok/s on CPU
(~59 s), and that is real. But measured across the fixtures the cost was spread evenly:

```
7 calls  ·  ~10 s each  ·  ~70 s total
```

So the driver is the **number of calls** as much as the length of any one of them. A stop
sequence saves a little off the tail of each and risks truncating a legitimate list of
`HIT` lines to buy a saving that did not appear.

**The real fix is fewer or cheaper calls — a design change, not a flag.**

## Why this is written down twice

It lived as a ten-line comment in `rust/src/verdict.rs`, beside the request it describes,
so the next person to think "why is there no stop sequence here" reads the answer instead
of spending an afternoon rediscovering it.

On 23 Sep I overwrote `verdict.rs` wholesale from a working clone that had never carried
the comment, and destroyed it. @Galahad caught it in review: *"That comment WAS the
mechanism. It existed so the next person does not spend an afternoon rediscovering a dead
end."*

It is back in the source, and it is here as well, because a file that can be overwritten
is not where a finding should live alone.

## Still true?

The condition has not changed: the check makes one pass over the file with the rules, then
a narrow confirmation per candidate line. **Measured again 23 Sep on 4 CPU cores with the
33-rule Node book: one full check, 53 seconds.** Same shape, same conclusion.

If the number of calls ever drops — one pass instead of a pass plus confirmations — this is
worth re-measuring. Until then, no stop sequence.
