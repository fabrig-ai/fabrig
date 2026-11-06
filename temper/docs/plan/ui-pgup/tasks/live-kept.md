+++
id = "live-kept"
part = "live-kept"
kind = "feature"
needs = ["reproduce"]
files = [
  "jig/crates/jig-client/src/thread/model.rs",
  "jig/crates/jig-client/src/tests/thread.rs",
  "jig/docs/design/client/domain/client.status.md",
]
effort = "high"
minutes = 35
+++

# A run's end leaves the words until their committed turn takes their place

## Goal

`Thread::run` clears a thread's live turns the moment a run state says there is no attempt. If that delivery comes before the committed turn's page, the words leave the page for as long as the read takes: the log's height collapses, the scroll clamps to the new foot, the view measures as at its end, and it follows from then on. A person reading back is thrown to the foot by a turn's ending, which the design forbids. A run's end must take nothing away by itself: `Thread::end_run`, called by each owner once the committed activity has been read, already drops what no committed turn replaced.

Acceptance:

- A run state with no attempt leaves `Thread::live` as it is.
- A new attempt still clears every live turn, a committed turn still replaces its own, and `end_run` still drops a lost turn.
- The tests below pass, and the readers' tests in step 5 pass.
- The **Not yet** bullet this plan added to `jig/docs/design/client/domain/client.status.md` is gone.

## Inputs

- `temper/docs/plan/ui-pgup/notes/common.md`, then `temper/docs/plan/ui-pgup/reproduced.md`.
- `jig/docs/design/client/domain/client.md`, section 10, *Live, then committed*. Those lines are written; you build them.

**Departure from `common.md`:** this goes in whatever `reproduced.md` says of a turn's ending. Which of the run's end and the task's move a client hears first is the engine's order at that moment, so a run that happened to see the words stay doesn't show they always do. If it saw them stay, say so in your report.

## Owns and may change

- **Owns:** the files in this spec's header.
- **May change, how:** a test elsewhere whose oracle or helper trips on the words staying a moment longer: repair it, keeping what it checks for everything else, and name it in your report.
- **Options:**
  - If a reader's test shows a lost turn's words never going, because that owner never reaches `end_run`, fix that owner's call in its own package, one line, with a test, and say so.
  - If `reproduced.md` shows the words leaving by another path (`end_run` called too early, say), fix that too if it is in `jig-client`. If it is in `temper-client-domain`, leave it for `ending-keeps`, under `## Left`.

## Steps

1. Read `jig/crates/jig-client/src/thread/model.rs`, `Thread::run`, and `jig/crates/jig-client/src/thread.rs`, `end_run`. The callers that decide when it runs are `temper/crates/temper-client-domain/src/pages/chat.rs`, in `told` (`self.run.is_none() && self.thread.loaded >= self.thread.committed`), `temper/crates/temper-client-domain/src/pages/task.rs`, two places, and `jig/crates/jig-client-view/src/state/activity/routing.rs`, `finish`.

2. In `jig/crates/jig-client/src/tests/thread.rs`, beside `a_new_attempt_clears_every_provisional_turn`, using `model`, `run`, `provisional` and `commit` as they stand. Run the first red:
   - `a_runs_end_keeps_its_live_turn_until_its_committed_page`: a run and a provisional turn; a run state with `attempt: None`: `thread.lines()` still holds the same `Line::Provisional`. Then `commit` the turn: the `Line::Turn` alone.
   - `end_run_takes_a_turn_lost_with_its_host`: a run and a provisional turn; a run state with no attempt; `thread.end_run()`: no line.
   - `a_new_attempt_after_a_runs_end_clears_the_lost_turn`: a run and a provisional turn; no attempt; then a new attempt: no line.

3. In `Thread::run`, return before the attempt's comparison when `run.attempt` is `None`, and clear nothing:

   ```rust
   if run.task != self.task {
       return;
   }
   // A run's end takes no line away: `end_run` does, once the committed activity has been read (client.md, 10).
   if run.attempt.is_none() {
       return;
   }
   if self.attempt != run.attempt {
       self.live.clear();
       self.attempt = run.attempt;
   }
   ```

   `self.attempt` keeps the ended attempt, so the next attempt still differs and clears. Update the doc comment on `Thread::run` if what it says stops being true.

4. Remove the **Not yet** bullet from `jig/docs/design/client/domain/client.status.md`.

5. **A change of meaning runs its readers' tests.** Once: `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-client --all-targets -- -D warnings`; then `heavy cargo nextest run -p jig-client -p jig-client-view -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-world`; `python3 -I -B tools/citations/citations.py --base origin/main`.

6. Commit and push.

## Report

What `reproduced.md` said of a turn's ending, and the order of the two deliveries, if it could see it. The red you saw first, the rule you broke for each new test and the failure seen. Any test elsewhere you repaired. The tests, how many, and your minutes.
