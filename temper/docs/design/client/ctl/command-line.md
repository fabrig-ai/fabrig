# The ctl command line

This document holds the ctl command line, split from [`README.md`](README.md).

## 1. The command line: `temper ctl`

### 1.1 Its shape

```
temper ctl DIRECTORY GROUP VERB [ARGUMENT...] [--json]
temper ctl DIRECTORY me | inbox | sign-in | sign-out | link | pending [ARGUMENT...] [--json]
temper ctl DIRECTORY --call FILE [--json]
temper ctl help [GROUP [VERB]]
```

- **DIRECTORY stays explicit,** first, in every call: an unattended
  agent never acts on a deployment it did not name.
- **`GROUP VERB`** is a tool's name (`tools.md`, 1); `temper ctl DIRECTORY GROUP
  VERB --help` prints its row, as `temper ctl help GROUP VERB` does.

- **DIRECTORY** is the client's private directory, as for `temper-box open`
  (`../line.md`, 1): its credential, `token`, its small store, `store`,
  and the engine's address, `address`, all `jig-client-shell-native`'s
  or beside them, `0o700` and refused unless the user's own. A
  DIRECTORY is `temper ctl`'s or `temper-box open`'s, never both's (`README.md`, section 12).
- **`address`** is `temper-client-shell-cli`'s: one line,
  `http://HOST:PORT`, `0o600`, written by `sign-in` and read by every
  other call.
- **`party`** is `temper-client-shell-cli`'s too: one line, the decimal
  number of the party DIRECTORY is signed in as, `0o600`, written by
  each `sign-in`, the link's and `--service`'s (`README.md`, section 7), once its
  token is kept, and read by the link's `sign-in` alone, only beside a
  non-empty `token` (*signed in*, below). A DIRECTORY signed in before
  `party` was written has none, and one absent, unreadable or not one
  decimal line is read as no party: never an error.
- **A DIRECTORY the call cannot use** ends it, exit 1, with one line
  naming what failed: `DIRECTORY does not exist`; `DIRECTORY is not
  private` (or its `address` or `token`); `DIRECTORY is not signed in`,
  with no `address`, no `token` or a zero-byte one; `DIRECTORY is busy
  with another call` (1.3); otherwise the step and the error,
  `DIRECTORY could not be read: STEP: ERROR`, STEP `open`, `lock`,
  `address` or `token`, or `DIRECTORY could not be written: STEP:
  ERROR`, STEP `address` or `party`, when `sign-in` writes it. ERROR is the file layer's error by
  its name (`NotFound`, `Permission`, ...), `too large` past the file's
  bound, or `unexpected answer`.
- **A FILE the call reads** that cannot be read ends it the same way,
  naming the file and the error: `FILE could not be read: ERROR` for
  `sign-in`'s link and `--call`'s call, `text file could not be read:
  ERROR` for a text file, `key file could not be read: ERROR` for a key
  file, `record file` for `account import`'s. A DIRECTORY failure is
  never printed as a FILE's, nor a FILE's as DIRECTORY's.
- **`--call FILE`** reads one call as JSON, `{"tool": ..., "arguments":
  {...}}`, from FILE, or from stdin for `-`.
- **A call `--call` reads gives no argument of the type a file** (`tools.md`, 1),
  a path or `-`: one that does is a usage error, exit 1, once the call
  is decoded and before DIRECTORY opens. So `sign-in`, and each tool
  of areas 3 to 6 that reads or writes a FILE, is called from its
  arguments. A call opens a FILE only beneath a root its start opened
  before the loop, the directory of the FILE its arguments name, and
  `--call`'s call is decoded on the loop, from FILE or from stdin
  alike.
- **`sign-in FILE|-`** signs in by the link `temper-box serve` printed,
  or an asked one, in either of its forms (`../sign-in.md`, section 1), read
  from FILE or stdin and never an argument (`README.md`, section 7), on loopback, and
  writes the token and the address the link names, as `temper-box open` does
  (`../sign-in.md`, section 1), and `party`. A DIRECTORY that holds a
  token, a non-empty `token`, is refused before anything is sent
  (*signed in*, below): a link is never sent beside a held token, so
  the link stays unspent, and `sign-out` is the way out. A service link
  (`README.md`, section 7) is no person's link: it signs in as `sign-in --service`
  does, a signed-in DIRECTORY too. Its answers,
  each exit 1 with one line, and none writing `token`, `address` or
  `party`:
  - *empty* (no link after stripping one trailing newline): `temper ctl:
    sign-in: stdin held no link; give the box's sign-in link on stdin:
    temper ctl DIR sign-in -`; stdin's end is kept even when it arrives
    before DIRECTORY is ready, so empty stdin is refused without waiting
    or sending a sign-in;
  - *signed in* (a non-empty `token`, found when DIRECTORY's store
    loads, before any sign-in or other request is sent): `temper ctl:
    DIRECTORY is signed in as party N; this link is not spent: sign out
    first, temper ctl DIRECTORY sign-out, or sign in to another
    directory`, N `party`'s number; with no party read (`party`,
    above), the same line without ` as party N`;
  - *spent*, *expired* and *unknown*, the engine's three refusals of a
    link (jig's `protocol/client/sign-in.md`, *Why a link can't sign
    in*), each its own line, naming both ways to a fresh link:
    `temper ctl: the sign-in link was refused: it was already used;
    temper-box serve prints a fresh one, and a signed-in client asks one
    with temper ctl DIRECTORY link --out FILE`; `temper ctl: the sign-in
    link was refused: it has expired; temper-box serve prints a fresh
    one, and a signed-in client asks one with temper ctl DIRECTORY link
    --out FILE`; `temper ctl: the sign-in link was refused: temper does
    not know it, as another deployment's link or one it no longer
    remembers; temper-box serve prints a fresh one, and a signed-in
    client asks one with temper ctl DIRECTORY link --out FILE`, each
    `DIRECTORY` and `FILE` as written;
  - *refused* (any other refusal of the link): `temper ctl: the
    sign-in link was refused`; `temper-box serve` prints a fresh link;
  - *full* (jig's `domain/people/README.md`, section 3): `temper ctl: all N
    sign-ins are in use; the earliest ends at T; this link stays usable
    until U; after that, temper-box serve prints a fresh one when it is
    tried`, N the deployment's room, T the earliest saved expiry among
    its sign-ins and U the link's own, in universal time to the second,
    `YYYY-MM-DDTHH:MM:SSZ`. The link is not spent: the same `sign-in`
    signs in once a sign-in has ended, before U;
  - *not ready* and *busy* pass: the call sends the same link again,
    with the same token, after 100 ms, doubling to 2 s, within its
    deadline (1.2); past it, after busy, `temper ctl: temper is busy;
    this link stays usable until it expires`, and after not ready,
    `temper ctl: temper is not ready: it is starting, or its core is not
    answering, and temper-box serve's log says why; this link stays usable
    until it expires` (`../../protocol/service-process.md`, 1.2). A sign-in never
    answered keeps the deadline's own line.

  `sign-out`
  ends the sign-in, and jig's native shell, keeping none, leaves `token`
  a zero-byte file, which a call reads as no token. A sign-out whose
  token the engine answers and refuses, its sign-in already ended or
  never this engine's, ends the same way, `token` left a zero-byte
  file, exit 0 and nothing printed: the sign-in is over either way,
  so *signed in*'s way out always works once the engine answers. A
  sign-out the engine never answers keeps `token` and says so, exit 1:
  `nothing listens at ADDRESS; is temper running?`, or `the engine at
  ADDRESS did not answer within 30 s`. Every other tool
  signs in with the token DIRECTORY holds (`SignIn { with: Held }`).
- **`link --out FILE`** asks the engine, with the token DIRECTORY
  holds, for a fresh one-time sign-in link for the party DIRECTORY is
  signed in as (jig's `protocol/client/sign-in.md`, the asked link), and
  writes it to FILE: one line, the printed link's URL on DIRECTORY's
  `address`, at the link's page, `http://HOST:PORT/sign-in?secret=SECRET`,
  made by jig's mapping given temper's page (jig's
  `protocol/client/default-http.md`, 1 and 4; `../sign-in.md`, section 1), so that
  a browser on the machine opens it as it opens the printed link, and
  `sign-in` reads it into another DIRECTORY. FILE is written as `party
  make --out` writes a service's credential (`README.md`, section 7): `0o600`,
  refused if it exists, before anything is sent, and durable before
  success prints; the link is never printed, logged or an argument. A
  link asked again replaces the one before, which signs no one in after.
  Its answers:
  - *written:* exit 0, one line, `sign-in link written to FILE; it signs
    in once, until U`, U the link's expiry, `YYYY-MM-DDTHH:MM:SSZ`, and
    with `--json` the record `{"file": FILE, "expires": U}`;
  - *not signed in* (no `address`, no `token` or a zero-byte one):
    `DIRECTORY is not signed in`, as every call's (above), nothing sent;
  - *FILE exists:* `party make --out`'s refusal of an existing file,
    naming FILE, nothing sent;
  - *refused* (the engine gives asked links only to a person's
    sign-in, where its printed link is enabled): `temper ctl: the engine
    refused a sign-in link: it gives one to a person's sign-in alone,
    where its printed link is enabled`;
  - *signed out:* as any call answered signed out;
  - *not ready* and *busy* are sent again as `sign-in`'s are, within the
    call's deadline (1.2); past it, `temper ctl: temper is busy`, or
    `temper ctl: temper is not ready: it is starting, or its core is not
    answering, and temper-box serve's log says why`;
  - *FILE not written* once the link is asked: `FILE could not be
    written: ERROR`, exit 1; the link is dropped unwritten, and the next
    `link` replaces it.
- **Loopback only,** as `temper-box open`, until the clients' listener
  serves TLS: a call checks that `address` is on loopback, `127.0.0.0/8`
  or `[::1]`, before it sends the token, and otherwise exits 1, "the
  engine's address must be on loopback".
- **No configuration file;** its inputs are its arguments and DIRECTORY.

### 1.2 One call, one process

The host of `jig-client`'s one-shot mode (jig's
`client/domain/client/README.md`, section 16), ordinary Rust on skein's loop,
as the terminal's host is:

1. parse the arguments against the table; a usage error exits 1 before
   anything is opened;
2. open DIRECTORY and check its address (1.1); for a call that sends,
   take its lock (1.3) and load the small store's records;
3. start the tools' root, sign in with the token, wait for `me`;
4. send what the store kept for `me`'s party again, each answered as a
   line on stderr, `kept: ...`, a restored proposal's without its
   recipient (`results.md`, 1), and its record erased once its answer is durable;
   the finish notice uses the party last known even after a stream drop;
   the step ends once each is sent, not answered, so that a kept request
   the engine does not answer (busy past its retries, a bound full)
   holds no other act: the call's own local keys are drawn past those
   the kept requests hold, and the call goes on beside them. One still
   unanswered when the call ends stays kept, named on stderr as the
   record allows, which keeps the request's digest and no name (`results.md`, 1):
   `kept: adopt, first sent T, unanswered: it clears when the engine
   answers it, or past its retention; pending lists it`, a line apart
   from the call's own output on stdout, printed for the party last
   known even when the stream dropped before the call's end;
5. run the call: a read answered, an act's answer durable, a wait's
   condition met or its timeout;
6. print the result on stdout, close the stream, exit with the call's
   status.

- **A call's stream ends with its connection.** Step 6's close ends the
  stream's connection, and so does the kernel when the process ends any
  other way. The clients' listener takes the peer's end as the call's
  end (`../engine.md`), and the engine ends the stream then, not
  at its next heartbeat. So a call holds one of the engine's streams,
  which its profile bounds (jig's `protocol/client/README.md`, section 10),
  only while its process runs, and calls that run one after another
  never wait on their predecessors' streams. A read or a wait sends no
  stream gone itself (1.3). An opening refused full by either bound retries
  by backoff within the link's deadline; if a stream opens, the call goes
  on. If the deadline passes with the opening still refused full, it exits
  1 with one stderr line, at most 240 bytes:
  `temper ctl: the engine's N streams are all in use, by waits and open clients; end one, or wait and call again`.
  It names the bound's size in place of the link-down deadline line.
- **A termination signal,** `SIGINT` or `SIGTERM`, arrives as skein's
  `Shutdown` (skein's `runtime/io/README.md`, section 4), and ends the call at once:
  it sends nothing more, leaves DIRECTORY's kept requests as they are,
  prints nothing more, and exits 128 plus the signal's number, 130 or
  143. What it sent stays in the small store, the next call that sends
  sends it again under its key, and a rerun under its name gets its
  first answer (`requests.md`, section 1).
- **A link down** past the link's deadline (`bounds.md`, section 1), at step 3, 4
  or 5, exits 1; what was sent stays in the small store, as after a
  signal.
- **A call is never left waiting with nothing to wait for.** A request
  answered *busy* or *not ready*, or whose answer was lost, and a `me`
  refused, are tried again, and the spans between tries are the tools'
  own (`bounds.md`, section 1): short beside the link's deadline, so that a call has
  a request in flight or due within its span until that deadline, then
  ends in its answer or in `results.md`, 3's line. A longer span would leave the call
  asleep past its last try, its request kept and nothing in flight, and
  would keep an answer that was ready from it for as long.
- **Recovery writes** wait in an uncapped queue, with one write in
  flight. The CLI's internal event and request queues are uncapped
  too; file staging, kernel submission room and output credit keep
  their limits. No output-count reservation bounds recovery.
- **Another party's kept requests,** in a DIRECTORY signed in again as
  another party, stay parked and unsent, until their party signs in
  there again or the retention passes and they are answered unknown
  (jig's `client/domain/client/small-store.md`, section 1). Step 4 does not wait
  for them; the call says how many (`results.md`, 1) and goes on.
- **`pending`** lists what the small store keeps (`host.md`).

### 1.3 Several calls at once

An agent runs a wait while it acts, or several agents share a party.

- **A call that sends** takes an exclusive lock on DIRECTORY before it
  loads the small store's records (1.2, step 2), so two never write the
  small store over each other: skein's `Lock` on a root (skein's
  `runtime/io/files.md`), on DIRECTORY itself, since `store` is replaced by
  a rename on each write. A second tries again every 100 ms, and past
  the lock's wait (`bounds.md`, section 1) exits 1, "DIRECTORY is busy with another
  call".
- **The lock ends when the call is done with DIRECTORY,** before its
  process exits: once the call has its status and its reads and writes
  of DIRECTORY have completed, a store write in flight at a signal
  included, it closes the descriptor it locked, and the lock ends with
  that `Close` (skein's `runtime/io/files.md`: "The lock lasts as long as
  the root's descriptor: until its `Close`"). Its output drains and its
  process exits after, touching nothing of DIRECTORY, so the next call
  never waits on a slow reader of this one's output. A process that ends
  sooner, however it ends, ends the lock with its descriptor.
- **Which take it:** every act, `sign-in` and `sign-out`; never
  `pending`, `help`, a read or a wait.
- **A read and a wait** take no lock, and hold their client's store in
  memory, empty, never DIRECTORY's file: they write no record and send
  nothing but reads and watches. A client loaded from DIRECTORY's
  records would send the stream they name gone (jig's
  `client/domain/client/small-store.md`, section 1), ending the watches of the
  call that opened it.
- **Several parties** are several directories, each signed in as its
  own.

## 2. Open questions

- **The core's reason at the client's deadline** (`results.md`, 3, `command-line.md`, 1.1): a not-ready
  refusal distinguishes readiness from capacity, but names no start step
  or other gate reason. Carrying that reason through jig's wire and link
  is deferred (`../../protocol/README.md`, section 11). Default: say not
  ready and direct the caller to serve's log, with the same 30 s bound.

- **The cause of a long-chat wait** (`command-line.md`, 1.2): a retry's growing span can
  keep a ready answer from a call, but that alone does not say why a chat
  whose model refused its resume answered late or never. The next QA walk
  must capture the engine's answer and the client's pending state together,
  using `results.md`, 3's diagnostic and the service's long-pass line
  (`../../protocol/service-process.md`, 1.1), before attributing that wait to the engine
  or the client. The 30 s deadline stays.

- **A kept request the engine cannot decode** (`command-line.md`, 1.2): the HTTP client
  treats every status other than 200 as a lost answer, so a request that
  is not a record retries until retention. Distinguishing a terminal 400
  from a lost answer needs the protocol owner's decision. Until then it
  keeps that retry behaviour; unsupported settings efforts not caught by
  `tools-acts.md`, 1's local check take the existing final refusal (jig's
  `client/domain/contract/answers.md`, section 1).

- **The reported settings silence** (`tools-acts.md`, 1, `command-line.md`, 1.2): a final refusal in the
  scripted world does not reproduce the walk's kept, unanswered change.
  Its cause needs the engine's answer and the client's pending state
  captured together, as the long-chat wait does. Default: the local effort
  check, with no change to the deadline or the kept request's protocol.

- **`temper-box open` and `temper ctl` on one DIRECTORY:** not supported and
  not detected (`../README.md`, section 5). Default: the lock of `command-line.md`, 1.3 is
  `temper ctl`'s alone, until `temper-box open` takes it.
