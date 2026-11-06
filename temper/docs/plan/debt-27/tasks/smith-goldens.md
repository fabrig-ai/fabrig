+++
id = "smith-goldens"
part = "smith-goldens"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-transcript/tests/data/**", "jig/crates/jig-protocol-transcript/src/tests.rs", "jig/testing/jig-client-test-records/src/activity_reads.rs", "jig/testing/jig-client-protocol-kit/src/fixtures.rs", "jig/testing/jig-test-worker/src/tests/fixtures.rs", "jig/docs/development/debt/D037-copied-goldens.md"]
effort = "medium"
minutes = 15
+++

# jig's tests read smith's goldens from `smith/`

## Goal

Pay jig's D037. The copies go, and every reader takes `include_bytes!` from smith's golden path. The cut compared them: both `.bin` files are byte-equal to `smith/crates/smith-transcript/golden/v6/`, and `SMALLEST` is byte-equal to `smith/crates/smith-charter/golden/v4/record_charter_smallest.bin`. So no test's outcome moves.

Acceptance:

- `jig/crates/jig-protocol-transcript/tests/` is gone, its README with it.
- `rg -n 'tests/data/record_turn|smith revision' jig -g '!**/docs/development/**'` finds nothing, and `rg -n 'const SMALLEST: \[u8' jig` finds nothing.
- `jig/docs/development/debt/D037-copied-goldens.md` is deleted.

## Inputs

- `jig/docs/development/debt/D037-copied-goldens.md`, whole.
- `temper/docs/plan/debt-27/notes/common.md`.

## Owns and may change

Owns the files in the header. Precedent for a path across trees: `jig/crates/jig-protocol-accounts/src/host_tests.rs` includes `../../../../skein/tests/tls/fixtures/root.der`; jig may name smith (`AGENTS.md`, 1), and `tests/boundary/` reads manifests.

`FULL_TURN` and `TURN_BYTES` keep their names and types; their readers, in jig and in temper, don't change. The kit's `TURN_BYTES` wasn't in the item: it's the fourth reader of the same copy.

## Steps

| File | From there, the path |
|---|---|
| `jig-protocol-transcript/src/tests.rs`, both tests | `../../../../smith/crates/smith-transcript/golden/v6/record_turn_smallest.bin`, `.../record_turn_full.bin` |
| `testing/jig-client-test-records/src/activity_reads.rs` | `../../../../smith/crates/smith-transcript/golden/v6/record_turn_full.bin` |
| `testing/jig-client-protocol-kit/src/fixtures.rs` | the same |
| `testing/jig-test-worker/src/tests/fixtures.rs` | `const SMALLEST: &[u8] = include_bytes!("../../../../../smith/crates/smith-charter/golden/v4/record_charter_smallest.bin");`, and `Reader::new(SMALLEST)` |

Delete the two `.bin` files and the README. A move of bytes needs no hand break.

`heavy cargo nextest run -p jig-protocol-transcript -p jig-test-worker -p jig-client-protocol-link`, then the worlds that read `FULL_TURN` and `TURN_BYTES`: `jig/tests/core` and `jig/tests/client-protocol`, by the names in their manifests. `heavy cargo check --tests -p temper-client-shell-terminal`. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-transcript -p jig-client-test-records -p jig-client-protocol-kit -p jig-test-worker --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says, with the net lines.
