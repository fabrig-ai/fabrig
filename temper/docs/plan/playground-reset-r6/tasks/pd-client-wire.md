+++
id = "pd-client-wire"
part = "pd"
kind = "interface"
needs = ["pd-vocabulary", "pd-test-connector"]
files = [
  "jig/crates/jig-client-domain/**",
  "jig/crates/jig-client-protocol-wire/**",
  "jig/crates/jig-client-protocol-link/**",
  "jig/crates/jig-protocol-engine-clients/**",
  "jig/tests/client-wire/**",
  "jig/tests/engine-clients/**",
]
effort = "high"
minutes = 100
+++

## Goal

jig's client contract, wire (client-v19, changed in place), link and engine-clients carry the
owner's `RemoveProject { project }` request, its `ProjectRemoved { project }` outcome and its
`RemoveProject` offer kind, exactly as they carry `Rename`'s (contracts J7, J2).

Acceptance:
- `jig_client_domain`: `contract::Ask::RemoveProject { project: u32 }` (after `MakeProject`),
  `OwnerOutcome::ProjectRemoved { project: u32 }` (after `ProjectMade`), `ActKind::RemoveProject`
  (appended last; `MAX_OFFERED_ACTS` stays 25, which `ActKind` then fills exactly: assert it in
  a test); `requests.rs` `kept()` lists the request as kept, as `Rename`.
- Wire, client-v19 in place (`schema/client-v19.schema`, `client-v19-part1.schema`): record
  `RemoveProject { project: u32 }`, `Ask` alternative `remove_project` (after
  `make_project`), record `OwnerOutcomeProjectRemoved { project: u32 }` and `OwnerOutcome`
  alternative `project_removed` (after `project_made`), `ActKind` alternative
  `remove_project` (appended); generated `src/generated/v19.rs` and goldens regenerated; the
  historical schemas untouched.
- Link: `translate/asks.rs` `ask()`, `translate/store.rs` `restore_ask()`,
  `translate/answers.rs` `owner_outcome()`, `translate/documents_part2.rs` `act_kind_fields()`,
  `translate/compound.rs` (refused as a compound operation in both directions, as `Rename`).
- Engine-clients: `translate/asks.rs` (wire to `people::Ask::RemoveProject`),
  `translate/answers.rs` `owner()` (replacing `pd-acts`' `Err` arm), `translate/task.rs`
  `act()` (replacing `pd-vocabulary`'s `Err` arm), `request.rs` arms as `Rename`'s,
  `translate/llm.rs` `refused()` (no model check).
- Readers elsewhere get compiler-forced arms only, each routed as `Rename`'s arm is
  (common.md): `jig-client-view` (`owner_form.rs`, `sections/project/owner.rs`,
  `activity/sending.rs`, `outcome.rs`, `owner_outcome.rs`, `owner_outcome_words.rs` (a new
  phrase id after the last used, 6475 per the survey: check), `cards/common.rs`, `confirm.rs`,
  `confirmation.rs`, `acts.rs`), `jig/testing/jig-client-scripted-engine`,
  `jig/testing/jig-client-test-kit`, `jig/testing/jig-test-domain/src/routes.rs` (after
  `pd-test-connector`, hence its need),
  `jig/crates/jig-client/src/{facts.rs,thread/model.rs}` (`dangerous`: a deletion is
  dangerous), `jig/tests/client/src/**` harness matches, `jig/tests/client-protocol/src/front/observe.rs`;
  temper: `temper/crates/temper-client-tools/src/jig/render/{acts.rs,outcome.rs,document/admissions.rs,document/act_answers.rs}`
  (`remove_project`), `temper/crates/temper-client-view/src/cards.rs`,
  `temper/tools/gallery/src/worlds.rs`. The view's Delete form is `pd-client-view`'s: here the
  view only compiles. The report lists each temper file.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J2, J7.
- `jig/docs/design/protocol/client.md`, the versions rule (~432, ~769: v19 changes in place).
- `jig/docs/design/client/domain/contract.md`, the asks table (~226).

## Owns and may change

Owned: the `files` above. Reader files: the arms named above only.

Options, pre-approved:
- If a client-view phrase table must name the new outcome, add the phrase "Deleted the
  project" with the next free id; if the id range header must move, move it and say so.
- If a fuzzy wire check lists records by hand (`jig/tests/client-wire/tests/suite/fuzzy/client_wire.rs`),
  add the two records beside `Rename`'s and `OwnerOutcomeRenamed`'s.

## Steps

1. Contract, schema, regenerate:
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world --test suite -E 'test(/drift/)'`
   (then the same without the variable must pass).
2. Link and engine-clients arms; forced arms elsewhere.
3. Tests, beside `Rename`'s:
   - link `src/tests/projects.rs`: `a_project_removal_round_trips_and_is_no_compound_operation`
     (contract ask to wire and back, outcome, `act_kind_fields`), and `src/tests/store.rs`'s
     restore table test with a kept `RemoveProject`.
   - engine-clients `src/tests/asks.rs`: `a_project_removal_reaches_people_with_its_project`;
     `src/tests/answers.rs`: `project_outcomes_keep_the_project_number` gains `ProjectRemoved`.
   - client-domain: `offered_acts_fill_their_bound_exactly` (`ActKind` count equals
     `MAX_OFFERED_ACTS`).
   Commands, one at a time: `heavy cargo nextest run -p jig-client-protocol-link --lib`,
   `heavy cargo nextest run -p jig-protocol-engine-clients --lib`,
   `heavy cargo nextest run -p jig-client-domain --lib`,
   `heavy cargo nextest run -p jig-client-wire-world --test suite`,
   `heavy cargo nextest run -p jig-engine-clients-world --test suite`,
   `heavy cargo nextest run -p jig-client-view --lib`,
   `heavy cargo nextest run -p jig-client-world --test suite`.
4. `cargo fmt`; `heavy cargo clippy --all-targets` on every changed package, temper's
   included; old forms; commit and push.

## Report

Every file outside `files` you changed; the phrase id used.
