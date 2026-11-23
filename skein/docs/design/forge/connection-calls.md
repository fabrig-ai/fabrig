# The forge component: calls, repositories and users

## 1. Calls, repositories and users

### 1.1 Calls

- **Admission is the owner's.** At most `calls` calls are outstanding;
  the component never refuses one, and `Busy` is the owner's word
  (vocabulary.md, section 1.4). A call for a forge no one configured ends
  `Unavailable` at once, one for a forge with no access `NotConnected`
  (`connection.md`, 3.5), and one for an account with no accepted installation
  `NotInstalled` (1.4), each at once.
- **A call runs its plan:** each step sends one request and hears its
  answer. It waits, holding no connection, for its forge's start; one
  whose repository has no path kept resolves it (1.2), and one that
  needs a login not held looks it up (1.3). A locate, which finds a
  repository by owner and name, names repository 0 (vocabulary.md, 1).
- GitHub File keeps the requested head within the answer bound; PullFiles keeps whole file bodies to their bound, reading each before side at the merge base, and Compare keeps names alone (vocabulary.md, 1.2).
- **Cost** is the component's count, never a plan's (dialects.md, 1.2): one
  for each request whose first byte was written, the requests that
  resolve a path or a login included. It goes up with the one terminal
  (forgejo.md, section 1.4). Beside it go `free`, how many of those
  requests the forge answered `304`, which its rate does not count
  (1.6), and `rate`, the window the forge reported on the call's
  last answer, when it reported one:
  `Rate { limit, remaining, reset_at }`, wall seconds, per installation
  on GitHub.
- **The deadline** is `deadline` from admission, at most the owner's
  `lifetime`, so no byte of a write goes out past the time the owner
  counts on. A call still waiting at it ends `Unavailable`. An exchange
  that has written no byte fails `Unseen`, and one that has, `Seen`; the
  plan gives the `Error` (forgejo.md, section 1.4). The connection is
  aborted.

### 1.2 Repositories

The component keeps a row for each repository, by forge and id: its path
(owner and name), object format, default merge style and `html_url`,
and, on GitHub, its owner's account id, which chooses its installation
(1.4). The plan reads a row and fills it from the answers it hears
(dialects.md, 1.2).

- **Filled** by the read of the repository by id, counted in the call
  that needed it; by a locate; and by a read of its settings. On
  GitHub, each delivery read whole (`connection-webhooks.md`, 1.5) renames a kept row within its
  owner, and drops it for a new owner, whose account may differ. Its
  repository names neither the account nor the settings, so it makes
  no row.
- **Read once:** a row filled by the read by id or a locate holds the
  settings they answered, and `Settings` is answered from it with no
  request (forgejo-operations.md, 1), until it is dropped.
- **Dropped** when a call by its path ends on a `404` or is answered a
  redirect, a merge is refused its style or a write `Forbidden`
  (forgejo.md, 1.4); the next call resolves the id again. A `404` that
  is an answer (forgejo.md, 1.4, a leaf's) keeps it.
- **At most `repositories` rows;** the one used longest ago goes first.

### 1.3 Users

A table of logins by user id, filled from every user object an answer
carries (a user, a status's creator when not `null`, the requested
reviewers, a collaborators' listing). An id it lacks is looked up by the
dialect's search, counted in the call (Forgejo's:
`GET /api/v1/users/search?uid=<id>&limit=1`); a user not found fails the
call `Missing`. At most `users` rows; the one used longest ago goes
first. A lost row costs one request, and a restarted component starts
with none.

### 1.4 A GitHub App's installations

An App is installed on accounts, users or organizations, one
installation per account, and each installation has its own tokens and
its own rate (github-app.md, section 1.5). The owner accepts
installations (its setup's, setup/github.md, section 1.4) and hands their
ids down in `installations`, which replaces the accepted set, whenever
it changes.

- **The table:** at most `installations` rows for each GitHub forge, by
  installation id. Each holds the account's id and login, whether it is
  suspended, whether the owner accepted it, and, for an accepted one
  that is not suspended, its API token and git's, with their expiries.
  A full table keeps the accepted rows first. It is read whole from
  `GET /app/installations` at each start (`connection.md`, 3.4), when `installations`
  names an id it has no row for, and when the log says the
  installations changed (`connection-webhooks.md`, 1.5).
- **A call takes its installation by account:** its repository's row
  gives the owner's account id, and the accepted row for that account
  gives the token. A locate takes the row whose login is its owner's. A
  call that names no repository, a user's lookup or the identity read,
  takes the accepted installation used most recently.
- **Refused at once, cost 0:** an account with no accepted row,
  `NotInstalled`; a suspended one, `Forbidden`.
- **Not refused at once:** a repository that its owner's installation
  does not select. Its call is sent, answered `404`, and fails `Missing`
  (forgejo.md, section 1.4). The component keeps no table of the
  repositories an installation selects.
- **A repository with no row:** the component reads it by id with each
  accepted installation's token in turn, most recently used first, until
  one answers. That is at most `installations` reads, each counted in
  the call. The first that answers fills the row, and the call goes on
  with the owner's installation. If every one answers `404`, the call
  fails `Missing`. Rows come from the calls' own reads and the
  deliveries (1.2), so this path runs only for an id first named after
  a start, or one whose row the table dropped.
- **The first successful whole read** compares with the owner's accepted
  set: an accepted id absent is `Deleted`; an accepted suspended row is
  `Suspended`; an unaccepted retained row is `Created`; an accepted
  unsuspended row is silent. Failure preserves this baseline, consumed
  once per construction; reconnects compare tables.
- **Missing accepted ids** in a changed set share one whole reread; a
  read already running for that set suffices. On success each id still
  missing is `Deleted` once, deduplicated against the baseline and the
  old table's deletion. Failure retries with backoff without marking
  absence checked. Equal sets, even reordered, and later timer fires
  cause no reread or repeated deletion; a changed set rearms the rule.
  A set whose ids all have rows needs no read. A changed accepted set
  invalidates an in-flight read's staging; its written exchange finishes
  before page 1 is read for the latest set.
- **Changes go up** as `installation` events, later reads comparing
  the table read with the one it replaces: `Created` for a new id,
  `Deleted` for one gone, `Suspended` and `Unsuspended`, and `Renamed`
  for an account whose login changed. New and deleted rows report only
  membership; a suspension change precedes a rename on the same row.
  Acceptance alone emits no installation change. A rename leaves calls by
  repository untouched, since they choose by account id. A log row that
  says an installation's repositories changed sends `Repositories` up
  for that installation, without names: the owner lists them afresh
  with its setup host's `Reachable` (setup/forgejo.md, 1.8, and setup/github.md, 1.9),
  which the component takes no part in.
- **Retention is bounded across pages.** Accepted rows win, displacing
  strays when needed; among strays the earliest encountered stay. A row
  seen at the forge but lost to local capacity is not `Deleted`.
  `Fact::Unkept { installation }` is emitted at most once per whole
  page-1-through-end read attempt, its id naming the first discarded or
  displaced row. Further overflow in that attempt emits no fact. A
  failed attempt ends it, and its retry may emit one again; a fact
  already emitted remains a capacity diagnostic if a later page fails.
  An unretained row emits no `Created` and goes no further. A pending
  fact holds the next page until output room is available. Only whole
  successful reads commit; a bounded cursor resumes their changes
  within each entry point's remaining reservation, before startup's
  mints and `connected`.

### 1.5 A job log's redirect

GitHub answers a job log's read with a `302` to a signed address on
another host, valid for minutes (github.md, section 10). The plan
answers `Redirect { origin, target }`. Its origin is the scheme, host
and port of `Location`, and its target is the path and query.

- **Followed once,** and only to an origin the owner configured for the
  forge, `log_origins`: tls alike, host alike in ASCII case, port alike.
  Each origin comes with its address, which its owner resolved, since
  io resolves no name (../runtime/io/README.md, section 3). The configured origins are at most `log_origins`.
  An empty list, the default, reads no job log:
  a job's read answers `Refused`.
- **To another host,** the call fails `Refused` at once, and
  `Fact::Origin { origin }` names the origin, as `Redirect`'s, so the
  operator can add it. It is said once for each origin from each
  connect: the component remembers at most `log_origins` origins said,
  the oldest forgotten first when full, and forgets them all at the
  forge's next connect. A fact is a diagnostic: no decision rests on it,
  and a call fails alike whether its origin was said or not. The origins
  only grow (`connection.md`, 3.1); the client does not retry.
- **The request** carries no credential and no `Range`, which that host
  ignores. It runs on a connection of the pool to the origin's address,
  as a call's do, with TLS to the origin's host and the forge's trust,
  within the call's deadline, and is counted in the call. The API's
  connection goes back idle. Its answer streams through the plan as
  the log's body. A redirect answered by a redirect fails `Unavailable`.
  The exchange carries the plan's retention unchanged: a failed-step suffix
  stops at the runner's failed-process line while the connection drains
  cleanup within `body_bytes_in` (github.md, 10). Head and job-tail
  retention keep their bounds and meaning.

### 1.6 Conditional reads

A read of something unchanged costs the forge a `304` and no body.

- **What is kept.** For each request of a call's plan that reads
  (`GET`), the component keeps, per forge, the last answer's validators,
  its `ETag` and its `Last-Modified`, each at most `VALIDATOR_BYTES`
  of printable ASCII (vocabulary.md, 1.3), else that one is not kept, with
  the answer as it arrived: its head's fields the plans read (paging's
  among them) and its body. It is keyed by the request: its path
  and query, its `Range` when it sends one, and the identity it was sent
  as (the forge's token, or a GitHub installation), so one installation's
  answer never answers another's read. A `206` answering a `Range` is
  kept as a `200` is. Only a body received whole is kept: one the step
  cut, or that ended early, keeps nothing. An answer with neither
  validator keeps nothing, so where the forge sends none, as Forgejo's
  branch JSON, nothing changes.
- **What is sent.** A request with a kept entry carries `If-None-Match`
  with its `ETag` and `If-Modified-Since` with its `Last-Modified`,
  whichever it has.
- **A `304`** is answered from the kept answer, decoded by the step as
  that `200` was: the call's answer is the one the forge
  would have sent, and it counts as a read, for the client's keep as any
  answer does. The forge's time (`Date`) and its rate are the `304`'s
  own, read before the kept head is replayed, so a listing's `now`
  never goes back to the kept answer's. The request counts in the call's cost and in its `free`
  (1.1). A `304` to a request sent with no validator, or whose entry
  is gone, is an invalid answer, as today.
- **A `200`** replaces the entry, or drops it when the answer has no
  validator; any other answer leaves it as it is.
- **Bounded, oldest out.** The entries, their keys, validators, heads
  and bodies, take at most `kept_bytes` in all, the owner's `calls`
  times its `answer_bytes` (`connection-machines.md`, 1.2), no figure of its own. An answer past
  `kept_bytes` is not kept. To
  keep a new one, the entries used least recently go first, never one
  a request in flight was sent with. Entries live in memory alone: a
  restart, or a forge's ending, drops them, and the next reads are
  whole.
- **Every read takes them,** in both dialects, the delivery log's first
  page among them (`connection-webhooks.md`, 1.5).
