+++
id = "world-exemptions"
part = "jig-kits"
kind = "interface"
needs = []
files = ["jig/tests/boundary/src/tests/kits.rs", "jig/tests/one-process/Cargo.toml", "jig/testing/jig-test-service/Cargo.toml", "Cargo.lock"]
effort = "medium"
minutes = 25
+++

# jig's kit rule exempts what's a kit by design

## Goal

Jig D031's first step. 26 `PENDING` edges point at `skein-world`, which is skein's world harness and lives in `skein/testing/`, and two at `smith-agent-process-world`, which the item's **Paid when** calls one of smith's own world kits; `jig/docs/design/domain/testing.md`, 7 now says both. Two more are declared and unused.

Acceptance:

- `kits.rs` has a named list the rule skips, whatever the package's name:

  ```rust
  // Kits by design, whatever their names say (jig/docs/design/domain/testing.md, 7).
  const WORLD_KITS: [&str; 2] = ["skein-world", "smith-agent-process-world"];
  ```

- `world_breaches` doesn't count an edge onto those. Their 28 `PENDING` lines go.
- The two unused edges go, manifest line and `PENDING` line both: `jig-one-process-world` to `skein-llm-world`, `jig-test-service` to `jig-protocol-accounts-world`.
- The Measure, `awk '/^const PENDING/,/^\];/' jig/tests/boundary/src/tests/kits.rs | grep -c '^    ("'`, reads 27.
- `kits.rs`' rule test gains one row: an edge onto a `WORLD_KITS` name isn't a breach, and one onto another `-world` is. **It must bite:** empty the list, watch it fail, and restore.
- `heavy cargo nextest run -p jig-boundary` passes.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `jig/docs/development/debt/D031-worlds-as-kits.md`
- `jig/docs/design/domain/testing.md`, 7

## Owns and may change

- **Owns:** the files in the header.

## Steps

The list; the lines; the row; the break; `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-boundary --all-targets -- -D warnings`; `heavy cargo check -p jig-one-process-world -p jig-test-service --tests`. Commit and push.

## Report

The common note's items; the break and its failure.
