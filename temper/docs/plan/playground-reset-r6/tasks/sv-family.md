+++
id = "sv-family"
part = "rf"
kind = "feature"
needs = ["cl-wire", "fe-surface"]
files = ["temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/family/**", "temper/crates/temper-service/src/clients.rs", "temper/crates/temper-service/src/tests/family.rs"]
effort = "medium"
minutes = 50
+++

## Goal

The client family's translation of T8's new requests and the engine's answers to them
(`notes/contracts.md`, T4 to T6 and T8; `temper/docs/plan/playground-reset-r6/notes/cl-wire.md`,
2 and 3): `make_forge`, `remove_forge` and `end_playground` reach the root as their forge
asks instead of the at-once `not_offered` `cl-wire` left, and their answers reach the client
as the wire's outcomes. `Read::playground` is not this task's (`sv-playground`).

Acceptance:
- `family::to_root`: `w::Request::MakeForge(v)` → `root::Application::Forge(forge::Ask::
  MakeForge { choice })` with `w::ForgeChoice::{Owned, Github, Forgejo(c)}` →
  `forge::ForgeChoice::{Owned, GitHub, Forgejo { api, git, web }}` (the wire's bytes moved
  in); `w::Request::RemoveForge(v)` → `forge::Ask::RemoveForge { forge }`;
  `w::Request::EndPlayground` → `forge::Ask::EndPlayground`; each with the request's
  `scope`, `reply_to`, `sign_in`, `key`, `digest` and `carried`, as `SetUp` is built.
  `w::Request::StartPlayground` stays as `cl-wire` routes it (`forge::Ask::StartPlayground`),
  its key file still read first (`clients.rs`).
- `family::answer` encodes, per `notes/cl-wire.md`, 3: `MakeAnswer::Made { forge }` →
  `w::Outcome::Made(w::MadeForge { forge })`; `Refused(MakeRefusal::Full)` → `Full`;
  `Refused(MakeRefusal::NotOffered)` → `NotOffered`; `RemoveAnswer::Removing` → `Removing`;
  `Refused(RemoveRefusal::NoSuchForge)` → `connect_refused { why: no_such_forge }` (the
  existing outcome, as `Connect`'s refusal encodes it); `Refused(RemoveRefusal::Playground)`
  → `Playground`; `EndAnswer::Reset` → `Reset`; `EndAnswer::NotOffered` → `NotOffered`.
  An answer past retention stays `unknown`.
- The routing arms `fe-surface` and `cl-wire` added as placeholders (rg `not_offered`,
  `NotOffered` in `family.rs` and `clients.rs`) are gone.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/cl-wire.md` (whole: the wire's names and the
  answers' reading).
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T4 (`MakeForge`), T5
  (`RemoveForge`), T6 (`EndPlayground`), T8.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/design/protocol/setup.md`, 8.4 (the demo's start, its key read and its
  answers, which the new requests' answers sit beside).

## Owns and may change

- Owns: `temper/crates/temper-service/src/family.rs` (and `src/family/`),
  `src/tests/family.rs`; `src/clients.rs` only where a placeholder arm for the new
  requests sits.
- The engine's answer types are fe-surface's (`temper_engine_domain_forge::{MakeAnswer,
  MakeRefusal, RemoveAnswer, RemoveRefusal, EndAnswer, ForgeChoice}` and the
  `ApplicationAnswer`/`Answered` variants it added for them): read their names there; if a
  name differs from this spec's, use fe-surface's and say so.

## Steps

1. `family.rs::to_root`: the three request arms (beside `StartPlayground`'s); a helper
   `fn choice(value: w::ForgeChoice) -> forge::ForgeChoice`.
2. `family.rs::answer`: the new arms, each building the wire's outcome with
   `w::MadeForge::new(limits, w::MadeForgeParts { forge })` and the unit variants.
3. Tests in `src/tests/family.rs`:
   - `a_make_forge_of_each_choice_reaches_the_root_as_its_ask`
   - `a_remove_forge_reaches_the_root_as_its_ask`
   - `an_end_playground_reaches_the_root_as_its_ask`
   - `each_new_answer_is_encoded_as_the_wire_reads_it` (every row of `notes/cl-wire.md`, 3,
     decoded back with the wire's decoder)
4. Narrow runs, one at a time:
   `heavy cargo nextest run -p temper-service --lib -E 'test(/tests::family::(a_make_forge|a_remove_forge|an_end_playground|each_new_answer)/)'`
   (4 tests); then `heavy cargo nextest run -p temper-service --lib -E 'test(/tests::family::/)'`;
   `heavy cargo nextest run -p temper-client-world` (it joins temper-service's family);
   `heavy cargo check -p temper-service --tests`.
5. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets`; old forms (`python3 -I
   -B scripts/old_forms.py` in `temper/`). Commit and push.

## Report

Nothing beyond the implementer's text.
