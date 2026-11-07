+++
id = "box-lifetime"
part = "box"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/suite/box_lifetime.rs",
  "temper/crates/temper/tests/suite/box_listeners.rs",
  "temper/crates/temper/tests/suite/box_listeners/lifetime.rs",
  "temper/crates/temper/tests/suite/forge_land.rs",
  "temper/crates/temper/tests/fake_backends/box_link.rs",
  "temper/crates/temper/tests/process.rs",
  "temper/docs/plan/limits-heavy/figures/box-lifetime.md",
]
effort = "high"
minutes = 90
+++

# `temper::suite`'s real boxes and landing launcher

## Goal

Two rows of `temper::suite`. **`box_listeners`,** 8 tests and 17.6 CPU s: `lifetime::a_hangup_stops_a_box_and_leaves_nothing_running` 8.1 and `lifetime::a_killed_box_takes_its_steps_with_it` 7.8 start a box's playground steps, and are the process tier's by `temper/docs/design/domain/testing.md`, 2.6. Move those two there, whole, under the same names. The other six stay; profile them and fix the cause if it pays. **`forge_land`,** 44 tests and 4.0 CPU s, largest `the_launcher_gives_each_landing_story_its_own_store` 1.1: profile it and fix the cause.

Acceptance:

- `a_hangup_stops_a_box_and_leaves_nothing_running` and `a_killed_box_takes_its_steps_with_it` run in the `process` binary and in no other, with every assertion they have today. `a_killed_box_takes_its_steps_with_it` stays `cfg(target_os = "linux")` and `a_hangup_stops_a_box_and_leaves_nothing_running` keeps building on every platform it builds on today.
- Both passed by name in `suite` on your base, before the move, and the figures say so. After it, `heavy cargo nextest run -p temper --test process --no-run` builds. **They aren't run in `process`:** after-landing runs them.
- `suite`'s remaining `box_listeners` tests and `forge_land` pass.
- The design's citations say where the two now live, and the **Not yet** entry below is gone.
- `temper/docs/plan/limits-heavy/figures/box-lifetime.md` holds a row for each test of the two rows that you profiled or moved, and each row of `forge_land` you touched.
- The next cause's gain is under about 0.2 CPU s, or what's left is under *Found and not fixed*.

## Inputs

- `temper/docs/plan/limits-heavy/inputs/request.md`.
- `temper/docs/plan/limits-heavy/notes/common.md`: how to measure, the figures' form and the rules.
- `temper/docs/plan/limits-heavy/plan.md`, *Belonging*: why these two move and the six stay.
- `temper/docs/design/domain/testing.md`, 2.6, its bullet *The box's lifetime with its steps running*, which this cut wrote, and 2.8's table of Linux's own tests.
- `temper/docs/design/protocol/setup.md`, 13.1, *The box's lifetime*.
- `skein/docs/foundation/testing-strategy.md`, 2.9 and 8.1.

## Owns and may change

Owns the files in the header. `temper/crates/temper/tests/suite/box_lifetime.rs` is new.

Departures from the implementer's text, for this move's citations alone:

- `temper/docs/design/domain/testing.md`, 2.8's table: the row for `a_killed_box_takes_its_steps_with_it` names `crates/temper/tests/suite/box_lifetime.rs`. The next row, for `stop`'s check in `box_listeners.rs`, stays.
- `temper/docs/design/protocol/setup.md`, 13.1, *The box's lifetime*: its last sentence says that on the shipped binary, in `crates/temper/tests/suite/box_listeners/lifetime.rs`, a box started ignoring hangups runs on; and in the process tier, in `crates/temper/tests/suite/box_lifetime.rs`, a hangup stops a box, nothing left, its log's last line `Stopped`, and, on Linux, a box killed takes its steps with it.
- `temper/docs/design/domain/testing.status.md`: remove the **Not yet** entry for 2.6 that begins *the two stories of the box's lifetime with its steps running*.

Readers: `rg` over the repository, hidden files included, finds the two tests' names only in `box_listeners/lifetime.rs` and the two design lines above. `temper/docs/design/protocol/setup.status.md` cites `box_listeners.rs` for the bare form and for `the_first_link_opened_at_once_waits_and_signs_in`, which stay, so it needs no edit. No nextest filter names them.

May change, at the size the cause needs: `temper/crates/temper/tests/support/**` (`model-catalogue-2/process-fixtures` is here too), `temper/scripts/proof-live.sh`, and `temper/crates/temper/src/**`, `temper-box`, `temper-init` and `temper-service` for a cause a profile puts there. `proof_live.rs` and the other readers of `proof-live.sh` must keep passing if you change it.

Options, each decided:

- **A step of `main`'s first run costs the six most of what they read** (`init`, a key, parsing a template): fix it in the product, with its own focused test. A person's box pays it too.
- **A real-box test trips its patience under callgrind:** measure it by `time`, as the common note says.
- **`a_box_ignoring_hangups_runs_on` stays in `suite`:** it asks for no step, and is one path through `main`. Don't move it, and don't move any of the other five.
- **The two moved tests overlap `process.rs`'s `a_story_box_and_its_steps_die_when_the_test_process_is_killed`:** leave both as they are. 2.6's *a variant grows an existing story* isn't a way to make room here (`temper/docs/design/domain/testing.md`, 6.1). Say in your report how much of one the other holds, for `plan.md`.
- **`box_link`'s `cfg` in `process.rs`:** `#[cfg(target_os = "linux")] mod box_link;` goes, since the hangup story builds wherever it builds today. If that leaves something unused on Linux, a scoped `#[expect]` is fine; the macOS build can't be checked here, so say so.

## Steps

1. Run both tests by name in `suite` on your base, and note the result for the figures. Read `temper/crates/temper/tests/suite/box_listeners/lifetime.rs`.
2. The move:
   - `temper/crates/temper/tests/suite/box_lifetime.rs` takes `listening`, `exited` and the two tests, with a module comment citing `temper/docs/design/protocol/setup.md`, 13.1. It uses `crate::box_link` and `crate::support::{boxed, parent}`.
   - `process.rs` gains `#[path = "suite/box_lifetime.rs"] mod box_lifetime;` beside `box_kinds` and `startover`.
   - `command` in `box_listeners.rs` moves to `box_link.rs` as `pub fn command(root: &Path, action: &str) -> Command`, and `box_listeners.rs` and `box_lifetime.rs` both call it. `suite/main.rs` isn't touched.
   - `box_listeners/lifetime.rs` keeps `a_box_ignoring_hangups_runs_on` and what it needs.
3. Fix the two design lines and the status entry, as above.
4. Check: `heavy cargo nextest run -p temper --test process --no-run`, then the remaining `box_listeners` tests and `forge_land` in `suite`.
5. Profile, with `-p temper --test suite`: `a_later_box_refuses_a_taken_port_naming_the_port_and_document` (init and a refused box, with `--trace-children=yes`) and `a_fresh_box_prints_its_block_and_nothing_else`. Together the six read about 1.7 CPU s. A cause under 0.2 CPU s across them goes in the figures and stays.
6. Profile `forge_land::the_launcher_gives_each_landing_story_its_own_store` and one median test of the row. Starting points, to check and not findings: the launcher test runs `temper/scripts/proof-live.sh` three times against a stubbed `cargo`, and what does the script itself start each time? What do the other 43 share?
7. Fix the causes, largest first, and write the figures.
8. `python3 -I -B tools/citations/citations.py --base origin/main`, then the implementer's text's end of a run. Commit and push.

## Report

Where the two tests live now and what passed before the move, and that they were compiled and not run. How much `a_killed_box_takes_its_steps_with_it` and `a_story_box_and_its_steps_die_when_the_test_process_is_killed` overlap, by reading. The six's and `forge_land`'s before and after, and the causes found and not fixed. Your implementation and check minutes apart.
