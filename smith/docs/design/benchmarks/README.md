# Benchmarks

Provisional, 2026-10-09, revised 2026-10-19. How smith shows that a change has the effect it
claims and keeps it, and how smith compares with other agents. Tests are
../testing.md's; the events smith is observed through are
../protocol/events.md's; its commands, settings and exit codes are
../shell/README.md's. What is still open is listed in section 14; parity with
Codex, the comparison's verdict, is `codex-parity.md`.

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
  declared window could reach (8.1).

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
workspace, ending in one result (section 8). The kinds are a closed set:

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
  (section 9). `probes` lists every probe and variant, calibrated or not
  (`catalogue.md`, 5); its task file's `[calibration]` block, one per task and one per
  variant, says which (5.2). Calibrated means ten passes within at
  most twelve attempts on the binary it guards, the other attempts failed
  only by the model, never by smith (a model deviation, 3.2; section
  8.3's ends), and a failure on the binary before the fix or,
  for a behaviour already in place, on a control arm. Where neither can
  run, as for a behaviour older than the oldest binary the harness runs
  (6.2) that no setting turns off, the block says why, and the
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
  the end (section 8.3) and, for smith, its exit code and the words on
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
  (../domain/session.md, sections 8.1 and 8.2). With that check passing, a
  summary that misstates or drops what it was given is the model's, and
  so is the count that follows from it.
- **A request smith retried** (../domain/session.md, section 4) is judged
  through its retry. A request that failed with no blocks made no calls,
  and its prompt is the one its retry sends (../protocol/events.md, section
  3.11), so the failed request's missing usage leaves no check
  unavailable. Its tokens are a retried failure's, or none when it failed
  unsent (8.2). A failure smith
  did not recover from is the attempt's end (8.3).
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
  with two arms, its parent and the change, interleaved (`README.md`, section 9),
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
  repetition beyond that is the statistics' (section 9).

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
- **Modules by concern,** one per agent (section 6). smith's events are read through `smith-events`, so a
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
| `[budget]` | turns, time and spend, given to each agent as far as it takes them (section 6.1) |
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
attempt whose total tokens miss only retried failures (8.2) charges the
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
  that keep it unattended and isolated (section 10). It restates no
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
  (section 9). Smoke runs, and a fix's before and after, go in commit
  bodies; a calibration goes in its task file (5.2).

## 6. Agents as adapters

`enum Agent { Smith, Codex, ClaudeCode }` is matched exhaustively, one
module per variant, in five steps. **Prepare** the attempt's private
directories and the agent's pinned configuration, writing nothing in the
user's tool directories.
**Command:** an exact environment from an allow-list, nothing inherited,
and the prompt on standard input. **Observe** the output as it arrives,
as neutral observations: a response and its usage, a tool call, a child,
the answer, which ends task wall time and starts teardown. **Collect**,
once the tree has ended, what the agent persisted, as metrics with their
scope and convention. **Classify** one end (section 8.3).

| Step | Smith | Codex | Claude Code |
|---|---|---|---|
| prepare | `XDG_CONFIG_HOME` holds the pinned settings as the user's layer: the account's file and the trace, capturing `calls` so the scope gate sees each call's input (`everything` where a task needs prompts), and nothing else of a comparison's; `XDG_STATE_HOME` the attempt's state; `credentials` the harness's credentials directory, and the account one of its own (section 10). Before an arm's first attempt, `smith check --json` with the attempt's own flags, those of `exec` for a headless arm and the interactive `smith`'s, its trace among them, for a Chat arm, records the effective limits, the teardown bound, schema versions and credentials of the run it will start; a nonzero exit refuses the arm as setup. The attempt's events are its standard output's, under `--json`: the harness gives `exec` no trace file, so the run has one event destination | a `CODEX_HOME` of the attempt's own, holding only an access-only `auth.json` the harness writes from the harness's own Codex login (section 10), so none of the user's instructions, skills, plugins, memories or configuration reach it and its rollouts are the attempt's; only the keys an attempt must own (5.4) pinned by `-c` overrides: no MCP servers, the attempt root trusted, workspace edits allowed, no approvals, the model and effort, and the attempt's private caches among its sandbox's writable roots, so that it can write them; no other key, so that the rest is Codex's default; its background service disabled where it can be, or ended with the tree | `CLAUDE_CONFIG_DIR` the harness's own configuration directory, signed in once with `claude auth login` (section 10); the attempt's own settings, instructions and empty MCP configuration given by flags (`--settings`, `--strict-mcp-config`, `--setting-sources`), so that none of the login's directory's reach it, and its session found by its id; edits and commands allowed without asking |
| command | `smith exec --json -C <workspace> -m <endpoint>/<model> --effort <effort> -`, overrides as `-c KEY=VALUE` (../shell/commands.md, section 2); interactive, `smith --trace <file>` at a pseudo-terminal | `codex exec --json -m <model> -c model_reasoning_effort=<effort> -C <workspace> -o <final> -` | `claude -p --output-format stream-json --verbose --model <model> --effort <effort> --max-budget-usd <ceiling> --settings <pinned> --mcp-config <empty>`; `--input-format stream-json` for later messages |
| observe | the event stream, through `smith-events`; standard error is never parsed | JSON lines: thread, turns, items, and each turn's usage, of the root thread only, cached input within input | the opening record, assistant messages with usage, sub-agent messages with their parent's call, the result with durations, turns, cost and per-model usage, in the pinned version's recorded shapes |
| collect | nothing beyond the stream | the rollouts of the thread named, of the children its stream's `spawn_agent` calls name, and of each thread linked to one of those as a child (codex-cli 0.160.0 names its parent in the child's opening record), each thread's file the one under the attempt's `CODEX_HOME/sessions/` or `archived_sessions/` whose opening record names it, found in the rollouts alone and never in Codex's state database (6.3): per-response usage, children and their spans, and each command's span against its call's; where a child is not linked to its parent, child scopes are unavailable and totals lower bounds | nothing; transcripts are archived |
| classify | the exit code (../shell/commands.md, section 3), held against `run.completed` | the exit, the last turn's completion or failure, the deadline | the result's subtype and error flag, the exit, the deadline |

- **One runner.** Each attempt runs on skein's world harness, end to end
  (skein's examples.md, section 6): the agent starts as it ships; a
  scripted person types where a probe is interactive; the attempt's
  referee reads the output as it arrives and keeps the hard deadline.
  The live tier runs smith the same way.
- **An adapter fails loudly.** A known record in an unknown shape ends the
  attempt as a harness error naming it. Unknown record types are skipped
  (../protocol/events.md, section 6).

### 6.1 What each agent sees

- **Instructions.** A seed's are in `AGENTS.md`, and every agent gets the
  same text through the file it reads, whole: smith reads it into its
  system text by its default guide convention (../domain/run.md, 3.3), Codex
  into its first user message, and Claude Code's `../../../CLAUDE.md` is made at
  preparation and left out of the diff. smith runs with its shipped role
  (../shell/settings.md, 2.6): its pinned settings add nothing and remove nothing. No
  home or directory above the attempt's root holds
  instructions: each agent's home is the attempt's (`README.md`, section 10), since
  an agent may read its home's instructions whatever its command line
  says (codex-cli 0.160.0 reads `CODEX_HOME/AGENTS.md` unconditionally).
- **Models.** A comparison's smith declares each model as its presets do
  (../shell/settings.md, 2.1): its window, output and deadlines. A smaller window or
  output is a probe's, named in its task (5.2).
- **The environment.** `HOME`, `TMPDIR` and the build caches are the
  attempt's; `PATH` and the toolchain's home are the machine's,
  read-only; then the locale and the agent's home. A task's variables
  reach smith's commands through its settings, which own their
  environment whole, and the others' through their own process.
  A login shell sees the same `PATH`: the attempt's home holds a
  `.profile` that sets it to the machine's. Agents that run commands in
  a login shell would otherwise lose it, since the system profile
  resets `PATH` (codex-cli 0.160.0 runs each command as `bash -lc`, and
  Debian's `/etc/profile` drops `~/.cargo/bin`).
- **Budgets.** smith takes turns, time and spend; Claude Code a spend
  ceiling; Codex none. In a comparison every agent's own limits sit above
  the task's hard deadline, the bound they share: a fixture or a
  repository task names no `[budget]`, so smith runs on its run defaults
  (../shell/settings.md, 2.4), and `check` refuses such a task that names one, or
  whose `deadline_seconds` is not below the defaults' `budget.seconds`.
  Each result
  records the effective limits, and a budget end is labelled as one.
- **Caches.** Each agent can write the attempt's build caches: a sandbox's
  writable roots include them. A task that builds warms them in `setup`,
  outside the timer.

### 6.2 smith as it ships

- **One face.** The Smith adapter drives smith only as it ships: `smith
  exec --json` headless, or `smith --trace <file>` at a pseudo-terminal
  for an interactive task, with its settings as the product's TOML
  layers (../shell/settings.md, section 1) and a task's overrides as `-c` settings.
  A binary that predates them is not run, and no adapter face, settings
  template or reader of an earlier binary is kept.
- **What a trace lacks is unavailable,** never estimated. A result names
  the face, and results of different faces are never compared.
- **Baselines** taken on an earlier face are retaken or dropped (section
  9). Results from before this harness are never a baseline.

### 6.3 Codex's threads

The Codex adapter's collect step finds the attempt's threads and their
files in its `CODEX_HOME`'s rollouts alone. Every rollout there is the
attempt's (section 10), so no file need be asked for by name, and the
harness links no SQLite and runs no `sqlite3`.

- **The files.** Every regular file under `CODEX_HOME/sessions/` and
  `CODEX_HOME/archived_sessions/`, at any depth and whatever its name,
  taken under the canonical `CODEX_HOME`; a missing directory holds none.
  The walk meets entries in name order and follows no symbolic link: one
  whose canonical target lies inside either directory is skipped, its
  target met on its own; any other, out of them or dangling, refuses the
  collection, the fault naming the link's path under `CODEX_HOME` and
  nothing it points at. An entry that is neither a directory, a regular
  file nor a link is skipped unopened.
- **A file's thread.** A file's opening is its first line, read up to its
  newline within the opening bound. It names a thread when it is a JSON
  `session_meta` record whose `payload.id` is a nonempty string. Its
  parent is `payload.source.subagent.thread_spawn.parent_thread_id`, else
  `payload.parent_thread_id`, when that is a nonempty string, and none
  otherwise. Any other opening (not JSON, another record, no id, or cut at
  the bound) marks an unrelated file, skipped and read no further. Two
  files naming one thread refuse the collection.
- **The threads.** Known first: the root the stream names and the
  children its `spawn_agent` calls name (their `receiver_thread_ids`).
  Then, until none is added, each thread whose file names a known thread
  as its parent. A known thread's file is the one naming it. A known
  child with no file, or whose file names no known parent, is unlinked:
  its scope unavailable and the totals lower bounds (8.2). The root's own
  parent links nothing.
- **Bounds.** Discovery meets at most 8,192 entries under the two
  directories, directories and files alike, and reads at most 1 MiB of
  each file's opening (an opening stops at its newline); past the first
  bound the collection is refused, and an opening that meets the second
  is cut. The known threads stay within 4,096 and their files, read whole,
  within 64 MiB together; past either, the collection is refused.
- **A known thread's file** is read whole, its canonical path checked
  inside the two directories again, and decoded: its first record is its
  owner's metadata, a later one an ancestor's, and malformed, repeated
  or conflicting metadata refuses the collection, naming the thread and
  never a credential's bytes. Its bytes are kept for the attempt's
  archive, the files in thread-id order and the children too.
- **Why not the database.** codex-cli 0.160.0's `state_5.sqlite` holds
  each thread's children (`thread_spawn_edges`) and its file
  (`threads.rollout_path`). Its rollouts hold both: each file's opening
  record names its thread, and a child's names its parent and is written
  before the child's first response
  (`smith/benchmarks/tests/recorded/codex/live-child-rollout.jsonl`). A
  child the database alone knows would
  have neither a rollout nor a response, so no case this design needs
  reads it.

## 7. Models as tiers

| Tier | `openai` | `anthropic` | Used by |
|---|---|---|---|
| `small` | `gpt-6-luna`, low effort | `claude-haiku-5-5`, low effort | probes, smoke, the live tier |
| `working` | the user's working model and effort | the same | fixtures, repository tasks |

- **`agents/models.toml`** holds this mapping, pinned: the working tier
  changes by commit when the user's working model does, so that a
  comparison never mixes models. A new tier is an entry; a new model for
  a tier starts new baselines.
- **A tier's model is one smith's presets declare** (../shell/settings.md, 2.1), so
  that smith runs it as it ships; a model the presets lack gets its row
  there first.
- **Providers are a closed set,** named as smith's presets (../shell/settings.md,
  section 2.2). smith runs on both, Codex on `openai`'s models, Claude
  Code on `anthropic`'s. Each result records the model and effort asked
  for and those the agent reports; a mismatch is flagged.
- **smith's accounts.** smith's own loop runs on `openai` with a sign-in
  of the harness's, and on `anthropic` with a key. smith on a Claude
  subscription runs Claude Code's loop under smith's adapter
  (../protocol/programs.md): its results are an arm of their own,
  `smith-claude-code`, never pooled with smith's own loop's.

## 8. The result schema

### 8.1 One result per attempt

A JSON document. Like every format it is pre-release: one version, any
change a reader would notice is a new one, and a reader refuses others.
Committed summaries and baselines of an older version are regenerated or
dropped, never read across versions.

| Part | What it holds |
|---|---|
| identity | run, attempt and arm; the task's id, version, seed and prompt digests; the agent, its version, its binary's SHA-256 and, for smith, its face; the configuration's digest; the provider; model and effort, requested and resolved; the harness's commit; the host and CPUs used; the start, in wall time |
| timing | on the harness's one clock: process wall, to the tree's end; task wall, to the answer observed; teardown, from the answer to the tree's end; grading wall. The agent's own times beside them: smith's `first_byte_ms`, `largest_gap_ms` and longest completion, and its `t_ms`, from its own start. The time record: the seconds of the main conversation (Codex's root thread) in model requests, with their first-byte and largest-gap sums; in each tool, by name; in compaction; waiting on children; and the rest; and the seconds of commands that ran outside any call |
| tokens | per scope (section 8.2) |
| counts | responses, provider attempts and retries; completions; tool calls by name, not comparable across agents; tool failures by verdict; children; conversations and compactions; messages by terminal |
| delegation | for each child: its effective share, how it ended (an answer, its budget, failed, closed) and its turns and seconds; the time the parent was blocked on children |
| resources | CPU time and peak memory, each with its scope: the tree's accounting, the agent's process alone, or unavailable. smith's own peak resident size is its `session.ended`'s `peak_rss_bytes` (../protocol/events.md, 3.2) |
| spend | the agent's own figure, such as smith's `spent` or Claude Code's cost, with its basis: notional, priced or reported; and the common priced spend, every agent's tokens priced on one basis, as its provider bills them: each response's tokens at the list rates of the tier its own prompt reaches, from smith's presets' price list (../shell/settings.md, 1.2 and 2.1) at the harness's commit, which the summary names. The agent's declared window plays no part. The adapter reads each response's usage from the agent's records: smith's `response.completed`, Codex's usage per model response. Without a response's usage, its prompt's tier is unknown, and the common priced spend is unavailable, or a lower bound under 8.2's rule for a retried failure. A response that failed unsent reached no provider, and costs nothing (8.2). The harness keeps no catalog of its own |
| outcome | the end; the exit, a code or a signal; whether the harness forced it (no, terminated, killed); each check's verdict and evidence; protected files changed; writes outside the workspace |
| health | smith's loss per sink; headroom against each limit, as the fraction of its bound used; the passive checks |
| artifacts | each raw file's path and digest |

Every measurement is a `Measure`: observed, a lower bound, or unavailable
with its reason, never written as zero and never filled from another
scope.

### 8.2 Tokens

One normalised record per scope: **fresh** (input not read from a cache,
cache writes included), **cache read**, **cache write**, **output**
(reasoning included) and **reasoning**, each where reported. Each
convention is a tag on the record, with a fixed mapping:

| Convention | Fresh is | Not reported apart |
|---|---|---|
| Codex | input less cached input; cache writes (`cache_write_input_tokens`) are within input, reasoning (`reasoning_output_tokens`) within output | nothing, as codex-cli 0.160.0 reports them; a version that omits a field has it unavailable, never zero |
| Claude Code | input plus cache writes | thinking, within output |
| smith's events (../protocol/events.md, section 3.10) | `input_tokens` plus `cache_write_tokens` | whatever is null |

- **Once per response.** A response's usage counts once however many
  records repeat it: smith's by its `response`, the others' by message
  id. A retried attempt that was billed is a response of its own.
- **A retried failure** is a response that the provider failed before
  any block and smith retried, with none of its usage reported
  (../protocol/events.md, section 3.5). Its `response.completed` has
  `outcome` `failed`, `blocks` 0, a failure whose `evidence` is
  `response`, and every usage count null. A later
  `response.started` of its conversation and `turn` has a higher
  `attempt`.
  - **Its usage is unknown, never zero.** A provider that answered with
    a failure may have read the prompt first. skein's failed call
    carries no usage, whatever the provider sent with it (skein's
    `llm/README.md`, section 2: a completion carries the token usage).
  - **Its scope's totals and the attempt's are lower bounds,** and each
    names it as a retried failure, apart from any other gap. They miss
    the failed request alone. Its prompt is its retry's (../protocol/events.md,
    section 3.11), and the retry's usage counts that prompt.
  - **The common priced spend** (8.1) of an attempt whose totals miss
    only retried failures prices what was reported. It is a lower bound
    naming them. Any other gap in the tokens it prices leaves it
    unavailable.
  - **A failure whose `evidence` is `maybe_sent` is never one.** The
    provider never answered it: a deadline of smith's cut it, or the
    connection dropped, and the provider may have carried the request
    to its end. What it cost is open.
- **A response that failed unsent costs nothing.** A `response.completed`
  whose failure's `evidence` is `unsent` reached no provider: smith
  refused the request before any byte of it was sent, as the guard does
  past the line (../protocol/llm.md, section 6), or the connection failed
  before it. Its usage is zero, observed, though its counts are null,
  and the common priced spend prices it at nothing, whether smith
  retried it or compacted after it. It is no retried failure, and
  leaves no gap.
- **Scopes:** the root conversation; each child, under its parent;
  compactions; helper models; the total. A total is observed only when
  every scope is; otherwise it is a lower bound naming what is missing,
  and unavailable when the root is.

### 8.3 Ends

| End | Meaning | smith's exit (../shell/commands.md, section 3) |
|---|---|---|
| `Completed` | answered and exited by itself | 0 |
| `Failed(reason)` | ended without an answer: the provider, the model, a tool, input needed, or another typed reason | 1; 5 for input needed |
| `Budget(which)` | stopped on its own limit: turns, time or spend | 4 |
| `Refused(setup)` | never reached inference: the seed, the warm-up, the configuration or the credentials | 2; 3 for credentials |
| `Timeout` | the harness's hard deadline ended the tree | 130 after the harness's signal, or killed |
| `HarnessError(what)` | the harness failed: an unknown persisted shape, a grader that crashed | |

A budget end is not a failure, and a setup refusal is not the agent's:
rates leave both out and report them beside. A task whose story is a
startup refusal (a `failures` variant) is judged by its checks: its
refusal is the agent's end, not `Refused(setup)`, and it made no provider
request. The grade is apart: a
completed attempt can fail its grader. A forced exit after an accepted
answer is a warning on a completed attempt.

## 9. Statistics

- **Probes.** A probe passes when every check passes, and reruns once on
  failure. Two failures of two are a regression; calibration makes one
  unlikely to be chance. One of two quarantines the probe: reported, and
  gating nothing until it passes calibration again.
- **Comparisons interleave in one session.** Each block holds one attempt
  of each arm, in an order drawn from the run's seed, so that drift,
  warming caches and shared rate limits fall on every arm alike. At
  least five attempts per arm: three can never reach significance.
- **A difference is reported** per task and metric when the 95%
  bootstrap interval of the ratio of the arms' medians, resampled with
  the run's seed, excludes 1, and the effect exceeds the task's minimum
  detectable effect: about 2.8 · CV · √(2/n) for n attempts per arm, from
  its committed variation. Otherwise the summary says none was detected,
  not that the arms are equal.
- **Every attempt counts.** Pass rates are counts with exact intervals; a
  median over successes alone is never reported without the rest. Only
  outcome, wall time, tokens by convention and spend on one basis are
  compared across agents. Cache-read volumes and cached shares are
  reported, never compared across agents as rates: an agent with a
  larger fixed prompt reads more from its cache.
- **Parity** with another agent is non-inferiority, not "none detected":
  its time and cost criteria hold when the upper bound of the 95%
  interval of smith's ratio to the other agent lies below each one's
  margin, and its outcome criterion when smith's count of passes reaches
  the lower bound of the exact 95% interval of the other agent's count,
  over as many attempts (`codex-parity.md`). It is the
  verdict of its own suite, never inferred from a comparison that
  detected nothing.
- **A repository task's large effect** is section 3.1's disjoint ranges,
  computed by the summary for every metric both arms observe.
- **Baselines** are committed summaries of a suite for one agent
  configuration and tier: each task's medians, each metric's coefficient
  of variation, n and minimum detectable effects, with the binary and the
  date. A run outside one by more than a task's minimum detectable effect
  raises a drift notice, which asks for a comparison in one session and
  is never a verdict. A baseline is retaken by commit after an intended
  change, a new model, a new task version or a new result version. A
  suite is summarised once, whole; each baseline is cut from that summary
  by one command naming one agent, configuration digest, model and
  effort.
- **Audits are sampled.** A summary draws a few attempts with its seed,
  plus every attempt whose checks disagree with its grader, for a person
  or a model to audit: did the agent do the task asked, without reading
  the grader or changing what it must not. The scope gate is automatic:
  protected files unchanged, nothing written outside the workspace, no
  read of the benchmark tree in what the agent shows of its calls.

## 10. Isolation

- **Credentials are logins of the harness's own,** each signed in once
  by the developer with its agent's own flow, never a person's own tool
  directory and never a copy of a refresh token: issuers rotate them,
  and a copy is spent by whichever holder refreshes first.
  - **smith** runs on a credentials directory of the harness's, signed
    in once with `smith login` as the live suite's is (../testing.md, 2.3),
    and shared by attempts under each record's lock (../shell/credentials.md, 3);
    each attempt keeps its own state directory. Its Anthropic models run
    on a key.
  - **Codex** gets a `CODEX_HOME` of the attempt's own. Its `auth.json`
    is written by the harness from a Codex login of the harness's own: a
    `CODEX_HOME` the developer signs in once with Codex's own flow, and
    that Codex alone refreshes, never the person's `~/.codex`. The
    attempt's file holds the access and identity tokens and the account,
    with the refresh token left empty. It is `0o600` in a `0o700`
    directory and removed when the tree has ended. Codex cannot refresh
    it, so it never rotates the harness's login. Codex runs from a
    private copy of its pinned release's whole package under the
    harness's state, never the user's installation; its pin names the
    executable's SHA-256, and setup refuses another.
  - **Claude Code** runs with a configuration directory of the
    harness's, signed in once with `claude auth login`, never a copy of
    a login, which would carry a person's subscription credential
    outside its program. The login is shared; each attempt's settings,
    instructions and MCP servers are its own, given by flags (section
    6). Attempts on one login run one at a time.

  An access token that expires before the attempt's hard deadline (its
  expiry read from its claims) is a setup refusal, like a lapsed login.
  The refusal names the command to run once, whose own use refreshes the
  login where it lives. Nothing in the harness writes to a person's own
  tool directories.
- **Configuration is pinned, not inherited.** An agent's home is the
  attempt's, so nothing of the user's configuration, instructions,
  skills, plugins or memories reaches it. The keys it is given come
  from the command line, and only those an attempt must own (5.4): MCP
  servers, trust, approvals, model, effort and the caches it may write.
  smith's settings come only from the harness's layer (../shell/settings.md, section
  1).
- **Out of sight, nothing shared.** Attempts run under the harness's
  scratch root, outside every repository and the benchmark tree, each
  with its own temporary directory, build caches and target directory,
  warmed outside the timer and copied by reflink where the filesystem
  allows. A grader reaches the workspace's copy only after the tree has
  ended. Without contained trees nothing keeps an agent from the
  user's files, so the scope gate looks for it, and lists the user's entries
  under `/tmp` before and after each attempt: a new or changed one is a
  write outside the workspace. The directory it lists is an input of the
  attempt, its request's `shared_temporary`. A planned request names `/tmp`
  for every real run, and a document naming none means `/tmp`. Only `probe`
  takes another (`--shared-temporary`), and an offline test names a
  directory of its own, so what a test costs never depends on what else the
  machine has run.
- **Process trees.** An attempt ends when its whole tree has ended. At
  the hard deadline the tree is terminated, then killed after a grace;
  after an observed answer, the deadline is the agent's teardown bound
  (smith's from its effective limits, read by `smith check --json exec`
  with the attempt's flags;
  another agent's from its pinned configuration)
  plus that grace. Accounting of a whole tree (a cgroup scope, its CPU
  and peak memory) is skein's; without it each attempt runs
  in its own process group, and its resources carry the narrower scope.
- **Setup is not the agent's.** A seed digest that differs, a failed
  warm-up, a refused configuration or missing credentials end the attempt
  as `Refused(setup)`, and mark the suite incomplete.
- **Load and disk.** A grader sensitive to load runs serially on an idle
  machine. Raw output is bounded per attempt, and a suite checks free
  space against its estimate before it starts.

## 11. How it relates

- **To smith's tests.** Benchmarks find and show; tests specify and gate.
  A benchmark does not replay and its outcome varies, so what it finds is
  reproduced in the lowest tier that shows it (skein/docs/foundation/testing-strategy.md,
  section 9) and fixed there with that test; the probe stays as evidence
  of the effect. Benchmarks never stand in for a world's story.
- **To the live tier** (../testing.md, section 2.3). The live tier checks
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
  (6.3); Claude Code's are streams with repeated message ids, sub-agent
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
  spend observed (8.1, 8.2).
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
- **Fuzzy:** parsers fed truncated and corrupted lines never panic and
  never count a response twice.

Each runs in milliseconds, within the focused and fuzzy budgets.

## 14. Open questions

- **Codex's later versions:** whether a version after 0.160.0 still
  writes each thread's rollout under `sessions/` or `archived_sessions/`
  with its own `session_meta` as the first line, still links each child
  thread to its parent in that opening record, written before the child's
  first response, and still trusts the attempt root by
  `projects.<root>.trust_level`. A new pin checks each, on a recorded
  attempt with a child and a grandchild, before its first comparison: the
  collector finds every thread its stream and its state database name,
  each linked (6.3).
- **Claude Code:** the settings that allow edits and commands headless;
  whether `CLAUDE_CONFIG_DIR` keeps every user file out;
  `claude-haiku-5-5` at low effort on a subscription.
- **Claude Code's attempts at once on one login:** whether attempts that
  share a configuration directory interfere. Default: one at a time
  (section 10).
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
- **Task kinds beyond code,** such as chat, research or review, with
  graders of their own; and **audits:** how many attempts a run samples,
  and who audits them.
- **Recording smoke runs:** whether a commit hook checks that a runtime
  change's body carries one.
- **A failed response's usage:** skein's failed call carries none, so
  usage a provider sends with its failure is told as null (8.2, a
  retried failure). Whether a failure should carry it is skein's
  `llm/README.md`'s question.
- **A failure's kind in a summary:** a harness failure has a kind, and only
  its words reach a result: `HarnessError` and `Refused` hold text (8.3).
  Default: so; a summary that counts by kind changes the result's format.
