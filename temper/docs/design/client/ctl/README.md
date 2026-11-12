# Tools: temper for agents

Provisional, 2026-10-17, revised 2026-11-05. How agents drive temper:
they read what it holds, act through jig's operations and wait for what
changes, with `temper ctl`, a command line. It serves any agent working
with temper for a person, which reaches the systems temper works with,
the forge first, by tools of its own choosing (`tools.md`, 2). QA's passes after
landing are one such agent (`../../../development/qa.md`). The tools'
bounds are section 9, and what they take of jig section 10; an MCP endpoint and the tools' extraction into jig are later
(section 11). What is still open is listed in section 12.

## 1. In one page

- **Agents drive the plumbing, not the pages.** The tools are jig's
  reads, watches and operations (jig's `client/domain/contract/README.md`),
  jig's helpers where they fit (`shapes`), and temper's forge family.
  An agent never reads a page, a position on a screen or a layout
  (section 3).
- **Each result says what can be done next.** jig's documents carry the
  acts the party may make (made, proposable, refused); every result
  prints them as the calls that make them, with the revisions they name
  (`results.md`).
- **One table of tools.** Each tool is defined once: its name, its
  arguments, its result, its text. The parser, the help, the JSON and
  the tool names the pages may use are derived from it (`tools.md`, 1).
- **A wait is a read.** A read of a watched subject, given `--since` or
  `--until`, blocks until the subject changes from what the agent saw,
  or meets a condition, or a timeout passes (`tools.md`, 4).
- **Each request once.** A tool that makes something requires a key the
  agent names; asked again under that name, it gets the first answer,
  never a second chat. Acts that decide name the revision the agent saw
  (section 5). The named key is jig's (section 10).
- **A turn too large for a page is read alone,** by jig's read of one
  turn (`tools.md`, 2, `README.md`, section 10).
- **Thin command line.** `temper ctl DIRECTORY GROUP VERB ...` makes one
  call per process, in `jig-client`'s one-shot mode, over jig's native
  transport, and prints its result (section 6).
- **The deployment under test is a box** (`../../protocol/setup/box.md`
  ): the agent's harness starts it, signs in by its printed link, adds
  the deployment's LLM key, and throws it away (`README.md`, section 8).
- **LLM accounts are an area of their own** (`areas.md`, area 5): a sign-in
  prints the address to open and returns, and the agent follows it with
  `account list --until settled`; a key is read from a private file and
  sent in a request that is never kept (section 5).
- **Forges are an area of their own** (`areas.md`, area 6): `forge list` reads
  each forge the store keeps as the `Forges` page shows it; a set up or
  a connect without FORGE makes one, and `forge remove` removes one; a
  connect prints
  its link as text, never opened, or reads its pasted value from a
  private file, and returns, and the agent follows it with `forge list
  --until settled`; `forge repositories` reads the picker's page, whose
  rows ride `repository adopt`.
- **Grown by areas,** the first, the fifth and the sixth designed in
  full, the others named (`areas.md`); **later, named:** the MCP endpoint and
  the extraction into `jig-client-tools` (section 11).

## 2. What it is for, and what not

- **Any agent working for a person.** An agent driving temper for a
  person, or a service of a project acting as a party, uses these tools;
  MCP gives them to agents that have no shell (section 11). How it
  reaches the forge, or any other system temper works with, is its own
  choice: temper serves its own part and names the rest by id and link
  (`tools.md`, 2; the user, 10-26).
- **QA's passes first.** QA after landing plays such an agent
  (`../../../development/qa.md`): coding work of growing size, through growing parts of
  temper's surface: a chat that changes one file; then larger changes,
  held changes released, questions answered; then goals with plans,
  decisions, several parties. What breaks is fixed, and what is slow
  made faster, each fix with a story in the worlds (`testing.md`, 6).
- **Not the person's pages.** temper's pages are the person's, proven by
  the fake person's stories in `tests/client` (`../testing.md`).
  No tool waits for a page: a tool reads jig's documents, so no page is
  a tool's prerequisite.
- **Not the browser.** Once temper has one, its QA is agents driving the
  browser as a person does. These tools are no way around it.
- **Not the deployment.** Starting temper with a forge, a runner and a
  model is `../../protocol/setup/README.md`'s; resetting it is the harness's, never a
  tool's (section 8).

## 3. Why the plumbing, not the pages

- **The pages are for people.** The text view's numbers are positions
  that change between prints, the banner and navigation cost an agent's
  context on every read, and a page shows only what its mockup chose. So
  the text view is no agent's driver; these tools are, for temper.
- **jig's client was made for this.** It is presentation-agnostic: "a
  browser, a terminal, a chat bot, a one-shot command and a service with
  no presentation hold the same client" (jig's `client/domain/client/README.md`,
  section 1). Its documents are typed, with stable numbers, revisions
  and offered acts (jig's `client/domain/contract/documents.md`, 1 to 3).
- **Offered acts replace discovery.** An agent learns what it may do
  from the document, as a person learns it from buttons: each act made,
  proposable (to whom) or refused (why), with the bounds a draft is
  compared with.
- **The engine judges again.** Whatever an agent sends, the engine
  checks as it checks a person's request: its role, its pool, the
  revision it names. The tools add no authority and remove no check.

## 4. The crates

```
temper-client-tools          the table, a call, the runner, digests and conditions, the rendering (text,
│                            JSON), the argv parser and help, the limits; the tools' root
├── jig-client               jig's client and its helpers (shapes, facts, thread, words)
├── jig-client-protocol-link the configured settings wire limits
├── jig-client-protocol-wire bounded whole settings values
├── temper-client-domain     the forge's client side and its words (../README.md, 3.1 and ../forge/family.md)
├── temper-client-protocol-wire  the forge family's records (../forge/records.md)
├── skein-ui                 phrases: jig's words and the tools' own, rendered as text
└── skein-json               the JSON writer and tokenizer
temper-client-shell-cli      the host: argv to a call, the one-shot host over jig-client-shell-native,
│                            the directory's lock and address, printing, exit status
├── temper-client-tools
└── jig-client-shell-native  jig's native transport: HTTP, the small store's file, the token's
temper                       the binary: `temper ctl` runs the host
```

- **Named by the client naming rule** (jig's `README.md`, section 15):
  the tools are to agents what `temper-client-view` is to people.
- **jig's part apart from temper's.** `temper-client-tools`' module
  `jig` holds what is generic: jig's part of the table, the call, the
  runner, digests and conditions, the rendering. It names no temper item:
  a boundary test fails the gate when a file under `src/jig/` names
  `temper-box-shell`, or an item of the crate outside `crate::jig`. Extraction is then
  that module moved into a crate of jig's (section 11).
- **The forge's side is reused, not copied.** The tools' root holds the
  forge's client side of `temper-client-domain`, its adoption's ask,
  owner and words, and none of its pages.
- **The tools' root** is a client root in jig's one shape (jig's
  `client/domain/root.md`, section 3): `jig-client` held; two children,
  each an owner and an asker, the runner and the forge's side; routing
  by asker. It holds no page and no view, as a service's root holds
  none (jig's `client/domain/root.md`, section 11). The runner is one
  owner, `Runner`, 3 (`../forge/records.md`), and so one asker (jig's
  `client/domain/root.md`, section 15). Each handle and key it gives
  the client carries its call's number in the high 32 bits of its
  number, calls numbered from one past the highest among the keys
  kept at `Start`: several calls in flight need nothing new of the
  root, and a new key never meets a kept one.
- **Internal hand-offs are uncapped.** Each step settles the tools'
  root's client and child outputs, bounded by retained calls and
  handles; engine, store and timer answers arrive in later inputs.
  Client outputs leave before the next child output
  (`client_outputs_dispatched`): a retirement's erase precedes its
  printed result and a follow-on request under the same name. A test
  holds this order at the admitted-call quota (skein/docs/foundation/programming-model.md,
  7.2). The client has no watermark (`../README.md`, 3.1).
- **Words:** jig's for jig's enums (`jig-client`'s `words`) and the
  forge's of `../forge/family.md`, 1 and 6, so an agent and a person name
  things alike. The view's overrides for its mockups stay the view's.
- **Step code under the subset:** the runner, the rendering, the table,
  the tools' root. **Ordinary Rust:** the host.

## 5. Requests: once, and named

- **Through jig's request table.** Every act but one that carries a
  value (below) is a keyed request: `jig-client` keeps it before sending
  it, sends it again under its key until its answer is durable, and
  answers it once (jig's `client/domain/client/requests.md`).
- **The agent names the key** of every tool marked `K`: `--key NAME`, a
  word of 1 to 64 bytes of `a` to `z`, `0` to `9` and `-`, such as
  `health-chat`. A call without it is refused as a usage error, before
  anything is sent. So is `--key` with no name, or a name outside that
  rule, and that refusal says the rule, as `results.md`, 3 has it and the same for argv
  and `--call`: `temper ctl: task message: --key KEY takes 1 to 64
  lower-case letters, digits or hyphens; temper ctl help task message`.
  `temper help ctl` says the rule too.
- **A named key is the engine's key:** the first 16 bytes of the SHA-256
  of `temper-tools`, a zero byte and the name. jig scopes it to the party
  (jig's `client/domain/contract/requests.md`, 1). The same name asked again by
  the same party, within the engine's retention, is the same request: a
  rerun after a lost answer, an agent's own timeout or a lost context
  gets the first answer, never a second chat. The same name with another
  ask is refused by the engine as a key conflict. The name, not
  DIRECTORY, makes a request the same: a rerun from any directory signed
  in as the party is the same request, and DIRECTORY's store keeps a
  request only until it is answered (6.2). Past the engine's retention,
  which `me` prints, the name is free again, and asking under it is a
  new request.
- **A named request is built from the call and `me` alone,** never from
  the clock, so a rerun builds the same ask: a deadline a template gives
  as a span is sent as that span, which the engine makes a wall when it
  makes the task (jig's `client/domain/contract/requests.md`, 2). A rerun
  after `me` changed what the request is built from (a template, a
  bound) is another ask, refused as a key conflict.
- **A name the client holds already** is answered `Held`, nothing sent
  (jig's `client/domain/client/requests.md`). A call sends what
  DIRECTORY kept again before it sends its own (6.2). A call whose name
  a kept request of its party holds, the same ask, is that request: it
  waits for its answer as for its own, and prints it, so that a rerun
  still gets the first answer; one under that name with another ask, or
  under a name another party's parked request holds (6.2), meets
  `Held`, and exits 1.
- **A spent-pool refusal keeps its own figures first:**
  `chat start: refused: your pool in project P had LEFT left, NEEDED needed`.
  If today's `me` pool still has less than NEEDED, the `read it:` and
  `raise it:` calls follow. With room now, the next line is
  `now: your pool has L left; budget B; this key keeps its first answer: start new work with a new --key`.
  Today's pool never replaces the refusal's figures.
- **Names are the party's.** Agents sharing a party give their names a
  prefix of their own; the page `ctl` says so (`testing.md`, 5).
- **Acts that decide name the revision the agent saw;** a moved one is
  refused, and the agent reads again. A decision asked again under its
  key prints its first answer. A retained decision, by the party itself
  or another, answers `decided first:` with exit 0 and changes nothing.
  An unknown proposal answers exit 2: "this proposal isn't waiting; it
  may have been decided or replaced." A task's automatically replaced
  settings proposal keeps no decision to report (jig's
  `client/domain/contract/requests.md`, section 2, and `jig/docs/design/client/domain/contract/README.md`, section 3).
- **The other acts** (`task stop`, `task cancel`, `proposal decide`,
  ...) take `--key` optionally; without it the table draws one, which
  the small store keeps until answered (6.2).
- **A request carrying a value,** `account add-key`'s and `account
  replace-key`'s (`areas.md`, area 5), and area 6's `forge connect` with a FILE
  and `forge replace`, is sent once and never kept: `jig-client` writes
  no record of it to DIRECTORY's store and never sends it again (jig's
  `client/domain/client/requests.md`; jig's `protocol/client/README.md`,
  sections 3 and 7), and the engine takes the value out as it decodes
  the request, so no committed request or answer holds it (jig's
  `domain/accounts.md`, section 5). A forge's pasted value is kept by
  jig's clients' component in the secrets' store under temper's tag, at
  a number jig gives it, before temper's core sees the request; a
  request refused, or a duplicate, has jig erase it
  (`../../protocol/credentials.md`, pasted values). The tool names no
  number. The client's own answers *unsent*
  (the engine did not make it) and *lost* (it may have) exit 1 with
  nothing kept, as a call cut or a link down before its answer does, and
  `pending` lists nothing of it. The agent asks again with the same
  `--key` and the same FILE: within the engine's retention, `account
  add-key` gets the first answer, never a second account, and `forge
  connect` never a second connect. `account replace-key` and `forge
  replace` take `--key` optionally, as the other acts do, but the name
  drawn is not kept, so only a name the agent gives can be asked again;
  a rerun without one replaces the key or the tokens again.
- **An account's move** (`areas.md`, area 5): `account import`'s `move_in` is
  a request carrying a value, as `account add-key`'s is, under its
  `--key`. `account export`'s `move_out` is a request whose answer
  carries one, the record: sent once under a name the table draws,
  never kept and never sent again, since the record comes with the
  first answer alone and the name asked again is answered with none,
  *lost* (jig's `client/domain/contract/requests.md`, 1). So `account export`
  takes no `--key`: an export asked again is a new request, refused
  "moving" until the first move is cancelled at its deadline. Its
  `written` is an ordinary request under a drawn name, kept in
  DIRECTORY until answered (6.2).

## 6. The command line: `temper ctl`

### 6.1 Its shape

```
temper ctl DIRECTORY GROUP VERB [ARGUMENT...] [--json]
temper ctl DIRECTORY me | inbox | sign-in | sign-out | pending [ARGUMENT...] [--json]
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
- **A DIRECTORY the call cannot use** ends it, exit 1, with one line
  naming what failed: `DIRECTORY does not exist`; `DIRECTORY is not
  private` (or its `address` or `token`); `DIRECTORY is not signed in`,
  with no `address`, no `token` or a zero-byte one; `DIRECTORY is busy
  with another call` (6.3); otherwise the step and the error,
  `DIRECTORY could not be read: STEP: ERROR`, STEP `open`, `lock`,
  `address` or `token`, or `DIRECTORY could not be written: address:
  ERROR` when `sign-in` writes it. ERROR is the file layer's error by
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
- **`sign-in FILE|-`** signs in by the link `temper-box serve` printed, read
  from FILE or stdin and never an argument (section 7), on loopback, and
  writes the token and the address the link names, as `temper-box open` does
  (`../README.md`, section 4). It offers the token DIRECTORY holds, if any,
  beside the link: a sign-in kept leaves `token` as it was. Its
  answers, each exit 1 with one line, and none writing `token` or
  `address`:
  - *empty* (no link after stripping one trailing newline): `temper ctl:
    sign-in: stdin held no link; give the box's sign-in link on stdin:
    temper ctl DIR sign-in -`; stdin's end is kept even when it arrives
    before DIRECTORY is ready, so empty stdin is refused without waiting
    or sending a sign-in;
  - *refused* (the link spent, expired or unknown): `temper ctl: the
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
    deadline (6.2); past it, after busy, `temper ctl: temper is busy;
    this link stays usable until it expires`, and after not ready,
    `temper ctl: temper is not ready: it is starting, or its core is not
    answering, and temper-box serve's log says why; this link stays usable
    until it expires` (`../../protocol/README.md`, 6.2). A sign-in never
    answered keeps the deadline's own line.

  `sign-out`
  ends the sign-in, and jig's native shell, keeping none, leaves `token`
  a zero-byte file, which a call reads as no token. Every other tool
  signs in with the token DIRECTORY holds (`SignIn { with: Held }`).
- **Loopback only,** as `temper-box open`, until the clients' listener
  serves TLS: a call checks that `address` is on loopback, `127.0.0.0/8`
  or `[::1]`, before it sends the token, and otherwise exits 1, "the
  engine's address must be on loopback".
- **No configuration file;** its inputs are its arguments and DIRECTORY.

### 6.2 One call, one process

The host of `jig-client`'s one-shot mode (jig's
`client/domain/client/README.md`, section 16), ordinary Rust on skein's loop,
as the terminal's host is:

1. parse the arguments against the table; a usage error exits 1 before
   anything is opened;
2. open DIRECTORY and check its address (6.1); for a call that sends,
   take its lock (6.3) and load the small store's records;
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
  stream gone itself (6.3). An opening refused full by either bound retries
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
  first answer (section 5).
- **A link down** past the link's deadline (section 9), at step 3, 4
  or 5, exits 1; what was sent stays in the small store, as after a
  signal.
- **A call is never left waiting with nothing to wait for.** A request
  answered *busy* or *not ready*, or whose answer was lost, and a `me`
  refused, are tried again, and the spans between tries are the tools'
  own (section 9): short beside the link's deadline, so that a call has
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
  (jig's `client/domain/client/README.md`, section 8). Step 4 does not wait
  for them; the call says how many (`results.md`, 1) and goes on.
- **`pending`** lists what the small store keeps (`tools.md`, 5).

### 6.3 Several calls at once

An agent runs a wait while it acts, or several agents share a party.

- **A call that sends** takes an exclusive lock on DIRECTORY before it
  loads the small store's records (6.2, step 2), so two never write the
  small store over each other: skein's `Lock` on a root (skein's
  `runtime/io/files.md`), on DIRECTORY itself, since `store` is replaced by
  a rename on each write. A second tries again every 100 ms, and past
  the lock's wait (`README.md`, section 9) exits 1, "DIRECTORY is busy with another
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
  `client/domain/client/README.md`, section 8), ending the watches of the
  call that opened it.
- **Several parties** are several directories, each signed in as its
  own.

## 7. Who the agent is

- **Any credential.** An agent acting for a person may hold the person's
  credential or one of its own: the tools, their calls and their
  results are the same either way, and jig judges each request as the
  party whose credential it is (jig's `domain/people/README.md`, section 3).
- **The person's own:** the printed link's sign-in. An agent's first
  identity is the deployment's first owner.
- **Its own, a service's** (area 3): an owner makes a service in a
  project, with a role (`party make`), and jig's clients' component answers
  its credential once; the tool writes it to the file `--out` names,
  `0o600`, refused if the file exists, and never prints it. The agent
  signs in with it (`sign-in --service FILE`) and acts within that
  role's authority and pool. QA plays several parties this way.
- **Credentials are files,** private, never arguments, never printed;
  the printed link too, which `sign-in` reads from a file or stdin.
- **The agent's LLM is not the deployment's.** The LLM the agent itself
  runs on, to drive `temper ctl`, is its harness's. The deployment's
  runs are paid by jig's LLM accounts, added from the client
  (`../../protocol/credentials.md`, section 3): on a box, by the deployment's
  key the harness adds (section 8), which pays for every party's runs
  (jig's `domain/accounts.md`, 4.2). An agent acting for a person
  connects that person's own accounts with area 5's tools, as the
  person would on the accounts page.

## 8. The box

The deployment an agent drives is an owned setup run by `temper box`
(`../../protocol/setup/box.md`), with no playground, in a scratch
directory of its harness's:

- **Started:** `temper box`, given the harness's three directories
  (`../../protocol/configuration/directories.md`); its stdout carries `temper-box
  serve`'s sign-in links, and the harness passes the first to `temper
  ctl DIRECTORY sign-in -` on stdin.
- **The deployment's LLM key:** a fresh box has no LLM account, so no
  run starts (`../../protocol/credentials.md`, section 3). Once signed in,
  the harness adds one before the agent's first act: `account add-key
  PROVIDER FILE --deployment --key box-llm`, from a private file of its
  own, then `account list --until usable` (`tools.md`, 4). The key is the
  harness's own, with limits of its own; the agent is
  never handed its file, and the harness removes the file with the box.
  On the process tier and in the real loop, it is a key skein's fake
  LLM peer accepts (`../../testing/real-loop.md` and `../../testing/process-tier.md`).
- **The person on Forgejo:** the harness plays the person of
  `../../protocol/setup/owned.md`, 5: it signs in on Forgejo's web with
  `person_password` and changes it, and makes the repository to adopt
  in an org, which the agent adopts with `repository adopt` as the first
  act of area 1.
- **The forge beside temper:** its API and `git` on `owned.listen`,
  reached as the agent chooses (`tools.md`, 2), as the person, never with the
  preparation's `admin_token`, which the engine's setup host alone
  takes, once (`../../protocol/setup/README.md`, sections 3 and 5). The person's
  two tokens there, one that reads and one that writes, are the files
  the preparation leaves, `person_read.header` and `person_write.header`
  (`../../protocol/setup/owned.md`, 3).
- **The forge through temper:** a fresh box keeps no forge; the
  harness sets up the box's Forgejo with `forge setup` (`areas.md`, area 6),
  which makes it as a record, owned, its group run by the box
  (`../../protocol/forge/engine.md`, 2; `../../protocol/setup/box.md`, 1), and
  connected by the box's own preparation and the setup host's plans
  (`../../protocol/setup/README.md`, owned), so it takes no connect, and the
  agent is never handed a value to paste. With area 6, the agent reads it with
  `forge list` and the repositories to adopt with `forge repositories`,
  every one the site admin can see, and adopts one with `repository
  adopt --ids`.
- **Logs:** each step's, the engine's and the worker's among them, in
  its file under `<data>/box/logs/`, and on the box's stderr prefixed
  by its step with `--echo steps` (`../../protocol/setup/box.md`, 2);
  Forgejo's in its directory. Outside the tools.
- **Thrown away** by stopping the box with a termination signal and
  removing its directory: the harness's, never a tool's.

A person's first run starts with the store empty, no forge and no
account; choosing the playground makes its records, its fake forge,
the scripted model's account and its project, the engine adding the
account and adopting its repository itself (`../../protocol/setup/playground.md`
). A harness does as the person does: `sign-in -` with the box's
first link, then `playground start` (`areas.md`, area 6), which answers once
the playground is ready, and `playground end`, which answers once it
is reset, the store empty again.

## 9. The bounds

settings FILE (or stdin):1MiB, tools.ask_bytes; item/member/value
bounds are checked before the request is sent.

Four bounds decide how much of a document a call prints:

| Bound | Whose | It decides | Past it |
|---|---|---|---|
| a page's bytes | the engine's (jig's `client/domain/contract/documents.md`, 1) | whether a turn comes in a page by its bytes or by its size | the turn's line, its size and the call that reads it (`tools.md`, 2) |
| one read's bytes | the engine's (the read of one turn, jig's `client/domain/contract/README.md`, section 4) | whether `task turn` is answered whole | its size, and that it is past the limit |
| one document's bytes | the client's, `document_bytes` | whether the client takes a document whole | a turn: its size, and that it is past the limit; any other document: the client's `Limit`, exit 1 |
| the tools' window | the tools' | how much of a result prints | the rest by its cursor, or `--part` and `--from` (`results.md`, 4) |

- **A turn past either read bound** answers exit 0, with its size and
  "past the limit; report it": the agent reports it as a finding, and
  never works around it.
- **The tools' window is within the client's limit:** a step test fails
  when one of the tools' ceilings passes the client limit it reads from
  (`testing.md`, 1). A result's text is within one document's bytes and a page
  within a turn's. The client's text is the tools' preset's own, within
  a result's text, so any one of its words prints whole, and an act's
  words are checked against it before they are sent. A page, its
  position line and its end line fit a result's text with 512 bytes to
  spare (`results.md`, 4), and its JSON, six bytes for each source byte, a result's
  bytes.

The numbers are constants of `temper-client-tools`' `limits` module. The
client's are the tools' own preset of `jig-client`'s `Limits`, not the
terminal's, whose window shows two rows:

| What | Bound |
|---|---|
| rows: a list's page, the inbox's needs and updates together, the activity lines `task show` adds, a conversation's page; the client's rows per window | 32, the most jig's client wire takes in a limit of rows (its schemas' ceilings; jig's `protocol/client/wire.md`, section 1) |
| a result's text | 80 KiB: a page, and the 16 KiB a result keeps beside it |
| a page: a turn's part, and the source one `task text` page holds | 64 KiB, 65,536 source bytes |
| a result's bytes, as JSON | one document's bytes six times, and 1 KiB: JSON may spend six bytes on one source byte, so a page's JSON fits |
| the client's text, `jig-client`'s `text` and `turn_text`; an act's words and a model's or effort's name, checked before sending | 32 KiB. jig's wire takes at most 64 KiB of it (`task_text_words`), short of a page with its lines; `task text` joins the engine's chunks, so a page needs no more |
| one document, the client's `document_bytes` | 1 MiB |
| project memberships in `me` | 9, covering the Small and Standard deployment profiles |
| a path's segments; an authority's entries | 9; 12, as the terminal's (`../view.md`, 4) |
| the link's deadline: the engine unreachable, or a call's request or answer unheard | 30 s |
| a retry's span, a request's and a watch's: at least half of it, jittered | from 1 s, doubling to at most 2 s, the most a sign-in's link waits between tries (6.2) |
| the lock's wait | 60 s, tried every 100 ms |
| a key's file, less a final newline (`areas.md`, area 5) | 4 KiB; past it, or empty, exit 1 with nothing sent; a key the engine refuses is its refusal, exit 2 |
| a moved record's file, whole (`areas.md`, area 5) | 128 KiB, jig's bound on a keyed request's value (jig's `protocol/client/README.md`, section 3); past it, or empty, exit 1 with nothing sent; a file past its form's bound at the engine is its refusal, a limit, exit 2 |
| a forge's pasted value: its file, and the bundle the tool makes of it (`areas.md`, area 6) | 4 KiB, jig's bound on a carried value; past it, or empty, exit 1 with nothing sent; a value the engine refuses, one past the deployment's `records.value_bytes` among them, is its refusal, exit 2 |
| a picker's page, `forge repositories` (`areas.md`, area 6) | 100 rows, the most `picker.page` takes, printed whole: temper's own family carries them, not jig's rows; a step test fails when 100 rows at the client's bound on a field's bytes pass a result's text (`testing.md`, 1) |
| a wait's timeout | 300 s by default, at most 3600 (`tools.md`, 4) |

The client's other limits are the terminal's (`../README.md`, section 2).
The mandatory `me` frame takes every membership the deployment may serve
(jig's `client/domain/contract/documents.md`, 1), including a project just made.
Its project list grows as it fills up to nine entries; the whole document
keeps its 1 MiB wire bound. Templates and models keep their per-project
bounds. Raising the membership count reserves no storage for absent
memberships.

## 10. What it takes of jig

The tools work around nothing of jig's and skein's. Besides the
contract's reads, watches and requests, they take:

- **A named key:** `jig-client`'s `Send` under a key the application
  names, refused `Held` when the client holds it already (jig's
  `client/domain/client/requests.md`), on skein's request table's
  given keys (skein's `runtime/lib.md`, section 12); everything else of keyed
  requests is the contract's (jig's `client/domain/contract/requests.md`, 1).
- **A read of one turn,** `turn{n, attempt, turn}` (jig's
  `client/domain/contract/README.md`, section 4), answered from the store's
  `Scope::Turn` (jig's `protocol/store/loads.md`, 3); on a page, a turn past
  the page's bound keeps its size (jig's `client/domain/contract/documents.md`,
  1).
- **A deadline as a span** in a request a template gives one (jig's
  `client/domain/contract/requests.md`, 2), so that a named request is built
  without the clock (`README.md`, section 5).
- **For area 5,** jig's accounts (jig's `domain/accounts.md`, section 5)
  and their move (its section 6, for `account export` and `account
  import`):
  their requests (jig's `client/domain/contract/requests.md`, 2), each under a
  key as the others, a key's bytes held to the tools' own bound
  (`README.md`, section 9) and the engine's refusal past its own exit 2; the
  `accounts` watch (jig's `client/domain/contract/README.md`, 5.1); the models
  a party may use, in `me` (jig's `client/domain/contract/documents.md`, 1),
  and the LLM choice in a task's request (jig's `domain/tasks/README.md`,
  section 3); and a request carrying a value, and one whose answer
  carries one (the moved record, `jig-client`'s `move_out`),
  sent once and never kept nor resent (jig's `client/domain/client/requests.md`
  ; jig's `protocol/client/README.md`, section 7).
- **For area 6,** temper's FORGE requests (`../engine.md`), each
  under a key as the others, through `jig-client` as the forge family's
  bytes; jig's request carrying a value (jig's `protocol/client/README.md`,
  section 3) for a pasted connect and a replacement, the value numbered
  and kept by jig's clients' component (`../../protocol/credentials.md`,
  pasted values), sent once and never kept nor resent; and the forge's
  deployment part, read and watched (`../forge/served.md`).
- **The rest as jig designs it:** its helpers (`shapes`, `facts`,
  `thread`, `words`; jig's `client/domain/client/helpers.md`), the
  documents' acts and `me`'s bounds (jig's `client/domain/contract/documents.md`,
  3), the client's bound on a path's segments, a service's sign-in,
  and the one-shot mode, whose modes "differ in the host alone" (jig's
  `client/domain/client/README.md`, section 16).

## 11. Later

- **Other account-act refusal reasons** (`areas.md`, area 5): a key's add or
  replace and a sign-in's start or finish can answer not offered for
  several core causes. Giving each its reason and next step needs those
  causes distinguished in jig's account refusals and client contract
  (jig's `client/domain/contract/README.md`, section 9). Default: their generic
  words; declaration's name taken and not offered, and retry's not
  offered, have the specific words of 5.5.

- **The core's reason at the client's deadline** (`results.md`, 3, `README.md`, 6.1): a not-ready
  refusal distinguishes readiness from capacity, but names no start step
  or other gate reason. Carrying that reason through jig's wire and link
  is deferred (`../../protocol/README.md`, section 11). Default: say not
  ready and direct the caller to serve's log, with the same 30 s bound.
- **The browser's membership boundary** (section 9): check a third
  membership and the nine-membership bound through the browser after
  landing. The native decoder's table and one-process restart stories
  cover the bound below that tier; the web preset uses the same nine.

- **The settings reference's help pointer** (`tools.md`, 3): `settings-fields`'s
  `reserve_time` description says "See settings help". The field table
  that generates it needs to name `temper help settings`, with a check
  over that generated pointer. Default: leave this non-waiting reference
  to separate wording work; `results.md`, 1's waiting-line pointers are checked now.

- **A replaced proposal's decision** (7): returning `decided first:`
  for an automatically replaced task proposal needs jig to retain that
  decision and name its decider (jig's `client/domain/contract/README.md`,
  section 9). Default: the unknown-proposal explanation and exit 2;
  no archive or new decider.

- **Model catalogue changes** (`areas.md`, area 5): `model list` reads the
  offer; it fetches no provider list and adds no model. Catalogue settings,
  adding by name and refreshing an account's list follow the store and list
  steps (`../../draft/model-catalogue.md`, section 7).

Each is named here and designed when it comes.

- **The cause of a long-chat wait** (6.2): a retry's growing span can
  keep a ready answer from a call, but that alone does not say why a chat
  whose model refused its resume answered late or never. The next QA walk
  must capture the engine's answer and the client's pending state together,
  using `results.md`, 3's diagnostic and the service's long-pass line
  (`../../protocol/README.md`, 6.1), before attributing that wait to the engine
  or the client. The 30 s deadline stays.
- **A kept request the engine cannot decode** (6.2): the HTTP client
  treats every status other than 200 as a lost answer, so a request that
  is not a record retries until retention. Distinguishing a terminal 400
  from a lost answer needs the protocol owner's decision. Until then it
  keeps that retry behaviour; unsupported settings efforts not caught by
  `tools.md`, 3's local check take the existing final refusal (jig's
  `client/domain/contract/README.md`, section 3).
- **An effort refusal for a model absent from `me`** (`tools.md`, 3): the local
  check knows only the models `me` shows in the party's projects. An item
  naming another model goes to the engine, whose refusal names the model
  alone. Naming the effort too needs jig's refusal to carry it, a protocol
  decision. Default: the local check and the engine's model-only refusal.
- **The reported settings silence** (`tools.md`, 3, `README.md`, 6.2): a final refusal in the
  scripted world does not reproduce the walk's kept, unanswered change.
  Its cause needs the engine's answer and the client's pending state
  captured together, as the long-chat wait does. Default: the local effort
  check, with no change to the deadline or the kept request's protocol.
- **A failed-run hold's failure text** (`results.md`, 1): the log directions keep no
  text with the task. Storing the last failure needs the user's bound and
  readers, with its stored and wire changes, as `../../domain/worker/README.md`,
  section 10 asks. Default: direct the reader to the engine's log.
- **A busy watch's precise cause** (`results.md`, 3): the deadline's line uses jig's
  `watches_full`, whose refusal carries no count and does not distinguish
  watch capacity from an opening's load slot. Distinct lines wait for jig
  to distinguish the causes (`jig/docs/design/client/domain/client/README.md`, 15).
  Until then the line names watches in use, as the `watch-room` request asks.
- **An MCP endpoint of the engine,** for agents with no shell:
  `temper-service` serves the same tools at `/mcp` on the clients'
  listener, each session a tools' root holding `jig-client` over an
  in-process transport to jig's clients' component (jig's
  `protocol/service.md`, section 10), signed in by its bearer, a wait a
  long call. It needs jig's client in the engine's process, and an
  agent other than a shell agent to drive temper. Area 1 leaves it
  wiring: a call decoded from JSON (`--call`), the rendering told how to
  print a call, a call per asker.
- **Extraction into jig,** `temper-client-tools`' module `jig` moved to
  `jig-client-tools`, when a second application wants the tools.
- **The forge's read family's tools,** beside `forge list` (area 6), as the
  engine serves each of the forge's other parts (`../forge/served.md`).
- **A service's credential reissued, or a service removed** (jig's
  `domain/people/README.md`, section 3): tools of area 3.
- **An account's uses** set, as a tool of area 5 (section 12).
- **An older moved form patched at `account import`** (`areas.md`, area 5, the
  file's versions): jig's form patches, then skein's record patches,
  before the engine's checks, when a form 2 exists.
- **`--until phase:` in the words `task show` prints** (`tools.md`, 4, `results.md`, 1): the
  header prints jig's phrase-book word, and several are no phase word
  (`Waiting for news`, `Due to run`, `Preparing a host`, `Claimed by a
  host`, `Retrying`, `ChipExecutor active`); a wait takes the phase
  word, in any case, until the header and the condition share one
  spelling.
- **A refusal past a person task's contract that carries its bound**
  (`areas.md`, area 3): jig's `LimitKind::Contract` says the same for a wrong
  form, an unknown choice and text past the bound, so the tool names
  the bound from the contract its read gave, and a restored request,
  which read nothing, prints the refusal bare. Carrying the bound in
  the refusal changes a stored form and jig's protocol.
- **Who answered, on a closing person task** (`areas.md`, area 3): jig's live
  document does not say, so with no holder `task show` prints neither
  line until the task has ended. Naming it changes jig's document and
  its protocol.
- **`temper-box open` at a stopped box** (`../line.md`, 1): ctl says at once
  that nothing listens (`results.md`, 3); the terminal keeps the native shell's reset
  and retry. Whether it says so too waits on a timing of what a person
  meets there.
- **Protection on a forge temper owns** (the `forge setup` row, `areas.md`): temper's
  bot stays a writer, so a rule a person later sets there stays unread, and
  `repository adopt` on that forge still prints the line. Granting the bot
  admin would let whatever holds its tokens change that rule, the second net
  (`../../domain/forge/adoption.md`), so it isn't done. Reading with the site admin's
  token, which temper holds there, would widen nothing, at the price of a
  path from the setup's host to adoption and to its later reads. Default: as it
  stands, until a person on an owned forge sets a rule temper should know.
- **An account with no source for this work** (`results.md`, 1): a wait that names its
  account with the why `Empty` keeps its number on `next run:`, above `no
  account serving this work yet`, and `waiting on: Nothing awaited` beneath
  says the opposite. Whether it names what the account lacks, and whether
  `waiting on:` prints `an account` there too, waits on QA meeting one: its
  walks of a stock box meet the wait that names no account.
- **A bounded act other than `task amend`** (`results.md`, 1): jig bounds no other act on
  a task's document, so no engine prints the general line. The goldens
  `task_bounded_proposed` and `task_bounded_refused` hold it on `task stop`,
  and a person task's `acts:` prints such bounds with no space before them
  and nothing of what lies beyond. Whether the line stays, and its person
  form, is settled when jig bounds another act.
- **A step before a project goes public** (`tools.md`, 3): `project amend --visibility
  public` acts at once, so a person who has skipped the publication help
  meets its words only in the completed answer's stderr, after visibility
  has changed. A flag that names them could stand first. Default: none,
  the request `ui-public-3` asking the answer or the help to say so.
- **A turn's outcome past the client's text** (section 9): the client
  keeps at most its text, 32 KiB, of a call's outcome in a turn (jig's
  transcript bound `outcome`), so an outcome's part never fills a 64 KiB
  page. Keeping more raises the client's text, and jig's wire takes none
  past 64 KiB. Default: 32 KiB.
- **Whole reads saying a cut first** (`results.md`, 4): `task turn` without
  `--part`, `task activity` and `chat show` could print a cut part's
  position before its text, and `task show` its asked words'. Default:
  their end line alone; the paged read each names says so first.

## 12. Open questions

- **`temper-box open` and `temper ctl` on one DIRECTORY:** not supported and
  not detected (`../README.md`, section 5). Default: the lock of 6.3 is
  `temper ctl`'s alone, until `temper-box open` takes it.
- **The one-shot host's place:** temper's, in `temper-client-shell-cli`.
  Default: it moves into `jig-client-shell-native` with the extraction
  (section 11), if jig wants one.
- **Naming a source** in area 5's `--source SOURCE`: by its kind, which
  names one source while an account holds one of each kind. Default:
  by kind, and by a program source's name once an account may hold two
  of a kind.
- **An account's uses:** no tool, each account taking jig's defaults
  (jig's `domain/accounts.md`, 4.2). Default: a tool of area 5 once a
  service paid by a person's account needs one (section 11).
- **An export's `written` answered to a later call** (`areas.md`, area 5): when
  `account export` ends before `written` is answered, the next call
  that sends it reports its answer as any kept request's, and nothing
  removes FILE when it is refused "cancelled", though FILE's record is
  then dead (an import of it makes an account needing sign-in at its
  first refresh). Default: so; the
  alternative keeps FILE's path with the kept request, a stored format.
  Until then the agent tells which by `account list`, which sends the
  kept `written` first: the account's `sign_in` source gone (or the
  account), FILE is its only holder; the source still listed, signed in
  or moving, the move was cancelled and FILE's record is dead.
- **Why a moved record is "not ours"** (`areas.md`, area 5): jig's refusal
  carries no reason, so a form newer than the engine's jig reads, another
  application's record and another registration's read alike. Default:
  `account import`'s own check of the form (`areas.md`) covers ctl's jig; a
  reason on jig's refusal (form, application, registration) is jig's,
  when a form 2 exists.
- **A tokens file's layout** (`areas.md`, area 6): temper's bundle is items by
  number (`../../protocol/credentials.md`, pasted values), which no person
  writes. Default: `NAME VALUE` lines, as `areas.md` has them; JSON, or a file
  per token, if an agent needs one.
- **An agent and GitHub's browser steps:** a manifest's or an install's
  link needs a person's browser signed in on GitHub. Default: the tools
  print the link for a person and open nothing; tests drive its steps as
  plain requests on skein's fake forge (`testing.md`, 2, 3); an agent alone
  connects a Forgejo, or an App of its own with `github-app`.
- **The picker's read as a keyed request:** `forge repositories` draws
  a key as the other unmarked acts do, and the engine keeps its answer
  for the retention (`../README.md`, section 5, the picker's page as a
  read).
  Default: kept as any keyed request; a read of the forge's family once
  the engine serves one.
- **Instructions, the charter's or an amendment's:** a task's document
  carries the instructions in force, and nothing says whether an
  amendment set them (`results.md`, 1). Default: `task show` prints them as they are;
  a word for their source once jig's document says it.
- **A milestone's phase in `task activity`:** the activity line holds only
  the position of the task's history row (jig's `OwnerActivity::
  Milestone`), and the phase is on that row (`OwnerMilestoneLine`).
  Printing it costs a second read per page, a history read of the task
  after the page's first position (jig's `Read::History`), whose page
  is bounded and holds other rows too, so it may not reach every
  milestone the page names, and a cut to print where it does not.
  Default: `phase changed` (`results.md`, 1); the history read with a `history`
  tool, if an agent needs one.
- **Raw values left in the text** (`results.md`, 1's words do not reach them yet):
  - a role, as its number, in acts and in `me`'s held policy (`role N`);
  - a grant's connector and kind, as numbers, not 4.1's words;
  - a grant's resource pattern, as bytes and `*`, not 4.1's words and
    `…`;
  - an amendment's implications, as numbers;
  - a turn's resource references and part references, as bytes and
    JSON;
  - an account's lines in `me`, as the record's fields;
  - charter and procedure numbers in creation bounds and delegation
    authority, and an authority's tool and note numbers;
  - a bounded authority's walls, as nanoseconds;
  - a change contract on a connector other than the forge's, in the
    header, as `change, connector C kind K`;
  - a choice's option in a result, as `option N`, not the option's
    words, which the contract holds.
- **Waiting on one forge:** `forge list`'s wait watches every forge's
  row, so another forge's change answers `--until changed`. Default:
  `--until settled` or `--until connected:FORGE` names what is waited
  for; a watched read of one forge, `forge show FORGE`, if an agent
  needs one.
