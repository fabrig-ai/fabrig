# Limits

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
- **Two profiles, as jig's:** `small`, for one person's machine with
  the mechanics' smallest work, and `standard`, a project of fabrig's
  scale, its own development, with five times the
  largest figure seen (`stock.md`); another name is refused. `temper-box init`'s
  document names `standard` for the engine's, the worker's and the
  application's limits alike (`../setup/init.md`, 3). Their numbers are
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
  (`../../domain/agent/tools.md`); trimming is the fallback. Where a limit
  of temper's is a quantity jig's profile also sets, its constant cites
  jig's profile's.
- **`small` starts as it stands** against an agent of its size:
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
- **A project's landing history:** `forge.history_days`, 14 in both
  profiles, from 1 to 3,650, the days a change's events stay after its
  last; and `forge.history_events`, 131,072 in both, at least 1, the
  events a project keeps, its changes whose last event is oldest swept
  first past it (`../../domain/landing-history.md`, 4.5 and 6). Each
  applies to every project; the events are demand-loaded and take no
  forge memory.
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
  | `box.` | `temper-box-machine`'s `Limits`: the box's restarts and stop, its pipe's lines, its logs and tails (../setup/box.md, 1 and 2) |
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
- **`forge.landing_members` bounds a landing's changes,** 2 in both
  profiles: at least the largest `batch` a project names, which the
  start checks (`engine.md`, the batch), so a deployment whose project
  raises `batch` raises it with it. A landing holds a branch per member
  (`../../domain/landing.md`, 5.1), so `forge.resources_per_task` has
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
  (`../../domain/forge/change/README.md`, 7). So it bounds the changes
  live at once, never every change made. Each tier row of the store
  holds up to `2 * forge.changes + 1` landings (its pending ones, a run's
  and a finding's blamed ones), within the store schema's ceiling for
  them, 759 (`../store.md`, 4), so `forge.changes` is at most 379,
  which the start checks (below). The forge's landed commits, the
  merges temper made, number `max(forge.landings, forge.changes)` at
  most; a merge finding them full erases one, those of no kept change
  first, and never waits on it to move its branch's head
  (`../../domain/forge/change/README.md`, 7).
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
  | `records.value_bytes` | a forge's record or a carried value of temper's, the largest of any item; a pasted value past it is refused, named (../credentials.md, 2.1 and 2.2). The client wire's value ceiling is the larger of it and a moved record's bound (jig's `protocol/client/README.md`, section 3; `../../client/ctl/README.md`, 9), so a moved record's file reaches the engine and temper's own values stay within this bound |
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
  | `box.restarts`, `box.window_ms` | a long step's exits within the window past which the box gives up; `box.restarts` at most 64; the backoff counted from the exits within the window (../setup/box.md, 2) |
  | `box.grace_ms` | a long step's grace, after each of its stop's two signals (../setup/box.md, 2) |
  | `box.line_bytes` | a line of the box's pipe, either way, its line feed among them; the box and `temper-box serve` each read within it (../setup/box.md, 1) |
  | `box.log_bytes` | a step's log file, past which it moves to `<name>.log.1` and starts afresh (../setup/box.md, 2) |
  | `box.tail_lines` | the lines of a step's log the box holds, and writes under a failure's terminal line (../setup/box.md, 2) |
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
- **Limits checked against each other at start,** each failure naming
  both limits:
  - the forge component's: its connections equal the forge client's
    calls in flight; its `kept_bytes` is the forge client's `calls`
    times its `answer_bytes`, set by temper, not configured (skein's
    `forge/connection.md`, 4.6); its deadline is at most the forge client's
    lifetime; its body bound holds the forge client's largest write and
    a key's marker; every listing's page size is at least 1 (../forge/README.md,
    section 6);
  - the checkout's capacity, whole: `checkout.conflicts`,
    `checkout.path_bytes` and `checkout.repositories` feed every bound
    that carries a merge's conflicted paths, and the start derives each
    from them before any of its other checks returns:
    `git_component.stdout_bytes`, at least `checkout.conflicts *
    (checkout.path_bytes + 1)` (../git.md, section 8); the agent's
    conflicts and conflict path bytes, at least the checkout's (the
    worker's, below), within smith's channel ceilings, 1,024 conflicts
    of 4,096 bytes; the worker wire's saved work, `Saved`'s encoded
    bound, within jig's ceiling for it, 16 MiB (../channel.md, section
    8); and the largest answer a run's saved work makes, a frame with
    its header, within each link's room, the engine's and each worker's
    `io.output`. A configuration one of them does not hold is refused
    once, naming every one that falls short, a line each, as the start's
    other refusals name that bound, after a first line naming the
    checkout's values they follow:

    ```
    checkout.conflicts 1024, checkout.path_bytes 4096, checkout.repositories 2: raise these with them
    git_component.stdout_bytes: 262144 bytes, short of checkout.conflicts * (checkout.path_bytes + 1), 4195328 bytes
    engine.limits.io.output: 1048576 bytes, short of the largest answer a run's saved work makes, 8401196 bytes
    workers[0].limits.io.output: 1048576 bytes, short of the largest answer a run's saved work makes, 8401196 bytes
    ```

    Both profiles hold 512 conflicts of 255 bytes in 2 repositories
    (`stock.md`), and an operator may raise `checkout.conflicts` to
    1,024, temper's payload ceiling, at those path bytes, with
    `git_component.stdout_bytes` alone raised with it, which this
    refusal names; nothing derived from the checkout's values refuses
    on its own;
  - the git component's: each rule of ../git.md, section 8 (its operations
    at least the checkout's workspaces, its output caps at least what
    it parses and the diagnostic it keeps);
  - the records': `records.value_bytes` within the largest value jig's
    secrets' store keeps (jig's `protocol/store/secrets.md`), and
    large enough for the maximum encoded `AppKey`, `Accepted`, `AppSlug`
    and `Origin`: the component's `key_bytes`, 8 bytes per
    `forges.installations`, or `forge.name_bytes`, each with its version
    byte; `Origin` counts both its kind and version prefixes and the
    largest API address, a host of 253 bytes with its port
    (../credentials.md, 2.1);
  - the forges': `forges.installations` at most
    `forge_component.installations`; `forges.log_origins` at most
    `forge_component.log_origins`, and each kind's origins within it; `forge_component.renew_before_ms` below the hour an
    installation's token lives; `forge_component.log_catch_up_ms` at
    least `forge_component.log_poll_ms`; each of these limits, and
    `routes.query_bytes` and `picker.page`, above 0;
  - the setup's and the box's: each backoff at most its maximum;
    `setup.backoff_max_ms` below `setup.deadline_ms`; `box.backoff_max_ms`
    below `box.window_ms`; `setup.states`, `setup.state_life_ms`,
    `setup.runs` and `setup.fetch_bytes` above 0; `box.log_bytes` at least
    one whole record (the time, the box's longest piece and a line feed);
    `box.tail_lines` above 0 and at most 100; `box.line_bytes` at
    least 64, which holds each line but a `failed` one's `WHY`, which
    the box cuts to fit (../setup/box.md, 1);
  - the bundle's: `clients.bundle_files` and `clients.bundle_bytes`
    above 0;
  - the sign-ins': each above 0; `sign_in.body_bytes` within skein-oauth's response bound;
    `sign_in.members` within the forge's bindings' bound;
  - the facts': `forge.judge_projects` at most 1024, an eighth of the bound on
    a party's facts on the client's wire
    (`../../client/forge/records.md`);
  - the store's tier rows': `forge.changes` at most the most they hold,
    `(c - 1) / 2` for `c` the least of the store schema's ceilings on a
    tier row's pending landings, a run's landings and a finding's blamed
    ones (`../store.md`, 4), 379 at their 759, checked before the store's
    sizing, which would otherwise refuse it as past its codec's bounds,
    naming no limit:
    `application.limits.overrides.forge.changes: 380, past its bound of
    379, the most the store's tier rows hold (2 * forge.changes + 1
    within 759)`. Within it, a store bound the larger rows outgrow
    refuses the start as the store's sizing names it (`engine.limits.store:
    the store's map value, N bytes, past its bound of M bytes`), the
    store limit to raise with it;
  - the webhooks' listener's: each of its limits above 0;
    `hook_secret_bytes` at least 32; `hook_head_bytes` at most the
    HTTP server's largest read within io's intake; `hooks` within the
    sockets left once the clients' listener and the forge's connections
    are counted. The sockets are temper's own, a constant of
    `temper-service`'s limits module that its io check takes in place
    of the sum of jig's engine's and worker's, and hold, under `standard`, a
    document with both listeners and a sign-in, as the owned example has (../setup/init.md,
    1);
  - the channel's (../channel.md, section 8): each of temper's largest
    parts within the ceiling jig's schema gives its field; the largest
    part the engine can send within what the worker's limits take; the
    checkout's `repositories` used by both sides, independently of the
    forge's watched-name bound; the longest remote the routing can make
    on the machine's places (the longest of `machine`'s remotes' bases
    and github.com's, an owner and a name of the forge's `name_bytes`,
    two `/` and `.git`), and the forge's names and branches, within the
    checkout's `name_bytes` (../channel.md, section 8); and the smallest
    resumed start the engine's link can cut a start to (jig's
    `protocol/channel/README.md`, 8 and 10), within the engine's room, its
    `io.output`, so a run that committed every call `core.call_records`
    admits can still resume. Its failure names both: `engine.limits.io.output:
    4096 bytes, short of the smallest resumed start, 27201 bytes`, the
    start's frame with its header, as `standard` has it. A document
    reaches it only by an `io.output` override below that start, which
    the worker wire's ceilings keep at most 218,177 bytes;
  - the worker's, against the agent's configuration
    (`temper-worker-domain::valid`, entrance constraints and count
    representability, with no heap sum), among its rules: the
    checkout's repositories, names, conflicts and paths within the
    agent's directories, their names, conflicts and paths; the agent's
    largest call within the checkout's commit message; and, of the hub's
    (jig's, in the worker's `limits`), its accounts within the agent's,
    its largest event with smith's 2-byte separator within the agent's
    message, and the agent's message less that separator within the
    hub's event. So `hub.event_bytes` is the agent's `message_bytes` less
    2, and no other value starts. Payload-related refusals name the
    exact override, its configured bytes and its required lower or
    upper bound: `checkout.message_bytes`, `hub.event_bytes` and
    `hub.turn_queue_bytes`. The event's requirement names
    `agent.json`'s `profile.declared.tool_payload` less the two-byte
    separator, so the value to set is explicit;
  - the turn's, against the agent's configuration: smith's largest
    turn, the `turn_body` its configuration derives (smith's
    `smith-agent-service`, `profile`; its models' reasoning allowances
    among what it derives from, `README.md`, section 4), is a whole conversation
    turn's body as the hub measures it, and each bound on its way holds
    it. Within each hub's `turn_bytes`, the engine's and the worker's,
    since each side of the link takes a turn of its own hub's bound;
    as a frame of the worker's link, header included, within the
    worker's `io.output`, the room its stream grants at once, or the
    worker's link is not made (jig's `protocol/channel/README.md`, 8 and 10);
    with its 4-byte length, within `core.resume_bytes`, so that a chat's
    largest turn resumes rather than starting afresh, and so within
    `root.journal.transcript_bytes`, a journal row's (the resume
    setting's check below); within `root.loads.transcript_bytes`, a
    loaded row's, and with a row's slot, within `root.loads.bytes`, a
    loaded page's, so that the engine loads it back; and with a read's
    row, within `root.loads.read_bytes`, so that a person's read
    carries it whole. Then the engine's resume window, as each hub
    counts it in an assignment's transcript (jig's `domain/hosts/README.md`):
    `core.resume_bytes` and, for each turn it can hold, a box past the
    4 bytes it already counts, within each hub's `transcript_bytes`.
    Each failure names the bound, its bytes, and the figure it must
    hold: `workers[0].limits.overrides.hub.turn_bytes: 65536 bytes,
    short of the agent's largest turn, 1066989 bytes`.
- **In the cluster's shapes** (`../cluster.md`, 1), each process checks what
  its document holds: an engine alone, every check above but the
  worker's and those of the turn's that need the agent; a worker alone,
  the worker's, the checkout's and the git component's, the channel's
  for what its side receives, and the agent's largest turn within its
  hub's `turn_bytes`. What only an engine's document and a worker's
  hold together, the engine's largest part against the worker's
  `limits` among it and the agent's largest turn against the engine's
  bounds, is held by the cluster's description, whose one template
  writes the same `application.limits` into every document (README,
  15.5), and at the link's opening by jig's hello and terms (jig's
  `protocol/channel/README.md`, 4.3).
- **Call input derivation for a local agent:** when
  `core.call_answer_bytes` is omitted from an engine's or worker's
  `limits.overrides`, temper uses the larger of its selected jig profile's
  allowance and the agent's `profile.declared.tool_payload`. An explicit
  override stays unchanged, including an explicit profile-default value.
  This composition precedes the core, store, worker wire and memory bounds;
  offline store readers use the same checked agent metadata. Engine-only
  processes retain their configured allowance. A cluster template must give
  its engine and workers consistent explicit allowances; per-kind terms
  reject a disagreement before hello.
  An explicit allowance below the local agent's `tool_payload` is
  refused at startup by its path, in two lines: what it is short of and
  the remedy, then the bounds a call meets, for instance
  `workers[0].limits.overrides.core.call_answer_bytes: 2048 bytes, short
  of the agent's call input (profile.declared.tool_payload), 4096 bytes;
  raise it to at least 4096 bytes, or omit it to follow the agent's call
  input` and `workers[0].limits.overrides.core.call_answer_bytes: call
  input 4096 bytes; wire answer 2048 bytes; smith host_reply_bytes 4096
  bytes`. Every entrance that composes the allowance refuses it alike,
  the offline store readers included. An answer past the agent's
  `host_reply_bytes` reaches its run as smith's `TooLarge`
  (`../../domain/agent/tools.md`).
- **jig's limits stay in jig's parts:** `engine.limits` and each worker's
  `limits` (jig's `protocol/limits.md`).
- **What the root takes of `engine.limits`:** its core children's
  (`tasks.`, `authority.`, `people.`, `fleet.`, `accounts.`, `notes.`
  and `documents.`), with `core.templates`, `core.call_records`, `core.policy_bytes` and
  `core.call_answer_bytes`, after the local-agent composition above. The
  rest of the core's it gives the core itself (`../../domain/root.md`,
  section 9): from its own `root.` limits, in place of `brief.`,
  `core.brief_parts`, `core.brief_core_budgets.`, `core.resume_bytes`,
  `core.run_bytes`, `core.escalation_reason_bytes`,
  `core.load_slots` and `core.load_bytes`, a person's read of the store,
  from `root.loads.read_bytes`; as constants, in place of `views.`, but
  for its watch rooms (below), `core.fill_mark` and `core.connectors`. An
  override of one of these changes nothing the root holds, but for
  `core.resume_bytes`: the core's resume setting is still taken from it,
  above 0 and within `root.journal.transcript_bytes`.
- **The engine's documents fit temper's client.** A document past the
  bounds a client decodes is refused there whole, as too large (jig's
  `protocol/client/wire.md`, section 1), so the documents the engine serves
  must be within what temper's client takes: its client takes
  `jig-client`'s limits from jig's small profile with the same
  `documents.` overrides the owned example's `engine.limits` names
  (`../../client/README.md`, section 2; ../setup/init.md, 1). The engine cuts each
  text to its bound and says so (jig's `client/domain/contract/documents.md`,
  3). It checks none of this at start: it serves whatever client the
  deployment lets in, whose limits are the client's. A test holds the
  owned example's and the process tier's configuration to temper's
  client's limits, so the two cannot drift apart unseen.
- **The engine's watch rooms follow what it admits.** A watch past the
  room the engine allows a party, a stream or the deployment is refused
  busy, and each holds its place until the engine has ended it (jig's
  `client/domain/contract/README.md`, 5.2; jig's `protocol/client/README.md`, section
  10). The deployment admits `clients.streams` clients at once, each
  holding up to its own watches, `me`'s included, and temper's shipped
  clients hold at most 31 (`../../domain/root.md`, section 9): the web's
  and the terminal's `watches`, 31, and `handles`, 33, so that Home on
  All projects holds four projects' statuses, `decided`, `follows` and findings
  documents with the session's own watches, 30 at most
  (`../../client/pages/home.md`, 1, *Home on All projects*). So:
  - *a stream's room* is 31, `WATCHES_PER_STREAM`, one shipped client's;
  - *the deployment's room* is `clients.streams` times that: 248 at
    `temper-box init`'s 8 streams (`stock.md`), 62 at the small profile's 2;
  - *a party's room* is the deployment's, since a person's tabs and their
    `temper ctl` calls are one party and may hold every stream.
    `views.watchers_per_party` never bites below `views.watchers` here:
    what one person can use up is the streams, and that refusal names
    them.

  `clients_http.streams` bounds the transport's side of those same
  streams and adds no room. The same test as for documents holds a
  stream's room to temper's client's watches, and a test of the limits
  holds the other two to `clients.streams`.
- **The rooms have no setting of their own.** More room is more streams,
  `clients.streams`, and the 31 is the shipped clients' bound. An
  `engine.limits.overrides` naming `views.watchers` or
  `views.watchers_per_party` is refused at the start, naming
  `clients.streams`, and `temper help limits-engine` lists neither: the
  root never took them, so a document naming them was told nothing.
  Those two names stay jig's, for an application whose root takes them
  (jig's `protocol/limits.md`). `views.`'s other names
  stay as above.
- **What a watch costs.** A watch holds a views slot, a name, a flight
  and its stream's entry, and shares a subject's slot with the other
  watches on that subject. It keeps no document between deliveries (jig's
  `domain/engine/views.md`), and these capacities allocate nothing
  until used, so a larger room costs nothing until it is held.
  - *Work:* an opening scans the watches held and builds its first
    document. Each commit's marks scan the watches held, and build one
    document for each watch whose subject changed, none shared between
    watches on one subject, with one delivery in flight per watch.
  - *Heap:* that pass holds each document it built until the document is
    delivered, so its peak is what those documents hold, each within the
    documents' bounds. No heap sum admits the root (`../../domain/root.md`,
    section 9), so no figure at the start moves. The engine's world
    measures a full room (`temper/tests/engine/domain/tests/suite/memory.rs`).
    With one or eight streams the empty root held 4,742 heap bytes; measured rooms
    of 17 and 136 watches held another 14,112–15,256 and 114,720–126,336 bytes after
    delivery, respectively, and one marking commit peaked at 116,816 and
    452,176 bytes in the engine world including its driver and fake store.
  - *The journal's door:* a full room puts at most 208 deliveries behind
    one commit, within `root.journal.deliveries`' 512. The watermark
    paces new work as the door fills, and nothing fails.
  - *The store:* its bounds don't follow the rooms. A watch holds no row,
    and an opening on a retained subject takes one of `root.loads.loads`,
    refused busy past them.
- **Past the room.** With the shipped clients, each stream's room fits
  the deployment's; watches still closing can overlap their replacements:
  a stream's predecessor holds its place until its terminals come (jig's
  `client/domain/client/watches.md`, 1). Each place is given back at its terminal,
  and a client refused busy opens again after a backoff. A refusal of `me`
  leaves its link offline; an application's watch can be behind with the
  link still live. Both set `watches_full` (jig's
  `client/domain/client/watches.md`, 2), and `temper ctl` says so at its deadline
  (`../../client/ctl/results.md`, 3). Busy can also mean an opening's load slot is
  unavailable; it carries no cause distinguishing that from a full room.
- **The forge's paths fit a document.** jig's documents carry a
  resource's path and a pattern whole, with as many segments as the
  engine admits in a name (jig's `client/domain/contract/documents.md`, 3), and
  `documents.lines` bounds a document's short lists alone, at most the
  client's rows. So `engine.limits` must name the bounds on a name's
  segments, `authority.segments`, `tasks.hold_segments` and
  `tasks.authority_segments`, at least the segments of the deepest path
  the forge names: a branch's, the forge's five segments then the
  branch's own (`../../domain/forge/README.md`, section 2), 8 as temper's branches
  name them. temper's client takes a path's segments, 9, apart from its
  rows (jig's `client/domain/client/README.md`, section 11), so a page of the
  forge's work is never refused for a path, and a long path shows cut,
  with jig's mark (`temper/docs/design/client/view.md`, 4).
- **The forge's authorities fit a document.** jig's documents carry an
  authority's lists whole, with as many entries as the engine admits in
  each, and neither cut nor refuse a document or an offer for one (jig's
  `client/domain/contract/documents.md`, 3), so a party who may amend a task
  reads its document whatever `documents.lines` is. temper's client
  takes an authority's entries, 16, apart from its rows (jig's
  `client/domain/client/README.md`, section 11): the most the process tier and
  the owned example admit in a list, their `authority.grants` and
  `tasks.authority_grants` (../setup/init.md, 1). A long authority shows
  in part, with jig's mark (`temper/docs/design/client/view.md`, 4). The test above holds
  the engine's bounds on an authority's lists to temper's client's.
- **The engine's byte budget** is `engine.memory_bytes`, metered at
  the root (`../../domain/root.md`, 4.1). No start compares a memory
  declaration with a composed sum or names a memory deficit
  (skein/docs/foundation/programming-model.md, 6.3). Edge bounds and compatibility checks
  still refuse invalid limits. The owned example's `engine.limits`,
  worker `limits` and `memory_bytes` are ../setup/init.md, 1's.

- **A worker alone's ring** holds its io's and its worker's kernel
  operations, staged and in flight, independently of its domain's room.
  With the owned example's worker it fits the kernel's 32,768 entries.

A goal's deployment capacity is sized with `engine.limits.overrides`:
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

## 1. Open questions

- **The real-browser check of watch sizing.** The domain story uses the
  pages' watch sets without HTTP, a browser or a kernel (`limits.md`).
  Checking Home at 1440 CSS pixels and a long chat at 420 while fresh tools
  clients read is a manual check after landing, as the `watch-room` request
  assigns it. Default: that check, with no additional browser-tier story.

- **A party's fair share of the watch rooms.** A party's room is the
  deployment's (`limits.md`), so one person with every stream leaves another
  none, and the refusal they meet is the streams'. Nothing has asked for
  less, and a smaller share would lock a person's own tabs out again.
  Default: none.

- **`views.`'s other names.** `views.runs`, `views.subjects_per_change`,
  `views.text_bytes` and `views.facts` are still taken and change nothing
  the root holds (`limits.md`), where `views.watchers` and
  `views.watchers_per_party` are refused. Nobody was misled by them on
  record. Default: they stay until one is.

- **Profiles' numbers** beyond fabrig's development, which sizes
  `standard`'s work (`stock.md`): `small`'s, and each profile's machine
  concurrency, from the first deployments' measurements (jig's
  `protocol/configuration.md`, section 8). Default: as written.

- **The rest of an assignment's transcript and of a person's read.**
  The turn's check (`limits.md`) holds the resume window's share of each
  hub's `transcript_bytes`, and the turn's share of
  `root.loads.read_bytes`. The answered calls an assignment carries
  beside its window (`core.call_records`, each within
  `core.call_answer_bytes`), and the owner's task record and ancestors
  a read carries with its row, have no bound the start can name: jig
  gives neither's worst case. Default: the hub refuses an assignment
  past its bound at run time, and the documents leave room beside the
  turn for the read's evidence (../setup/init.md, 1).
