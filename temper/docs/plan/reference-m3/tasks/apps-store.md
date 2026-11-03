+++
id = "apps-store"
part = "appsettings"
kind = "feature"
needs = ["apps-proposals"]
files = ["jig/crates/jig-protocol-store/schema/settings-v1.schema", "jig/crates/jig-protocol-store/src/generated/settings_v1.rs", "jig/crates/jig-protocol-store/src/codec/settings.rs", "jig/crates/jig-protocol-store/src/keys/settings.rs", "jig/crates/jig-protocol-store/src/reads/settings.rs", "jig/crates/jig-protocol-store/golden/settings/**", "jig/crates/jig-protocol-store/schema/people-v14.schema", "jig/crates/jig-protocol-store/src/generated/people_v14.rs", "jig/crates/jig-protocol-store/src/codec/people/**", "jig/crates/jig-protocol-store/golden/people/**", "jig/crates/jig-protocol-store/schema/tasks-v9.schema", "jig/crates/jig-protocol-store/src/generated/tasks_v9.rs", "jig/crates/jig-protocol-store/src/codec/tasks/proposals.rs", "jig/crates/jig-protocol-store/golden/tasks/**", "jig/crates/jig-protocol-store/src/tests/**", "jig/docs/design/protocol/store.status.md", "jig/crates/jig-protocol-store/src/generated/layouts.rs"]
effort = "high"
minutes = 120
+++

# apps-store: the store's records of an application's settings

## Goal

jig's store keeps what the core keeps of an application's settings and nothing of their
values: each setting's state, size and named charters under the settings family's key
(kind, then scope, then number), a revision record's and a proposal's set's application
items as their edits and sizes, the tasks child's settings heads naming an application's
setting, and a *judged* refusal with its words among the people's answered keys. A format
change in place, as the store's rules have it before the first release.

Acceptance:

- `Record::Application`, `Revision.applications`, `Proposal.applications`,
  `Setting::Application`, `people::Refusal::Judged` and `people::Ask::ChangeSettings`'
  `projects` round-trip through the store's codecs; every `unreachable!("..: apps-store")`
  arm `apps-surface` and `apps-change` left in `jig-protocol-store` is gone.
- The keys of an application's setting sort after jig's kinds within tag 8's defaults and
  changes subfamilies, by kind, scope, number.
- The crate's tests and goldens pass, the goldens remade for the changed records only; the
  `jig/tests/store` world's settings stories pass (a change of meaning of a stored
  family: run them).
- The **Not yet** "4.4: the settings family keeps no state or size of an application's
  kind ..." in `jig/docs/design/protocol/store.status.md` is removed.

## Inputs

- `jig/docs/design/protocol/store.md`, 4.4 (the settings family's row and its key), 5.1,
  5.3 (**The levels**: no version moves before the first release), 5.4.
- `jig/docs/design/domain/engine.md`, 17.5 (**The core keeps the rest**), 17.7 (**What the
  store keeps**).
- `temper/docs/plan/reference-m3/notes/common.md` (store schemas and goldens).

## Owns and may change

- Owned: the `files` above.
- Neighbour: `jig/tests/store/**`, a fixture's new field or arm only;
  `jig/testing/jig-client-scripted-engine/src/codec/values/settings.rs` and
  `jig/testing/jig-client-test-records/src/settings*.rs` where they match on a stored
  setting (the arm only).
- **Versions:** the schemas this task changes are `settings-v1`, `people-v14` and
  `tasks-v9` as this trunk has them; each is changed in place, its generated code and
  its goldens made again with the crate's own generator and golden tooling (store.md, 5.3;
  no version moves before the first release). If the crate's tests or a layout check
  require a version to move for a changed record, move only that family's version by one
  and its format row's expectation, and say so in the report.
- Options, pre-approved:
  - If the settings schema keys a setting by a single enum (jig's four kinds), add an
    `application` arm carrying `kind`, `scope`, `number` (default); if it keys by a kind
    number, give an application's kinds a kind tag above jig's and keep the number layout.
  - A judged refusal's words are bounded by `people::Limits.words` in the schema's size
    bound; if the people schema has no bounded bytes type at hand, reuse the one a party's
    words use.

## Steps

1. `settings-v1.schema`: `Setting` gains `application { kind: u32, scope: u32, number:
   u32 }`; a new record `ApplicationState { follows: Follows, size: u32, charters: [u32] }`
   (`Follows`: `default`, `changed { revision }`, `removed { revision }`), stored at
   `Key::Changed` of an application's setting; `Revision` and `Proposal` gain
   `applications: [Recorded]`, `Recorded { kind, scope, number, edit: SizedEdit }`, `SizedEdit`:
   `set { size }`, `remove`, `reset`. Regenerate; codec both ways in `codec/settings.rs`;
   key in `keys/settings.rs`; `reads/settings.rs` serves a revision's applications with its
   items.
2. `people-v14.schema`: `Refusal` gains `judged { at: u32, setting: Setting, why: bytes }`
   and the `ChangeSettings` ask its `projects: [u32]`; `Setting` its application arm, as
   in step 1. Codec in `codec/people/`.
3. `tasks-v9.schema`: the settings head's `Setting` gains the application arm; codec in
   `codec/tasks/proposals.rs`.
4. Tests in `src/tests/` (the crate's codec and settings tests): round trips of each new
   record at its smallest and largest; key order (an application's setting after
   `Setting::Rules`, ordered by kind, scope, number); a revision read back with its
   applications. Remake the goldens the changed records name, no others.
5. Narrow tests: `heavy cargo nextest run -p jig-protocol-store --lib`;
   `heavy cargo nextest run -p jig-store-world -E 'test(/settings/)'` (the package
   `jig/tests/store` names; check the count). `cargo fmt`; `heavy cargo clippy --workspace
   --all-targets`. Edit the status file. Commit and push.

## Report

Which schemas changed and whether any version moved; the goldens remade, by name.

## Answers

2026-11-03. **Q:** may this task add `Setting::Application` to `jig_domain_tasks::Setting`
(`proposals.rs`) and regenerate `src/generated/layouts.rs`?

**A:** `layouts.rs` yes (now in `files`): it is the generator's output for the schemas
changed in place, no version moves. `jig_domain_tasks::Setting` no: the tasks child keeps
no application head. `apps-proposals`' answer (2026-11-03, from `engine.md`, 17.6, **The
owner keeps the value**) settled that the tasks child's heads are jig's kinds only and the
core keeps the set, its application items included; a set of application items alone has
no heads.

- Step 3 is dropped: `tasks-v9.schema` and `codec/tasks/proposals.rs` stay unchanged.
- In the acceptance, `Setting::Application` means the settings and people families' setting
  (steps 1 and 2), not a tasks head; the Goal's "the tasks child's settings heads naming an
  application's setting" no longer applies.
- `jig/tests/core`'s `apps-store` arms (`start_settings.rs`, `fuzzy/settings.rs`) are outside
  `jig-protocol-store` and outside this acceptance: leave them, and list them under `## Left`.

2026-11-03. **Q:** `jig-store-world`'s `test(/settings/)` selects nothing: may this task add
settings stories under `jig/tests/store`?

**A:** No new stories. The store world has no settings-named stories: it tests the
component (pages, generations, crashes, patching), and the settings family is among the
records it writes (`model.rs`'s `CopySettingsRevision`, `root.rs`'s settings limits) and
its layout and transcript goldens hold. The settings records' round trips, key order and
revision read-back are the crate's own tests (step 4), which passed.

- The acceptance's "the `jig/tests/store` world's settings stories pass" reads "the
  `jig/tests/store` world passes": run it whole, `heavy cargo nextest run -p
  jig-store-world` (check the count, about 176), in place of step 5's filter.
- If a golden of that world (`jig/tests/store/golden/**`) fails only because a changed
  record's bytes moved (an empty `applications`), remake it with the world's own golden
  tooling, name each remade file in the report; a fixture's new field stays allowed as
  before. Any other failure there: stop and ask.
- Then the checks still pending (the direct dependents' focused suites, workspace clippy);
  compile errors outside this task's files and neighbours go under `## Left`.
