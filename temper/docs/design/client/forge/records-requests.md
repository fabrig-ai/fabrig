# The forge's request records

This document holds the forge's request records, split from [`records.md`](records.md).

## 1. Requests
- **The request, `Requesting`:** which of the family's requests it is,
  `Request`, as `Reading` names a read: `adopt`, `Adopt`, `set_up`,
  `SetUp`, `connect`, `Connect`, `install`, `Install`, `reachable`,
  `Reachable`, `accept`, `Accept`, `remove_forge`, `RemoveForge`,
  `make_repository`, `MakeRepository`, `dismiss_making`, `DismissMaking`,
  `dismiss`, `Dismiss`, `looked`, `Looked`, `set_draft`, `SetDraft`,
  `start_playground` and `end_playground`, the last two with no fields
  (below). Each is answered by the family's one record,
  `Answer`, whose outcome is one of either request's (below); the client
  refuses as undecodable an outcome that does not fit the request it
  sent.
- **`Checkpoint`,** with no field: a deployment owner's checkpoint of
  the serving deployment, answered `Checkpointed` with its name and
  figures or `CheckpointRefused` with why
  (`../../protocol/checkpoints.md`, 3).
- **`Adopt`:** the project (u32); the forge's number
  (u16); the owner, the name, the prefix and the landing branch (byte
  strings); the role (owned, fork or context); home and ci (booleans);
  the checks (a list of u32); and, when the owner and name are a picker
  row's, the forge's id for the repository and its owner's id (u64
  each, both or neither).
- **Its outcome,** `Answer`'s:
  - *adopted,* with the repository as the forge connector recorded it:
    its forge's number, its kind and its id; its owner and name; its role;
    its landing branch, `landing`, at most 1,024 bytes, empty for an
    outcome kept before it (`../../domain/forge/adoption.md`, 12);
    whether it is the project's home; what temper may do there (the
    forge's `Kinds`: read, push, open, land, review, status, comment,
    issue, branch); whether branch protection was read (unknown,
    absent, or a rule); its merge style; and the landing branch's rules
    as read, each with the setting that would let temper land where it
    keeps temper from landing (`repositories.md`, 1);
  - *refused by the core,* when the core refused the adoption the root
    made on the party's behalf: jig's refusal facts, carried whole as
    `jig-client-protocol-wire` encodes them, and shown in jig's words
    for its refusals;
  - *refused by the forge,* with the forge connector's reason: no such
    forge; not connected; not installed; not granted, with the
    installation's page; not found; no access; refused; in flight; too
    large; unreachable; invalid answer (`family.md`, 6).

  A refusal at admission (signed out, the party's role in the project,
  a key sent again with another request, a limit) is not an outcome:
  it is jig's answer, which never reaches the connector (jig's
  `client/domain/contract/requests.md`, 3), and the form shows it in jig's
  words.
- **The forge a request makes or names, `Making`:** `kept`, `Kept`, a
  kept forge's number (u16); or `new`, `NewForge`, a forge for the
  request to make, `ForgeAsk`: `github`; `owned`; `managed`,
  `ManagedForge`, a Forgejo of the person's connected as its admin:
  its API's host (at most 255 bytes) and port (u16), its TLS server
  name (at most 255 bytes, none for plaintext), its git base and its
  web address (at most 2048 bytes each, the web address optional) and
  its git program (at most 4096 bytes, none for the worker's); or
  `connected`, the same `ManagedForge`, a Forgejo of the person's
  connected with its bot's tokens, its mode connected
  (`../../protocol/setup/README.md`, section 4). A new forge is made as a record
  in the request's decision and numbered next, never reused
  (`../../protocol/forge/engine.md`, 1 and 2); the client finds its number
  on the `forges` watch's rows once the request is answered.
- **`SetUp`:** of the deployment's scope (`../engine.md`): the forge,
  `Making`, a kept owned Forgejo or a new one; and `first`, `SetUpFirst`,
  with no project (`../../protocol/setup/owned-repositories.md`, 1): `empty`, `FirstEmpty`, the
  org and the name (byte strings, bounded as `Adopt`'s owner and name), made
  unadopted, and `playground`, a bool, the repository made as the
  playground's copy, `playground/widget` with its files, the org and name then
  `playground` and `widget`, and `sample`, a bool, appended, the
  repository made as the sample with its files, the org and name then
  `work` and `hello`, as the welcome's sheet sends it; neither set, the
  starter workflow alone (`../../protocol/setup/owned-repositories.md`, 1); `copies`, `CopyAddresses`, at most
  `FIRST_COPIES` (16) addresses, each at most 512 bytes in the form the
  client's check gives (`../../draft/repository-copies.md`, 5.4), which make no
  repository at set up; or `nothing`. Its outcome, `Answer`'s: *setting up*, the box asked
  for the forge's group; *set up refused*, with why: not offered (the
  forge's connection does not offer it, or no such forge), not under
  the box, busy (`../../protocol/forge/engine.md`, 8),
  or invalid, a name Forgejo
  would refuse (the org a login of at most 40 bytes, the repository
  1 to 100 bytes of letters, digits, `.`, `_` or `-`, neither `.` nor
  `..`; skein `skein/docs/design/forge/setup/forgejo.md`, 1.9) or past the engine's
  `name_bytes`; or *unknown*, its
  outcome no longer held (`../engine.md`), the same unknown as an
  adoption's. A refusal at admission is jig's answer, as for `Adopt`.
- **`DismissMaking`:** of the project it names, or of the deployment with
  none (`../engine.md`), as `MakeRepository`'s `into` gives its scope, so that a
  key sent again has the same scope and is replayed once the row is gone: the
  forge's number (u16), the org and the name, bounded as `MakeRepository`'s,
  and `project`, none or a u32, the failed row's `into` as the client reads
  it on the `forges` watch. It erases a failed row (`../../domain/forge/adoption.md`,
  9), freeing its place in `forge.makings`, and is answered *kept*; with no
  row at the name, so that a request sent again joins, *kept* too; with a row
  whose `into` is not the project it names, refused as `not_owner`; with a
  row that is not failed, refused as `not_failed`.
- **`MakeRepository`:** of the project `into` names, or of the deployment
  with none (`../engine.md`): the forge's number (u16); the org and the name
  (byte strings, bounded as `Adopt`'s owner and name); `from`, `MakeFrom`:
  `empty`, or `copy`, `MakeCopy`, its address (at most 512 bytes), the
  form `https://HOST/OWNER/NAME` the client's check gives
  (`../../draft/repository-copies.md`, 5.4); and `into`, none or `MakeInto`:
  the project (u32) and `home` (bool). Its outcome, `Answer`'s, kept as an
  adoption's is (`../engine.md`): *kept*, the row written; *make refused*,
  `MakeRefusal`: no such forge, not offered, not owner, invalid, home
  held or full (`../../domain/forge/adoption.md`, 9); or *unknown*, as `SetUp`'s.
  A refusal at admission is jig's answer, as for `Adopt`. The course of
  what it asked is the forge's row's, never the answer's.
- **`StartPlayground`:** of the deployment's scope (`../engine.md`), with
  no fields: the start makes the playground's records
  (`../../protocol/setup/playground.md`). Its outcome, `Answer`'s: *running*,
  the `Playground` row running, its account kept and its repository
  adopted; *start refused*, not offered (a real forge kept); *start
  failed*, with why, `StartFailure`: the group's failure, with the
  box's `WHY` (a byte string of at most `box.line_bytes`), the account
  refused, the adoption refused, or the forge not connected in time;
  or *unknown*, its outcome no longer held (`../engine.md`), as `SetUp`'s.
  A refusal at admission is jig's answer, as for `Adopt`.
- **`EndPlayground`:** of the deployment's scope, with no fields: the
  playground's reset (`../../protocol/setup/playground.md`). Its outcome:
  *reset*, once the reset's last act has erased the `Playground` row;
  *end refused*, `EndRefusal`, not offered (no playground); or
  *unknown*, as `SetUp`'s.
- **`RemoveForge`:** of the deployment's scope, the forge's number
  (u16) (`../../domain/forge/adoption-connection.md`, 1). Its outcome: *removed*, the forge
  ended in the request's decision, its removal's kept acts following
  it; *remove refused*, `RemoveRefused`, with a `ForgeRefusal` (below):
  `role`, `no such forge` (none kept, or one ended already), or `not
  taken`, the playground's forge, which only its reset removes; or
  *unknown*.
- **`SetAway`:** of the project it names, from any party with a role
  there, as `Dismiss` (`../../domain/forge/attention.md`, 4): `project`, u32; `on`,
  bool; `until`, an optional u64, nanoseconds since the epoch as
  `ChangesLanded.at`. What Away may decide is the project's classes', not
  the switch's (`../../domain/forge/attention.md`, 6). Its outcome: *away*, `away`, no fields, once the row has committed;
  *forge refused*, `too large`, the family's bound reached; or *unknown*,
  no longer held, which the subject `away` answers.
- **`Overturn`:** of the project it names, from any party with a role
  there (`../../domain/forge/attention.md`, 5): `project`, u32; `number`, u32. Its
  outcome: *overturned*, `overturned`, no fields, once the row has
  committed; *overturn refused*, `overturn_refused`, `OverturnRefusal`:
  `already` or `unknown`; or *unknown*, no longer held, which the subject
  `decided` answers.
- **`Dismiss`:** of the project it names, from any party with a role
  there (`../../domain/forge/attention.md`, 1): `what`, `PersonFact`, a fact the party
  keeps: `to_begin`, `ToBegin`, the project (u32), "Skip" on that project's
  "To begin"; or `goal_from`, `GoalFrom`, the project (u32), the chat (u64), the
  goal (u64) and `at` (u64), that the person started that goal from that chat's
  document, at the client's clock.
  The root names the project as the request's scope, whatever scope the record
  came under.
  Its outcome: *dismissed*, `dismissed`, no fields, once the row has
  committed, and the same for a second, under any key, from any device;
  *forge refused*, `too large`, the family's bound reached (the forge
  reason of `family.md`, 6); or *unknown*, its outcome no longer held (`../engine.md`),
  as `SetUp`'s, which a client answers from the subject `dismissed`
  below. A refusal at admission is jig's answer, as for `Adopt`.
- **`Looked`:** of the project it names, from any party with a role
  there, as `Dismiss` (`../../domain/forge/attention.md`, 2): `project`, u32, the
  person entering that project's Home, not while To begin shows. Its
  outcome: *looked*, `looked`, no fields, once the row has committed;
  *forge refused*, `too large`, the family's bound reached; or *unknown*,
  no longer held, which the subject `status` answers.
- **`AddDevice`:** of the party's own scope, from a person
  (`../../domain/forge/attention.md`, 7.1): `endpoint`, 1 to 1,024
  bytes; `p256dh`, 65 bytes; `auth`, 16 bytes; `label`, 1 to 60 bytes of
  UTF-8. The browser takes them from its push subscription, the keys
  as `PushSubscription.getKey` gives them; `temper ctl push add` from a
  subscription's JSON (`../ctl/tools-acts.md`, 1). Its outcome: *added*,
  `device_added`, the device's number (u32), once the row has
  committed; *device refused*, `device_refused`, `DeviceRefusal`:
  `person` (the party is a service), `off` (this deployment doesn't
  push), `service` (the endpoint isn't a URL of one of the deployment's
  push services), `malformed` (a key or the label) or `too_many`; or
  *unknown*, no longer held, which the subject `devices` answers.
- **`RemoveDevice`:** of the party's own scope
  (`../../domain/forge/attention.md`, 7.1): `number`, u32. Its outcome:
  *removed*, `device_removed`, no fields, once the row's erasure has
  committed; *device refused*, `unknown`, the party holds no such
  device; or *unknown*, no longer held.

  Both requests' variants follow `FollowAct`'s, the last, in the
  order above, and their outcomes follow the last outcome's, so the
  records already on the wire keep their bytes.
- **`SetDraft`:** of the project it names, from any party with a role
  there, as `SetAway` (`../../domain/forge/drafts.md`, 2): `project`, u32;
  `repository`, an option of `DraftRepository`: `owner` and `name`, at
  most 1024 bytes each, none for the project's home; `path`, at most 1024
  bytes; and `change`, `DraftChange`: `ready`, `ReadyDraft`, with `chat`,
  an optional u64; `not_ready`, no fields; or `started`, `StartedDraft`,
  with `goal`, u64. Appended after the family's last request, so that
  every request before it keeps its tag. Its outcome: *drafted*,
  `drafted`, the row as it stands once committed, a `LandedDraft`
  (*A project's landed drafts*, below); *draft refused*,
  `draft_refused`, `DraftRefusal`: `no_repository`, `path`, `missing`,
  `not_a_file`, `unreachable`, `unknown` or `full`, as the domain names
  them; or *unknown*, no longer held, which the subject `drafts`
  answers. Both outcomes are appended after the family's last.
- **`FileFinding`:** of the project it names, from any party with a role
  there (`../../domain/areas.md`, section 7): `project`, u32; `kind`, a
  `FindingKind`, `qa`, `flake`, `red`, `review` or `feedback`;
  `severity`, a `Severity`, `low`, `medium` or `high`; `area`, option,
  an area's name, at most 32 bytes; `title`, 1 to 200 bytes; `body`, at
  most 4,096 bytes (`../../domain/findings.md`, section 2). Its outcome:
  *filed*, `filed`, `Filed { number }`, once the row has committed;
  *finding refused*, `finding_refused`, a `FindingRefusal` (below); or
  *unknown*, no longer held, which the subject `findings` answers.
- **`HandFinding`:** of the project it names, from the finding's holder
  or a project owner (`../../domain/areas.md`, section 7): `project`,
  u32; `number`, u32; `area`, at most 32 bytes; `why`, 1 to 200 bytes.
  Its outcome: *handed*, `handed`, no fields, once committed; *finding
  refused*; or *unknown*.
- **`TriageFinding`:** of the project it names, from the same: `project`,
  u32; `number`, u32; `act`, a `Triage`: `route`, the goal's task (u64);
  `backlog`, no fields; `drop`, its reason, 1 to 400 bytes
  (`../../domain/findings.md`, section 6); `repair`, `more`, the
  findings after `number`, at most 15 u32s, and `words`, 1 to 4,096
  bytes (`../../domain/red-repairs.md`, section 4); or `blame`,
  `landing`, option, a commit of 20 or 32 bytes, none for no landing,
  and `why`, 1 to 200 bytes (`../../domain/red-repairs.md`, section
  3). Its outcome: *triaged*, `triaged`, with `goal`, option u64, the
  goal a `repair` made, none for the other acts, once committed;
  *finding refused*; or *unknown*. `repair`, `blame` and `goal` came
  with the wire's next layout, unbridged (`../../protocol/wires.md`,
  section 9).
- **`TierActRequest`,** `tier_act`, appended after `TriageFinding`: of
  the project it names, from a project owner or the deployment's owner,
  as `FollowActRequest` (`../../domain/after-landing-runs.md`, section 2, *A
  person's hold*): `project`, u32; `repository`, an option of
  `DraftRepository`, none for the project's home; `tier`, at most 32
  bytes; `act`, `TierAct`: `hold`, `TierHoldAct`, its `reason`, 1 to
  1024 bytes, or `release`, no fields. Its outcome, `tier`, appended
  after the family's last, `TierOutcome`: `refusal`, an option of
  `TierRefusal`: `no_repository`; `no_tier`; `held`, `TierHeldBy`, the
  holding `party` (u64) and its `reason`, for a hold of a held tier;
  `not_held`, for a release of a tier not held; or `words`, the reason
  past its bounds or not UTF-8. None once the hold or the release has
  committed.
- **`OpenChange`,** `open_change`, appended after `TierActRequest`: of
  the project it names, from any party with a role there, the core
  judging whether the role may make the change
  (`../../domain/forge/change/opened.md`, 2 and 3): `project`, u32;
  `repository`, an option of `DraftRepository`, none for the project's
  only repository; `branch`, 1 to 1024 bytes; `at`, an option of 40 or
  64 bytes, lowercase hexadecimal digits; `into`, an option of 1 to 1024
  bytes, none for the repository's landing branch
  (`../../domain/forge/adoption.md`, 12); `title`, 1 to 256
  bytes; `body`, an option of 1 to 16,384 bytes; and `budget`,
  `OpenBudget`: `amount`, u64, or `as_allowed`, no fields. Its outcomes,
  appended after the family's last: *opened*, `opened`, `Opened`, its
  `task` (u64), once the core has made it; *open refused*,
  `open_refused`, `OpenRefused`: `several`, the repositories by owner
  and name, a list of at most 64 `DraftRepository`; `no_repository`;
  `not_a_repository`; `under_prefix`, `UnderPrefix`, the repository's
  `owner` and `name` and its `prefix`, at most 1024 bytes each; or
  `name_past`, the deployment's forge name bound (u32), for a branch or a
  landing branch past it; the family's *refused by the core*,
  `core_refused`, jig's refusal facts whole, for the core's refusals and
  the builder's (`../../domain/forge/change/opened.md`, 3); or
  *unknown*, no longer held.
- **`FindingRefusal`:** `no_area`, the project's areas' names, at most
  16; `full`, how many are `new`; `not_yours`, its holder, a `Holder`,
  `task`, `party` or `role` with its number; `no_finding`; `no_goal`, a
  route's task not a live goal of the project; `ended`, the finding
  already `routed` or `dropped`; or `words`, at most 1,024 bytes: a
  field past its bounds or not UTF-8, naming it, or a `repair`'s or
  `blame`'s refusal in its words (`../../domain/red-repairs.md`, 4.2
  and section 3).
- **`Connect`:** of the deployment's scope (`../engine.md`): the forge,
  `Making`, a kept forge or a new GitHub, managed or connected Forgejo,
  and how it connects, `How`: `github_app`, `GitHubApp`,
  the App's id (u64, never 0) and optional `read_only` (bool, absent
  meaning false), set by ctl connect `--read-only` and kept on the forge
  record (`../../protocol/setup/github.md`, 7); `github_manifest`,
  `GitHubManifest`; `forgejo_admin`, `ForgejoAdmin`; or
  `forgejo_tokens`, `ForgejoTokens`, with no field, a connected
  Forgejo's bot's tokens (`../engine.md`'s table). The App's key is no
  field: it rides beside the record as jig's carried value, a bundle of
  one `Pasted` item, and so do a Forgejo's tokens, a bundle of
  `ApiToken`, `GitToken` and `WebhookSecret` when given
  (`../../protocol/credentials.md`, 2.2). Its outcome,
  `Answer`'s: *connecting*, the run started; *forge refused*, with why,
  `ForgeRefusal` (below); or *unknown*, as `SetUp`'s.
- **`Replace`:** of the deployment's scope: the forge's number (u16).
  The new key or tokens are no field: they ride as its carried value,
  a bundle of one `Pasted` item, a GitHub App's new key, or of a
  connected Forgejo's `ApiToken`, `GitToken` or both, and
  `WebhookSecret` when it changes (`../../protocol/credentials.md`, 2.2).
  Its outcome is a `Connect`'s: *connecting*, the run started; *forge
  refused*, `ForgeRefusal`; or *unknown*. It takes a GitHub forge or a
  Forgejo of the connected mode, connected or stopped
  (`../../protocol/setup/README.md`, section 4, and `../../protocol/setup/github.md`, section 3).
- **`Install`:** of the deployment's scope: the forge's number (u16)
  and `back`, an optional byte string checked at decode (`../engine.md`). Its outcome: *a link*, `Link`, the install page's
  address (a byte string of at most 2048 bytes); *connecting*, an
  `Install` that stopped, its failure said on the forge's row; *forge
  refused*, `ForgeRefusal`; or *unknown*.
- **`Accept`:** of the deployment's scope: the forge's number (u16)
  and an installation's id (u64, never 0), one that exists already on
  GitHub (`../../protocol/setup/github.md`, 4). Its outcome: *accepted*,
  `AcceptedInstallation`, with the installation's id, once it is kept,
  and the same when it was accepted already; *forge refused*,
  `ForgeRefusal`, as `Install`'s, before any run; *unknown*; or, its run
  stopped, *forge refused*, `ForgeRefused`, with the reason nearest the
  stop: `not found`, GitHub having no such installation of the App;
  `no access`, GitHub refusing the App's key or its read; `unreachable`,
  GitHub down or past the run's deadline; `too large`, the accepted set
  full (`forges.installations`); `invalid answer` otherwise. The forge's
  row is unchanged by a stopped `Accept`.
- **`ForgeRefusal`,** a `Connect`'s, a `Replace`'s, an `Install`'s, an
  `Accept`'s or a `RemoveForge`'s, by the root before any run: `role`,
  the sender not the deployment's owner; `no such forge`; `not taken`,
  the forge's kind or mode does not take the request (a Forgejo's
  install, the playground's fake, a `Replace` on a managed or owned
  Forgejo, a `ForgejoTokens` `Connect` on a GitHub or managed forge);
  `connected`, a `Connect` on a forge connected or connecting already,
  whose key or tokens `Replace` replaces; `not connected`, an `Install`
  or an `Accept` on a forge with no App, or a `Replace` on a forge
  neither connected nor stopped, nor connecting; `busy`, a run holding
  the forge (a `Replace` on a connecting forge among them), or
  `setup.runs` full; `no key`, a `Connect` or a `Replace` that takes a
  carried value and has none; `first is bot`, a `ForgejoAdmin` `Connect`
  whose `first` organization is the bot's login, case-folded
  (`../../protocol/setup/reconcile.md`, 5). Its carried value, if any, is named unused
  (`../engine.md`).
- **The small store's record of a pending request** is `jig-client`'s
  kept request, its request encoded by this crate as the family's bytes
  at `CLIENT_VERSION` (jig's `protocol/client/small-store.md`, section 1).
- **The FORGE requests and their outcomes:** `Connect`, `Install`, `Accept`,
  `Replace`, `Forget`, `SetUp`, `Reachable`, `RemoveForge`, `Dismiss`,
  `StartPlayground` and `EndPlayground`, beside `Adopt` (`../engine.md`).

## 2. Open questions

None.
