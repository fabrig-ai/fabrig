# Adopting repositories

- **A repository joins a project with a role** (jig's `domain/model.md`,
  6.1): *owned*, where temper pushes its branches, opens pull requests
  and lands; a *fork* of an upstream (`README.md`, section 8); or *context*, read
  only. An owner of the project adopts it in the client, through the
  adoption form (`../../client/forge/family.md`, 4). A repository temper writes
  belongs to one project; others may adopt it as context, and each such
  adoption is kept for its project beside the writer's, so that the
  project's lookups, runs and follows read it (`follows.md`, section 8).
- **A repository is named by its owner and name, and kept by the forge's
  id for it.** The adoption names the forge, the repository's owner and
  its name; the connector first locates it, a read whose answer is the
  forge's own id for the repository, and names the repository by that id
  from then on (`Repository { forge, repository }`), so a rename or a
  transfer on the forge keeps it the same repository. A repository the
  forge does not have refuses the adoption as not found.
- **A task names a repository by its tag, the pair packed.** jig's
  resource number, opaque to jig, is temper's tag: `(forge << 32) |
  repository`, the forge's number (16 bits) in the high half and the
  forge's id for the repository (32 bits) in the low, packed and read
  back by one pair of functions in the forge's domain; a tag with a bit
  set above the forge's names none. A delegation's resource,
  a workspace's repository, what a run left and a pushed head carry it,
  and a saved resource's path is the pair itself (`../root.md`, section
  5), so two forges' repositories with the same id in one project stay
  two. A tag whose pair the task's project has not adopted names none.
- **The adoption request is the forge's own,** keyed as any party's
  request (`../../client/forge/family.md`, 4): the root takes it from the protocol
  layer typed, and the connector refuses a forge the deployment does not
  keep, and an ended one as not connected (1); then locates the
  repository (`Read::Locate { owner, name }`, answered
  `Answer::Located { repository }`); then the root makes jig's
  adoption request on the person's behalf (jig's `domain/people/acts.md`,
  1), in which the connector reads what temper may do there (below)
  before the core commits the adoption (jig's `domain/connectors.md`,
  12). A forge not connected refuses it first, as such (1); a managed
  or owned Forgejo's access is asked for before the reads (2). Its
  outcome is the adoption's: adopted, refused by the core with jig's
  reasons, or refused by the forge with the connector's. The
  connector keeps it under the people child's token until its retention
  ends, to give again when the request is sent again (jig's
  `domain/connectors.md`, 5.5; `../../protocol/store.md`, section 2).
- **The forge's host** in a repository's resource path, and in what a
  person reads of it, is the forge's as its record has it: the host of
  its `git` base, with `:port` when it names one
  (`../../protocol/forge/engine.md`, 1). The adoption does not ask a person for
  it.
- **What temper may do is read at adoption:** its user's permission on
  the repository; the repository's settings (the merge styles it allows,
  its default branch); the branch protection on the landing branch; and,
  but for context, the branches already under temper's prefix. Write
  permission, or admin, gives every kind of effect on an owned or fork
  repository, merging only where the repository allows a merge style;
  read gives reads alone; no permission refuses the adoption as
  forbidden. Forgejo refuses protection reads to a writer who is not an
  admin (skein's `forge/forgejo.md`, section 2): adoption records
  protection as unknown, never as absent, and its words say why
  (`../../client/forge/family.md`, 6). On GitHub temper's user is
  its App's bot, its permission the installation's on that repository,
  and protection the rules GitHub applies to the branch (4). A write
  the forge refuses narrows what temper may do there, for every later
  decision (jig's `domain/connectors.md`, 12).
- **Its CI workflows are read once it is adopted** (7): the names of
  the workflows on its landing branch, kept on its record and current
  with its landings, which CI's card names (`change/branch-path.md`).
- **The project's ceiling follows** (jig's `domain/authority.md`,
  section 6) from each repository's role and what was read: on an owned
  repository, pushing under temper's prefix, opening pull requests and
  landing into its landing branches; on a fork, pushing into the fork;
  on a context repository, reading. Protection that keeps temper from
  merging makes landing a wait for someone else's merge (`README.md`, section 7).
- **One deployment's prefix.** Adoption refuses a repository where
  branches under temper's prefix exist that the store records no making
  of: they are another deployment's (jig's `domain/connectors.md`, 12).
  A prefix whose branches' names would pass the authority's segments
  (`authority.segments`, counting forge, host, owner, repository and
  `branch` before the branch's own parts) is refused as too large.
- **People the forge knows.** Adoption reads the repository's
  collaborators, each with its permission (read, write or admin), and
  gives the human collaborators to the core as parties known by their forge
  account: provider 0, with the subject of the repository's forge and the
  collaborator's user (`../../protocol/store.md`, section 2). temper's own writer
  is excluded, and so is the user the forge's link names (6): the link's
  person is known there already. The
  project's configuration maps each permission to a role, given to those
  with no role in the project yet (jig's `domain/people/README.md`, section 4):
  temper's usual mapping is admin to maintainer, write to member, read to
  observer (roles 1, 2 and 3 respectively, owner 0 reserved for the project
  owner). Re-adoption seeds people still without a role; it does not replace
  existing project roles. After that, roles are the core's: a permission that changes
  on the forge changes no role.
- **Who a forge's user is: one lookup, by forge and user.** A user's number
  means nothing without its forge: GitHub's user 3 isn't Forgejo's, and anyone
  may post a review on a public repository's pull request. The root's
  `forge_person` answers the link's person when the forge's link names that
  user (6), else the party whose key is provider 0's subject for that
  pair, with their role in the project, else unknown. A party seeded for
  the link's user before the link was made stays, role and all, but is
  never answered on that forge: nothing merges. Every reader of a
  person's forge account goes through it: the reviews counted (`change/gates.md`, 2), the people
  eligible (`change/gates.md`, 3) and the inventory's people (`../agent/agents.md`, 1). Going the other
  way, `person_forge_user` answers the user a party is on a forge only where
  `forge_person` answers that party for that user: the link's user for the
  link's person, else the user their key names. So a seeded party shadowed
  by the link is known on no forge it's shadowed on. A party counts as a
  person on a forge, `forge_human`, when `person_forge_user` answers them a
  user there who isn't that forge's retained writer. The subject's form is
  known to the forge's `user_key` and `key_user` alone.
- **Access is given per repository, at adoption.** On a managed or
  owned Forgejo, the adoption asks for it first: temper's user made a
  collaborator with write, and the repository's hook made when the
  deployment has a webhooks' listener; a refusal refuses the adoption,
  named (2). On a connected Forgejo a person gives temper's user its
  access, as a collaborator or through a team, and adds its hook or
  leaves the repository to poll. On GitHub the App's installation on the
  repository's account grants it, and its hints come from the App's
  delivery log, with no hook per repository (`../../protocol/forge/README.md`,
  4.1). The adoption then reads what it was given as any other.
- **The adoption names, per repository:** its landing branch; temper's
  branch prefix, `temper/` by default, which protection can reserve to
  temper's user; whether it is the project's home, where its goals'
  issues are kept (`issues.md`), one per project and never a context
  repository; and whether it runs CI, and if not, the checks its changes
  are gated on (`change/gates.md`, 1). The merge style is the forge's for the repository,
  and the adoption shows it (4).
- **Nothing is required:**
  - no webhook: polling is the backstop, webhooks only cut the latency,
    and the client shows a repository whose hints never arrive;
  - no label: temper writes none, and decides nothing by one, but for a
    project that turns findings on (`../findings.md`, section 3);
  - no protection: temper's rules are its enforcement, protection a second
    net;
  - no CI: a repository without it gates its changes on checks its
    adoption names (`change/gates.md`, 1).
- **Provisioning** (creating a repository, its webhook, its protection)
  is an optional set of effects a person may ask for, never a step of
  adoption (jig's `domain/connectors.md`, 12). The access a managed or
  owned Forgejo's adoption asks for is the setup host's run, not an
  effect of the connector's (2), and so are the labels a project's
  findings use (`../findings.md`, section 2).

## 1. Connected or not, ended and removed

- **The root knows each forge's connection** from the routing's events
  (`../../protocol/forge/README.md`, 7.1): not connected; not set up, an owned
  forge the box starts when asked, whose files are absent; setting up,
  while the box runs its group, at the step it last named, if any; set
  up failed, and why; connecting, while a setup's run that connects it
  holds it; connected, as its user, by id and login; or stopped, and
  why. The connector keeps it as the root
  tells it, in memory: nothing of it is stored, and each start rebuilds
  it from the routing's events, every forge not connected until its
  `Connected`.
- **A failed setup names the step it held,** alongside the connection in
  `Row.failed`: setting up at fetch, prepare or run names that step;
  connecting names connect; already set up failed keeps its first step;
  every other held state names none. This rule also applies when the
  box pipe closes during setup. Every nonfailure state clears the step,
  so a retry that reaches setting up or connecting fails at its own
  step. Both fields are volatile; no store or record version changes.
  Making the first repository has its own `first.state` failure.
- **A forge not connected answers `NotConnected`,** at once, at cost 0:
  one stopped, never connected, or whose connect failed
  (`../../protocol/forge/engine.md`, 7).
- **A connecting forge is waited for, not refused.** While a forge
  connects (a start's connect, a set up's, a reconnect), a person's act
  on it (an adoption) and an agent's read or effect wait for the
  connection within the call's own deadline (ctl's patience, a host
  call's timeout), then go on as on a connected forge; past the
  deadline, or once the connect fails, they answer `NotConnected`,
  naming connecting or the failure. So an agent that follows the box's
  walk straight from its start to an adoption is not refused for the
  moment the start takes to connect.
  A read it answers is final; a write's entry fails at once, as a typed
  refusal does, having reached no forge (skein's `forge/client.md`,
  sections 5 and 7). The connector takes it as the forge's access
  lacking, never as a fact of a repository: nothing is narrowed, no
  change is held, and no effect is found again for it.
- **What waits on it.** The connector steps no change on a forge not
  connected, and a stall does not count that wait (`change/repairs.md`); an effect whose
  entry failed `NotConnected` is committed again once the forge
  connects, since it reached no forge. A change's step on such a forge
  (a restart's, a wake's, a release's) sends no read, and a read of a
  step answered `NotConnected` (its branch's, its base's, its pull's or
  any later one, a connecting forge's past its deadline among them) ends
  that step with nothing decided, its partial reads dropped: either way
  the change waits for its forge, holding nothing (no
  `ResourceUnavailable` or `ResourceRefused` hold) and polling nothing.
  The forge's `Connected` steps each change waiting on it afresh, from
  its branch read, and takes the time it waited off its stall as an
  outage's unanswered time is (`keeping-up.md`, 5), so what the stall had
  counted before the wait stays counted. Lost writers' reads and a goal
  branch's reads wait for it the same way (`branches.md`, 1). The client shows why on the
  forge's row (`../../client/forge/README.md`, the deployment's page).
- **The offer to set a forge up,** decided here alone. The connector
  keeps each forge's record, its number, kind, mode and setup start
  among it (`../../protocol/forge/engine.md`, 1), and the host tells it, at
  construction, whether serve runs under the box (`--box`,
  `../../protocol/setup/box.md`, 1). One function gives a forge's offer from
  those, the box and the forge's connection: *offered* for an owned
  forge whose setup says `"start": "asked"`, under the box with its pipe open, its connection not set up,
  setting up or set up failed (setting up so that its row keeps showing
  the set up under way); *not under the box* for such a forge, in such a
  state, without the box or once its pipe closed; *not offered*
  otherwise, and for a forge not kept or ended. The row and the answer
  read the same function, so they never disagree. The same function gives
  each action's offer from the forge's kind, mode, setup and connection:
  the engine domain decides every action a row offers, including
  "Connect as its admin", which needs a managed setup entry
  (`../../protocol/setup/reconcile.md`, 5).
- **Setting up a forge,** `SetUp { forge }` (`../../client/engine.md`
  ): the root gives it to the connector, which refuses it by
  the forge's offer when not offered, "not offered" or "not under the
  box"; "busy" while any forge is setting up, itself included; and
  otherwise asks the routing's box link to ask the box
  (`../../protocol/forge/engine.md`, 8). The link decides nothing: it carries
  the ask out and the box's lines in, which the routing tells the
  connector as connections, and the box's answer back. The pipe's close
  comes to the connector as the root's `BoxClosed`: each offer becomes
  not under the box, and a forge setting up becomes set up failed, "the
  box closed the pipe". The connector changes no state at the ask: the
  routing tells it each, as it does the others: at the box's `started`,
  connecting, then connected, or set up failed with the engine's step
  that stopped (`../../protocol/forge/engine.md`, 8), offered again. `SetUp` is
  a keyed request, admitted by the core first (`../../client/engine.md`
  ): the connector keeps its answer as its outcome, in memory
  alone, the latest per kept forge under its request's token and
  one more for any forge not kept, and answers unknown for a token
  it no longer holds.
- **The deployment's forges,** the subject `forges`
  (`../../client/forge/deployment.md`): the connector makes its document, one
  row per kept forge in number order, but an ended one (below),
  each its number, kind, mode, connection as the routing last told it,
  offer, first repository, on GitHub its installations as the routing
  last told each (`Event::Installation`: its id, its account when
  heard, suspended, stray), in id order, and its last connect's end:
  none, or the reason a `Connect`'s or an `Install`'s run stopped with
  (`ConnectEnded`, `InstallAnswered`), held until the forge's next
  connect or install is admitted or it connects; all its own
  (nothing is added where it is encoded), and
  marks the subject at each connection, installation and connect's end
  it is told and at `BoxClosed`, so that a watch
  follows a set up through its steps and a connect to its end. Nothing
  of it is kept. A restart starts every row with no connect's end, and
  with the installations the forge's records accept (its `Accepted`
  ids), each told by the setup host at the restored connect, its
  account none, neither suspended nor stray, since the component's
  first read tells only those not accepted or changed (skein's
  `forge/connection.md`, 4.4); the account is filled from the first
  `GitToken` minted for it (`keeping-up.md`, 3 there), and changes follow as before.
- **A forge ended** is one being removed (below), from its removal's
  decision on: the connector's own state, kept in its record
  (`../../protocol/forge/engine.md`, 1), not the routing's, so a start finds it
  ended as the decision left it. An ended forge is in no list: no row
  of `forges`, so that `temper ctl forge list` names it no more, no
  offer, so that `SetUp` is refused "not offered", nothing in the
  picker (3), and nothing in the setup host's scans, which read the
  same state. Its connection reads not connected, so every call, read,
  effect and adoption on it is answered `NotConnected` (above) and no
  call reaches it; its hints are dropped; and the routing's events for
  it change nothing (`../../protocol/forge/README.md`, 7.1), so that a read
  waiting on its connection (16) is never asked again.
- **Removing a forge,** `RemoveForge { forge }`
  (`../../client/forge/deployment.md`): a request of the deployment's scope,
  keyed as any party's (`../../client/engine.md`), its owners'
  alone, refused *role* to any other party; refused *no such forge* for
  a forge not kept or already ended, and *not taken* for the
  playground's forge, which the playground's reset removes in a
  decision of its own, the deployment's removal
  (`../../protocol/setup/playground.md`). The deployment removes a forge by that
  reset alone, the connector's own act, which needs no grant: `engine.acts`
  grants a connector jig's acts in the deployment's name (jig's
  `domain/people/acts.md`, 3), and a forge's removal is none of them. In
  one decision the forge is ended; the rest follows as kept acts:
  - *ended at once,* as above, for any forge: its connection, its first
    repository's intent, a GitHub forge's installations and its writer
    are dropped, its reads and watches retired, and its repositories
    leave their projects (below);
  - *its live work cancelled:* each live change on its repositories is
    cancelled by the deployment, as the deployment's compound cancels
    (jig's `domain/tasks/changes.md`), so its tasks end as a cancel
    ends them; each of its outbox's entries is withdrawn, sent or not,
    and a release on it (`change/README.md`, 4) closes no pull request and deletes no
    branch, its hold erased. A chat that worked on it stays open; its
    next delegation to the forge is refused as on a forge not connected;
  - *its box group stopped and removed,* an owned forge's or a fake's:
    the connector asks `remove N` on the pipe (`../../protocol/setup/box.md`,
    1), and the box stops the group and removes its file, its
    `started` record and the forge's directory, and keeps the cache,
    Forgejo's binaries with it. A connected or managed forge has no
    group to remove, nor has any forge without the box
    (`../../protocol/setup/box.md`, 4): for it the act is answered in the
    removal's decision;
  - *its secrets erased at once:* the service drops the forge from the
    forge component and the setup host, which forget its calls, runs and
    callbacks (skein's `forge/connection.md`, 3.1; skein's
    `forge/setup/README.md`, 4.3), and erases each of its secrets, the admin's
    token, the bot's tokens, the webhooks' secret and the App's key
    among them (`../../protocol/forge/engine.md`, 3);
  - *its records erased:* its writer and its outbox's entries in its
    decision; each of its repositories' rows once no kept task names
    it, live or ended (below), so that a chat that worked on one still
    reads its name until the chat itself is erased; and its record once
    both acts below are answered and its repositories' rows are gone,
    its number never given again;
  - *on the forge itself,* nothing is undone but what temper ran: an
    owned Forgejo goes with its directory; a managed Forgejo's bot and
    hooks stay, and the page says so; a GitHub App stays the person's,
    and the page links to its settings to delete it
    (`../../client/forge/deployment.md`). temper's branches and pull requests
    there stay as they are;
  - *kept until whole:* the record keeps which of the two acts that
    follow its decision, the secrets erased and the group removed, has
    been answered, and each one not yet answered is asked again at each
    start until it is, so that a removal a restart cut ends whole
    (`README.md`, section 9).
- **A repository ended** is one adopted from an ended forge. Nothing
  is stored for it: it ends with its forge. It leaves every list of a
  project's repositories: the project's home, so that the project's
  next adoption may name a home, the one-home check skipping it; the
  context repositories a workspace is given (`items`); and the adopted
  ids the setup host reconciles. Until its forge's removal erases its
  row (above), a lookup by its name or tag answers it, so that its
  tasks, chats and landings read as before.
- **A task's names, kept until it is erased.** A kept task is one the
  store keeps: live, or ended and not yet erased (jig's
  `domain/tasks/README.md`, section 3). At a task's release (jig's
  `domain/connectors.md`, 4.6) its watches, holds and subscriptions go,
  and its `Names` row stays under its key as a kept row: the
  repositories its names lie in, one repository's name each, in name
  order (`../../protocol/store.md`, section 2). A kept row watches nothing
  (`keeping-up.md`, 1) and is never named again: `Rename` changes a live task's names
  alone, and `Names` for a task with a kept row changes nothing. A
  removed forge's rule (above) reads the kept rows beside the live
  tasks' names, holds and subscriptions, so that a removed forge's
  repository's row stays while any kept task names it. jig tells the
  connector when a task is erased, in the decision that erases it whole
  (jig's `domain/connectors.md`, 3.4), and the root gives it
  `Unnamed { task }`: the connector erases the task's kept row, or a row
  still live with its watches, then each ended forge's repositories'
  rows that no task names any more, and the forge's record once it is
  whole.
- **Their bound:** at most `forge.repositories` kept rows, beside the
  `forge.tasks` live ones, so that a kept row never takes a live task's
  room. A release that would keep one past it first erases, of the kept
  rows and the released task's, the oldest each of whose repositories a
  task numbered after it, live or kept, names too. There is always one:
  a row no later one covers is the last to name one of its
  repositories, and no more rows can be last than there are
  repositories. So a removed forge's repository's row stays at least
  while the last task that named it is kept.
- **A project deleted** (jig's `domain/people/acts.md`, 2): when the core
  asks the connector to release the project, the connector erases the
  rows of the repositories the project adopted, and the forge client's
  row of each, by a removed forge's rule (above), at once, since the
  project's tasks that named them are erased by then; it takes those
  repositories out of every kept row, erasing a row left with none, so
  that a kept row names only repositories with rows, and lets go of an
  ended forge whose last repository went, as `Unnamed` does; a row
  another project adopted stays. temper's kinds' settings in the
  project's scope go as `../../protocol/configuration/settings.md`, 2 says. It
  answers the core once that has committed, and again, at once, when
  asked again.
- **An adoption on a forge not connected** is refused as such, before
  any read, and the client offers Connect; the picker offers none of
  that forge's repositories (3). One on a connecting forge waits for
  it, as above.
- **The writer given at runtime.** The forge client refuses every effect
  on a forge with no writer, `Forbidden` (skein's `forge/client.md`,
  7.1). The forge's record names none (`../../protocol/forge/engine.md`, 1):
  the connector gives the client a forge's writer, its
  user's id (the Forgejo user, or the GitHub App's bot user), when the
  routing says the forge is connected (`../../protocol/forge/engine.md`, 2),
  through the client's `writer(d, env, Writer { forge, author }, out)`.
  The same writer given again changes nothing. A forge's writer never
  changes: a replacement that answers as another user is refused before
  it reaches the forge (`../../protocol/forge/engine.md`, 2), since the keys and
  echoes already on the forge are the old user's.
- **The forge client keeps the writer as its own record** (skein's
  `forge/client.md`, 7.1): `Stored::Writer` under `Key::Writer(forge)`,
  in temper's small store under its tag 16, and
  restored at start among the forge's rows, in any order: the client
  takes them all before `Restored`, and runs no restored effect before
  it, so a restored effect never meets `Forbidden` (skein's
  `forge/client.md`, section 8; `README.md`, section 9). The connector keeps no
  copy, and erases it in a removal's decision (above).

## 2. Access at adoption

- **On a managed or owned Forgejo** the connector asks for access
  before the adoption's reads: `Adopt { forge, repository }`, through
  the root and the routing, which runs it on the setup's host with the
  admin's token (`../../protocol/forge/engine.md`, 6). The repository is the
  path the request carries, since temper's user may not see it yet. It
  makes temper's user a
  collaborator with write on the repository, and its hook when the
  deployment has a webhooks' listener and serves the forge one
  (`../../protocol/setup/reconcile.md`, 5). Done, the adoption reads as any
  other; refused, the adoption is refused, with the run's failure named
  (`../../protocol/setup/reconcile.md`, 4).
- **The connector never holds the admin's token,** and the forge
  component never does: the setup's host alone uses it, for reconcile,
  for access at adoption and for the picker's reads (3)
  (`../../protocol/credentials.md`, least power).
- **At each start** the repositories adopted on a managed or owned
  Forgejo are named to the routing for its reconcile, which checks and
  repairs each one's collaborator and hook (`../../protocol/forge/engine.md`,
  1).
- **Elsewhere there is nothing to ask:** on a connected Forgejo a person
  gives the access, and on GitHub the App's installation does (3). A
  repository this leaves temper's user only able to read is still
  adopted, and its answer says so (`../../client/forge/family.md`, 6).

## 3. What the picker offers

The adoption form lists, per connected forge, the repositories temper
can reach, a page at a time, by number (`../../client/forge/family.md`, 4; the
FORGE family's `Reachable { forge, installation?, page }`, answered
`{ rows, more }`). The request names no project, so the project's own
repositories are among the rows, and adopting one it holds is refused
as any adoption of it is. Each row is a repository's id, its owner's id
and login, its name and whether it is private.

Every kind is read the same way: the connector asks the root, which has
the routing run the setup host's plan `Reachable { forge, installation?,
page }` (skein's `forge/setup/forgejo.md`, 1.8; `../../protocol/forge/engine.md`, 6),
pages counted from 1, at most `reachable` rows (`picker.page`), ending
`Done(Reachable { rows, more })`. The plan writes nothing, so it runs
beside the forge's one writing plan, and no call of the forge client
reads the picker:

- **GitHub:** one accepted installation's repositories (`GET
  /installation/repositories`), `installation` its id, read on a token
  the host mints for the page and drops. An id the forge has not
  accepted is refused. The document is skein's (`forge/github.md`, 8.3).
- **A connected Forgejo:** the repositories temper's API token can push
  to, read on that token (skein's `forge/setup/forgejo.md`, 1.6).
- **A managed or owned Forgejo:** every repository the admin can see, by
  id ascending, since adoption gives temper's user its access (2). The
  admin's token stays the setup host's. Forgetting it makes the forge
  connected, and its picker with it.
- **"Not listed":** any repository by owner and name, located as any
  adoption is (`adoption.md`), and, when it cannot be reached, what would
  make it reachable, in words.

What a refusal says:

- **`NotConnected`:** the forge is offered for Connect, not listed.
- **`NotInstalled`** (GitHub: no accepted installation on the owner's
  account): "temper's App is not installed on <owner>", with "Install
  on another account".
- **`Missing`** on a repository whose account has an installation that
  selects repositories: "not granted to temper's App", linked to
  GitHub's installation page; on a connected Forgejo, "make <login> a
  collaborator". Once the forge says so, the repository is listed,
  with no restart.

An installation's repositories changing, or its account renamed
(`Repositories`, `Renamed`; `../../protocol/forge/engine.md`, 4), reaches the
connector with no names: it lists that installation again, shows the
account's login as that listing gives it, and marks each adoption whose
repository left the installation; a later listing that has it unmarks
it. Each `Connected` lists again every accepted installation that holds
an adoption, so a change made while temper was away is found. An
installation deleted marks every adoption on its account, and one
suspended marks them until it is unsuspended. The marks are held in
memory, never stored. A marked repository is shown with what would make
it reachable again (`../../client/forge/README.md`, a repository's health). Its
calls meet `Missing`, `NotInstalled` or `Forbidden`, which the connector
takes there as access lacking, as `NotConnected` is (1): nothing is
narrowed, and its changes wait.

## 4. What adoption says of the branch's rules

- **The rules on the landing branch** are read at adoption and said in
  words, each with what it means for landing, and the one setting that
  would let temper land where a rule keeps it from landing. temper
  never changes a repository's rules.
- **On GitHub,** the rules GitHub applies to the branch, from rulesets
  and classic protection alike, neither's absence taken for the
  other's: required approvals, required checks, code owners, a merge
  queue (`queues.md`, 1), signed commits, linear history, and whether temper's App
  may bypass them, `Never`, `PullsOnly` or `Always` (the App's own,
  never the ruleset's list). skein's `Protection` carries them as
  `queue`, `signatures`, `linear`, `code_owners` and `bypass` (skein's
  `forge/README.md`, 3.6; `skein/docs/design/forge/github.md`, 5.3).
  Forgejo answers each false, and `Never`.
- **What each means:**
  - a merge queue: temper's changes enter it (`queues.md`, 1);
  - required approvals or code owners: a person reviews, or the App
    bypasses;
  - required checks: CI, as the landing rule's;
  - signed commits: temper's workers do not sign their commits, so
    where the forge refuses temper's merge for it, landing is a wait for
    someone else's merge (`README.md`, section 7; `README.md`, section 11);
  - linear history: a merge style that keeps it;
  - temper's prefix reserved to temper's user: welcome.
- **The merge style** is shown at adoption. On Forgejo it is the
  repository's default (skein's `forge/README.md`, section 10). GitHub keeps
  none: it is the first the repository allows of merge, squash and
  rebase (skein's `forge/github.md`, 5.1). On a branch a merge queue
  guards, the queue merges by its own method, set in the branch's rules,
  and the adoption says so.
- **Commit statuses on GitHub:** temper's App reads statuses and writes
  none: `status` is false in the kinds there from adoption on, and
  nothing is held for it. Every adoption on GitHub says that the gates'
  verdicts (`change/gates.md`, 1) stay temper's own, and that GitHub's rules cannot
  require them. For a project that lands by turn, the App writes
  statuses and dispatches workflows (`../../protocol/setup/github.md`, 1), so
  that its receipts and verdicts are mirrored (`../landing.md`, 10), and
  adoption says, for each landing branch, whether its bypass is
  `Always`, which the turn needs (`../landing.md`, 8).

## 5. A repository made or copied

`MakeRepository` (`../../client/forge/records.md`) asks the connector for a
repository on an owned or managed Forgejo, empty or copied from a
public address, adopted into a project when `into` names one
(`../../draft/repository-copies.md`, 5).

- **Admitted** by key, as any party's request: the root's scope is the
  project `into` names, or the deployment with none, and the connector
  then refuses any role but owner, `NotOwner`. In order, it refuses a
  forge it does not keep or that has ended, `NoSuchForge`; a forge that
  is not an owned or managed Forgejo, `NotOffered`; a name or an address
  that breaks `keeping-up.md`, 4 of the draft, or past `name_bytes`, `Invalid`; a
  `home` in a project that has one already, `HomeHeld`, which spares a
  copy that would be refused at its end; and a new row past
  `forge.makings`, `Full`. Its outcome is the request's, kept as an
  adoption's is (`../../protocol/store.md`, section 2): *kept*, or *refused*
  with its reason.
- **One row per name,** `RepositoryMaking`, the family `Making`
  (`../../protocol/store.md`, section 2). A second request for a name whose
  row is not failed is answered *kept*, as the first, and changes
  nothing. One for a failed row replaces it, keeping `copying` when it
  copies from the same address.
- **A failed row is dismissed** by `DismissMaking` (`../../client/forge/records.md`
  ): erased, its place in `forge.makings` free. No row at the name is
  answered *kept*; a row not failed is refused, `NotFailed`, and goes on.
  Admitted as `MakeRepository` is, from the project the request names; a
  row whose `into` is not that project is refused, `NotOwner`, and goes on.
- **Its state** moves one way: `Waiting`, `Making`, `Made { id, branch }`,
  `Adopting { id, branch }`, then erased; or `Failed { step, reason,
  words }`, the step `Admin`, `Org`, `Repository`, `File` or `Adoption`,
  the reason `Taken`, `Empty`, `TimedOut`, `Refused(status)`, `NotAllowed`
  or `Other`, and the forge's words, cut to 256 bytes.
- **The connector holds the rows; the setup runs them.** Only the
  service sees the forge's writing runs, so it pulls: when no writing
  run is live on a connected forge, it asks the connector for the
  forge's oldest `Waiting` row, starts its plan and tells the connector
  (`../../protocol/setup/owned.md`, 9). So a row waits for reconcile, a set up,
  an adoption's access or another copy, and restarts need no path of
  their own.
- **`Copying`** (skein's `forge/setup/forgejo.md`, 1.10) is the row's `copying`:
  the plan's write sets it in a decision, and the service answers
  `Written` once that decision is kept. A stop at `Taken` clears it.
- **Made,** with no `into` the row is erased: the repository waits for
  an adoption as any made one does. With `into` it is `Adopting`.
- **The adoption is the connector's own act,** jig's `Act::Adopt` under
  `engine.acts` (jig's `domain/people/acts.md`, 3, as the
  playground's reset acts; `../../protocol/setup/init.md`, 3), not a party's
  request, so that a sign-in that ended during a long copy, or a restart,
  cannot lose it. It names the repository by the id the plan gave, so
  nothing locates it: owned, landing on the branch the plan gave, with
  CI and no gates, the prefix `temper/`, the project's home when `home`.
  The act is keyed from the row's forge, org, name and wall time, so a
  restart joins it or is answered from its saved answer, adopted once.
  jig asks the connector back as it does for a party's adoption, and
  the reads and refusals of this section hold.
- **Refused,** the row is `Failed` at `Adoption`, `NotAllowed` for jig's
  refusals and `Refused` for the forge's, with what the adoption's
  refusal says; the repository stays made, adopted nowhere. A row that
  is asked again runs its plan, which finds its copy finished, and
  adopts it.
- **Adopted,** the row is erased in the decision that hears jig's answer.
- **A forge removed** erases its rows in its removal's decision (1),
  and its live run ends with the forge's runs (skein's `forge/setup/README.md`, 4.3).
- **A project deleted** while a row names it: the act is refused, and the
  row says so.

## 6. A box's person on its forges

On a box, the forge knows who its one person is on each forge it can vouch
for, with no collaborator and no adopting again (`../../draft/forge-identity.md`,
sections 3, 4 and 9). It keeps that as one fact of the forge's record, its
**link** (`../../protocol/forge/engine.md`, 1): on this forge, that party is that user.
One person on a box means at most one link per forge, so it has no family, no
bound and no restore of its own, and it goes with the forge if that's removed.

- **The person is the deployment's sole owner,** handed by the root at each
  start, once the core's and the forge's records are loaded (`../root.md`, 8.1):
  the party jig's people answer as the deployment's sole owner (jig's
  `domain/people/README.md`, section 4), or none. In that decision every link whose
  person isn't that party is cleared and logged, and while the root has
  handed none, no link is made.
- **Only under a box** (`../../protocol/setup/box.md`, 1): a forge connector built
  without `--box` makes no link, whatever it's told, and keeps none: in the
  owner's decision above, every link it restored is cleared and logged,
  whoever the owner is, so a box's data started by plain `temper serve` keeps
  no link and its lookups fall back to seeds (`../../draft/forge-identity.md`,
  section 9). It is the `--box` the connector was built with that decides,
  not the box's pipe: `BoxClosed` (section 1) stops new links for the rest of
  the run but clears none, and the next start judges them again.
- **From an owned Forgejo's `Reconcile`** (`../../protocol/setup/reconcile.md`, 2): given
  the owned forge's person's login, it answers that account's id or none for
  an account gone. An id links that user to the person; the same link again
  changes nothing; a different id replaces the link, as a person may remake
  their account, and that's logged. An account gone leaves the link as it is,
  and that's logged. A managed or fake Forgejo is given no login, so it never
  links.
- **From a GitHub App's account** (`../../protocol/setup/github.md`, 2): each `GET
  /app` a plan answers says whether the App is the box's person's, a
  manifest's App whose `AppMade` equals its `AppId` and whose account's
  `type` is `User`. If it is, its account's id links that user to the person,
  as above; if not, the link is cleared and logged. So a restart before the
  link was kept loses nothing, a pasted App replacing a manifest's has no link,
  and an App transferred away and back links again. A read that fails changes
  nothing.
- **Never temper's writer:** a link to the forge's retained writer is refused
  and logged, and the forge keeps what it had. Reviews skip the writer before
  any lookup (`change/gates.md`, 2), so one human counts once on a forge, and temper never.
- **Its log lines** name the forge and what happened: the account gone and the
  link kept, refused as temper's writer, replaced, or cleared.
- **A link replaced** leaves its old user a stranger, or the seed that was made
  for it before the link: a second account of a person who made one on
  purpose, which counts only if a rule asks for two people.

## 7. Its CI workflows

Neither forge's API lists the workflows a repository runs, so the forge
reads the directory each forge runs them from, and keeps on the
repository's record what it found there, which CI's card names (`change/branch-path.md`,
*CI's card*).

- **The read** is the workflows' directory at a tip of the landing
  branch, the record's `settings.default_branch`: its immediate entries,
  one `Read::File` (skein's `forge/README.md`, 3.2) answered as a directory's,
  `skein_forge::api::Answer::Directory`, within the client's
  `answer_bytes`, as a charter document's read is
  (`../../protocol/configuration/settings.md`, 6). On GitHub it is
  `.github/workflows`; on Forgejo `.forgejo/workflows`, and where that
  is absent `.github/workflows`, as Forgejo itself falls back. Where
  what asks for it names no tip, a `Read::Branch` of the landing branch
  gives it first.
- **A workflow** is each file entry whose name ends `.yml` or `.yaml`,
  named by its file name whole, as "ci.yml", in the forge's order; a
  directory, a submodule and any other file are none. The first
  `WORKFLOWS` (16) are kept, each at most `WORKFLOW_BYTES` (255) bytes,
  a file name's bound; a longer name, or one not UTF-8, is skipped
  whole. A listing the forge cut (`truncated`) gives what it listed.
- **What the record keeps:** `workflows`, the names, saved with the
  record (`../../protocol/store.md`, section 2) only when they differ from
  what it held. No directory there, on Forgejo neither, or a file where
  the directory would be, is no workflow: the list empties. A read
  refused or failed, or answered in another shape, keeps what the record
  held, and the landing branch's next move reads again whatever its
  files (below), by a mark held in memory alone: a restart forgets it,
  and the record keeps its last list.
- **When it is read,** at the landing branch's tip each time:
  - once an adoption saves the repository's record (`adoption.md`; the act
    of 5 alike), and once a refusal's refresh of what temper may do
    there saves it (jig's `domain/connectors.md`, 12), whose settings
    name the landing branch afresh. It is a read of its own, after the
    adoption's: the adoption's answer waits for none of it, and its
    failure refuses nothing. A context adoption that leaves another
    project's record as it is (`adoption.md`) reads nothing;
  - at the first tip the forge observes of the landing branch, which no
    landing's comparison precedes;
  - at a later move of the landing branch, at the tip it moved to, only
    where that landing's files name a path under `.github/workflows/` or
    `.forgejo/workflows/`, or are not known: the comparison refused, past
    its bound, or not made. The forge reads those files
    for every landing it publishes (`README.md`, 5.1, the landings on a repository's
    branch), so a landing that touches no workflow costs no read, and a
    move by someone else is a landing as temper's own merge is (`README.md`, 5.2).

  The landing branch's tip is watched while a live task or a
  subscription names it (`keeping-up.md`, 1); a move made while none does is seen at
  the next observation, from the tip the forge last held, and its files
  read then, as any landing's.
- **One read per repository at a time:** what asks for one while one is
  in flight reads once more when it ends, at the tip then. A repository
  released while its read is in flight keeps nothing of the answer.
- **Every workflow there is kept,** a dispatch-only one among them: CI's
  card leaves out those the repository's test gates and tiers dispatch
  (`change/branch-path.md`), since those settings change apart from the record.
