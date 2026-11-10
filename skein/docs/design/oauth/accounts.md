# The accounts driver

`skein-oauth-accounts` runs sign-in, refresh, keeping and sign-out end to
end for the protocol layer that owns it, and lends access tokens: a local
host's accounts with an LLM provider, or a service's sign-in of people
through an issuer.

## 1. In skein

- **A component of a protocol layer,** as `skein-llm-connection` is
  (../llm/connection.md, 1): a step machine with its own state, limits, worst
  case and most outputs per entry point, which its owner routes io's events
  to by the tokens it gave (../../foundation/programming-model.md, 4.2). It depends on lib,
  io's vocabulary, `skein-http`, `skein-tls`, `skein-link` (../net/link.md) and the
  client of `README.md`, sections 1 to 5. It knows no service's domain, account names or
  policy.
- **Its owner** reports its earliest deadline and fires its deadlines where
  it fires its own, and seeds its cryptographic generator (../runtime/lib.md,
  section 8), from which each sign-in's state, verifier and nonce are drawn
  (../../foundation/programming-model.md, 9).

## 2. Accounts

An account is configuration, named by a number. Those given when the
component is made are named by their index, 0 to n − 1, and the owner adds
others while it runs, each under a number of its choosing that no live
account holds (3). A name is stable for the account's life: the component
never renames or reuses one on its own. An account has:

- **its registration** (`README.md`, section 1), and the token endpoint's address,
  resolved before the loop (../runtime/io/README.md, 3), with its transport: TLS, with the
  roots it trusts (../runtime/shell.md, 6.2), or plaintext on loopback. Where the
  registration's terms name endpoints on other origins (the metadata, the
  key set, revocation), the account has each origin's address and transport
  too. A key set's URL found in the metadata on an origin the account was
  not given is refused, and the sign-in fails as unverified: the component
  resolves no name while it runs;
- **its source:** sign-in, whose records come from the issuer through this
  component; or handed in, whose access-only records the owner supplies;
- **its keeper** (4), and whether its record is shared with other
  processes (4).

## 3. Requests and events

| From the owner | What it does |
|---|---|
| sign in | starts an account's sign-in, optionally naming the subject it expects; the owner shows the visit |
| redirected | a redirect handed in: the one the owner's web server received, for a confidential client; or the address a person pasted, for a public client's sign-in whose browser is on another machine (5); answered `taken`, or refused with the sign-in still waiting |
| return | where a public client's listener sends the browser on once it has taken the redirect: an account's return location, for its sign-in under way (5) |
| hand in | an access-only record, for a handed-in account |
| grant | asks for an account's access token, held until released |
| rejected | the provider refused a lent token |
| release | the owner no longer holds the account's grant |
| kept | the answer to a keep, for an account the owner keeps: kept, or not |
| cancel | ends an account's sign-in, refresh or sign-out; its terminal follows |
| add | a new account, under a name and with its configuration (2), while the component runs |
| remove | ends an account; `removed` follows |
| sign out | revokes a signed-in account's record and forgets it; `signed out` follows |
| want | marks an account wanted, or not wanted: a wanted account is refreshed though nobody holds its grant |
| locked | the answer to a lock, for an account whose record is shared: the lock is held, with the record read under it; or busy (4) |
| close | the owner's close: refuse new requests, drain the rest (6) |
| abort | a close that does not wait: cancel every exchange, abort every socket (6) |

| To the owner | What it says |
|---|---|
| visit | the authorization URL to show or open, once per sign-in |
| taken | the sign-in took the redirect the owner handed in: `redirected`'s terminal; the sign-in's own follows (5) |
| signed in | a sign-in that completed: its record kept (4), its grant ready to ask for, its subject, the profile claims it read (`README.md`, 2.2), and the required scopes not granted, if any; the sign-in's success terminal |
| refreshed | a refresh whose record is kept and whose answer gave profile claims (`README.md`, 2.2): the account, its generation and those claims; before the `granted` of that generation, and none where it gave none |
| granted | an account's access token, its generation and how long it is valid |
| keep | a record to keep durably, for an account the owner keeps (4) |
| expiring | an access-only token has reached its lead before expiry, once |
| failed | a sign-in, refresh, sign-out or grant failed: `README.md`, section 4's class, or expired; a sign-in whose record could not be kept fails as not kept, and nothing is lent; an account whose private record did not load fails as unloaded, saying where and why (4); a grant of a record without a required scope fails as not granted; a sign-in or a shared record of another subject fails as another subject; a shared record whose lock was not taken fails as busy (4); a sign-in whose listener read its own redirect past the redirect bound fails as a long redirect, naming the address's length and the bound (5) |
| signed out | a sign-out that settled: revoked, or unconfirmed; the record is forgotten either way |
| removed | a removal that settled: the account and its sockets are gone, and its name is free |
| lock | take the shared record's lock, and read the record under it (4) |
| unlock | release that lock: once for each `locked` |
| closed | the component has settled: its one terminal |

- **One exchange per account at a time** (`README.md`, section 2). A grant asked for
  while the account's refresh runs is answered by that refresh.
- **Kept before lent.** A new record is kept (4) before its token is
  granted. An owner's `kept` that arrives after an abort, or for a keep
  no longer pending, is stale and dropped. One that cannot be kept is not lent, and the previous generation
  stays usable while it is valid (`README.md`, section 1).
- **Refreshed while held.** While a grant is held, the component refreshes
  the account's record its refresh lead before expiry and tells the owner
  the new generation, `granted` again. A grant asked for when the token
  expires within the lead refreshes first. An account whose grant nobody
  holds is not refreshed, unless it is wanted. A `rejected` token refreshes once; a second
  rejection of the same generation fails the grant. No refresh is asked
  before the record's earliest refresh time (`README.md`, section 2); a grant asked for
  meanwhile is answered with the token held while it is valid.
- **Never back to back.** A record the component has just kept from its own
  sign-in or refresh is not refreshed by its lead before half its remaining
  lifetime has passed: its next refresh is at the later of its lead before
  expiry and that half. A refresh lead longer than a token's lifetime thus
  refreshes a held grant at once, then at each half-life, never in a loop.
  A record loaded or read, not kept by this exchange, keeps its lead as it
  is, so that a raised lead refreshes it at once.
- **A wanted account** is refreshed at its refresh lead before expiry
  whether or not a grant is held, so that an idle account keeps its refresh
  token alive and a run finds a fresh token. With no grant held, the new
  record is kept and nothing is lent; a refresh that fails tells the owner
  `failed`, so that it learns the account needs a sign-in without waiting
  for a run. A wanted account that holds no record, or an access-only one,
  is not refreshed. `want` is not an exchange: it answers only when refused.
- **Granted scopes.** The required scopes a sign-in was not granted are
  named in `signed in`; its record is kept and the sign-in has succeeded. A
  grant of a record that lacks a required scope fails as not granted, and
  nothing is lent, until the person signs in again with them.
- **Another subject.** A sign-in that names the subject it expects, and
  returns a record of another, keeps nothing and fails as another subject:
  signing in again never puts another person's record under an account's
  name.
- **Accounts come and go.** `add` admits an account under a name no live
  account holds, within the accounts limit, or is refused (the name taken,
  or full); an account kept in private files starts loading its record
  (4). It answers only when refused. `remove` refuses the account's new
  requests; a sign-in waiting for a person, a grant held or asked, and a
  refresh not yet sent end as cancelled; an exchange already sent, and its
  keep, finish first, so that a rotated refresh token is never lost; then
  its listener and sockets settle, and `removed` follows. The name is then
  free for `add`. Removing an account forgets nothing it kept: its record
  stays with its keeper. The configured accounts are accounts like any
  other, and may be removed.
- **Sign-out** is an exchange, for a signed-in account that holds a
  record. It revokes the record's refresh token where the registration has
  a revocation endpoint (`README.md`, section 2.3), then forgets the record: an account
  kept in private files removes its file (../runtime/io/files.md, 3), and for one the
  owner keeps, `signed out` tells the owner to drop its own. A held grant
  ends as expired. `signed out` says whether revocation was confirmed: an
  unconfirmed one, or a registration without revocation, leaves the issuer
  holding a grant that the owner tells a person to disconnect at the
  provider. A file that cannot be removed fails the sign-out as not kept,
  after revocation; the account holds no record either way, and a record it
  loads again later has a revoked refresh token, which its first refresh
  finds. A handed-in account is refused (its source is the owner's), and so
  is an account with no record.
- **Access-only records are lent, never refreshed.** A grant of one is
  answered while it is valid. At its lead before expiry the owner hears
  `expiring`, once, so that it can ask a person to renew the token at its
  source (a sign-in's, by signing in again); past expiry, or once its token
  is rejected, a grant fails as expired, until a newer record comes: handed
  in by the owner, or from a sign-in. The component never
  writes to the record's source.

## 4. Keeping records

- **The keeper is the owner's choice, per account:**
  - skein's private files (../runtime/io/files.md, 3): one file per account, replaced
    whole and durable before a grant (../runtime/io/files.md, 2), and loaded at the start;
  - or the owner's own store, as a service keeps its secrets among its own
    records: the owner hands in the kept record when it makes the
    component, and each new one goes out as `keep`, lent only once the
    owner answers that it is kept.
- **A handed-in record is not kept:** its source is the owner's, which
  hands in a fresh one.
- **Nothing else is kept:** a sign-in's state, nonce, code and verifier,
  the issuer's keys, and the client secret, live only in the exchange.
- **The record's form is a schema's.** A kept record is the versioned
  record `SavedToken` of skein-oauth's family, `skein-oauth`, in
  `schema/oauth-v3.schema` beside the crate, its codec generated (../data/codec/README.md,
  section 4), at version 3 (../data/codec/README.md, section 5): the version first,
  then the key, the generation, the expiry, the access token, the
  refresh token and the metadata, each `option` where it may be absent,
  and the fields version 3 added. A record of another form is
  unreadable, and its account signs in again, as after any change of a
  record's form (below). Its ceilings are generous and fixed, since from the baseline
  raising one is a new version: each token and the metadata 65,536
  bytes, the issued client id, the scopes and the subject 4,096, the
  times nanoseconds of wall time. The owner's `Limits` stay its runtime
  limits, under those ceilings, refused past one where the owner makes
  them, and `record_bytes` still bounds the whole: every bound is the
  runtime limits', never the schema's ceilings. From the first release skein keeps each
  version's text and its patches (../data/codec/command.md), so an owner
  whose store keeps records patches them with the store's own records,
  by skein's patch set, and hands the component records of the current
  version only; where it holds one inside a record of its own, its field
  is a `secret N holds skein-oauth SavedToken` (../data/codec/README.md, section 3),
  and the owner still decodes and checks it with `decode_record`. A
  record kept in private files is read by the current version alone
  (`README.md`, section 6).

**Loading a private record.** At the start, an account kept in private
files opens its private directory, making it if absent, then loads its
file (../runtime/io/files.md, 3). A grant or a sign-in that arrives meanwhile waits for
the load, which ends in one of three ways:

- **A record:** the account holds it, as if the owner had handed it in
  as kept.
- **No file:** the account holds no record. A grant fails as expired,
  as for an owner's store with nothing kept. A sign-in goes ahead, and
  stores its record expecting the name absent.
- **Unloaded,** with where (the directory or the file) and why:
  - **Unreadable:** the file loaded but holds no record this version
    reads, either damaged or of another version (2), or, under a
    registration whose client id is issued, without one (`README.md`, section 1). A grant fails as
    unloaded. A sign-in goes ahead and replaces the file, stating the
    digest of the bytes it loaded. This is how a person signs in again
    after a record's format changes.
  - **Refused:** io refused the directory or the file unread, naming
    what it found (a link, the wrong kind, group or other bits, two
    links, another owner).
  - **Failed:** io's error.
  - **Stalled:** no answer within the keeper's file deadline.
  - **Too large:** a file larger than a record can be.

  For these four, a grant and a sign-in both fail as unloaded. The
  sign-in fails before its `visit`, so a person is never sent to an
  issuer for a record that cannot be kept. Nothing is replaced unread,
  and nothing a check refused is overwritten.

**Loading again.** An account that holds no record loads again before
it answers its next grant or sign-in. A refresh that fails as sign in
again drops the account's record, whose refresh token is then dead, so
the account holds none and loads again before its next grant. A person who fixes or removes the
file, or signs in from another process, is then heard without a
restart. An account that holds a record never reloads it on its own: it
owns that record, and a store states the record's digest, so a writer
elsewhere shows up as a conflict, a keep that failed. The one exception is
a shared record, which its owner opts into (below): it is read again under
the owner's lock before each write. Each account runs one load
at a time, and requests that arrive during one wait for it.

**A shared record.** An owner whose several processes share one account's
record, each with a component of its own, marks the account's record
shared (2). Each write of it then runs under the owner's lock, and starts
by reading the record again, so that a refresh token another process
rotated is never spent twice:

- **Before each write,** a refresh's exchange, a sign-in's keep and a
  sign-out's revocation, the component emits `lock`. The owner takes its
  lock, as with io's `Lock` on a directory beside the record (../runtime/io/files.md),
  and answers `locked`: for an account it keeps, with the record it read
  under the lock, or none; for one kept in private files, the component
  then loads its file again, as above. An owner that cannot take the lock
  in its own time answers busy, and the write fails as busy, a transient
  failure that leaves the account holding what it held; so does a lock
  with no answer within the lock wait (7).
- **Under the lock,** the component compares what it read with what it
  holds:
  - **a newer generation of the same subject:** it holds that record and
    makes no exchange; a refresh's grant is answered with its token where
    it is valid past the lead, as `granted`, and refreshed under this lock
    where it is not;
  - **no record:** another process signed out. The account holds none, a
    refresh fails its grant as expired, a sign-out ends as signed out with
    nothing left to revoke, and a sign-in goes ahead;
  - **another subject:** another process signed in as someone else. A
    refresh or sign-out fails as another subject, and the account drops
    what it held and loads again before its next grant; a sign-in goes
    ahead, subject to the subject it expects;
  - **otherwise** it makes its exchange and keeps its record, a sign-in's
    at a generation above the one read.
- **Then `unlock`,** once the write is kept or has failed and the
  exchange's sockets have settled; until then the exchange keeps its
  account (5). Each `locked`
  is answered by exactly one `unlock`, whatever happens meanwhile: a
  cancel, a removal, a close or an abort. A `locked` that arrives for no
  pending lock is answered by `unlock` at once. `closed` ends this: the
  component answers nothing after it, so a lock the owner takes and
  answers `locked` after `closed` is the owner's to release, as it
  releases any lock it holds when the component is gone. With io's
  `Lock`, the owner releases it by closing the root it opened (../runtime/io/files.md).
- **Only writes lock.** A grant, a load and lending never wait for the
  lock; a record another process wrote is heard at this account's next
  write.

**Close and abort.** A load is the component's own work for requests it
has admitted, so a close lets it finish within its deadline, and the
waiting requests then end as above. An abort cancels it, and they fail
as cancelled. A `cancel` of a sign-in that waits for a load ends that
sign-in at once. The load goes on for the account.

## 5. The redirect

- **A public client's redirect** reaches a loopback listener the component
  binds on the redirect's address and port (127.0.0.1 for `localhost`) only
  while a sign-in waits for it. It reads one request head with skein-http's
  server, within its limits, checks the request's URI exactly against the
  registration (`README.md`, section 1), answers a small fixed page, "Sign-in received.
  You can close this tab.", and closes the connection. A request for another path, or past the limits, is answered
  with its refusal page, "This is not the registered redirect.", and
  closed, the sign-in still waiting, but for the sign-in's own redirect
  past the redirect bound (*an over-long redirect*, below). The listener
  closes with the sign-in's terminal.
- **The return.** The owner may give a sign-in under way a return location
  (`return`), the address the person's browser goes on to once the listener
  has taken the redirect: the page the person left, in an application that
  sent its own tab to `visit`. With one, the listener answers the redirect
  it takes `303 See Other`, its `Location` the return, with
  `Cache-Control: no-store` and `Referrer-Policy: no-referrer`; with none,
  the small fixed page above. It answers every redirect it takes so,
  whatever the redirect says: the provider's error ends the sign-in, which
  the page gone back to reads from its owner, as it reads a success. For a
  redirect it takes, the listener's answer never says how the sign-in
  ended. Its own over-long redirect instead gets the long-answer page
  below, with no return. A return is the owner's alone: the listener sends the
  browser only where its owner said, never to an address of the request's
  (its Host, its query), and a request it refuses gets no return.
  - **Its bounds:** an absolute `http` or `https` address of at most
    `RETURN_BYTES`, 2048 bytes, of visible ASCII (no space, no control
    byte). One that is not, or one for an account with no sign-in under
    way, is dropped, with no event: a return is a setting, not a request
    with a terminal.
  - **Its life:** it lives as long as its sign-in. A later `return` for
    the same sign-in replaces it; the sign-in's terminal drops it, and a
    new sign-in starts with none. It is never kept: a sign-in its owner starts
    again after a restart has none, and its listener answers the fixed page. A confidential client's redirect reaches the
    owner's own server, which answers it as it likes: `return` changes
    nothing there.
- **A redirect whose port varies** (`README.md`, section 1) binds a fresh listener for
  each sign-in: on port zero, or on its preferred port, and on port zero
  when binding that one fails. The port io's `Listening` reports (../runtime/io/sockets.md,
  2) is the sign-in's: the component gives it to the client in `SignIn`,
  before any `visit`. Two sign-ins at once on such a redirect each have a
  port of their own. A redirect whose port does not vary binds only its
  port, and a sign-in that cannot bind it fails before its `visit`.
- **A redirect is taken only when it is the sign-in's,** wherever it comes
  from, the listener or the owner's `redirected`: its address within the
  driver's redirect bound (`redirect_bytes`, 7) and each field the
  client reads within the client's bounds (`README.md`, section 5), of visible ASCII, with no fragment and no
  field the client reads given twice; then whatever the client's `admits`
  refuses (`README.md`, section 3): its address the one the sign-in bound (the port the
  listener bound among it, for a port that varies), a code or the
  provider's error but not both, an issued client id (`README.md`, 2.1) beside a code,
  its state the sign-in's, and the sign-in still waiting, its deadline not
  reached. The component judges each, and only then does the listener
  answer: a listener holds no state, so it hands up the redirect it read
  and answers as the component says.

  Any other is refused, and the sign-in still waits: the listener answers
  its refusal page, the same for every reason; `redirected` is answered
  `refused`, as another address (`redirect`), another sign-in's state
  (`state`), or not waiting. A redirect at or past the deadline ends the
  sign-in first, as its tick would, and is refused as not waiting. The
  state is the sign-in's alone, drawn fresh for each, at least 24 random
  bytes, and never logged: a client state bound under 32, which would
  draw fewer, makes the component unusable, as its limits (2). A refusal so tells its sender only
  that its state is not this sign-in's. The owner's pastes are bounded as
  well: after `STATE_REFUSALS`, 4, `redirected`s refused for their state,
  a sign-in refuses every later one as full (`Full { bound: 4 }`), and
  still waits for its listener and its deadline. The listener's own
  refusals are not counted: they are bounded by its one connection at a
  time and its request head's limits (7).

  `redirected` refuses in this order: closed (a closing component still
  serves a sign-in that waits, 6); another account; a handed-in account
  (`source`); not waiting (no sign-in, its `visit` not yet out, or its
  redirect already taken); full; another address, by the bounds above,
  against the address as bound; the deadline; then what `admits` refuses.
- **An over-long redirect** ends its sign-in, from the listener alone. A
  request head whose Host is the bound authority and whose target is the
  bound path, then its query, with no fragment, is the sign-in's answer
  past the bound when its address is longer than `redirect_bytes` and its
  one `state` field, decoded within the client's state bound, is the
  sign-in's while it waits (the client's `awaits_state`, compared in
  constant time; `README.md`, section 3). The component judges it as it judges a
  redirect, and in that step the listener answers `414 URI Too Long`, with
  no return, and the page "This sign-in's answer is longer than
  *application* accepts. Go back to *application* and start the sign-in
  again.", *application* the name its owner gave the component (7);
  the sign-in fails as a long redirect, naming the address's length and
  the bound (`failed`, 3), at once rather than at its deadline; and the
  listener closes once its page is sent. One with another state, with no
  state or two, or for a sign-in no longer waiting, is refused as any
  other, its refusal page, the sign-in still waiting: an over-long request
  ends its own sign-in alone. A pasted redirect past the bound is refused
  as another address (`redirect`), as before: a paste has its own
  terminal. A head past the listener's own head bound is skein-http's
  rejection (414 or 431, http.md), the sign-in still waiting; the head is
  sized so that a browser's request for a redirect at the bound fits
  (7).
- **One redirect per sign-in.** Taking one leaves waiting in the step that
  takes it: the listener stops listening, a redirect it has read and not
  yet answered is refused, and so is any after it; a later `redirected` is
  refused as not waiting. A sign-in whose browser reaches the listener and
  whose person pastes its address too makes one exchange, with whichever
  the component judged first. Its deadline is its own from its start: no
  redirect, taken or refused, starts a timer.
- **A pasted redirect,** for a public client's sign-in whose browser is on
  another machine than the listener: the provider sends that browser to the
  redirect on its own loopback, where nothing answers. The person copies
  that address, and the owner hands it in (`redirected`) while the sign-in
  waits, once its `visit` is out. `taken` answers it in the step that
  takes it, and the sign-in's terminal follows as for any redirect; a
  confidential owner's `redirected` is answered `taken` the same way. The
  component checks it as the listener's (above), never fetches it, and
  never reads `localhost` for `127.0.0.1` or the reverse. The listener
  stays up meanwhile, so a browser on the engine's machine, or one through
  a tunnel to its port, still returns as before. Nothing of a paste is
  kept: a sign-in a restart ended is started again.
- **A fixed port, one listener at a time.** A sign-in on a redirect whose
  port does not vary binds once any listener of the component's on that
  port has settled, so that an owner that cancels a sign-in and starts
  another at once, as for a person starting again, gets the port, rather
  than a sign-in failed before its `visit`. A sign-in that has ended leaves
  its account at its terminal, while its listener still settles: the
  account admits its next sign-in at once, and that sign-in's listener
  binds once the last listener on the port has settled; a sign-in that
  still holds the owner's lock (4) keeps its account until its sockets
  have settled and it unlocks, since `unlock` names only the account. A
  listener that still settles counts toward the listeners limit (7).
- **A confidential client's redirect** reaches the owner's web server. The
  owner hands it in (`redirected`), and the component checks it the same way.
- **The component never opens a browser:** the owner shows `visit`.

## 6. Owner close

The component follows ../../foundation/programming-model.md, 5.2.

- **Close drains.** From the owner's close on, it refuses every request
  at its entrance, as closed, never as the request's fault. What is idle
  closes at once: an account with no exchange holds nothing, and no
  refresh starts after the close, as a refresh lead is policy for a live
  service. An exchange in flight with the issuer finishes, its record is
  kept, and then its connection closes. A sign-in that waits for a person is the owner's
  request too, and still waits, within its sign-in deadline: the
  component never cancels its owner's work on a close. An owner that
  will not wait for a person cancels the sign-in first (`cancel`), which
  ends it at once, failed as cancelled, and closes its listener.
- **`closed`** goes up once, when no exchange is left and every socket
  the component made has settled at io.
- **Abort** is a close that does not wait: every exchange ends failed as
  cancelled, and every socket, the listeners' among them, is aborted at
  io. A rotated refresh token whose answer an abort cuts off may be lost,
  as after a crash (`README.md`, section 1). Every shared record's lock still held, or
  answered `locked` before `closed`, is released with `unlock` before
  `closed`; one answered after it is the owner's to release (4). An
  abort after a close turns what still closes into aborts, as a
  termination signal does for the process; a close after an abort
  changes nothing.
- **No keep time.** Each exchange opens its own connection and closes it
  when the exchange ends: exchanges are minutes apart, and a pool would
  only add a keep time to close.

## 7. Limits

| Limit | What it bounds |
|---|---|
| accounts | the accounts at once, configured and added |
| exchanges | sign-ins, refreshes and sign-outs at once, across accounts |
| listeners | loopback listeners at once |
| refresh lead | how long before expiry a held or wanted grant is refreshed, and an access-only one announced |
| lock wait | how long a write waits for the owner's `locked` before it fails as busy: a `Duration` from the `lock` it emits |
| redirect bytes | the longest returning redirect the listener or a paste reads, the whole address with its query (`redirect_bytes`, 5), apart from the address the client sends out (`url_bytes`, `README.md`, section 5) |

The owner gives these limits when it makes the component. The lock wait
is a deadline on the injected monotonic clock (`README.md`, section 3), as each other
deadline is, fired by the owner; once it passes, the write fails as busy,
and a `locked` that comes later is answered by `unlock` at once, until
`closed`, after which it is the owner's to release (4).

- **The redirect bound holds a full answer.** `redirect_room` gives the
  shortest `redirect_bytes` that holds the registered URI at the client's
  `uri_bytes`, then `?code=` and a code at `code_bytes`, `&scope=` and a
  scope at `scope_bytes`, `&state=` and a state at `state_bytes`, and
  `&client_id=` and an issued client id at `issued_client_bytes`, each
  unescaped: their bounds plus 31 bytes, or none past `u32`.
- **The listener's head holds a browser's request** for a redirect at the
  bound. `listener_head` gives the least `server.head` for it: the request
  line, `GET `, a target of `redirect_bytes` and ` HTTP/1.1` with its line
  end, then `BROWSER_FIELDS`, 8,192 bytes, for a browser's own fields with
  their line ends, the cookies it holds for the loopback host among them,
  and the blank line: 17 bytes past the two. A cookie is not scoped to a
  port, so every loopback application's cookies for `127.0.0.1` reach the
  listener: a browser's navigation fields take about one KiB, and the rest
  holds its cookies.
- `valid` checks neither, since an owner sizes its own redirects: one whose
  profile holds them checks both (jig's
  `jig/docs/design/protocol/limits.md`).
- **The application's name** is the owner's, given when it makes the
  component, for the listener's page (5): 1 to `APPLICATION_BYTES`, 64,
  bytes of visible ASCII and spaces. Another makes the component unusable
  as its limits do (`Unusable::Limits`).

The client's own limits are `README.md`, section 5's. `skein_oauth_accounts::valid`
checks account, exchange and listener capacities, client limits and the
HTTP, TLS and io edges. `Component::check` checks a list of accounts
against the limits as making the component does, refusing with the same
`Unusable`, and makes nothing: it takes no generator, so an owner checks
an account it will add later without making one (../runtime/lib.md, section 8). The redirects it refuses take no additional
connection slot. Trusted roots are shared configuration. Neither the
accounts component nor its listener requires a sum of heap prices.

## 8. Testing

- **One owner harness,** with one `Client` and `Process`, holds the
  account and sign-in owners. One protocol driver serves both, retaining
  their wire seeds and trace order; one restart wrapper retains each
  owner's signal-close predicate. The scenarios' visible events and
  outcomes are the same through these shared paths.
- **A protocol world** (../../foundation/testing-strategy.md, 2.5): the component, its
  owner's routing, the fake issuer (`README.md`, section 5), and a scripted browser that
  follows each visit to the listener, over streams cut at random. Sign-in of
  both clients, refresh while held, rotation, a rejected token, a keep that
  fails, an access-only record expiring and handed in again, a `localhost`
  redirect, wrong redirects, a public sign-in's pasted redirect (taken; a
  paste and the listener's redirect in both orders, one exchange; a second
  paste; another host, port or path, a fragment, a field twice, past its
  bounds, a space or a control byte, a code and an error both, without a
  code or an issued client id, each refused with the sign-in still
  waiting; another sign-in's state refused, and after `STATE_REFUSALS`
  every paste refused as full, the listener still taking its own; a paste
  before the `visit`, at the deadline (the sign-in timed out), after the
  cancel or the terminal, refused as not waiting; a paste judged between
  the listener's read and its answer), the listener's redirect with
  another state refused with the sign-in still waiting; a cancelled
  sign-in on a fixed port and a new one started in the same step, which
  binds once the first's listener settles; a state bound under 32,
  unusable; a sign-in's return (a `303` to it; the fixed
  page without one; one given for no sign-in, or after its terminal,
  dropped; one past its bounds dropped), and the owner's close in every state, a
  sign-in waiting for a person among them, cancelled by the owner or
  still waiting; an abort after a close, and a close after an abort; one
  terminal per request, and `closed` once and last.
- **A simulated world** (../../foundation/testing-strategy.md, 2.7), with io over the
  simulator, the fake issuer as a simulated server, and private files on the
  minimal fake machine (../testing/README.md, 4): a filesystem that fails or stalls a
  keep, and a private directory or file with the wrong mode. Every world
  ends under the teardown invariant (../../foundation/testing-strategy.md, 6).
- **The additions,** in both worlds: a new registration and signing in
  again, with the issuer's keys fetched before each visit; a redirect whose
  port varies, its preferred port taken; required scopes declined, then a
  grant refused as not granted; another subject; accounts added and
  removed in every state, a name reused after `removed`; a wanted account
  refreshed with no grant held, and its refresh failing; sign-out
  confirmed, unconfirmed and without revocation, for both keepers; a shared
  record under a scripted owner lock that is busy, slow, or answered after
  a cancel, a removal or an abort, finding a newer generation, no record,
  or another subject; exactly one `unlock` for each `locked` before
  `closed`, and none after it, a `locked` answered past `closed` left to
  the owner.
- **Focused rows** beside the redirect's (5): a redirect with an
  issuer's `code`, `scope`, `state` and `client_id`, its code synthetic
  and 1,900 or 3,900 bytes long, taken from the listener and pasted under
  bounds that hold it, where the client's `url_bytes` alone would refuse
  it; one at the redirect bound, from a browser's real header set with a
  cookie, within `listener_head`; and an over-long one from the listener,
  with the sign-in's state (the long page, and the sign-in failed at once,
  naming its length and the bound), another state, and another path (the
  refusal page, the sign-in still waiting).
- **Memory:** every limit at once, under named scenario ceilings with
  headroom.
- **Transports:** the replaying tiers run plaintext issuers on loopback; TLS
  to an issuer runs in the real loop (../net/tls.md).
- **For an application's real loop** (../../foundation/testing-strategy.md, 2.8), the fake
  issuer runs in its process, on loopback, with no added latency, for the
  stories that sign a person in: the test drives the redirect, following
  each `visit` to the listener as the scripted browser does. Stories that
  need an account but no sign-in give it a key instead (../llm/README.md, 2.6), and
  stories of a program that signs in by itself use a fake program; neither
  needs the issuer.
