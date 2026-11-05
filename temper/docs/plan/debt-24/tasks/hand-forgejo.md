+++
id = "hand-forgejo"
part = "connection-worlds"
kind = "migration"
needs = ["hand-face"]
files = ["skein/tests/forge/machine/tests/suite/hand/forgejo.rs", "skein/tests/forge/machine/tests/suite/hand/forgejo/**", "skein/tests/forge/machine/src/hand.rs", "skein/crates/skein-forge-connection/src/tests.rs", "skein/crates/skein-forge-connection/src/tests/contents.rs", "skein/crates/skein-forge-connection/src/tests/writes.rs", "skein/crates/skein-forge-connection/src/tests/reads.rs", "skein/crates/skein-forge-connection/src/tests/connection.rs", "skein/crates/skein-forge-connection/src/tests/artifact.rs"]
effort = "medium"
minutes = 35
+++

# The Forgejo tests move to the machine world

## Goal

Skein D030's Forgejo batch: `contents`, `writes`, `reads`, `connection`, `artifact`, and `src/tests.rs`' six sites, about 83 tests. `abort` is `hand-face`'s. By `temper/docs/plan/debt-24/notes/hand.md`.

Acceptance:

- Their `Harness` tests are in `tests/suite/hand/forgejo/`, under the same file names. `artifact` leaves the crate; the mixed files keep what isn't on `Harness`. `reads`'s and `connection`'s tests that use a private drive call stay in the crate until `connection-seam`, for `hand-rows`.
- Each test's asserts read as they did; `artifact`'s ceiling holds.
- `heavy cargo nextest run -p skein-forge-connection --lib` and `-p skein-forge-machine-world` pass; the world gains what the crate loses.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/hand.md`
- `skein/docs/design/forge-connection.md`, 10

## Owns and may change

- **Owns:** the files in the header. `hand.rs` only gains what its files read.
- **`hand/github.rs` and `hand/github/` aren't yours:** `hand-github` holds them at once.

## Steps

The script from `hand-face`'s report; `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-machine-world -p skein-forge-connection --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the assert comparison; the time. A move: no rule to break by hand.

## Answers

### 2026-11-05: Complete and merge hand-face before rerunning hand-forgejo (recommended), or authorize this task to supply the missing shared harness and suite scaffolding? (debt-24/hand-forgejo-1, foreman debt-24/hand-forgejo/foreman-1)

Complete and merge hand-face before rerunning hand-forgejo.
The hand-forgejo spec declares `needs = ["hand-face"]` and requires “The script from `hand-face`'s report”.
Hand-face owns the missing shared harness and suite scaffolding; its acceptance requires “`pub mod hand;` in `lib.rs`” and “`tests/suite/hand.rs`, `hand/forgejo.rs` and `hand/github.rs`”.
Hand-forgejo's ownership says “`hand.rs` only gains what its files read.”
The next hand-forgejo run uses the completed prerequisite's harness, scaffolding and reported script, then performs its migration and acceptance checks.

### 2026-11-05: Publish hand-face's completed report with its required move script and rerun this task (recommended), or authorize hand-forgejo to write its own migration script? (debt-24/hand-forgejo-2, foreman debt-24/hand-forgejo/foreman-2)

Publish hand-face's completed report with its required move script, then rerun hand-forgejo.
The task's Steps require “The script from `hand-face`'s report”.
Hand-face step 3 requires “the one-shot script the later tasks reuse; keep the script in your report, as text.”
The recorded answer requires the next run to use “the completed prerequisite's harness, scaffolding and reported script”.
The shared code is present; the required reported script remains missing.
The next hand-forgejo run uses that script, performs its migration and acceptance checks, then commits and pushes.
