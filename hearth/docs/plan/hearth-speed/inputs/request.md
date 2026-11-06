# Request: hearth's focused suite under its 30 CPU s

From fabrig/hearth, 2026-11-06. The user ruled on 11-04 that hearth's focused suite stays at 30 CPU s: "I'm sure there's room for improvement in test performance". The memory migration has landed (`max_out` 0, `worst_case` 0) and didn't get there. The queue's own record is 31.7 CPU s over 2,189 tests (full batch, 11-06 08:56Z). Plan `hearth-memory-6` measured 41.7 on a builder, with the wrapper and `NEXTEST_DOUBLE_SPAWN=0`. **The figure that counts is the queue's, as `tools/tiers/budgets.toml`'s `[gate.hearth.default]` records it.** Aim for 27 on it, so ordinary noise doesn't put it back over. Low priority, and small.

**Every assertion and every case stays.** Don't move a test out of the focused suite to escape the budget: `AGENTS.md`, 5 calls that raising it. Don't raise the budget. Don't shrink a scenario, cut a seed, or lower a count. What a test proves doesn't change, and neither does what hearth does for a user.

## Where it goes

`hearth-memory-6`'s measurement, by family, on its machine: the builder suite 12.0 CPU s (106 tests), the tree suite 10.0 (54), the builder's real-git binary 8.5 (183), the render suite 4.1 (109). No single test is over 1.3, so this takes shared savings. Measure before deciding: a profile says more than this request does.

Leads, not instructions:

- **What many tests build the same way:** fixtures, repositories, scenarios and worlds built per test. Under nextest a test is a process, so a saving must survive that: build less, cheaper, or from a file already in the tree. Tests run as they do; a world's seed still replays.
- **The real-git binary:** 183 tests at about 46 ms each. How much is process spawn and repository setup, and can a test share a checked-in or template repository?
- **Debug builds of the hot code:** the root `Cargo.toml` already gives some dev packages optimization (`[profile.dev.package.sha2]`, `rustls`). If a profile shows digests or another pure dependency dominating, one line there may be most of it: `sha1` is a candidate. It affects build time and everyone's builds, so give the build cost and the saving in the commit, and keep it to what measures well.
- **Production:** where a profile shows a real inefficiency in hearth's own code, fixing it is welcome, with what hearth does unchanged.

## What to report

Before and after, by the method of `hearth/docs/design/domain/testing.status.md`, 9.1, on one machine, and the gate's record after landing. Each commit says what it saved, measured. `testing.status.md`, 9.1 and `STATUS.md` state the result; if the suite is under 30, the departure goes.

## Rules

- `skein/`, `smith/`, `jig/` and `temper/` change only if a measurement shows a shared kit is the cost. That's a question first, not a task, except the root `Cargo.toml` line above. skein and jig are shared with other sessions.
- A memory ceiling doesn't move for this.
- `tools/citations/citations.py --all` stays at 0, and `tests/boundary/tests/outward.rs` passes.
- Keep it small: stop once the aim is met. If it can't be reached by what this request allows, say what's left and by how much, and land what helps.
- All parts must be in the plan when the cut ends.
