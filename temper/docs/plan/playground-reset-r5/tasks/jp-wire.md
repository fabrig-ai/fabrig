+++
id = "jp-wire"
part = "jig-remove-project"
kind = "feature"
needs = ["jp-acts", "jp-people"]
files = ["jig/crates/jig-client-protocol-wire/**", "jig/crates/jig-client-domain/src/**", "jig/crates/jig-client-protocol-link/src/**", "jig/crates/jig-protocol-engine-clients/src/**", "jig/testing/jig-client-scripted-engine/src/**", "jig/testing/jig-client-test-kit/src/**"]
effort = "high"
minutes = 80
+++

## Goal

Carry the owners' `Ask::RemoveProject` and its answer across jig's
client wire (draft `temper/docs/design/draft/playground-reset.md`,
section 5, "its client"), so a client can ask to delete a project and
hear `ProjectRemoved`: the client contract's ask and outcome, the wire's
records, both translators and the client test kits. The page's Delete
control is task `jp-view`'s.

Acceptance:

- `jig_client_domain::contract::Ask::RemoveProject { project: u32 }` and
  the owner outcome `ProjectRemoved { project: u32 }`
  (`contract/owner_outcome.rs`, beside `Renamed`), each with a doc line
  citing the draft's section 5 as `Rename`'s cites its contract section.
- The wire: `client-v19-part1.schema`'s `Ask` gains `remove_project:
  RemoveProject` and `client-v19.schema` gains `record RemoveProject {
  project: u32 }` and `OwnerOutcomeProjectRemoved { project: u32 }` in
  `OwnerOutcome`, changed in place (as commit `3a2461deb6` changed
  `client-v19.schema`); generated code regenerated with the crate's
  generator (its header names the command).
- Translators: `jig-client-protocol-link` (`translate/asks.rs` beside
  `Rename`, `answers.rs`, and every exhaustive match S4 section 4 lists:
  `store.rs`, `documents_part2.rs`, `compound.rs`); `jig-protocol-engine-clients`
  (`translate/asks.rs` to `people::Ask::RemoveProject`, `answers.rs` from
  `people::Outcome::ProjectRemoved`, and its `request.rs` matches).
  `people::Outcome::ProjectRemoved` reaches the client as the owner
  outcome; for the deployment's act it never does (acts have no client).
- The client test kits answer it (`jig-client-scripted-engine`'s
  `requests/projects.rs` beside rename: the project leaves its scripted
  lists; `jig-client-test-kit`'s `check.rs` names it).
- Every exhaustive match over the client contract's `Ask` and owner
  outcome compiles, in jig (`jig-client`, `jig-client-view`, the client
  worlds) and temper (`temper-client-tools`, `temper-client-view`'s
  `cards.rs`, `temper/tools/gallery`), the new arm shaped as `Rename`'s
  neighbour: words for the outcome "Deleted project <name or number>" where
  an arm must say something.
- Tests: the link's round trip (`jig-client-protocol-link`'s
  `src/tests/projects.rs`, beside rename's); the engine side's
  translation (`jig-protocol-engine-clients`' tests where `Rename`'s is);
  the wire's golden or round-trip case where the wire crate keeps one per
  record.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 1,
  "the owner's ask").
- `temper/docs/plan/playground-reset-r5/notes/jig-remove-project.md`,
  section 2, step 1.
- `jig/docs/design/client/domain/contract.md` (its asks and owner
  outcomes, where `Rename` is); `jig/docs/design/protocol/client.md`.
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s4-jig-projects.md`,
  section 4, layers 2 to 5 and the client readers: every file `Rename`
  and `MakeProject` cross.

## Owns and may change

- Owns the files in `files`.
- May change: compiler-forced arms only, in `jig/crates/jig-client`,
  `jig/crates/jig-client-view` (no control: `jp-view` adds it),
  `jig/tests/client*`, `jig/testing/*`, `temper/crates/temper-client-*`,
  `temper/tools/gallery`, `temper/tests/client`.
- Options: if the wire keeps its generator in a test (`SKEIN_CODEGEN_WRITE=1`
  drift test, as the store does), run it that way and name it; if a
  `ProjectRemoved` outcome must carry the project's name for the words,
  carry the number only and let the client name it from its last
  document.

## Steps

1. Contract, wire schema in place, regeneration.
2. Both translators and their tests: `heavy cargo nextest run -p
   jig-client-protocol-link --lib -E 'test(/projects/)'`; the engine
   side's narrow run.
3. Kits and forced arms; `heavy cargo check --tests` for each package
   touched (`-p jig-client-view -p jig-client -p temper-client-tools -p
   temper-client-view -p temper-gallery` and the client worlds).
4. `cargo fmt`; `heavy cargo clippy` naming every package changed, with
   `--all-targets -- -D warnings`; the focused tests of
   `jig-client-protocol-wire`, `jig-client-protocol-link`,
   `jig-protocol-engine-clients`, `jig-client-domain`, `jig-client-world`,
   `jig-client-protocol-world`. Commit and push.

## Report

As `notes/common.md` says; name the generator run and every file outside
the owned list you edited for forced arms.
