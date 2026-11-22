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
| T | the agent's largest turn, which smith derives from the agent's profile and endpoints (smith's `protocol/limits.md`, 3.6) | a box and a worker take it from their agent's configuration, as today's checks do; an engine without an agent takes the stock agent's at the engine's P (section 3) |
| P | the larger of the limit `tool_payload` (`limit-settings.md`, 2.3) and the agent's own `declared.tool_payload` from the `agents` setting (`figures.md`, 5), each taking its stock value when absent | fixed at the engine's start; an `agents` edit above P is refused by name, naming the `tool_payload` limit to raise and restart with first |
| `live_goals` | the most goals live at once, the limit `live_goals` (`limit-settings.md`, 2.3), 40 in the stock, at most 100, the live goals the store is sized for (`limit-settings.md`, 3) | the engine and the box |
| a worker's slots | the worker document's `slots` | the worker |
| the stream bounds | `clients.streams` and `clients_http.streams`, after the document's overrides | the engine, in jig's reader (jig's `protocol/limits.md`, stream overrides) |

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
2. **The sources' settings:** `live_goals` and `tool_payload` from
   temper's limits setting, or the stock's (`limit-settings.md`, 3).
3. **The derivations** of 5.1, from T and P, and of 5.4, from
   `live_goals`, written into that stock. The result is the base that the
   settings' overrides are read over (jig's `protocol/limits.md`, the
   stock).
4. **The settings' overrides,** by dotted name, jig's kind's over jig's
   limits and temper's over temper's (`limit-settings.md`, 3). A worker
   alone reads its document's instead. A figure derived from another
   figure of the same set (5.2) follows that figure's value after the
   overrides, unless it is overridden itself.
5. **The start's checks,** as today. They now check temper's derivations.
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
- **A worker's own link** caps its output at a u32 too: its places
  times a run's turns, calls and asks, past 147 MB a place at the stock
  (8 turns of 18,431,842 bytes), so it holds fewer places than
  `fleet.slots`. A worker whose places pass it refuses its start,
  naming its figures (`figures.md`, section 3, step 5).
- **A box's worker** is built and admitted at its setting `local`'s
  places, kept in its data directory (`figures.md`, section 4); its
  document's `slots` gives none.

### 5.4 From `live_goals`

- **A count that scales with the work in flight** is
  ⌈stock × `live_goals` / 40⌉, its stock figure at the stock's 40 live
  goals, so the stock resolves to the figures `limits.md` gives, and 80
  live goals to twice them.
- **The counts that scale:**
  - jig's `tasks.tasks`, `tasks.project_tasks`, `tasks.project_goals`,
    `tasks.placed_holds`, `tasks.project_placed_holds`,
    `tasks.party_placed_holds`, `tasks.exemptions`, `tasks.funders`,
    `tasks.pools`, `tasks.hold_waiters` and `people.inbox_tasks`;
  - temper's `forge.tasks`, `forge.holds`, `forge.subscriptions`,
    `forge.entries`, `forge.client.entries`, `forge.client.resources`,
    `forge.issues` and `forge.landings`.

  The rest keep their sources: a task's own fan-out and lifetime, the
  catalogues, history, a worker's places and the readers' streams don't
  grow with goals in flight. `forge.changes` keeps its own figure, since
  the store's tier rows cap it (`limits.md`, `forge.changes`).
- **A tie the start checks is kept by derivation,** not by scaling both
  sides: `people.inbox_tasks` is twice `tasks.tasks`, `tasks.exemptions`
  at most `tasks.tasks`, and `forge.landings` at most `forge.holds` over
  `forge.landing_members`, each from its scaled source.
- **The other side of a tie follows its scaled side:** where a check
  ties a count of the list to one outside it, the outside figure is
  derived, at most its stock unless the check needs more, and never below
  what the check needs: `forge.landing_members` is the least of its stock
  and `forge.holds`; `tasks.delegates` the least of its stock and
  `tasks.tasks` less one; `people.operations` the larger of
  `tasks.delegates` plus one and `tasks.project_goals`, at most
  `tasks.tasks`. At the stock's 40 live goals each is its stock figure.
  An override of one still wins, and the checks judge it.
- **A cap a format sets holds:** a scaled count takes the least of its
  formula and its cap, `forge.holds` the store's 256 goals of a tier
  row, `forge.tasks` its 1,024 closer's selections and
  `tasks.exemptions` its 4,096, so a `live_goals` past a cap scales the
  rest and holds the capped count there.

### 5.5 Where the code is

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

### 5.6 The engine's stop and report bounds

From `../../draft/reconciling-start.md`, 4 and 5: the bounds derive from
one another, and a unit takes its timeouts from the engine's.

- **The stop bound** is the settled stop's (`../../domain/root-restart.md`,
  1.2): twice the sum of the forge client's write lifetime and clock
  margin, `forge.client.lifetime_ms` and `forge.client.clock_margin_ms`
  (skein's `forge/client.md`, 7.8), from the limits in force at the
  start (`limit-settings.md`). At the stock's 300 s and 1 s it is 602 s.
  One function computes it, `skein_forge_client::stop_bound(&Limits)`,
  which the client's settle reads too. The drain after the settle, its
  commits answered and its components closed, is local and has no bound
  of its own.
- **A unit's stop timeout** is the stop bound plus at least 30 s,
  rounded up to a whole minute: 660 s at the stock, the stop bound plus
  58 s (`temper/docs/help/server.md`, 3). A deployment that raises
  either limit raises its unit's timeout by the same rule.
- **No start timeout:** a unit sets none, since the engine's start lasts
  as long as its store's open and its steps take, and a step past the
  report bound says what holds it.
- **The report bound** is 30 s, `WAIT_REPORT`, one fixed figure for
  every step of a start and every full table (`../README.md`, 6.3),
  with `Report`, the clock of its doublings, beside it in
  `temper-engine-domain-forge`, the lowest crate every one of its
  readers names (the forge, the root and the engine's shell).
- **Printed** by `temper-engine versions` (`../binaries.md`, 2), both at
  the stock limits, in whole seconds rounded up.

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

- **The agents' figures from the engine** (`../../draft/config.md`, 6.3):
  each worker then derives from the figures its opening brings, not from
  its own document.
- **A worker's places from its setting,** and the engine's limits on
  workers from the workers' settings (`../../draft/config.md`, 6.7).
- **The graces' derivation:** a worker's stop bound from its graces, and
  the engine's fleet grace above it (`../../draft/config.md`, 6.5 and
  section 7).
