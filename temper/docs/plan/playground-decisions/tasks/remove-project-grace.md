+++
id = "remove-project-grace"
part = "deletion-grace"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-client-protocol-wire/schema/client-v19.schema",
  "jig/crates/jig-client-protocol-wire/src/generated/v19.rs",
  "jig/crates/jig-client-protocol-wire/golden/**",
  "jig/crates/jig-protocol-store/schema/people-v14.schema",
  "jig/crates/jig-protocol-store/src/generated/people_v14.rs",
  "jig/crates/jig-protocol-store/src/generated/layouts.rs",
  "jig/crates/jig-protocol-store/golden/people/**",
  "jig/crates/jig-protocol-store/src/codec/people.rs",
  "jig/crates/jig-protocol-store/src/codec/people/asks.rs",
  "jig/crates/jig-protocol-store/src/codec/people/acts.rs",
  "jig/crates/jig-protocol-store/src/tests/codec.rs",
  "jig/crates/jig-domain-people/src/**",
  "jig/crates/jig-client-domain/src/contract/asks.rs",
  "jig/crates/jig-client-protocol-link/src/translate/asks.rs",
  "jig/crates/jig-client-protocol-link/src/translate/store.rs",
  "jig/crates/jig-client-protocol-link/src/tests/**",
  "jig/crates/jig-protocol-engine-clients/src/translate/asks.rs",
  "jig/crates/jig-protocol-engine-clients/src/tests/asks.rs",
  "jig/tests/client-wire/tests/suite/remove_project.rs",
  "jig/tests/people/tests/suite/deletion.rs",
  "jig/docs/design/client/domain/contract.status.md",
  "jig/docs/design/protocol/store.status.md",
  "jig/docs/design/domain/people.status.md",
]
effort = "high"
minutes = 120
+++

# `RemoveProject` and the kept deletion carry a grace, unused

## Goal

The user decided (`inputs/request.md`, decision 3) that a deleted project's conversations are
cancelled with their close's grace. The engine reads no template, so the request carries the
grace (`notes/contracts.md`, C2). This task carries the field everywhere a `RemoveProject` or a
kept deletion crosses, with **no change of meaning**: the core still cancels with no grace, and
passes `grace: None` where it keeps a deletion. `deletion-grace` gives it its meaning.

Acceptance:

- jig's client wire `RemoveProject`, the client contract's `Ask::RemoveProject`, the people
  child's `Ask::RemoveProject` and `Act::RemoveProject`, its kept `Stored::Deletion` and its
  `Event::Deleting` each carry `grace: Option<Duration>`, translated across every layer and
  round-tripping in the store's and the wire's codecs; the schemas change in place
  (`client-v19.schema`, `people-v14.schema`), their generated code, goldens and layout
  fingerprints regenerated.
- The people child keeps a deletion's grace beside its step, saves it with each step and restores
  it at start (`deletions()` gives both).
- An owner's `RemoveProject` with a grace is admitted exactly as one without (see the trap in
  "Owns and may change": the core's party route compares the ask today).
- The status edits below; the tests below pass; nothing else moves.

## Inputs

- `temper/docs/plan/playground-decisions/notes/common.md` and `notes/contracts.md`, C2.
- `jig/docs/design/domain/people.md`, 5.1 (the `RemoveProject` row), 5.2 (the paragraph on the
  grace, and "Kept until whole"), 5.3 (the `RemoveProject` act row).
- `jig/docs/design/client/domain/contract.md`, 3.2 (the `RemoveProject` row).
- `jig/docs/design/protocol/store.md`, 4.4 (the people family's row: "each project being
  deleted, with the step its deletion has reached and the grace it gives its conversations").

## Owns and may change

Shapes (names binding; `Duration` is `skein_lib::Duration`, as `Cancel`'s grace is at each
layer):

- **Client wire,** `client-v19.schema:808`, in place:
  `record RemoveProject { project: u32 grace: option duration }` (as `Cancel`'s, `:305`).
- **Contract,** `jig-client-domain/src/contract/asks.rs:213`:
  `RemoveProject { project: u32, grace: Option<Duration> }`.
- **Link** (`jig-client-protocol-link`): `translate/asks.rs:679` and `translate/store.rs:182`
  carry it both ways, as `Cancel`'s grace is (`translate/asks.rs:610`).
- **Engine's clients** (`jig-protocol-engine-clients`): `translate/asks.rs:131`, as `Cancel`'s
  (`:108`).
- **People child** (`jig-domain-people`, `boundary.rs`):
  - `Ask::RemoveProject { project: u32, grace: Option<Duration> }` (`:230`);
  - `Act::RemoveProject { project: u32, grace: Option<Duration> }` (`:678`);
  - `Stored::Deletion { project: u32, step: DeletionStep, grace: Option<Duration> }` (`:758`);
  - `Event::Deleting { project: u32, step: Option<DeletionStep>, grace: Option<Duration> }`
    (`:844`); the grace is ignored when `step` is `None`;
  - a new public type, exported from `lib.rs` beside `DeletionStep`:

    ```rust
    /// A kept deletion's step and the grace its conversations are cancelled with
    /// (jig/docs/design/domain/people.md, 5.2).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct KeptDeletion {
        pub step: DeletionStep,
        pub grace: Option<Duration>,
    }
    ```

    `Domain.deletions: Map<u32, KeptDeletion>`, `pub fn deletions(&self) -> &Map<u32, KeptDeletion>`;
    `domain/deletion.rs` `keep` saves `Stored::Deletion { project, step, grace }`;
    `domain/start.rs:231` restores both. Patterns elsewhere in the crate gain `..`.
- **Store** (`jig-protocol-store`), `people-v14.schema`, in place:
  `record AskRemoveProject { project: u32 grace: option u64 }` (`:191`),
  `record ActRemoveProject { project: u32 grace: option u64 }` (`:314`),
  `record StoredDeletion { project: u32 step: DeletionStep grace: option u64 }` (`:319`); the
  codecs (`codec/people/asks.rs:299,518`, `codec/people/acts.rs:6,39`, `codec/people.rs:160,236`)
  convert with `compound.rs`'s `encode_grace`/`decode_grace`, as `AskCancel`'s grace is.
- **Generated and goldens:** `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p
  jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'`, then once without the
  variable. Keep only what these three records and `RemoveProject` move (`git diff --stat`).

Reader edits (each line only in the field's name or value, or a pattern's `..`):

- **The trap,** `jig-domain-core/src/routing/party/people.rs:261`:
  `if *ask == (people::Ask::RemoveProject { project })` lets an owner's removal past
  `acts::request` (which refuses it *not offered*). With `grace: None` added it would refuse every
  graced removal. Make it a pattern on the project alone, any grace:
  `matches!(&*ask, people::Ask::RemoveProject { project: named, .. } if *named == project)`.
  `:275` and `routing/party/deployment_acts.rs:57` bind `{ project, .. }` and call `begin` as
  today.
- `jig-domain-core/src/admission/deletion.rs`: `keep` sends
  `people::Event::Deleting { project, step, grace: None }`; `restored` reads `kept.step` from
  `core.people.deletions()`. Nothing else in the file.
- jig-domain-core's tests (`src/tests/admission/deletion.rs:92,114,142`,
  `src/tests/start/links.rs:583`), `jig-protocol-store/src/tests/codec.rs:1457,1465,1479`,
  `jig-client-protocol-link/src/tests/projects.rs:100` and `tests/fixtures/link_store.rs:46`,
  `jig-protocol-engine-clients/src/tests/asks.rs:310,318`, `jig-domain-people/src/tests.rs`
  (2436-2495), `jig-client-scripted-engine` (`requests/projects.rs:14`, `requests.rs:699`),
  `jig-conformance` (`src/core/naming.rs:156,204`), and the worlds
  (`jig/tests/people/tests/suite/deletion.rs`, `jig/tests/core/tests/suite/deletion.rs`
  183/303/351/574, `deployment_acts.rs:470`, `parties.rs:468`, `jig/tests/core/src/conformance/naming.rs:239`,
  `jig/tests/conformance/src/deletion.rs:345`, `jig/tests/client/tests/stories/projects.rs:160,176`).
- temper, two lines: `temper/crates/temper-engine-domain/src/route/playground.rs:291`
  (`people::Act::RemoveProject { project, grace: None }`) and
  `temper/crates/temper-client-tools/src/tests/jig/acts.rs:153` (`grace: None`). Nothing else in
  temper (`kept-names` and `box-words` own those crates).

Options, pre-approved:

- If a memory-pricing test's figure moves because `KeptDeletion` or a variant grew, recompute
  the figure the way the test computes it from `size_of`; never move a configured limit or a
  budget. Say so in the report.
- If a generated file outside the three records and `RemoveProject` changes on regeneration,
  keep it only if the generator demands it, and say why.

Status, as `notes/common.md` says (date line today):

- `jig/docs/design/client/domain/contract.status.md`, **Not yet** 3.2: remove `; \`RemoveProject\`'s
  grace` and nothing else.
- `jig/docs/design/protocol/store.status.md`, **Not yet**: remove `4.4: a kept deletion's grace:
  \`people-v14.schema\`'s \`StoredDeletion\` holds its project and step alone. `
- `jig/docs/design/domain/people.status.md`, **Not yet**, the second 5.2 entry becomes:
  `5.2: a deleted project's conversations asked to finish within the grace its \`RemoveProject\`
  gives (5.1, 5.3 and 7): the core neither checks nor keeps the request's grace, and the
  deletion's \`cancel\` cancels every live root with none
  (\`jig-domain-core/src/admission/deletion.rs\`).`

## Steps

1. Schemas, regeneration, codecs; the round trips:
   - `jig/tests/client-wire/tests/suite/remove_project.rs`, `a_remove_project_ask_round_trips`:
     each project of `[0, 3, u32::MAX]` with `grace` `None` and `Some(Duration::from_millis(5000))`.
   - `jig-protocol-store/src/tests/codec.rs`: the act, ask and deletion round trips (1457, 1465,
     1479) each also with a grace.
   `heavy cargo nextest run -p jig-client-wire-world --test suite -E 'test(remove_project) or test(drift::)'`;
   `heavy cargo nextest run -p jig-store-codecs-world --test suite -E 'test(drift::)'`;
   `heavy cargo nextest run -p jig-protocol-store --lib -E 'test(codec) or test(words)'` (`words`
   reads `jig/docs/design/protocol/store.md`, whose 4.4 row this plan's design changed).
2. People child: shapes, keep and restore. In `jig/tests/people/tests/suite/deletion.rs`, the
   keep and restore stories (around 101-153) carry a grace: a row restored with
   `grace: Some(..)` reads back from `deletions()` as `KeptDeletion { step, grace }`, and
   `Event::Deleting { .., grace }` saves it. `heavy cargo nextest run -p jig-domain-people` and
   `-p jig-people-world --test suite -E 'test(deletion)'`.
3. Contract, link and engine's clients, with a graced round trip in
   `jig-client-protocol-link/src/tests/projects.rs` and `jig-protocol-engine-clients/src/tests/asks.rs`.
4. The core's reader edits and one new unit test in
   `jig-domain-core/src/tests/admission/deletion.rs`, beside
   `an_owners_ask_removes_it_and_a_members_is_refused`:

   ```rust
   #[test]
   fn an_owners_graced_removal_is_admitted_as_an_ungraced_one() {
       // as an_owners_ask_removes_it_and_a_members_is_refused, the owner's ask carrying
       // grace: Some(Duration::from_millis(5000)): answered ProjectRemoved, project closed
   }
   ```

   `heavy cargo nextest run -p jig-domain-core --lib -E 'test(deletion)'`.
5. The other readers compile: `heavy cargo check -p <pkg> --tests` for jig-client-scripted-engine,
   jig-conformance, jig-core-world, jig-conformance-world, jig-client-world, temper-engine-domain,
   temper-client-tools.
6. The status edits; the citations check.
7. `cargo fmt`; old forms for jig and temper; `heavy cargo clippy` naming every package you
   changed, `--all-targets -- -D warnings`; focused tests: `heavy cargo nextest run -p
   jig-client-protocol-wire -p jig-client-wire-world -p jig-protocol-store -p jig-store-codecs-world
   -p jig-domain-people -p jig-people-world -p jig-client-domain -p jig-client-protocol-link -p
   jig-protocol-engine-clients -p jig-domain-core -p jig-core-world -p jig-client-world`, then
   `-p temper-engine-domain --lib` and `-p temper-client-tools --lib -E 'test(acts)'`. One commit
   for the code and tests, one for the status text; push.

## Report

As the implementer's text says; the generated files and goldens that moved (by `--stat`), and
whether any memory figure moved.
