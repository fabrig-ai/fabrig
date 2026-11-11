# Tier

Kind: concept

A tier is a slow check of a branch after changes land on it: each run covers every landing since the last, and its verdict clears them or blames the one whose commit broke a test.

## What it is

- **A tier** is a workflow too slow for a change's CI, such as a
  browser suite or an exhaustive profile. A repository names its tiers
  beside its gates, and each tier names the branch it follows: the
  repository's landing branch, its main, unless it names another, or
  `goal`, the goals' own branches. A landing on a branch no tier
  follows, or one only a tier of goal branches follows, is cleared as
  it lands.
- **A run** covers every landing since the tier's last judged run. One
  run per tier goes at a time, and the next starts at the newest tip
  once the last ends. temper dispatches it on a branch of its own,
  `temper/tier/NAME.N`, made at the commit it runs, and deletes the
  branch when the run ends.
- **A tier of goal branches,** `"branch": "goal"`, checks each goal's
  branch at its newest tip: at most one run per span per branch, one
  at a time across branches. It judges no landing. On red, it reruns
  failed tests alone. Failures that repeat go to the goal holding the
  branch, marked new or still failing; new failures wake it. Green
  says nothing.
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
- **A tip already judged:** if the files the tier reads match a tip
  already run, its receipt judges it without a run; documentation
  every tier ignores clears at once.

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
- **A red goes to its goal** as news: the tier, the run, the
  landing's commit, the failing tests blamed on it and the end of
  each failed job's log, waking the goal. The goal repairs it with a
  change naming the red landing (`repairs`, temper help change);
  when a later run passes the failures, the red landing is cleared,
  repaired by that change's landing, and its change's result ends
  `repaired by COMMIT.`
- **A runner's fault,** a lost run or a failed setup, runs the part
  again, up to the project's `tier_retries`. Past them, the tier's run
  task is held, `Tier NAME run N held after K runner faults, last:
  KIND`, until a person releases it, which runs it again.

## What temper guarantees

- **A change lives to cleared:** landed, then verifying, until every
  tier following its branch has cleared its landing, and its result
  comes then (temper help change).
- **Bounded:** past the project's verification bound, two hours unless
  set, with no verdict, the change ends landed and uncleared, saying
  so; a later verdict still counts on the tier.
- **Only temper's runs judge:** undispatched workflow runs are not read.

## Acting on it

- **The configuration:** in `CONFIG/temper.json`, a repository's tiers
  go at `application.landing.projects[].gates[].tiers`.
  The project's `gates` array holds repository entries; each
  repository's `gates` array holds workflow gates, beside its `tiers`.
  For project 1, forge 1, repository 7:

  ```json
  {
    "application": {
      "landing": {
        "projects": [{
          "project": 1,
          "by": "queue",
          "check_timeout_ms": 900000,
          "tier_retries": 2,
          "verification_bound_ms": 7200000,
          "gates": [{
            "forge": 1,
            "repository": 7,
            "gates": [],
            "tiers": [{
              "name": "browser",
              "workflow": "browser.yml",
              "ignored": ["docs"],
              "artifact": "report",
              "format": "junit"
            }]
          }]
        }]
      }
    }
  }
  ```

  Use project, forge and repository numbers from
  `project show PROJECT`, `forge list` and `forge repositories FORGE`
  through ctl (temper help ctl). Preserve the existing configuration:
  add the tier to the matching repository and project, creating missing
  entries as above. Keep other members,
  repositories, workflow gates and tiers, and the project's existing
  `by` (`queue` or `turn`). Set `tier_retries` and
  `verification_bound_ms` beside the project's `by`; the example shows
  their defaults. Stop and restart the deployment afterward with the
  same directories (temper help deployment); set `branch` to another
  branch's name to follow it instead, or to `goal` to follow the goals'
  own branches. Set `span_ms` to keep a tier's runs on one branch at
  least that many milliseconds apart; none when absent. Set optional
  `description` to what it checks: 1 to 120 UTF-8 bytes without control
  characters, shown as Checks on its card and by `see settings gates`.
  For goal-sized delivery checks, set optional `check_timeout_ms` beside
  `by`: 1,000 to 86,400,000 milliseconds, 900,000 in the example. It bounds
  each delivery check of the project's change producers, repairs and
  resolutions, including goal landings; absent, the agent's bound applies.
  Other runs keep the agent's bound. Each attempt reads it anew; it need
  not fit the reserve, and a check is cut at the run's end.
- **The report format:** set `format` to `junit` for JUnit reports or
  `lines` for one failing test id per line.
- **The workflow** is dispatch-only and takes the inputs
  `temper_commit` and `temper_key`, and `temper_tests`, the ids to run
  alone, one per line, when temper gives them: it runs only those then.
- **The box's runner:** workflows using `temper-host` execute on the
  host. JavaScript `uses:` actions such as `actions/checkout@v4` and
  `actions/upload-artifact@v3` need compatible Node.js installed there
  as `node`, on `application.machine.owned.runner.path` (default
  `/usr/local/bin:/usr/bin:/bin`). After changing that configuration,
  restart the box. Node.js only in an interactive shell is insufficient.
- **Runner capacity:** CI and tiers using `temper-host` share a runner.
  `application.machine.owned.runner.capacity` defaults to 1:
  one job at a time, delaying CI and landings. Use
  2 for one single-job tier plus CI, or 3 for two such tiers plus CI.
  Count jobs in multi-job workflows and other repositories too.
  Stop the real-backend box with Ctrl-C or SIGTERM; wait for it to exit.
  Edit only that setting in `CONFIG/temper.json`, then restart
  `temper box --config CONFIG --data DATA --cache CACHE` with the same
  directories. Prepared runners keep their registration and secrets.
- **ctl:** `change show TASK --until cleared` waits for `cleared` or
  ended landed and uncleared; `verifying` and `red` remain pending.
  Inspect the verdict with `change show TASK`; use `--until changed`
  for panel updates.
  `task release` releases held runs. `task show TASK`'s `asked:` names
  repository, full tested commit and owned branch. Runs, failed-test reruns
  and bisection probes retain commit, branch, forge run number and URL
  after branch deletion and restart. Red pairs commits with owned or
  suspected full IDs; inherited failures blame no landing. Omissions say
  `and N more failing tests; change show TASK`; the panel keeps IDs whole,
  overflow `many tests`. Past ctl's bound, use
  `task text TASK --field specification --from OFFSET`.

## Limits

For the configuration members that need a restart, read `temper help configuration`.

- A repository has at most two tiers in the small profile. A run names
  at most 16 failing tests, each id at most 256 bytes; past them its
  failures are many, blamed by landing alone.
- A tier follows one branch: following two takes two tiers.
- temper posts no verdict to the forge, and a red whose goal has
  ended, or of a landing temper did not make, reaches no one.
