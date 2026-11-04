+++
id = "core-prepare-agent"
part = "jig-core-steps"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/routing/run/preparation.rs",
  "jig/crates/jig-domain-core/src/routing/mod.rs",
  "jig/crates/jig-domain-core/src/routing/stopping.rs",
  "jig/crates/jig-domain-core/src/vocabulary.rs",
  "jig/crates/jig-domain-core/src/tests/**",
  "jig/examples/ops/domain/src/route.rs",
  "jig/examples/ops/domain/src/domain.rs",
  "jig/testing/jig-test-domain/src/routes.rs",
  "jig/testing/jig-test-domain/src/lib.rs",
  "temper/crates/temper-engine-domain/src/route.rs",
  "temper/crates/temper-engine-domain/src/route/now.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/tests/engine/domain/tests/suite/upgrade_transcripts.rs",
  "jig/docs/design/domain/root.status.md",
  "jig/docs/design/STATUS.md",
]
effort = "high"
minutes = 110
+++

# jig D001: the core prepares an agent without asking its root

## Goal

`Now::PrepareAgent` and `Event::PreparedAgent` go: `route_activate` continues into `route_prepared_agent` itself,
in the same decision. `Now::Activate` stays. A root's full read table reaches the core as `busy` on
`Event::Activate`, and an admitted agent with a window to load stays `Due`. Behaviour stays unchanged.

Acceptance:

- `rg -n 'Now::PrepareAgent|Event::PreparedAgent|transcript_waiter|load_transcript' jig temper --type rust` reads nothing.
- `rg -c 'core::Now::' jig/examples/ops/domain/src/route.rs` prints 39.
- `temper-engine-domain/src/route/now.rs` has no `unreachable!` in `start_preparation`.
- `release` in `temper/tests/engine/domain/tests/suite/forge_routes/reviews.rs` is unchanged and passes.
- The tests of step 6 pass, and each bit when its rule was broken by hand.
- `jig/docs/design/domain/root.status.md` has no entry for 3, and `jig/docs/design/STATUS.md` no sentence on it.
- `tools/citations/citations.py --all` reads 0.
- What `temper/docs/plan/debt-8/notes/common.md` calls `What runs` passes.

## Inputs

- `temper/docs/plan/debt-8/notes/common.md` and `temper/docs/plan/debt-8/notes/contracts.md`, whole: the shapes bind.
- `AGENTS.md`, `jig/AGENTS.md`, `temper/AGENTS.md`.
- `jig/docs/design/domain/root.md`, 3. `temper/docs/design/domain/root.md`, 4.1.

## Owns and may change

- Owns: `jig-domain-core`'s `routing/run/preparation.rs`, `routing/mod.rs`, `routing/stopping.rs`, `vocabulary.rs`
  and its tests.
- temper's root, as the note sketches: `route.rs`, `route/now.rs`, `boundary.rs`, and the `Work` list in
  `route/forge_route.rs`. This departs from a branch staying in its tree: the variants' removal is one atomic change.
- Removing the variants, only their arm or line: `jig/crates/jig-protocol-store/src/ended.rs`,
  `jig/testing/jig-test-service/src/route.rs`, `jig/testing/jig-test-service/src/clients/translation.rs`,
  `jig/testing/jig-test-store/src/route.rs`, `jig/tests/client/src/joint/engine.rs`,
  `jig/tests/client-protocol/src/engine.rs`, `jig/tests/core/src/client_peer.rs`, `jig/tests/ops/src/joint/engine.rs`,
  `temper/crates/temper-service/src/clients/translation.rs`, and `jig-domain-core/src/tests/runs/completion_retry.rs`.
  debt-7 is changing `temper-service` and `jig-test-service`: if an arm has moved on the trunk, edit the one `rg`
  finds. If a file is gone, the arm went with it.
- `Now::StartPreparation`'s `from` and its lost waiter, that field only: `jig/testing/jig-test-store/src/route.rs`
  (its `Some` pattern and `from: None` arm), `jig/testing/jig-test-store-files/src/tests/routes.rs`,
  `jig/tests/core/tests/suite/store_adapter.rs`, `jig/tests/ops/src/conformance/mod.rs` and its reader
  `jig/testing/jig-conformance/src/core/peers.rs`.
- `Event::Activate`'s new field, `busy: false` in each literal: `jig/examples/ops/domain/src/route.rs` and `domain.rs`,
  `jig/testing/jig-test-domain/src/routes.rs` and `lib.rs`, `jig-domain-core/src/tests/jobs.rs`,
  `tests/runs/notes_room.rs`, `tests/start/runs.rs`, and any other the compiler names.
- Status: `jig/docs/design/domain/root.status.md`, the entry for 3 and the date line; `jig/docs/design/STATUS.md`, the
  sentence `Every root still carries an agent's preparation back to the core`, and its date line.
- `jig/docs/design/domain/root.md` is already written; change it never. D001 and D004 are `d001-close`'s.
- Pre-approved: a `Token` or `remember_due` import an edit leaves unused goes. If a test needs an admitted agent, it
  uses `notes_room.rs`'s `ready()`. If a decision that activates two windowed agents can't be reached in the world,
  skip step 6's third story and say so in the report.

## Steps

1. `routing/run/preparation.rs`, as the note says: add `transcript_from`; `route_activate` takes `busy`; its `Allow`
   arm goes as sketched; `route_prepared_agent` loses `transcript_waiter`, `busy` and its busy branch, takes `from`,
   and keeps its phase guard. It emits `Now::StartPreparation { task: number, from }` when `from` is `Some`, else
   pushes `Work::StartBrief` as it does today.
2. `vocabulary.rs`, `routing/mod.rs`, `routing/stopping.rs`: the variants go, with their arms; `Event::Activate` gains
   `busy`.
3. temper, as the note sketches: `Work::Due`, `activate`, the `Now::Activate` arm, `start_preparation`. The arm for
   `Now::PrepareAgent` goes. `domain.rs`'s `Resume` pass does not change: the held spec's gate is wrong here, since it
   would hold jobs and agents with no window.
4. ops and the test root: their `Now::PrepareAgent` arms go; each `Now::Activate` arm answers `busy: false`, and each
   `StartPreparation` arm keeps its form.
5. Tests that named a removed variant keep their assertions and take the next step as a root would. Model:
   `origin/debt-5/core-prepare-agent`'s `f000f71c42` for `notes_room.rs`, `start/runs.rs`, `documents/virtual_words.rs`
   (read it, with `busy: false` added; drop nothing it did not drop). `Event::PreparedAgent { context, .. }` becomes
   `Event::Activate { context, ready: true, busy: false }`.
6. New tests:
   - Core, in `tests/runs/notes_room.rs`:
     `a_busy_root_keeps_an_agent_with_a_window_to_load_due`: `ready()`, a proof with an open window and
     `tries.transient == 0`, `Event::Activate { ready: true, busy: true }`: phase stays `Due`, `core.due` holds it, no
     `core.contexts` or `core.transcripts` entry, no `Now::StartPreparation`; `take_due`, then `busy: false`: one
     `Now::StartPreparation { from: (1, 1) }`.
     `a_busy_root_still_prepares_an_agent_with_nothing_to_load`: two cases with `busy: true`, each reaching
     `Preparing` with no load: no proof; an open window with `tries.transient != 0`. A third, a closed window with
     `tries.transient != 0`, does load, so it stays `Due`. This pins `transcript_from` against `begin_transcript`.
   - Job, in `tests/jobs.rs`: `busy: true` activates a job as `busy: false` does.
   - temper, in `temper/tests/engine/domain/tests/suite/upgrade_transcripts.rs`, built on its `waiting`/`pending_reply`:
     `a_full_read_table_keeps_a_waking_chat_due_until_a_read_retires`. Park `limits.loads.loads + 1` chats, each with a
     committed turn, wake them all, and hold each transcript reply in `driver.events`: the last chat issues no load
     and its task stays `Active::Due`. Answer one held reply: the next `Resume` issues the last chat's load. If the driver's
     fixture can take limits, use the least `loads.loads` that `limits::valid` accepts, 2, to keep the story short.
   - Third story: `two_activations_in_one_decision_share_the_last_read_slot`, in the same file, if a decision can wake
     two windowed chats: one activates, the other stays `Due`, and nothing panics.
   Break each rule by hand and report it: remove `busy` from the `Allow` arm; make `transcript_from` ignore
   `tries.transient`; read `busy` in the `Now::Activate` arm for the third story.
7. Status, as `Owns and may change` says. Run what `notes/common.md` names, and the citations check; commit the code
   and its tests (`D001: the core prepares an agent without asking its root`, saying why), the status in a second
   commit, and push.

## Report

The measure. Whether any test's visible events changed (none should). What each bite broke and what failed. Each
literal or arm outside the lists above. Lines added and removed.
