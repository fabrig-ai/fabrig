# Benchmarks: results and statistics

## 1. The result schema

### 1.1 One result per attempt

A JSON document. Like every format it is pre-release: one version, any
change a reader would notice is a new one, and a reader refuses others.
Committed summaries and baselines of an older version are regenerated or
dropped, never read across versions.

| Part | What it holds |
|---|---|
| identity | run, attempt and arm; the task's id, version, seed and prompt digests; the agent, its version, its binary's SHA-256 and, for smith, its face; the configuration's digest; the provider; model and effort, requested and resolved; the harness's commit; the host and CPUs used; the start, in wall time |
| timing | on the harness's one clock: process wall, to the tree's end; task wall, to the answer observed; teardown, from the answer to the tree's end; grading wall. The agent's own times beside them: smith's `first_byte_ms`, `largest_gap_ms` and longest completion, and its `t_ms`, from its own start. The time record: the seconds of the main conversation (Codex's root thread) in model requests, with their first-byte and largest-gap sums; in each tool, by name; in compaction; waiting on children; and the rest; and the seconds of commands that ran outside any call |
| tokens | per scope (section 1.2) |
| counts | responses, provider attempts and retries; completions; tool calls by name, not comparable across agents; tool failures by verdict; children; conversations and compactions; messages by terminal |
| delegation | for each child: its effective share, how it ended (an answer, its budget, failed, closed) and its turns and seconds; the time the parent was blocked on children |
| resources | CPU time and peak memory, each with its scope: the tree's accounting, the agent's process alone, or unavailable. smith's own peak resident size is its `session.ended`'s `peak_rss_bytes` (../protocol/events.md, 3.2) |
| spend | the agent's own figure, such as smith's `spent` or Claude Code's cost, with its basis: notional, priced or reported; and the common priced spend, every agent's tokens priced on one basis, as its provider bills them: each response's tokens at the list rates of the tier its own prompt reaches, from smith's presets' price list (../shell/settings.md, 1.2 and 2.1) at the harness's commit, which the summary names. The agent's declared window plays no part. The adapter reads each response's usage from the agent's records: smith's `response.completed`, Codex's usage per model response. Without a response's usage, its prompt's tier is unknown, and the common priced spend is unavailable, or a lower bound under 1.2's rule for a retried failure. A response that failed unsent reached no provider, and costs nothing (1.2). The harness keeps no catalog of its own |
| outcome | the end; the exit, a code or a signal; whether the harness forced it (no, terminated, killed); each check's verdict and evidence; protected files changed; writes outside the workspace |
| health | smith's loss per sink; headroom against each limit, as the fraction of its bound used; the passive checks |
| artifacts | each raw file's path and digest |

Every measurement is a `Measure`: observed, a lower bound, or unavailable
with its reason, never written as zero and never filled from another
scope.

### 1.2 Tokens

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
    `llm/vocabulary.md`, section 1: a completion carries the token usage).
  - **Its scope's totals and the attempt's are lower bounds,** and each
    names it as a retried failure, apart from any other gap. They miss
    the failed request alone. Its prompt is its retry's (../protocol/events.md,
    section 3.11), and the retry's usage counts that prompt.
  - **The common priced spend** (1.1) of an attempt whose totals miss
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

### 1.3 Ends

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

## 2. Statistics

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
- **A repository task's large effect** is `README.md`, section 3.1's disjoint ranges,
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
