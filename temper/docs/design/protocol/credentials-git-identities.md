# Git identity credentials

This document holds git identity credentials, split from [`credentials.md`](credentials.md).

## 1. Git identities

- **What one is:** the token git authenticates with, the login it goes
  with, and the author (a name and an email) of the commits made with
  it. It is an application credential (jig's `domain/engine/README.md`, 6.1),
  numbered by its forge: its number is the forge's (forge/engine.md, 1).
- **One per forge.** Each kept forge has exactly one identity, and each
  repository's runs use its forge's: the forge's record gives its
  number, its login and its author, its bot's; its value is the
  forge's. The root's `GitIdentity { forge, credential }` comes with the
  forge's record, made or restored, and each repository of an
  assignment's workspace names its forge's.
- **On Forgejo:** the forge's `GitToken` record, with the bot's login,
  the record's, which the forge's `Connected` names. Its validity
  is unbounded (`Duration` at its largest, as jig's reference service
  gives its credential): Forgejo's tokens do not expire.
- **On GitHub:** a list of entries, one per installation that has a
  token, each the installation's account id and login and its token,
  built from the forge component's `Event::GitToken { forge,
  installation, account, token, expires }` and held in memory only, by
  installation. The component mints each token with the App's key, for
  contents alone, for every accepted installation that is present and
  not suspended, and again `renew_before` ahead of its expiry (skein's
  `forge/connection.md`). The login is `x-access-token`; the author
  defaults to the App's bot, `<slug>[bot]` with
  `<id>+<slug>[bot]@users.noreply.github.com`, `<slug>` the `AppSlug`
  record's and `<id>` the `User` record's. The value's validity is the
  earliest expiry among its tokens, minus now, and zero once that has
  passed; a list with no entry is valid without bound, as Forgejo's
  value is.
- **A GitHub forge's entries change** on its events, in the routing:
  a `GitToken` puts its installation's entry, replacing the one before;
  an `Installation` change `Deleted` or `Suspended`, or an installation
  no longer accepted, takes its entry out, when it has one. A forge that
  stops (its access refused, whatever the `why`: `credentials.md`, section 5) takes all
  its entries out at once, in one change, as skein's stopped component
  holds no access (`skein/docs/design/forge/connection.md`, 8.3): temper routes no git
  request with a token it can no longer vouch for. Its accepted
  installations stay as they are; a forge connected again puts its
  entries back as its new tokens come.
- **Generations.** Each start gives each identity that has a value
  generation 1. Each change of value makes the next generation, strictly
  increasing per number: a Forgejo forge's `GitToken` replaced; on
  GitHub, each change of its entries. With N accepted installations, a
  GitHub forge's identity takes about N generations an hour, each one
  `Event::Credential` to the live attempts holding it. A generation is
  never kept.
- **The order:** the routing puts each value in the identities' table,
  then gives the root `Event::Credential { number, generation, valid }`
  (jig's `domain/engine/README.md`, 6.1; `../domain/root-translation.md`, 2.1). At start,
  each Forgejo identity with a value is announced before the root's
  restart, so it is usable when decisions open. A GitHub forge's entries
  gathered before its first `Connected` since the start make no
  generation: that `Connected` announces generation 1, holding every
  token the component handed up before it, which is one per accepted
  installation it could mint for, so that no run starts on a partial
  list. Each change after it announces the next. Until generation 1, a
  run that needs the identity waits (jig's `domain/engine/README.md`, 6.1).
- **The identities' table, on the engine:** each identity's two newest
  values, by number and generation, each encoded once by
  `temper-protocol-worker-wire` and held as one secret; the value server
  answers either. A grant of an older generation is left out as it is
  encoded, and the newer one follows it (jig's `protocol/channel/README.md`,
  section 7). The table is temper's own: jig's accounts' table holds LLM
  credentials' schemes alone. No root, record of the main store, fact or
  log ever holds a value.
- **The value's codec,** engine to worker, is temper's, versioned
  (channel.md, section 6): a kind holding Forgejo's one token or GitHub's
  list, with the login and the author. It is never stored.
- **Granted with the assignment.** temper's assignment names the
  identity's number in each workspace item that names a repository, and
  on GitHub the account id of the entry whose account login is the
  repository's owner, compared in ASCII without case, as the routing
  last made the value (channel.md, sections 4 and 6); jig pushes an attempt only
  the credential numbers its workspace names. `jig-protocol-engine-workers`
  fills the value in from the identities' table, through the routing, as
  it encodes the assignment or a grant record, and grants follow their
  attempts as an account's do (jig's `protocol/channel/README.md`, section 7). A
  run with no repository is granted none.
- **On the worker,** `jig-protocol-worker-link` takes the value out into
  the worker's identities' table, by number, at most the two newest
  generations, so that a push in flight across a change ends on the
  value it began with; the hub hears the name only. temper's worker-side
  protocol layer hands the git component the identity's value with each
  operation that names it, for git's HTTPS transport, and the component
  holds it only while that operation is in flight; it never reads the
  table itself (git.md, sections 2 and 5). On GitHub the worker picks the
  entry whose account id is the item's; with none, the
  push is refused, named "temper's App is not installed on the account
  owning <name>" (channel.md, section 6).
  An operation whose identity has no value in the table fails `Broken`
  without running git. An entry goes as jig's do (jig's
  `protocol/channel/README.md`, section 7): a newer generation pushes it out, or
  its validity ends.
- **In git,** the environment only: an `http.extraHeader` of HTTP basic
  authentication from the login and the token, and the author's
  variables; never an argument, a URL, a repository's configuration or a
  file (git.md, section 5).
- **Agents never get one.** jig's hub never forwards an application's
  credential to an agent: an agent process's environment holds no
  credential (jig's `protocol/service.md`, section 6), and the hub hands
  an agent its LLM grants only.
- **A forge that refuses the identity** fails the workspace, never the
  credential: a preparation it refuses is `Preparation::Refused`
  (`../domain/worker/hosted-runs.md`, 1.3), a push `Landing::Refused`
  (`../domain/worker/checkouts.md`). Limited, refused and unauthorized
  are LLM accounts' notices only (jig's `domain/accounts.md`, 3.5).
- **Withdrawn with its forge.** A forge's removal takes its identity's
  values out of the identities' table at once, and withdraws its grant
  in jig's accounts with a last generation valid for no time, which
  lapses at once (jig's `domain/engine/README.md`, 6.1), so no run is granted
  it again and its room in jig's table is free (forge/engine.md, 3). Its number is never another forge's.
- **Replaced** from the client: `Replace` on a Forgejo forge's git token
  (`credentials.md`, section 5) keeps the new record, then announces the next generation.
  A GitHub forge's tokens renew themselves. Generation 1 at each start is
  sound only because no worker outlives the engine: in standalone the
  worker is in the engine's process, and the cluster restarts whole,
  its workers stopped before its engine and never left under a new one
  (`cluster.md`, 4; `credentials.md`, section 8).

## 2. Open questions

- **Git's environment, readable by agents.** An agent run as the same
  user could read a git process's environment. Default: accepted in
  standalone, where every process is the person's; between hosts,
  contained process trees give agents a process namespace of their own,
  which closes it (jig's `protocol/README.md`, section 10). Without
  them, the token would reach git through a pipe and a credential helper
  instead. An owned runner's jobs run as the same user too; their
  isolation, CI's, is setup/README.md's (`setup/README.md`, section 9).
- **Generations across starts.** Generation 1 at every start names a new
  value once a record is replaced or a token minted, which is sound only
  while no worker outlives the engine; a GitHub forge's identity moves
  about one generation an hour per installation. Default: standalone,
  and the cluster restarted whole (`cluster.md`, 4); a fleet whose workers
  outlive the engine waits for generations kept across starts, with
  hosts apart (`README.md`, section 11).
- **A GitHub entry chosen by the owner's id.** The routing matches a
  repository to an entry by its owner's login, since an adopted
  repository keeps no owner's id, so an account renamed after adoption
  matches no entry, and its pushes are refused as not installed.
  Default: the login; keeping the owner's id at adoption (the store's
  `Adopted`, from the forge's read) would let the routing match the
  entry's `account` directly.
