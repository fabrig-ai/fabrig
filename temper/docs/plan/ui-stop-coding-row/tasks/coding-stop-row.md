+++
id = "coding-stop-row"
part = "coding-stop"
kind = "feature"
needs = ["jig-stop-interrupts"]
files = [
  "temper/tests/one-process/tests/suite/one_process.rs",
  "temper/tests/one-process/src/world.rs",
  "temper/tests/one-process/src/stories.rs",
  "temper/tests/one-process/src/referee.rs",
  "temper/tests/one-process/src/referee/**",
  "temper/tests/one-process/src/fixtures.rs",
  "temper/docs/design/protocol/README.status.md",
]
effort = "high"
minutes = 200
+++

# A stop costs a coding run nothing: the one-process row

## Goal

The request's row, in temper's one-process world. A coding run's task has its agent mid-turn, after a committed edit, at serve's first signal. After the drain and a restart on the same store, five things hold. The task is due again and its next attempt starts. No try is counted against it. Its change's counts are as before the stop. Its saved work is kept. Its writer slot and checkout serve the new attempt. The jig half has merged: on hearing it stops, the core settles each agent claim on an in-process worker as interrupted (`tasks::End::Recheck`, no try), and a crash still settles it lost (`jig-stop-interrupts`; jig's `domain/engine.md`, 6.1). So the row passes on arrival. Its red is shown by the bite, with that fix undone locally. The trunk's chat row in the same test, `a_signal_mid_run_drains_and_the_next_start_settles_the_claims`, is red since that merge. Its restart waits for a `Failed(Lost)` settle that no longer comes. Repair it as the coding row's sibling.

This task departs from the implementer's text in two ways: its bite undoes a merged fix in another tree, locally and never committed, and one status entry is yours (acceptance 6).

Acceptance:

1. **One table test of two rows.** `a_signal_mid_run_drains_and_the_next_start_settles_the_claims` (`temper/tests/one-process/tests/suite/one_process.rs`) becomes `a_signal_mid_run_interrupts_its_run_and_the_next_start_takes_it_again`: one runner, `world::drain_mid_run(settings, story, true)`, over two rows.
   - **chat**, `Story::ChatReadThenWait { words: b"Hello" }`: today's run, whose first turn is durable at the signal. It keeps its check that `settings_revision == 2`.
   - **coding**, a new `Story::CodingAtStop`: a chat plans and delegates the `@edit` change, as `Story::Adopt` does without its hooks. The producer's first turn commits its edit (`shell`, `echo fixed > conflict.txt`). Its next model reply is held by the fake LLM (`skein_fake_llm_domain::api::Line::Pieces` with a `Piece { hold: Some(..) }`), and the signal comes once the provider has seen that query.
2. **What every row checks**, from a new `Outcome.stopped: Option<world::Stopped>`, the run the signal found mid-turn:
   - its claim was interrupted at the stop: one `CoreRecord::Terminal` with `End::Recheck` for that task and attempt among the drain phase's rows; `claims_settled_at_restart == 0` (no `Failed(Lost)`);
   - **no try counted:** its `tries` (`jig_domain_tasks::Tries`) at the restart's end equal those at the signal, `lost` zero;
   - **due again:** the restart claimed it and started a newer attempt, whose agent's `Start` (`referee::agent::Record::Start`) names a prepared workspace;
   - **the writer slot:** no `Stored::Writer` slot names the stopped attempt at the restart's end, and the slots the new attempt's claim needs are its, none `lost`.
   The coding row also checks:
   - **its change's counts:** the `Stored::Change` row's `repairs`, `resolutions`, `updates` and `gate_failures` are equal at the signal and at the restart's end;
   - **its saved work:** the producer task's `saved` resources (`TaskRecord.saved`) are equal at the signal and at the end, and the new attempt's checkout starts from them (`temper/docs/design/domain/worker.md`, 5, *Saved work*). This is non-empty by the option below where it can be.
3. **The bite, both ways.** The row fails with jig's fix undone (acceptance 4) and passes with it. Its CPU time alone is in the report, and the coding row's story is the shortest that shows the rule.
4. **The red, shown locally:** in your slot, revert the merged `jig-stop-interrupts` fix commit (not its rows) with `git revert --no-commit`. Run the table alone and see the coding row fail on `tries.lost == 1` and on no `Recheck` terminal. Then `git checkout` the reverted files back and run it again, green. Nothing of the revert is committed. The report quotes the failure.
5. Clippy is clean for `temper-one-process-world`; its focused tests pass, and its `fuzzy::` tests (`--profile fuzzy`) too. `heavy cargo check --tests -p temper-one-process-world` compiles.
6. **Status:** the **Not yet** for 8 that names `ui-stop-coding-row` is gone from `temper/docs/design/protocol/README.status.md`, the rest unchanged.

## Inputs

- `temper/docs/plan/ui-stop-coding-row/notes/common.md`, all of it.
- `temper/docs/plan/ui-stop-coding-row/inputs/request.md`.
- `temper/docs/design/protocol/README.md`, 8, *Starting and ending together*, as this plan revised it; jig's `jig/docs/design/domain/engine.md`, 6.1, *Stopping* (a run in the engine's own process), and section 6, step 3; `jig/docs/design/domain/tasks.md`, 5.2 and 5.5.
- `temper/docs/design/domain/worker.md`, 4.2, *Lifecycle* (step 7, the save at park and cancel), and 5, *Saved work*; `temper/docs/design/domain/testing.md`, 2.5: the one-process world.
- What this cut's survey found, read only, in `temper/tests/one-process`:
  - **The drain's signal.** `Phase::Drain` completes, and the referee sends `ServiceSignal::Terminate`, once the store holds `Receipt::Turn(1, 1, 1)` (`src/referee.rs`, `phase_complete`, near 271). There is no hold: `read_then_wait_provider` answers at once (`src/fixtures.rs` near 120).
  - **The restart's completion.** `Phase::Restart` completes once `run.settled > 0` and the cold start has passed (near 274). `settled` counts `Failed(Lost)` terminals (near 563), and that is what goes red after jig's fix.
  - **Holding a reply.** `Line::Pieces` with `Piece { text, hold: Some(name) }` (`skein/crates/skein-fake-llm-domain/src/api.rs` near 402 and 441) waits until `Peer::release(name)` (`skein/crates/skein-fake-peers/src/llm.rs` near 202). A hold is not observed, but the query is (`Observation::Query`). An unreleased hold keeps the peer from emptying, which fails the world's end (near 192 and 511). Release it once the stopped agent can no longer read the reply. The referee shuts peers right after it asks for serve's stop (near 384).
  - **Records.** A task is `Stored::Live(TaskRecord)`, with `phase`, `attempt`, `tries` and `saved`. A change is `temper_engine_domain_forge::Stored::Change(ChangeRow)`, whose `delegate` names the producer and whose `change` holds the counts. A writer slot is `jig_domain_tasks::Stored::Writer(WriterSlot { writer: Writer::Run { task, attempt }, lost, .. })`. `Store::restored` is the recovered prefix before the restart's commits.
  - **The stories.** Only `HeldChange` and `Phase::Fault` carry the fake forge across phases (`forge_for_run`, `src/world.rs` near 1035). `Adopt` requires its hooks after every phase (near 1166). A fresh `Drain` sets `first_start` false, so the forge's connection setup must not depend on it for the new story (`src/stories.rs` near 336). A new `Story` variant forces two exhaustive matches, `Person::new` (`src/world.rs` near 433) and `phase_complete` (`src/referee.rs` near 278), and the `matches!` uses beside them need reading too.

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours, edits allowed:**

- `temper/tests/one-process/src/**` beyond the owned files (`ctl.rs`, `oracle.rs`, `public/*.rs`, `lower/*.rs`, `world/tests.rs`): what the new variant and the `drain_mid_run` signature need, and an oracle this plan's correct behaviour trips, each named.
- `temper/tests/one-process/tests/suite/fuzzy/*.rs` and `tests/suite/*.rs`: call sites of a changed helper only.
- Nothing in jig, skein, smith or temper's crates changes. A cause there is a `Question:` with the evidence (common notes).

**Options, each with its default:**

- **Saved work before the stop.** Default: the producer saves once before its held turn, so `saved` is non-empty at the signal. It parks, its save pushed and recorded (`worker.md`, 4.2, step 7), and is woken again: the `Nothing left to await` notice wakes a parked result-ended task (jig's `tasks.md`, 5.2). This works if the world's producer can park and be woken with what its tools and the fake already do. The park's run try then shows in `tries` before and after alike. If it can't without new machinery, keep `saved` empty. Then check that it is equal, that the new attempt's checkout starts from the base, and that the saved-work branch on the fake forge is not rewound, and say in the report that the row holds no saved work and why.
- **The coding story's start.** Default: `Story::Adopt`'s person and provider (`stories::person`, the `@plan` delegate and `@edit` scripts) with `hooked: false` and no hook check, the forge carried across phases as `HeldChange` carries it. If adoption is not needed for a change on the fixture's repository, start from `HeldChange`'s instead, and say which.
- **The drain's moment.** Default: `phase_complete`'s `Phase::Drain` per story. The chat row keeps `Receipt::Turn(1, 1, 1)`. The coding row waits for the producer's first turn to be durable and the provider to have seen its next query.
- **The restart's moment.** Default: `Phase::Restart` completes once the stopped task's newer attempt has an agent `Start`, for both rows. The coding row's restart provider holds the same reply again, and the referee releases each hold after serve's stop has ended that agent. If the chat row's resumed run ends before its `Start` is observed, take its newer attempt's claim instead.
- **What `claims_settled_at_restart` becomes.** Default: it stays, counting `Failed(Lost)` terminals, now `0` for both rows, and `Stopped` carries the rest. Keep `Outcome`'s other fields as they are for `run`, `crash` and `refused_restore`.
- **A row the bite doesn't turn red.** If the coding row passes with jig's fix reverted, read why first. Perhaps the producer committed no turn before the signal (its claim is then not started, `End::Refused`, no try either). Move the signal after a durable producer turn. If jig's fix is shown not to hold for a coding run (a lost try after it, a writer slot left `lost`, a prepare refused), stop with `Question:` and what you saw.

## Steps

1. **The story and the outcome** (acceptance 1 and 2), by the options. Sketch, inside which the choices are yours:

   ```rust
   // src/world.rs
   pub enum Story { /* … */ CodingAtStop }

   /// What the first signal left of the run it found mid-turn, and what the
   /// next start made of it (protocol/README.md, 8).
   #[derive(Debug)]
   pub struct Stopped {
       pub task: u64,
       pub attempt: u64,
       pub interrupted: u32,
       pub tries: [jig_domain_tasks::Tries; 2],
       pub saved: [Box<[jig_domain_tasks::SavedResource]>; 2],
       pub change: Option<[Counts; 2]>,
       pub next: Option<u64>,
       pub writers: Vec<(u64, bool)>,
   }

   #[must_use]
   pub fn drain_mid_run(settings: Settings, story: Story, restart: bool) -> Outcome
   ```

   `Counts` is yours: the change's four counts, compared whole.
2. **The table** in `tests/suite/one_process.rs`. Run it alone (`heavy cargo nextest run -p temper-one-process-world --test suite -E 'test(=one_process::a_signal_mid_run_interrupts_its_run_and_the_next_start_takes_it_again)'`, checking the count), with each row's CPU time: it passes. Commit ("temper: a coding run stopped mid-turn costs nothing at the next start").
3. **The bite** (acceptance 4), then the restore, both run alone.
4. **The status entry** (acceptance 6).
5. **The world's suites,** once each: `heavy cargo nextest run -p temper-one-process-world`; `heavy cargo nextest run -p temper-one-process-world --profile fuzzy`. Repair what the change trips (Owns and may change).
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p temper-one-process-world --all-targets -- -D warnings`. Commit ("temper: the one-process drain table holds a coding run's stop"), push.

## Report

As the implementer's text says, and: the bite's red (acceptance 4) and the restore; each row's CPU time alone; whether the coding row holds saved work and how (option); the story's start and the drain's and restart's moments as taken; each oracle repaired and why; anything under `## Left`.
