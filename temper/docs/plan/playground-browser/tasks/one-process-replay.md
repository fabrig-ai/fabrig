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
judging its end: the CSV change landed on `main`, every move of `main` green.

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
  (relative path from the test file), replays `Complete`: no difference at any input, every move
  of `main` green, the scripted CSV edit on `main` (the referee's `complete` checks those that do
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
  split so a replay can judge the forge alone; the one-process world's verdicts unchanged.
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
