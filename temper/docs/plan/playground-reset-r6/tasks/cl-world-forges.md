+++
id = "cl-world-forges"
part = "rf"
kind = "feature"
needs = ["cl-pg-goes", "fe-make", "fe-remove", "fe-kept-root"]
files = ["temper/tests/client/**"]
effort = "high"
minutes = 110
+++

## Goal

The joined client world (`temper-client-world`: the real client joined to the real engine,
`temper/docs/design/domain/testing.md`, 6.3) proves forges made by kind, removed, and
started over, on the engine's records (draft `temper/docs/design/draft/playground-reset.md`,
sections 3, 4 and 6.4; `notes/contracts.md` T4, T5).

- The world's bridge (`src/translate/forge.rs`) translates `make_forge` and `remove_forge`
  to `forge::Ask::MakeForge` and `forge::Ask::RemoveForge` and their answers to the wire as
  `notes/cl-wire.md` 3 reads them, exhaustively both ways, as it does `SetUp`'s.
- Its welcome and forge fixtures start from the engine's `Machine` templates with **no**
  forge record (`fe-surface`'s root `Config`), as a first run of `temper init` does, unless
  a test seeds records.
- The world plays serve for the removal's kept acts where the engine world's driver does
  not already: `RemoveGroup` answered `GroupRemoved`, `EraseSecrets` answered
  `SecretsErased` (contracts T5).
- New acceptance cases pass (step 2); every existing case passes, those that named the
  document's forges seeded instead with records made by `MakeForge` (or the test's own
  rows), each listed.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T1, T4, T5,
  T8; `notes/cl-wire.md`; `notes/fe-surface.md` (the root's `Config`, `Machine`, the
  forge `Ask`s and answers).
- `temper/docs/design/domain/testing.md`, 6.3 (the joined world's contract).
- `tasks/cl-choices.md`, `tasks/cl-forges.md`, `tasks/cl-welcome.md` (the client
  behaviour proved here); `tasks/fe-make.md`, `tasks/fe-remove.md` (the engine's).

## Owns and may change

Owned: `temper/tests/client/` (src, tests, fixtures). Not yours: the engine world library
`temper/tests/engine/domain/` (fe's): if it lacks a seam this needs (answering serve's
requests, seeding records), write it in the world's own `src/front.rs` over the driver's
public API; a seam that cannot be had so is a question.

## Steps

1. **Bridge and fixtures:** `translate/forge.rs::request` arms for `MakeForge` and
   `RemoveForge` (the `panic!` arm keeps `start_playground` and `end_playground` until
   `cl-world-pg`); `answered` arms for the engine's `MakeForge`/`RemoveForge` answers;
   `front.rs`/`front/account_acts.rs`/`fixtures.rs`: the root `Config` with `Machine`
   (owned template under the box, the GitHub template, the playground's fake template) and
   no records; a helper `Joint::records(&self) -> Vec<forge::ForgeRecord>` reading the
   engine store's `Forge` rows for assertions.
2. **Cases** (`tests/suite/connect_forge.rs`, `forges.rs`, `welcome.rs`):
   - `run_here_on_a_first_run_makes_forge_one_and_sets_it_up_to_its_first_repository`
     (the welcome's "Run on this machine": one `MakeForge`, `SetUp` of forge 1, the first
     repository adopted, the welcome at Model; the record's `first` kept);
   - `connect_github_on_a_first_run_makes_its_record_then_connects_it`;
   - `a_forgejo_elsewhere_is_made_from_its_addresses_and_opens_its_details`;
   - `remove_on_forges_ends_the_forge_and_it_leaves_the_list` (confirmed; the record
     `Removing` then erased once its kept acts are answered; the row gone from Forges; a
     forge with no open change, so that jig's `Act::Cancel` is not this case's subject);
   - `a_set_up_cut_short_is_unfinished_after_a_reload_and_starts_over` (a forge whose set
     up failed: a reload of the client opens the welcome at Forge, "not finished", Start
     over removes it and the choices return; a second Run here makes forge 2);
   - `github_without_an_installation_goes_on_after_a_reload` (Go on sends `Install`);
   - `an_account_added_before_stays_ready_at_model_after_a_start_over`.
3. `heavy cargo nextest run -p temper-client-world --test suite`; `heavy cargo check -p
   temper-client-world --tests` (the `qa_all`, `qa_model` binaries build; run them only if
   your changes touch their files: `heavy cargo nextest run -p temper-client-world
   --profile qa-temper --test qa_all -E 'test(/forge/)'`). `cargo fmt`; `heavy cargo clippy
   -p temper-client-world --tests`; commit and push.

## Report

- Each existing case whose fixture moved from the document's forges to records.
- Any engine-world seam you wished for (for fe).
