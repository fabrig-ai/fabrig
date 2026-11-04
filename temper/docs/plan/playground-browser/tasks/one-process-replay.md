+++
id = "one-process-replay"
part = "replay"
kind = "feature"
needs = ["native-walk-a"]
files = [
  "temper/tests/one-process/Cargo.toml",
  "temper/tests/one-process/src/lib.rs",
  "temper/tests/one-process/src/replay.rs",
  "temper/tests/one-process/src/referee/forge.rs",
  "temper/tests/one-process/tests/suite/main.rs",
  "temper/tests/one-process/tests/suite/replay.rs",
  "Cargo.lock",
]
effort = "high"
minutes = 120
+++

# Story A's kept log replayed natively under the one-process world's referee

## Goal

"A log replayed: a session log of story A, kept as a fixture, replays natively under the
one-process world's referee to the same end" (`temper/docs/design/playground.md`, 9); "The
application's referee runs as in any world, so a bug a visitor met is in the lowest world from the
start" (`skein/docs/design/playground.md`, 6). Replay the kept log
(`temper/tests/playground/tests/fixtures/story-a.log`, `notes/contracts.md`, C7) over the
playground's composition, with the one-process world's forge referee observing every turn and
judging its end: the CSV change landed on `main`, every merge green at its head
(`temper/docs/design/domain/forge.md`, 8.7: "Judged at the merge's head"), every other
move of `main` green at its target, and `main`'s head green at the end.

Acceptance:

- `temper/tests/one-process/src/replay.rs` gives:

  ```rust
  /// The one-process world's forge referee as a `skein_sim_world::Turns` over the playground's
  /// composition (skein's playground.md, 6.2): it observes each turn as in the one-process world.
  pub struct Refereed { /* the forge referee */ }
  impl skein_sim_world::Turns<temper_playground::programs::Process<()>> for Refereed { .. }
  /// Replays `log` (its text) over `temper_playground::world` built from story `story` at the
  /// log's seed and start, with `story::reach()` and the parameters installed; the referee's
  /// verdict at its end.
  pub fn replay(story: &temper_playground::stories::Story, log: &[u8]) -> Replayed;
  pub enum Replayed { Complete, Incomplete { why: String }, Differs(skein_playground::replay::Differs) }
  ```

- `temper/tests/one-process/tests/suite/replay.rs`, `story_a_kept_log_replays_under_the_referee`:
  the kept log, read by `include_bytes!` from `temper/tests/playground/tests/fixtures/story-a.log`
  (relative path from the test file), replays `Complete`: no difference at any input, the green
  checks as the Goal says them, the scripted CSV edit on `main` (the referee's `complete` checks those that do
  not need the terminal client's person).
- The referee's `Forge::observe` takes the composition's processes for any harness (`&[Process<H>]`
  for `H: Harness`, or the slice it reads), so the one-process world's own uses are unchanged.
- `heavy cargo nextest run -p temper-one-process-world` passes with the trunk's count plus one,
  within a world's budget (`temper/docs/design/domain/testing.md`, 6.1); `heavy cargo clippy -p
  temper-one-process-world --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/playground-browser/notes/common.md`;
  `temper/docs/plan/playground-browser/notes/contracts.md`, C1, C2 and C7.
- `temper/docs/design/playground.md`, 9; `skein/docs/design/playground.md`, 6 and 6.2 (the
  replay's loop: the referee observes each turn through the world's `Turns`; the world handed
  back unsettled).
- `skein/crates/skein-playground/src/replay.rs` (`replay`, `Differs`);
  `skein/crates/skein-ui-web/src/log.rs` (`read`); `skein/crates/skein-sim-world/src/lib.rs`
  (`Turns`, ~28).
- The referee: `temper/tests/one-process/src/referee.rs` (`impl skein_world::Referee`, ~274) and
  `temper/tests/one-process/src/referee/forge.rs` (`Forge::{new, observe, complete}`, `green`);
  story A's outcome, `temper/tests/one-process/src/stories/playground.rs` (~83-115).
- How the native walk replays the same log with no referee:
  `temper/tests/playground/tests/suite/walk.rs` (`story_a_kept_log_replays_natively`).

## Owns and may change

- The files in `files`. `Cargo.toml` gains `skein-playground`, `skein-ui-web` and
  `skein-sim-world` if not there, workspace dependencies.
- `src/referee/forge.rs`: `observe` made generic over the harness, and what `complete` needs
  split so a replay can judge the forge alone; a landed commit judged by its merge's head (the
  Answers' 2026-11-04 entry); no test of the one-process world changes its result for it.
- Not yours: `temper-playground`, `temper/tests/playground` (the fixture is read, never written:
  a log that does not replay here is a question), the one-process world's other stories, and its
  `#[ignore]`d story A (`tests/suite/story_a.rs`), whose stall is another plan's.
- Pre-approved options:
  - If the forge referee needs a part of the one-process referee that observes the person or the
    kept links (`Links::Keep`), leave that part out of `Refereed` and say which checks the
    replay judges; the log's digests prove the visitor's answers.
  - If the forge referee cannot read the forge from `Process<()>` (it reads it through the
    harness's types), build `Refereed` over the shared forge handle the composition's start
    gives (`Start::fakes.forge`), cloned before the world is built.
  - If `replay` differs on the kept log here while the playground world's own replay passes,
    report the `Differs` whole and stop and ask.

## Steps

1. `Forge::observe` generic; the one-process world's tests unchanged (run them).
2. `replay.rs`: `Refereed`, `replay`, `Replayed`.
3. The test; `cargo fmt`, the old-form count, clippy and the package's tests; commit per step;
   push.

## Report

As the implementer's text says, and also: which checks of the referee the replay judges and which
it leaves, and why; the replay's world time, step count and wall time.

## Answers

### 2026-11-03: Resume this task after native-walk-a publishes C7's kept story-a.log (recommended), or reassign production of that fixture? The adapter and required test are committed and pushed; the missing fixture prevents acceptance. (playground-browser/one-process-replay-1, foreman playground-browser/one-process-replay/foreman-1)

Resume one-process-replay after native-walk-a publishes C7’s kept story-a.log; fixture production stays with native-walk-a.
The spec declares `needs = ["native-walk-a"]` and says “the fixture is read, never written.”
The native-walk-a spec owns `temper/tests/playground/tests/fixtures/story-a.log` and requires recording it with the test that keeps it.
Acceptance requires the kept log to replay `Complete`, the package tests to pass with trunk’s count plus one within budget, and clippy to pass.
The task remains incomplete until those checks run successfully.
The next run consumes the dependency’s fixture, runs the required checks, and reports replay measurements and budget assessment.

### 2026-11-04: Assign investigation and repair of the pending-CI landing to the story/forge owner, or authorize changing the referee's green-at-move criterion? (playground-browser/one-process-replay-2, foreman playground-browser/one-process-replay/foreman-2)

Neither owner nor a contract change: the repair is this task's, in `src/referee/forge.rs`, and the
Goal and acceptance above now say it. The product, the story, the pace and the fixture stay.

The cause, read from the code (the answer's machine builds nothing, so the next run proves it):
head 3 is not the head that landed. temper merges a change with `MergeHow::Style`
(`temper/crates/temper-engine-domain-forge/src/domain.rs`, `Effect::Merge`), so the forge writes a
new commit onto `main` (`skein/crates/skein-fake-forge-domain/src/pulls.rs`, `merge`), and
`git::moved` starts CI on it. That CI is run 2, after the pushed head's run 1, and it is Pending
when `main` moves: the design gives CI 4000 ms in story A. `Forge::observe` checks `green` on
`Moved.to`, so it asks for the verdict of a commit that has only just been made. The rule the
design holds is "Judged at the merge's head" (`temper/docs/design/domain/forge.md`, 8.7),
which the `Merged` arm already checks at `head`. This is the first run to
reach this check under a pace, because story A's terminal walk is `#[ignore]`d.

Make these changes:

1. In `observe`, keep the check of `Merged.head` and remember each `Merged` with `commit != head`
   as (head, commit). A `Moved` of the default branch whose `to` is such a commit is judged by that
   merge: no `green` on `to`. Any other move of the default branch stays judged at `to`, so a
   direct push to `main` is still caught. Key this on the merge, never on the story.
2. `complete` and `forge_complete` have the same gap: `self.pushed.any(commit == outcome.main)`
   can never hold for a commit the forge wrote, since `pushed` holds branch heads. Pass when
   `outcome.main` is an observed pushed head, or the `commit` of a remembered merge whose `head` is
   an observed pushed head. Keep `outcome.green`: `main`'s own CI must pass by the end, and the
   native replay shows it does. Reword `replay`'s `Incomplete` to match.
3. The kept log's replay is this change's test; no other is asked for. Keep `fe806c37ef`'s
   diagnostic fields.

Stop and ask if the cause is otherwise: if the panic's head is a `Merged.head`, or that merge's
`commit == head`, a change landed on Pending CI, which is a product finding, not this repair. Put
what you see (the `Merged` and `Moved` of head 3) in the report.

The left failure, `public::a_store_written_before_public_reading_reads_every_project_private`
(`delivery document: TooLarge`), is not this task's. Keep it under `## Left` with its run ID; the
acceptance's package run reads as the trunk's count plus one apart from that test, and the trunk's
end check takes it. Report the replay's world time, step count, CPU and wall as the spec asks; over
a world's budget is 8.1's question, never a raised guide.

### 2026-11-04: Keep the 2-second story guide and authorize profiling and repair beyond this task's files, or set a different guide for this replay? The kept log completes, at 2.028213 CPU seconds. (playground-browser/one-process-replay-3, foreman playground-browser/one-process-replay/foreman-3)

Neither yet: the guide stands, nothing beyond `files` is authorized, and no miss is established.
The guide is compared by "the median of three runs", "run alone" with `-j 1`
(`temper/docs/design/domain/testing.md`, 6.1). 2.028213 s is one sample, taken in a package run
under the collector. The same replay with no referee took 1.876 s to 1.945 s
(`story_a_kept_log_replays_natively`, playground-browser/native-walk-a-15), and that run's walk
ranged 2.138 s to 2.221 s, so 28 ms over the guide is inside the noise. The commit
`8c902a6e45` stands: this run measures, and changes code only as 2 says.

1. On one machine, in one sitting, run each of these three times, alone, as native-walk-a-15 did:
   nextest's default profile, `-j 1`, through heavy, `TEMPER_BUNDLE=none`, user plus system time of
   the test binary with `time` as nextest's target runner:
   `heavy cargo nextest run -p temper-one-process-world --test suite -j 1 -E
   'test(=replay::story_a_kept_log_replays_under_the_referee)'` and `heavy cargo nextest run -p
   temper-playground-world --test suite -j 1 -E 'test(=walk::story_a_kept_log_replays_natively)'`.
   Report every sample and the machine.
2. A median of the first at or under 2 s meets the guide: no code changes, and the package and
   clippy results of `8c902a6e45` stand. If it is over, and the first's median exceeds the
   second's by more than the overrun, the referee is the cause and it is this task's: read `Forge::observe` (it runs on every one of the
   12,972 turns) and make it cheaper in `src/referee/forge.rs`, weakening no check, then measure
   again, and rerun the package and clippy once.
3. If the median is still over, stop with a `Question:` carrying both sets of samples and this
   recommendation for the user: extend 6.1's 3 s line for a playground story walked through the
   web client's host to its kept log replayed natively, since it runs the same 12,972 steps. A
   guide is the user's, so this run changes neither the guide nor the fixture, the pace or the story.

The other half of the question is closed: no profiling or repair in skein, jig, smith,
`temper-service` or `temper-playground`. Skein stays untouched in this plan (`notes/common.md`),
and those costs are already the backlog's by the user's answer to native-walk-a (the simulator
iterating idle processes, the service's readiness). The replay belongs in this tier
(`temper/docs/design/playground.md`, 9). The other tests past 2 s and the package's sum, which you
listed under `## Left`, stay there, with the inherited `public` failure: a sum is "never a landing's" (6.1).
