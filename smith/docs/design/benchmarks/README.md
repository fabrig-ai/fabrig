# Benchmarks

Provisional, 2026-10-09, revised 2026-10-19. How smith shows that a change has the effect it
claims and keeps it, and how smith compares with other agents. Tests are
../testing.md's; the events smith is observed through are
../protocol/events.md's; its commands, settings and exit codes are
../shell/README.md's. What is still open is listed in section 14; parity with
Codex, the comparison's verdict, is `codex-parity.md`.
Agent adapters and model tiers are in `agents.md`.
The result schema and statistics are in `results.md`.
Attempt isolation is in `isolation.md`.

## 1. In one page

- **Tests specify; benchmarks show.** A behaviour is tested in the lowest
  tier that shows it. A benchmark runs the product against real
  providers to show that a change has its effect in the world, and keeps
  it. Benchmarks are never the merge gate.
- **One notion of measurement.** A task is a seed, a prompt and checks,
  run by an agent configuration some number of times. **Probes** force
  one behaviour and pass or fail; **fixtures** are small coding tasks,
  for trends and comparisons; **repository tasks** are real repositories
  at a frozen commit, run rarely.
- **Cheap by default.** Tasks name a model tier, not a model. Probes run
  on the small tier at low effort, cheapest first; comparisons and
  repository tasks run by choice.
- **A fix names its probe,** and its commit body records the probe
  failing before it and passing after it. A change to the runtime,
  limits, providers or host records a smoke run there.
- **Agents are adapters.** `Smith`, `Codex` and `ClaudeCode` are a closed
  set, each run as it ships, at a pinned version, on a login of the
  harness's own, signed in once with the agent's own flow, never a
  person's own tool directory and never a copy of a refresh token, under
  a hard deadline on its whole process tree. A pin changes only what an
  attempt must own: its paths, its model and effort, its account, and
  the keys that keep it unattended and isolated. smith is observed only
  through its event stream.
- **Unknown is never zero.** A measurement is observed, a lower bound, or
  unavailable with its reason. Tokens count once per response.
- **Statistics fit the question.** A probe reruns once on failure. A
  comparison interleaves its arms in one session and reports a median
  ratio with a bootstrap interval against the task's minimum detectable
  effect. Baselines are drift alarms. Raw results stay outside the tree.
- **Parity is a verdict, not an absence of effects.** smith is on par
  with Codex when one run of the `parity` suite shows, for outcome, time
  and cost, that smith is not worse than Codex by more than a stated
  margin. One offline command reads the committed summary and exits 0 at
  parity (`codex-parity.md`). A verdict short of it is committed too, and is
  answered by a change that names its cause, never by another run of the
  same smith (`codex-parity.md`, 5).
- **On the way to a verdict, a change shows the criteria.** After a
  parity series' baseline, a change to what the model is sent runs
  against its parent on the verdict's tasks when that run can tell its
  expected effect from none, reports each criterion's ratio, and stops
  for its series owner's decision when one moves the wrong way past the
  baseline's interval, whatever its own comparison concludes. Any other
  lands on the counts it targets, and the closing run measures it with
  the others (`codex-parity.md`, 6).
- **Spend is priced as billed.** Each response's tokens are priced at
  the list rates its own prompt reaches, never at the rates an agent's
  declared window could reach (`results.md`, 1.1).

## 2. What benchmarks are for

- **To show a change is effective,** and that it stays so while
  providers, models and other agents change under smith: a probe sees a
  provider's wire, routing and caching as they are.
- **To compare agents** on the same tasks, prompts and graders, so that a
  difference in outcome, time, tokens or cost is the agent's, not the
  harness's.
- **To find what the tests missed:** passive checks on every attempt hold
  smith's evidence to its contracts (`catalogue.md`, section 2).

They are not tests (section 11) and not the merge gate: no gate check
runs an agent. Nor are they a ranking: a comparison states an effect
with its interval, or that none was detected.

## 3. What is measured

### 3.1 Tasks and kinds

A task (section 5.2) holds a prompt, byte-identical for every agent, a
seed, a model tier, budgets, checks and the design sections it guards.
An attempt is one run of a task by one agent configuration in a fresh
workspace, ending in one result (`results.md`, section 1). The kinds are a closed set:

| Kind | Judged by | Default design | Cost per attempt (estimate) |
|---|---|---|---|
| probe | every check passes | one attempt, rerun once on failure | seconds to a minute |
| fixture | the grader's pass rate, and metrics against another arm | interleaved arms, at least five attempts each | minutes |
| repository | the grader's pass, and large effects only | three attempts per agent | tens of minutes |

- **A probe forces one behaviour.** Its prompt leaves the model no choice
  ("with one `write` call", "delegate with `max_turns` 3"); its seed is
  tiny and its outcome exact. One that cannot be calibrated on the small
  tier at low effort names a higher effort and says why. Its outcome
  checks hold for any agent; its event checks read smith's stream, and
  hold smith to its contracts or count what the prompt asks of the
  model (3.2 says whose a failure is).
- **A probe is calibrated** before it gates anything: before it joins
  `smoke`, and before its failure in `probes` counts as a regression
  (`results.md`, section 2). `probes` lists every probe and variant, calibrated or not
  (`catalogue.md`, 5); its task file's `[calibration]` block, one per task and one per
  variant, says which (5.2). Calibrated means ten passes within at
  most twelve attempts on the binary it guards, the other attempts failed
  only by the model, never by smith (a model deviation, 3.2; `results.md`, section 1.3
  's ends), and a failure on the binary before the fix or,
  for a behaviour already in place, on a control arm. Where neither can
  run, as for a behaviour older than the oldest binary the harness runs
  (`agents.md`, 1.2) that no setting turns off, the block says why, and the
  calibration stands on its passes. A probe that cannot
  fail guards nothing. A calibration stands while the variant's prompt,
  seed and checks keep the digests its block records; a new task version
  that changes none of them for a variant keeps that variant's block.
- **A fixture** has a hidden grader, run after the agent has exited.
  Every healthy agent passes it, so it tells agents apart by time, tokens
  and cost. **A repository task** is frozen with every dependency
  vendored, so that it builds offline for good and its grader may rely on
  that commit's interfaces; it shows long conversations, delegation,
  compaction and budgets at scale. Every grader fails on the seed and
  passes on the task's reference solution, and the reference passes the
  task's outcome and protected checks.
- **A large effect** is the repository kind's only claim across agents:
  for a metric both observe, the two arms' observed ranges over their
  attempts do not overlap. Overlapping ranges claim none, and a missing or
  lower-bound value keeps its reason and claims no separation. Nothing is
  inferred about significance from three attempts. The summary computes
  it. Parity runs the repository task interleaved, ten attempts per arm,
  with intervals (`codex-parity.md`).

### 3.2 Checks and arms

- **Outcome checks** are agent-neutral, read once the agent's tree has
  ended: a file's content, digest, absence or pattern; protected files;
  writes outside the workspace and the attempt's private directories;
  the end (`results.md`, section 1.3) and, for smith, its exit code and the words on
  its standard error (../shell/commands.md); the grader's commands.
- **Event checks** read smith's event stream (../protocol/events.md, section
  3): a closed set of named, typed checks with parameters, such as "one
  `write` call with `input_bytes` of at least N, `written`", each with an
  offline test.
- **Passive checks** run on every attempt (`catalogue.md`, section 2); a task that
  expects a failure names those it waives. A failed check carries its
  evidence: the records or files it read.
- **What decides.** A probe names its deciding checks. A graded task (a
  fixture or a repository task) decides on its end, its grader, its
  protected files and its tree's end. A passive check that cannot be
  observed for an agent, such as reads of the benchmark tree or writes
  outside the workspace before contained trees, is reported beside and
  audited, and decides nothing.
- **Whose failure.** A failed deciding check is the model's when it
  judges what the prompt asks of the model: the answer, the files, a
  report of what the model was told (`budget-report`), and how many
  calls of a tool it makes where the prompt prescribes the number
  (`call-count`). It is smith's when it judges smith's contracts: its
  records, a call's verdict, a request's tool choice or content, a
  bound, the window. An attempt that completed, and whose failed
  deciding checks are all the model's, is a model deviation; any other
  failure is smith's. A deciding check of smith's contracts left
  unavailable, other than "not exercised" (below), leaves the attempt
  unjudged: neither smith's failure nor a deviation (section 1).
- **Not exercised.** Some of smith's contracts apply only once their
  condition arises: a call past a bound, a call that lasts over 60 s, a
  window at its compaction threshold. A probe's prompt asks the model to
  bring the condition about, and a model can stop short of it: it
  honours the bound, answers quickly, or finishes before its reading
  fills the window. A check of such a contract, in an attempt where its
  condition never arose, is "not exercised": neither a pass nor a fail.
  What kept the condition away was the model's, so in a calibration the
  attempt counts like a model deviation (3.1): never a pass, and within
  the attempts the model may fail. `catalogue.md`, section 1 says, for each probe,
  what the condition is and how the stream shows it. A check that
  cannot tell whether its condition arose, because a record it reads is
  missing, is unavailable instead, and leaves the attempt unjudged.
- **A count is the model's only where smith's side is checked.** A call
  is made again, or skipped, as readily when smith lost what the model
  had as when the model did: a result missing from a compaction's
  request, or a summary missing from the next window. So a task whose
  windows compact also decides on `compaction-carried`: each
  compaction's request holds a result for each call its window owes it,
  and the window after it opens with that compaction's summary
  (../domain/session-context.md, sections 1.1 and 1.2). With that check passing, a
  summary that misstates or drops what it was given is the model's, and
  so is the count that follows from it.
- **A request smith retried** (../domain/session.md, section 4) is judged
  through its retry. A request that failed with no blocks made no calls,
  and its prompt is the one its retry sends (../protocol/events.md, section
  3.11), so the failed request's missing usage leaves no check
  unavailable. Its tokens are a retried failure's, or none when it failed
  unsent (`results.md`, 1.2). A failure smith
  did not recover from is the attempt's end (`results.md`, 1.3).
- **An arm** is a binary, a configuration override or an agent. Two arms
  make an experiment: a **fix** (before and after), a **control** (the
  mechanism switched off), a **decision** left to measurement (`catalogue.md`, section 3
  ), or a **comparison** (another agent, or main against a
  candidate). An arm built from a commit is built in the harness's own
  clone, outside every developer checkout, and named by the commit and
  the binary's SHA-256. smith arms use Cargo's default dev profile,
  without `--release`. One build runs at a time: it holds an exclusive
  lock over the clone and its target, and a second waits.

## 4. Tiers

### 4.1 When each runs

| Tier | Contents | When | Agents | Cost (estimate) |
|---|---|---|---|---|
| offline | parsers on recorded outputs, manifests, statistics | in the focused suite | none | under a second |
| smoke | a few probes and the passive checks, one attempt each | each change to the runtime, limits, providers or host, and each fix | smith on `openai`, small tier; `anthropic` too for a change to its path | a few minutes, under 100k tokens |
| probes | every probe and variant, both providers; outcome halves on other agents | weekly | smith; Codex; later Claude Code | about an hour, a few million tokens |
| compare | fixtures, interleaved | on demand | smith and the agents compared, working tier | one to two hours |
| repository | repository tasks | at milestones | smith and Codex, working tier | several hours |

Only the offline tier is a test (section 13). The others are runs of
`smith-bench`, by choice. Each tier is a suite file (section 5.3).

The costs are estimates for planning, not gates. The offline tier's
tests are held to the workspace's suite budgets, as every focused test is.
A test sits with what it tests: a parser's beside the parsers, an
adapter's among that adapter's tests.

### 4.2 A fix and its probe

- **Proof comes before the merge.** A change merges, and counts as done,
  only with its proof in its commit body. Evidence is never left "pending".
- **Every improvement names the probe that guards it** in its commit
  body, except one whose effect does not depend on a model, which names
  its counting test instead. A probe comes with the runs the harness prints ready for
  it:
  - **after,** on the change, passing;
  - **before,** on its parent, failing, wherever the parent can run the
    probe live;
  - **Codex,** on the same task, where the probe compares agents.
- **A fix with no probe adds one,** calibrated, in the same change. A
  defect no probe can force, such as a race or a rare provider stall, is
  guarded by a passive check or an event check on the closest probe, and
  the commit says which.
- **An improvement shown on fixtures,** such as a change to smith's role
  (../shell/settings.md, 2.6), names the `compare` suite in place of a probe: one run
  with two arms, its parent and the change, interleaved (`results.md`, section 2),
  whose comparison goes in its commit body. It adds no probe. In a
  parity series (`codex-parity.md`, 6), after its baseline, a change to what
  the model is sent names the `parity-change` suite in its place when
  that run can tell the change's expected effect from none, with `codex-parity.md`, 6's
  report; any other names the counts it targets (`codex-parity.md`, 6).
- **Every smith landing that changes code runs `smoke-tools` live,**
  once, on its branch rebased on `origin/main` just before merging. A red
  smoke stops the line until it is green again.
- **A parent that cannot run live.** When a parent fails the probe for a
  reason unrelated to the change (for example, its limits refuse what the
  provider sends today), the before is the closest evidence that exists:
  - the change's deterministic tests, which fail on the parent;
  - recorded measurements;
  - a same-code run with a setting that turns the feature off, where one
    exists.

  The commit says which. No switch is added only for this.
- **The probe stays** in its suites with its `guards`, so a later failure
  names the design sections at stake.
- **Choosing what to run.** The harness lists the tasks guarding the
  sections a change cites, cheapest first by their committed medians or
  estimates, within the suite's wall and token budgets, and reports what
  did not fit. A fix's probes run at least once before it and once after;
  repetition beyond that is the statistics' (`results.md`, section 2).

## 5. Layout and formats

### 5.1 The crate

```
benchmarks/
  Cargo.toml               smith-bench, a member of the workspace
  clippy.toml              the harness's own lints
  README.md                how to run it, and sign each agent in once
  src/                     the harness
  tests/                   the offline tier and its recorded outputs
  tasks/probes/<id>/       task.toml, seed/
  tasks/fixtures/<id>/     task.toml, seed/, grader/, reference/
  tasks/repository/<id>/   task.toml, seed/ (a manifest), grader/, reference/
  suites/<name>.toml       what a run runs
  agents/<agent>/          the pinned configurations
  agents/models.toml       the model tiers
  baselines/               per suite, agent configuration and tier
  summaries/               one per committed run
```

- **Ordinary Rust** under the test-harness rules (skein/docs/foundation/programming-model.md,
  section 10.2): its orders, samples and bootstrap draw from a seed it
  records, and it measures time on one monotonic clock.
- **Its own `clippy.toml`.** The root file's shell bans include
  `std::process` (../shell/README.md, section 3.2), and the harness starts
  processes. Like `tests/clippy.toml`, its file keeps the bans that make
  a run reproduce from its seed and read one clock.
- **Modules by concern,** one per agent (`agents.md`, section 1). smith's events are read through `smith-events`, so a
  changed record fails the build or the offline tier, never a run.
- **Commands** run a suite or a task with its arms, calibrate a probe,
  record a fix's before and after, list the tasks guarding a section,
  summarise, write a baseline, sign an agent's home in, and check the
  tree offline. Their spelling is the README's.

### 5.2 Tasks

```toml
id = "large-write"
version = 1
kind = "probe"
guards = ["../domain/tools.md, section 3", "../protocol/limits.md, section 6"]
tier = "small"
deadline_seconds = 120
prompt = """
Create numbers.txt containing the integers 1 to 6000, one per line, \
with a single write call. Then finish.
"""

[[outcome]]
check = "file-digest"
path = "numbers.txt"
sha256 = "..."

[[event]]
check = "tool-call"
tool = "write"
input_bytes_at_least = 16384
verdict = "written"
```

| Field | What it holds |
|---|---|
| `id`, `version`, `kind`, `title` | `version` changes with the prompt, seed, grader or checks; results of different versions are never compared |
| `guards`, `behaviours` | design sections, cited as the design cites them; the design's names for what is guarded |
| `tier`, `effort` | the model tier; an effort only where calibration needs one |
| `prompt` | delivered whole, through a file or standard input |
| `deadline_seconds`, `estimate` | the hard deadline on the attempt's process tree; seconds and tokens, until measured |
| `[budget]` | turns, time and spend, given to each agent as far as it takes them (`agents.md`, section 1.1) |
| `[smith]`, `[environment]` | settings overrides, passed as `-c` settings, never through the seed; variables the agent's commands see |
| `setup`, `protected` | commands that warm caches outside the timer, with the agent's own build profile; files that must not change. A task that builds warms its build in `setup`. A task never protects a path its prompt asks the agent to change: that the seed's tests still pass is its grader's, which holds a copy of them |
| `[[outcome]]`, `[[event]]`, `[[grade]]`, `waives`, `deciding` | the checks; the passive checks the task expects to fail; the checks that decide its verdict (3.2) |
| `[calibration]` | a probe's calibration, per task and per `[[variant]]`, written by `calibrate` and committed with the summary it names: the arm's commit and SHA-256, the date, that summary, the attempts run, the passing attempts, the model deviations, the control and the attempts that failed before, and the digests of the variant's prompt, seed and checks. A probe without one gates nothing (3.1) |
| `repetitions` | where the kind's default does not fit, as for a probe judged on a median |
| `[[variant]]` | a named change to the prompt, overrides or checks, reported as a task of its own (`large-write/oversized`) |
| `[[person]]` | for an interactive task: the scripted person's lines, each sent at the start or when an event the task names is observed (for example `follow-up` sends its request when `finish` is called), and an end of input |

- **`seed/`** is the workspace as the agent first sees it, committed in a
  fresh repository the agent's configuration trusts, its digest in the
  manifest. It holds no `.smith/` settings unless the task is about
  them, never `benchmarks/`, and no file of any grader or reference. A
  repository seed is the bundle's manifest (source commit, vendored pins,
  digest); the bundle lives in the harness's data directory. Every check
  that reads the seed, the protected files' included, reads what the
  workspace was copied from: the bundle, for a repository task.
- **`grader/`** holds hidden tests and their data. Each `[[grade]]`
  command runs after the agent's tree has ended, in a copy of the final
  workspace with the grader beside it, under a deadline, in its own
  process group, with bounded logs; its exit is its verdict.
  **`reference/`** is a solution, applied to the seed to validate it.

### 5.3 Suites

```toml
name = "smoke"
tier = "small"
design = "single"              # or "interleaved"
repetitions = 1
rerun_failed = 1
max_wall_seconds = 600
max_tokens = 100000
tasks = ["probes/first-request", "probes/budget/wrap-up"]

[[agents]]
agent = "smith"
provider = "openai"
config = "standard"
```

A suite names its tasks (`<kind>/<id>[/<variant>]`, or by kind and
behaviour), its agent configurations, a tier overriding the tasks' own,
its design, repetitions and budgets, and its arms where it compares. A
suite may give one task repetitions of its own, as `parity` does (`catalogue.md`, 5).
A task listed under `when_asked` runs only when a run names it
(`--include`). A new schedule is a new file. An attempt whose every
provider request was refused with a 4xx status before any output charges
no tokens to the suite's `max_tokens`; its usage stays unavailable. An
attempt whose total tokens miss only retried failures (`results.md`, 1.2) charges the
total's lower bound, what the provider reported, and the suite goes on.
Any other unknown in an attempt's total stops the suite as incomplete,
since what it would charge is not known.

### 5.4 Agent configurations and results

- **`agents/<agent>/`** holds pinned configurations, each for a stated
  version of the agent, changed only by commit. "Vanilla" is the agent's
  defaults plus this file. A suite's agent configuration is an agent, a
  provider, a pinned configuration and a tier, and its digest is in
  every result's identity.
- **A comparison's pin changes nothing the agent ships** but what the
  attempt must own: its paths (home, temporary directory, caches, trace),
  its model and effort, its account, one of the harness's, and the keys
  that keep it unattended and isolated (`isolation.md`, section 1). It restates no
  default. An experiment's arm names each key it changes, and only those
  (`catalogue.md`, 3). A probe that needs another value, a small window or an empty
  brief, names it in its own task's `[smith]` overrides (5.2).
- **Raw results stay outside the tree,** under
  `$XDG_STATE_HOME/smith-bench/runs/<run>/<attempt>/`: the result, the
  agent's output and records, standard error's tail, the grader's logs,
  and the diff against the seed's commit, the agent's commits included.
  Workspaces are deleted once archived; anything raw is scanned for
  credentials before it leaves the machine.
- **Committed:** `summaries/`, one per run of the probes, compare,
  repository and parity suites (per task and arm, counts by end, medians,
  ranges, comparisons, drift notices and audits, and each attempt's
  identity, end, judgment, task wall, common priced spend, token fields,
  responses and the agent's own peak resident memory, so that a verdict
  recomputes from the file), and `baselines/`
  (`results.md`, section 2). Smoke runs, and a fix's before and after, go in commit
  bodies; a calibration goes in its task file (5.2).

## 11. How it relates

- **To smith's tests.** Benchmarks find and show; tests specify and gate.
  A benchmark does not replay and its outcome varies, so what it finds is
  reproduced in the lowest tier that shows it (skein/docs/foundation/testing-strategy.md,
  section 9) and fixed there with that test; the probe stays as evidence
  of the effect. Benchmarks never stand in for a world's story.
- **To the live tier** (../testing-tiers.md, section 3). The live tier checks
  once, by outcome, that each wire, issuer refresh and push is right.
  Benchmarks repeat, measure and compare. Both read smith through
  `smith-events`, take logins of their own, signed in once, under one
  credential guard, and default to
  the small tier.
- **To skein's test kit.** The harness is not a world: no schedule,
  replay, counting allocator or referee plumbing of its own. What it
  needs that is generic to processes, such as a tree's settlement and
  resource accounting, lands in skein first, smith-bench its first user.
- **To the product.** It runs smith as a person would (../shell/commands.md
  ); a gap it meets there is the product's to close, never the
  harness's to work around.

## 12. Adding to it

| To add | Where |
|---|---|
| an agent | a variant of `Agent` and its module; its pinned configurations; its recorded outputs and their offline tests |
| a provider | a variant; its token convention; an entry per tier in `agents/models.toml` |
| a task | a directory under its kind, calibrated if a probe, validated if graded |
| a task kind | a variant with its judgement and defaults, and a directory under `tasks/` |
| a check | a variant of the outcome or event checks, with its offline test |
| a metric | a `Measure` in the result, with its scope, and a new result version |
| a tier or a schedule | an entry in `agents/models.toml`; a suite file |

## 13. The world

The offline tier runs in the focused suite, with no agent and no network:

- **Parsers on recorded outputs,** per agent and version: smith's are the
  event codec's goldens (../protocol/events.md, section 8); Codex's are JSON
  lines and rollouts with root-only usage, an unlinked child and repeated
  usage, and rollout trees with an archived rollout, an unrelated file,
  a link out of the session directories and two files naming one thread
  (`agents.md`, 1.3); Claude Code's are streams with repeated message ids, sub-agent
  messages and helper models. Unknown records are skipped, a known record
  in a new shape is a harness error, usage counts once per response, and
  unavailable is never zero.
- **Classification:** one recorded end of each class per agent, and each
  of smith's exit codes.
- **Manifests:** every task, suite and configuration parses; seed digests
  match; a `[calibration]` block whose digests differ from its variant's
  prompt, seed or checks gates nothing (3.1); prompts are whole; every
  check is known; every guarded section
  exists in the design; no seed holds `benchmarks/` or a grader's or
  reference's file; suites name existing tasks and agents.
- **Statistics:** the bootstrap, the interleaving and the audit sample
  reproduce from their seeds; the rerun rule; the minimum detectable
  effect; the large effect's disjoint ranges.
- **Parity:** the verdict recomputes from a committed summary; the
  stratified bootstrap keeps each task's size in every draw; an arm
  without a pass; a ratio's upper bound just below and just above its
  margin; the reference chosen by agent; each end of 16.3.
- **Pricing:** recorded attempts whose prompts stay below a tier's size
  and pass it, each response priced at its own prompt's tier, whatever
  the declared window; a response without usage leaving the spend
  unavailable, a retried failure a lower bound, and a response failed
  unsent, retried or followed by a compaction, priced at nothing, the
  spend observed (`results.md`, 1.1, 1.2).
- **A change's report:** a recorded `parity-change` summary gives each
  criterion's ratio of the change to its parent, the parent chosen by
  its commit, and its projection on a recorded baseline; a projection
  just below and just above the baseline's upper bound, and passes just
  within and past O's bound, stop it or not (`codex-parity.md`, 6).
- **An attempt against a replaying agent:** a small program that prints a
  recorded stream, leaves a grandchild behind, or never ends, observed,
  ended at the deadline and classified. A file it leaves in the shared
  temporary directory its request names is a write outside (10); that a
  planned request names `/tmp` is read from the request, without listing
  `/tmp`.
- **A tree's containment:** `isolation.md`, section 1's choice as a table, each
  platform with each `force_walk`, run on Linux; and a replay's counts
  name the platform's method, the walk on Linux and the group elsewhere.
- **The harness over a test tree.** The harness reads and writes files
  through one face: the native filesystem in a run, the same calls in the
  same order; in the offline tier, a tree laid on skein's fake machine,
  with its modes, owners and links. Over it run a Codex rollout tree's
  discovery, a working directory's and mounts' admission, a pin's and a
  binary's digests, a credential's staging and removal, a bundle's intake,
  a scratch root's refusal, the catalogue's walk, a calibration's and a
  summary's writes, a parity change's named inputs and a written file's
  settled bytes. A test reads back what was written and asserts what was
  never read, a login or a credential source above all.
- **Receipts from supplied completions:** the supervisor's deadlines,
  capture bounds and overflow, an answer before the tree settles, a
  partial record and a refused product check are judged over completions,
  times and exits the test supplies.
- **What stays a process,** in `tests/suite`: real git and archives, login
  shells, a write outside, an answer observed before the tree settles, a
  lingering descendant's teardown, one pipe's overflow, a terminal peer's
  hangup and a final record drained at its end of file, this checkout's
  own history choosing its face, and one path each through the shipped
  `probe` and `suite` commands.
- **Fuzzy:** parsers fed truncated and corrupted lines never panic and
  never count a response twice.

Each runs in milliseconds, within the focused and fuzzy budgets.

**Linux's own tests, by name.** A test that only Linux can run is built
for Linux alone, `cfg(target_os = "linux")` on the test with its reason
beside it, and listed here, as skein's are
(skein/docs/design/runtime/shell.md, 8); none is dropped silently. A
helper only they use takes the gate too, so each test binary builds on
macOS with nothing unused. A fixture naming a command or path macOS
lacks (`head -n -1`, `/bin/true`) is ported instead. Every other test
runs on both.

| Test | Its reason |
|---|---|
| `benchmarks/tests/suite/runner.rs`: `an_answer_with_a_lingering_grandchild_keeps_its_forced_teardown` | only the walk names a descendant alive at the leader's exit (`isolation.md`, section 1) |
| `benchmarks/tests/suite/runner.rs`: `a_file_left_in_tmp_by_an_attempt_is_a_write_outside`; `benchmarks/tests/suite/pipeline.rs`: `grading_and_archives_keep_agent_commits_and_untracked_files_outside_the_timer` and `login_shells_keep_machine_path_in_setup_agent_and_grading` | an attempt copies its caches by GNU `cp --reflink` and reads the shared temporary directory's owner through `/proc/self` (section 14) |
| `benchmarks/tests/suite/probe_command.rs`: `a_replay_arm_runs_through_the_probe_command_and_prints_one_line` | the probe's plan reads `/proc`, its attempts are the row above's, and its arm needs the source's history |
| `benchmarks/tests/suite/smith_adapter.rs`: `the_current_source_arm_selects_the_chat_face` | it reads the source's history, which macOS's run lacks: its checkout holds the tested commit alone (`temper/docs/design/testing/macos.md`, 1) |
| `benchmarks/tests/suite/smith_adapter.rs`: `a_terminal_peer_can_close_its_stdio_before_the_leader_exits_without_hangup` | its peer reads the hangup's disposition from `/proc/self/status` |

## 14. Open questions

- **Codex's later versions:** whether a version after 0.160.0 still
  writes each thread's rollout under `sessions/` or `archived_sessions/`
  with its own `session_meta` as the first line, still links each child
  thread to its parent in that opening record, written before the child's
  first response, and still trusts the attempt root by
  `projects.<root>.trust_level`. A new pin checks each, on a recorded
  attempt with a child and a grandchild, before its first comparison: the
  collector finds every thread its stream and its state database name,
  each linked (`agents.md`, 1.3).
- **Claude Code:** the settings that allow edits and commands headless;
  whether `CLAUDE_CONFIG_DIR` keeps every user file out;
  `claude-haiku-5-5` at low effort on a subscription.
- **Claude Code's attempts at once on one login:** whether attempts that
  share a configuration directory interfere. Default: one at a time
  (`isolation.md`, section 1).
- **Tree accounting:** cgroup v2 delegated to the user on the development
  machine, as containment also needs (../testing.md, section 8).
- **The shared credential guard:** which crate the live tier and
  smith-bench both take it from.
- **Repository bundles:** where a bundle is kept durably, so that another
  machine can run the task.
- **A bundle's frozen source:** a bundle's `Cargo.toml` and `Cargo.lock`
  keep the git source they were frozen with, which its
  `.cargo/config.toml` replaces with `vendor/`, so a grader over the same
  crates must name that source too. `shell-cwd`'s first `[[grade]]`
  copies those lines into its grader's manifest at grade time, and no
  tracked file names the source.
  Whether a bundle frozen anew takes another. Default: it stays: a new
  bundle changes the seed, so the task's version (5.2), and parity is
  shown again (`codex-parity.md`, 4).
- **Concurrency:** attempts run serially; parallel ones would need
  disjoint CPUs.
- **The harness on macOS:** whether `run` and `probe` run there. An
  attempt copies its caches by GNU `cp --reflink`, reads the shared
  temporary directory's owner and the probe's machine through `/proc`,
  and under the process group `tree-ended` cannot see a descendant that
  left the group (`isolation.md`, section 1). Default: they stay Linux's, and macOS
  runs smith-bench's tests alone (`README.md`, section 13).
- **Task kinds beyond code,** such as chat, research or review, with
  graders of their own; and **audits:** how many attempts a run samples,
  and who audits them.
- **Recording smoke runs:** whether a commit hook checks that a runtime
  change's body carries one.
- **A failed response's usage:** skein's failed call carries none, so
  usage a provider sends with its failure is told as null (`results.md`, 1.2, a
  retried failure). Whether a failure should carry it is skein's
  `llm/README.md`'s question.
- **A failure's kind in a summary:** a harness failure has a kind, and only
  its words reach a result: `HarnessError` and `Refused` hold text (`results.md`, 1.3).
  Default: so; a summary that counts by kind changes the result's format.
