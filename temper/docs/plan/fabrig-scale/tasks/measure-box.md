+++
id = "measure-box"
part = "gate"
kind = "docs"
needs = ["fresh-box", "reference-trim"]
files = ["temper/docs/design/protocol/setup.status.md"]
effort = "high"
minutes = 120
+++

# The resource gate: a fresh box's memory and disk at the new defaults, measured

## Goal

The user's gate before landing (`inputs/request.md`, "The resource gate"): measure a fresh box
at the new defaults, temper's resident memory idle and with one goal running, and the store's
size on disk, fresh and after reference-scale work, and write the figures in
`temper/docs/design/protocol/setup.status.md`. Expected: tens of MB of memory, up to about
100 MB; a few hundred MB on disk, up to about 1 GB. Past either bound, at any point, stop and
ask with the figures and what drives them; never change a figure to come under it.

This task departs from the implementer's text twice, as stated: it runs the built binary
outside a test to measure it (in the slot, on loopback, with the scripted model and no network),
and it edits a design status file (`setup.status.md`), the one place the user asked the
figures to be written.

Acceptance:

- **Built once:** `heavy cargo build -p temper --bin temper` (the gate's profile, as the
  binary suite builds it); the binary from that build's target directory.
- **A fresh box:** `temper init` into a scratch directory under the slot (untracked, deleted
  before the push), with no override and nothing edited but what init asks for; `temper box`
  started on it, as `temper/crates/temper/tests/suite/startup.rs`'s
  `a_box_on_a_fresh_init_reaches_serves_link_with_forge_3_dormant` starts one (read it for the
  arguments and the ports; loopback only).
- **Memory,** read from `/proc/<pid>/status` of the box's own processes (the box and `temper
  serve`, and the worker if it is a process of its own; `VmRSS` and `VmHWM`), never another
  process's environment:
  1. idle: a minute after `serve` answers;
  2. with one goal running: a goal started on the scripted model through `temper ctl`, as the
     binary suite's playground support drives one (`temper/crates/temper/tests/support/playground.rs`,
     `support/ctl.rs`), read while its run is live; and the peak (`VmHWM`) once it has ended.
- **Disk:** `du -sb` of the data directory's deployment store (its `generations/`, `active`,
  `transcripts/`) and of the whole data directory:
  1. fresh, after the first start;
  2. after reference-scale work on the same box: 80 chats started at once through `temper ctl`
     on the scripted model (the stock funds them: `fresh-box`), each answered by the scripted
     model's `otherwise` turn, and the playground's change run to its landing, as the
     playground support drives it; then stopped and restarted once (the store's snapshot and
     generation).
- **Written:** `setup.status.md`, one entry after its **Not yet** line, exactly in this form,
  with the figures and today's `date -u` and the machine's name (`hostname`):

  `- **Measured** (8.3, the resource gate, YYYY-MM-DD, on HOST): a fresh box at init's defaults:
  resident memory idle A MB (serve B MB, box C MB), with one goal running D MB, its peak E MB;
  the store on disk fresh F MB, after 80 chats and the playground's change, and a restart, G MB
  (the data directory H MB).`

  and its 8.3 **Not yet** clause "the resource gate, ... not measured." removed (the entry keeps
  the rest, if any).
- **Past a bound** (resident memory past about 100 MB at any reading, or the store past about
  1 GB): stop before writing, and ask (`Question:`), with every figure, what drives the largest
  (from `/proc/<pid>/smaps_rollup`, the store's files' sizes, the profile figures that size
  them: jig's `standard` `STORE_KV`, temper's journal), and the options you see. Commit nothing
  but what the question needs.
- The citations check passes.

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K2 (the store's figures
  within the gate).
- `temper/docs/plan/fabrig-scale/inputs/request.md`, "The resource gate".
- `temper/docs/design/protocol/setup.md`, 8.2 and 8.3 (init's document, its box's steps), 7.1
  (`temper box`).
- `DESIGN.md`, section 2 (status files).

## Owns and may change

- `temper/docs/design/protocol/setup.status.md`, for what acceptance names; nothing else is
  committed. Scratch directories, logs and scripts stay untracked in the slot and are deleted
  before the push.
- No network: the scripted model, the fake forge, loopback; no sign-in, no real provider, no
  real repository. If a step needs one, stop and ask.

## Steps

1. Build; init; start the box; read memory idle.
2. One goal; memory while it runs and at its end.
3. 80 chats and the playground's change; restart; disk.
4. Under both bounds: the status entry; the citations check; commit and push. Past either: the
   question.

## Report

As the implementer's text asks, with every reading (time, process, figure) and how each was
taken.

## Answers

2026-11-04. **Does idle summed RSS of 100.585472 MB pass "up to about 100 MB", and how far may
the later readings go?** The user's answer: it passes. 92 MB of the 98 MB RSS is the 245 MB dev
binary's executable pages, shared between box and serve; temper's own anonymous memory is
12.5 MB, the same as `small`. This replaces the spec's memory method and bound above; the rest
of the spec stands, the dev build among it.

- **The gate is anonymous memory:** the `Anonymous` line of `/proc/<pid>/smaps_rollup`, summed
  over the box's own processes (the box and `temper serve`; a transient child such as the git
  probe is left out of the sum and named in the report). Beside it, at the same reading, the
  summed `Pss` and `VmRSS`, and at the end the summed `VmHWM`.
- **Stop and ask only** if anonymous memory passes 100 MB (100,000,000 bytes) at a reading, or
  the store on disk passes 1 GB (1,000,000,000 bytes, the `du -sb` of `generations/`, `active`,
  `transcripts/`). RSS or PSS past 100 MB stops nothing; a question then carries the figures
  and what drives them, as "Past a bound" says, and nothing is written in the status file.
- **Every reading is taken, on one box:** the first run's box and scratch are gone, so the
  rerun starts from a new init and build and takes idle again. Memory at each of: idle; one goal
  running; its end; the 80 chats live; after the playground's change lands; after the restart,
  once idle. Disk as the spec says. While the goal runs, sample `smaps_rollup` at least once a
  second: the kernel keeps no anonymous high-water mark, so the peak is the largest sample,
  and the report says so.
- **The status entry,** in place of the spec's form, in this form exactly:

  `- **Measured** (8.3, the resource gate, YYYY-MM-DD, on HOST, dev build): a fresh box at
  init's defaults: anonymous memory (smaps Anonymous, box and serve summed) idle A MB (serve
  B MB, box C MB), with one goal running D MB, its sampled peak E MB, after 80 chats and the
  playground's change I MB, after a restart J MB; beside it, summed, RSS idle K MB, with one
  goal running L MB, PSS idle M MB, with one goal running N MB; the store on disk fresh F MB,
  after 80 chats and the playground's change, and a restart, G MB (the data directory H MB).`

  MB is 1,000,000 bytes. The 8.3 **Not yet** clause goes as the spec says.
