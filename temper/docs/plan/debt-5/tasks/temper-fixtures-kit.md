+++
id = "temper-fixtures-kit"
part = "temper-items"
kind = "migration"
needs = []
files = [
  "temper/testing/**",
  "temper/tests/engine/domain/src/lib.rs",
  "temper/tests/engine/domain/src/tier_task_show.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/tier_task_show.rs",
  "temper/tests/engine/domain/Cargo.toml",
  "temper/docs/development/debt/D010-path-shared-fixtures.md",
  "temper/docs/development/debt/README.md",
]
effort = "medium"
minutes = 60
+++

# temper D010: shared fixtures are a kit crate, not `#[path]` includes

## Goal

The files compiled into several crates by `#[path]` become one kit crate, `temper-fixtures`, at `temper/testing/temper-fixtures`; `turn_bounds`, which the playground holds already, loses its copy. `tier_task_show` becomes a module of the engine world's library, which the client world already depends on. Each fixture compiles once, its users show in `Cargo.toml`, and the `dead_code` expectations on the includes go. This task closes the item.

Acceptance:

- `rg -c --type rust '#\[path = "[^"]*(\.\./testing/|real-loop/|engine/domain/)' temper | awk -F: '{s+=$2} END {print s+0}'` prints 0.
- `temper/testing/` holds only crates: `ls temper/testing/*.rs` finds nothing.
- `rg -n 'shared document fixtures|shared ctl fixture|shared transcript' temper --type rust` reads nothing.
- `temper/docs/development/debt/D010-path-shared-fixtures.md` and its row are gone.
- `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-tools -p temper-client-shell-terminal -p temper-client-gallery` passes, its count above zero.
- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(forge_routes::tiers::)'` and `heavy cargo nextest run -p temper-client-world --test suite -E 'test(clearing::)'` pass, each count above zero.
- `heavy cargo nextest run -p temper-real-loop-world --test suite -E 'test(template_bounds::)'` passes, its count above zero.
- `heavy cargo check --tests -p temper -p temper-client-browser-world -p temper-one-process-world -p temper-real-loop-world -p temper-client-world -p temper-engine-domain-world -p temper-service` passes.
- `heavy cargo nextest run -p fabrig-boundary` passes.
- `heavy cargo clippy -p temper-fixtures -p temper-engine-domain-world --all-targets -- -D warnings` passes, and on the packages of the first test line.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `temper/docs/development/debt/D010-path-shared-fixtures.md`, whole.
- `temper/docs/design/README.md`, the paragraph on test kits ("named by their kind"), and `temper/docs/design/domain/testing.md`, section 7, its table of kits and worlds.
- `temper/testing/temper-github-app/{Cargo.toml,clippy.toml}`: the shape to copy.

The item leaves two choices, and the cut read them so:

- **One kit.** `browse` goes in `temper-fixtures` with the three files. Nothing builds it for wasm32: `temper/tools/gallery` is a host tool, and the other users are tests. So one crate with `jig-client`, `skein-lib` and `skein-io` is simplest, and it needs no features.
- **`turn_bounds` goes to no kit.** `temper-playground`'s public `documents::turn_bounds` already has the same code, and it is a library the worlds share (the item's words). The real loop's copy goes, and its users name the playground's. So the client crates' tests and the gallery link neither smith's agent shell nor the engine's domain.
- **`tier_task_show` goes to the engine world's library,** the lowest crate that names its types and one the client world depends on already. `temper-engine-domain-world` moves `temper-client-tools` from `[dev-dependencies]` to `[dependencies]`. No crate that `temper-client-tools` depends on names a world, so nothing cycles.

## Owns and may change

- Owns: `temper/testing/temper-fixtures/**` (new), and the moved files by `git mv` so history follows.
- Root `Cargo.toml`: the member line and the `[workspace.dependencies]` line, beside `temper-github-app`'s. `Cargo.lock` as cargo writes it.
- Each user's `Cargo.toml`: one `temper-fixtures.workspace = true` line, in `[dependencies]` for `temper-one-process-world`, `temper-real-loop-world` and `temper-client-gallery`, and in `[dev-dependencies]` for `temper-client-domain`, `temper-client-view`, `temper-client-tools` and `temper-client-shell-terminal`.
- For `turn_bounds`: `temper-playground.workspace = true` in `temper-real-loop-world`'s `[dependencies]` and `temper`'s `[dev-dependencies]`. `temper-client-browser-world` has it already, and drops `smith-agent-shell` and `temper-engine-domain`, which only its include named.
- Each includer, its `#[path]` lines and the `#[expect(dead_code, ..)]` on them, and nothing else.
- `temper/tests/engine/domain/Cargo.toml`: move to `[dependencies]` what `tier_task_show.rs` now needs.
- `temper/docs/design/domain/testing.md`, section 7: one table line, in step 5.
- Pre-approved: `pub(super)` and `pub(in ..)` items of the moved files become `pub`. Workspace lints the includers' test modules hid are met by a crate-level `#![expect(.., reason = "...")]` in the kit, or by `clippy.toml` as `temper-github-app` has it. A fixture item nothing uses any more goes, and the report says which.
- Other `#[path]`s stay: those that name a file of their own crate (`temper/tests/support/...`, `src/agent.rs`, `host_tests_*.rs`) are not the item's.

## Steps

1. `temper/testing/temper-fixtures/`: `Cargo.toml` as `temper-github-app`'s, with `description = "The documents, transcripts and harness helpers temper's client crates and worlds share"`, dependencies `jig-client`, `skein-lib`, `skein-io`; a `clippy.toml` as that crate's, with a first comment saying the kit is ordinary Rust; `src/lib.rs` holding a module doc and:

   ```rust
   pub mod browse;
   pub mod client_documents;
   pub mod first_run;
   pub mod memory_literal;
   ```

   `git mv` `temper/testing/{client_documents,first_run,memory_literal}.rs` and `temper/tests/real-loop/src/calls/browse.rs` into its `src/`. `git rm temper/tests/real-loop/src/turn_bounds.rs`: its code is `temper/crates/temper-playground/src/documents/turn_bounds.rs`'s, which stays as it is.

2. The includers. Keep each local module name, so no reader of it changes:

   | File | Becomes |
   |---|---|
   | `temper-client-view/src/tests.rs`, `temper-client-shell-terminal/src/tests.rs`, `temper-client-domain/src/tests.rs`, `temper-client-domain/tests/suite/main.rs`, `temper-client-tools/src/tests.rs`, `temper/tools/gallery/src/worlds.rs` | `use temper_fixtures::client_documents as <its module's name>;` |
   | `temper-client-view/src/tests/fixtures.rs`, `temper-client-shell-terminal/src/tests/fixtures/chat.rs` | `use temper_fixtures::first_run as <its module's name>;`, `pub` as the module was |
   | `temper-client-view/src/tests/accounts.rs` | the `#[cfg(target_pointer_width = "64")]` stays on `use temper_fixtures::memory_literal;` |
   | `temper/tests/client-browser/src/configuration.rs` | `use temper_playground::documents::turn_bounds;` |
   | `temper/tests/real-loop/src/fixtures.rs` | `pub use temper_playground::documents::turn_bounds;` in place of its `#[path]` module |
   | `temper/crates/temper/tests/support/configuration.rs` | `pub use temper_playground::documents::turn_bounds;` |
   | `temper/tests/real-loop/src/calls.rs` | `pub use temper_fixtures::browse::Browsed;`, `use temper_fixtures::browse::Browser;` |
   | `temper/tests/one-process/src/ctl.rs` | `use temper_fixtures::browse;` |

3. `tier_task_show`: `git mv` to `temper/tests/engine/domain/src/tier_task_show.rs`; `pub mod tier_task_show;` in the world's `src/lib.rs`; its `temper_engine_domain_world::` paths become `crate::`, and what `tiers.rs` and `clearing.rs` call becomes `pub`. `tests/suite/forge_routes/tiers.rs` takes `pub(super) use temper_engine_domain_world::tier_task_show as task_show;` and `temper/tests/client/tests/suite/clearing.rs` takes `use temper_engine_domain_world::tier_task_show as task_show;`, its `dead_code` expectation gone.

4. `python3 -I -B scripts/old_forms.py` from `temper/`; lower a count your change lowers.

5. `temper/docs/design/domain/testing.md`, section 7's table, beside the `testing/temper-github-app` line: `testing/temper-fixtures             the documents, transcripts, memory literals and local browser that temper's client crates and worlds share`.

6. The item's file and its row; citations. Run what the acceptance names; commit (`D010: shared fixtures are a kit crate`, saying why; the design line and debt text in a second commit) and push.

## Report

The measure's final value, the `dead_code` expectations left in temper and why, each fixture item removed as unused, and the lint expectations the kit took.

## Answers

2026-11-04. May `temper-fixtures` also depend on `smith-agent-shell` and `temper-engine-domain`, which `turn_bounds` needs? No. `temper-playground/src/documents/turn_bounds.rs` is the same code and public, so the real loop's copy goes and `temper-client-browser-world`, `temper-real-loop-world` and `temper`'s tests name the playground's. Adding the two would put smith's agent shell and the engine's domain into the client crates' tests and the gallery, which link neither today, and keep a duplicate. `serde_json` left the kit's list too: only `turn_bounds` used it. Goal, Inputs, Owns, steps 1, 2 and 5 above say this.
