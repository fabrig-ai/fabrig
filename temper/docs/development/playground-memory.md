# The playground world's native heap

These are native counting-allocator bytes, not browser memory or resident size.
The playground binary's test holds the service's 2026-11-13 figures below
with a quarter's margin rounded up to two significant digits. Other
regression bounds retain their 2026-11-12 figures plus a quarter
([playground.md, section 9](../design/playground.md)). Sections 8 and 11 leave
the memory's bar itself to the user.

## Commands and scope

From the repository root, with seed 7, the epoch wall clock, calm simulator,
and trace off:

```sh
cargo nextest run -p temper-playground-world --test memory --no-capture </dev/null
cargo nextest run -p temper-one-process-world --test memory --run-ignored only --no-capture </dev/null
```

The playground measurement passes. The first step is one world step, including
one turn of every admitted process. Start is the first idle boundary with
nothing due now: 159 world steps in the current table. The two snapshots use separate identical
worlds, so each disk can be destroyed and its retained heap measured separately.
The service process contains both engine and worker; they are not separate
process measurements. The visitor is admitted at both snapshots; no agent
runs before the visitor acts.

The test's `skein_sim_world::Meter` uses `skein_heap::Span` for admission,
process calls, release and restart. Held bytes accumulate signed net growth;
peak bytes are the largest held amount plus a call's transient growth.
Story A constructs peers before admission. A disposable identical `Start`
measures the bytes freed by each peer, while retaining the shared forge, and
charges those retained bytes once to the corresponding process at admission.
The shared forge is reported separately. Construction transients before
admission are outside the reported process peaks.

The fake disk is retained outside the world, then its last owner is dropped
under a span after the world is dropped. Its figure includes filesystem
metadata, file contents, durable copies and pending writes, rather than just
logical file lengths. Simulator bookkeeping, shared forge state and the
composition's standard-output capture stay outside process spans. Trace is off;
the composition still captures the sign-in line as its normal `step` does.

## Measured bytes

Measured 2026-11-03 (`date -u +%F`), on a builder, at commit
`79d1b7c27b20bce64fcc5e6b3e75c3db3732948e`. This historical table is retained
for comparison; the service's bounds now follow the 2026-11-13 table below.

| Process | First step held | First step peak | Start held | Start peak | Story A high water |
|---|---:|---:|---:|---:|---|
| 0: service (engine + worker) | 122,754 | 160,247 | 186,163 | 190,459 | unmeasured: story A does not walk yet |
| 1: fake LLM | 27,219 | 27,219 | 27,955 | 27,955 | unmeasured: story A does not walk yet |
| 2: fake forge HTTP | 29,570 | 29,570 | 30,311 | 30,355 | unmeasured: story A does not walk yet |
| 3: hosted git | not admitted | not admitted | 3,050 | 8,215 | unmeasured: story A does not walk yet |

| Outside the processes | First step retained | Start retained |
|---|---:|---:|
| Fake disk | 8,778 | 43,539 |
| Shared forge before start | 6,981 | not separately measured at start |

Measured 2026-11-08 (`date -u +%F`), on a builder, at trunk commit
`2c7e0d079e92266a03d667dba08dca3fd52ca062`. This historical table is retained for comparison; the service's bounds now follow the
2026-11-13 table below. Start took 147 world steps. The visitor is process 3, so four processes are measured at
the first step and five at start, with hosted git now process 4. The bounds
are keyed by the test's process names, including `visitor` and `git`.

| Process | First step held | First step peak | Start held | Start peak |
|---|---:|---:|---:|---:|
| 0: service (engine + worker) | 144,717 | 226,963 | 239,146 | 285,982 |
| 1: fake LLM | 27,449 | 27,449 | 28,185 | 28,185 |
| 2: fake forge HTTP | 30,002 | 30,002 | 30,743 | 30,787 |
| 3: visitor | 1,054 | 1,068 | 1,054 | 1,068 |
| 4: hosted git | not admitted | not admitted | 3,410 | 5,952 |

| Outside the processes | First step retained | Start retained |
|---|---:|---:|
| Fake disk | 9,393 | 124,117 |
| Shared forge before start | 6,981 | not separately measured at start |

Measured 2026-11-12 (`date -u +%F`), on a builder, from the stock-charter
source at commit `a983a9a0aff2d8d51333bd917164fedeefe61e35`. This table is
retained as history. All figures came from one run of the focused
`story_as_world_at_its_first_step_and_start` test, with its regression
comparisons suspended for measurement and then restored against this table
at `figure + figure / 4`. Start took 150 world steps.

| Process | First step held | First step peak | Start held | Start peak |
|---|---:|---:|---:|---:|
| 0: service (engine + worker) | 206,908 | 340,891 | 341,137 | 450,031 |
| 1: fake LLM | 27,449 | 27,449 | 28,185 | 28,185 |
| 2: fake forge HTTP | 30,130 | 30,130 | 30,871 | 30,915 |
| 3: visitor | 1,054 | 1,068 | 1,054 | 1,068 |
| 4: hosted git | not admitted | not admitted | 3,450 | 5,989 |

| Outside the processes | First step retained | Start retained |
|---|---:|---:|
| Fake disk | 9,393 | 216,638 |
| Shared forge before start | 7,061 | not separately measured at start |

The stock's five files contain 31,630 bytes. Their assembled instructions,
including the parts' joining blank lines, hold 33,185 bytes across the goal
and implementer charters, up from 11,451 bytes. Service held growth is
62,191 bytes at the first step and 101,991 bytes at start, much more than the
21,734-byte text growth: this points to multiple owned copies of the text,
not one copy. These snapshots do not attribute each allocation to a copy.

Measured 2026-11-13 (`date -u`), on a builder, from source commit
`157d31cc0c709e45cd720859f78706992c613e6c` on `temper62287/c2316`.
Every figure below came from one run of
`story_as_world_at_its_first_step_and_start`, with comparisons suspended
only for measurement and restored afterwards. First step takes one world
step; start takes 159 world steps.

| Process | First step held | First step peak | Start held | Start peak |
|---|---:|---:|---:|---:|
| 0: service (engine + worker) | 264,685 | 419,385 | 478,318 | 605,314 |
| 1: fake LLM | 27,585 | 27,585 | 28,321 | 28,321 |
| 2: fake forge HTTP | 30,394 | 30,394 | 31,135 | 31,179 |
| 3: visitor | 1,054 | 1,068 | 1,054 | 1,068 |
| 4: hosted git | not admitted | not admitted | 3,450 | 5,990 |

| Outside the processes | First step retained | Start retained |
|---|---:|---:|
| Fake disk | 9,393 | 249,617 |
| Shared forge before start | 7,229 | not separately measured at start |

The service's parsed bridges and roots at both ends add 20,518 held bytes
at the first step (244,167 to 264,685), isolated against an opaque-Application
control. Nonservice drift from the 2026-11-12 table came with the prerequisite
changes and is identical in that control: LLM +136, forge +264, shared forge
+168, git start peak +1, disk start +32,979 bytes. Only the service's ceilings
move: each measured figure plus a quarter, rounded up to two significant
digits, giving 340,000/530,000 bytes at the first step and 600,000/760,000
at start (held/peak). Every nonservice ceiling stays unchanged.

The following one-process observation is the unbound 2026-11-08 measurement;
its ignored test has no active figures to repin. The one-process binary takes the same entry as
`temper/tests/one-process/tests/suite/story_a.rs`:
`world::run(fixtures::calm(fixtures::SEED), world::Story::PlaygroundA)`, with
`Memory::Checked` and trace off. The story walks all three requests in
44.8 simulated seconds. The memory measurement reaches the same end, then
fails the exact ownership check at process 0's drop: it frees 1,200,446 bytes
against 1,828,886 metered bytes. Its observer allocates shared referee store
records within the service's spans; extracting those records before the
process drops leaves their heap outside that drop. No outcome or peaks return,
so the memory test remains ignored and its high water is not yet bound
(`temper/docs/design/playground.status.md`, 9).
Measuring that figure needs a skein mechanism for a service world's observer
to keep its storage outside the process's span first
(`skein/docs/design/testing/README.md`, 5).

Measured 2026-11-12 (`date -u +%F`), on a builder, on branch
`temper6684/c748`. R2's stock charters' prompts/settings and the expanded
inventory account for the growth. The focused playground test replaces
only the exceeded service rows and fake disk's start row with these figures
plus a quarter; every other bound keeps the 2026-11-08 baseline. Start
now takes 150 world steps.

| Process | First step held | First step peak | Start held | Start peak |
|---|---:|---:|---:|---:|
| 0: service (engine + worker) | 187,703 | 295,333 | 306,622 | 391,748 |
| 1: fake LLM | 27,449 | 27,449 | 28,185 | 28,185 |
| 2: fake forge HTTP | 30,130 | 30,130 | 30,871 | 30,915 |
| 3: visitor | 1,054 | 1,068 | 1,054 | 1,068 |
| 4: hosted git | not admitted | not admitted | 3,450 | 5,990 |

| Outside the processes | First step retained | Start retained |
|---|---:|---:|
| Fake disk | 9,393 | 178,778 |
| Shared forge before start | 7,061 | not separately measured at start |

The one-process memory binary was run once by name with `--run-ignored only`.
It completed the three requests at 45.8 simulated seconds, then failed the
same exact-release check: process 0 freed 1,287,344 bytes against 1,863,104
metered bytes. It returned no high-water row, so that row remains unmeasured
and the test remains ignored; no one-process bound changed.

## Halving the knobs

Each row below is a separate local, uncommitted edit at the 2026-11-03 commit,
followed by the playground command above, then `git checkout -- <edited-file>`
before the next edit. No changed knob is committed. The fake-limit rows each
halve a group of capacities together; they do not attribute effects to individual
fields within that group. The detailed groups below name every edited input.
All passing rows keep 147 steps, both fake-disk figures, and the shared-forge
figure unchanged. An unchanged peak here says nothing about the unwalked story.

| Knob and edited file (from repository root) | Baseline → halved | Effect on first-step/start peaks | Floor observed at startup |
|---|---|---|---|
| `STORE_KV.budget`, `jig/crates/jig-protocol-config/src/profile/small.rs` | 268,435,456 → 134,217,728 | none; passes | none reached; half passes |
| `STORE_KV.commit`, same file | 355,308,810 → 177,654,405 | no process snapshot; startup refused | half is below admission floor: validator requires map commit 300,905,662 bytes |
| `STORE_KV.queued_bytes`, same file | 1,065,926,430 → 532,963,215 | no process snapshot; startup refused | half is below admission floor: validator requires map queue 902,716,986 bytes |
| `engine.memory_bytes`, `temper/crates/temper-playground/stories/a/deployment.json` | 1,152,921,504,606,846,976 → 576,460,752,303,423,488 | service peaks 160,247 → 160,246 and 190,459 → 190,458; git start peak 8,215 → 8,214; others unchanged | none reached; half passes |
| `workers[0].memory_bytes`, same file | 1,152,921,504,606,846,976 → 576,460,752,303,423,488 | same one-byte peak changes as engine; others unchanged | none reached; half passes |
| `profile.declared.llm_pool`, `temper/crates/temper-playground/src/documents/smith.rs` | standard value 134,217,728 → 67,108,864 (append `/ 2` to the assignment) | none; passes | none reached; half passes |
| `agent_profile().declared.read_window`, `temper/crates/temper-playground/src/documents.rs` | 256 → 128 | none; passes | none reached; half passes |
| Fake LLM capacities, `temper/crates/temper-playground/src/fakes/llm.rs` | group below, each halved | none; passes | not exercised by Story A's peer construction |
| Fake forge capacities, `temper/crates/temper-playground/src/fakes.rs` | group below, each halved | none; passes | none reached on the startup path |
| `client_limits().connections.body_bytes`, `temper/tests/one-process/src/fixtures.rs` | 262,144 → 131,072 | none; passes | no client in the playground startup measurement |

The one-byte changes in both service held/peak snapshots and git's start peak
come from shortening one decimal literal in the retained deployment JSON by one
byte, rather than reserving half the declared memory. No other peak moves.
The store failures are startup validation of configured edges, not evidence
that startup allocated hundreds of megabytes. Only the half-value probes were
run; no minimum passing value was searched for, and these are not story floors.

The fake LLM group's baseline inputs, each divided by two in its run:

- `llm_config`: `calls=4`, `query_bytes=65_536`, `script_bytes=32_768`,
  `answer_bytes=32_768`, `cache_entries=16`.
- `llm_limits`: `calls=4`; HTTP `head=4096`, `headers=32`, `body=65_536`,
  `read=256`, `response=4096`, `send=128`; SSE `event=16_384`, `chunk=128`;
  document `model_ceiling=4096`; OpenAI overrides `request=65_536`,
  `strings=16_384`, `tokens=4096`, `output_items=256`; Anthropic
  `output_items=256`. The `bounds.answer.max(32768)` literal becomes `16384`;
  the derived bounds and the declaration used to derive them stay as written.
- `llm_peer_limits`: io `sockets=8`, `intake=32_768`, `output=65_536`;
  peer `connections=4`, `queue=256`, `plaintext=32_768`, `ciphertext=65_536`,
  `observations=256`, `observation_bytes=4_194_304`. Inherited io limits,
  timeouts, probabilities and response semantics stay as written.

The fake forge group's baseline inputs, each divided by two in its run:

- `peer_limits`: io `sockets=16`, `refusals=8`, `intake=65_536`,
  `receive=1024`, `output=65_536`, `sends=8`, `accepts=4`, `backlog=8`;
  peer `connections=4`, `queue=64`, `plaintext=32_768`, `ciphertext=32_768`;
  `STORY_OBSERVATIONS=1024`, `STORY_OBSERVATION_BYTES=2_097_152`.
- `forge_limits`: `calls=4`, `connections=4`, `raw_bytes=4096`;
  HTTP `head=4096`, `headers=16`, `body=25_165_824`, `read=1024`,
  `response=4096`, `send=1024`; document JSON `chunk=1024`,
  `string=25_165_824`, `number=32`, `depth=16`, `length=25_165_824`;
  writer `depth=16`, `length=25_165_824`; document `string_bytes=512`,
  `body_bytes=25_165_824`, `inner=256`, `target_bytes=4096`.
  Names remain zero; timeouts, retry, tokens and forge contents stay as written.

Story A's `Start` builds its LLM and forge peers through `temper-fake`, not
`fakes::scripted_provider` or `fakes::forge_peer`. Thus the measured peer peaks
are not governed by the composition's fake-limit groups above. The default
harness's `git_limits` does take `fakes::forge_limits` for routed git; that
halving still leaves every measured startup peak unchanged. No hosted agent or
native client runs at this boundary, so their knobs cannot establish a story
floor here.

## What these knobs do not bound

No document names a cap for the simulator's bookkeeping, the fake filesystem's
metadata and retained durable/pending copies, the shared forge's retained object
graph, or the composition's sign-in-output capture as a whole. Their containers
grow with use. Prebuilt peer construction transients, world setup/teardown,
allocator bookkeeping and the test runner are outside the reported process
peaks. The native figures do not measure the wasm linear-memory reservation,
browser runtime, module compilation, page/frame, or download.

The service's internal dynamic containers and startup parsing allocate as used;
its `memory_bytes` declaration does not reserve that many bytes. The measured
store budget likewise does not reserve its full amount at startup. As
`skein/docs/foundation/programming-model.md`, section 6.3 specifies, these
measurements neither sum capacities into a process-wide worst case nor compare
such a sum against a declaration at startup.

The standing-roles R2 merge with upgrades U1 measured the service at 232,223
held and 381,487 peak bytes on its first step, and 438,969 held and 565,289
peak bytes after the same 150-step start. R2 adds findings and routing state
and combines it with the newer stock instructions. The start held and peak
bounds use these measurements plus a quarter; the first-step measurements
remain within the existing bounds. The focused memory test passes with these
bounds.
