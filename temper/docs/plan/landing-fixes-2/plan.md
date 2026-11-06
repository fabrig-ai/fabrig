# landing-fixes-2: the plan

From `inputs/request.md`, what `landing-fixes` dropped. Ranked, with a ceiling of three or four tasks. Everything below was judged from reading the code on 2026-11-06. No test has run yet, so each task reproduces first.

This plan is being released in pieces, and this piece holds item 4 alone. Items 1, 2 and 3 are still being traced.

## Parts and tasks so far

| Request's item | Part | Task | Needs |
|---|---|---|---|
| 4, a flake | `port-reservation` | `port-reservation` | |

**Critical path, as it stands:** `port-reservation`, about 45 implementer-minutes.

## What was decided

- **4. The flake is a lost reservation, from reading.** The test takes no port, but `prepare` takes kernel-chosen ones in two places. `reserve`'s `?` returns on the first bind error, so its 32 tries only cover excluded ports. `Io::listened` retries only a preferred port. Under `SO_REUSEADDR`, another process can bind a port explicitly and listen first, between this socket's bind and its listen. Both now take another port, staged in one thread. The evidence's log isn't in this repository, so which of the two was seen is unknown, and the task fixes both.

## Overlaps not avoided

- `port-reservation` changes `temper/crates/temper/src/init.rs`, which `ui-repo-link/box-sign-in` also changes. Its hunk stays inside `Io`.
