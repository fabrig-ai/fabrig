+++
id = "path-fixtures"
part = "skein-kits"
kind = "migration"
needs = ["forge-kit"]
files = ["skein/testing/skein-forge-kit/src/fake_github.rs", "skein/testing/skein-forge-kit/src/lib.rs", "skein/testing/skein-forge-kit/Cargo.toml", "skein/tests/fake-forge/tests/github/fixtures.rs", "skein/tests/fake-forge/tests/github.rs", "skein/tests/fake-forge/tests/suite/forgejo_memory.rs", "skein/tests/fake-forge/tests/suite/main.rs", "skein/tests/fake-forge/Cargo.toml", "skein/tests/fake-peers/tests/suite/github_memory.rs", "skein/tests/fake-peers/Cargo.toml", "skein/tests/forge/dialect/tests/common/mod.rs", "skein/tests/forge/dialect/tests/common/admitted.rs", "skein/crates/skein-fake-forge-forgejo/src/tests.rs", "skein/crates/skein-fake-forge-forgejo/src/tests/memory.rs", "skein/crates/skein-fake-forge-forgejo/Cargo.toml", "skein/examples/ui/shell-web/src/lib.rs", "skein/examples/ui/shell-web/src/folds_fixture.rs", "skein/tests/browser/tests/suite/folds_fixture.rs", "Cargo.lock"]
effort = "medium"
minutes = 35
+++

# skein: no `#[path]` crosses a package

## Goal

Skein D028's four `#[path]`s that reach into another package. The item counted three; `examples/ui/shell-web` is the fourth. A `#[path]` inside one package (a `tests/` file reading its `src/`) isn't one: the item's count and rule are about crates apart, and the gate reads the package's directory.

Acceptance:

- `rg -n '#\[path' skein -g '*.rs'` lists no path that resolves outside the file's package.
- **`fake-peers/tests/suite/github_memory.rs`:** `tests/fake-forge/tests/github/fixtures.rs` moves to `skein_forge_kit::fake_github`, with `fake-forge`'s GitHub suite and `github_memory.rs` reading it. Both dev-depend on the kit.
- **`fake-forge/tests/suite/forgejo_memory.rs`:** its six tests' file moves into the crate whose fixtures it reads, as `skein-fake-forge-forgejo/src/tests/memory.rs`, with the counting allocator a `#[cfg(test)] #[global_allocator]`, as `skein-forge-connection/src/tests.rs` has it, and `skein-heap` and what it meters with as dev-dependencies. Its asserts read as they did, with a ceiling that shrinks or grows under the common note's rule. **If a crate-level allocator would change a test the crate already has,** keep the world's test and give it its own small `Harness` over public items, and say so.
- **`forge/dialect/tests/common/mod.rs`:** `common/admitted.rs` is the world's own, reduced to what the dialect world reads. The crate keeps its fixture for its unit tests. A crate's unit tests can't share a fixture with a world any other way. This is the one copy the rule forces, and the report names its lines.
- **`examples/ui/shell-web/src/lib.rs`:** `git mv` `tests/browser/tests/suite/folds_fixture.rs` to `examples/ui/shell-web/src/folds_fixture.rs`, since only the example compiles it, and drop the `#[path]`.
- `heavy cargo nextest run -p skein-fake-peers-tests -p skein-fake-forge-tests -p skein-fake-forge-forgejo -p skein-forge-dialect-tests` passes; `heavy cargo check -p skein-ui-example-shell-web --tests` passes on the host; if the example builds only for wasm32, say so and run its tier's check by name, never `tools/tiers/run`.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `skein/docs/development/debt/D028-worlds-as-kits.md`

## Owns and may change

- **Owns:** the files in the header. Other `.rs` files of `skein/tests/fake-forge/tests/github*` may change a `use`.
- **The five `#[path]`s within one package and the three manifests' `path =`** stay: they cross no package. `tests/world`'s and `tests/io`'s bins compile their own files.

## Steps

One site, one commit, each with its tests; `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy` naming each changed package `--all-targets -- -D warnings`. Commit and push.

## Report

The common note's items; `admitted.rs`'s lines before and after. A move: no rule to break by hand.

## Rebase (2026-11-05, step merge, foreman debt-24/path-fixtures/foreman-1)

reconcile the forge-kit extraction with the trunk’s forge-connection seam wiring
The merge conflicts in three files; resolve them against the landing trunk.
The overlapping trunk commits are `7c2a927c25` (rebased forge-kit extraction) and `38d49979da` (machine-world seam access).
- `skein/testing/skein-forge-kit/src/lib.rs`: retain the extracted module declarations and this task’s `pub mod fake_github;`, each once.
- `skein/tests/forge/machine/Cargo.toml`: retain the kit dependencies and dependency cleanup, with `skein-fake-forge-domain` under dev-dependencies. Preserve the trunk’s `skein-forge-connection = { workspace = true, features = ["seam"] }`.
- `skein/tests/forge/machine/src/lib.rs`: retain `pub mod hand;`, `pub mod memory;`, `pub use skein_forge_connection::seam::GitHub;`, and `pub use skein_forge_kit::*;`. Keep the extracted modules in the kit.
Preserve the installation transcript migration from trunk commit `1b752646a0`; its machine-world tests need the seam feature and GitHub re-export.
Regenerate `Cargo.lock` through the prescribed tool if the combined manifests make it stale.
Run the spec’s acceptance checks and the affected `skein-forge-machine-world` suite through heavy; retain test assertions and counts.
