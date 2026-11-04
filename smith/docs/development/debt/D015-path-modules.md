# D015: Code is shared between targets by `#[path]`

- **Kind:** tests
- **Where:** `benchmarks/src/lib.rs`, `benchmarks/src/agent/smith/observe.rs`, `tests/agent-process/tests/suite/main.rs`, `smith-agent-shell/src/config/tests/mod.rs`, `smith`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 13, Q7, ratchets)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's ratchets (`smith/scripts/old-forms.txt`, `path_attribute`, baselined at 66)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (shared test code has one owner and one compile; production builds no test file)
- **Size:** M
- **Status:** open
- **Paid when:** `path_attribute` in `smith/scripts/old-forms.txt` is 0, or each remaining `#[path]` is a plain module layout with its reason beside it, and no production target includes a file under `tests/`.
- **Measure:** `grep '^path_attribute ' smith/scripts/old-forms.txt` is 57

## What

57 `#[path]` attributes remain, from 66. Most lay out child modules in a
sibling directory. 15 reach up a directory to compile another target's file
into this one, among them:

- the benchmark library compiles its tests' common code:
  `benchmarks/src/lib.rs:95` and `:99` include `../tests/common/request.rs`
  and `../tests/common/json.rs`, and `agent/smith/observe.rs:263` includes
  `tests/common/events.rs`;
- `smith-agent-shell`'s inline tests include `tests/support/product.rs`
  (`config/tests/mod.rs:398`), which its fuzzy suite includes again;
- `tests/agent-process` builds its live probe into the suite
  (`tests/suite/program_configuration.rs:3`).

## Why it matters

A file compiled into several targets has no single owner: its lints, its
dead code and its visibility are judged per copy, and a production target
that includes a test file links test code. smith's other shared test code
lives in world crates or behind an owner's `fixture` feature.

## What pays it back

Move each shared file into the owner crate behind its `fixture` feature, or
into the world crate that already serves those tests; turn plain layout
`#[path]`s into ordinary module directories. Lower the baseline as they go.
