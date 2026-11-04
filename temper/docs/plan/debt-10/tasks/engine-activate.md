+++
id = "engine-activate"
part = "tidying"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/route.rs", "temper/crates/temper-engine-domain/src/route/now.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/tests/engine/domain/tests/suite/upgrade_transcripts.rs"]
effort = "high"
minutes = 45
+++

# A3, A5: one `Work::Activate`, and two tests that end alike

## Goal

`Work::Activate(context)` and `Work::Due { context, ready }` differ only in who read `ready` and in a gate at their
arm in `route_into`. The request: prove `domain.work` is empty at that `Resume` first; if it isn't, skip A3. Then A5.
Behaviour stays unchanged.

Acceptance:

- **A3,** if the proof holds: one `Work::Activate { context, ready }`, no `Work::Due`, and no gate on its arm:
  `resume_routed` pushes `ready: true`, `route/now.rs` pushes what it read. If the proof fails, nothing of A3 changes
  and your report says which step broke it.
- **A5.** The two tests in `upgrade_transcripts.rs` share one helper for their common end, and `pending_reply` calls
  `transcript_task`. Each test still pins what it pinned, exact `Resume` count included.
- `temper-engine-domain`'s tests and its world pass with no assertion changed.
- **Size cap:** net at most -20 lines.

## Inputs

- `temper/docs/plan/debt-10/notes/common.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `temper/docs/design/domain/root.md`, 7.1, 8 and 9; jig's `jig/docs/design/domain/engine.md`, 6.1 and 7.2.

**What this cut read, for you to prove.** In `resume_routed` (`domain.rs`):

1. `if !domain.work.is_empty() { route; close; return }` stands first, so below it `domain.work` is empty.
2. Then come `backlog`, `domain.core.stopping()`, which returns, and the `ready` block, which returns or falls through
   having pushed nothing. So the push of `Work::Activate` finds `domain.work` empty, `settling` inactive (the same `if`
   checked it), the core not stopping, and `domain.ready()` true.
3. `route(domain, env)` follows at once, and `route_into` pops it first, unless one of its three early returns fires:
   `start_stop`, `start_failure` or `refuse_forge_fit`. Each of those stops the start, and `resume_routed` then returns
   at its first line forever, so the work is never popped. **Check that for `start_failure`'s `stop_core`.**

So the gate at the arm can't fire, and the `continue` there, which would drop the context, is dead. The review calls
one half of it `stopping::gate`; there's no such function, and the check is `resume_routed`'s `stopping()`.

## Owns and may change

- Owns: the files above.
- `route/forge_route.rs` has an exhaustive match naming both variants: one arm line.
- **Don't change a `Resume` count.** If A3 moves one, the proof missed something: undo A3 and report it.

## Steps

1. The proof, by a run: in your slot, not committed, add `assert!(domain.work.is_empty())` before the push, and at the
   arm in `route_into` `assert!(!domain.settling.active() && !domain.core.stopping())`. Run `heavy cargo nextest run -p
   temper-engine-domain -p temper-engine-domain-world`, then the same with `--profile fuzzy`. Report the counts. Remove the asserts. Read the three early returns and report what you found.
2. A3: the variant, `route.rs`'s arm, `now.rs`, `domain.rs`, `forge_route.rs`.
3. A5: from `driver.send(reply)`'s neighbourhood to the end, the two tests are the same. One helper, with `pending_reply`
   built on `transcript_task`. Keep every assertion and comment, and keep each `Resume`.
4. `heavy cargo nextest run` on both packages, and `-p temper-service --lib -E 'test(activat) | test(transcript)'`. fmt,
   clippy, old forms, citations. Commit, push: one commit per finding.

## Report

As the implementer's text says, with the net lines and the proof's run first: what was asserted, how many tests ran
under it, and what the three early returns do.
