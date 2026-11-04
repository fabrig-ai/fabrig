+++
id = "skein-now-with"
part = "temper-forms"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-shell/src/clock.rs",
  "skein/docs/design/shell.status.md",
]
effort = "medium"
minutes = 15
+++

# skein-shell: `Now::with` gives one iteration's clocks to a step

## Goal

A shell turns its `Now` into an `Env` field by field, `Env { now: now.now, wall: now.wall, limits }`,
13 times in temper, where `Env::with` says it for a child step. `skein_shell::Now` gets the same
call (`skein/docs/design/shell.md`, 5). A small addition; `temper-env-now` calls it.

Acceptance:

- `Now::with` exists as sketched, with one unit test.
- `heavy cargo nextest run -p skein-shell --lib` passes, and the new test is in the count.
- `skein/docs/design/shell.status.md` has no entry for 5.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/shell.md`, 5, which this cut wrote; `skein/docs/design/lib.md`, 9.

## Owns and may change

- Owns: `skein-shell/src/clock.rs`, `skein/docs/design/shell.status.md`.
- May change: `skein-shell/src/tests.rs`, one added test.
- Nothing in skein, smith or jig moves onto the call: `skein/docs/development/debt/D003-env-rewraps.md`
  and the worlds' rewraps are not this task's.

## Steps

1. In `clock.rs`:

   ```rust
   impl Now {
       /// These times, for a step with `limits`.
       #[must_use]
       pub fn with<L>(self, limits: L) -> Env<L> {
           Env { now: self.now, wall: self.wall, limits }
       }
   }
   ```

2. A test in `src/tests.rs`: `Now { now, wall }.with(7_u32)` has those clocks and those limits.
3. Remove the status's `Not yet` entry for 5, with its date line today's.
4. `cargo fmt`; `heavy cargo clippy -p skein-shell --all-targets -- -D warnings`; the test; the
   citations check. Commit and push.

## Report

The commit and the test count.
