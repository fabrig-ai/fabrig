# Limits

## 1. Stock and sizing

```
"limits": {"profile": "small", "overrides": {"<name>": value}}
```

- **Every limit temper's own domains and components take:** the forge's
  and its change and issue policies, the queue window, the forge
  client's, the checkout's, the root's own beyond jig's core, the forge
  component's, its webhooks' listener's and its GitHub App's among them,
  the git component's, the engine's routing's records and forges, the
  setup host's, the clients' listener's forge routes' and the picker's;
  and the box's, which `temper-box serve` checks and takes none of but
  `box.line_bytes`, its pipe's (../setup/box.md).
- **One stock** (`../../draft/config.md`, 5.1): temper's figures are
  one set, the stock, which is jig's stock with temper's figures over it,
  and a document chooses no profile. The engine's and the box's limits
  are two kinds of settings over the stock (`limit-settings.md`), and
  their documents refuse `limits` by name (`moved.md`). A worker alone's
  `limits.profile` may be absent or `standard`, and either reads the
  stock. Any other name,
  `small` among them, refuses the start, naming it. `small`'s figures
  are a test's, in the fixtures crate `temper-limits-fixtures` (jig's
  are in `jig-protocol-config-fixtures`), and never a document's. Where
  this document gives `standard`'s figure, that is the stock's, and
  `small`'s is the fixtures'. `temper-box init`'s
  worker part names `standard` (`../setup/init.md`, 3). Their numbers are
  constants in `temper-service`'s limits module, and the setup's and
  the box's in their own crates, each documented with what it bounds;
  the design names limits and what they bound (jig's
  `protocol/limits.md`), and the figures the draft
  sized in 13.4. The stock fit below states the figures that must agree
  across the agent and its carriers. `standard` is `small` with the
  forge's work at that scale: `forge.tasks`, `forge.holds` and
  `forge.changes` 128, the forge client's `resources` 128 (at least the
  holds; it bounds the branch heads too, `../../domain/forge/keeping-up.md`,
  1), `forge.landings` 32, a change's policy's `questions`, `tasks`,
  `repairs`, `resolutions` and `updates` 80 (its `reruns`, the times a
  job the runner failed is run again at one head, is 1 in both,
  `forge.change_policy.reruns`: `../../domain/forge/change/README.md`, 2,
  step 3), an issue's `plan_items`
  384, and the brief's task section a whole spec of jig's `standard`
  (131,072 bytes) with its contract's and requester's lines; what the
  start's checks tie to these moves with them, the least that passes.
  small's brief admits 14,592 body bytes, with 4,608 for reviews
  (`root.brief.budgets.reviews`); in each profile
  `root.brief.brief_bytes` and `root.journal.result_bytes` are equal,
  so gathering and journalling hold the same brief. The stock host part
  holds the authorised brief whole at `temper-box init`'s models
  (`../../domain/agent/room.md`, 1); trimming is the fallback. Where a limit
  of temper's is a quantity jig's profile also sets, its constant cites
  jig's profile's.
- **Sized from what** (`../../draft/config.md`, 5.2):
  - **Counts:** a count (tasks, holds, inboxes, charters, executors,
    changes, subscriptions, resources, repositories, projects, workers,
    slots, client streams) is twice dev.fabrig.ai's figure of 2026-11-16
    where that is above `standard`'s. It's rounded up to the least number
    of the form d × 10ⁿ, 2ⁿ or 3 × 2ⁿ (34 → 40, 768 → 768, 758 → 768).
    Elsewhere it stays at `standard`'s. A count a wire caps below that
    is the cap, never below dev's: `forge.landing_members` is 5, the
    worker wire's, not twice dev's 4. `forge.changes` is 768, twice
    dev's 379 rounded, its store raised for it: the tier rows hold 1,537
    landings, since a failing test's id is within 1,024 bytes (below).
  - **A failing test's id:** `forge.tiers.test_id_bytes` is 512, twice
    the longest id today (about 221 bytes, a fuzzy case's), the most a
    finding's signature holds of an id (`../../domain/findings.md`,
    section 4), within the store's 1,024 (below).
  - **A tier's failing tests:** `forge.tiers.failing_tests` is 64, the
    most ids a finding's signature holds (`../../domain/findings.md`,
    section 4), so that a run's failures are named by test as far as a
    red finding can take them whole
    (`../../draft/after-landing-repairs.md`, section 4): dev's main tier
    failed 28 to 48 tests a run on 2026-11-17 and 18. It bounds each
    list of failing tests it bounds in the stock, a tier's record's
    failures and flakes, a gate's selection and a receipt's among them
    (`../../domain/after-landing-runs.md`, section 3, and `../../domain/after-landing-storage.md`, section 1). `small`'s
    figure, a test's, stays 16.
  - **Times:** a time is dev's where that is above `standard`'s: the
    brief's gathering 300 s and the checkout's local timeout 60 s, for
    example.
  - **Other figures:** a figure with no source is dev's where that is
    above `standard`'s. A figure with a source is derived
    (`derived.md`).
  - **The sign-in's lifetime stays a day.** dev's seven days serves its
    automations, and dev keeps it as an override.
  - **Each constant's comment says what it was sized from:** a figure
    of dev's, a formula or a design's bound. A constant whose comment
    cites a test world is sized again from what it bounds at the
    stock's counts. What the start's checks tie to a figure moves with
    it, by the least that passes.
- **The stock covers fabrig's development** (`../../draft/config.md`,
  5.2):
  - **The deployment figures** are dev.fabrig.ai's documents' overrides
    of 2026-11-16, named as fabrig's own development. They're recorded in
    `temper-engine-service-vocabulary`'s limits module, each as its
    document (the engine's or a worker's), its dotted name and its
    value.
  - **A focused test** resolves the stock as each document resolves it
    with no override, the stock agent's derivations included
    (`derived.md`, section 3). It holds every stock figure at or above
    its recorded figure, but for the exceptions it names, each with its
    reason: the sign-in's lifetime (above), and a worker's figures that
    derive from its `slots` (`derived.md`, 5.2), which are held at dev's
    `slots` instead.
- **The fixtures' `small` starts as it stands** against an agent of its size:
  smith's, whose run bounds (its directories, their names,
  its accounts) are smith's own and no configuration changes (smith's
  `smith-domain-run`, `derive`; `smith-agent-service`), and whose largest
  call, its run's `outcome_bytes`, follows from its agent configuration.
  The owned example's agent is `standard`'s (../setup/init.md, 1): its 48
  calls pass `small`'s `hub.run_calls`, 8. Every constant passes the
  checks below with an agent of `small`'s size: the checkout's
  repositories within the agent's directories, the checkout's names
  within theirs, and a run's push message, which smith bounds as a call,
  within the checkout's commit message. So `small` takes a forge's `git`
  base of at most 122 bytes, the longest remote's bound (below). A test
  whose agent has other bounds states the overrides it needs.
## 2. Project capacities

- **A project's decided rows:** `forge.decided_per_project`, the rows of
  what was decided for its people and of what asked them, which the forge
  keeps 7 days (`../../domain/forge/attention.md`, 5): 1,024 in both profiles, so a
  busy night's every task fits beside the day before. The forge's family
  holds it times `forge.judge_projects`.
- **A project's follows:** `forge.follows` per project, 2 in `small` and
  8 in `standard`; `forge.follow_queue`, 16 and 64; and, in both,
  `forge.follow_paths` 8, `forge.follow_merge` 8, `forge.follow_commits`
  8, `forge.follow_files` 32, `forge.follow_message_bytes` 512,
  `forge.follow_skips` 8 and `forge.follow_reason_bytes` 256, as
  `../../domain/forge/follows.md`, section 10 bounds them.
- **A project's dependencies:** `forge.dependencies` per project, 4 in
  `small` and 16 in `standard`, kind 13's numbers and the most the
  `follows` document carries (`../../domain/forge/dependencies.md`,
  section 2).
- **A project's findings:** `forge.findings_per_project`, the findings
  the forge keeps for a project (`../../domain/findings.md`, section 9):
  256 in both profiles, room for weeks of reds, words and outside
  parties' filings between triages. The forge's family holds it times
  `forge.judge_projects`.
- **A goal's reds:** `forge.branch_reds`, the reds a goal holds at
  once, 64 in both profiles, at most 256; `forge.red_key_bytes`, a
  red's key, 256 in both, at most 512 (`../../domain/forge/reds.md`,
  2 and 3), so that the engine's answer to a worker's ask, at most
  `forge.branch_reds` keys of `forge.red_key_bytes` with their heads,
  stays far within the channel's frame (`../../domain/forge/reds.md`,
  5.4). The forge's families hold `forge.branch_reds` times
  `forge.holds` reds and `forge.branch_reds` times `forge.changes` passes
  (`../../domain/forge/reds.md`, 6). `checkout.reds_read_ms`, how long
  the worker waits for the engine's answer at a check's start, 5,000 in
  both, from 100 to 60,000 (`../../domain/forge/reds.md`, 5.3): the
  worker's, since the worker waits, though the draft names it among the
  forge's. `checkout.rerun_processes`, the most failing tests' processes
  a check runs again after its run, which the worker writes to
  `temper/rerun`, 4 in both, from 0 to 16, 16 a report's most flakes
  (`../../domain/forge/reds.md`, 5.2 and 5.6): the worker's too, for the
  same reason.
- **A project's landing history:** `forge.history_days`, 14 in both
  profiles, from 1 to 3,650, the days a change's events stay after its
  last; and `forge.history_events`, 131,072 in both, at least 1, the
  events a project keeps, its changes whose last event is oldest swept
  first past it (`../../domain/landing-history.md`, 4.5 and 6). Each
  applies to every project; the events are demand-loaded and take no
  forge memory.
- **A repository's landing log:** `forge.landings_kept`, the landings
  every tier following their branch has judged, 16 in both profiles;
  and `forge.landings_pending`, those some tier has still to judge, 64
  in both, at least 1, so that a project whose changes end at landing
  (`../../domain/after-landing.md`, section 6) keeps a slow tier's
  backlog of landings with their members until it is judged, past which
  the oldest is judged with no member to comment on (`../../domain/after-landing-storage.md`, section 1 there).
  Their sum is at most the store's 256 a log holds, and with
  `forge.landing_members` and `forge.changes` it bounds a tier's posted
  comments, `(forge.landings_kept + forge.landings_pending) *
  forge.landing_members + forge.changes` within the store's 8,192, both
  checked at start (below).
## 3. Overrides and refusals

- **Overrides by dotted name:** the path of the field in its owner's
  `Limits` type, a duration's last segment with `_ms`. A name temper does
  not know, or a value outside its type, refuses the start, naming it.

  | Prefix | Owner |
  |---|---|
  | `root.` | the engine root's own limits, beyond those jig's `engine.limits` sets |
  | `forge.` | `temper-engine-domain-forge`'s `Limits`: `forge.client.`, `forge.change_policy.` and `forge.issue_policy.` for its children, `forge.queue_window_ms`, `forge.sitting_ms` |
  | `checkout.` | `temper-worker-domain-checkout`'s `Limits` |
  | `forge_component.` | the forge component's, its webhooks' listener's and its GitHub App's among them (../forge/README.md, section 6): the owner's, given once for every forge as the component is made (skein's `forge/connection.md`, 7.2), so none is per forge |
  | `git_component.` | the git component's (../git.md, section 8) |
  | `records.` | the engine's routing's: a forge's records and carried values in jig's secrets' store (../credentials.md, 2.1) |
  | `forges.` | the engine's routing's, per forge: a GitHub forge's accepted installations and every forge's log origins (../forge/README.md, section 6) |
  | `setup.` | `temper-forge-setup`'s `Limits`: reconcile's (../setup/reconcile.md), the fetch's (../setup/owned.md, 6), and the engine's setup host's states and runs (../setup/README.md, the setup host in the engine) |
  | `box.` | `temper-box-machine`'s `Limits`: the box's restarts and stop, its pipe's lines, its logs and tails (../setup/box.md, 1, and ../setup/box-supervision.md, 1) |
  | `clients.` | `temper-service`'s clients' server's `Limits`: the browser's bundle (`../../client/web/README.md`, 2.2) |
  | `routes.` | `temper-service`'s clients' listener's forge routes (`../../client/README.md`, the routes) |
  | `sign_in.` | `temper-service`'s Google sign-ins on the clients' listener (`../sign-in.md`, section 6) |
  | `picker.` | the picker's reads of a forge's repositories (`../../domain/forge/adoption.md`) |

  For example `forge.repositories`, `forge.client.calls`,
  `checkout.remote_timeout_ms`.
- **The forges are counts, not a list.** No forge is configured
  (`engine.md`), so the limits that bound forges are the deployment's:
  `forge.forges`, the forges the forge connector keeps at most, of
  every kind and mode, an ended one counted until its removal is whole
  (../forge/README.md, section 6), at most the accounts child's `credentials` in
  `engine.limits` and in each worker's `limits`, since each kept forge
  has a git identity; and `forge.repositories` and its kin, the
  connector's and the forge client's rows, shared by every kept forge,
  never a sum over them, `forge.makings` among them, the rows of
  repositories being made or failed, at most 64, the `forges` document's
  bound (`../../client/forge/records.md`). Memory is metered as any held bytes
  are.
- **Watched names and checkout repositories have separate capacities.**
  `forge.resources_per_task` bounds distinct forge names watched by one
  task, including held resources and subscribed branches. Both profiles
  give a goal room for its g and r branches and landings on main.
  `checkout.repositories` bounds repositories in one checkout specification;
  watching more branches in a repository needs no more directories. Forge
  subscription admission and the connector both enforce the watched-name
  bound.
- **`forge.landing_members` bounds a landing's changes,** 5 in the
  stock, its cap below: at least the largest `batch` a project names, which the
  start checks (`engine.md`, the batch), so a deployment whose project
  raises `batch` raises it with it. A landing holds a branch per member
  (`../../domain/landing-prepare.md`, 1.1), so `forge.resources_per_task` has
  room for `forge.landing_members` branches beside the train's, and
  `forge.holds` for that many per live landing, where the start ties
  them as it ties the rest. It is at most 5, and the start refuses one
  past that, naming `forge.landing_members`. The cap is the worker
  wire's `Prefixes` bound, 5 branches of 1,024 bytes
  (`../../domain/worker/jobs.md`, *The prepare kind*): a sixth would
  take the job's items past the channel's 1 MiB at their ceilings. The
  client wire's landing panel, which holds 8 members, is within it.
  It bounds a red round's search too: at most `ceil(log2(n))` probes
  for n units, three at 5 members, each probe's runs bounded as any
  run, by its project's `gate_timeout`
  (`../../domain/landing-search.md`, 3), so no limit of its own.
- **`forge.changes` bounds the changes kept at once,** live and ended
  alike: the forge's change table, whose settled changes leave it,
  oldest first, as new ones need room, those of live goals kept
  (`../../domain/forge/change/records.md`, 1). So it bounds the changes
  live at once, never every change made. Each tier row of the store
  holds up to `2 * forge.changes + 1` landings (its pending ones, a run's,
  a finding's blamed ones, a probe's waiting groups and a posting's),
  within the store schema's ceiling for them, 1,537 (`../store-versions.md`, 1),
  so `forge.changes` is at most 768, which the start checks (below), and
  the stock's is 768. The store holds that many because a failing test's
  id is within 1,024 bytes, `forge.tiers.test_id_bytes` at most that
  (below). The forge's landed commits, the
  merges temper made, number `max(forge.landings, forge.changes)` at
  most; a merge finding them full erases one, those of no kept change
  first, and never waits on it to move its branch's head
  (`../../domain/forge/change/records.md`, 1).
- **The forge client's lifetime and the forge call's deadline:**
  `forge.client.lifetime_ms` is how long a sent forge write may still
  arrive, its answer awaited before the client looks for it (skein's
  `forge/client.md`, 7.5), and `forge_component.deadline_ms` how long
  one forge call, or one of a forge's startup reads, may take before it
  fails (skein's `forge/connection.md`, 4.1). The slowest call a forge
  answers is a merge: Forgejo merges a pull request inside the call,
  bounded by its own git timeout, 360 s by default, and the merge of a
  pull request of about 7,000 files took longer than 10 s. So both
  profiles take a lifetime of 300,000 (5 minutes), under Forgejo's
  360 s, and a deadline of the lesser of the lifetime and 270,000, the
  most a GitHub forge takes, half a JWT's life (skein's
  `forge/connection.md`, 7.4): a GitHub forge starts at the defaults,
  and a Forgejo merge has 270 s. The deadline follows the lifetime: an
  override of `forge.client.lifetime_ms` moves a deadline that stands
  at the lesser of the lifetime it replaces and 270,000 to the lesser
  of the new lifetime and 270,000, and leaves a deadline overridden to
  any other value as it was set, whichever override the document gives
  first. The longer lifetime costs a lost write alone: a set waits out
  its first attempt's lifetime before it is written again, a write not
  found is looked for until its lifetime ends (skein's
  `forge/client.md`, 7.5), and a stop's settling waits at most twice
  the lifetime and `forge.client.clock_margin_ms`
  (`../../domain/root-restart.md`, 1.2), cut under the box by its long step's
  grace (`../setup/box-supervision.md`, 1). The engine forge world's fixture keeps
  its 10 s lifetime in its own limits; a test that needs a profile with
  the fixture's lifetime or deadline overrides them in its own
  configuration.
- **A limit's refusal names it:** every refusal a configured limit
  causes, at the start or in a step, names the limit by its dotted name,
  its value and what was needed. A step's refusal holds its task with
  the words of the forge's `RefusalCause::Limit`, such as `forge.changes =
  128; required usage 129; reduce the named work or raise this limit in
  the deployment configuration, restart, then release`, beside the
  operation and the resource it refused (`../../client/forge/README.md`,
  5). A step that holds several bounds in one check refuses on the one
  it passed, so each refusal names one limit. A refusal no limit causes
  (an unadopted repository, a malformed name, an unknown task, a
  duplicate) names its own cause where the forge has one, and keeps the
  forge's general words otherwise.
## 4. Component limits

- **The webhooks', the keep's settling, the records', the forges', the
  GitHub App's, the setup's, the routes', the picker's, the box's and
  the bundle's limits,** whose numbers are `small`'s constants, as every
  other's:

  | Name | Bounds |
  |---|---|
  | `forge.client.fallback_ms` | `fallback`: how long a repository hears after its last delivery, its timed reads and temper's own polls of it falling back to it meanwhile (`../../domain/forge/keeping-up.md`, 2; skein's `forge/client.md`, 4.1); 300,000 (5 minutes) in both profiles, any value accepted, 0 keeping the paces with no fallback |
  | `forge.client.settle_ms` | after a move of a pull request's head or base, how long a not mergeable answer, or a hinted job's pending CI, is read again at `hinted` and not believed, by the change procedure (`../../domain/forge/keeping-up.md`, 2); above 0, as `hinted` is |
  | `forge_component.hooks` | the webhooks' listener's connections in all |
  | `forge_component.hook_head_bytes` | a delivery's request head |
  | `forge_component.hook_body_bytes` | a delivery's body, hashed and decoded as it streams, so time and bytes, not memory; one past it is answered `413` and hints nothing |
  | `forge_component.hook_deadline_ms` | `hook_deadline`: a delivery's read and answer, from its connection's accept or its last answer |
  | `forge_component.hook_secret_bytes` | a Forgejo forge's webhooks' secret; a GitHub forge has none (../credentials.md, section 2) |
  | `records.value_bytes` | a forge's record or a carried value of temper's, the largest of any item; a pasted value past it is refused, named (../credentials.md, 2.1 and 2.2). The client wire's value ceiling is the larger of it and a moved record's bound (jig's `protocol/client/README.md`, section 3; `../../client/ctl/bounds.md`, 1), so a moved record's file reaches the engine and temper's own values stay within this bound |
  | `forges.installations` | the installations a GitHub forge accepts, its `Accepted` record's ids (../credentials.md, 2.1) |
  | `forges.log_origins` | a forge's job-log origins, its kind's (../forge/README.md, 2.6); default 20, at most 32 |
  | `forge_component.installations` | the installations the component holds tokens for, per forge |
  | `forge_component.log_origins` | the origins a job log's redirect may reach, per forge (default 20); as many origins said of other redirects are kept per forge (`Fact::Origin`, ../forge/README.md, 2.6) |
  | `forge_component.token_bytes` | a forge's token: an API token, a git token or a site admin's token (skein's `forge/connection.md`, 7.2) |
  | `forge_component.key_bytes` | a GitHub App's key |
  | `forge_component.renew_before_ms` | `renew_before`: how long before an installation's token expires it is minted again (../credentials.md, section 5) |
  | `forge_component.log_poll_ms` | `log_poll`: between two reads of a GitHub App's delivery log (default 5 s) |
  | `forge_component.log_catch_up_ms` | `log_catch_up` (default 24 h): how far back the delivery log is read when a forge starts again within one process (skein's `forge/connection.md`, 6.5); at most a day, the log's own reach. temper keeps no `guid` across process starts: a process's first read of a forge's log keeps the newest and reads nothing back, and skein's forge client reads each adopted repository afresh at restart |
  | `forge_component.log_pages` | the pages one read of the delivery log takes (default 10) |
  | `forge_component.delivery_bytes` | a delivery read from the log (default 1 MiB) |
  | `setup.deadline_ms` | a setup plan's run, its wait for the forge among it |
  | `setup.backoff_ms`, `setup.backoff_max_ms` | a setup plan's first request made again, doubling |
  | `setup.states` | the setup host's states, held in memory, for the manifest's and the installation's returns |
  | `setup.state_life_ms` | a state's life; at most an hour, the life of the manifest's conversion code |
  | `setup.runs` | the setup host's runs at once, a forge's one writing run and its one `Reachable` beside it both counted; a request past them, or a second of either kind on one forge, is refused busy |
  | `setup.fetch_bytes` | a fetched binary's body, written to its part file as it streams, never held; one past it removes the part file and is refused, naming the URL (../setup/owned.md, 6) |
  | `routes.query_bytes` | a forge route's query |
  | `picker.page` | the repositories one read of the picker takes, the setup host's `Reachable` plan's `reachable` on every kind (skein's `forge/setup/forgejo.md`, 1.8), within the forge's own page cap: GitHub's page, a connected Forgejo's; at most 100, GitHub's largest page |
  | `box.backoff_ms`, `box.backoff_max_ms` | a long step restarted, doubling |
  | `box.restarts`, `box.window_ms` | a long step's exits within the window past which the box gives up; `box.restarts` at most 64; the backoff counted from the exits within the window (../setup/box-supervision.md, 1) |
  | `box.grace_ms` | a long step's grace, after each of its stop's two signals (../setup/box-supervision.md, 1) |
  | `box.line_bytes` | a line of the box's pipe, either way, its line feed among them; the box and `temper-box serve` each read within it (../setup/box.md, 1) |
  | `box.log_bytes` | a step's log file, past which it moves to `<name>.log.1` and starts afresh (../setup/box-supervision.md, 1) |
  | `box.tail_lines` | the lines of a step's log the box holds, and writes under a failure's terminal line (../setup/box-supervision.md, 1) |
  | `box.groups` | the groups' files the box lists under `box/groups` at a later start, and so the groups it runs from them (../setup/box.md, 1) |
  | `clients.bundle_files` | the bundle's files, its page and each compressed form among them |
  | `clients.bundle_bytes` | the bundle's files' bytes in all, each compressed form's among them, held in memory while `temper-box serve` runs |
  | `sign_in.members` | the configured members, and the forge connector's bindings held (`forge.bindings`, the members' count) |
  | `sign_in.pending` | the sign-ins under way at once; a start past them is answered the busy page |
  | `sign_in.deadline_ms` | a sign-in's life, from its start to its return's answer; the sign-in cookie's `Max-Age` |
  | `sign_in.fetches` | the fetcher's exchanges with Google at once |
  | `sign_in.body_bytes` | an answer's body from Google: the key set or the token |
  | `sign_in.url_bytes` | a configured endpoint, and the authorization URL given to the browser |
  | `sign_in.secret_bytes` | the client secret |
## 5. Startup checks

Moved to [`limits-startup.md`](limits-startup.md), section 1.

## 6. Engine and client bounds

Moved to [`limits-client.md`](limits-client.md), section 1.

## 7. Memory and concurrency

- **The engine's byte budget** is the limit `transcript_memory`, a
  setting (`limit-settings.md`, 2.3), metered at
  the root (`../../domain/root.md`, 4.1). No start compares a memory
  declaration with a composed sum or names a memory deficit
  (skein/docs/foundation/programming-model.md, 6.3). Edge bounds and compatibility checks
  still refuse invalid limits. The owned example's worker `limits` are
  ../setup/init.md, 1's.

- **A worker alone's ring** holds its io's and its worker's kernel
  operations, staged and in flight, independently of its domain's room.
  With the owned example's worker it fits the kernel's 32,768 entries.

A goal's deployment capacity is sized with jig's limits setting
(`limit-settings.md`, 2.2), or with `live_goals`:
`tasks.tasks` and `tasks.project_tasks` bound live concurrency;
`tasks.tree_tasks` counts lifetime tasks including the root, and
`tasks.depth` counts depth below that root at zero. `tasks.delegates`
bounds live direct delegates, `tasks.batch` one atomic creation,
`tasks.project_goals` live tracked goals per project, and `documents.lines`
a charter document read. These are distinct from stored policy authority.
The inbox and compound-operation dependencies are jig's protocol/limits.md
; refusals preserve the source, dependent capacity and required
minimum. Task batch refusals preserve the codec field and ceiling. When
a task record drives the map value past the fixed profile ceiling, the
refusal names `TaskRecord`, its batch capacity and both sizes, and says to
reduce task record capacities; that ceiling has no scalar override.

## 8. Open questions

- **Profiles' numbers** beyond fabrig's development, which sizes
  `standard`'s work (`stock.md`): `small`'s, and each profile's machine
  concurrency, from the first deployments' measurements (jig's
  `protocol/configuration.md`, section 8). Default: as written.
