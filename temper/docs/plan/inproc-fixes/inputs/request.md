# inproc-fixes: the one-process mode keeps to its design before the gate uses it

`inproc-runner` landed the one-process mode (`tools/tiers/one_process.py`, `tools/tiers/design.md`, 11). The whole workspace's focused and fuzzy suites pass through it. `tools/tiers/design.status.md` lists where the code departs from 11, most of them with no reason recorded. The landing gate is about to run focused and fuzzy through this mode, so its cancellation and selection paths have to hold first.

## What's wanted

**Every one-process departure in `design.status.md` closes: the code keeps to 11, or 11 changes where the code's choice is the better one, saying why. Its entry goes.** In particular:

1. **Cancellation finishes (11.7).** A global timeout or a wall hard cancellation with jobs still waiting in the lanes' queue drops them as unobserved and completes a starved run, rather than looping until the lanes are killed and `run` refuses. A forced session kill that leaves a started process without its final event gives a starved result with that process's tests unobserved and its CPU unknown, not a `ValueError`: say in 11.7 how its CPU counts, without inventing rusage.
2. **A cancelled test is cancelled (11.6, 11.7, 11.8).** Once a process is marked cancelled, its running test is cancelled and later tests unobserved, whatever libtest reports after. A cancelled test whose process exits with a code rather than a signal still gets the `test abort` JUnit shape 11.8 gives.
3. **Selection agrees with the run (11.1, 11.2).** The listing gets the caller's configuration path, so a private configuration (the gate passes one with its own wall cap) selects what the run uses. A caller's filter in any of nextest's spellings, `-E EXPR`, `-EEXPR` and `--filterset`, counts as the caller's, and the default filter is compiled only when it's used. An empty selection with `--no-tests=warn` prints nextest's warning.
4. **Small ones (11.4, 11.8).** A fresh process after a crash takes its place in the queue by its tests' priority and estimate. `command-output.txt` carries the alone diagnostic's output and a line for a failure that also fails alone.
5. **The named pipe stays** (11.4): libtest opens its logfile close-on-exec, which keeps the writer out of tests' descendants. 11.4 and 11.8 say so, and the entry goes.

Each code fix comes with a row in `tools/tiers/test_one_process.py` that fails before it.

## Limits

- Nextest's path and 10.2's contract stay as they are. No budget, timeout, filter or tier changes.
- Run `tools/tiers/test_one_process.py`, `tools/tiers/test_cpu.py`, and the focused and fuzzy suites through `tools/tiers/one-process` once at the end, green. Read and compare no CPU figure.
- `walk = false`. Effort medium. One or two tasks.

## Sources

Read by section: `tools/tiers/design.md`, 10.2 and 11.

- `tools/tiers/design.status.md`
- `tools/tiers/one_process.py`
- `tools/tiers/one-process`

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories. No walk, render check or throwaway driver.
- **Decide, don't ask.** A small edit just outside a task's file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
