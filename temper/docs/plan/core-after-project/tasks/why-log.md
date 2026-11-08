+++
id = "why-log"
part = "core-says-why"
kind = "feature"
needs = ["hang-story"]
files = ["temper/crates/temper-service/src/core_wait.rs", "temper/crates/temper-service/src/tests/core_wait.rs", "temper/crates/temper-service/src/engine.rs", "temper/crates/temper-service/src/stop.rs", "temper/crates/temper-service/src/lib.rs", "temper/crates/temper-service/src/tests.rs", "temper/crates/temper-engine-domain/src/journal_stop.rs", "temper/crates/temper-engine-domain/src/route.rs", "temper/crates/temper-engine-domain/src/decision.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/lib.rs", "temper/crates/temper-engine-domain/src/tests.rs", "temper/crates/temper/src/host.rs", "temper/docs/design/domain/root.status.md", "temper/docs/design/protocol/README.status.md"]
effort = "high"
minutes = 180
+++

## Goal

A core that does not answer says why. While the service's gate keeps parties' inputs from the root, `temper serve`'s log names what holds it. A root whose journal has stopped ends the engine with the reason, never staying up undeciding in silence. In the incident (`temper/docs/plan/core-after-project/inputs/request.md`, what was seen) the engine logged nothing while every call failed, and the restart did the same.

This runs after `hang-story`, which found and fixed the cause. Its story's module doc comment (`core_after_project.rs`, in the world where it lives) names what the core waited on; read it first.

Acceptance:

- **The root's journal stop has a reason and a `Stop`** (`temper/docs/design/domain/root.md`, section 4, *A stopped journal ends the engine, naming why*). `temper-engine-domain` gains, in a new `src/journal_stop.rs`, exported from `lib.rs`:

  ```rust
  /// What stopped the root's journal (domain/root.md, section 4).
  #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
  pub enum JournalStop {
      /// The store's answer to `commit`, outside the commits sent and durable.
      Refused { commit: u64, sent: u64, durable: u64 },
      /// A decision past the store's per-commit write edge.
      PastEdge { writes: u32 },
      /// The deployment's commit counter at its end.
      Counter,
  }
  impl Domain {
      /// Why the journal stopped, when the root stopped it or refused an answer; a failed commit is the store's own stop.
      pub fn journal_stop(&self) -> Option<JournalStop>;
  }
  ```

  The `StoreInput::Committed` arm (`route.rs`) records `Refused` when `committed` stops the journal and emits `Request::Stop` in that step. `accept_routed`'s stop (`decision.rs`) records `PastEdge` or `Counter`. A root whose journal stopped emits `Stop` once.
- **The service ends on it, naming it.** `temper-service`'s `Stop` gains `Journal { why: temper_engine_domain::JournalStop }`. `stop::root` maps a root `Stop` with no start stop and a `journal_stop()` to it; every other mapping stays as it is. `temper`'s `host.rs` `ending` prints `protocol/README.md`, 8.2's line, e.g. `temper serve: the engine's journal stopped: the store answered commit 7, with 9 sent and 8 durable`, exit 1. A row of `ending`'s table (beside `Stop::CommitFailed`'s) for each `JournalStop` variant.
- **The gate's line** (`temper/docs/design/protocol/README.md`, 8.2). A new `temper-service/src/core_wait.rs`:

  ```rust
  /// Why the gate keeps a party's input from the root (protocol/README.md, 8.2).
  #[derive(Clone, Copy, PartialEq, Eq, Debug)]
  pub enum CoreWait {
      Starting { step: Option<jig_domain_core::start::Phase> },
      ForgeRecords,
      FirstPeriods,
      Resolving,
      Store { oldest: u64, in_flight: u64 },
      Watermark { backlog: u64, watermark: u32 },
  }
  /// One reason held; a line when it has kept inputs 1 s, again on change, and once when the gate passes again.
  pub(crate) struct Watch { /* since, written */ }
  impl Watch {
      pub(crate) fn kept(&mut self, now: Time, why: CoreWait) -> Option<crate::Note>;
      pub(crate) fn passed(&mut self) -> Option<crate::Note>;
  }
  ```

  `Note` gains `CoreWaiting { why: CoreWait }` and `CoreAnswers`, which `host.rs`'s `note_line` words as 8.2 says: `temper serve: the core is not answering: REASON` and `temper serve: the core answers again`. `engine.rs` calls `kept` where the gate answers a party's input not ready (`iterate`, `clients.not_ready`), and where `step` hands one back (`pending`, `blocked`), with the first reason of 8.2's table that holds. It calls `passed` where a party's input enters the root. The `Store` and `Watermark` reasons read the root's commit window and backlog through accessors you add on `Domain`, named for what they return. `Starting { step: None }` is the root not yet begun.
- **The cause `hang-story` fixed is nameable.** If its story's comment names a wait the gate can still meet that no reason above names, add it as one more variant and table row, worded like the others. Keep the 240 bytes.
- **Rows**, red first, in `temper-service/src/tests/core_wait.rs` (the `Watch` on supplied times, as `tests/long_pass.rs` does the long pass): no line before 1 s, one line at 1 s, none for the same reason again, a new line on a changed reason, `CoreAnswers` once after a line and none without one; one row per reason's words; a service stepped with its gate shut (its root before `Restart`, as `fixtures::started` leaves it) writes `starting: loading the core's records` once a party's input has waited 1 s. In `temper-engine-domain`'s `src/tests.rs`, beside `a_root_store_edge_overrun_releases_only_stop_and_discards_immediate_outputs`: a `Committed` the journal refuses emits `Stop` and records `Refused` with its numbers; an overrun records `PastEdge`. Break each rule once by hand (the `Stop` removed from the `Committed` arm; the 1 s check inverted), see its row fail, restore.
- `root.status.md`'s *Not yet* entry for section 4 and `protocol/README.status.md`'s for 8.2 are removed.
- `heavy cargo nextest run -p temper-engine-domain -p temper-service -p temper` passes, with `temper-engine-domain-world`'s and `temper-client-world`'s focused suites; clippy clean for the three.

## Inputs

- `temper/docs/plan/core-after-project/notes/common.md`.
- `temper/docs/design/protocol/README.md`, 8.2 (all of it) and 8.1 (the long pass's line, its rate and its tests: the model); `temper/docs/design/domain/root.md`, section 4, the last two bullets.
- `skein/docs/design/lib.md`, section 11 (the journal's stops).
- The story `hang-story` landed, and its module comment.
- The code: `temper-service/src/engine.rs` (`iterate`'s `open`, `input_pass`, `step`, `release`), `clients.rs` (`not_ready`, `early`), `stop.rs` (`root`), `lib.rs` (`Note`, `Stop`), `long_pass.rs` (the model for `Watch`); `temper-engine-domain/src/route.rs` (the `Committed` arm), `decision.rs` (`accept_routed`, `committed`), `domain.rs` (`finish_close`, `send_commit`, `start_phase`), `limits.rs` (`backlog`); `temper/src/host.rs` (`ending`, `note_line`).

## Owns and may change

- Owned: the files above.
- Readers, call sites only: every exhaustive match on `temper_service::Note` or `temper_service::Stop` in any crate (`temper`'s `host.rs`, the real loop's and the one-process world's note printers), each given its arm with the nearest existing behaviour.
- Options:
  - `Watch` as a field of the service's `Engine` (default), or of `Clients` if the not-ready answer sits there; whichever keeps `kept` and `passed` one call each.
  - If the gate shut by `hang-story`'s cause is a term of `open` not in 8.2's table, add the row (above); if it is a hold inside the root that `holding` cannot see, name it from the root's accessor you add, and say so in the report.
  - If a stop that already emits `Stop` today (an overrun in `finish_close`) would now emit it twice, keep it once.
- Not yours: the CLI (`sign-in-line`), jig's client and wire, and `hang-story`'s fix.

## Steps

1. Read the inputs and `hang-story`'s story comment.
2. The root's rows, red first; `JournalStop`, the `Stop` in the `Committed` arm, the reasons recorded.
3. The service's `Stop::Journal`, `stop::root`, the host's line and its table rows.
4. `Watch`, its rows red first, the reasons, the hooks in `engine.rs`, the notes and their words.
5. Break each rule by hand, see it fail, restore.
6. Remove the two status entries.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; clippy and the tests above.
8. Commit and push.

## Report

Each row and its broken-by-hand failure; the reasons' words as the tests assert them; where `kept` and `passed` are called; the match sites given arms; what `hang-story`'s cause reads as in the log.
