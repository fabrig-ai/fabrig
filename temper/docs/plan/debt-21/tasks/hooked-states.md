+++
id = "hooked-states"
part = "forge-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/src/hooks/connection.rs", "skein/crates/skein-forge-connection/src/hooks.rs"]
effort = "high"
minutes = 90
+++

# forge-connection: a webhook connection's four states hold their data

## Goal

`Hooked` keeps `malformed`, `ended`, `done`, `hint`, `hint_state`, `closing` and others beside no state at all. It gets the four states of `skein/docs/design/forge-connection.md`, 8.4, each holding its data and its deadline, and one `follow` sets the deadline after each transition.

Acceptance:

- `Hooked` holds `State` as the note sketches it, and those fields are gone.
- `rg -n '\b(self|c)\.deadline =' skein/crates/skein-forge-connection/src/hooks.rs skein/crates/skein-forge-connection/src/hooks/connection.rs` lists one line, in `follow`.
- `hooks.rs` sets no connection's state from outside: `fault` and `abort` on `Hooked` replace `hooks.rs:74, 116, 169`, and a constructor taking the accept time replaces `:161`. `new` stays under `#[cfg(test)]` if that spares its test callers.
- The note's H pins are written first and committed apart, passing on the trunk's code: H6 and H4 surely, H2 and H5 as its table says. They pass after, and each has been broken by hand. H6's break: restart the deadline on entering Body.
- The note's tests pass, with no assertion changed and no test retimed: `deadlines_end_each_hook_phase` and the 403, 404 and 405 tests among them. If a test's events change, stop and ask.
- Net production lines: the cut's estimate is about +30.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/forge-states.md`, binding
- `skein/docs/development/debt/D020-forge-connection-states.md` (its D016 line is stale: that closed, and `Connection` stands on `skein_link::Link`)
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/docs/design/forge-connection.md`, 8.4

## Owns and may change

- **Owns:** the two files in `files`. A pin goes beside `deadlines_end_each_hook_phase` in `skein/tests/forge/machine/`, or in `src/tests/`.
- **Stay off** `connection.rs` and `component*`: `connection-states` holds them, and may run beside you.
- **Status and the item:** not yours. `forge-states-close` writes them.

## Steps

1. Read 8.4, then `hooks/connection.rs` whole. Trace H2 and H5.
2. The pins; commit them.
3. `State`; `follow`; `hooks.rs`'s three sites and its constructor.
4. The breaks by hand, each restored.
5. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p skein-forge-connection -p skein-forge-machine-world --all-targets -- -D warnings`; the note's tests; the citations check. Commit and push.

## Report

The common note's items; what H2 and H5's traces found; each pin and what you broke; any state 8.4 doesn't have that the behaviour forced, and whether `follow` holds `Next`.

## Answers

2026-11-05. Question: accept the tested refactor at +100 net production lines against the cut's estimate of about +30, or require further reduction?

Accepted at +100 net production lines. fabrig/debt, the plan's owner, decided it (2026-11-05): a flags-to-states change with its pins, and the run says it has compacted; a spot review reads it after landing. Compact no further and change no behaviour for size: the pushed work on `debt-21/hooked-states` stands, and the size line of the acceptance holds at +100. Report Done if the rest of the acceptance holds after the trunk's merge.
