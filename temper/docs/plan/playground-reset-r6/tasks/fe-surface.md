+++
id = "fe-surface"
part = "fs"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge/**",
  "temper/crates/temper-engine-domain/**",
  "temper/crates/temper-protocol-store/**",
  "temper/tests/codecs/**",
  "temper/tests/engine/**",
]
effort = "high"
minutes = 110
+++

## Goal

Add the engine's surface for the run: the forge record and its kin, the machine's templates,
the four new store families at store version 11, the connector's new requests and answers,
the root's new requests to serve and serve's answers, and fe's own connector-to-root
vocabulary, exactly as `temper/docs/plan/playground-reset-r6/notes/fe-surface.md` spells
them. Each new request is decided as a refusal or a no-op (that note, section 7); nothing
else changes meaning. Other designers' tasks build on this surface at once, so build it
exactly and keep it small: no behaviour beyond section 7.

Acceptance:
- Every type, variant, field and constructor of `notes/fe-surface.md` sections 1 to 5
  exists with the names, fields, order and derives given there, each with a one-line doc
  comment in the design's words (cite `temper/docs/design/draft/playground-reset.md` and its
  section at type level).
- `temper-protocol-store` is at `STORE_VERSION` 11, its schema renamed `store-v11.schema`
  with section 6's records, its generated code `src/generated/v11.rs` and goldens
  regenerated, and every new row round-trips through `records::encode`/`decode`/`row` and
  `keys::encode`/`decode` in key order.
- `MakeForge`, `RemoveForge` and `EndPlayground` are admitted at the deployment's scope and
  answered as section 7 says; `StartPlayground` behaves exactly as `StartDemo` did.
- The workspace builds (`heavy cargo check --workspace --all-targets`), every test of the
  packages listed in Steps, 6 passes, and clippy is clean on the changed packages.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T1, T2, T4, T5, T6.
- `temper/docs/plan/playground-reset-r6/notes/fe-surface.md`, all of it: the shapes.
- `temper/docs/design/draft/playground-reset.md`, sections 3, 4 and 6 (for doc comments).
- `temper/docs/design/protocol/store.md`, sections 2 to 4 (families, keys, versions).
- `temper/AGENTS.md`; `skein/docs/foundation/programming-model.md`, section 1.

## Owns and may change

- Owns: `temper/crates/temper-engine-domain-forge/`, `temper/crates/temper-engine-domain/`,
  `temper/crates/temper-protocol-store/` (schema, generated code, goldens, keys, records,
  tests), `temper/tests/codecs/` (package `temper-codecs-world`), and the engine worlds
  `temper/tests/engine/domain/` (`temper-engine-domain-world`), `temper/tests/engine/forge/`
  (`temper-engine-forge-world`), `temper/tests/engine/protocol-forge/`
  (`temper-engine-protocol-forge-world`).
- May change, each only for this surface's compiler-forced edits (the implementer's text;
  list every file in the report):
  - `temper/crates/temper-service/`: `src/family.rs` (`forge::Ask::StartDemo` →
    `StartPlayground` at the start translation; `root::ApplicationAnswer::StartDemo` →
    `StartPlayground`; one new arm `Some(root::ApplicationAnswer::MakeForge(_) |
    root::ApplicationAnswer::RemoveForge(_) | root::ApplicationAnswer::EndPlayground(_)) =>
    w::Outcome::Unknown` placed just before the answer match's `None` arm: sv's task gives
    their wire answers), `src/clients.rs` and `src/clients/demo_tests.rs` (the rename),
    `src/route.rs` (`SetUpGroup { forge, request, .. }` ignores `entry`; `RemoveGroup`,
    `EraseSecrets`, `Register`, `Unregister` each a no-op arm with a comment naming sv's
    task to come), `src/limits.rs` (`forges: 4` in `FORGE`, with a comment "forge.forges,
    the small profile (contracts T4)"), the four root `Config` literals (`src/system.rs`,
    `src/startup/checks.rs`, `src/tests/fixtures.rs`, `src/tests/welcome_policy.rs`) gaining
    `machine: temper_engine_domain_forge::Machine::bare()`, `src/settings.rs`
    (`Demo::domain`'s `DemoConfig` literal gains `project_name: "Playground".into(),
    project_policy: 1`), and its tests' exhaustive matches on `root::Request` or
    `forge::Answered` (`src/tests/engine.rs`, `src/tests/setup_lifecycle.rs`,
    `src/tests/fixtures.rs`, `src/tests/setup_github.rs`, `src/tests/family.rs`).
    cl's `cl-wire` edits `src/family.rs` in parallel: touch only the lines named here.
  - `temper/crates/temper-fake/src/check.rs` (`v10` → `v11`; the four new `Stored`
    variants join the no-op arm) and `src/tests/check.rs` (`forges: 4` in its limits).
  - `temper/crates/temper/` (the binary) wherever the compiler names `RecordKind`,
    `Stored` or `v10` (`src/host.rs` is expected to need nothing).
  - `temper/tests/client/` (`src/front.rs`'s `SetUpGroup { forge, request }` pattern gains
    `..`; `src/translate.rs`, `src/translate/forge.rs`) and `temper/tests/system/`
    (`src/forge.rs`, `src/composed.rs`): the new `root::Request` variants join the arm that
    already takes `StopGroup`/`SetUpGroup`.
  - `temper/tests/conformance/`, `temper/tests/one-process/`, `temper/tests/real-loop/`,
    `temper/tests/client-browser/` only where the compiler names them.
- Pre-approved options:
  - Answers kept for the three refused asks: default, make `connect::State::answer`
    `pub(crate)` and call it from the step's `Event::Application` arms. If a separate small
    map is cleaner, keep it bounded by `l.applications` as `connect::State::answers` is,
    oldest dropped when full, and say so.
  - `Stored`'s size: `Stored::Forge(ForgeRecord)` makes the enum larger. Default: keep the
    variant unboxed (the existing `#[expect(clippy::large_enum_variant, ...)]` covers it).
    If clippy names another lint, box the record (`Forge(Box<ForgeRecord>)`) only if the
    lint cannot be met otherwise, and say so: the note's shape then changes and the report
    must say it first.
  - `stored_bytes` for the new rows: the sum of their byte strings' lengths (record:
    host, server_name, trust, git, git_program, web, the bot's three, setup, first's org,
    repository, label and new project name); `ForgeNumbers`, `Playground`, `KeptAct`: 0.
  - Restore of the new rows (`restore.rs` room check, `reconcile.rs` references,
    `domain.rs::restore`): accept and drop (no table yet). If the restore-fit tests
    enumerate every `RecordKind` and need a capacity, give the four kinds `Ok(())` and say
    so.
  - Golden files: generated by `SKEIN_CODEGEN_WRITE=1` as
    `temper/tests/codecs/tests/suite/drift.rs` says; commit what it writes, nothing else.

## Steps

1. Forge connector (`temper/crates/temper-engine-domain-forge/`):
   - `src/boundary.rs`: section 1's types after `Configured`; `ForgeChoice`, the answers,
     `ResetAct`, `Playground`, `PlaygroundState`, `ResetStep`, `KeptAct`, `KeptActKind`
     near `Demo`/`DemoAct`; the `Ask`, `Answered`, `Request`, `Event`, `Stored`, `Key`
     variants of sections 2 to 5, in the stated order; `DemoConfig`'s two fields.
   - `ForgeRecord::github_com` and `Machine::bare` in `src/boundary.rs` (or a new
     `src/record.rs`, re-exported through `boundary::*`), `#[must_use]`.
   - `src/limits.rs`: `Limits::forges` after `repositories`, doc comment "The forges kept at
     most, `Kept` and `Removing` records (draft playground-reset.md, 3)". `valid` unchanged.
   - `src/lib.rs`: `stored_key` and `stored_bytes` arms; `src/reconcile.rs`: `RecordKind`
     variants and no-op reference arms; `src/restore.rs`: no-op room arms.
   - `src/domain.rs`: `Domain::playground` (a `playground: Option<crate::Playground>`
     field, `None` at `new`); the step's arms: `Ask::StartPlayground` →
     `crate::demo::start` (the old `StartDemo` arm); `MakeForge`/`RemoveForge`/
     `EndPlayground` → section 7's answers; the new events as no-ops; `application_answer`
     returns `Answered::StartPlayground` for the demo's answers.
   - `src/connect.rs`: the `unreachable!("connect admission")` arms name the new asks.
   - `src/demo.rs`: `Request::SetUpGroup { forge, request, entry: Box::new([]) }` at both
     sites.
   - Unit tests (`src/tests/` beside the demo's): `make_remove_and_end_are_refused_until_their_tasks`
     (each ask through `Event::Application` answers its refusal through
     `application_answer`, and a carried value is answered `CarriedUnused`);
     `the_new_rows_restore_as_no_ops` (restoring one row of each new family leaves
     `forges()` and every map unchanged and `Restored` still reports `RestartDone`);
     `serve_and_root_answers_are_inert_until_their_tasks` (each new event leaves no output).
2. Root (`temper/crates/temper-engine-domain/`):
   - `src/boundary.rs`: `ApplicationAnswer` (section 2), `Request` (section 3; the root's
     `SetUpGroup` gains `entry`); `src/translate.rs`: `Config::machine`.
   - `src/route/applications.rs`: the four asks at `ApplicationScope::Deployment`;
     `request == Application::Forge(forge::Ask::StartPlayground)` keeps the demo context;
     `answer` maps the new `Answered` variants.
   - `src/route/forge_route.rs`: route `forge::Request::{RemoveGroup, EraseSecrets, Register,
     Unregister}` through `super::now` as `StopGroup` is, `SetUpGroup` passes `entry`;
     `SendKeptAct`, `PlaygroundStart`, `ResetAct` as no-op arms (comment: "fe-kept-root" /
     "fe-playground-root").
   - `src/domain.rs::stage_now`: the four new requests beside `StopGroup`.
   - `src/route/demo.rs`: the `adopted` match names the renamed and new `Answered`
     variants in its failure arm.
   - Tests: rename what names `StartDemo`; add `the_new_asks_are_the_deployments`
     (in `src/tests/`, beside the applications' admission tests: each of `MakeForge`,
     `RemoveForge`, `EndPlayground` reaches the core's `Application` with
     `ApplicationScope::Deployment`).
3. Store (`temper/crates/temper-protocol-store/`):
   - `git mv schema/store-v10.schema schema/store-v11.schema`, `git mv src/generated/v10.rs
     src/generated/v11.rs`; `pub mod v11`; `STORE_VERSION = 11`; section 6's schema text.
   - `src/keys.rs`: the four constants and key arms; `largest` unchanged unless a new key
     exceeds it (it does not: 9 and 17 bytes).
   - `src/records.rs` (or a new `src/records/forges.rs`): total translations for the four
     rows, both ways; `Forge`'s `first` through `FirstProject`/`FirstState`.
   - Regenerate: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world
     --test suite -E 'test(drift::)'`, then the same without the variable must pass.
   - Tests: `src/tests/keys.rs`: `forge_families_take_bytes_thirty_one_to_thirty_four_in_order`
     (each new key's first byte, order against `Flight` and `Receipt`, round trip, a
     `Forge` key past `u16` and a 15-byte `KeptAct` key refused); `src/tests/records.rs`:
     `forge_records_round_trip_at_smallest_and_ceiling` (a record with every option `None`
     and one with every option at its ceiling, `Plaintext` and `Tls`, each `First` state
     and project), `forge_numbers_playground_and_kept_acts_round_trip` (each
     `PlaygroundState`, each `KeptActKind`), and a mismatched key refused for each;
     `src/tests/sizes.rs`: the version test renamed `the_store_version_is_eleven`, the
     application sizes' `version: 11` (the key and value bounds unchanged: the record's
     ceiling is about 20 KB, below the largest row); `src/tests/fixtures.rs`: `forges: 4`
     and the key generator draws the new families.
4. Codecs world (`temper/tests/codecs/`): `drift.rs` names `"v11"`; `store_protection.rs`,
   `store_queue.rs` name `v11`; `tests/support/store.rs` `forges: 4`;
   `tests/suite/fuzzy/store.rs`: the golden prefix `[0_u8, 11]`, and the draw adds one
   `Forge`, `ForgeNumbers`, `Playground` and `KeptAct` row per iteration through
   `row_and_key`.
5. Engine worlds (`temper/tests/engine/*`): compiler-forced arms and `forges: 4`; rename
   `StartDemo`; `walking.rs`'s config `machine: forge::Machine::bare()`.
6. Run, one at a time, reading any failure's log before anything reruns:
   - `heavy cargo nextest run -p temper-protocol-store --lib`
   - `heavy cargo nextest run -p temper-codecs-world --test suite`
   - `heavy cargo nextest run -p temper-engine-domain-forge --lib`
   - `heavy cargo nextest run -p temper-engine-domain --lib`
   - `heavy cargo nextest run -p temper-engine-domain-world --test suite`
   - `heavy cargo nextest run -p temper-engine-forge-world`
   - `heavy cargo nextest run -p temper-engine-protocol-forge-world`
   - `heavy cargo nextest run -p temper-service --lib` (the rename reaches its family tests)
   - `heavy cargo nextest run -p temper-fake --lib`
   - `heavy cargo check --workspace --all-targets`
   - `cargo fmt --all`; `heavy cargo clippy -p temper-engine-domain-forge -p
     temper-engine-domain -p temper-protocol-store -p temper-codecs-world -p
     temper-service -p temper-fake --all-targets -- -D warnings`
   - `python3 -I -B scripts/old_forms.py` from `temper/` (no count may rise).
7. Commit (small commits that say why), push your branch.

## Report

- First, any shape that differs from `notes/fe-surface.md` (there should be none), and why.
- Every file outside the owned crates you edited, one line each with what changed.
- Under `## Left`, readers in files you do not own that still name `StartDemo` or
  `start_demo` (the client wire's are `cl-wire`'s).

## Answers

- 2026-11-01. Q: may this task change `temper/crates/temper-service/src/tests/stop.rs`'s
  two `assert_eq!(wanted, 10)` (lines 154 and 163) to 11? A: yes. They pin
  `STORE_VERSION`, which this task raises, so they are this surface's forced edits, as the
  service files listed under "May change" are: change both literals to 11 and nothing else
  in that file (no new `found: 10` case; a v10 store's refusal is not this task's). The same
  holds for any other test, in a package Steps, 6 runs or the workspace check reaches, whose
  only failure is a literal store version 10 (or a `[0, 10, ..]` prefix): update it to 11
  and list each file in the report. Anything else failing outside the owned crates is a
  question again. Then finish Steps, 6 from the service run on.
