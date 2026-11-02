# Tier

Kind: concept

A tier is a slow check of a branch after changes land on it: each run covers every landing since the last, and its verdict clears them or blames the one whose commit broke a test.

## What it is

- **A tier** is a workflow too slow for a change's CI, such as a
  browser suite or an exhaustive profile. A repository names its tiers
  beside its gates, and each tier names the branch it follows: the
  repository's landing branch, its main, unless it names another. A
  landing on a branch no tier follows, a goal's own branch for one, is
  cleared as it lands.
- **A run** covers every landing since the tier's last judged run. One
  run per tier goes at a time, and the next starts at the newest tip
  once the last ends. temper dispatches it on a branch of its own,
  `temper/tier/NAME.N`, made at the commit it runs, and deletes the
  branch when the run ends.
- **The baseline:** A newly configured tier first runs at its followed
  branch's current tip as a baseline, without any landing. Baseline
  failures are blamed on no landing and inherited by the next run.
  Subsequent ranges start after the baseline tip. Landings arriving
  during the baseline wait for the next run to verify them.
- **Its report:** the run uploads an artifact, named in the tier's
  settings, holding a JUnit file or one test id per line, and temper
  reads the failing tests' ids from it. A failed run that names no
  test, or whose artifact is absent or unreadable, is a runner's fault,
  never a red.
- **A tip already judged:** where the files the tier reads at a new tip
  are those of a tip it already ran, that run's receipt judges it with
  no new run, so a landing of documentation every tier ignores clears
  at once.

## How it changes

- **Green** clears every landing in the run's range.
- **Red** runs the failed tests once more, alone, at the same tip.
  Those that pass are a flake: counted on the tier, blamed on no one.
  Those that failed in the tier's previous run too are inherited,
  blamed on no new landing. A new failure belongs to the one landing in
  the range, or to the first landing whose commit fails it, found by
  running the failing tests alone at each landing's commit, oldest
  first; a landing that changes nothing the tier reads is skipped.
- **A landing's verdict is its own:** red only by the failures blamed
  on it, until a later run passes them, which clears it.
- **A runner's fault,** a lost run or a failed setup, runs the part
  again, up to the project's `tier_retries`. Past them, the tier's run
  task is held, `Tier NAME run N held after K runner faults, last:
  KIND`, until a person releases it, which runs it again.

## What temper guarantees

- **A change lives to cleared:** landed, then verifying, until every
  tier following its branch has cleared its landing, and its result
  comes then (temper help change). A green tip never hides a red
  landing behind it.
- **Bounded:** past the project's verification bound, two hours unless
  set, with no verdict, the change ends landed and uncleared, saying
  so; a later verdict still counts on the tier.
- **Only temper's runs judge:** a run of the workflow that temper did
  not dispatch is not read.

## Acting on it

- **The configuration:** a repository's tiers go in the `landing`
  member, beside its gates, and the project's `tier_retries` and
  `verification_bound_ms` beside its `by`; then restart. Main's tiers,
  the slow suites, follow main:

  ```json
  {"forge": 1, "repository": 7, "gates": [],
   "tiers": [{"name": "browser", "workflow": "browser.yml",
              "ignored": ["docs/"], "artifact": "report", "format": "junit"}]}
  ```

- **The workflow** is dispatch-only and takes the inputs
  `temper_commit` and `temper_key`, and `temper_tests`, the ids to run
  alone, one per line, when temper gives them: it runs only those then.
  It uploads its report as the named artifact.
- **ctl:** `change show TASK --until cleared` waits for a landed
  change's verdict, its step `verifying`, `red` or `cleared` naming the
  tiers and their runs; `task show` reads a tier's run task, and `task
  release` releases a held one.

## Limits

- A repository has at most two tiers in the small profile. A run names
  at most 16 failing tests, each id at most 256 bytes; past them its
  failures are many, blamed by landing alone.
- A tier follows one branch: following two takes two tiers.
- temper neither tells a goal of a red after landing nor posts a
  verdict to the forge.
