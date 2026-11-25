# A forge's connection and removal

This document holds a forge's connection, access state and removal, split from [`adoption.md`](adoption.md).

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
  refusal does, having reached no forge (skein's `forge/client.md`, section 5, and `forge/client-effects.md`, section 1
  ). The connector takes it as the forge's access
  lacking, never as a fact of a repository: nothing is narrowed, no
  change is held, and no effect is found again for it.
- **What waits on it.** The connector steps no change on a forge not
  connected, and a stall does not count that wait (`change/stalls.md`, 1); an effect whose
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
  `forge/connection-calls.md`, 1.4); the account is filled from the first
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
    decision; each of its repositories, released (*A repository
    released*, below) once no kept task names it, live or ended
    (below), so that a chat that worked on one still
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
    (`effects.md`, section 2).
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
  `domain/connectors-effects.md`, 1.6) its watches, holds and subscriptions go,
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
  still live with its watches, then releases each ended forge's
  repository that no task names any more (*A repository released*,
  below), and erases the forge's record once it is whole.
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
  asks the connector to release the project, the connector releases
  (below) each repository the project adopted that no other project
  adopts as context, at once, since the project's tasks that named them
  are erased by then, and lets go of an ended forge whose last
  repository went, as `Unnamed` does. A row another project adopts as
  context stays, with everything that names it: its writer becomes the
  first of those projects, its other contexts kept, and only the deleted
  project's own receipts of it go. temper's kinds' settings in the
  project's scope go as `../../protocol/configuration/settings-kinds.md`, 1 says, and so do
  the project's own records: its follows, findings, facts, looks,
  aways, decisions, dismissals, landed drafts and receipts; its landing
  history is swept after the answer, in pages, under its own mark
  (`../landing-history.md`, 4.5). The connector answers the core once
  its release's last page has committed (*A release in pages*, below),
  and again when asked again: a release asked again, after a restart or
  not, takes what is left of it and goes on, and one that finds nothing
  left writes nothing and answers at once.
- **A repository released** is erased with every record that names it,
  through one helper that every path erasing a repository takes: a
  project's release (above); a removed forge's, once no kept task names
  the repository (*Removing a forge*, `Unnamed`); and an adoption rolled
  back when its collaborators' seeding failed (the root's
  `ForgetAdoption` with nothing to restore). Nothing on the forge itself
  is touched: no branch, pull request, issue, hook or repository there is
  deleted, closed or written. What goes, in the decision that releases
  it:
  - *its work retired,* as a removed forge's is (*Removing a forge*) but
    for the one repository: each read of it in flight is dropped, and
    its answer, when it comes, changes nothing and saves nothing (a
    branch, pull, CI, item, tree, workflow, artifact, collaborator, file
    or draft read; a lost writer's, a goal branch's, a tier's, a
    landing's gather, a follow's, a closer's, a brief's); its waits and
    deadlines go (a connection or outage wait, a change's, a landing's,
    a tier's or a projection's due); and the client's watch set is
    replaced by one without its resources, so that the client stops
    reading them and erases their live rows;
  - *its pending work ended, never carried out:* each outbox entry on it,
    sent or not, withdrawn as a removed forge's are, with its flight, an
    outcome that comes for it later saving nothing; each proposed effect
    on it; each hold of one of its resources; each release whose pending
    resource lies in it; each landing with a member in it; each goal's
    issue projection kept in it; each tier on it, its run in flight
    with it; and each change on it, live or ended, its two parts
    together, with its red passes and the merges it recorded (`Landed`);
  - *its observations:* its branch heads and pull states, its CI
    verdicts and the subscriptions to its topics, its verifications, its
    landing log, its receipts under any project, its landed drafts, the
    client's live rows of its resources and its listing mark, and the
    retained adoption outcomes that name it;
  - *its place in others' rows:* it is taken out of every `Names` row,
    a row left with none erased; a live task's names never name a
    repository its project doesn't adopt, so a release finds it in kept
    rows alone;
  - *then its own row,* `Repository`, last.

  Kept on purpose: a repository another project adopts as context, and
  everything of it; what is deployment-wide: the forge's record, its
  writer, the forge numbers, settings outside the project's scope,
  bindings and devices; a project's landing history, whose events name
  a repository by value and which its project's own sweep erases; a
  kept project's follow, and its setting, whose source or target was a
  removed forge's repository, until the setting's removal (`follows.md`,
  9); a making into a deleted project, which fails its row (5); and the
  deletion's own marks, jig's deletion row and the history's `removing`,
  until their own steps end them. A removed forge's record goes as
  *Removing a forge* says, once its repositories' last pages are
  written.
- **A release in pages.** In memory a released repository and
  everything above go at once, in the decision that releases it. In the
  store its writes go in that decision when they fit a page, 512 writes,
  the client's erasures of its live rows aside, which the watch set's
  replacement writes in that decision; past it, the erasures left go in
  later decisions, at most 512 each, each page the connector's own
  deadline, due at once, as the landing history's sweep pages
  (`../landing-history.md`, 4.5). Every save goes in the first decision:
  a kept `Names` row without the repository, a shared context's row
  with its new writer. The erasures go in a fixed order that keeps each
  page whole: pending work first, then observations, then the
  `Repository` row; a change's two parts in one page; and no row before
  a row that links to it, so that no page leaves a row linking to one
  gone (`../root-restart.md`, 1.1). A restart between pages so finds a
  store whose every row links whole and starts; the core asks for the
  release again once the start opens (jig's `domain/people/acts.md`,
  2), and the release, finding the project's repositories still kept
  with what is left of them, takes up the rest; a removed forge's
  repositories that no kept task names are released again at the
  start, beside its removal's steps asked again (*kept until whole*,
  above). A project's answer
  waits for the last page of each of its repositories. Nothing of a
  release's progress is stored: what is left in the store is its
  progress.
- **An adoption on a forge not connected** is refused as such, before
  any read, and the client offers Connect; the picker offers none of
  that forge's repositories (3). One on a connecting forge waits for
  it, as above.
- **The writer given at runtime.** The forge client refuses every effect
  on a forge with no writer, `Forbidden` (skein's `forge/client-effects.md`,
  1.1). The forge's record names none (`../../protocol/forge/engine.md`, 1):
  the connector gives the client a forge's writer, its
  user's id (the Forgejo user, or the GitHub App's bot user), when the
  routing says the forge is connected (`../../protocol/forge/engine.md`, 2),
  through the client's `writer(d, env, Writer { forge, author }, out)`.
  The same writer given again changes nothing. A forge's writer never
  changes: a replacement that answers as another user is refused before
  it reaches the forge (`../../protocol/forge/engine.md`, 2), since the keys and
  echoes already on the forge are the old user's.
- **The forge client keeps the writer as its own record** (skein's
  `forge/client-effects.md`, 1.1): `Stored::Writer` under `Key::Writer(forge)`,
  in temper's small store under its tag 16, and
  restored at start among the forge's rows, in any order: the client
  takes them all before `Restored`, and runs no restored effect before
  it, so a restored effect never meets `Forbidden` (skein's
  `forge/client.md`, section 8; `effects.md`, section 2). The connector keeps no
  copy, and erases it in a removal's decision (above).

## 2. Open questions

- **A change whose forge is not connected,** or whose repository is not
  reachable (an installation suspended, a repository it does not select):
  it waits, its stall not counting the wait, and nothing is narrowed
  (`adoption-connection.md`, 1, `adoption.md`, 7). The alternative is holding it for a person past a bound.
  Default: it waits, and the client shows why.

- **Connecting a forge as another user** (`adoption-connection.md`, 1): refused, since the keys
  and echoes already on the forge are the old user's. Moving a
  deployment to another App or user is later; a forge goes whole, by
  its removal (`adoption-connection.md`, 1).
