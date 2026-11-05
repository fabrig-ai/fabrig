+++
id = "connection-residue"
part = "connection-worlds"
kind = "feature"
needs = ["hand-seeded"]
files = ["skein/crates/skein-forge-connection/src/tests.rs", "skein/crates/skein-forge-connection/src/tests/**", "skein/crates/skein-forge-connection/src/component/tests.rs", "skein/crates/skein-forge-connection/src/component/github/tests.rs", "skein/crates/skein-forge-connection/src/connection/**/tests.rs", "skein/crates/skein-forge-connection/Cargo.toml", "skein/tests/forge/machine/src/hand.rs", "Cargo.lock"]
effort = "high"
minutes = 30
+++

# forge-connection's crate tests feed its step functions

## Goal

Skein D030's **Paid when**, the crate's half. `Harness` leaves the crate, and what stays feeds step functions events (`skein/docs/foundation/testing-strategy.md`, 2.1).

Acceptance:

- `rg -lw Harness skein/crates/skein-forge-connection/src` reads nothing. `ConnectionHarness` and `HookHarness` stay, as `temper/docs/plan/debt-24/notes/hand.md` says.
- `src/tests/fixtures.rs` holds `ConnectionHarness`, `HookHarness` and the fixtures the crate's tests read, and no more. `hand.rs`'s temporary copies, and the crate's of helpers nothing in the crate reads, go.
- **`component/tests.rs`' three `Harness` tests** can't move: they read `common`, `forges`, `own_closed` and `shed`, which are private to `Component`, and so aren't the seam's. They feed `Component` events by hand, as its other tests there do, with the same asserts.
- The crate's dev-dependencies lose what only `Harness` read, `skein-world`'s `stream` among them if nothing else does.
- `heavy cargo nextest run -p skein-forge-connection --lib` and `-p skein-forge-machine-world` pass. The report gives the crate's test count and lines before and after, and how many tests left it.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/hand.md`
- `skein/docs/foundation/testing-strategy.md`, 2.1
- `skein/docs/design/forge-connection.md`, 10

## Owns and may change

- **Owns:** the files in the header.
- **A third test in `component/tests.rs` that can't feed events without a socket's bytes:** it asserts over what `Harness` played, so stop and ask, naming it.

## Steps

1. The three tests; then `Harness` and the helpers go.
2. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-connection -p skein-forge-machine-world --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the counts above. **The hand-fed tests must bite:** break the rule each pins by hand, name the failure.

## Answers

### 2026-11-05: Convert all ten remaining component Harness tests to hand-fed step tests (recommended), or extend the machine-world seam and move the eight renewal tests there? The spec names three; the merged dependency leaves ten. (debt-24/connection-residue-1, foreman debt-24/connection-residue/foreman-1)

Convert all ten remaining component Harness tests to hand-fed step tests, preserving their assertions.
The acceptance requires: “`rg -lw Harness skein/crates/skein-forge-connection/src` reads nothing.”
The common note says: “Line numbers and counts in a spec are at this plan's base: find the code by name, and re-measure before you start.”
Thus “three” is a baseline count; the ten remaining consumers are in the owned `component/tests.rs`.
The goal says: “`Harness` leaves the crate, and what stays feeds step functions events.”
`skein/docs/foundation/testing-strategy.md`, 2.1: “A crate's own tests feed a step function events and inspect the requests it emits”.
The next run converts the ten within the allowed files, preserves assertions, performs the required mutation checks, removes unused helpers and dependencies, and completes the specified checks, counts, commit and push.
The explicit stop condition remains: a test “that can't feed events without a socket's bytes” requires asking, naming it. The report identifies no such test.
Extending the seam and moving the eight renewal tests is not authorized by this answer.

### 2026-11-05: Should the owning task repair `crate::hand` in `skein/testing/skein-forge-kit/src/github.rs` first, or may this task repair that file? (debt-24/connection-residue-2, foreman debt-24/connection-residue/foreman-2)

Neither: it's repaired already, and this task doesn't touch that file. `9741ae1b81` and `50fc5d966c` are in the trunk, and `files_service` reads `crate::fixtures::limits()`, with no `crate::hand` left; your run's start merges the trunk in. A compile failure in the kit past those three references, which name resolution may have hidden, is a new question if the run can't fix it inside its files.

Keep `621e9ce965` and finish what its report leaves: the crate's run after the cleanup, `heavy cargo nextest run -p skein-forge-connection --lib`, and `heavy cargo nextest run -p skein-forge-machine-world` with all its test binaries, no `--lib`; the two-package clippy; and the hand-breaking of the thirteen converted tests, naming what was broken and what failed.

`src/component/github/tests.rs` joins this task's files, for the `events` module declaration that reaches the private completion transitions; nothing else in it changes. `skein/docs/development/debt/D030-forge-connection-harness-tests.md` and the status entries stay as they are: they're `connection-close`'s.
