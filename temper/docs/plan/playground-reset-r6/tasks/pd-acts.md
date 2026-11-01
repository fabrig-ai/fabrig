+++
id = "pd-acts"
part = "pd"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-people/**",
  "jig/crates/jig-protocol-store/schema/people-v13.schema",
  "jig/crates/jig-protocol-store/src/codec/people.rs",
  "jig/crates/jig-protocol-store/src/codec/people/**",
  "jig/crates/jig-protocol-store/src/keys/people.rs",
  "jig/crates/jig-protocol-store/src/intervals.rs",
  "jig/crates/jig-protocol-store/src/restore.rs",
  "jig/crates/jig-protocol-store/src/restore/reader.rs",
  "jig/crates/jig-protocol-store/src/generated/people_v13.rs",
  "jig/crates/jig-protocol-store/src/generated/layouts.rs",
  "jig/crates/jig-protocol-store/golden/people/**",
  "jig/crates/jig-protocol-store/src/tests/**",
  "jig/crates/jig-protocol-config/src/acts.rs",
  "jig/crates/jig-protocol-config/src/tests/acts.rs",
]
effort = "high"
minutes = 110
+++

## Goal

Add the people vocabulary of a project's deletion and of J8's two acts, with their store
codecs (people v13 changed in place), the configuration's two grant words and the deletion
row kept by the people child. Every other reader gets the arm the compiler asks for; no
deletion is decided yet (the core refuses the new act and request `NotOffered` until
`pd-core-decide`).

Acceptance:
- `jig_domain_people` has exactly the shapes of contracts J1, J2, J4 (the row) and J8:
  `Act::RemoveProject { project: u32 }` and `Act::Cancel { project: u32, task: u64, reason:
  Box<[u8]> }` appended after `Adopt` in that order; `Act::AddKey` gains `uses:
  Option<Box<[Use]>>` (people's own `Use`); `ActKind::RemoveProject` and `ActKind::Cancel`
  appended after `MakeProject`, and `ActKind::ALL` ends `…, MakeProject, RemoveProject,
  Cancel`; `Ask::RemoveProject { project: u32 }`; `Outcome::ProjectRemoved { project: u32 }`;
  `Stage::{Ending, Erasing, Releasing}`; `Stored::Deletion { project: u32, stage: Stage }`
  and `Key::Deletion(u32)` (appended last, `live()` true).
- The people child keeps the deletion rows: restored at start, readable by
  `Domain::deletion(project) -> Option<Stage>` and `Domain::deletions() -> &Map<u32, Stage>`,
  and written by `Event::Deletion { project: u32, stage: Option<Stage> }`. It also has the
  events `Event::EraseTask { task: u64, project: u32 }` and `Event::DropProject { project: u32 }`
  whose bodies are `pd-people-purge`'s: here they change nothing and emit nothing.
- The configuration parses `remove_project` and `cancel` as grant words.
- The store encodes and decodes every new value; a `Stored::Deletion` row restores at start.
- jig and temper build; the named tests pass.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md` (all of it).
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J1, J2, J4, J8.
- `temper/docs/design/draft/playground-reset.md`, section 5.
- `jig/docs/design/domain/people.md`, 5.1 and 5.3 (keyed requests, deployment acts).
- `jig/docs/design/protocol/store.md`, 5 (versions change in place before the first release)
  and 7.2 (restart ranges).

## Owns and may change

Owned: the `files` above. In `jig-domain-people`: `boundary.rs`, `domain.rs`,
`domain/acts.rs`, `domain/start.rs`, `domain/start/fit.rs`, `capacity.rs`, `words.rs`,
`batch.rs`, `models.rs`, `lib.rs` and its `src/tests/`.

Reader files, compiler-forced arms only (common.md), plus what is said here:
- `jig/crates/jig-domain-core`: every exhaustive match on `people::Act`, `ActKind`, `Ask`,
  `Outcome`, `Stored`, `Key`, `Event` (survey: `routing/deployment_acts.rs` digest match and
  `route`, `routing/people.rs` (Ask route, Outcome, Key erase), `routing/application_routes.rs`,
  `routing/watch_marks.rs` `touch_people_row`, `routing/party_controls.rs`, `restart/census.rs`,
  `acts/requests.rs`, `acts/standing.rs`, `roles.rs`, `person_task.rs`,
  `routing/vocabulary.rs` `people_new_work`). The new act and request route to
  `Outcome::Refused(Refusal::NotOffered)`; `acts/standing.rs` judges `Ask::RemoveProject` as
  it judges `Rename` (project's `Owner`, `Unknown` when `!core.people.has_project`). The new
  people events are not new work. Also, not compiler-forced: `restart/cursor.rs` maps
  `Key::People(Deletion(_))` to the same interval index as `Acted`.
- `jig/crates/jig-protocol-engine-clients`: `translate/answers.rs` `owner()` maps
  `Outcome::ProjectRemoved` to `Err(problem())` (the wire carries it from `pd-client-wire`);
  `request.rs` arms for `Ask::RemoveProject` beside `Ask::Rename`'s.
- `jig/testing/jig-test-domain/src/routes.rs` (`application_answer` Outcome arm),
  `jig/testing/jig-test-service/src/clients.rs` (people `Key`), `jig/examples/ops/domain/src/route.rs`
  (Outcome arm), and the worlds' exhaustive matches: `jig/tests/people/src/world.rs`,
  `jig/tests/client/src/joint/wire/cursor.rs`, `jig/tests/core/src/watches.rs`, and any other
  the compiler names.
- temper: `temper/crates/temper-engine-domain/src/store.rs` (`record_bytes`: `Stored::Deletion`
  beside `SignIn`'s `Some(0)` group; `Ask::RemoveProject` beside `Rename`; act sizes:
  `RemoveProject` and `Cancel` as `RemoveAccount` and `Close` are sized, `AddKey`'s `uses` its
  owned bytes; `program_answer_bytes`: `ProjectRemoved` beside `Renamed`),
  `temper/crates/temper-engine-domain/src/route/demo.rs` (`end_acted`: `ProjectRemoved` as
  `Renamed` is treated), any `people::Act::AddKey` literal in temper (`uses: None`), and
  `temper/tests/engine/domain/src/{roles.rs,roles_referee.rs}` exhaustive `Stored` matches.
  The report lists each temper file.

Options, pre-approved:
- The people restart interval: default, widen the second people interval of
  `intervals.rs` from the `[2,8]` prefix to `[2,8]..[2,10)` (Acted and Deletion), keeping two
  people intervals, so `restore/reader.rs`, `restore.rs` `people_interval` and core's
  `restart/cursor.rs` count stay as they are. If a test shows the secrets sweep or the census
  depends on the old bound, add a third interval `prefix [2,9]` instead and set the count to 3
  in each of those places; say so.
- If a `people::Act::AddKey` literal outside the files named needs `uses`, give it `None`.

## Steps

1. `boundary.rs`: the variants and types above, in the stated order. `Stage` derives
   `Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug`. `Stored::key()` maps
   `Deletion { project, .. }` to `Key::Deletion(project)`. Doc comments cite
   `draft playground-reset.md, 5` and contracts' words (`a deleting project's stage`).
2. Acts (`domain/acts.rs`): `valid` — `RemoveProject` refuses `Unknown` for project 0; `Cancel`
   as `Close` (task 0 `Unknown`, reason over `limits.words` `Limit { Words }`); `AddKey` with
   `uses: Some(list)` refuses `Limit { which: LimitKind::Resource }` when `list` has more than
   `USES` entries or a duplicate. `kind` maps the new acts. `valid_outcome` (the restore
   check): `RemoveProject` accepts `ProjectRemoved { project }` of the same project and
   `Refused(_)`; `Cancel` accepts what `Close` accepts. `fit::act`: both new acts as
   `RemoveAccount` (no owned bytes beyond the reason for `Cancel`, as `Close`); `AddKey`'s uses.
3. Request (`Ask::RemoveProject`): in every people match, beside `Ask::Rename` (its project is
   `project`, its words none, its answer shape `ProjectRemoved` of the same project).
4. The row: `Domain` gains `deletions: Map<u32, Stage>` with capacity `limits.projects`.
   `Event::Restore` of `Stored::Deletion` inserts it (refused as a duplicate row is refused;
   project 0 refused `Unknown`); `fits` (start) admits it within `limits.projects`;
   `Event::Deletion { project, stage: Some(s) }` replaces the entry and emits
   `Request::Save { record: Stored::Deletion { project, stage: s } }`; `stage: None` removes it
   and emits `Request::Erase { key: Key::Deletion(project) }` (absent: nothing).
   `Event::EraseTask` and `Event::DropProject`: accepted after restore, no output.
5. Codecs (people v13 in place, `jig-protocol-store`): append to `people-v13.schema` the
   records and enum alternatives for the two acts, `AddKey`'s optional uses (a list of at
   most 16 of the existing use form if one exists in the schema; otherwise a new
   `Use { party | project }` enum), the ask, the outcome, `Stage`, and the `Deletion` row value;
   regenerate with `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-store-codecs-world
   --test suite -E 'test(/drift/)'` (it rewrites `src/generated/people_v13.rs`, the people
   goldens and `layouts.rs`); add the encode and decode arms in `codec/people.rs`,
   `codec/people/acts.rs`, `codec/people/asks.rs`, `codec/people/outcomes.rs`; the key
   `Key::Deletion(p)` as tag 9 (`writer.tag(9); writer.u64(u64::from(*p))`, read as `Roles`
   is) in `keys/people.rs`; the interval (options). Extend `src/tests/fixtures/keys.rs`'s
   `people_key` generator to yield `Acted` and `Deletion`.
6. Config: `b"remove_project" => ActKind::RemoveProject`, `b"cancel" => ActKind::Cancel` in
   `jig-protocol-config/src/acts.rs`.
7. Tests:
   - people (`src/tests/`, new `deletion.rs`): `a_deletion_row_is_saved_restored_and_erased`
     (Event::Deletion Some, then a fresh domain restoring the saved row reads its stage, then
     None erases it); `a_deployment_act_needs_its_grant_for_both_new_kinds` (admit
     `RemoveProject` and `Cancel` refused `NotOffered` without a grant, routed with one);
     `an_add_key_with_too_many_uses_is_refused`.
   - store (`src/tests/codec.rs`): extend the every-`Act` round trip with both acts and
     `AddKey { uses: Some([Use::Project(3)]) }`; extend the ask/outcome pairs with
     `RemoveProject`/`ProjectRemoved`; new `a_deletion_row_round_trips_at_each_stage`; in
     `src/tests/keys.rs` new `the_deletion_key_takes_people_tag_nine` (modelled on the `Acted`
     tag test); in `src/tests/intervals.rs` people ranges include `Deletion`.
   - config (`src/tests/acts.rs`): `a_valid_grant_keeps_every_kind_including_make_project`
     gains `"remove_project","cancel"` and the two kinds.
   - Commands, one at a time:
     `heavy cargo nextest run -p jig-domain-people --lib`,
     `heavy cargo nextest run -p jig-protocol-store --lib`,
     `heavy cargo nextest run -p jig-store-codecs-world --test suite`,
     `heavy cargo nextest run -p jig-protocol-config --lib`,
     `heavy cargo nextest run -p jig-people-world --test suite`,
     `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/deployment_acts/)'`.
8. `cargo fmt`; `heavy cargo clippy` on every changed package, temper's included
   (`-p temper-engine-domain -p temper-engine-domain-world --all-targets`); old forms
   (common.md); commit and push.

## Report

List every file outside `files` you changed and why (compiler-forced or the named edits), and
which interval option you took.
