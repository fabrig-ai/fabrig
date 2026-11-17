# Derived figures

Provisional, 2026-11-16. The figures temper computes from their sources
when it starts, instead of reading them from a document: the turn's and
the payload's carriers, a worker's places, and the clients' carriers
(`../../draft/config.md`, section 7). `settings.md`, section 7 gives the
relations, and this document makes them computed. The derivations start
from the stock (`limits.md`, one stock).

## 1. In one page

- **A figure with a source is computed from it,** at the least that
  passes today's start check, with any margin in the code. No document
  writes it.
- **The sources** are the agent's largest turn, T, and its tool
  payload, P (section 2), a worker's `slots`, and the clients' stream
  bounds.
- **The order:** first the stock, then the derivations, then the
  document's overrides, then the start's checks (section 4). An override
  of a derived figure is still read, and one below its derivation is
  refused by today's check, naming both.
- **An engine without an agent,** a cluster's, derives from the stock
  agent's T and P (section 3).
- **Each formula has a focused test** that its figure passes today's
  check at the stock's sources and at twice them (section 6).

## 2. The sources

| Source | What it is | Where a process takes it |
|---|---|---|
| T | the agent's largest turn, which smith derives from the agent's profile and endpoints (smith's `protocol/limits.md`, 3.6) | a box and a worker take it from their agent's configuration, as today's checks do; an engine without an agent takes the stock agent's (section 3) |
| P | the agent's `tool_payload` | as T |
| a worker's slots | the worker document's `slots` | the worker |
| the stream bounds | `clients.streams` and `clients_http.streams`, after the document's overrides | the engine, in jig's reader (jig's `protocol/limits.md`, stream overrides) |

Until the agents' figures come from the engine (`../../draft/config.md`,
6.3), a worker's agent configuration is its own document, and a worker
derives from it.

## 3. The stock agent

- **temper's stock agent is smith's `standard` profile with temper's
  figures over it** (`../../draft/config.md`, 5.1). Its figures are
  dev.fabrig.ai's agent's of 2026-11-16, which is the reference's: the
  declared figures and endpoints that `temper-box init` writes in
  `agent.json` (`../setup/init.md`, 3), with `tool_payload` 63,740 and
  every endpoint's `output` 48,000 (`settings.md`, 7, the reference's
  room).
- **Its payload is 63,740, not 64 KiB.** skein's rule between a call's
  payload and an answer, (payload + 260) × 6 ≤ output × 8, makes 63,740
  the most that an output of 48,000 admits. 64 KiB would need an output
  above 49,347 (`../../draft/config.status.md`, Design departs).
- **Its T is 18,431,842 and its P 63,740.** temper's limits hold both as
  the stock's sources, and a focused test holds each equal to smith's
  derivation over the stock agent.
- **smith's own figures stay smith's.** smith's `standard` is sized for
  its own presets, whose smallest output, 32,000, bounds its payload at
  40 KiB (smith's `shell/settings.md`, 2.3). temper's figures go over it
  and don't replace it.

## 4. The order

1. **The stock:** jig's stock, with temper's figures over it (`limits.md`).
2. **The derivations** of 5.1, from T and P, written into that stock. The
   result is the base that the document's limits are read over (jig's
   `protocol/limits.md`, the stock).
3. **The document's overrides,** by dotted name, as today (`limits.md`,
   overrides by dotted name). A figure derived from another figure of the
   same document (5.2) follows that figure's value after the overrides,
   unless it is overridden itself.
4. **The start's checks,** as today. They now check temper's derivations.
   An override below its derivation is refused in today's words, naming
   the figure and its source.

## 5. The formulas

### 5.1 From T and P

The relations of `settings.md`, section 7, computed. "Both" means the
engine's and each worker's.

| Figure | Computed as |
|---|---|
| both `hub.turn_bytes` | T |
| the engine's `core.resume_bytes` | the larger of T + 4 and 4,194,304 |
| both `hub.transcript_bytes` | resume + 12 × ⌊resume / 4⌋, the start's resume window |
| both `hub.turn_queue_bytes` | the agent's unacknowledged turns × T |
| the engine's and each worker's `io.output` | the larger of fabrig's own development's 18,432,866 (`limits.md`, the stock covers fabrig's development) and the least room the start's checks take for a turn of T's frame: the links' frames, the smallest resumed start and the checkout's answer |
| `root.journal.transcript_bytes` | resume |
| `root.loads.transcript_bytes` | T |
| `root.loads.bytes`, `root.loads.reply_bytes` | T + a temper record's size |
| `root.loads.read_bytes` | the larger of 2 × T and a read's room: T, a row and a record |
| `tasks.message_bytes`, `tasks.result_bytes` | P − 2 |
| `tasks.inbox_bytes` | the agent's inbox × (P − 2) |
| both `hub.event_bytes` | P − 2; the engine sends its hub limits to the workers (`figures.md`, 2) |
| `checkout.message_bytes` | P |
| `forge.client.op_bytes` | `tasks.result_bytes` |
| both `hub.outcome_bytes`, both `core.call_answer_bytes` | the larger of the stock's figure and P |
| `forge.client.answer_bytes` | the larger of the stock's figure and the least that its validity check takes, the largest row of every listing |

### 5.2 From the document's own figures

| Figure | Computed as |
|---|---|
| a worker's `hub.slots` | its `slots` |
| a worker's `checkout.workspaces` | its `slots` |
| a worker's `git_component.operations` | its `checkout.workspaces` |
| `forge_component.body_bytes` | 2 × `forge.client.op_bytes` + 21, skein's marker length |
| `forge_component.repositories` | `forge.repositories`, the most the start's check takes |
| `clients_http.calls` | the larger stream bound + 4 |
| `clients.exchanges` | the larger of the larger stream bound + 4 and `clients_http.calls` |

The worker's three figures are its places: a worker's slots decide its
hub's slots, its workspaces and its git operations, and its document
writes only `slots`.

### 5.3 The engine's fleet slots

- **The engine admits each worker at its hello's slots,** up to
  `fleet.slots`, a ceiling per worker. The stock's ceiling covers any
  worker of fabrig's development, so a worker's own `slots` decide its
  places. It is the largest round figure (`limits.md`, sized from what)
  at most 80, twice dev.fabrig.ai's 36 rounded up, that the engine's link
  to a worker holds: that link's cap is a u32, slots times a slot's
  assignment, message frames and inbox, near 61 MB a slot at jig's
  stock. It holds at jig's stock and at the figures temper's engine
  starts with, so the generated deployment starts: 40 on 2026-11-17
  (temper's base with its derived carriers: 4,144,964,324 bytes at 40, including the figures opening,
  past the u32 at 48), where jig's
  stock alone holds 64. It is never below dev's 36. Where temper's
  derived figures make a slot larger still, temper's base sets its own
  lower `fleet.slots` by the same rule. The
  engine's document needs neither `fleet.slots` nor `hub.slots`, and an
  override of either is still read.
- **A box's worker** is admitted at its document's `slots`, which the
  start checks against the ceiling, as today.

### 5.4 Where the code is

- **temper's derivations** live in `temper-service-core`'s `derived`
  module, which the engine's, the box's and the worker's starts share.
  These names cross changes:
  - `Sources`: a record of `turn` (T), `payload` (P), `inbox` (the
    agent's unread messages per run) and `unacknowledged` (its turns
    held unacknowledged), each a `u32`.
  - `STOCK`: the stock agent's `Sources`, which are 18,431,842, 63,740,
    82 and 8 (section 3).
  - Taking a process's `Sources` from its agent's declaration, as
    today's checks read the agent.
- **jig's reader** derives the stream carriers and a worker's hub slots
  (5.2), and reads a document over a base: `jig-protocol-config`'s
  `stock()`, `engine_over` and `worker_over` (jig's
  `protocol/limits.md`, the stock).
- **`Sources` is a projection** of the agent's declaration, the four
  figures that the derivations read. It isn't a second agent type.

## 6. Tests

- **Each formula of 5.1 and 5.2 has a focused test** in the crate that
  computes it. The test computes the figure at the stock's sources and
  at twice them (2T and 2P, twice the slots, twice the stream bounds),
  and checks that it passes today's check and that one below it is
  refused, naming both.
- **Where twice a source passes a ceiling** of a wire or the store
  (jig's worker wire's `turn_body` of 33,554,432, and the store's
  transcript ceiling, take no 2T at the stock's T of 18,431,842), the
  test checks the formula's arithmetic at twice, and today's check at
  the largest source that ceiling takes. At twice, it checks that the
  start refuses, naming the figure and its source, and never panics.
  The ceilings stay: raising one moves its wire.
- **The stock agent's T and P** have a focused test that holds them
  equal to smith's derivation over the stock agent (section 3).

## 7. Later

- **`live_goals` and `tool_payload` as settings,** with the counts that
  scale with work in flight written as formulas of `live_goals`
  (`../../draft/config.md`, 5.2 and 6.1).
- **The agents' figures from the engine** (`../../draft/config.md`, 6.3):
  each worker then derives from the figures its opening brings, not from
  its own document.
- **A worker's places from its setting,** and the engine's limits on
  workers from the workers' settings (`../../draft/config.md`, 6.7).
- **The graces' derivation:** a worker's stop bound from its graces, and
  the engine's fleet grace above it (`../../draft/config.md`, 6.5 and
  section 7).
