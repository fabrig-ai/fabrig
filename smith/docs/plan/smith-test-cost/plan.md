# smith-test-cost: the plan

smith's focused suite is back under its 30 CPU seconds by removing what its own tests and fixtures waste (`inputs/request.md`). This file is updated with each push; parts 2 and 3 are still being read.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| scope-input | `scope-input` | | the directory the bench's scope gate lists becomes the request's `shared_temporary`: `/tmp` for a real run, a test's own in the tests |

## Critical path

`scope-input`, alone so far.

## Decisions

- **The directory lives on `run::Request`, not `Machine` or an environment variable.** A request is what `smith-bench attempt` and `probe` carry across their child processes, and an archived `prepared.json` then says what the attempt watched. `Machine` holds tool paths, and an environment variable would let a machine's environment move a real run's gate unseen.
- **The field has a serde default of `/tmp`.** A prepared document written before it reads as a real run, and the status file records that stored forms are kept as they are (6, `codex_home`).
- **Only `probe` takes `--shared-temporary`.** `probe_command::a_replay_arm_runs_through_the_probe_command_and_prints_one_line` plans and dispatches inside one child, so it needs a way to name its directory on the command line. `suite`, `calibrate` and `fix` refuse the flag, so a suite or a calibration can't be moved, and a run that uses it says so in its evidence and its `prepared.json`.
- **The test fixture's default names a directory that doesn't exist.** A test that forgets gets an unavailable listing, never the machine's `/tmp`. `fault::tests::a_refused_attempt_keeps_the_seed_step` reaches an attempt (the request's "may be one", now read) and needs no edit.
- **`runner::a_file_left_in_tmp_by_an_attempt_is_a_write_outside` stops re-running itself.** Nothing `run::attempt` reaches reads the environment that re-run set, read at the cut; the spec keeps it if the implementer finds otherwise.

## Overlaps with drafts beside this one

None for `scope-input`: `model-catalogue-2/smith-limits` touched `smith/benchmarks/**` and has merged, and the `limits-*` drafts in flight are temper's.
