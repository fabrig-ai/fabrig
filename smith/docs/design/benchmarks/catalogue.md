# The initial catalogue

Costs are estimates on the small tier until measured.

## 1. Probes

Every probe's outcome is its file, or the answer it asks for. The outcome
halves of `smoke-tools` and `large-write` also run on Codex and Claude
Code.

| Probe | Guards | Forces | Checks | Cost |
|---|---|---|---|---|
| `first-request` | a start's messages, the opening, awaiting (../domain/run.md, sections 3 and 6) | an empty brief and a one-line task that writes a file; headless with the task in the start, and interactive typed after startup; with and without a workspace | under `everything`, the first request's new messages hold the task; no response or `wait` before the task's `message.received`; at most two completions; time to the first `response.started` | seconds |
| `smoke-tools` | the commands' environment; one path namespace, the directory as `.`; a complete stream; teardown; exit evidence (../domain/tools.md, sections 2 and 5; ../protocol/events.md, section 5) | list `.`, read the README, `printenv` a nonce variable and `command -v cargo` into a file through the shell, answer with a README token. `child`: one child first, so two pooled connections are open at the answer | `.` listed; one shell call, exit 0; tool records paired by `call`; the responses' `spent` and `usage` sum to `run.completed`'s; nothing lost; `teardown_ms` within bound, not forced | ten seconds, ten thousand tokens |
| `follow-up` | message terminals, the fence, the unread report, a finish crossing a message, deferred `wait` (../domain/run.md, section 6); the local host's unread lines (../domain/host.md, section 8) | interactive: a greeting, then, once `finish` shows, a file asked for through the shell. `eof`: two lines, then end of input. `chat`: three exchanges with wait authority | exit 0; each `message.received` has one terminal; the request handled once, in the run it crossed or the next one's opening; in `chat`, one completion per exchange. When the local host shows "waiting" has no record in the stream, so the local worlds check it (../domain/host.md, section 8), not the probe | thirty seconds |
| `large-write` | the declared tool payload, per-call outcomes, each model's `output`, the truncated Anthropic `tool_use` (../domain/tools.md, section 3; ../protocol/limits.md, section 6) | one `write` of tens of kilobytes within the payload: a seed file of non-repetitive plain prose, a little larger than the size asked, copied, so that the content has an obvious end. A model asked to count out a repeated character neither counts nor stops reliably. What is judged is smith: the saved file equals what the call sent. How faithfully the model copied is recorded, not judged. Where the small tier will not copy reliably, the content may instead be numbered lines the prompt specifies (`Line 0001: <a fixed sentence>`, and so on, to a stated last number): no read, and an end the model can see. `oversized`: a call past the bound on one call's input, which arrives as an oversize block (../protocol/llm.md, section 4). With the production payload that bound, six payloads and more, is past what one response can write, so the variant declares a small payload, such as 2 KiB, and the bound follows from it. Its prompt says the first call is meant to be refused: one `write` of numbered lines past the call's bound; then a smaller file, past the payload, in pieces: a `write`, then an `edit` that extends it | one `write` of at least the size asked, `written`; no `max_tokens` stop. `oversized`: one call past the call's bound, as an `oversized` block, then the file in pieces, accepted. A refusal by the tool's payload instead (a `write` past the payload, within the call's bound) is not this variant's: ../domain/tools.md section 4's size refusal is proven by its deterministic test. An attempt that makes no call past the bound, and otherwise makes the file in pieces, accepted, is "not exercised (bound honoured)": neither a pass nor a fail, since a model honouring a bound smith states is what smith asks of it. The variant is proven by three exercised passes within forty-eight attempts | under a minute, ten thousand output tokens |
| `history` | the window, compaction, derived limits, selective decoding (../domain/session.md, section 8; ../protocol/limits.md, section 3) | sum values held in many small files, read one call at a time, the values out of order so that a summary must carry them, and the answer's form stated whole in the prompt. The running sum and the next file are written as text beside each read, so that what a summary must carry is in the window's history, not only in the model's reasoning. `small-window`: a window several times smaller than the reading, with a compaction due just before the last read | `request_bytes` and prompt tokens per request recorded. Each read's verdict is smith's; the number of reads is the model's (`call-count`, `README.md`, 3.2). Each compaction's request holds the results owed to it, and the next window opens with its summary (`compaction-carried`). `small-window`: a `conversation.opened` of kind `compaction`; every compaction request at tool choice `none` (`compaction-choice`); no prompt above the window; cache reuse around it recorded. With those passing, a summary that drops or misstates a read it was given, so that the read is made again or skipped, is a model deviation. The compaction's condition is a window past its line: the window less one output, the summary instruction and the markers, which the stream gives as the conversation's model's `window_tokens` less its `output_tokens`, from `conversation.opened`, less `run.compaction_material`, from the effective limits `session.started` carries (../protocol/limits.md, sections 3.9 and 9). A request has passed it when its prompt, as its usage counts it, is above the line. A stream without `run.compaction_material` gives no line, and leaves `conversation-opened` and `compaction-choice` "not exercised", naming why. The guard (../domain/session.md, section 8.1) sends no ordinary request whose prompt may pass the line, so smith makes no ordinary completion past that point: it compacts first. A prompt between the threshold and the line is smith's estimate missing, and is no failure. An attempt that opened no compaction, and none of whose requests reached that point, leaves `conversation-opened` and `compaction-choice` "not exercised" (`README.md`, 3.2): no compaction was owed, since the model stopped before its reading filled the window. An attempt with a request past that point and no compaction fails `conversation-opened`, which is smith's | two minutes, up to a million input tokens; weekly |
| `cache-reuse` | affinity, the stable prefix, Anthropic breakpoints, usage (../protocol/llm.md, section 2) | pointer chasing over a dozen files, one read each, a fixed number of requests. `children`: two children in turn. `crossing`: a message part way, at medium effort, so reasoning crosses a user turn. `idle-gap`: a gap past the cache's lifetime before a message | reuse R, the cache read over the previous request's prompt, summed where the predecessor can be cached: median of three at least 0.85, and full misses (under half) at most one per ten requests over the three, until calibration sets them; the three repetitions are one attempt (5's `probes` suite). On `anthropic`, cache writes about each turn's addition. A control arm without affinity falls clearly below: one attempt of it fails these criteria, which makes it the probe's before (`README.md`, section 4.2) | a minute each; `idle-gap` weekly |
| `budget` | the reserve, wind-down, the final turn, budget text, the clamped share, real spend (../domain/run.md, section 5.3 and ../domain/budget.md) | `starvation`: a tight budget and a delegated survey. `wrap-up`: a child given `max_turns` 3. `child-quota`: a child given `max_turns` 2. `visibility`: a report of what is left. `spend`: notional prices, a ceiling about twice a calibrated spend | `starvation`: accepted, main's last completion after the child's return, turns within budget. `wrap-up`: under `everything`, `stop: budget` and text, the last request at `tool_choice` `none`. `child-quota`: the child answers within two turns, its `share` in `conversation.opened`. `visibility`: main reports what is left as it was last told before the report (with a child, the last child's usage line, ../domain/budget.md 2). The told turns and time equal the `budget` record of the step that wrote them, time in whole seconds rounded down; a mismatch there is smith's. The report repeats the told values; a mismatch there is the model's. No child opens into the reserve. `spend`: never past the ceiling | under a minute each |
| `parallel-read` | calls per response and the reads run at once (../protocol/limits.md, sections 2.2 and 2.3) | twenty files read in one step | the reads in one response's calls; no `limit` failure; no failed send | thirty seconds |
| `parallel-children` | concurrent conversations, a pool at capacity waits, the write lease (../domain/run.md, section 5.3; ../protocol/limits.md, section 2.2) | four read-only children in parallel, one directory each. `reviews`: three independent reviews | two children overlapping in time; no call refused by the pool | a minute |
| `failures` | typed failures in words, headless exit codes, limit failures and HTTP status, clipped provider text (../domain/host.md, section 8; ../shell/commands.md, section 3) | `unknown-model`; `derived-limit`, by a test profile; `one-turn`, a budget of one turn, which cannot hold the finishing reserve; `bad-setting`, a malformed setting | a run's failure in `run.completed`'s typed failure: the class, and the HTTP status and clipped provider text, or the limit and bound. `unknown-model` declares the model's `window` and `output`, so that its request reaches the provider. `one-turn` is refused at the entrance (../domain/budget.md, 1): `run.completed` `refused` with `invalid` naming the budget's reserve, and no provider request. `bad-setting` and `derived-limit` are refused at startup, in either mode (../shell/commands.md, 1): exit 2, standard error naming the setting, or the limit and bound, no provider request, and no run, so no `run.completed`. Headless only (../shell/commands.md, 3): the exit code of the class and standard error's last line, judged from `smith exec`. A run that runs out of budget once admitted is the budget probes' | seconds |
| `paths` | one path namespace (../domain/tools.md, section 2) | `app` writable and `spec` read-only: read both, answer in `app`. `single`: one directory, its name never needed. The prompt names the files as any agent finds them on disk, from the working directory `app`: `spec`'s through `../spec/` (../domain/tools.md, section 2: siblings by path). smith's own text tells its model `/spec/...`, and either route must resolve. Every agent runs in `app`, with `spec` beside it, read-only and protected | no `missing` verdict or path refused as outside; at most two completions before `finish` | seconds |
| `tool-output` | output budgets, read windows and timeouts as the model chooses them; UTF-8 passed through; jobs (../domain/tools.md, sections 3 and 5) | `test-output`: which assertion fails, in over ten kilobytes of output. `quote`: a sentence deep in a long document. `timeout`: a command past its timeout, and one asking past the ceiling. `utf-8`: replace an en dash. `background`: a command of about a minute started as a job, beside reads the answer also needs | one shell call; at most two reads; `timed_out` naming its bound, `clamped` set, no hang; one edit, matching first time. `background`: the command answered `running`, a completion started between the job's start and its end, one terminal for the job | seconds each; `background` a minute |
| `process-tree` | a command in its own process group, signalled whole (../domain/tools.md, section 5) | `sh -c 'sleep 600 & wait'` with a short timeout | no `sleep` left, by tree accounting; answered within timeout and grace; accepted | seconds |
| `credentials` | a sign-in record shared across processes (../shell/credentials.md, 3) | two smith attempts at once on one `sign_in` account of the harness's, the refresh lead raised so that both refresh | both accepted; the record's generation rose, and no refresh was refused as reused | seconds |
| `slow-completion` | deadlines that measure progress; an oversized reasoning item as a typed failure (../protocol/limits.md, section 7) | long reasoning at high effort, from a real provider (a fake slow source is the deterministic worlds', not this probe's) | the answer accepted; no timeout of any phase while events arrive; no reasoning-size failure; the call lasting over 60 s, a whole-response cap met in practice (`../../development/benchmarks/wall-clock-plan.md`). An attempt under 60 s is "not exercised": neither a pass nor a fail, counted like a model deviation (more than two in twelve strengthen the prompt, or raise the effort). Recorded per provider and model, for ../shell/README.md's `head`, `idle` and `reasoning_item`: `first_byte_ms`, `largest_gap_ms`, the call's length and the largest reasoning item | minutes; occasional |

Two probes more: **`mid-work`**, a message offered
between completions of long work, and threshold notes the same way;
**`build-environment`**, a seed with an ambient build wrapper that builds
and tests, for trees that declare build profiles.

## 2. Passive checks

On every smith attempt:

- one `session.started`, one `run.completed` per run, one
  `session.ended`, and the exit agreeing with the answer;
- no authoritative record lost, each sink's `loss` counted, none under
  `complete`;
- `teardown_ms` within the agent's bound and not forced; a forced exit
  after an accepted answer is a warning;
- every `response.started` and `tool.started` completed, a tool's under
  its `call`; every `conversation.opened` closed; every
  `message.received` with exactly one terminal;
- the responses' `spent` summing to the run's; no failure of class
  `limit` the task does not expect;
- headroom, with a warning at half: `request_bytes`; prompt tokens
  against the window; `first_byte_ms` against `head`; `largest_gap_ms`
  against `idle`; turns; spend; `peak_rss_bytes` against `memory`.

On every attempt of any agent: the tree ended; protected files
unchanged; nothing written outside the workspace and the private
directories; no read of the benchmark tree seen.

## 3. Experiments

| Experiment | Over | Arms |
|---|---|---|
| `affinity` | `cache-reuse` | the cache key and session header; plus the thread header; plus private routing state and reasoning context, adopted only on a material gain |
| `cache-lifetime` | `cache-reuse/idle-gap` | Anthropic's default lifetime, and a longer one |
| `compaction` | `history/small-window` | the threshold, and what the new window holds |
| `delegation` | `shell-cwd` | each agent with its sub-agents off; both with them on |
| `edit-format` | `03-planner`: "add a doc comment to each public function and run the tests" | each dialect's formats, by completions before the first edit and edits that fail |

## 4. Fixtures and the repository task

| Task | Asks for | Grader | Runs |
|---|---|---|---|
| `01-backoff` | overflow-safe delay and deadline arithmetic in a small crate | hidden edge-case and regression tests; formatting | by default |
| `02-line-decoder` | an incremental line decoder with a bounded buffer | hidden tests; fixed-seed comparisons with an oracle | by default |
| `03-planner` | a closure planner with ordered topological output and error priority, across files | hidden tests; an oracle over generated graphs | occasionally |
| `shell-cwd` | a working-directory option for the shell tool, through protocol, domain, machine and documentation, on smith at a frozen commit with its skein pin vendored | an oracle crate over that commit's interfaces; formatting, lints, both suites within budget, serially on an idle machine | at milestones |

A fixture takes minutes per attempt at the working tier; `shell-cwd`
tens of minutes and millions of tokens (estimates). Where a correct
solution could fail a grader, or a wrong one pass it, the grader is
strengthened first. Each grader holds a copy of the seed's own tests and
runs them against the agent's work, so that a weakened test cannot hide a
regression; a task's references add the tests its prompt asks for. A
prompt asks for nothing a pinned agent as shipped cannot do, and invites
no delegation: delegation is its own experiment (3). Per change, `budget` and `history` stand in for the
repository task's delegation and length.

## 5. Suites

| Suite | Tasks | Agents | Repetitions |
|---|---|---|---|
| `smoke` | `first-request`, `smoke-tools`, `large-write`, `budget/child-quota`, `budget/wrap-up`, `failures/one-turn`, each only while its task file holds a calibration | smith on `openai`, small | one, rerun on failure |
| `probes` | every probe and variant | smith on both providers; Codex, later Claude Code, on outcome halves | one; three where judged on a median |
| `compare` | `01-backoff`, `02-line-decoder`; `03-planner` when asked | smith and Codex, later Claude Code, working | at least five per arm, interleaved |
| `repository` | `shell-cwd` | smith and Codex, working | three, outcomes and large effects only |
| `parity` | `01-backoff`, `02-line-decoder`, `03-planner`, `shell-cwd` | smith and Codex, working, each as it ships | eight per arm for each fixture and ten per arm for `shell-cwd`, interleaved, no rerun (`codex-parity.md`) |
| `parity-change` | the `parity` suite's tasks | two smith arms, working: a change and its parent | five per arm for each task, interleaved, no rerun (`codex-parity.md`, 6) |
| `calibrate` | one probe | smith, the probe's tier | ten passes within twelve on the fix; then the binary before, or a control arm; writes the variant's `[calibration]` block (`README.md`, 5.2) |
