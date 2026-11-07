+++
id = "box-survivor"
part = "survivor"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/support/forge_land.rs",
  "temper/crates/temper/tests/support/forge_land/survivors.rs",
  "temper/crates/temper/tests/suite/box_lifetime.rs",
  "temper/crates/temper-box/src/**",
  "temper/crates/temper-service/src/stop.rs",
  "temper/crates/temper-service/src/worker.rs",
  "temper/crates/temper-service/src/worker/**",
  "temper/crates/temper-service/src/tests/stop.rs",
  "temper/crates/temper-service/src/tests/stop/**",
  "temper/crates/temper-forge-setup/src/runner.rs",
  "temper/crates/temper-forge-setup/src/tests/golden.rs",
  "temper/crates/temper-forge-setup-shell/src/**",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "high"
minutes = 240
+++

# A box leaves no process behind it

## Goal

After-landing's `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge` (`temper/crates/temper/tests/live.rs`) failed at `21a9819cc3` on `no surviving children`: the box exited 0 on its `SIGTERM` with a descendant still alive (`outcome.left`, from skein-world's `tree_status().live_at_leader_exit`, which holds process ids alone). Name the survivor, find why it outlives the box, and fix the cause at its lowest place under a failing test, or stop with the evidence where the cause isn't temper's to fix here.

This task departs from the implementer's text in four ways, each the request's: it runs one live story by name (the common note's *Runs*), its first fix commit is a test that fails, it may end in a `Question:` the options below name, and one status entry is yours (acceptance 5).

Acceptance:

1. **The survivor is named in the live log.** When the box exits with descendants alive, the live story's judge notes, once, a `SURVIVOR` line for each: its pid, parent, process group, session and command line, read from `/proc` when the box's exit is first seen; and for each, the row the judge kept at its stop of that survivor's parent and of its group's leader, where it kept one. Every line goes through `support::live::notice`, which filters `secret=` and tokens. The story's check, `assert_eq!(outcome.left, Some(vec![]), …)`, stays as it is.
2. **The red is reproduced and read.** The live story runs by name, alone (step 2), at most three times in all: once to record the survivor (a second time only if the first never reached the box's stop or left nothing), and once after the fix. The report quotes the `SURVIVOR` lines, filtered, and says which candidate below it is, with the box's log lines for the stop (`data/box/logs/box.log` in the story's store: did serve, the runner and Forgejo's step each exit, or did the box close it after its second grace?).
3. **The cause is fixed at its lowest place,** where an option below says it's this task's: a focused test there is committed red first, then passes with the fix; the live story, run once more, passes its `no surviving children` check. Where the cause is a coding run's own child, or skein's or smith's, the task ends in `Question:` instead, as the options say, with the recorder (acceptance 1) pushed.
4. Clippy is clean for every package changed; the focused tests of every package changed and of its direct dependents pass; `heavy cargo check -p temper --tests` compiles the live and process binaries.
5. On a fix, the **Not yet** entry for 7.2, *Its children*, that names `ui-box-children` is gone from `temper/docs/design/protocol/setup.status.md`, the rest of the file unchanged. On a `Question:` or a red that doesn't reproduce, it stays, and you may rewrite its last two sentences to say what the record showed.

## Inputs

- `temper/docs/plan/ui-box-children/notes/common.md`, all of it: the runs, the credentials and the logs rules bind.
- `temper/docs/plan/ui-box-children/inputs/request.md`.
- `temper/docs/design/protocol/setup.md`, 7.1 (the tree and *A step's start*), 7.2 (*Its children*, and the stop's order), 6.4 (*The hidden `internal forge-child`*).
- `skein/docs/design/io.md`, 6, *Exit* to *A group is not a tree*: closing a child kills its group and reaps the child; it doesn't wait for the group's other members, and a descendant in another group or session escapes.
- `temper/docs/design/domain/testing.md`, 2.7, *The owned-forge landing story on Claude Code* and *After every landing*: what the story needs of its runner.
- What this cut's survey found, read only: the box spawns each step in a process group of its own and, on a stop, sends `SIGTERM` to each long step's leader last first, then again after `box.grace_ms` (10 s), then closes it after a second grace (`temper-box/src/machine.rs`, around `Phase::Grace`; `host.rs`, `Order::Signal` and `Order::Close`). serve's worker spawns each run's agent, `temper internal agent`, in a group of its own with no parent-death guard (`temper-service/src/worker.rs`, near 697); the agent spawns `claude` in yet another group (`smith/crates/smith-protocol-program/src/backend/start.rs`); Claude Code starts its own children (its Bash tool, the MCP relay `--mcp-relay`). `temper forge run … runner` execs `forgejo-runner daemon` in its step's group, with `shutdown_timeout: 0s` (`temper-forge-setup/src/runner.rs`); Forgejo runs under `forge run` in a group of its own, through `internal forge-child --signal kill`.

## Owns and may change

**Owned:** the `files` above. Most of them are where a fix may land; change only the ones your fix needs, and name them in the report.

**Readers and neighbours, edits allowed:**

- `temper/crates/temper/tests/support/forge_land.rs`: the `mod survivors;` line, two fields on `LiveJudge` and their initialisers in `run_live`, the snapshot in `stop`, and the note in `drive`. Nothing else in the judge changes.
- `temper/crates/temper/tests/live.rs` stays as it is.
- A design line your fix makes true or false in `temper/docs/design/protocol/setup.md`, 7.1 or 7.2 (a bound, an order): the line only, named in the report.
- Generated files your fix makes stale (a golden, `docs/help/reference/*`): regenerated by their own tool, named.

**Options, each with its default.** Read the `SURVIVOR` lines against the stop's rows and pick the first that fits:

- **A. A run's agent or its `claude`, left by serve** (the survivor is `temper internal agent …` or `claude …`, or its parent is one, and its group is the agent's or `claude`'s own): the box closed serve before serve's drain ended its agents, or serve exited before they did. Default: fix serve's stop so serve exits only once every agent process it spawned has closed, and so its drain ends inside the box's two graces without widening either (`temper-service/src/stop.rs`, `worker.rs`); the failing test is a row or a case in `temper-service/src/tests/stop.rs` or `stop/`. If the agent itself exits before its `claude` has, that's smith's: stop with `Question:`, as for D.
- **B. A coding run's own child** (the survivor's ancestry runs through `claude`, and it sits in a group or session `claude` or its sandbox made: a Bash tool's command, a background shell, the MCP relay, a sandbox helper): **stop with `Question:` and the evidence; that's temper/migration's** (the request, item 2). Options to name: smith ends a program's whole tree (a cgroup of its own, skein's `io.md`, 6.3), or the run's sandbox does; recommend the first.
- **C. A runner's job** (the survivor's ancestry runs through `forgejo-runner`): fix the runner's own configuration so it ends its jobs before it exits, inside the box's grace (`temper-forge-setup/src/runner.rs`), the failing test the runner's golden in `temper-forge-setup/src/tests/golden.rs`, and `temper/docs/design/protocol/setup.md`'s line for it. If only `forge run`'s runner path can end the job (not exec'ing the runner, so it can wait on its jobs), make that change in `temper-forge-setup-shell` with its test there, and say so. If neither can without a skein mechanism, stop as for D.
- **D. A member of a step's own group, or of Forgejo's, still dying** (the survivor's group is a step's leader's or Forgejo's, and it's gone from `/proc` moments later, or the stop's row shows it was a group member when the box closed its step): skein-io's close kills the group and reaps the leader without waiting for the group to empty (`skein/crates/skein-io/src/process.rs`, the close). That's skein's: stop with `Question:`, the evidence, and the fix you'd make there with its failing test (`skein/tests/io/tests/suite/processes.rs`, `closing_a_child_whose_group_still_runs_ends_the_group`, extended to wait for a member). If a box-level order shows it first (a step closed before its grace ran out, or a once-step's group left running), fix that in `temper-box`, its failing test a case beside `a_stop_signals_each_long_step_twice_last_first` (`temper-box/src/tests/host.rs`) or in `src/tests/machine.rs`.
- **E. Anything else** (a Forgejo `git` child in a group of its own, a `temper forge source` once-step, a step not started through `internal forge-child`): the same rule: fix it in the temper crate that spawns it, failing test first, in that crate's focused tests; a cause in skein or smith is a `Question:` as D.
- **A box_lifetime story:** default none. Add one beside its siblings in `temper/crates/temper/tests/suite/box_lifetime.rs`, on their fixture, only if the cause shows with real processes alone and no focused test can show it; it's compiled, not run (the common note), and the live run is its red-then-green evidence.
- **The red doesn't reproduce** in two runs (both pass, or both stop before the box's stop for a reason that isn't this task's): push the recorder, report `Done:` with the runs' figures, and list red-70 under `## Left`: after-landing's next red names its survivor.
- **No login:** if `TEMPER_LIVE_CLAUDE` is unset in your environment, or `claude` isn't on `PATH`, or the story can't reach them through `heavy`, never look for a login: push the recorder and report `Question:` with the options (fab reruns this task where the held login is; or the recorder lands and after-landing's next red names the survivor), recommending the first.

## Steps

1. **The recorder,** in a new `temper/crates/temper/tests/support/forge_land/survivors.rs`, `mod survivors;` in `forge_land.rs` beside `pub mod rung;`:

   ```rust
   //! The box's descendants that outlive it, named for the live log (the plan ui-box-children).

   /// One process as `/proc` shows it.
   pub struct Row { pub pid: u32, pub parent: u32, pub group: u32, pub session: u32, pub command: String }

   /// The rows of `pids` still in `/proc`, in order; a pid gone has none.
   /// `/proc/PID/stat`'s fields are read after its last `)`; `cmdline`'s NULs become spaces.
   pub fn rows(pids: &[u32]) -> Vec<Row>

   /// One `SURVIVOR pid=… parent=… group=… session=… command=…` line for each of
   /// `left`, a pid gone from `/proc` saying `gone`; then, prefixed `AT STOP`, the
   /// rows of `at_stop` whose pid is a survivor's parent or group.
   pub fn lines(left: &[u32], now: &[Row], at_stop: &[Row]) -> String
   ```

   In `LiveJudge`: `at_stop: Vec<survivors::Row>`, taken in `stop()` from `binary.tree_status().live_pids` before the `SIGTERM`; `survivors_noted: bool`. In `drive`, once stopping and not yet noted, when `binary.tree_status().live_at_leader_exit` is `Some(left)`: if `left` isn't empty, `super::live::notice(&survivors::lines(left, &survivors::rows(left), &self.at_stop))`; then noted. Test `lines` in forge_land's `mod tests` with rows built by hand (a plain survivor, one gone, one whose parent and group the stop's rows hold), and the stat parse with a `comm` holding `) (` and spaces: one table test, `survivor_lines_name_each_process_its_parent_and_group`. The focused suite runs it (`heavy cargo nextest run -p temper --test suite -E 'test(=support::forge_land::tests::survivor_lines_name_each_process_its_parent_and_group)'`; check the count, since the module's path may differ). Bite: drop the `AT STOP` rows once and see it fail. Commit ("temper: the live story names a box's surviving descendants").
2. **The live run,** by name, alone, through the proof script with `heavy` as its wrapper, from the repository root, a fresh store and a private `TMPDIR` of your own outside the checkout, under `~/src/fabrig/`, each made 0700:

   ```sh
   TEMPER_WRAPPER=heavy TEMPER_LIVE_STORE=<fresh store> TMPDIR=<private dir> \
     temper/scripts/proof-live.sh forge-land-claude
   ```

   `TEMPER_LIVE_CLAUDE` comes from your environment as it is; the script checks it by `stat` and uses it in place. Read the result with `grep` (`SURVIVOR`, `AT STOP`, `FAILURE`, `no surviving children`) and the store's `box.log` around the stop, never whole. Remove the store and the directory afterwards, once the report has what it quotes. If the build fails, it's this branch's: fix it before running again, which counts no run.
3. **The cause,** by the options: read the code the record points at, and say why that process outlives the box.
4. **The fix,** where an option makes it this task's: the failing test first, committed alone ("…: a test that fails while …"), its red named in the report; then the fix, the test passing, and the bite (the fix removed once, the test failing for the rule's reason). Then the live run once more (step 2), which passes `no surviving children`; if it fails on anything else that isn't this branch's, report it under `## Left` with the filtered lines.
5. The status entry (acceptance 5).
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p temper [-p each package changed] --all-targets -- -D warnings`; `heavy cargo nextest run -p temper --lib` and the focused tests of each package changed and its direct dependents (`-p temper-service` reaches `temper`; `-p temper-box` and `-p temper-forge-setup` reach `temper-service` and `temper`; name them in the report), the new test alone first with its CPU time; `heavy cargo check -p temper --tests`. Commit ("…: <the cause> ends with the box"), push.

## Report

As the implementer's text says, and: the `SURVIVOR` and `AT STOP` lines of each live run, filtered; the box's log lines for the stop; the option that fits and why; the runs made (at most three) with each one's wall time and how it ended; the failing test's red and the bite; and, on a `Question:`, the evidence, the options and your recommendation, with the recorder's commit pushed.

## Answers

2026-11-07 (temper/ui). Land the recorder: your second option. No held login reaches a worker, and after-landing runs this story with one at every landing, so its next red will name the survivor through your rows. I'll cut the repair from them. Finish this task as the recorder alone, and leave the live assertion and setup.status.md as they are. A lead for the record, not to act on here: just before the stop, the box's runner logged that it was forcing a shutdown of a job it had started 8 s earlier. So the survivor is likely one of that job's descendants, in a process group of its own, which the box's group kill misses (setup.md 7.2, Its children).
