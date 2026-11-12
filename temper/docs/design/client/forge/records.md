# The forge's records

`temper-client-protocol-wire`, generated from one schema (jig's
`protocol/client/wire.md`, section 1, and `jig/docs/design/protocol/client/README.md`, section 8):

- **`FORGE`:** the forge connector's number, 1, which is the family's
  owner number on the wire, under which its records travel as bytes
  (jig's `protocol/client/README.md`, section 8), and the engine's one
  configured connector (`engine.connectors`,
  `../../protocol/configuration/README.md`, section 2).
- **`Owner`:** temper's owners, one enum the engine's root and the
  client root share, with the functions from an owner to its number and
  back (jig's `client/domain/root.md`, section 15): `Forge`, `FORGE`'s
  number; `Pages`, temper's pages, 2; and `Runner`, the agents' tools'
  runner (`../ctl/README.md`, section 4), 3: numbers no connector takes, never
  reused. `Pages` and `Runner` ask only for jig's operations, reads and
  subjects, so no request, read or subject of their own reaches the
  engine.
- **`CLIENT_VERSION`:** the version of every record below, 1 at first,
  2 since the playground's read, 3 since a change's panel, 4 since a forge's
  set up and the deployment's forges, 5 since jig's client wire 13
  carried in its refusals, 6 since the first repository (`SetUp`'s
  fields and the row's `first`), 7 since the playground's state (its
  read's `state` and its start), 8 since a sign-in's return, since gone,
  9 since connecting a GitHub App (`Connect`,
  `Install`, `ForgeRefusal`), 10 since the forges row's installations
  and last connect's end, 11 since GitHub's manifest (`Connect`'s
  `GitHubManifest` and the row's `app`). It stays at 11 until the first
  release, as `../../protocol/README.md`, section 7 says (skein's
  `data/codec/README.md`, section 5). A request, an outcome, a read, a subject or a
  field added to the family (the FORGE requests of `../engine.md`, the
  picker's ids in `Adopt`, the forge's kind in the adopted outcome, the
  playground's read, a change's panel) changes its schema, generated code and
  one set of golden bytes in place. The version moves at a release.
  The engine serves exactly one version (jig's `protocol/client/README.md`, section 6): a client of another
  is out of date for the forge (`family.md`, 4), and its kept adoption is dropped
  as kept work at start (`../line.md`, 6; jig's `protocol/client/README.md`, section 7).
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
- **`Adopt`:** the project (u32); the forge's number
  (u16); the owner, the name, the prefix and the landing branch (byte
  strings); the role (owned, fork or context); home and ci (booleans);
  the checks (a list of u32); and, when the owner and name are a picker
  row's, the forge's id for the repository and its owner's id (u64
  each, both or neither).
- **Its outcome,** `Answer`'s:
  - *adopted,* with the repository as the forge connector recorded it:
    its forge's number, its kind and its id; its owner and name; its role;
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
  with no project (`../../protocol/setup/owned.md`, 7): `empty`, `FirstEmpty`, the
  org and the name (byte strings, bounded as `Adopt`'s owner and name), made
  unadopted, and `playground`, a bool, the repository made as the
  playground's copy, `playground/widget` with its files, the org and name then
  `playground` and `widget`; `copies`, `CopyAddresses`, at most
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
  5), freeing its place in `forge.makings`, and is answered *kept*; with no
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
  held or full (`../../domain/forge/adoption.md`, 5); or *unknown*, as `SetUp`'s.
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
  (u16) (`../../domain/forge/adoption.md`, 1). Its outcome: *removed*, the forge
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
  at `CLIENT_VERSION` (jig's `protocol/client/README.md`, section 7).
- **The FORGE requests and their outcomes:** `Connect`, `Install`, `Accept`,
  `Replace`, `Forget`, `SetUp`, `Reachable`, `RemoveForge`, `Dismiss`,
  `StartPlayground` and `EndPlayground`, beside `Adopt` (`../engine.md`).
- **The playground's read, `playground`:** of the deployment's scope,
  with no fields, and no subject: what it answers changes with the
  `Playground` row, made by a start and erased by the reset (`family.md`, 7;
  `../../protocol/setup/playground.md`). Its answer is none, with no row, or
  `PlaygroundAnswer`: its forge's number (u16); its project (u32),
  once its start has made it, else none; its model's LLM provider
  (u32); its repository's owner and name (byte strings, bounded as
  `Adopt`'s); its state, `PlaygroundState`, starting, running or
  resetting; and its cues, at most 8, each 1 to 256 bytes of text, in
  the configuration's order (`../engine.md`). On the wire it is
  `Reading`'s `playground`, `Reading` naming which of the family's
  reads a read is; none is the read's empty answer.
- **The deployment's read, `deployment`:** of the deployment's scope,
  with no fields and no subject: what it answers is the document's, so
  it changes only with a start. Its answer is always a record,
  `DeploymentAnswer`: `welcome` (bool), whether the document shows the
  welcome (`../../protocol/configuration/engine.md`); and `owned`
  (bool), whether Forgejo on this machine can be set up: the document
  has `application.machine.owned` and the engine runs under the box
  (`../../protocol/setup/box.md`, 4). A first run's store keeps no forge, so
  no row of the `forges` watch can say so (`deployment.md`), and a wait
  for a host reads it as the fact of a box (`../pages/accounts.md`, 2); and
  `program_login` (optional bytes, at most 8,192), *the printed command*
  below. On the wire it is `Reading`'s `deployment`, beside `playground`.
- **The printed command:** what the pages print works when pasted. Where
  the deployment's one worker is on the engine's machine
  (`../../protocol/configuration/README.md`, section 1, the standalone shape),
  `program_login` is that machine's whole command but NAME, the running
  binary's word and the three directories in effect, each as a word
  (*The words of a command for this machine*): `~/temper-box program-login
  --config '…' --data '…' --cache '…'`. The pages print it with NAME after "On this machine,
  run:". It is a path of a machine that the party signed in to it
  may read: a read carries a sign-in (jig's `protocol/client/README.md`), and no
  answer goes before one. A path that isn't UTF-8, a line past its
  bound, and a deployment whose workers are elsewhere send none: the pages
  then print the placeholders the tools do, `temper-box program-login --config
  CONFIG --data DATA --cache CACHE NAME`, "On each host, with its
  directories:". Once the owner's `program_environment` answer comes, it
  replaces this command: an empty answer prints `temper-box program-login NAME`,
  and any other prints its words followed by `program-login NAME`.
- **The program's environment, `program_environment`:** of the deployment's
  scope, with no fields and no subject: what `temper-box program-login` needs
  around it to act on this deployment, from this machine's shell. It is the
  party's read, and only the deployment's owner has it, at any time, since the
  welcome's Model step shows them the command during set up and Connect a model
  after it; anyone else's is refused, and their page prints the command without
  its words. Its answer is
  the record `ProgramEnvironmentAnswer`: `environment` (bytes), the words the
  command starts with, which a page draws before `program-login NAME`. They are
  none when `temper-box` runs this binary and each of the three directories
  (`../../protocol/configuration/directories.md`) is the one the command finds by itself
  in the environment the engine started in. Otherwise they are the word that
  runs the binary, after `TEMPER_CONFIG=…`, `TEMPER_DATA=…` and `TEMPER_CACHE=…`,
  in that order, each only where its own directory isn't that one: a box with
  its data alone moved says `TEMPER_DATA=~/x temper-box`. Where that environment
  gives the command no directories by itself, no `HOME` or XDG variable placing
  them, all three are named. It is one string, composed where the
  binary and the directories are known, so that the client holds no path as a
  fact, quotes nothing and decides no default; it is a read of its own,
  separate from `DeploymentAnswer`. A page asks when its address names a
  program's way, the only place that draws the command with these words;
  Accounts' has none of them (`../README.md`, section 5). On the wire it is `Reading`'s
  `program_environment`.
- **The words of a command for this machine:** how `program_environment`'s are
  made.
  - *The word that runs the binary.* The running process's executable, less the
    ` (deleted)` Linux adds to one rebuilt while it runs. The word is `temper-box`
    when the first executable file named `temper-box` on the process's `PATH` is
    that file, by canonical paths: only the first counts, as that is what a
    shell runs, and an entry of `PATH` that isn't absolute counts for nothing,
    since the person's shell stands elsewhere. Otherwise it is the executable's
    path, as a word (below). An executable that can't be read gives `temper-box`.
  - *A path as a word,* for the executable and the three `TEMPER_` values alike:
    repeated slashes collapsed, then a leading `$HOME`, whole components of it,
    as `~/`, and the path alone as `~`. It stays unquoted where it can, so that
    the shell expands `~/`, which it does right after `=` in an assignment
    prefix too. A path needing quotes quotes only what follows `~/`
    (`~/'my dir/temper-box'`), and whole where no `$HOME` leads it (`'/my dir/temper-box'`).
    `$HOME` unset, relative or `/` leads nothing.
  - *The bound.* The words fit the answer's 49,216 bytes: a word for the binary
    that would pass it gives way to `temper-box`.
- **A change's panel, the subject `change`** (`served.md`, 2): of a
  project's scope, its window unused, naming the change task's number
  (u64). On the wire it is `Watching`'s `change`, `Watching` naming
  which of the family's subjects a watch is, as `Reading` names a read.
  Its document, `ChangePanel`:
  - the task (u64); the repository's forge's number (u16), its owner
    and name; the change's branch and its landing branch (byte strings,
    each bounded as `Adopt`'s); its head: none before its branch is
    pushed, or a commit (32 bytes, as the forge connector keeps one);
  - its pull request: none yet, or its number (u64) and its address on
    the forge (a byte string of at most 2048 bytes, empty for a forge
    with no address: GitHub's `web`, Forgejo's `git` base, none for a
    fake, `served.md`, 2);
  - its step: producing, opening, recreating, reopening, checking,
    gating, queued (its place, from 1 (u32), and the tasks ahead of it,
    nearest first, at most 64 (u64 each)), first, updating, resolving,
    repairing (its reason: CI, a gate by its number (u64), a semantic
    conflict, or a person's request for changes on the forge,
    `../../domain/forge/change/gates.md`, 1), landing, or landed (its merge commit, 32 bytes); verifying,
    red and cleared also carry the merge and at most 32 tier strings of at
    most 8192 bytes each. A tier string names its run and every full failing
    ID owned by or suspected of that landing, or explicitly `many tests`.
    The bound holds the 16 admitted IDs of up to 256 bytes each, their
    separators and tier/run words; IDs are never cut to fit the wire;
  - its hold: none, or why: stalled; past its repairs, its resolutions
    or its updates; its branch moved; its branch missing; its pull
    request closed; its pull request retargeted; a delegate or effect
    failed; a person rejected a gate. A held change's step is the one it
    was held at;
  - the task its step runs: none, or its number (u64);
  - CI at its head: unknown, pending, passed or failed;
  - its gates, at most 64, each with its number (u64), its kind (review,
    approval or check), whether it blocks (bool), and its verdict at the
    head: none, pending, approved, or changes asked;
  - its repairs, its resolutions and its updates, each used of its
    limit (u32 each);
  - its questions (`../../domain/forge/change/questions.md`, 1): its route, none
    or up; the questions taken, used of its limit (u32 each); and its
    open questions, at most 8 in the order taken, each with its asker's
    task (u64) and the question's number there (u32), the first line of
    its words (at most 256 bytes, cut on a UTF-8 boundary and marked),
    and where it stands: taken, asked
    of the change's requester, or answered and held for the change's
    dependencies. A change past its bound
    shows the question that held it among them, as its hold says.
- **The deployment's forges, the subject `forges`** (`deployment.md` and `served.md`
  ): of the deployment's scope, with no fields, its window
  unused; on the wire `Watching`'s `forges`. Its document, `Forges`, a
  `ForgeRow` for each forge the store keeps and has not ended, at most
  `FORGES` (16), in number order, the playground's fake among them while
  it runs (`../../protocol/forge/engine.md`, 1): its number (u16); its canonical resource `host` (bytes, at most
  2,048, from the git authority including its explicit port, the same
  identity adoption uses); its kind (Forgejo or GitHub); its mode
  (connected, managed, owned or fake); and its connection: not
  connected; not set up; setting up, with its step when the box has
  named one (fetch, prepare or run), and while it fetches and the box
  has said how far, `fetched`: the binary (Forgejo or its runner), the
  bytes written (u64) and the size (u64), if known
  (`../../protocol/setup/owned.md`, 6), which a client reads as none where the
  size is zero or the bytes pass it, and which changes in place, no
  version kept for an older side; set up failed, with its optional step (fetch, prepare, run or connect)
  and why (text of
  at most 256 bytes, the box's line bound); connecting; connected, with
  the user's id (u64) and login (a byte string bounded as `Adopt`'s);
  or stopped, with the component's reason;
  `admin`, whether the forge is managed, its connect taking a site
  admin's token; `read_only`, its App's read-only choice
  (`../../protocol/setup/github.md`, 7), false on Forgejo; `ended_stop`, none
  or `AdminStop`: not admin, refused with status (u16) and `SetupStage`
  (admin, bot, token, org, repository, file, app, installation or state),
  unmarked, or other. The diagnostic `ended` stays separate; the client
  selects its stop's words from this typed fact, never its text;
  its first repository, `first`: none, or the set up's org, name,
  whether it is the playground's copy (`playground`, appended), runner
  label (at most 64 bytes) and its state, `FirstState`, with no project:
  making; made, with the repository's id (u64);
  or failed, with its step, `CreateStep`: admin, org, repository (its
  bot's access with it) or file (`../../protocol/setup/owned.md`, 7); its
  intent, `starts_with`, `StartsWith`: `repository`, that `first` or none
  known yet; `copies`, `CopyAddresses`, the set up's, kept after they are asked;
  or `nothing`; its
  installations, `installations`, on GitHub, at most
  `INSTALLATIONS` (64, the configuration's `forges.installations` at
  most it) in id order, empty on Forgejo, each
  `Installation`: its id (u64), its account, none until the forge has
  heard it or its id (u64) and login (a byte string bounded as
  `Adopt`'s), whether it is suspended and whether it is a stray, never
  accepted through temper's link (booleans); its last connect's end,
  `ended`: none, or `ConnectStopped`, why the run of a `Connect` or an
  `Install` stopped, text of at most 256 bytes, in the setup host's
  words (`deployment.md`); on GitHub, its App's way, `app`: none,
  `Manifest` or `Pasted`, how the App its records hold was connected
  since the engine started, the forge domain's memory alone, so none
  after a restart and while no App is connected (`deployment.md`,
  Connecting); its `web` address, none on a forge with none; the
  repositories adopted on it, `repositories`, at most 16, each
  `Adoption`, its owner, name and project (u32); on an owned Forgejo,
  `owned`, `OwnedForge`, the Forgejo and runner versions the box pins
  (at most 64 bytes each, the runner's optional) and the path of the
  person's first password's file, never the password (at most 512
  bytes; `deployment.md`); `makings`, the repositories being made or
  failed on it, at most `MAKINGS` (64) in key order, each
  `RepositoryMaking`: its org and name, its `from` and `into`, and its
  state, `MakingState`: waiting; making; made, with the id (u64) and the
  branch (bounded as `Adopt`'s landing branch); adopting, with both; or
  failed, `MakingFailed`, with its step (`MakeStep`: admin, org,
  repository, file or adoption), its reason (`MakeFailure`: taken, empty,
  timed out, refused with status (u16), not allowed or other) and the
  forge's words (at most 256 bytes); and its
  offer, the forge domain's (`../../domain/forge/adoption.md`, 1): offered, not
  under the box, or not offered, which is what "Set up a Forgejo here", "Set up again"
  and the words without the box follow (`deployment.md`). Beside the rows,
  `linked`, whether the printed link has signed a person in since the
  engine started (jig's `protocol/client/sign-in.md`, `LinkSignedIn`):
  the clients' component's memory alone, as `app` is the forge domain's, so
  false after a restart; the engine delivers the document again when it
  turns true. A TUI in front of a box reads it for its welcome's "Signed in
  as the owner" (`../tui/box.md`). The read of the forges, the
  tools' `forge list` (`../ctl/areas.md`, area 6), is the watch's first
  document. Text prints `host: HOST`, and JSON carries `host` unchanged
  for a charter document's `forge` (`../../protocol/configuration/settings.md`, 6).
  *The client's forge side watches it once,* for the
  session, from when `me` is first delivered while the link is live,
  for every person, so that the engine judges the role: a watch
  refused or ended is kept as such, with no rows, and not opened again
  until the next sign-in. Its rows feed the Forges page,
  `ConnectForge`, the welcome and its store's state (`../pages/README.md`), and the
  Forges entry's dot.
- **A person's Away switch, the subject `away`:** of the party's own scope,
  with no fields, its window unused; on the wire `Watching`'s `away`, one
  subject for the deployment and each watcher's own rows, as `dismissed`
  (below). Its document, `Aways`: `rows`, at most 1,024, each
  `AwayRow { project, since, until, landed, off }`. *The client's forge
  side watches it once,* for the session, as it does `dismissed`, a watch
  refused or ended kept as such, with no rows: then the switch draws off and
  its act is not offered.
- **What was decided for a person, the subject `decided`:** of the project's
  scope, on the wire `Watching`'s `decided`, `DecidedSubject { project }`, its
  window unused. Its document, `Decisions` (`../../domain/forge/attention.md`, 5):
  `project`; `rows`, at most 128, each `Decision { number, task, at, question,
  took, why, overturned, reach }`, `took` the option's words; `asked`, at most
  128, each `Asked { number, task, at, reach }`, a call that was answered ask;
  each list the asking party's alone, newest last; and `cut`, whether either
  left older rows out. `reach` is a `Reach` (`../../protocol/configuration/settings.md`,
  2): the kind, the class, the reason, the item used, the sets touched, the
  patterns matched and whether Away was on. *The client watches it* per
  project while a page that shows it is open, at most `DECIDED_PROJECTS`, 4,
  at once (`../pages/home.md`, 1, *Decided for you*), and decodes each reach by the kit's
  `reach_from`.
- **A person's facts, the subject `dismissed`:** of the party's own
  scope (jig's `client/domain/contract/README.md`, 5.5), with no fields, its
  window unused; on the wire `Watching`'s `dismissed`. One subject for
  the deployment: the core names each watcher's party, and each
  watcher gets its own party's document (`../../domain/forge/attention.md`, 1). Its
  document, `Dismissals`: `dismissed`, the party's rows in their key's
  order, at most `DISMISSED` (8,192), each a `PersonFact`: `ToBegin {
  project }`; `GoalFrom { project, chat, goal, at }`, that the person started
  goal `goal` from chat `chat`'s document at the client's clock's `at`; or
  `KeepAsking { project, subject, at }`, Keep asking pressed on a need of
  that project for the item of `subject`, a `ClassSubject`, at the client's
  clock's `at` (`../../domain/forge/attention.md`, 1). A start whose
  `forge.judge_projects` is past an eighth of that bound is refused, a party
  holding at most one `ToBegin` a project (`../../protocol/configuration/limits.md`
  ). A party's rows are bounded by `DISMISSED` too, and all rows by
  `limits.dismissed()`: over either, `Dismiss` answers too large, and the link
  alone is lost: the goal has started, and the client draws no refusal for it. A
  fact reaches the watch of every client of the party's once it
  commits. *The client's forge side watches it once,* for the session, as
  it does `forges`, from when `me` is first delivered while the link
  is live, for every party: a watch refused or ended is kept as such,
  with no rows, and not opened again until the next sign-in. Its rows are
  the forge side's facts, which Home reads to compute To begin (`../pages/README.md`).
- **A project's changes, the subject `changes`:** of the project's scope
  (jig's `client/domain/contract/README.md`, 5.5), for a party with a role
  there, its window unused; on the wire `Watching`'s `changes`,
  `ChangesSubject`: `project`. `Changes`' page watches it while open. Its
  document, `ChangesPanel`:
  - `project`;
  - `landing`, the branch to name while nothing is queued: the newest
    landing's, else the default branch of the project's home repository,
    else of its first, else empty;
  - `queues`, at most 32, by repository then branch, one per landing branch
    with ready changes, a change owning its queue, or red (`README.md`, section 3), each a
    `ChangesQueue`: `repository`, `owner/name`; `branch`; `red`; and
    `changes`, in order (`../../domain/forge/queues.md`), each a
    `ChangesQueued`: `task`, `title` (at most 256 bytes), `ready_since`
    and, on a queue's first row, `checks_since`, when the step it is at
    began, both nanoseconds since the epoch;
  - `more`, the rows past `forge.changes_rows` (32 in `small`), counted
    across the queues, which fill in order; the changes queued are the
    rows listed and `more`;
  - `landed`, at most `forge.changes_landed` (8 in `small`), newest first,
    across the project's repositories, a row for each change of each
    landing, each a `ChangesLanded`: `task`, `title` and `at`, the landing
    log's (`../../domain/after-landing.md`, section 10); `commit`, the
    landing's, 32 bytes; `default_branch`, whether the branch the
    landing moved is its repository's default branch; and `signed`, at
    most 8, the landing log's sign-offs of the change, in its order, each
    a `ChangesSigned`: `party`; `head`, the commit their review named, 32
    bytes; and `at`, when they reviewed, nanoseconds since the epoch
    (`../../domain/forge/change/gates.md`, 2, *Sign-offs kept at
    landing*);
  - `landed_more`, what `landed` cut;
  - `landed_default`, the rows of `landed`'s kind whose `default_branch` is
    set, at most `forge.changes_landed`, newest first, so that landings
    into goal branches never push them out (`../pages/work.md`, 1, *Work*);
    a landing with no change, a push from outside temper, gives no row;
  - `landed_default_more`, what `landed_default` cut.

  It is marked as a change enters, leaves or moves in a queue, as the step
  of the change that owns one changes, as a branch's red changes and at each
  merge.
- **A project's landed drafts, the subject `drafts`:** of the project's
  scope, for a party with a role there, its window unused
  (`../../domain/forge/drafts.md`, 3); on the wire `Watching`'s `drafts`,
  `DraftsSubject`: `project`, appended after the family's last subject.
  Work watches it while open, as it does `changes` (`../pages/work.md`,
  1). Its document, `DraftsPanel`: `project`; and `drafts`, at most 64,
  newest `at` first, then the lower path, each a `LandedDraft`:
  - `forge`, u16; `owner` and `name`, its repository's, at most 1024
    bytes each; `path`, at most 1024 bytes;
  - `branch`, at most 1024 bytes, and `commit`, at most 64 bytes, the
    commit's id as the forge names it;
  - `title`, at most 200 bytes, empty with none;
  - `writer`, the party, u64; `at`, nanoseconds since the epoch;
  - `chat` and `goal`, optional u64s; `ready`, bool.

  It is marked at each row written or erased.
- **What changed since the person last looked, the subject `status`:** of
  the project's scope, for a party with a role there, its window unused;
  on the wire `Watching`'s `status`, `StatusSubject`: `project`. `Home`
  watches it while open. Its document, `ProjectStatus`, is each watcher's
  own, the core naming the watcher's party (`../../domain/forge/attention.md`, 2):
  - `project`;
  - `landed`, u64, the changes landed into every landing branch of the
    project; `last_landed`, the time of the latest, option;
  - `brief`, option, `StatusBrief`: `sentence` (at most 200 bytes), `lede`
    (at most 600), `at`, and `landed`, the count it covers;
  - `previous`, option, the party's last look of the sitting before; and
    `seen`, u64, the `landed` count then;
  - `queued`, u32, what the subject `changes` counts; `red`, whether the
    home repository's landing branch is red; and `branch`, its name, at
    most 1024 bytes, empty with no home.

  Times are nanoseconds since the epoch. It is marked with the subject
  `changes`, and at a look and a brief.
- **The changes waiting on a person's review, the subject `approvals`:** of
  the party's own scope, with no fields, its window unused; on the wire
  `Watching`'s `approvals`. One subject for the deployment: the core names
  each watcher's party, and each watcher gets its own rows, which the root
  chooses by the watcher's role (`../../domain/forge/change/questions.md`, 2). Its document,
  `Approvals`: `rows`, at most 32, oldest wait first, each an `ApprovalRow`:
  - `task`, the change's; `project`; `goal`, option, the goal it was made under;
  - `forge`, u16; `kind`, `Kind` (`forgejo` or `github`); `owner` and `name`,
    at most 1024 bytes each; `pull`, the pull request's number; `address`, at
    most 2048 bytes, its files page on the forge's web address
    (`README.md`, 4.1), empty for none;
  - `title`, at most 256 bytes; `since`, when the change began waiting at
    its head;
  - the approval it waits on: `scope`, 0 for the deployment's rules or the
    project's number; `rule`; `role`; `required`; `counted`, the people
    counted so far;
  - `behind`, the changes waiting behind it in its landing branch's order;
    `whole`, whether its reviews were read whole;
  - `yours`, `ApprovalYours`: `counts`, `no_account`, `too_few` or
    `other_role`, whether the watcher's review would count and, if not, why;
  - `size`, option, last, `ApprovalSize`: the pull request's size at the
    head its reviews were read at, as the forge counts it
    (`../../domain/forge/change/questions.md`, 2): `files`, `additions`
    and `deletions`, u32 each, a count past 4294967295 kept at it; and
    `cut`, bool, whether one was; none where the forge gave none;

  then `more`, the rows past the bound. Times are nanoseconds since the
  epoch. Every client of the party's watches it once, for the session (`../pages/needs-you.md`, 1, *What the client
  keeps*). The engine fills `kind` and `address` from the
  forge's configuration as it does a change's panel's pull request's
  (`served.md`, 2).
- **A path, `BranchPath`** (`../../domain/forge/change/branch-path.md`): `repository`, option,
  `owner/name`, none for a project with no repository; `branch`; `stops`,
  at most 24, in order, each a `Stop`:
  - `actor`, `Actor`: `person`, `agent`, `code` or `bar`;
  - `step`, `PathStep`: `ask`, `draft`, `plan`, `produce`, `produce_many`,
    `check`, `review`, `repair`, `approve`, `conflict`, `resolve`, `merge`,
    `catch_up`, `prepare`, `prove`, `turn`, `verify`, `branch`, `design`
    or `close_out`, the last two a goal's coordinator's
    (`../../domain/forge/change/branch-path.md`);
  - `words`, at most 64 bytes, as the path draws them ("you ask",
    "produce ×n", "an owner approves", "main");
  - `needed`, whether the stop is made only when needed, drawn outlined;
    `over`, at most 16 bytes, the small words over its own ("tests,
    review", "into goal"), empty for none (`../../domain/forge/change/branch-path.md`,
    *Drawn apart*);
  - `opens`, `Opens`: `nothing`, `charter` (`charter`), `rule` (`kind`,
    `scope`, `number`), `gate` (`number`, and `charter`, its reviews'
    charter, the role its card opens), `tier` (`name`), `queue` or
    `turn`;
  - `card`, `Card`: `who`, at most 128 bytes, as "an agent · holds a
    change until it approves", or a role's "For each chat: talks with
    you and asks for the changes"
    (`../../domain/forge/change/branch-path.md`, *The project's roles*); `name`, at most 64, as "the reviewer",
    "owner", "CI" or "main"; `facts`, at most 4, each a `Fact`, `label`
    at most 32 bytes and `value` at most 184, as "Model" and "Opus `../pages/accounts.md`, 2 ·
    high", "Can", "Asked to": the engine makes every value at most 128
    bytes but an agent's "Asked to", its instructions' first sentence
    in quotes, at most 184 (`../../domain/forge/change/branch-path.md`), so that the worst
    ways document and the worst view answer still fit one decision's
    delivery in the smallest profile.
- **A project's ways, the subject `ways`:** of the project's scope, for a
  party with a role there, its window unused; on the wire `Watching`'s
  `ways`, `WaysSubject`: `project`. The root makes its document, since
  roles, charters and models are the core's (`../../domain/forge/change/branch-path.md`).
  Its document, `WaysPanel` (`../../protocol/configuration/settings.md`, 8):
  - `project`; `ways`, at most 4, the stock's in their order, each a
    `StockWay`: `name`, at most 64 bytes; `headline`, at most 120, How
    work runs' heading while the project runs it; `line`, at most 256; `path`, its
    path over the project; `items`, at most 8, each a `ProposalItem`, what
    "Use this" sends; `delegation`, whether the project's policy lacks the
    delegation the way needs (13.10 there);
  - `current`, option, the index of the way the project runs; `own`, the
    project's own path; `words`, option, the project's words, kind 10, a
    `ProjectWords`, `headline` at most 120 bytes and `lede` at most 480
    (`../../protocol/configuration/settings.md`, 2); `rules`, a `PathRules`,
    the project's rules by path (`../../domain/forge/change/branch-path.md`,
    *Rules by path*): `rows`, at most 9, each a `PathRule`, `paths` at most
    256 bytes, `review`, `sign_off` and `lands` at most 64 each, an empty
    `review` or `sign_off` for none; and `note`, at most 160, empty for
    none; no rows where the project holds no rule by path; `may_change`,
    whether the reader may change the project's settings, an owner of it.

  Marked at each committed change of settings, at each adoption into the
  project or removal from it, and at a change of any person's role in the
  project, since how many hold a role words its sign-offs. A
  path's bounds are set so that the worst document the root makes, the
  stock's ways and the project's own path, each at 24 stops of the
  largest agent's card, with its rules by path at every bound, fits one
  decision's delivery in the smallest profile.
- **A view, the read `View`:** `ViewRead`: `project` and `address`, at most
  512 bytes, a `view` block's line (`../pages/chat/conversation.md`, 3), read in the project's scope by a
  party with a role there. Its answer, `ViewAnswer`, holds a `View`: `way`,
  a `WayView`, `name`, `line`, `path` and `current`; `path`, a `BranchPath`;
  `proposal`, a `ProposalView`, `before` and `after`; `charter`, a
  `CharterView`, `charter` and its `card`; `rule`, a `RuleView`, `kind`,
  `number` and its `card`; `roles`, a `RolesView`, `roles`, at most 16,
  each a `CharterView` (`../../domain/forge/change/branch-path.md`, *The project's
  roles*); or `not_drawn`, `words`, at
  most 512 bytes, saying in a line why it is not drawn.
- **A proposal's items, the read `ProposalItems`:** its answer,
  `ProposalItemsAnswer`, holds the items and `owned_approval`, whether one of
  them is a landing rule with an enforcing approval covering a writable
  repository on an owned Forgejo (`../../domain/forge/change/questions.md`, 2), for its card's
  words (`../pages/settings.md`, 2).
- **The forge's documents' records:** its other reads, subjects and
  documents, and its other requests, beside `Adopt` and `SetUp`
  (`served.md`).
- **Bounds:** the schema's ceilings bound every record at both ends,
  and the generated worst cases price each. Within them, the client
  bounds its fields by its own limits (`family.md`, 4), and the engine's forge
  connector refuses what is past its own as too large (`family.md`, 6), so a
  record a client can make always decodes.
