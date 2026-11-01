+++
id = "jp-row"
part = "jig-remove-project"
kind = "interface"
needs = ["jp-acts", "jp-erase", "pg-jig-uses"]
files = ["jig/crates/jig-domain-core/src/removal.rs", "jig/crates/jig-domain-core/src/removal/**", "jig/crates/jig-domain-core/src/stored.rs", "jig/crates/jig-domain-core/src/reads.rs", "jig/crates/jig-domain-core/src/tests/removal_rows.rs", "jig/crates/jig-protocol-store/schema/core-v6.schema", "jig/crates/jig-protocol-store/src/generated/core_v6.rs", "jig/crates/jig-protocol-store/src/generated/layouts.rs", "jig/crates/jig-protocol-store/src/codec/core.rs", "jig/crates/jig-protocol-store/src/codec/core/**", "jig/crates/jig-protocol-store/src/keys/core.rs", "jig/crates/jig-protocol-store/src/tests/codec/core_rows.rs"]
effort = "high"
minutes = 70
+++

## Goal

Lay the surface of a project's deletion in jig's core (draft
`temper/docs/design/draft/playground-reset.md`, section 5, "kept until
whole"): its durable row and the totals that outlive a deleted project,
their codec, their restore at start, and the `removal` module's skeleton
whose entries the part's later tasks fill (`jp-close`, `jp-steps`,
`jp-rows`, which run at once on disjoint files of this skeleton). No
behaviour changes for any live project.

Acceptance:

- The types of the part note's section 1, exactly:
  `jig_domain_core::removal::{Removal, Step, Spender}` (re-exported at the
  crate root as `Removal`, `RemovalStep`, `Spender`);
  `CoreRecord::Removal(Removal)` with `CoreKey::Removal(u32)`, and
  `CoreRecord::Spent { spender: Spender, spent: u64 }` with
  `CoreKey::Spent(Spender)`; both keys live (`CoreKey::live` true);
  `Core::removals: Map<u32, Removal>` (capacity `limits.authority.projects`),
  `Core::spent: Map<Spender, u64>` (capacity the people child's party
  limit plus one), and `pub fn removing(&self, project: u32) -> bool`.
- `restore_core` retains both (a `Removal` whose project is 0 or past the
  deployment header's project counter, or a full map, is `Rejected` or a
  start refusal as other live rows are: follow `Call`'s two modes); the
  start's live load reads them, before the people family's rows.
- The codec: the core key's tag 10 then the project as `u64`; tag 11 then
  0 (deployment) or 1 and the party's `u64`; `core-v6.schema` changed in
  place with `Removal`, `RemovalStep`, `Spender` and `Spent` and the two
  `Value` variants; generated code and `layouts.rs` regenerated (the
  core line only changes; `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run
  -p jig-store-codecs-world --test drift`, per `layouts.rs`' header);
  golden bytes where the core family has them.
- The skeleton, as the part note's section 4 has it:
  `src/removal.rs` (module doc citing the draft's section 5 and the part
  note, `mod close; mod steps; mod rows;`, the types, `resume`, `load`,
  `loaded`), `src/removal/close.rs`, `src/removal/steps.rs`,
  `src/removal/rows.rs`, each entry with the stub body the table gives
  (`#[expect(dead_code, reason = "...")]` where unused until its task).
  - `removal::load(core, env, project, scope: reads::Scope) -> Option<Request>`:
    a pending read with `Destination::Removal { project }` (new, in
    `reads.rs`), party 0 and sign-in 0 as `reads/settings.rs`' internal
    load is (line 54), rows and bytes the configured read bounds,
    returned as `Request::Held(Held::ReadLoad { .. })`; `None` when the
    load slots are full (the caller tries again at its next decision).
  - `reads::loaded` hands a `Destination::Removal` result to
    `removal::loaded(core, env, project, result) -> Requests`, which
    dispatches by the kept row's step (`Erasing` to `steps::loaded`;
    `Spend`, `Notes` to `rows::loaded`; else drops it).
  - `removal::resume`: called once, at the first decision after the start
    has opened (find the hook: `start/script.rs`' `start_resume` or
    `restart.rs`' `restart_done`; pick the one where a decision may emit
    writes and loads, and say which), dispatching each kept row by step
    (`Ending`, `Erasing` to `steps::begin`; `Spend`, `Notes`, `Own` to
    `rows::begin`).
- Tests: `jig-protocol-store` library tests, in
  `src/tests/codec/core_rows.rs`: a `Removal` of every step and a `Spent`
  of each spender read back, registered as the core's other rows are
  (`src/tests/codec/cases.rs`, its core case); `src/tests/keys.rs`' every-
  key round trip covering the two keys; `jig-domain-core` unit tests in a
  new `src/tests/removal_rows.rs`: both rows restored and found
  (`removing`), a rejected one, and a start with a kept `Removal` at
  `Own` erasing it through the stub `rows::begin` in its first decision.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 1).
- `temper/docs/plan/playground-reset-r5/notes/jig-remove-project.md`,
  sections 1, 2 (the internal loads, the resumption) and 4.
- `jig/docs/design/protocol/store.md`, 4.4 (jig's families) and 5 (in
  place before the first release); `jig/docs/design/domain/engine.md`, 5.4.
- The precedent for a live core row: `CoreRecord::RunProof` /
  `CoreKey::RunProof` end to end (`stored.rs`, `keys/core.rs`,
  `codec/core/runs.rs`, its restore).

## Owns and may change

- Owns the files in `files`: in `jig-domain-core`, the new `removal`
  module and its tests, `stored.rs`, `reads.rs` (the `Destination`
  variant and its dispatch only); `lib.rs` for the `mod removal;` line,
  the re-exports and the two `Core` fields with their constructor lines.
  In `jig-protocol-store`, the core family's schema, generated code,
  codec, keys and tests, and `layouts.rs`' core line.
- May change: compiler-forced arms over `CoreRecord` and `CoreKey` (S4
  and `rg 'CoreKey::EscalationDecision'` list them: `jig-domain-core`'s
  `reads/*.rs` and `routing/tasks.rs`; `jig-protocol-store`'s `commits.rs`,
  `historical.rs`, `open.rs`, `patching/`, `reads/point.rs`,
  `transcripts/cut.rs`, test fixtures; `jig/tests/client`, `jig/tests/ops`;
  temper's `temper-engine-domain` `archive.rs`, `decision.rs`, `restart.rs`,
  `store.rs`, `route/*`, `temper-service`'s `store/ranges.rs` and tests,
  `temper/tests/engine/domain`). Each new arm treats the rows as live core
  rows the way `RunProof`'s arm does (a store's byte accounting charges
  them as `RunProof` does). The start's live ranges, if they list the
  core's tags (`jig-protocol-store`'s `intervals.rs`, `restore/`,
  temper-service's `store/ranges.rs`): add tags 10 and 11 there.
- Not yours: `routing/deployment_acts.rs`, `routing/people.rs`,
  `routing/vocabulary.rs` (other tasks of this run), the people family's
  schema and generated code.
- Options: if a `Map` keyed by `Spender` needs `Ord` that the type has,
  use it; if the core's limits hold no party bound, use the people child's
  `Limits` field that bounds parties (`people` limits) and name it. If the
  first decision after opening cannot be found without a new core event,
  resume from the root's first input of any kind after `Phase::Open` and
  say so in the report.

## Steps

1. Types and the skeleton; `stored.rs` arms and restore; `reads.rs`'
   destination; `heavy cargo check -p jig-domain-core --tests`.
2. Keys, schema in place, regeneration, codec, codec and key tests:
   `heavy cargo nextest run -p jig-protocol-store --lib -E
   'test(/core_rows|keys/)'`.
3. Forced arms across trees; `heavy cargo check --tests` naming each
   package touched (`-p temper-engine-domain -p temper-service
   -p temper-engine-domain-world -p jig-client-world -p jig-ops-world`
   as needed).
4. Unit tests: `heavy cargo nextest run -p jig-domain-core --lib -E
   'test(/removal_rows/)'`.
5. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p
   jig-protocol-store --all-targets -- -D warnings`; the focused tests of
   `jig-domain-core`, `jig-protocol-store`, `jig-store-world`,
   `jig-store-codecs-world`, `jig-core-world`. Commit and push.

## Report

As `notes/common.md` says; name the stub entries' final signatures (the
later tasks build on them), the resumption hook you chose, and every file
outside `files` you edited for forced arms.
