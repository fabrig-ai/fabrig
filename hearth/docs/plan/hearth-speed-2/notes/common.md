# hearth-speed-2: rules every task names

The request is `hearth/docs/plan/hearth-speed-2/inputs/request.md`. The rule is `skein/docs/foundation/testing-strategy.md`, 8, "A slow test is understood, never hidden", and 8.1. The budget is `hearth/docs/design/domain/testing.md`, 9.1. The profile and the figures are in `hearth/docs/plan/hearth-speed-2/figures.md`, once the `profile` task has written them.

## What doesn't move

- **No build setting changes:** no `opt-level` and no `[profile.dev.package.*]` line, for any crate, and no `RUSTFLAGS`, linker or nextest setting. `.config/nextest.toml` and `tools/tiers/budgets.toml` don't change.
- **Every assertion and every case stays.** No test leaves the focused suite. No scenario shrinks, no seed goes, no count falls. A world's seed still replays. What a test proves doesn't change, and neither does what hearth does for a user: the same answers, bytes and records, the same refusals at the edges. A golden, a recorded digest and a fixture's expected value don't change: a change to one is a finding, not a new figure.
- **A memory ceiling doesn't move,** unless a measurement shows the peak moved, and the commit says why. If a repair fails a ceiling test or a test that reads a peak, that's a finding first: read it, then take the repair out or say what the peak did.
- **skein, smith, jig and temper change in no file.** If the profile shows a shared kit is the cost, that's a question first: put it under `## Left` in your report with its measured share, and leave the code. It is not your task's to change.
- **A failing test is a finding first.** Read the log and find what changed. Don't weaken a test to get past it.

## The aim

The figure that counts is the queue's, `[gate.hearth.default]` of `tools/tiers/budgets.toml`: 30.5 CPU s over 2,189 tests at the plan's start, 11-06 14:49Z, and the aim is 27. No task can run the queue, and a builder reads higher. So a task's aim is a ratio: **the method's figure at most 85% of the same machine's figure at the base commit.** A figure from one machine is never compared with another's without saying so. **Stop once the aim is met:** what's left unsaved isn't a reason to go on.

## The base commit

`39b490f13846b81fe78c5834c7852ffd28a25155`, the main this plan started from. Every "before" is measured at it, whatever the trunk has merged since: in your slot, `git checkout --detach` it, measure, and return to your branch with `git checkout`. No file changes on the way, and the driver below stays untracked across the checkout.

## The method

`tools/tiers/design.md`, 7, 7.3 and 10.2 with a caller of its own, as the queue's record reads it, outside the queue: user plus system CPU of each test and its waited and reaped descendants, summed; compilation, listing and nextest excluded.

- **A throwaway driver, pre-approved:** copy this to `.measure.py` in your slot's root. It stays untracked and is deleted before the push. `python3 -I -B .measure.py default target/hs2/base-1` measures every hearth package (34 on 11-06); `python3 -I -B .measure.py default target/hs2/x -p hearth-tree-world --test suite` measures one binary. A fresh DIR for each run.

  ```python
  import json, os, runpy, subprocess, sys
  from pathlib import Path

  profile, directory, selection = sys.argv[1], Path(sys.argv[2]).resolve(), sys.argv[3:]
  env = {**os.environ, "NEXTEST_DOUBLE_SPAWN": "0", "TEMPER_BUNDLE": "none"}
  wrapper = Path("target/hs2-cpu-wrapper").resolve()
  if not wrapper.exists():
      subprocess.check_call(["heavy", "cc", "-std=c11", "-O2", "-static", "-Wall", "-Wextra", "-Werror",
                             "tools/tiers/cpu-wrapper.c", "-o", str(wrapper)])
  if not selection:
      metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"]))
      tree = os.getcwd() + "/hearth/"
      selection = [arg for package in metadata["packages"] if package["manifest_path"].startswith(tree)
                   for arg in ("-p", package["name"])]
  command = ["heavy", "cargo", "nextest", "run", "--profile", profile, *selection]
  subprocess.check_call([*command, "--no-run"], env=env)
  collector = runpy.run_path("tools/tiers/cpu.py")
  extra = collector["prepare"](directory, profile, wrapper)
  code = collector["execute"]([*command, *extra], env, directory, profile, {})
  result = collector["summary"](directory)
  families = {"hearth-builder-world::suite": "builder", "hearth-tree-world::suite": "tree",
              "hearth-builder-world::real": "real-git", "hearth-render-world::suite": "render"}
  rows = {}
  for test in result["tests"]:
      row = rows.setdefault(families.get(test["binary_id"], "rest"), [0, 0.0, 0.0, 0.0])
      for index, value in enumerate((1, test["cpu_s"], test["user_s"], test["system_s"])):
          row[index] += value
  print(json.dumps({"suite": result["suite"], "families": rows, "failures": result["failures"]}, indent=1))
  sys.exit(code)
  ```

- **`result["tests"]`** in the collector's directory (`summary(directory)`) gives CPU, user and system by test: the by-test and by-binary tables come from it.
- **Twice on each side,** back to back on one machine. Give both figures, and compare the lower of each side.
- **The families:** `hearth-builder-world::suite` (the builder suite), `hearth-tree-world::suite` (the tree suite), `hearth-builder-world::real` (the real-git binary), `hearth-render-world::suite` (the render suite), and all the rest.
- **Instructions are the noise-free check.** The same code reads a few percent apart from run to run and more across machines. Callgrind's instruction count (`Ir`) of the test a repair changes doesn't vary with load: quote it beside the CPU, before and after (`hearth/docs/plan/hearth-speed-2/tasks/profile.md`, "How to profile").
- **A figure you can't get is `unknown`,** with why. No figure is a guess. The machine comes from `hostname` through heavy, the time from `date -u`.

## The end of every task

- `cargo fmt`, and `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` for what you changed.
- From `hearth/`, `python3 -I -B scripts/old_forms.py`, with `--write` only to lower a count for what your task removed.
- From the root, `python3 -I -B tools/citations/citations.py --all` stays at 0.
- `heavy cargo nextest run -p fabrig-boundary --test outward` passes. Files outside `docs/plan/` name no machine, tool or session of whoever runs this development: `hostname`'s answer stays in `figures.md`, a commit and a report. Status files say what's true now, with no plan's name and no history.
- **Each commit says what it saved, measured:** its family's binary figure before and after, twice each side, with the machine, and `Ir` of the test it changed. A commit that saves nothing measurable doesn't land: take it out, and say so in the report.
- The method's figure for the whole suite, both sides, in your report.
