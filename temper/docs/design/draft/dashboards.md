# Dashboards made by talking to temper

Draft, 2026-10-27, from the user's idea of 10-26: "something similar to
what we have for configuration.md … but for the UI, for example you ask
an agent that you want a dashboard like the one I pointed [at] and it
configures temper to show that dashboard in the client. Sort of "UI as
data" approach with some flexible palette of options/widgets can be
used to expose engine data (possibly with custom calculations. LUA?)".
The page pointed at is https://status.fabrig.ai, which agents'
`scripts/devel/status-page` writes (agents' `docs/design/status-page.md`).
It takes `configuring.md` from the settings to the client's pages, on
the settings jig has since designed (jig's `domain/engine.md`, section
17). It crosses jig and temper on purpose; an owner splits it. Nothing
here is adopted, and nothing here is a part of v0.1.0 (`v0.1.0.md`).

## 1. In one page

- **The person describes a page; temper shows it.** "A page like
  status.fabrig.ai for my deployment", in their words or by pointing at
  one. The configuring agent asks what matters, then proposes a
  dashboard; an owner accepts it on the review page, and it is one of
  the client's pages from then on (section 6).
- **A dashboard is a setting.** A kind of temper's, defaulted, changed,
  proposed and decided as every setting is (jig's `domain/engine.md`,
  17.5), each change a revision with its history. No new process: the
  proposal card, the review page and "How work runs" are
  `configuring.md`'s (`../client/README.md`, 5.13).
- **Three layers:** *facts*, what the engine and its connectors already
  keep (section 3); *metrics*, named calculations over one owner's
  facts, settings too (section 4); *panels*, a dashboard's parts from a
  closed palette, each showing metrics (section 5).
- **The hard part is the numbers, not the panels.** agents'
  `status-page` spends more than half of its two thousand lines on
  twelve sources and their medians, p90s, rates and paces; its layout
  is the easy part.
  jig's client reads windows of tasks by a fixed set of keys, and
  nothing aggregates (jig's `client/domain/contract.md`, 6.1): a client
  cannot compute "p90 of gate time per repository, the last week" from
  a window, and should not try.
- **A metric is its facts' owner's.** jig computes those over tasks,
  runs, spend, people's waits, accounts and hosts; the forge those over
  changes, landings and gates. Each folds a fact in in the decision that
  makes it, into fixed buckets, so a read is never a scan (4.2).
- **Bounded and counted.** A metric is buckets and bins of a fixed
  number; its bytes are known when it is proposed, and the review page
  says them. Dashboards, panels and metrics are limits (section 8).
- **No language at first** (section 7). A metric is one of a closed set
  of operations, which the agent says back in words and an owner
  understands before accepting. An operation two dashboards need joins
  the set; a sandbox for code of an agent's comes only on evidence.
- **Semantic, not a layout.** Panels become skein-ui's tree in
  `temper-client-view`, so the terminal and the browser show the same
  dashboard; a dashboard says what each part is and their order, never
  where it goes or how big (skein's `ui.md`, section 1). Every chart has
  its words.
- **Read-only, and never wider.** A panel leads to temper's own pages
  and acts on nothing; a dashboard shows only what the viewer's role in
  its scope may read, judged by the metrics' owners (4.3).
- **Its test:** given agents' `status-page.md`, the agent proposes a
  dashboard; what it cannot show is the list of facts temper does not
  keep (section 9).

## 2. Where it fits

- **On jig's settings.** jig's `domain/engine.md`, section 17: a
  setting is a kind and a number, an application's kind kept and judged
  by the connector that owns it (17.5); a change is one decision, made
  against a revision and checked whole (17.2); a run whose charter gives
  it the settings family proposes a set (17.6). A dashboard and the
  metrics it shows are items of one set, accepted or refused together.
- **On temper's kinds.** temper declares three kinds today, all the
  forge's (`../protocol/configuration.md`, 13.2). This draft adds a
  dashboard kind, scoped to the deployment or to each project, owned by
  a small child of temper's root, the dashboards child, and the forge's
  metrics kind (section 4). jig adds a metric kind of its own.
- **On jig's watches.** A metric's values are its owner's subject (jig's
  `client/domain/contract.md`, 7.2): named by its owner, its scope and
  its window, judged by its owner, marked as its owner folds facts in,
  and level-triggered, so a busy metric never floods a client: the view
  reads the latest when it is ready.
- **Not on streams.** An application's streams (7.4) carry what is not
  committed state, marked provisional, and are designed but not built
  (jig's `STATUS.md`). Metrics are derived from committed facts in the
  decisions that commit them, so they are documents. A stream is for a
  live gauge the engine does not commit, a host's load if hosts ever
  report it (section 9).

## 3. Facts

What the engine keeps today, as the designs say. A metric adds no fact:
a dashboard that needs one the engine does not keep needs it designed
by its owner first, and the agent says so (6.3).

| Owner | Facts kept | Where |
|---|---|---|
| jig's tasks | each task's project, label, executor kind, charter and its revision, model and effort; its phase; its times (made, changed, ended, last activity); its outcome or failure's reason; a goal is a tracked task | jig's `domain/tasks.md`, sections 3 and 5.1; `domain/core.md`, 6.2; `domain/engine.md`, 17.3 |
| jig's runs | each run begun and ended, in its task's activity; each turn's spend | jig's `domain/engine.md`, 7.2 |
| jig's spend | each funder's spend per period, against its budget | jig's `domain/authority.md`, section 7 |
| jig's people | what waits on each party: questions, proposals, escalations and person tasks, counted exactly | jig's `client/domain/contract.md`, 5.1 (`me`); `domain/people.md`, section 6 |
| jig's accounts | each source's state; *limited*, with its reset where the provider says it | jig's `domain/accounts.md`, 3.5 |
| jig's hosts | connected or lost, slots used and free, the runs each hosts | jig's `domain/hosts.md`, section 2; `client/domain/pages.md`, section 7 |
| temper's forge | each change and its result; the merges temper made, `Landed`, without a time; gates as tasks; CI at a change's head, in memory | `../domain/forge.md`, 8.2, 8.3 and 8.8; `../protocol/store.md`, section 2 |

- **Not kept:** token counts as numbers (smith's turn records them,
  smith's `protocol/transcript.md`, section 2, and jig's core reads only
  a turn's size, its spend, what it took and whether it opens a window,
  jig's `domain/engine.md`, 7.2); a host's load, memory or disk; how much
  of a provider's plan is used, beyond limited; a landing's own time
  (its change's ended time stands in); CI's steps, and CI across a
  restart, which forgets it.
- **Ended tasks are history enough.** They are kept to the deployment's
  retention horizon, whole while none is set (jig's `protocol/store.md`,
  11.1). A metric outlives the tasks it counted: that is what it is for.

## 4. Metrics

### 4.1 What a metric is

- **A setting of its owner's kind,** in the deployment's scope or a
  project's, with a name, and:
  - *its source:* one of its owner's kinds of fact, either events (a
    task ended, a run ended, a turn's spend, a change landed or ended
    otherwise, a gate ended) or live state (tasks by phase, what waits
    on people, hosts' slots, accounts by state, spend this period);
  - *its filter:* any conjunction of the source's keys, a fixed set per
    source, as a task query's are (jig's `client/domain/contract.md`,
    6.1): project, charter, model, effort, executor kind, label and
    outcome for a task; repository and outcome for a change;
  - *its group:* at most one key, its groups at most a bound, the rest
    counted as "other";
  - *its operation,* from 4.4; and *its window,* a number of buckets of
    a length (24 of an hour, 42 of four hours, 7 of a day).
- **Two shapes.** A *gauge* is live state counted now, from what its
  owner already counts, with no buckets; it may keep its value at each
  bucket's end, as a series of its own. A *series* is events folded into
  time buckets by their decision's time, on the engine's clock.
- **A project's metric counts that project's facts;** the deployment's
  counts all of them.

### 4.2 Kept by its owner

- **Folded in the decision that makes the fact.** When a task ends, the
  tasks child adds it to every metric whose source it is and whose
  filter it matches, in the same decision; the forge does the same when
  a change lands. Matching is bounded by the metrics per source, each a
  fixed set of keys, as marking a task query's watch is (jig's
  `client/domain/contract.md`, 7.2).
- **One shape, owned per owner.** jig gives the metric's record, its
  operations and its folding as one library; jig's tasks child folds
  jig's facts, and a connector, the forge here, folds its own with the
  same code.
- **In the owner's family,** written in the decision that folds the
  fact, so that a restart restores a metric as it restores any record.
  A bucket past the window is dropped in the decision that opens the
  next.
- **From its acceptance on.** A new metric starts empty, and its panel
  says "since 14:02". Filling it from the ended indexes is open (section
  11): they serve tasks by requester and by project, and lose the
  forgotten ones, so it would be partial.
- **No content.** Counts, sums and bins: no names, no text, no task's
  words. A metric grouped by charter names charters by number; the
  client words them.

### 4.3 Read by the client

- **A subject of its owner's,** a metric's values over its window, its
  groups and its buckets, bounded by its own bounds, so that every
  document fits the client's limits (jig's `client/domain/contract.md`,
  7.5).
- **Admitted on the scope:** a project's metrics to a party with a role
  in that project, the deployment's to one with a role in the
  deployment, as the settings themselves are shown
  (`../client/README.md`, 5.13). The owner judges what the party may see,
  and the client hides nothing of its own.

### 4.4 The operations

A closed set, each said in words on the review page and in a panel's
caption:

| Operation | Of | Said as |
|---|---|---|
| count | events, or live state now | "Landings, the last 24 h" |
| sum | a value: spend, a duration | "Spend this week" |
| rate | a count over its window, per hour or per day | "Landings per hour" |
| ratio | two counts of one source, its filter and a narrower one | "Gates failed, of all gates" |
| latest, oldest | an event's time | "Last landing: 3 min ago" |
| mean, minimum, maximum | a value | "Mean run time" |
| quantile | a value, at any fraction | "p90 of gate time", "median wait for a host" |
| projection | a sum or a gauge against a bound: a budget, a limit | "At this pace the week's budget runs out on Thursday" |

- **Durations** are an event's own spans, from the facts' times: a task
  made to ended, a run begun to ended, a change made to landed, a run's
  wait for a host until its placement.
- **Quantiles are about.** A value is counted into a fixed number of
  bins spaced by ratio, 128 by default, so that a quantile is within
  about 5% of the true value; a window's quantile merges its buckets'
  bins. The panel says "about 4 min", never a precision it lacks.
- **The projection is the status page's pace** (agents'
  `docs/design/status-page.md`, 2.2): the rise over the look-back, then
  either full before the period's end, at when, or what is left at its
  end.

## 5. Dashboards

### 5.1 The value

- **A record of the dashboards child,** in temper's codec, bounded by
  its kind's largest bytes: its name and line; its scope, the
  deployment or a project; then its sections in order, each a heading
  and its panels in order.
- **A panel** is its kind from the palette (5.2), its title, the metrics
  it shows, each by owner, kind and number, and its options, a fixed
  set per kind: a unit, a sort, a threshold that turns it to a warning,
  a projection's bound, and where it leads.
- **Order and grouping, never position.** A dashboard says what each
  part is and which come together. Each shell lays them out: the
  browser in columns where it has room, a phone in one, the terminal in
  lines.
- **It names metrics,** and a change that would remove a metric a
  dashboard shows is refused, naming the dashboard, as one that would
  leave a template's charter unoffered is (jig's `domain/engine.md`,
  17.5, "Charters a value names"). jig's check of references widens from
  charters to settings of any kind: a jig change.

### 5.2 The palette

Each panel is nodes of skein-ui's closed roles (skein's `ui.md`, 3.1),
drawn by `temper-client-view`:

| Panel | Shows | In skein-ui |
|---|---|---|
| tile | one number and a line under it: "Missions 26 · 18 active, 8 paused" | a region: its heading, its text |
| table | a grouped metric's rows, a column per metric | a table, its column headers the metrics' names |
| chart | a series over its window | a drawing, its words a sentence of its extremes and its last bucket, with the table beside it in a disclosure, "As a table" |
| use | a sum or a gauge against its bound, with its projection: "63% of the week's budget · runs out Thursday" | text; a drawing of the bar where a shell draws |
| cards | a tile per group: a host, a project, an account | a list, an item per group |
| lines | a task query's live tasks, as the board's lines (jig's `client/domain/contract.md`, 6.1) | a list of temper's task lines, each leading to its task's page |
| warning | a threshold crossed: "hetzner3: no free slots for 20 min" | an alert, said at once |
| link | one of temper's pages: the inbox, a project's board, the changes, Accounts, How work runs | a link |

- **It grows by skein-ui's rule:** a panel two dashboards need, which
  no panel shows, joins the palette by a temper change. No agent makes
  a panel; it arranges them.
- **Nothing said by style alone.** A warning is an alert in words, a
  chart's colour is a token the shell may ignore, and the fake person
  reads every panel by role, name and text (skein's `ui.md`, section 17).

### 5.3 In the client

- **"Dashboards" in the navigation,** after "How work runs", whatever
  the project (`../client/README.md`, 5.3): the deployment's dashboards
  the person may read, then the chosen project's. Each opens on its own
  page, `Dashboard { scope, number }`.
- **Live.** A dashboard watches its metrics, and the view builds the
  next tree when one moves; no polling, no "refreshes every 10
  minutes". The terminal prints a dashboard as it prints any page
  (`../client/README.md`, 6.3).
- **Times** are `../client/README.md`'s, 5.12; a metric younger than its
  window says since when.
- **A dashboard's acts** are its links alone. A held task is answered
  on its task's page, an account fixed on Accounts: the confirmations,
  and the revision a person was shown, are those pages' (3.6 there).

## 6. The person's way

### 6.1 Asking

- **In the configuring chat** (`configuring.md`, section 6), with its
  offer on the Dashboards page as on How work runs: "Tell temper what
  you want to see". Its input is the conversation, pasted text, or a
  document in an adopted repository read by its path:
  `status-page.md`, in the test.
- **Its tools grow by three reads:** the facts' catalogue, each owner's
  sources with their keys and operations, from the documentation in the
  binary (`configuring.md`, section 7); the dashboards and metrics as
  they are; and a metric's values, as a person in the same role would
  see them. It proposes through the same `propose` call.
- **It asks** what to watch, for whom (the scope) and over what span,
  then posts a card: "Proposal 14 · Dashboard "Fleet" (new) · 7 tiles,
  3 tables, 2 charts · 9 metrics, 5 of them new", and "Review".

### 6.2 The review

The review page of `../client/README.md`, 5.13, with the dashboard's
own blocks:

```
Proposal 14 · from "A page like the status page" · against revision 9

temper shows
  Fleet (new)              the deployment's · 4 sections, 12 panels
  [Preview]
temper computes
  Gates, p90 by repository  new · the last 7 days, by 4 h · about 170 KiB
  Landings per hour         new · the last 24 h, by 1 h
  Spend this week           already counted, for "Budgets"
  …

Effect
  New metrics count from the moment it is accepted: the last 7 days
  fill in over the week.
  Who sees it: people with a role in the deployment.

[Show the settings]            [Reject…]  [Change something]  [Accept]
```

- **The preview** is the dashboard drawn on the page, its metrics that
  exist with their values and the new ones as "starts when accepted".
- **No "your agents follow".** A dashboard is structure alone; it
  changes nothing an agent reads.
- **Running work** is untouched, and the page offers no choice of it.

### 6.3 What it cannot do

- **What temper does not keep,** it says, with the nearest thing:
  "temper doesn't know your machines' load; I can show each host's free
  slots and how long runs wait for one"; "temper counts spend, not
  tokens"; "p90 of gate time counts from today, not for the past".
- **Each such answer is design input** for the fact's owner, as
  `configuring.md`'s gaps are (its 5.3).

## 7. No language at first

The user asked about custom calculations, perhaps in Lua. The answer
here is a closed set of operations first, for four reasons:

- **The status page fits them.** Every number on status.fabrig.ai is a
  count, a sum, a rate, a ratio, a quantile, a latest or a projection
  (section 9).
- **An owner accepts what they understand.** "p90 of gate time per
  repository, the last 7 days" can be said back on the review page;
  the meaning of a script cannot, which is `configuring.md`'s premise.
- **Bounded and counted** (skein's `programming-model.md`): a metric's
  bytes and its cost in a decision are known before it is accepted. A
  script folded into the engine's decisions is computation that no
  bound prices unless it is metered.
- **The palette is the usual limit,** not the calculation: a result
  still has to be a tile, a table or a chart.

When a dashboard needs what the set cannot say, it grows by one
operation, a jig change, generic for every application. Only if such
requests keep coming does a sandbox follow: metered and capped in
memory, run by the metric's owner within its decision's budget, over
the same facts and buckets, its code shown folded on the review page
beside the agent's words. wasm with a fuel meter is the likely choice
over an embedded Lua: one toolchain, and temper already builds for
wasm32 (`browser-playground.md`). Default: none.

## 8. Bounds

- **Limits, priced at the start** by jig's worst case, as every other
  limit is: dashboards per scope, panels per dashboard, metrics per
  owner and scope, groups per metric, buckets per metric, and bins per
  quantile's bucket, fixed.
- **A metric's bytes** are its groups × its buckets × what a bucket
  holds: a counter or a sum, 8 bytes; a quantile's bins, 128 × 4 bytes.
  "Gates, p90 by repository, the last 7 days by 4 h", 8 repositories ×
  42 buckets × 512 bytes, is about 170 KiB, and the review page says so.
- **A decision's cost** is the metrics its facts match, bounded by the
  metrics per source; the rest of the decision is unchanged.
- **The numbers** come from the first deployments, as the profiles'
  and `configuring.md`'s do (its section 10).

## 9. Its test

- **In the gate,** real-loop stories on the scripted fake LLM: a
  person asks for a dashboard and accepts it; its numbers move as fake
  work ends; a metric removed while a dashboard shows it is refused; a
  restart keeps the buckets; a member without a role in the deployment
  is refused its dashboards; the fake person reads a chart's words.
- **The status page as a golden.** agents' `status-page.md` goes through
  a real model as the person's description, and the proposal is scored
  against a dashboard made by hand, within the gaps below. A lab
  experiment, run when the agent's instructions or the documentation
  change, not in the gate (`configuring.md`, section 8).
- **What it maps to:**

  | The status page | temper's dashboard | Gap |
  |---|---|---|
  | missions: active, paused, ended this week | goals by phase; ended goals by project | none |
  | what each mission runs now, its last activity | live tasks by charter under each goal; a task line's last activity | none |
  | agents by role, busy | live agent tasks by charter and phase class | none |
  | waiting on the user | what waits on people, from `me`'s counts | the forge's issues and their labels are not read back |
  | the director's post, 24 h | messages to a party, by kind of sender | a source of jig's to add: inbox entries made |
  | landings per hour, recent outcomes | changes ended, by outcome; a rate | a landing's own time: its change's ended time stands in |
  | land gates: count, failures, median, p90, by repository | gate tasks ended: count, ratio failed, quantiles, by repository | none |
  | gate steps (fmt, clippy, tests) | none | CI's steps are not kept, and CI is forgotten at a restart |
  | builds by kind, heavy's queue | runs' wait for a host slot | heavy is the factory's, not temper's |
  | machines: load, memory, disk | hosts: connected, slots used and free | hosts report no load, memory or disk |
  | implementer runs by model and effort: duration | runs ended by model and effort: count, quantiles | none |
  | their tokens and cache hits | none | token counts stay in smith's turns |
  | subscriptions: use and pace | budgets: spend against each period's budget, its projection; accounts by state | a provider's plan use, beyond limited |

- **The gaps are the experiment's other output:** each a fact to design
  by its owner, or to leave out, as `temper-on-temper.md`'s 6.8 is for
  `configuring.md`.

## 10. Order

1. **The design in jig:** metrics over its facts (sources, keys,
   operations, buckets, its kind, its subject), the library a connector
   folds with, and the check of references widened to settings. Then
   temper's: the forge's metrics, the dashboard kind and its child.
2. **Mockups:** the Dashboards page, a dashboard of the status page's
   panels, and the review with its preview.
3. **The code,** jig's, then temper's: the metrics, the dashboards
   child, the panels in `temper-client-view`.
4. **The agent's reads,** its stories, then the golden and the
   experiment.

## 11. Open questions

- **A party's own dashboards.** Settings are the deployment's or a
  project's (jig's `domain/engine.md`, 17.1); a person's own would be an
  application's records in the party's own scope (jig's
  `client/domain/contract.md`, 7.5). Default: the two scopes only.
- **Filling a new metric from history,** from the ended indexes, paged
  and partial (4.2). Default: none; a metric counts from its acceptance.
- **The store's cost** of writing metrics in every decision that folds
  a fact, against buckets kept in memory and written as each closes,
  the open one lost at a crash. Default: in the decision.
- **Windows past a week,** for a month's trends. Default: a week; longer
  by coarser buckets, within the same bound.
- **Public dashboards,** a page for anyone as status.fabrig.ai is,
  holding no names. Default: later, the public-reading draft's question
  (on its branch `draft-public-reading`): a printed page of a dashboard.
- **Warnings that reach a person,** a threshold crossed as an entry in
  an inbox. Default: later; a warning panel only.
- **A run's own dashboard,** proposed by an agent for the goal it owns
  (jig's `domain/engine.md`, 17.6). Default: later; the configuring
  chat only.
- **Hand edits** of a dashboard, by an owner who also decides it.
  Default: proposals only, as `configuring.md`'s (its section 10).
- **Dashboards in jig.** The kind, the palette and the panels start in
  temper and move to jig when a second application wants them, with
  jig's reference UI showing them. Default: temper's.
