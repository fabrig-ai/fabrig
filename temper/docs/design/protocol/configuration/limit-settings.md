# Limits as settings

Provisional, 2026-11-17, from `../../draft/config.md`, 5.3, 6.1 and 6.6
(`../../draft/config.execution.md`, N2), and its store sized for 100 live
goals, 2026-11-19, from 5.2 and 5.3 (N9), and its live figures,
2026-11-20, from `../../draft/archive/worker-restarts.md`, 8. The limits leave the
documents and become two kinds of settings, one jig's and one temper's,
changed by an owner by asking temper, with `temper ctl` or from Needs
you. A raise of the figures 5.1 lists is taken at once, a change of the
live figures 5.2 lists is taken by the engine and every worker at the
next operation that reads it, and every other change at the next start. A limit reached names its setting, and its work goes on when room
frees or the setting rises. The stock is `limits.md`'s and `derived.md`'s;
this document says how settings lie over it.

## 1. In one page

- **Two kinds** (section 2): jig's `limits`, a kind of jig's core, holds
  overrides of jig's limits; temper's kind 22, `temper-limits`, holds
  overrides of temper's, `live_goals`, `tool_payload` and
  `transcript_memory` among them. Each value is a list of overrides, an
  override a name, a value and a reason. No override is the stock.
- **One home each.** The engine's and the box's documents no longer hold
  `limits`, `memory_bytes` or `application.limits`, and refuse them by
  name (section 8). A worker alone's document keeps its limits until its
  figures come from the engine (`../../draft/config.md`, 6.3).
- **The store is the stock's** (section 3): the store's own figures are
  derived from the stock alone, sized for 100 live goals, so a start
  opens its store before it reads a setting, reads the two kinds, and
  only then sizes the engine. Every figure a setting gives must fit the
  store the stock sized, `live_goals` at most 100. At rest, the larger
  store costs nothing.
- **Checked whole, before it's decided** (section 4): a change or a
  proposal naming either kind is resolved and checked by the start's own
  checks at entry, in the service, and refused there naming the figure
  and what fails. So an accepted setting is one the next start takes.
- **At once, live or at the next start** (section 5): a raise of
  `transcript_memory`, `forge.tasks`, `forge.holds`, `forge.entries` or
  `forge.client.entries` is taken at once by everything holding a copy;
  a change of a live figure, a time bound or a ceiling every holder
  reads at each use, such as `checkout.remote_timeout_ms`, is taken by
  the engine and every worker at the next operation that reads it, with
  no restart; every other change, and every other lowering, at the next
  start. Each limit's row says which.
- **A limit reached names its setting** (section 6), and the work it
  held goes on by itself when room frees or the setting rises, and at
  every start.
- **What a person sees** (section 7): the deployment's Limits page lists
  every limit with its stock, its setting and its reason, the figure in
  force and whether a change waits for the next start; Needs you offers
  an owner the raise of a limit that holds work, in one act; Ask temper
  reads and proposes both kinds.
- **A start a setting can't take** names it, and an operator's file,
  `limits-held`, starts it at the stock with the figures it gives
  (section 3).

## 2. The two kinds

### 2.1 An override

- **`Override`**, one record for both kinds, in jig's
  `jig-domain-core-vocabulary` (`settings`), since temper names jig's
  vocabulary and both mean the same thing:

  | Member | What |
  |---|---|
  | `name` | a limit's dotted name, as the limits' names tables give it (jig's `jig-protocol-config` `names::NAMES`; temper's `limits/names.rs` `NAMES`): 1 to 64 bytes of `a`-`z`, `0`-`9`, `_` and `.` |
  | `value` | its figure, a `u64`, in the name's unit (a count, bytes or milliseconds) |
  | `reason` | why it departs from the stock, 1 to 120 bytes of UTF-8, required (`../../draft/config.md`, 11's default) |

- **A value is a list of overrides,** strictly ascending by name, so
  each name at most once, at most 256 of them. These bounds are
  constants beside the record: `OVERRIDE_NAME_BYTES` 64,
  `OVERRIDE_REASON_BYTES` 120 and `OVERRIDES` 256, above either names
  table's length.
- **One shape check,** beside the record, `check_overrides`, judges a
  list's shape alone: its count, each name's bytes, the order, and each
  reason. jig's core check and temper's forge judge both call it. Its
  fault names the position of the first override that fails, or the
  count. Whether a name is a limit and its value a figure the start
  takes is the service's check (section 4), since the names tables are
  the protocol's.

### 2.2 jig's kind, `limits`

- **A kind of jig's core,** one setting with no number, as the rules are
  (jig's `domain/engine/settings.md`, section 1): `Setting::Limits`, its
  value `Value::Limits`, a list of overrides of jig's names. Its default
  is the empty list, the application's stock, given in the core's
  `Config` defaults as every kind of jig's is.
- **The core keeps it and never applies it.** The core checks its shape
  (`check_overrides`, fault *shape*) as one more item of its one check,
  keeps it as it keeps the rules, in its settings family, and shows it in
  the settings documents and reviews. Applying it is the application's,
  at its start: `jig-protocol-config` reads it over a base by name,
  `with_overrides(base, overrides)`, with the dotted names and checks a
  document's `limits.overrides` had, and its problems name the setting
  (section 3).
- **On jig's client wire** a `SettingValue` of its own, so the wire
  moves its version (jig's `protocol/client/README.md`).

### 2.3 temper's kind, 22, `temper-limits`

- **An application kind** of temper's (`settings-kinds.md`, section 1):
  number 22, the deployment's scope, the forge connector's, one value,
  naming no charter. Its value is a list of overrides of temper's names,
  kept and judged by the forge child as every temper kind is; its judge
  is `check_overrides`. Its default is the empty list.
- **Its settings file's name is `temper-limits`;** jig's kind's is
  `limits`. Each item is the whole list.
- **Three of its names are figures nothing else sizes:**

  | Name | Unit | Stock | What | Applies |
  |---|---|---|---|---|
  | `live_goals` | count | 40 | the most goals live at once, at most 100, the live goals the store is sized for (section 3); the counts of `derived.md`, 5.4 scale with it | at the next start |
  | `tool_payload` | bytes | 63,740 | the largest piece a tool writes, P; the carriers of `derived.md`, 5.1 follow it | at the next start |
  | `transcript_memory` | bytes | 1,073,741,824 | the memory the runs' transcripts and turns may take while work prepares, not the process's (`../../draft/config.md`, 5.3) | at once |

  `transcript_memory` is the root's budget, formerly `memory_bytes`
  (`../../domain/root.md`): new work waits to prepare while what the root
  holds is past it, as before.

### 2.4 The two lists as one

- **jig's names and temper's are disjoint,** a focused test holds it, so
  the two kinds read as one list by name: the Limits page, a review and
  `temper ctl settings show` list both kinds' overrides together, by
  name.
- **Which kind a name is in** is the names table that holds it: a
  proposal or a change naming one of jig's names in `temper-limits`, or
  the reverse, is refused at entry, naming the kind it belongs to.

## 3. At a start

- **The store is sized from the stock alone, for 100 live goals.** The
  start derives the store's own figures (its keys', values', operations',
  commits', pages', chunks' and segments' bounds, `../store.md`) from the
  stock with no setting, as it derived them from a document with no
  override, but with `live_goals` at the store's figure, 100, not the
  stock's 40: the figure of `../../draft/config.md`, 5.2's "plan for 100
  live goals". Neither kind can move them: a store's open replays its log
  and snapshot under these figures, and a lowered commit bound reads a
  sound log as a damaged tail.
- **The store's live goals,** `STORE_LIVE_GOALS`, 100, a constant beside
  the stock's `LIVE_GOALS` in `temper-service-core`'s `derived` module.
  The store's base is the stock resolved at those live goals
  (`derived.md`, 4 and 5.4), with the stock's `tool_payload` and the
  start's sources, and no setting; the store's records are laid out over
  it (`../store-versions.md`, 1) as they were over the stock. Its map value bound
  is the largest record's encoding there: a person's stored answer to a
  compound request of amendments (jig's `jig-store-people`), which grows
  with `people.operations` and so with `live_goals`, 100,881,496 bytes at
  40 and about 249 MB at 100. Its page bound is key plus value, or a
  turn's frame where that is larger, and its chunk the value's with its
  framing. Its commits', segments', snapshots' and queue's figures follow
  the map's budget, the root's fill mark, and the journal's writes, and
  don't move.
- **The store's ceilings are derived, never written.** jig's stock
  profile caps a value at 103,387,876 bytes and a page at 103,389,955,
  its own layout's needs (jig's `protocol/store/records.md`, 4). temper's
  store raises both, for the store and the secrets alike, to what its
  layout over the store's base needs, computed at the start, so a record
  that grows moves the ceiling with it and no figure is retyped. A typed
  world's stock (`temper-limits-fixtures`' `small`, a test's base) lays
  its store out over its own figures, as given.
- **`live_goals` at most the store's.** The start's checks refuse a
  `live_goals` past `STORE_LIVE_GOALS` before the store's sizing, naming
  the store's figure:

  `live_goals (setting temper-limits): 101, past its bound of 100, the most live goals the store is sized for; put limits-held in the data directory to start at the stock`

  and the entry check (section 4) refuses a change or a proposal setting
  it in the same words, as it runs the same function. Within it, a count
  overridden past what its live goals scale to, `tasks.project_goals` 400
  say, still meets the store's own check, naming its setting and the
  store's figure it passes, as today:
  `tasks.project_goals (setting limits): the store's map value, N bytes,
  past its bound of M bytes`, then the same ending. A `limits-held` line
  past either is refused naming its line.
- **At rest, the larger store costs nothing.** The store's bounds are
  admissions, not allocations: skein-kv's map, queue and census grow with
  what is held, a read's buffer is that read's own length, and the
  layout's conversion buffer grows with the values it converts (jig's
  `protocol/store/layouts.md`, 7). So the engine at rest holds what it
  held over the store for 40 live goals, within `../../draft/config.md`,
  5.3's few tens of MB, and so does one whose `live_goals` is 100, since
  the counts it scales are ceilings of lazily grown containers. A
  focused test measures it (section 9).
- **An existing store opens at the larger bounds** at its next start,
  its format unchanged: no bound is written in the store's files, which
  hold lengths, counts and checksums, so nothing converts and no version
  moves (`../../draft/config.execution.md`, N9's dev.fabrig.ai column).
- **So the start reads its settings before it sizes anything else:**
  1. it opens the store at the stock's figures;
  2. it reads the two kinds' current values, jig's from the core's
     settings family (its change of `Setting::Limits`, if any) and
     temper's from the forge's settings rows (kind 22's change, if any);
     a first start, or a kind no change is over, reads the empty list;
  3. it resolves the limits (below), checks them whole, and builds the
     engine with them, the store's codec ceilings among them;
  4. it restores, as today.
- **The store is admitted once.** The store's admission, its sizing
  above with its read and whole-file checks, ends with jig's check of
  the store's limits and gives its result, jig's `Checked` (jig's
  `protocol/store/README.md`, 8). The store opened at step 1 is made
  over it, and the engine built at step 3 over the runtime check's;
  neither evaluates a bound again. A limit jig's check refuses is
  refused at admission, as the store's ceiling, before any store is
  made, where it stopped the start at the store's making before. A
  world that reopens its maps at the figures it admitted them at takes
  that admission's `Checked`; at a changed figure it admits afresh.
- **The order of the resolution,** over `derived.md`, section 4:
  1. the stock: jig's, with temper's figures over it;
  2. `live_goals` and `tool_payload` from temper's kind, or the stock's;
  3. the derivations, from T, P and `live_goals` (`derived.md`, 5.1 and
     5.4), written into the stock;
  4. jig's kind's overrides over jig's limits, by name, and temper's over
     temper's;
  5. the figures that follow others of the same set (`derived.md`, 5.2),
     unless overridden;
  6. the start's checks, as today, and the store's: every record's
     largest encoding, the journal's writes and a turn's frame under the
     resolved figures within the store's stock figures.
- **Its refusals name the setting:** where a check named a document's
  path, `engine.limits.overrides.tasks.tasks`, it names the limit and
  where its figure came from, `tasks.tasks (setting limits)`,
  `forge.entries (setting temper-limits)` or `forge.entries (the stock)`,
  then today's words for what fails. A figure past the store's names the
  store's figure it passes.
- **A name a release no longer has** is left out: the start says once
  on stderr `limits: <kind> names <name>, which is no limit of this
  release; it is left out`, the Limits page shows it so, and the start
  goes on.
- **What the store holds past a lowered figure** refuses the restore,
  naming the setting, its figure and what is held, as a lowered
  document limit did (jig's `domain/engine/settings-store.md`, 1): a lowering
  is the owner's to make when the work has drained. What is held is
  given in the setting's own units, so the owner can compare it with
  the figure: the least figure of that limit that admits the record.
  Where a codec bound is the limit itself, that is the field's length or
  count; where a bound derives from it (`derived.md`, 4), it is the
  figure the derivation needs, not the field's length.
- **`limits-held`,** a file in the data directory, starts an engine
  that a setting would refuse, as `landing-held` holds landing
  (`../../domain/landing-settings.md`, 1.3). Each line is `NAME VALUE`,
  a limit and a figure. While it is there, the start takes neither
  kind's overrides: it resolves the stock with the file's figures over
  it, says once on stderr `limits held: <path>; the limit settings are
  not taken; remove it and restart to take them`, and the Limits page
  shows every setting as waiting for the next start. A start refused by
  a setting ends its words with `put limits-held in the data directory
  to start at the stock`. The owner then changes the setting, the
  operator removes the file and restarts. A line naming no limit, or a
  figure the start's checks refuse, refuses the start, naming its line.
- **Stopped tools** `checkpoint`, `restore`, `upgrade --back` and
  `upgrade --workers` take `limits-held` as a start does, through the
  same pure resolution without opening the deployment store during
  preparation.
- **`upgrade --check` sizes its engine as a start does,** from its
  copy: once its configuration is checked, it opens the copy at the
  stock's figures, reads the copy's two kinds, or takes `limits-held`'s
  figures in their place, resolves and checks them, and builds its
  engine with them before it restores (steps 1 to 4 above, over the
  copy). So it refuses what a start over the copy would refuse, in a
  start's words, and passes what a start would pass. A copy that
  doesn't open is reported by an engine at the stock's figures, or
  `limits-held`'s.
- **A box** resolves its engine from the two kinds. Its worker takes its
  limits from its engine (`../../draft/config.md`, 6.3), so temper's kind
  sets the names a worker reads (checkout, git, the peer's). Its document's
  `workers[0].limits` is refused as moved (section 8). A worker alone also
  takes its figures from its engine, not its document.
- **A forge command alone,** `temper-box forge prepare|run` and the
  runner it starts, opens no store: the engine owns it, and may hold it
  while the command runs. So it reads neither kind and takes the stock's
  `setup.` figures. The engine's setup host takes them from temper's
  kind. A test that needs other `setup.` figures in a forge command
  gives them through the command's reader as typed limits, in process
  (section 8).

## 4. A change

- **Checked at entry, whole.** A party's `ChangeSettings` or settings
  proposal, and a run's `propose_settings`, naming an item of either
  kind, is checked by the service before the root sees it: the limit
  settings in force with the request's items over them are resolved and
  checked by the same function the start runs (section 3, steps 1 to 6
  of the order), with the start's inputs that are no setting (the
  sources, the document's members) as the start had them. A failure
  refuses the request there: a party's as out of shape, its `at` the
  item and its words the start's; a run's with the start's words, which
  it reads as a tool's refusal. A name no names table holds is refused
  the same way, naming it.
- **So the next start takes what an owner accepted.** A proposal is
  checked when it is made, against its revision; any newer revision
  withdraws it (jig's `domain/engine/settings-proposals.md`, 1), so its acceptance
  needs no second check. A proposal a run sends in parts is checked part
  by part; the next start checks the whole, and `limits-held` covers a
  start it refuses.
- **The service keeps the settings in force:** the two lists it read at
  the start, and each value a decision commits. The root tells it, once
  a revision naming either kind is durable, the two kinds' values and
  the revision, and the service checks the next request against those.
- **What is taken at once:** once that revision is durable, the service
  resolves the limits again. When every figure the revision moved
  against the figures in force is a raise of one of 5.1's or a change of
  one of 5.2's, the engine takes the new figures at once, whole.
  Otherwise it takes the revision's live figures alone (below), and none
  of the others until the next start; the Limits page shows those
  waiting. A lowering of one of 5.1's figures is never taken at once; a
  live figure is taken either way.
- **The lists taken.** Beside the two lists in force, the service keeps
  the two lists its figures in force resolve from: at a start, the lists
  it read; after a whole take, the revision's. A take of the live
  figures alone forms its lists from those taken, with each live name's
  override, or its absence, from the revision, and resolves them by the
  start's own function (section 3, steps 1 to 6 of the order). It is
  taken when they resolve and pass its checks, and every figure their
  resolution moves against the figures in force is live, the figures
  that follow others (`derived.md`, 5.2) among them; otherwise nothing
  is taken until the next start. So the figures in force are always a
  resolution the start's checks passed.
- **Example.** The figures in force hold the stock's
  `checkout.remote_timeout_ms` 60,000; a revision sets `live_goals` to
  60, which waits for the next start. A later revision sets
  `checkout.remote_timeout_ms` to 600,000 too: its live figure is taken
  at once, and `live_goals` still waits. Lowered to 30,000 later, it is
  taken at once again.
- **What a take reaches:** the engine composes the workers' figures
  again and gives them to its workers host (`figures.md`, 2), which
  sends them to every linked worker, and every worker takes its live
  figures at the next operation that reads them (`figures.md`, 3.2).
  When a take moved one of the link's timers the engine's workers host
  reads, it gives the host them too (5.2). A take moving no figure a
  worker or the host reads gives neither anything.
- **jig's kind holds no figure the core reads:** the core's limits are
  the application's to resolve at its start, so jig's kind is taken at
  the next start, all of it but its live figures (5.2), which only the
  workers and the engine's workers host read.
- **`limits-held` takes nothing at once:** while it's there, no figure
  is taken at once or live, as no raise is.

## 5. What is taken at once

### 5.1 A raise

| Limit | Kind | What holds a copy and takes the raise |
|---|---|---|
| `transcript_memory` | temper's | the root's limits and its environment, read by its budget at each preparation |
| `forge.tasks` | temper's | the forge child's tables of participating tasks, their names, proposed effects, releases and changes due, the closer's, the reconciliation's capacity and the landing's copy; the store's codec bound on a closer's selections |
| `forge.holds` | temper's | the forge child's holds, lost holds, makings and reds tables, the reconciliation's capacity and the landing's copy; the store's codec bounds on a tier row's goals and a hold's design writers |
| `forge.entries` | temper's | the forge child's outbox and flights, the outage's writes, the reconciliation's capacity and the landing's copy; the store's codec bound on a landing row's entries |
| `forge.client.entries` | temper's | the forge client's outbox and its alarms' capacity |

- **How:** the service raises the store's codec bounds, then gives the
  root the raised limits, which it takes into its own copy and its
  environment, and hands the forge child, which raises each table's
  ceiling and the forge client's. A ceiling is raised in place: skein's
  containers take a raise without allocating (`skein/docs/design/runtime/lib-containers.md`, 1).
  Then the work held on a raised limit goes on (`limit-settings.md`, section 6).

### 5.2 A live figure

- **What is live:** a time bound or a ceiling that every holder reads
  at each use, as an operation begins, and that sizes no table, no
  carrier and nothing a worker's link agrees at its opening or its hello
  states. A change of one, a raise or a lowering, is taken at once, and
  each holder's next operation runs under it; one in flight keeps what
  it began with (`figures.md`, 3.2). No restart is needed, and none of
  them cuts a run.
- **The live figures:**

  | Limit | Kind | Read at |
  |---|---|---|
  | `checkout.remote_timeout_ms` | temper's | each git operation of a worker's checkout that reaches the forge, its deadline set as it's sent |
  | `checkout.local_timeout_ms` | temper's | each local git operation of a checkout, likewise |
  | `checkout.reds_read_ms` | temper's | each read of a check's inherited reds, as it starts |
  | `checkout.rerun_processes` | temper's | each check's reruns, as they start |
  | `checkout.preparations` | temper's | each preparation's admission: a raise admits the next one waiting at the next admission |
  | `checkout.message_bytes` | temper's | each commit message a save or a delivery writes, and each preparation's check of it |
  | `git_component.stdout_bytes` | temper's | each git process, its captured output sized as it's spawned |
  | `git_component.stderr_bytes` | temper's | each git process, likewise |
  | `git_component.read_bytes` | temper's | each read of a workspace's file |
  | `git_component.sweep_entries` | temper's | each page of a workspace's sweep |
  | `git_component.sweep_bytes` | temper's | each page of a workspace's sweep |
  | `link.handshake_ms` | jig's | each handshake, on the worker's dial and at the engine's workers host |
  | `link.hello_ms` | jig's | the engine's workers host's wait for each opening's hello |
  | `link.stall_ms` | jig's | each stall deadline, on either side of a link |
  | `link.backoff_ms` | jig's | a worker's next redial wait |
  | `link.ceiling_ms` | jig's | a worker's next redial wait |
  | `link.proved_ms` | jig's | a worker's next proving of its link |
  | `link.busy_ms` | jig's | a worker's next span of *busy* refusals |
  | `hub.event_bytes` | jig's | each event of a run at a worker's hub |
  | `hub.accounts` | jig's | each assignment's accounts at a worker's hub |
  | `hub.turn_queue_bytes` | jig's | each turn a worker's hub queues |

- **Checked as a start checks them.** The engine's take is checked
  whole (section 4), and a worker checks the figures it takes by its
  start's checks with its pinned agent's (`figures.md`, 3.2), since a
  ceiling such as `checkout.message_bytes` or `hub.event_bytes` must
  still fit what the agent was built with.
- **How:** the service sets its figures in force and composes the
  workers' figures again (`figures.md`, 2). Where the take moved
  `link.handshake_ms`, `link.hello_ms` or `link.stall_ms`, it gives its
  workers host the three, `jig_protocol_engine_workers::Timers {
  handshake, hello, stall }` through `host::Workers::retime`, which its
  connections read at their next use (jig's
  `protocol/channel/figures.md`, 3). The engine holds no other copy of a
  live figure.
- **Not live, though a time bound:** `link.ping_ms` and
  `link.silence_ms`, which make the stop bound a worker's hello states
  and the engine checks against its grace (`../../draft/archive/redeploys.md`,
  5.3), and which both sides of a link run; and the engine's own time
  bounds, which its children copy at construction (section 11).

### 5.3 Every other limit

- **Every other limit is taken at the next start,** as
  `../../draft/config.md`, 11's default has it: a child that holds a
  copy and can't take a raise at once makes its limit "at the next
  start". A later milestone moves a limit to 5.1's list once every
  holder takes its raise, or to 5.2's once every holder reads it at each
  use.
- **Each limit's row says which:** each names table gives each name
  `applies`, jig's `jig_protocol_config::names::Applies`: `AtOnce` for
  5.1's, `Live` for 5.2's, `NextStart` for the rest. temper's table
  names jig's type, since both mean the same thing; jig's rows are
  `Live` or `NextStart`.

## 6. A limit reached

- **The hold names its setting.** A forge step refused on a count, its
  `RefusalCause::Limit`, holds its task with words of this form, the
  branch and repository it was working on before them as today:

  `forge.entries is 8, and this needs 9: a limit in temper-limits. Ask temper to raise it, or raise it from Needs you. This goes on by itself once room frees or the limit rises.`

  and no "restart, then release". Every forge step today refused with the
  bare `Refused` because of one of its limits (`limits.status.md`) is
  refused with `RefusalCause::Limit` naming it, and holds in these words.
- **The held work goes on by itself.** The forge keeps each task it held
  on a limit, with the limit, what it needed and its hold's revision,
  across a restart. It ends that hold, so the task's work retries as a
  person's release would, when:
  - room frees on that limit (an entry leaves the outbox, a hold's row
    goes, a task's name goes), the oldest first, as many as now fit;
  - the limit rises at once (section 5), every one waiting on it;
  - the engine starts, every one, since the start may have taken a
    raise.

  A task whose work needs more still is held again, with a new revision.
  A hold its task's person released or cancelled meanwhile is no longer
  kept.
- **jig's live tasks** name their setting: a run reads `live task limit:
  tasks.tasks is 768, a limit in limits; ask temper to raise it` and a
  client `Engine admission limit: live tasks (tasks.tasks, 768)`. A
  procedure waiting for room goes on when room frees, as today, and after
  the next start.

## 7. What a person sees

- **The Limits page** (the deployment's, `Address::Limits`) lists every
  limit, jig's and temper's together by name: what it bounds, its unit,
  its stock figure, its setting's figure and reason when one is over it,
  the figure in force, when a change applies (`Raises apply at once`,
  `Changes apply live` or `Raises apply at the next start`, from its
  row's `applies`, section 5.3), and "at the next start" when the
  setting differs from the figure in force. A row's `applies` reaches
  the page as the client wire's `LimitApplies`, which gains `live`, so
  `temper-client-wire` moves its version (`../wires.md`, 9). `transcript_memory`'s row shows the bytes
  the root holds against it. The settings page links to it. Its rows come
  from one read, `limits`, which the service answers from its names
  tables, the stock it resolved and the settings in force, a page of rows
  at a time.
- **Needs you offers the raise.** A held task's card whose hold is on a
  limit (section 6) shows the limit, its figure and what was needed,
  and, for an owner, the raise in one act: `Raise forge.entries to 16`,
  which sends a `ChangeSettings` of `temper-limits`, its list with that
  override set and the reason `raised for task N, which needed M`. The
  figure is the least of `limits.md`'s round form (d × 10ⁿ, 2ⁿ or
  3 × 2ⁿ) at least twice the limit's figure and at least what was
  needed. A person who is no owner reads that an owner can raise it, and
  can ask temper. The card reads the hold from one read, `limit-holds`,
  the forge's tasks held on a limit, answered by the root.
- **Ask temper** reads both kinds in `see settings` and on the
  `settings-fields` and `limits` reference pages, and proposes a change
  of either with `propose_settings`, for an owner to accept
  (`../../draft/ask-temper.md`, 5.2). Its instructions stop keeping
  limits from it; listeners and workers stay the operator's.
- **`temper ctl settings show`** prints both kinds' lists, and `settings
  apply` takes items of `limits` and `temper-limits`.
- **The reference pages** `limits` and `limits-engine` give each limit
  its setting's kind and when a change applies, in their Applies column:
  `at once` for a raise, `live`, or `next start`, from its row; `settings-fields` gives both kinds' items; `configuration` and
  `configuration-engine` give the moved members' rows (`moved.md`).

## 8. The documents

- **Refused by name** (`moved.md`, section 3): in the engine's and the
  box's documents, `engine.limits`, `engine.memory_bytes` and
  `application.limits`; in every document, a worker's `limits`, which
  come from its engine, and `memory_bytes`, which nothing read. jig's
  reader takes `limits` and `memory_bytes` as
  optional members (jig's `protocol/configuration.md`), so that temper
  refuses them with its own words.
- **`temper-box init`** writes none of them, and the examples hold none.
- **A test that needs other figures** gives them to the start as its
  world's typed limits, as `temper-limits-fixtures` gives `small`, or as
  settings its world makes; never in a document (`moved.md`, section 2).
- **dev.fabrig.ai** moves by its operators' steps
  (`../../draft/config.execution.md`, N2).

## 9. Tests

Focused and fuzzy, in the crates that hold each piece:

- **The kinds:** an override list's shape, each fault by position; jig's
  kind set, reset and shown, kept across a restart; temper's kind 22
  judged and kept.
- **The start:** a store holding each kind's change starts with its
  figures; a first start with none takes the stock; a name a release
  dropped is left out, said once; a setting a check refuses, and a
  lowering past what's held, refuse the start naming the setting;
  `limits-held` starts it, and a bad line refuses naming it; the engine's
  and the box's documents' moved members are refused by name.
- **Entry:** a party's change and a run's proposal past a check are
  refused naming the figure and what fails; a name of the other kind's
  is refused naming its kind.
- **At once:** in a world, a task held on `forge.entries`,
  `forge.holds` or `forge.tasks` goes on once a raise is accepted, with
  no restart; a lowering is taken at the next start; a change moving one
  figure outside 5.1 and 5.2 waits for the next start whole.
- **Live,** focused, in the engine's service: a durable revision
  raising or lowering `checkout.remote_timeout_ms` is taken at once,
  whole, and its figure is in force; one moving it with `live_goals`
  takes the timeout alone and leaves `live_goals` waiting, and the
  Limits page shows it so; one whose live take its checks refuse takes
  nothing; a take moving `link.stall_ms` gives the workers host its
  timers; with `limits-held`, nothing is taken. The rows: each names
  table's `Live` rows are 5.2's names, and its `AtOnce` rows 5.1's. A
  worker's side is `figures.md`, 9.
- **A limit reached:** the hold's words name the setting; room freeing
  ends the oldest hold; a restart ends them all; the Needs-you card's
  raise is the round figure, and its act sends the list with the
  override set.
- **The store for 100 live goals** (section 3), focused: a `live_goals`
  of 100 resolves, at the start and at entry, and its largest records,
  laid out at its resolved figures, fit the stock store; 101 is refused
  in section 3's words at both; a count overridden past what 100 live
  goals scale to is refused naming its setting and the store's figure;
  the store's figures are the same at 20, 40 and 100 live goals, and its
  value bound is the largest record's at 100.
- **Memory at rest** (section 3), focused, under skein's counting
  allocator (`skein-heap`): the engine brought to rest over a store in a
  directory, and the heap it holds measured three ways, its store laid
  out for 40 live goals, as before this section; for 100, the stock's;
  and for 100 with a `live_goals` of 100 taken at its start. Each is
  within 38,000,000 bytes, what dev.fabrig.ai's whole engine held at rest
  (`../../draft/config.md`, section 2), the store for 100's within 1% of
  the store for 40's, and the test's ceiling is the largest measured
  figure plus a quarter, rounded up to two significant digits
  (`skein/docs/foundation/testing-strategy.md`, section 6), the measured
  figures in its comment.

## 10. Where the code is

These names cross changes:

- jig's `jig-domain-core-vocabulary`, `settings`: `Override { name,
  value, reason }`, `OVERRIDE_NAME_BYTES`, `OVERRIDE_REASON_BYTES`,
  `OVERRIDES`, `check_overrides`.
- jig's `Setting::Limits` and `Value::Limits(Box<[Override]>)`, and
  `jig_protocol_config::with_overrides`.
- temper's kind 22: `settings::Kind::Limits` in
  `temper-engine-domain-forge-vocabulary`, its value a list of
  `Override`; its settings file's name `temper-limits`.
- temper's names table: the rows `live_goals`, `tool_payload` and
  `transcript_memory`, and each row's `applies`; the root's
  `transcript_memory`, formerly `memory_bytes`.
- jig's `jig_protocol_config::names::Applies { AtOnce, Live, NextStart }`
  and its `Name.applies`; temper's `limits::Applies` is jig's, named
  from it (section 5.3).
- jig's `jig_protocol_engine_workers::Timers { handshake, hello, stall }`
  and `host::Workers::retime(&mut self, timers: Timers) -> bool`, and
  the worker link's `jig_protocol_worker_link::Timers { handshake,
  stall, backoff, ceiling, proved, busy }` and
  `Component::retime(&mut self, timers: Timers) -> bool`: each true when
  taken, false when its limits' check refuses them, leaving it as it
  was (jig's `protocol/channel/figures.md`, 3 and 4).
- `temper_engine_service_vocabulary::figures::Composer::take_limits`
  (`figures.md`, 2).
- The client wire's `LimitApplies::Live` and the page's word `Changes
  apply live`.
- `temper_service_core::derived::STORE_LIVE_GOALS`, 100, a `u32` beside
  `LIVE_GOALS`, and the store's base the start lays the store out over
  (section 3), the one every path that sizes the deployment's store
  takes.
- The service's limit settings in force: the two lists and their
  revision, resolved by the one function the start and the entry check
  run.
- The root's word to the service of a durable revision naming either
  kind, and its input taking raised limits at once.
- temper's client wire's reads `limits` and `limit-holds`.

## 11. Open questions

- **jig's limits at once.** None of jig's limits takes a raise at once:
  its children size their tables at construction. Default: `tasks.tasks`
  first, with the tasks child's and its readers' tables raised in place.
- **The engine's own time bounds.** A time bound the engine's children
  copy at construction, a forge client's or a sign-in's, waits for the
  next start; the workers' and the link's are live (5.2). Default: each
  moves to 5.2's list once its holders read it from their limits at each
  use.
- **Ping and silence live.** `link.ping_ms` and `link.silence_ms` wait
  for the next start, since the stop bound a worker's hello states
  follows them (5.2). Default: they stay, until a worker can state a new
  stop bound to its engine without a new opening.
- **A start that finds what's held past a lowered figure** refuses, as a
  document's lowered limit did. Default: stays, with `limits-held`.
