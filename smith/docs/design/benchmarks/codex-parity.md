# Parity with Codex

smith is **on par with Codex** at a commit when one run of the `parity`
suite on that commit's built arm meets every criterion of 16.2. Parity is
non-inferiority (`README.md`, section 9): on these tasks, smith is not worse than
Codex by more than each criterion's margin. It ranks nothing, and it
holds for one pinned Codex at one working tier.

## 1. The run

- **What runs.** The `parity` suite (`catalogue.md`, 5): the fixtures `01-backoff`,
  `02-line-decoder` and `03-planner`, eight attempts per arm each, and
  the repository task `shell-cwd`, ten attempts per arm: 68 attempts,
  interleaved in one session (`README.md`, section 9), with no rerun. Every attempt
  counts.
- **Why ten on `shell-cwd`.** RT and RC rest on that task alone, and on
  medians of its attempts. A verdict should turn on smith's time, not on
  the test's power: a smith as fast as Codex, or a little faster,
  should pass most runs, and one worse than the margin almost none. More
  attempts narrow the interval without moving the margin, so the second
  holds at any number.
- **The arms.** smith's built arm of the commit, and Codex at its pinned
  version, each as it ships (`README.md`, 5.4, 6.1, 6.2), at the working tier
  (`README.md`, section 7). Delegation is each agent's default; what delegation is
  worth is the `delegation` experiment's (`catalogue.md`, 3), not parity's.
- **The basis.** Outcome, task wall time and common priced spend (`README.md`, 8.1),
  which both agents report on one basis, each response priced at its own
  prompt's tier. Tokens by field, cache reads, cached share, reasoning,
  responses, process wall, the time record, the agent's own peak
  resident memory and each fixture's own ratios are reported beside, and
  decide nothing.

## 2. The criteria

| # | Criterion | Statistic | Holds when |
|---|---|---|---|
| O | outcome | smith's passes over all 34 attempts per arm, pooled over the four tasks | at least the lower bound of Codex's exact 95% interval for its own pooled passes, times 34 (31 when Codex passes 34) |
| FT | the fixtures' time per solved task | each arm's total task wall over the three fixtures divided by its passes there; smith's over Codex's | the 95% interval's upper bound is below 1.20 |
| FC | the fixtures' cost per solved task | the same, with common priced spend | the upper bound is below 1.20 |
| RT | `shell-cwd`'s time | the ratio of the arms' median task walls | the upper bound is below 1.20 |
| RC | `shell-cwd`'s cost | the ratio of the arms' median common priced spend | the upper bound is below 1.50 |

- **The intervals** are percentile bootstraps of 10,000 resamples,
  seeded by the run's seed. FT and FC resample attempts within each task
  and arm, so that each draw keeps the tasks' sizes. A draw in which an
  arm has no pass counts the ratio as infinite. RT and RC take the
  bootstrap of `README.md`, section 9's comparisons, at ten attempts per arm.
- **A pass** is an attempt whose deciding checks all pass (`README.md`, 3.2): its end,
  its grader, its protected files, its tree's end.
- **Why per solved task.** Every attempt's time and spend count, a
  failure's included, so a fast failure never looks cheap: the time and
  cost of getting the fixtures done.

## 3. Ends and missing values

- **Unsolved:** a `Completed` attempt whose grader or deciding check
  fails, `Failed`, `Timeout` and `Budget` (`README.md`, 8.3). Their task wall and
  spend stay in the sums.
- **Incomplete:** a `Refused(setup)` or a `HarnessError` attempt; an
  unavailable task wall or common priced spend; a lower bound of either
  on smith's side, other than a spend that misses only retried failures
  (`README.md`, 8.2); Codex without a pass on the fixtures or on `shell-cwd`, since
  such a task tells the agents nothing. A lower bound on Codex's side is
  taken as observed, which can only raise smith's ratio, and is flagged.
- **smith's retried failures are taken as observed,** and flagged: a
  spend that misses only retried failures counts as what was reported,
  and the attempt's row keeps the lower bound that names them.
  - The provider's failures are not the agent's. Both arms meet them,
    interleaved on one ChatGPT account, each on its own login.
  - Codex's tokens are its own running total, read as observed. The
    harness sees none of Codex's provider attempts, so whatever a Codex
    request that the provider failed and Codex retried added to that
    total is read as observed too.
  - Read the same way, smith's figure is what its provider reported to
    it, as Codex's is. Holding smith's failed requests to more would
    hold smith to an account that the harness cannot apply to Codex.
- **Not on par:** smith without a pass on the fixtures or on `shell-cwd`.

## 4. The verdict

- **`summarise`** writes a `parity` section into a `parity` run's
  summary: each criterion's statistic, interval, margin and verdict;
  the reference arm, chosen as the arm whose agent is Codex, never by the
  arms' names; the seed; and the commit whose presets priced the spend.
- **`smith-bench parity SUMMARY --commit SHA`** recomputes that section
  offline from the summary's per-attempt values (`README.md`, 5.4). It refuses a
  section that disagrees with them, and a summary whose smith arm was not
  built from `SHA`. It prints one line per criterion, and exits 2 when
  the verdict is incomplete (3), whatever the criteria; otherwise 1
  when a criterion fails or 3 finds smith not on par, and 0 at
  parity. A run of `suite parity` exits with the same code.
- **A parity series is done** when this command exits 0 over a committed
  summary of a run on its final `main`.
- **What moves it.** A new Codex pin, a new working tier, a new task
  version or a new result version starts a new comparison: parity is
  shown again, never carried across.

## 5. A verdict short of parity

- **A summary is committed whatever its verdict, 0 or 1.** A run
  measures its commit as it is. A verdict of 1 says where smith stands,
  and the runs that follow are read beside it. A run whose verdict is 2
  measured nothing whole: its summary is not committed, and its attempts
  are spent.
- **A smith arm is run once to a verdict of 0 or 1.** No run repeats an
  arm to pass (1, "with no rerun"). Each run of one smith is one more
  chance for a smith worse than a margin to pass it; one run per change
  keeps that chance to one per change.
- **A failed verdict is answered by a change, never by a run.** A later
  run is of a later commit whose smith arm differs, by a change whose
  commit body names:
  - the criteria it addresses;
  - their cause, read from the failed run's attempts: the measures
    reported beside (1), the counts and the traces, against Codex's in
    that run, or smith's in the last run where the criterion held;
  - the change's own proof (`README.md`, 4.2).

  A change that names no cause earns no run. A change that serves the
  suite's tasks and no other work, such as instructions that name a
  task's files or commands, is not a cause's answer.
- **The newest run is the verdict.** Runs are never pooled, and none is
  chosen among them: 4's command reads the newest committed summary,
  and the earlier ones stay committed beside it.
- **A parity series bounds its runs.** It says beforehand how many
  closing runs it holds. When the last it holds is short of parity, the
  series ends with its target unmet: it names the criteria that failed,
  its last summary is the evidence, and their answer is a later series',
  a change as above.
- **Baselines are cut from a series' last `parity` run,** whatever its
  verdict (`README.md`, section 9): they alarm on the drift of the smith that ships,
  on par or not.

## 6. A change on the way to a verdict

A *parity series*, changes whose aim is a parity verdict, runs a
baseline first: the `parity` run that measures where it starts. After
it, each step that changes what the model is sent shows its effect
before it lands: on every criterion, whatever criterion it claims to
move, by a run of its own when that run can tell its expected effect
from none; otherwise on the counts it targets, and the series' closing
run measures it with the others.

- **What counts.** A change to a prompt smith sends, or to when it sends
  one:
  - the task role, and any text smith writes into the system text or a
    message: the summary instruction and its preamble included;
  - a tool's description or schema, or what a tool answers when a call
    asks for nothing: a read's window, a command's kept output, a job's
    wait;
  - the window's check, which decides when a window is compacted
    (../domain/session.md, 8.1).

  A change that alters no prompt and no request's timing, such as the
  harness, the event stream or a refactor, is not one.
- **Measured alone, or with the others.** A run of two arms of smith
  resolves only an effect wider than its interval. The series states
  beforehand each change run's resolution, from the harness's planning
  commands (a simulation at the run's attempts on the spread of the last
  committed baseline, `benchmarks/baselines/`), and each change's
  expected effect, from its cause (5):
  - **a change its run can tell from none** runs `parity-change`, as
    below;
  - **any other** proves itself on the counts it targets, such as its
    task's responses (1), and lands. The closing run measures such
    changes together, and a criterion it misses has 5's answer.
- **Its proof is one run of `parity-change`** (`catalogue.md`, 5): the `parity`
  suite's tasks, five attempts per arm each, with two smith arms, the
  change and its parent, interleaved, with no rerun. It takes the place
  of the `compare` run of 4.2.
- **Its report** is what `summarise` writes for that run, and goes in
  the commit body; nothing else is asked for:
  - **each criterion of 2,** computed with the change in smith's
    place and its parent in Codex's: the change's ratio to its parent,
    with its interval, and, for O, each arm's passes. The reference arm
    is the one built from the change's parent, chosen by its commit;
  - **its projection:** the baseline's statistic times the change's
    ratio, beside the baseline's interval. `summarise` takes the
    baseline's committed summary as a named input. A product over the
    changes landed before it would count each of their codes twice, as a
    change and as the next one's parent
    (`../../development/benchmarks/wall-clock-plan.md`);
  - **beside them,** deciding nothing: each arm's responses an attempt,
    the time record's parts (requests, compaction, tools) and fresh
    tokens, per task.
- **A move the wrong way stops it.** The step stops for its series
  owner's decision when a criterion moves the wrong way past the
  baseline's interval:
  - **FT, FC, RT or RC:** its projection lies above the baseline's
    upper bound;
  - **O:** the change's passes fall below the lower bound of the exact
    95% interval of its parent's passes, over as many attempts.

  This holds whatever the step's own comparison concludes, "none
  detected" included. A report that is incomplete, a criterion
  unavailable or an arm with no pass on a task, stops the step the same
  way. The step lands only on the owner's answer: kept, with the
  cause the answer names for the move; reworked; or dropped.

  The cause may be noise rather than the change. The answer then shows,
  from the run's attempts, that the move lies in work the change does
  not alter, such as the reads after a compaction that more of one
  arm's attempts reached. Where an earlier `parity-change` run measured
  the parent's code, the answer gives that run's figures beside. An
  interval that holds 1 is no such showing: at five attempts, a
  comparison cannot show that a change does no harm. The projection
  keeps the ratio as measured.
- **Why the baseline's interval.** A comparison that detects nothing
  at five attempts per arm can hide a move larger than the baseline's
  own uncertainty; the projection sets the change's move on the
  baseline, where the series can see it. A series' position within the
  baseline's interval is within what it already measures as noise, and
  goes on. What the landed changes add up to, the moves no change run
  resolves included, is the closing run's to measure.
- **The baseline** here is the series' first `parity` run, its
  committed summary, never `baselines/`'s files, which are cut from a
  series' last run (5).
- **One run per commit.** A step reworked after a stop runs
  `parity-change` again, on its new commit. A `parity-change` run is a
  step's evidence for landing, never a verdict: the series' `parity` runs
  judge it (5), and its runs are never pooled with them.
