# Benchmarks: agent adapters and model tiers

## 1. Agents as adapters

`enum Agent { Smith, Codex, ClaudeCode }` is matched exhaustively, one
module per variant, in five steps. **Prepare** the attempt's private
directories and the agent's pinned configuration, writing nothing in the
user's tool directories.
**Command:** an exact environment from an allow-list, nothing inherited,
and the prompt on standard input. **Observe** the output as it arrives,
as neutral observations: a response and its usage, a tool call, a child,
the answer, which ends task wall time and starts teardown. **Collect**,
once the tree has ended, what the agent persisted, as metrics with their
scope and convention. **Classify** one end (`results.md`, section 1.3).

| Step | Smith | Codex | Claude Code |
|---|---|---|---|
| prepare | `XDG_CONFIG_HOME` holds the pinned settings as the user's layer: the account's file and the trace, capturing `calls` so the scope gate sees each call's input (`everything` where a task needs prompts), and nothing else of a comparison's; `XDG_STATE_HOME` the attempt's state; `credentials` the harness's credentials directory, and the account one of its own (`isolation.md`, section 1). Before an arm's first attempt, `smith check --json` with the attempt's own flags, those of `exec` for a headless arm and the interactive `smith`'s, its trace among them, for a Chat arm, records the effective limits, the teardown bound, schema versions and credentials of the run it will start; a nonzero exit refuses the arm as setup. The attempt's events are its standard output's, under `--json`: the harness gives `exec` no trace file, so the run has one event destination | a `CODEX_HOME` of the attempt's own, holding only an access-only `auth.json` the harness writes from the harness's own Codex login (`isolation.md`, section 1), so none of the user's instructions, skills, plugins, memories or configuration reach it and its rollouts are the attempt's; only the keys an attempt must own (`README.md`, 5.4) pinned by `-c` overrides: no MCP servers, the attempt root trusted, workspace edits allowed, no approvals, the model and effort, and the attempt's private caches among its sandbox's writable roots, so that it can write them; no other key, so that the rest is Codex's default; its background service disabled where it can be, or ended with the tree | `CLAUDE_CONFIG_DIR` the harness's own configuration directory, signed in once with `claude auth login` (`isolation.md`, section 1); the attempt's own settings, instructions and empty MCP configuration given by flags (`--settings`, `--strict-mcp-config`, `--setting-sources`), so that none of the login's directory's reach it, and its session found by its id; edits and commands allowed without asking |
| command | `smith exec --json -C <workspace> -m <endpoint>/<model> --effort <effort> -`, overrides as `-c KEY=VALUE` (../shell/commands.md, section 2); interactive, `smith --trace <file>` at a pseudo-terminal | `codex exec --json -m <model> -c model_reasoning_effort=<effort> -C <workspace> -o <final> -` | `claude -p --output-format stream-json --verbose --model <model> --effort <effort> --max-budget-usd <ceiling> --settings <pinned> --mcp-config <empty>`; `--input-format stream-json` for later messages |
| observe | the event stream, through `smith-events`; standard error is never parsed | JSON lines: thread, turns, items, and each turn's usage, of the root thread only, cached input within input | the opening record, assistant messages with usage, sub-agent messages with their parent's call, the result with durations, turns, cost and per-model usage, in the pinned version's recorded shapes |
| collect | nothing beyond the stream | the rollouts of the thread named, of the children its stream's `spawn_agent` calls name, and of each thread linked to one of those as a child (codex-cli 0.160.0 names its parent in the child's opening record), each thread's file the one under the attempt's `CODEX_HOME/sessions/` or `archived_sessions/` whose opening record names it, found in the rollouts alone and never in Codex's state database (1.3): per-response usage, children and their spans, and each command's span against its call's; where a child is not linked to its parent, child scopes are unavailable and totals lower bounds | nothing; transcripts are archived |
| classify | the exit code (../shell/commands.md, section 3), held against `run.completed` | the exit, the last turn's completion or failure, the deadline | the result's subtype and error flag, the exit, the deadline |

- **One runner.** Each attempt runs on skein's world harness, end to end
  (skein's examples.md, section 6): the agent starts as it ships; a
  scripted person types where a probe is interactive; the attempt's
  referee reads the output as it arrives and keeps the hard deadline.
  The live tier runs smith the same way.
- **An adapter fails loudly.** A known record in an unknown shape ends the
  attempt as a harness error naming it. Unknown record types are skipped
  (../protocol/events.md, section 6).

### 1.1 What each agent sees

- **Instructions.** A seed's are in `AGENTS.md`, and every agent gets the
  same text through the file it reads, whole: smith reads it into its
  system text by its default guide convention (../domain/run-start.md, 1.3), Codex
  into its first user message, and Claude Code's `../../../CLAUDE.md` is made at
  preparation and left out of the diff. smith runs with its shipped role
  (../shell/settings.md, 2.6): its pinned settings add nothing and remove nothing. No
  home or directory above the attempt's root holds
  instructions: each agent's home is the attempt's (`isolation.md`, section 1), since
  an agent may read its home's instructions whatever its command line
  says (codex-cli 0.160.0 reads `CODEX_HOME/AGENTS.md` unconditionally).
- **Models.** A comparison's smith declares each model as its presets do
  (../shell/settings.md, 2.1): its window, output and deadlines. A smaller window or
  output is a probe's, named in its task (`README.md`, 5.2).
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

### 1.2 smith as it ships

- **One face.** The Smith adapter drives smith only as it ships: `smith
  exec --json` headless, or `smith --trace <file>` at a pseudo-terminal
  for an interactive task, with its settings as the product's TOML
  layers (../shell/settings.md, section 1) and a task's overrides as `-c` settings.
  A binary that predates them is not run, and no adapter face, settings
  template or reader of an earlier binary is kept.
- **What a trace lacks is unavailable,** never estimated. A result names
  the face, and results of different faces are never compared.
- **Baselines** taken on an earlier face are retaken or dropped (`results.md`, section 2
  ). Results from before this harness are never a baseline.

### 1.3 Codex's threads

The Codex adapter's collect step finds the attempt's threads and their
files in its `CODEX_HOME`'s rollouts alone. Every rollout there is the
attempt's (`isolation.md`, section 1), so no file need be asked for by name, and the
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
  its scope unavailable and the totals lower bounds (`results.md`, 1.2). The root's own
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

## 2. Models as tiers

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
