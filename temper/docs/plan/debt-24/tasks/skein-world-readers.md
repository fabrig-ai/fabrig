+++
id = "skein-world-readers"
part = "skein-kits"
kind = "migration"
needs = ["llm-kit", "tls-kit", "forge-kit", "world-exemptions"]
files = ["smith/**/Cargo.toml", "smith/**/*.rs", "temper/**/Cargo.toml", "temper/**/*.rs", "jig/tests/channel/Cargo.toml", "jig/tests/channel/src/**", "jig/testing/jig-test/Cargo.toml", "jig/testing/jig-test/src/**", "jig/testing/jig-test/tests/**", "jig/tests/boundary/src/tests/kits.rs", "Cargo.lock"]
effort = "medium"
minutes = 30
+++

# Smith, temper and jig read skein's kits

## Goal

The other trees' edges onto `skein-llm-world`, `skein-tls-world` and `skein-forge-machine-world`: 22 packages, about 70 files, all a crate's name. The kits are in place, and the worlds' re-exports have kept these compiling until now. By `temper/docs/plan/debt-24/notes/kits.md`, step 4.

Acceptance:

- `rg -n '^skein-(llm|tls|forge-machine)-world\b' smith temper jig -g Cargo.toml` reads nothing, and `rg -l 'skein_(llm|tls|forge_machine)_world' smith temper jig -t rust` reads only `smith/benchmarks/tasks/repository/shell-cwd/grader/src/lib.rs`.
- **That grader stays:** it's outside the workspace, and its source is pinned at an older revision.
- `temper`'s unused `skein-llm-world` line goes, read nowhere.
- `kits.rs` loses `jig-channel-world` to `skein-llm-world` and `jig-test` to `skein-tls-world`: the Measure falls by 2.
- `heavy cargo check --tests` passes on each package changed, naming them; `heavy cargo nextest run -p jig-boundary`, `-p smith-protocol-llm-world`, `-p temper-service --lib` and `-p temper-engine-protocol-forge-world` pass.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `skein/docs/development/debt/D028-worlds-as-kits.md`

## Owns and may change

- **Owns:** in those paths, a manifest's dependency line and a crate path's name, nothing else. This departs from "a branch stays in its tree" for this task's crate names alone. No other line changes.
- **Other drafts hold temper's test files at once** (`temper/crates/temper/tests/fake_backends/**`, `temper/crates/temper-service/src/tests/**`, `temper/tests/engine/**`). A `use` line is all you touch. If trunk's merge at your run's start has added a use, the search finds it. Run the searches once more before you push.
- **The package names in `tools/tiers/packages.toml`** stay.

## Steps

A one-shot untracked `sed` per world over `rg -l`; manifests; `cargo metadata --offline --format-version 1 >/dev/null`; `cargo fmt`; for each tree, `python3 -I -B <tree>/scripts/old_forms.py`; the checks above. `heavy cargo clippy` on `jig-boundary` alone: the rest changed only in a name. Commit and push.

## Report

The common note's items; the packages and file counts. A rename: no rule to break by hand.

## Rebase (2026-11-05, step merge, foreman debt-24/skein-world-readers/foreman-1)

preserve trunk’s forge connection seam while retaining the kit migration
The merge conflicts in `skein/tests/forge/machine/Cargo.toml` and `skein/tests/forge/machine/src/lib.rs`.
The branch carries forge-kit commit `88a4b1e711`; trunk carries its rebased counterpart `7c2a927c25`, preserving the seam added by `38d49979da` (“Expose forge connection state only to the machine world”).
Resolve these two paths to trunk’s versions:
- In `Cargo.toml`, retain `skein-forge-connection = { workspace = true, features = ["seam"] }`, together with the kit dependency and migrated dependency placement.
- In `src/lib.rs`, retain `pub mod hand;`, `pub mod memory;`, `pub use skein_forge_connection::seam::GitHub;` and `pub use skein_forge_kit::*;`.
Keep the task’s smith, temper and jig reader renames; do not replay the older forge-kit extraction over trunk’s landed extraction.
Regenerate `Cargo.lock` through `cargo metadata --offline --format-version 1 >/dev/null` if the combined manifests make it stale.
Repeat the spec’s acceptance searches and required checks after rebasing.
