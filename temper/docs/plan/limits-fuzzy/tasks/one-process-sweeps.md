+++
id = "one-process-sweeps"
part = "one-process"
kind = "feature"
needs = []
files = [
  "temper/tests/one-process/**",
  "temper/docs/plan/limits-fuzzy/figures/one-process-sweeps.md",
]
effort = "high"
minutes = 110
+++

# The one-process world's two crash sweeps

## Goal

`fuzzy::one_process::drawn_kills_keep_the_conformance_rules_and_answer_adoptions_once` costs 4.9 CPU s and `fuzzy::one_process::drawn_power_losses_keep_the_conformance_rules` 4.7. Only this world's focused stories have been profiled. Profile both sweeps, name each cause, fix the largest in `temper/tests/one-process/**`, and stop when the next is under about 2% of a sweep. Each still crashes all four seeds, at the same drawn cut, under every check it has now.

Acceptance:

- `temper/docs/plan/limits-fuzzy/figures/one-process-sweeps.md` holds, in `notes/measuring.md`'s form, instructions and native CPU before and after for both sweeps, with each cause fixed, what it saved, what's left and what you stopped at.
- Both sweeps run seeds `1..=4`, and each seed draws the same `before` as on your base: the figures file has the table of seed and `before`, both arms, for both sweeps.
- Every assertion in `temper/tests/one-process/tests/suite/fuzzy/one_process.rs` stands. `world::crash`, its signature and its readers keep working, and that file's diff is empty unless you say why.
- The cold readback stays: every public document in `oracle.rs`'s `READS` is read from the live service before its shutdown, read again after the cold restart, and compared.
- `heavy cargo nextest run -p temper-one-process-world` and the same with `--profile fuzzy` pass, with the same tests by name as your base.

## Inputs

- `temper/docs/plan/limits-fuzzy/notes/common.md`, `temper/docs/plan/limits-fuzzy/notes/measuring.md`.
- `temper/docs/design/domain/testing.md`, 2.5 and 6.1; `skein/docs/foundation/testing-strategy.md`, 8.
- `temper/tests/one-process/tests/suite/fuzzy/one_process.rs`; `temper/tests/one-process/src/world.rs`, `crash` and `run_phase`; `src/oracle.rs`; `src/referee.rs`.

## Owns and may change

Owns `temper/tests/one-process/**` and the figures file. A cause in the service, the engine, a fake, or a kit under this world goes under `## Left`, with its share, for `under-causes`.

`world::crash` has these two sweeps as its only readers. `Phase` is `pub(crate)`, so a new variant touches only this crate.

Where to look first. Reading shows these; the profile decides whether any is the cause:

- **`world::crash` runs three deployments per seed:** a setup that runs the whole `Story::Adoption` to learn the pending window, the same story again with its cut armed, and `oracle::cold`. Both sweeps run the same four seeds, in separate processes. The setup's `Outcome` is dropped; it's read only for `run.window`.
- **`Referee::adoption_window`** runs at every `observe`: `saved_adoptions()` builds a set from the client's cache, and once a window has begun, it scans the store's rows, until the window closes.
- **`Referee::observe`:** `rules.observe(Durable { snapshot: snapshot(&self.committed) })` at each commit, where `snapshot` walks every committed row, and the forge's `observe(procs)` at each step. `limits-worlds` gave jig's conformance harness keyed, lazy views of current rows (`jig/testing/jig-conformance/src/core/projection.rs`), if the same shape would hold here.
- **`ServiceObserver`:** `started` builds `fixtures::codec()`, which parses the engine's document through `fixtures::engine`, and `temper_protocol_store::bounds(..)` per process start, and `before_iterate` runs `store.observe(..)` per iteration.
- **`check_roots`** formats `secrets::Roots(service)` at the cut, and `run_phase` rebuilds its programs, settings and forge per phase.

Options, each decided:

- **The setup's tail, past the window's close, is over 2% of a sweep:** stop the setup when `run.window` is set, with a new `Phase` (say `Phase::Window`) whose referee ends the world there. What the setup's `Outcome` would have said was never read, so it's no check. The referee's and the fakes' per-step assertions in that tail are checks: the armed run runs the same referee, the same seed and the same story to its end past its restart, and your report names which assertions in the tail it doesn't reach, by file and line. If any of those isn't held by another test by name, keep the tail and say so.
- **The armed run's prefix, which repeats the setup's, is still over 20% of a sweep after that:** you may use `skein/testing/skein-fork`, as `temper/tests/conformance` does, so that one run holds the prefix once. Fork so that the parent stays the armed run and a child alone finds the window's end, so the `Outcome` stays in the parent. Then add one fuzzy test, in `fuzzy::one_process`, that a forked crash ends as a fresh crash does on one seed, and count its cost in your figures. If `Phase::Story` and `Phase::Fault`'s prefixes differ in ways that a fork would change (the client's root, `stories::setup_settings`, `oracle::begin`), or the kit doesn't fit, don't build it, and say what you measured.
- **A per-step check scans the whole state:** give it a position or a key where it reads only what changed. It must still fail for the same fault: break it once by hand and name what you saw.
- **Setup rebuilt per deployment:** build it once per test, where nothing in it varies with a phase, as `fixtures::store_names` holds its names in a `OnceLock`.
- **A cause in a focused story:** focused tests share this code. If a fix pays there too, say by how much, and don't chase it.

## Steps

1. Profile both sweeps on your base, as `notes/measuring.md` says, and take their native CPU. Write down each seed's `before`, from the assertions' own formatting or a temporary `eprintln!` that's gone before you commit.
2. Fix the largest, re-profile, repeat until the stop rule.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-one-process-world --all-targets -- -D warnings`; the runs in the acceptance; commit and push.

## Report

Instructions and CPU seconds before and after, each cause and what it saved, how you know every crash, check and ending stands, what you left and whose it is, and your implementation and check minutes apart.
