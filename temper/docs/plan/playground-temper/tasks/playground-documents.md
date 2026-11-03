+++
id = "playground-documents"
part = "composition"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-playground/Cargo.toml",
  "temper/crates/temper-playground/clippy.toml",
  "temper/crates/temper-playground/src/lib.rs",
  "temper/crates/temper-playground/src/documents.rs",
  "temper/crates/temper-playground/src/documents/**",
  "temper/crates/temper-playground/src/fakes.rs",
  "temper/crates/temper-playground/src/fakes/**",
  "temper/crates/temper-playground/src/tests.rs",
  "temper/crates/temper-playground/src/tests/**",
  "Cargo.toml",
  "Cargo.lock",
]
effort = "high"
minutes = 90
+++

# `temper-playground` is made, with the composition's documents as its own

## Goal

The playground's world and the one-process world share one composition, `temper-playground`, a
product crate on product crates alone (`temper/docs/design/playground.md`, 3). This first task
makes the crate and gives it the composition's documents: every value the one-process world's
composition takes from test kits today becomes the playground's own (`playground.md`, 3, "depends
on product crates alone"). Nothing reads the crate yet: the one-process world is rewired by later
tasks, so nothing outside the crate changes but the workspace's two lines.

Acceptance:

- `temper/crates/temper-playground` builds natively, with `[dependencies]` on product crates
  alone (`notes/contracts.md`, C1; `notes/common.md`).
- `temper_playground::documents` and `temper_playground::fakes` hold, as public items, the
  composition's values below, each equal to what the one-process world builds today before its
  harness's edits.
- `documents::seeds(one: u64)` derives the service's, the agents' and the fakes' seeds from one
  seed (C3), deterministically, each different from the others for one seed.
- The crate's unit tests below pass; `cargo tree -p temper-playground -e normal` names no crate
  under `*/tests/*` or `*/testing/*`.

## Inputs

- `temper/docs/plan/playground-temper/notes/common.md` and `notes/contracts.md`, C1 and C3.
- `temper/docs/design/playground.md`, 3 and 4.
- `temper/docs/design/domain/testing.md`, 2.5 (what runs).

## Owns and may change

- The new crate: `Cargo.toml` (lints as `temper/tests/one-process/Cargo.toml`'s table),
  `clippy.toml` (C1), `src/lib.rs` (a module doc citing `temper/docs/design/playground.md, 3`;
  `pub mod documents; pub mod fakes;`), `src/documents.rs` and `src/documents/`, `src/fakes.rs`
  and `src/fakes/`, `src/tests.rs` or `src/tests/`.
- Root `Cargo.toml`: the member line and the workspace dependency, placed as C1 says.
  `Cargo.lock` as cargo writes it.

What moves in, copied (the one-process world keeps its own until `one-process-documents`), from
`temper/tests/one-process/src/` unless named:

- `documents`: the deployment's document as a `serde_json::Value` and its bytes
  (`fixtures.rs`, `document` ~105 and `world_document` ~110, without the harness's own edits at
  ~139 and ~181-183, which stay the one-process world's and which it will apply through
  `Harness::configure`); `files` ~187, `seeds` ~214 (as `seeds(one)`, C3), `agent` ~223,
  `agent_profile` ~253, `agent_limits` ~268, `agent_declarations` ~274, `agent_configuration`
  ~279, `forges` ~298; the constants they need (~10-86) but the harness's (`SEED`, `ANSWER`,
  `BRANCH_HEAD`, `MOVED_BASE`, ...); the stock settings and their helpers (`stories.rs`,
  `configure` ~268, `settings_file` ~488, `given` ~499; `temper/tests/real-loop/src/settings.rs`
  and `turn_bounds.rs`, which the one-process world path-includes: copied as modules of
  `documents`, the real loop keeping its own).
- The test kits' values those use, copied as literals or code of the crate's own:
  `jig_channel_world::fixtures` (`KEY`, the engine and worker documents, `files`, `seeds`,
  `agent`, `agent_configuration`, `provider_turns` if the composition's provider uses it);
  `smith_agent_process_world::{profile, declarations}` and `fake::limits` (not `fake::loopback`,
  which reads the process id: the loopback address is a literal); `skein_llm_world::fake::{config,
  limits}`; the hosted git's `identity`, `CONFIG` and `table` (`temper_fake_git_hosted::fixtures`
  today, `temper_worker_checkout_world::hosted` once `hosted-git-moves` lands: read whichever the
  trunk has).
- `fakes`: the fake LLM peer's connection config and limits (`fixtures.rs`, `scripted_provider`
  ~343 without its scripts: a function taking `Box<[Script]>` and the domain as now), the fake
  forge's table, setup peer and tokens (`restart_control.rs`, `table` ~8, `setup_peer` ~21,
  `ADMIN_TOKEN`, `API_TOKEN`, `GIT_TOKEN`), `peer_limits` ~416, `forge_limits` ~444,
  `stories.rs`'s `forge_table` ~338.

What stays the one-process world's, not copied: the stories' scripts and repositories
(`stories.rs`, `provider(story)`, `forge(story)`, `person`, ...), the client's limits, the codec
and `jig_test_service` limits, account steps, the referee's records. If an item mixes a story's
scripts with the composition's values, the values move and the scripts stay; say which in the
report.

## Steps

1. The crate, its manifests and the root lines; `heavy cargo check -p temper-playground`.
2. `documents` and `fakes`, copied as above, `pub` with a one-line doc each.
3. Unit tests, in `src/tests.rs`:
   - `the_deployment_document_is_one_temper_serve_starts`: its bytes pass the service's
     configuration checks as the one-process world's `service` (~660-716) starts them, up to
     the start's first problem list, which is empty. Use the product function that `service`
     calls; if starting needs a machine, check the configuration's parse and validation alone and
     say so.
   - `one_seed_derives_every_seed`: `seeds(7)` twice equal; its seeds pairwise different;
     `seeds(8)` different from `seeds(7)`.
   - `the_fakes_limits_are_their_constructors'`: the fake LLM peer and the fake forge peer are
     built from `fakes`' values (no scripts) without refusal.
   `heavy cargo nextest run -p temper-playground --lib`.
4. `cargo tree -p temper-playground -e normal` and check it (acceptance).
5. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-playground --all-targets -- -D
   warnings`. Commit; push.

## Report

As the implementer's text says, and: every public item of `documents` and `fakes`, with the
one-process world's item it copies (the next tasks read this list); the crate's `[dependencies]`.
