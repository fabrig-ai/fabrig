+++
id = "fr-svc-route"
part = "forge-records"
kind = "feature"
needs = ["fr-svc-forges", "fr-domain", "fr-root", "fr-group"]
files = ["temper/crates/temper-service/src/route.rs", "temper/crates/temper-service/src/route/**", "temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/family/**", "temper/crates/temper-service/src/lib.rs", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/engine.rs", "temper/crates/temper-service/src/tests/forge_records.rs"]
effort = "high"
minutes = 110
+++

## Goal

The service's half of the meeting point (draft
`temper/docs/design/draft/playground-reset.md`, section 3): a client's
`SetUp` or `Connect` naming a new forge reaches the connector filled from
the machine's members; the root's `ForgeKept` configures the forge
through the shell's runtime answer and `Engine::keep_forge`; a box kind's
`SetUp`/`SetUpGroup` writes its group file through the shell before
`start N` (`notes/forge-records.md`, section 5). After it, a fresh
deployment under the box can make an owned Forgejo record and have the
box asked to start it.

Acceptance:

- `family::to_root`: wire `ForgeChoice::New(choice)` →
  `machine::fill(&engine.machine, &choice.forge)` (`choice` the wire's
  `NewForgeChoice`, `notes/forge-records.md`, section 6) → `Ask::SetUp`/`Connect` with
  `ForgeChoice::New(NewForge::Filled(template))`; a refused fill answers
  `set_up_refused { invalid }` / `connect_refused { not_taken }`
  (replacing `fr-wire`'s `make_refused { not_offered }` placeholder).
  The domain's `SetUpRefusal::Make(r)` / `ForgeRefusal::Make(r)` answer
  wire `make_refused { why }` (replacing `fr-domain`'s forced arms).
- The root's `Config.playground` is `machine::playground(&settings.machine)`
  when under the box, else `None` (`system.rs`, `prepared`).
- `route::request`, `root::Request::ForgeKept { record }`:
  `machine::parse_record` (a refusal is a service note naming the forge
  and its problem, and the forge stays unconfigured); a first `ForgeKept`
  of a number queues a `ForgeAsk { forge, api, transport }`; a later one
  updates its intent through `keep_forge`. New public API in `lib.rs` and
  `system.rs`, beside the handover reads:
  `Service::take_forge_runtime() -> Option<ForgeAsk>`,
  `Service::forge_runtime(forge: u16, answer: Result<ForgeRuntime,
  Box<[u8]>>)` (an `Err` is a note naming the forge; the forge stays
  unconfigured until the next start), which calls
  `Engine::keep_forge(parsed, runtime)`.
- The connector's `Restored` (as the root reports it to the service
  today; find where the service learns the restart's stages) calls
  `Engine::restored_forges()` after every `ForgeKept` of that stage.
- **Group files:** `root::Request::SetUp` and `SetUpGroup` for a forge
  whose mode is owned or fake and which is a record forge first queue a
  `GroupWrite { forge, bytes }` (`temper_forge_setup::group::encode` of
  its parsed setup entry, kind `owned`, or `fake` with `llm` true for the
  playground's template forge) and send `start N` only on its
  `Service::group_written(forge, Ok(()))`; `Err(why)` answers the setup
  as a box `failed N why` is answered. A document forge's `start N` is
  unchanged. `Service::take_group_write() -> Option<GroupWrite>`.
- Tests (a new `src/tests/forge_records.rs`, under a box pipe as
  `src/tests/setup_create.rs` drives one):
  `a_set_up_of_a_new_owned_forge_makes_its_record_writes_its_group_then_starts_it`,
  `a_kept_forge_waits_for_its_runtime_then_serves`,
  `a_restored_record_is_configured_at_restart_and_its_box_group_is_not_rewritten_until_asked`,
  `a_refused_fill_answers_invalid_and_makes_no_record`,
  `a_full_deployment_answers_make_refused_full`,
  `a_group_write_failure_fails_the_set_up_naming_it`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (sections 3
  to 5), `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (sections 2 to 6).
- `temper/docs/design/protocol/setup.md`, 7.1 (the pipe) and 6.6 to 6.7
  (local set up, first repository).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s2-box.md`,
  section 3 (serve's path from acts to `start N`).
- `tasks/fr-svc-forges.md` (the entry you call), `tasks/fr-group.md`
  (the codec), `tasks/fr-wire.md` (the placeholders you replace).

## Owns and may change

- Owns: `temper-service`'s `src/route.rs`, `src/route/` (`box_link.rs`
  waits for the group write), `src/family.rs` and `src/family/`,
  `src/lib.rs` (the two new public types), `src/system.rs` (the four
  public calls; `prepared`'s playground template), `src/engine.rs` (the
  queues), the new test file.
- May change: `temper/crates/temper/src/host.rs`, only to drain the two
  new queues with an answer of `Err(b"unserved")` each, so the binary
  builds and a process run fails visibly until `fr-shell` serves them.
- Options: if the service learns the connector's `Restored` from no
  request today, make `restored_forges` follow the first decision after
  the restart's `Restored` stage and say so.

## Steps

1. The fill in `to_root`; the refusals' wire answers.
2. `ForgeKept`, the runtime ask and answer, `restored_forges`.
3. Group writes before `start N`.
4. Tests. Narrow: `heavy cargo nextest run -p temper-service --lib -E
   'test(/forge_records/)'`.
5. `cargo fmt`; `heavy cargo clippy -p temper-service -p temper
   --all-targets -- -D warnings`; focused tests of `temper-service`;
   `heavy cargo check -p temper -p temper-one-process-world -p
   temper-real-loop-world --tests`. Old-forms from `temper/`. Commit and
   push.

## Report

As `notes/common.md` says; where the service learns the restart's stage.
