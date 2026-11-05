+++
id = "connection-deadlines"
part = "forge-states"
kind = "feature"
needs = ["connection-states"]
files = ["skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/connection/**", "skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/**"]
effort = "high"
minutes = 60
+++

# forge-connection: a connection sets its deadline in one place

## Goal

`skein/docs/design/forge-connection.md`, 8.1: which deadline runs is a function of the phase, set in one place after each transition. Today fifteen sites across `connection.rs`, `component.rs`, `component/github.rs`, `component/calls.rs` and `component/github_calls.rs` set it. One `follow` does, reading what each state holds.

Acceptance:

- Every state of the note's `State` but `Closed` holds `until: Option<Time>`. `follow` writes `self.deadline` from an exhaustive match, and is called after every transition.
- `pub(crate) fn hold(&mut self, until: Time)` replaces `component.rs:703`, `github.rs:570`, `calls.rs:179`, `github_calls.rs:391`. `pub(crate) fn rest(&mut self, until: Option<Time>)` replaces `github.rs:919, 951`, `calls.rs:171, 272`, `github_calls.rs:460`. Each does what those lines do today in each phase: C4 and C5 say what that is.
- `rg -n '\b(self|c)\.deadline =' skein/crates/skein-forge-connection/src/connection.rs skein/crates/skein-forge-connection/src/component.rs skein/crates/skein-forge-connection/src/component` lists one line, in `follow`.
- The C2, C3, C4 and C5 pins are written first and committed apart, passing on the trunk's code. They pass after, and each has been broken by hand. C1, C6 and C11's named tests pass unchanged.
- The note's tests pass with no assertion changed and no test retimed. `tests/dialects.rs:294, 307` read `.deadline`: that field stays, as what `follow` writes. If a test's events change, stop and ask.
- Net production lines fall.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/forge-states.md`, binding
- `skein/docs/development/debt/D020-forge-connection-states.md` (its D016 line is stale: that closed, and `Connection` stands on `skein_link::Link`)
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/docs/design/forge-connection.md`, 8.1, the paragraph after the table

## Owns and may change

- **Owns:** the files in `files`. A pin goes in `src/tests/connection.rs` or `src/tests/dialects.rs`, beside its neighbours.
- **`until` is carried, not recomputed,** across a transition that today leaves the deadline alone: C1 is the case a test pins, and a recompute on entering Idle is the mistake to look for.
- **Stay off** `hooks.rs` and `hooks/connection.rs`.

## Steps

1. Trace C5. The pins; commit them.
2. `until`, `follow`, `hold`, `rest`; the fifteen sites.
3. The breaks by hand, each restored.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`; the note's tests; the citations check. Commit and push.

## Report

The common note's items; what C5's trace found; each pin and what you broke.
