# Limits

- **Policy above, numbers below.** The design documents name limits and
  what they bound; the numbers are in `jig-protocol-config`'s stock,
  as module documentation and constants, never in the design.

## 1. The stock

- **The stock** is one set of every limit jig's children and components
  take, consistent with one another, for a project of fabrig's scale
  (temper's `draft/config.md`, 5.1). `jig-protocol-config`'s
  `stock()` gives it. No document chooses a profile.
  - **Over a base.** An application reads a document's limits over a
    base of its own: `engine_over` and `worker_over` take it, and
    `engine` and `worker` read over the stock. An application's base is
    jig's stock with its own figures and derivations over it (temper's
    `protocol/configuration/derived.md`, section 4).
  - **A document's profile.** `limits.profile` may be absent or
    `standard`, and either reads the base. Any other name refuses the
    start at that member, and the rest of the document is checked
    against the base.
  - **Overrides.** A deployment overrides any limit by its name, as
    before.
  - **`small` is a test's.** Its figures are in the fixtures crate
    `jig-protocol-config-fixtures`'s `small()` (temper's
    `draft/cli.md`, 3.2), for tests alone.
  - **Reading this document.** Below, "standard" means the stock and
    "small" means the fixtures' figures.

## 2. Changing limits

- **Limits as settings** (`../domain/engine/settings.md`, 1): the core's
  kind `limits` holds overrides of jig's limits, each a dotted name of
  `names::NAMES`, a value and a reason. The core keeps it and checks its
  shape alone. An application reads it at its start over its base with
  `with_overrides(base, overrides)`, by the names and checks a document's
  overrides take, and the problems it gives name the setting rather than
  a document's path. A document's `limits` and `memory_bytes` are then
  optional members: absent, `engine_over` and `worker_over` read the
  base, and `Engine.memory` is none. temper's engine and box refuse them
  (temper's `protocol/configuration/limit-settings.md`).
- **When a change applies.** Each row of `names::NAMES` gives `applies`,
  `names::Applies { AtOnce, Live, NextStart }`. `Live` is a time bound
  or a ceiling every holder reads at each use, which sizes no table, no
  carrier and nothing a link agrees at its opening or its hello states:
  `link.handshake_ms`, `link.hello_ms`, `link.stall_ms`,
  `link.backoff_ms`, `link.ceiling_ms`, `link.proved_ms`,
  `link.busy_ms`, `hub.event_bytes`, `hub.accounts` and
  `hub.turn_queue_bytes`. Every other row is `NextStart`, since the core
  and its children size their tables from their limits at their start.
  `AtOnce`, a raise everything holding a copy takes at once, is for an
  application's names; none of jig's has it yet. An application takes a
  live change by `channel/figures.md`, 3 and 4 (temper's
  `protocol/configuration/limit-settings.md`, 5).

## 3. Sizing and derivations

- **Sized from fabrig's development:** the stock sizes its counts,
  times and other figures as temper's stock does (temper's
  `protocol/configuration/limits.md`, sized from what): a
  count at twice dev.fabrig.ai's figure of 2026-11-16 where that is above
  the figure the stock had, rounded up to the least d × 10ⁿ, 2ⁿ or
  3 × 2ⁿ. Its machine's concurrency is sized the same way: the hub's and
  the fleet's slots, the workers, and the clients' streams. The fleet's
  slots are the one exception: the engine's link to a worker carries a
  u32 cap, slots times a slot's assignment, message frames and inbox, so
  the stock's `fleet.slots` is the largest such figure at most 80 that the
  cap holds at the stock's figures and at those temper's engine starts
  with over them, and never below dev.fabrig.ai's 36: 40 on 2026-11-17,
  where the stock's figures alone hold 64.
- **Derived in the reader:**
  - **The stream carriers.** Where `clients.exchanges` or
    `clients_http.calls` isn't overridden, it follows the stream bounds
    after the overrides, the least the stream rule below takes.
  - **A worker's hub slots.** Where a worker's `hub.slots` isn't
    overridden, it is its `slots`. One overridden below its `slots` is
    refused, as before.
  Small's store page byte ceiling admits an application's page floor
  derived from the largest shipped ordinary transcript's full frame
  (`store/records.md`, 4); standard's admits the frame of smith's largest
  turn (`store/transcripts.md`, 1), so any turn smith admits is stored.
  Small also holds composed application settings history, including replaced
  values; its value, page and chunk ceilings grow together when that row grows.
  The ceiling is accounting alone: page rows, chunks, decoder capacity,
  queues and read slots keep their own bounds; the number and its
  derivation live beside `STORE_KV` in the profile.
- **Standard's room for a whole spec:** its `hub.charter_bytes` is
  262,144, its `hub.outcome_bytes` 131,072 and its `authority.batch` 32,
  where small's are 65,536, 4,096 and 2. None is allocated ahead: the
  first two bound a frame and an admission, and the third what a stored
  denial and a batch's aggregate may hold.
- **Standard holds fabrig's own development** as its agents run it,
  with five times the largest figure seen there: live tasks in the
  deployment and in a project, live goals, a goal's tree over its life
  and its depth, a task's live delegates, the tasks one batch makes, the
  messages a task's inbox keeps (a result from each live delegate, a
  question and an answer) and a run's inbox takes, a spec's bytes, and a
  message's and a result's, a message at least a result, since a
  parent's inbox holds a result as a message. What those figures couple
  moves with them, the least that passes the checks below: the inbox's
  bytes, a result's room for each live delegate and a message's for a
  question and an answer; the people child's operations per request and
  its inboxes; the notes' scopes over the grants; the hub's inbox with
  the run's; a document's lines, which hold every charter and executor a
  creating offer names; the funders, pools, waiters and exemptions with
  the live tasks; and a batch's bytes, which hold five times the largest
  plan seen, every task of it at the largest spec seen. Its store's
  value, page and chunk are what those limits make of the largest row
  (`store/records.md`, 4); its map's budget, journal
  and routes are a team's as they were. Its machine's concurrency (hub,
  fleet and worker slots, workers, clients' streams and calls) covers
  fabrig's development (above), and a worker's hub slots follow its
  `slots`.
- **Standard's first deliveries fit together.** Its stream watch count
  covers a full web client. Its queue holds that many deliveries at the
  wire's largest owned record, with the owner, version, length, watch
  number and stream-event tag. The profile derives this from the codec's
  encoded bound, plus the largest in-flight hello and its event tag. Its
  HTTP SSE event holds the largest stream record in base64, with the data
  prefix and line feeds. These are admission
  ceilings, never allocations ahead of use; small keeps its own figures.
  The application's startup test checks the web watch count and the
  resulting stream ceiling against its supplied memory budget. As below,
  that domain budget is no process-wide sum of all edge ceilings.

## 4. Overrides and startup checks

- **Client pressure overrides.** `clients.queued_bytes` is a u32 byte
  ceiling for one stream, including its in-flight record, at least 128;
  `clients.exchanges_per_client` is a positive u32 count of exchanges in
  flight per sign-in, shared by its clients, a batch one whatever its
  members; `clients.reads_per_client` and `clients.openings_per_client`
  are positive u32 counts of reads and of watches' openings in flight per
  sign-in, each a read's or an opening's whether alone or a batch's
  member, 32 and 31 in both profiles (`client/README.md`, section 10). The
  reader checks them through the component's validity predicate after
  applying overrides.
- **Stream overrides carry their calls.** `clients.streams`,
  `clients.exchanges`, `clients_http.streams` and `clients_http.calls`
  are positive scalar overrides. When any is overridden, both carriers
  (`clients.exchanges` and `clients_http.calls`) must hold the larger
  stream bound plus four ordinary exchanges. The component's exchange
  bound also carries every call the HTTP mount admits. The reader checks them
  together and refuses an undersized carrier by its bound's name.
- **Checked whole at startup, where limits size an edge:** the terms of
  the workers' channel follow from the same limits (`channel/README.md`, section
  10). No sum of worst cases is checked against memory: an engine's
  `memory_bytes` is the application's byte budget, handed to it
  unchecked (skein's ../foundation/programming-model.md, 6.1 and 6.3), and a worker's
  is optional, since no worker reads one.
- **A document whose profile names another.** Its `limits` are refused at
  that member, and the rest of the document is checked against the
  base, so that the refusal adds no bound the operator did not choose.
- **Made projects count too.** Authority's and people's
  `projects` bound every project the deployment holds, seeded and made
  after the first start (`../domain/model.md`, 6.1): a deployment that
  makes projects configures room past its seeds, and a `MakeProject`
  past either is refused as a limit.
- **Configured projects fit both owners.** After every override is
  applied, the actual distinct configured project count, including
  ownerless rows, fits authority's project capacity, and that count
  plus one, the deployment's roles (`../domain/people/README.md`, section 3),
  fits `limits.people.projects`, the people roster's capacity. A
  violation is `What::Bound` at the projects path, before the first
  start writes anything. The count is the configured rows, not a
  list's allocated capacity, and the deployment's roster is the only
  one implied. `Core::new` asserts the same people-roster relation as
  a programmer precondition (`../domain/engine/README.md`, section 8).

## 5. Public reading

- **Public reading's limits,** the clients' component's
  (`../domain/people/public-reading.md`, 5; `client/public-reads.md`):
  `clients.public.reads` in flight; `clients.public.rate` reads per
  second and `clients.public.rate_burst`, the most its bucket holds;
  `clients.public.bytes` of answers per second and
  `clients.public.bytes_burst`; `clients.public.answer_bytes`, the
  ceiling of one answer; and `clients.public.max_age` in seconds, at
  most 60. The HTTP mount's `clients_http.public_calls` in flight. The
  root's public figures follow from these (`../domain/root.md`, section
  10). The core's public index (`../domain/people/public-reading.md`, 9):
  `core.public.goals`, live goals per board, `core.public.ended`, ended
  goals kept per project, `core.public.lines`, lines per goal and
  delegates per card, and `core.public.depth`, a card's lineage.
  Every profile's public figures are zero, a rate of zero answering
  every public read busy: a deployment that opens sets its own.
- **Their relations, checked at start** once the rate is above zero
  (with a rate of zero only `max_age`'s bound is), each refusing the
  start at its path: a burst at least its rate, but for a rate of zero;
  `answer_bytes` within `bytes_burst`, every jig kind's and every
  declared application kind's bound within `answer_bytes`;
  `clients_http.public_calls` within `clients.public.reads`; `max_age`
  at most 60;
  and the public index's memory, its bounds times the deployment's
  projects, within the process's fit check (`../domain/engine/README.md`,
  section 7).
- **The role `public` has no number:** a policy, configured or in the
  rules, names roles by number, so none can name the public party's
  (`../domain/authority.md`, section 6).

## 6. Charters and models

- **Charters' budgets:** each charter's turns and time are above its
  reserve's, and its default budget holds its reserve and one of its
  default model's most costly completions (`../domain/authority-budgets.md`,
  section 1), or the engine's start is refused, naming the charter. A
  task's chosen model is checked with its run's check
  (`../domain/authority-checks.md`, 1.3).
- **A charter's documents:** at most `tasks.charter_documents` per
  charter (`../domain/engine/settings.md`, 9; 2 in Small), with the brief's
  sections 8 in both profiles, the core's own six kinds and two
  documents, which the start's check of 17.9 holds, each priced within a charter's
  part of the worst case at its bounds: a name at jig's pattern bounds,
  a path's 256 bytes and a heading's 80. The connector that gathers
  them prices their sections within its brief budgets.
- **A charter's parts:** at most `tasks.charter_parts` per charter, 8 in
  every profile (`../domain/tasks/messages.md`, 6). Each part's instructions
  count in the charter's stored policy, with the two bytes of the blank
  line that joins them to the text before (`../domain/engine/executors.md`, 1,
  step 4), and its `on` as one set; so the charter stays within
  `core.run_bytes` whichever parts a run reads, and parts make a run read
  less, not the charter store less.
- **LLM accounts' bounds:** accounts in the deployment, accounts per
  holder, sources per account, sign-ins at once (moves out and in
  count among them: `../domain/accounts.md`, 3.6), program sources per
  worker, and a credential's bytes, which bound a key (skein's
  `llm/vocabulary.md`, 1.6). Within them the accounts' table, skein's driver (its
  listeners twice the sign-ins at once, so that a sign-in started
  again has room while the one it cancels settles: `llm.md`, section
  7), the hub's grants, a worker's `sources` and `logins` and the secrets'
  store are sized; a request that would pass one is refused, naming it
  (`../domain/accounts.md`, 3.6). Clients receive at least the engine's
  document lines in each account, source and sign-in list
  (`../client/domain/client/README.md`, section 11). A sign-in waits for its
  redirect only within the sign-in deadline, a limit of the driver's
  (skein's `oauth/README.md`, section 3). The driver reads a returning redirect
  within its own bound, `redirect_bytes`, 16,384 in both profiles: at
  least what holds its client's registered URI, code, scope, state and
  issued client id at their bounds (skein's `oauth/accounts.md`, 7,
  `redirect_room`: 13,407 in small). Its listener's request head,
  `server.head`, holds a browser's request for a redirect at that bound,
  cookies for `127.0.0.1` among its fields (`listener_head`: 24,593). The
  address the driver sends out stays within the accounts' address bound,
  `ADDRESS`, 2,048 (`url_bytes`). A profile whose driver holds less than
  either, or sends out more, is refused at the driver's member
  (`limits.driver.redirect_bytes`, `limits.driver.server.head`,
  `limits.driver.url_bytes`), and so is an override whose driver would.
  The driver's own over-long listener redirect ends its sign-in with
  the long-answer page (skein's `oauth/accounts.md`, 5). The accounts protocol
  carries its failure as `SignInAgain`; its address's length and bound
  stay at the driver's terminal. Carrying those figures to an
  application's sign-in page is separate work (`configuration.md`, section 8).
  A move out waits for its client's
  *written* only within the move's deadline, the accounts child's, past
  which the move is cancelled (`../domain/accounts.md`, section 6); it
  is positive, or the start is refused.
- **A model's description** is optional. When present it is nonempty
  text of at most 512 bytes (`MODEL_DESCRIPTION_BYTES`), or the start is
  refused at the model's `description` path.
- **A model's prices and unit** are positive, including `cache_write`
  when given, or the start is refused at its `prices.<member>` path.
- **Configured models** are at most the bound on models an authority
  names (`../domain/authority.md`, section 3), or the start is refused.

## 7. Requests and retained answers

- **Operations per request,** the people child's limit on a compound
  request (`../domain/people/acts.md`, 1), is at least the larger of
  *delegates per task* plus one and *goals per project*, both the tasks
  child's (`../domain/tasks/README.md`, section 6), or the start is refused,
  naming the three. So closing a conversation, each of its live
  delegates cancelled or moved and the conversation then cancelled with
  a grace, fits one request; and so does reordering every live goal of
  a project.
- **A compound request's worst case** is `../domain/engine/README.md`, section
  7's, summed into the engine's worst case with the rest.
- **Request-answer capacity** is shared engine-wide: retained answers
  plus the ordinary pending flights that reserve answer places, not a
  per-party allowance or a network-concurrency limit. Ordinary admission
  reserves an answer for success or permanent routed refusal, and never
  evicts an unexpired answer. Replay or conflict for a retained key and
  joins or conflicts for a pending key precede new admission. A
  top-level `Seen` neither reserves nor saves an answer; its flights use
  the pending, waiter, load, journal and output room independently,
  even when completed-answer places are full (`../domain/people/acts.md`,
  1). These exceptions change no configured limit or worst-case
  figure. Ordinary unique-key workload, permanent
  refusals and pending reservations still need room throughout the
  advertised retention. Stock profiles size that room from a daily
  conversation workload per configured party (creation, messages and a
  decision), with fivefold headroom shared by all deciding acts,
  permanent refusals and pending reservations. The calculation lives
  beside each profile's answer limit; the advertised retention stays a day.

## 8. Retention and checkpoint limits

- **Retention's horizons** are three limits of the core, the same in
  every profile: an ended task's turns, its full record and its summary
  (`store/retention.md`, 2). The start refuses a horizon of 0 and three
  out of order.
- **Checkpoints kept,** `store.checkpoints.keep`, the store's
  `Limits`'s `checkpoints.keep`: how many of the newest checkpoints the
  store keeps (`store/checkpoints.md`, 4.7), the same in every profile,
  2 as temper's `draft/archive/checkpoints.md`, 3.4 has it. An override is a
  number from 1 to 64; any other refuses the start, naming it.

## 9. Deployment and application limits

- **A template's grace** is within the tasks child's limit on a grace's
  span, or the start is refused, naming the template (`configuration-templates.md`, 1).
- **The deployment's message bound** holds every kind of message a
  task's inbox admits, as a run renders it with its sender's label: a
  party's words and questions by the people child's bound, which is at
  most it, and every other kind by its own (`../domain/tasks/messages.md`, 2;
  `../domain/people/acts.md`, 1). Whether a host's agent takes messages that
  large, and runs the engine's charters, is the fleet's to check at its
  hello, never configuration's (`../domain/engine/fleet.md`).
- **Settings' limits,** the core's (`../domain/engine/settings.md`, 8): the
  revisions held besides the newest, and the charter versions they read
  besides the newest's, each priced in the core's worst case, the tasks
  child's and the fleet's tables sized for the charters and the
  versions together; the history, the revision records the store keeps
  beyond those held; and the pending proposals of settings, each priced
  at a whole set. The current settings, the defaults and the changes
  over them, are priced at two whole sets within the counts and bytes
  above. Each is positive, or the start is refused, naming it; settings
  held past one lowered since refuse the start, saying which.
- **The fleet's needs bound** (`fleet.needs_bytes`) holds the configured
  charters' agent needs, in owned bytes, as the fleet counts them
  (`../domain/engine/fleet.md`), or the start is refused as
  `What::NeedsBytes { bytes, limit }` at `limits.fleet.needs_bytes`, as a
  policy past its budget is as `What::PolicyBytes`. Stored settings whose
  needs pass it refuse the start as `NeedsPastLimit`
  (`../domain/engine/restart.md`).
- **An application's kinds of settings** declare a value's largest
  bytes and the values per scope (`../domain/engine/settings-application.md`, 1), which
  enter the core's worst case for what it keeps of each setting, and
  the connector's for the values it keeps; a default past either
  refuses the start, naming the kind.
- **The application's limits** are its own members, checked in the same
  sum.

Capacity dependencies are checked after all overrides, before family admission.
`What::Capacity` retains the source member, configured count, dependent
member and required minimum. Live task capacity needs at least twice as
many `people.inbox_tasks`; `people.operations` holds the larger of
`tasks.delegates` plus one and `tasks.project_goals`. A failed relation
names its source and the dependency to raise, without labeling unrelated
overrides invalid. Raising a dependency still requires the resulting
profile to pass every admission check.

The relations are every "at least" between two configured limits that the
core's and its children's admission holds, checked in this order, each
naming its source first and its dependency second: the two above and
`project_goals`'; `people.operations` within `tasks.tasks`;
`authority.grants` plus three within `notes.scopes`;
`tasks.message_bytes` within `tasks.inbox_bytes`; `tasks.run_inbox` and
`people.inbox_entries` within `tasks.inbox_messages`; `tasks.exemptions`
within `tasks.tasks`; `fleet.configured_workers` plus one within
`people.program_hosts` and `accounts.program_hosts`;
`fleet.worker_name_bytes` within `people.program_host_name_bytes`;
`accounts.program_names` within `fleet.program_sources`;
`authority.roles` within `documents.roles`; the inbox's waiting room,
twice `tasks.inbox_messages` with `authority.roles` and four, within
`people.inbox_recipients`; and the accounts' credentials with their
accounts, up to the models' 64, within `fleet.grants` and
`authority.accounts`. The first three are checked whatever their
figures, so a zero `people.operations` is refused as `tasks.delegates`
needing `people.operations` of at least its count plus one. After them a
relation with a zero on either side is not checked: a zero is a bound,
refused as an override past its family's bound.

The **effective store codec envelope** bounds the framed byte collections
people-v14 derives from `core.policy_bytes` and `people.amendment_bytes`,
as well as its independent fields. Configuration checks it after overrides
and again with the whole engine, beside settings history and journal-write
admission. It probes the people codec and the combined store codec; it
never lowers a configured limit or a derived bound to make it fit.
With the shipped profiles' other limits, the largest policy budget is
43,695 bytes and the largest amendment budget holds a whole `tasks.spec_bytes`:
a task a person creates is bounded by `tasks.spec_bytes`, a person's other words
by `people.words`.

An application composing its own effective `jig_domain_core::Limits` calls
`jig_protocol_config::check_effective_limits(&limits) -> Result<(), Problem>`
on the limits it will hand the store. It may derive its byte budgets within
`policy_bytes_max(&limits) -> Option<u64>` and
`amendment_bytes_max(&limits) -> Option<u32>`, exported by the same crate.
Each maximum holds every other limit unchanged; `None` means even zero
for that budget cannot be admitted. These are bounded searches over the
codec's admission predicate, so the codec remains the arithmetic's owner.
If both budgets exceed the envelope, the check diagnoses policy first,
isolating amendment to zero in probe copies only, then amendment on a
subsequent check. An independent ceiling is diagnosed before either.

Problems name their effective fields, or the original override member
when admission read one. `What::PolicyEnvelope` and
`What::AmendmentEnvelope` retain the configured value, maximum and the
binding record field's required bytes and ceiling. The binding field is
the generated first failure at maximum plus one; its required bytes are
read at the configured value. An arithmetic refusal has no representable
required bound. `What::CodecCeiling` retains the generated first independent
people failure or a task batch field's configured count and schema ceiling, without labeling item counts as bytes; `What::CodecArithmetic` and
`What::StoreCodec` distinguish arithmetic and a combined-codec failure
outside the people envelope.
`Problem::codec_message()` gives these problems' words as owned bytes.
The policy diagnostic includes its binding field; amendment's words are
short even though its structured problem retains the bound:

> limits.core.policy_bytes: 65536 exceeds the people-v14 codec envelope of 43695 bytes (PolicyValue.permissions would need 98304 bytes; ceiling 65536).
>
> limits.people.amendment_bytes: 8193 exceeds the people-v14 codec envelope of 8192 bytes.

Program-source capacity is per worker, independently of its run slots.
It bounds whole source lists, inventories and pending sign-outs on both
channel ends. The small profile reserves twelve sources (four accounts
with three sources each); status/sign-out failure detail uses the host's
bounded detail. Source-check intervals and command deadlines must be
positive, and source commands and their protocol resources are bounded by
the same capacities even when every run slot is occupied.
