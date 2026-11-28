# temper's wires

Provisional, 2026-11-12, revised 2026-11-24 for layouts. The wires
between temper's engine, its workers, their agents and its clients:
which they are, their frozen versions and their layouts, what the
engine and a worker state of them at their opening, how the engine
converts between its layouts and a worker's, what a mismatch costs, how
the host tools' contract is answered across versions, and how
`versions` prints them. It carries `../draft/layouts-both-ways.md`,
sections 3, 5 and 6, and `../draft/archive/redeploys.md`, sections 3.4, 4.4 and
8, for temper's parts. The mechanisms are skein's
(`skein/docs/design/data/codec/layouts.md`, plans;
`skein/docs/design/data/codec/fingerprints.md`, section 1, fingerprints)
and jig's (`protocol/channel/wires.md`, the opening's statement and
check; `protocol/channel/layouts.md`, conversion in the engine's
component); this document says what temper's deployment puts through
them.

## 1. In one page

- **Versions are frozen** (3): each worker wire's version stays at its
  header's, and a layout is named by its fingerprint. A layout change
  moves no version and brings no patch.
- **The opening states layouts and texts** (4): a worker states each
  wire's layout and sends its text; the engine refuses one whose text
  gives no plan, naming the wire, and otherwise converts.
- **The engine converts both ways** (6): what it receives from a worker
  by Read plans to its own layouts, what it sends by Keep plans to the
  worker's; the worker speaks its own layouts alone, so a redeploy of
  the engine alone keeps workers of the build before.
- **A value the worker cannot keep costs no task** (5): an assignment
  waits for a worker that can hold it, spending no try.
- **The host tools' contract is versioned whole** (7): a run's charter
  names its version, each call carries it, and the engine answers every
  version from its floor.
- **Clients' versions are frozen** (9), their layouts compared at the
  hello, with no kept text and no conversion; a page refused reloads
  once to the engine's bundle if it can record that reload.
- **`versions` prints each wire's layout** (10), and the store keeps the
  wires the engine's last start spoke and each worker's last opening
  since, with its texts (10.1), by which `upgrade --check` says which
  workers may stay (10.2).

## 2. The wires

| Wire | Crate | Between | Stated at the opening |
|---|---|---|---|
| `jig-worker-wire` | `jig/crates/jig-protocol-worker-wire` | the engine and its workers: the channel's records, its kinds' table included (jig's `protocol/channel/wires.md`, 4) | by jig |
| `smith-pieces` | `smith/crates/smith-channel`, its second family | the pieces of smith's channel the engine keeps or sees: a call's name and a delivery's evidence (`channel.md`, section 2; smith's `protocol/README.md`, section 8) | by jig |
| `smith-charter` | `smith/crates/smith-charter` | the engine writes charters, which the worker and its agent read; the agent writes the result, which the engine reads | by jig |
| `smith-transcript` | `smith/crates/smith-transcript` | the agent writes turns; the engine keeps them and resumes a window on a worker | by jig |
| `temper-worker-wire` | `temper/crates/temper-protocol-worker-wire` | temper's payloads in jig's channel: the workspace items, the work, the preparation, a job's spec and a git identity's value (`channel.md`) | by temper, through jig |
| `temper-host-tools` | `temper/crates/temper-engine-smith` | the host tools the engine offers in a charter and answers when a run calls them (7) | no: the engine's alone |
| `smith-channel` | `smith/crates/smith-channel` | a worker and its agents, per-build (`../draft/archive/redeploys.md`, 4.4) | no: smith's own opening, between a worker and its agent |

- **Each wire's name is its family's,** as its schema's header gives it;
  the host tools' contract, which is no schema, is named
  `temper-host-tools`.
- **`smith-channel` is per-build:** a worker pins its agent's program
  and configuration at its start, so the channel between them never
  spans two builds and nothing converts it; its version is frozen at its
  header's, as every wire's is (3.1). What the engine keeps of it is
  `smith-pieces`.
- **Clients' wires** are section 9's.

## 3. Frozen versions

### 3.1 The rule

- **A worker wire's version is frozen** (`../draft/layouts-both-ways.md`,
  3): `jig-worker-wire` at 22, `smith-pieces` at 1, `smith-charter` at
  5, `smith-transcript` at 8, `smith-channel` at 14 and
  `temper-worker-wire` at 12, their headers'. Their crates keep no old
  text and no patch: no `schema/kept/`, `patches/` or `schema/wires`. A
  layout change is made in the current text alone; the engine derives
  its plans at the opening (jig's `protocol/channel/layouts.md`, 3).
  Common changes need no written converter: adding a field with a
  default, dropping a field with its tombstone (`dropped name: type = default`),
  adding a variant, widening a bound (skein's `data/codec/layouts.md`, 2).
  But for `jig-worker-wire`, the engine refuses the opening if a
  defaulted `text`, `bytes`, `secret` or `list` field brings a bound name
  it does not know, even when its plans derive (jig's
  `protocol/channel/wires.md`, 3).
- **`smith-pieces` is compared, never converted:** the engine keeps its
  bytes naming no layout, so a worker whose layout of it is not the
  engine's is refused at the opening (jig's `protocol/channel/wires.md`,
  3), and a change of its layout is met by draining the workers of the
  build before (`../draft/archive/redeploys.md`, 6.1).
- **No old text is kept:** transcript frames written before the
  registry hold turns at `smith-transcript` 7, which a store reads by
  the registry row its open wrote for that layout; dev.fabrig.ai's has
  held it since its open of 2026-11-25 (jig's
  `protocol/store/transcripts.md`, 6; `../draft/archive/store-floor.md`).
  No text of a build that isn't running is kept.
- **Each wire crate says what it speaks,** in a module `wire`:

  ```rust
  pub mod wire {
      pub const FAMILY: &str;          // the family's name, as its header gives it
      pub const VERSION: u16;          // its version, as its header gives it: frozen
      pub const TEXT: &str;            // its current schema text, embedded by include_str!
      pub fn layout() -> [u8; 32];     // TEXT's family fingerprint (skein's data/codec/fingerprints.md, 1)
  }
  ```

  `layout` parses `TEXT` with `skein_codec::schema::parse` and returns
  its `fingerprints().family().digest()`. A focused test in each crate
  holds `FAMILY` and `VERSION` equal to the header and `VERSION` equal to
  its frozen number. No worker wire has a `FLOOR`; `temper-host-tools`
  keeps its own (7). Every writer and
  reader that names the version names `wire::VERSION`:
  `Charter::FORMAT_VERSION`, `Turn::FORMAT_VERSION`,
  `smith_domain_session::record::VERSION`, `kind::VERSION`,
  `KindRule::FORMAT_VERSION`, jig's `VERSION`, and the version prefixes
  checked by hand (smith's `decode_charter` and `decode_transcript`).

### 3.2 The landing that freezes them

- **Its redeploy is every part together** (`../draft/archive/redeploys.md`,
  7.3): the credential changes (jig's `protocol/channel/wires.md`, 2), so
  a worker of the build before, which states version ranges and no
  texts, is refused at its opening, and a worker of this build is
  refused by an engine of the build before. The old texts that bridged
  the builds before go with this landing.
- **From then on** a worker of another build stays across an engine's
  redeploy wherever the engine's plans derive, and is refused at its
  opening by name where one does not, to be drained and restarted on
  the new build (`../draft/archive/redeploys.md`, 6.1).

## 4. At the opening

- **The statement is jig's** (jig's `protocol/channel/wires.md`, 2 and
  3): the worker's credential lists each wire it speaks, with its layout
  and text; the engine checks each wire it speaks against it, refuses
  with *wires* naming the first that fails, derives its plans, and
  accepts with the worker's kinds' table.
- **jig states its four:** `jig-worker-wire`, `smith-pieces`,
  `smith-charter` and `smith-transcript`, from their crates' texts (jig's
  `protocol/channel/layouts.md`, 7).
- **The wires are computed from the texts when the service builds**
  (`../testing/checks.md`, 1):
  the engine where it makes its workers host, the worker where it checks
  its link's limits, `worker_settings::prepare`, which keeps its channel
  table and its `Spoken`s in what it prepares. A start that checks its
  worker before making it, the standalone worker's and
  `temper-service`'s, makes the worker from what its check prepared
  rather than checking again.
- **temper hands both components its own** (6), `temper-worker-wire`,
  as an `Application` (jig's `protocol/channel/layouts.md`, 7): both
  take it from `temper_protocol_worker_wire`; the worker's service
  links no engine crate. `temper-service`, which joins both, holds the
  two equal and checks the worker's entry against the engine's
  `versions` table (10).
- **Named in both logs,** in the shell's own lines:
  - the engine, at each worker's link opening accepted: `worker NAME opened:
    at the engine's layouts` where nothing converts, else `worker NAME
    opened: converting WIRE, WIRE`, the wires whose layouts differ, in
    the engine's order;
  - the engine, at each opening it refuses: `worker NAME refused at its
    opening: REASON: TEXT`, or, refused by skein's machine before the
    credential is read, `a worker refused at its opening: framing:
    TEXT` (skein's `net/channel.md`, 5.2), the machine comparing no
    version;
  - the worker, at each refusal it hears: `the engine refused the
    opening: REASON: TEXT`, then its redial as after any refusal;
  - the engine, at its settled stop, for each worker its component
    tells of (jig's `protocol/channel/restart.md`, 3): `worker NAME
    told the engine restarts: wait MS ms`, MS the wait sent, or `worker
    NAME not told the engine restarts: its layout lacks the word; it
    keeps its grace`;
  - the worker, at each word heard (jig's `protocol/channel/restart.md`,
    4): `the engine said it restarts: runs kept up to MS ms after the
    link drops`, MS the longer of the wait and its grace;
  - the engine, at each pause its component could not send (jig's
    `protocol/channel/pause.md`, 3): `worker NAME not told to MODE: its
    layout lacks the kind; the engine's placement follows the ask`,
    MODE `pause`, `drain` or `resume`;
  - the engine, at each message to a running attempt it could not send
    (jig's `protocol/channel/layouts.md`, 5.2): `cannot send KIND to
    worker NAME: TEXT`, TEXT the refusal's words.

## 5. A mismatch costs no task

- **What a worker reads with a version:** a charter, by its first two
  bytes against `smith-charter`'s; each turn of a window resumed, by its
  own against `smith-transcript`'s; the workspace items, by their own
  against `temper-worker-wire`'s. `Items` is a `versioned` record, so
  its first two bytes are the family's version, as a charter's are. The
  engine converts each to the worker's layout before it sends it (jig's
  `protocol/channel/layouts.md`, 4), so the hand-out reads its own
  version.
- **Another version is refused, not failed:** bytes the worker cannot
  read, of another version, answer the assignment with jig's version
  refusal, naming the wire, the version the bytes carry and the version
  it reads (jig's `protocol/channel/wires.md`, 6). Bytes that carry its
  version and still do not decode are invalid, as before.
- **What the worker's layout cannot keep is refused at the engine,** as
  *untranslated*, before anything is sent: an assignment or a job whose
  records or payloads hold what the worker's layouts lack, which waits
  for another worker (jig's `protocol/channel/layouts.md`, 5.1).
- **The engine's log names either,** in place of `Invalid(Start(Other)):
  charter`: `task T run R attempt A refused: REASON`, REASON the
  refusal's words: `version mismatch: WIRE N, the worker reads V`, or
  `the worker's layout cannot keep WIRE DECLARATION.PART: WHY`. The
  task's retry notice says the same reason.

## 6. temper's payloads across layouts

- **temper's `Application`** (jig's `protocol/channel/layouts.md`, 7):
  its `Spoken` for `temper-worker-wire`, from its embedded text
  (`temper_protocol_worker_wire::spoken()`), and its roots:

  | `Carried` | Declaration of `temper-worker-wire` | Its direction |
  |---|---|---|
  | `Items` | `Items` | down |
  | `Left` | `Left` | up |
  | `Saved` | `Saved` | up |
  | `Preparation` | `Preparation` | up |
  | `JobSpec` | `JobSpec` | down |
  | `JobResult` | none: the job's result bytes, carried without conversion (8) | up |
  | `Grant` | `Identity` | down |
  | `Ask` | `Ask` (`../domain/forge/reds.md`, 5.4) | up |
  | `Answer` | `Answer` | down |

  The declaration each part holds is the one temper's own encoder and
  decoder of it name; a focused test holds the table against them. The
  engine derives each root's plan in its direction alone (jig's
  `protocol/channel/layouts.md`, 2.1).
- **`application` checks its limits** with `v1::check_limits` before
  computing its `Spoken` from the text where each service builds its
  wires (4).
- **A git identity's value is converted at each send** (`channel.md`,
  section 6): the engine keeps the two newest generations encoded at its
  own layout, as before, and jig's component takes the value to each
  channel's worker's layout at the assignment, the job and each renewal,
  keeping the grant's number, generation and validity.
- **A renewal the worker cannot keep** is a message to a running attempt
  that cannot go (jig's `protocol/channel/layouts.md`, 5.2): discarded,
  logged, and the channel taking nothing more. A renewal is kept only
  if each field the worker's layout lacks is at its compared default or
  marked `droppable` in the engine's text (skein's
  `data/codec/layouts.md`, 2).

## 7. The host tools' contract

- **What it is:** each declaration `temper-engine-smith`'s `SPECS`
  holds, as a charter offers it: its name, its effect and its input's
  JSON schema. Not its description: a description's wording is no
  layout, and a change of what a tool means moves the version by its
  author's hand, as a layout's change moves it by the gate's.
- **Its text:** one line per declaration, in name order, ties in `SPECS`
  order, each a JSON object `{"name":NAME,"effect":"read"|"write","input":SCHEMA}`,
  SCHEMA the declaration's input schema's bytes as a JSON string; each
  line ends with a newline. Its layout is the SHA-256 of the text.
- **Its version** is `temper_engine_smith::wire::VERSION`, from 1, with
  `FAMILY` `temper-host-tools` and `layout()` the text's SHA-256 (3.1's
  module, its fingerprint the digest of this text rather than of a
  schema, and its `FLOOR` kept).
- **Kept:** `contract/host-tools-v<N>.txt` in the crate, one per version
  from 1, without a gap; the current version's is the text the code
  renders.
- **Its floor:** `contract/floor`, one decimal line, which
  `wire::FLOOR` equals; at most the current, never lowered. A landing
  that moves the version keeps the floor at or below the version
  before, unless it declares the move breaking with
  `contract/host-tools-v<N>.breaking`, N the new version, holding its
  reason, when the floor is N.
- **A run's charter names its version:** the engine writes
  `wire::VERSION` as the charter's `contract` (smith's
  `protocol/charter.md`, 2), and each of the run's calls carries it up
  (jig's `domain/hosts/hub.md`, 8).
- **The engine answers every version from its floor:** a call of the
  current version as today; of an older one from the floor, the tool's
  line in that version's kept text equal to the current's, as today too;
  a tool whose line differs, or that the current contract lacks, by its
  adapter, which takes the old call's tool and input to a current tool
  and input, or answers it itself; a version outside the floor to the
  current, as an error: `this run's host tools are version V; this
  engine answers F..C`. So a run adopted across an engine's restart
  calls its tools as before.
- **Held by tests:** the crate's own, focused: the rendered text
  equals the current version's kept text; `SKEIN_CODEGEN_WRITE=1`, the
  tree's rewrite switch, writes it for review; a difference fails naming
  the fix, "the host tools' layout moved: move `wire::VERSION` to N+1
  and write its text with `SKEIN_CODEGEN_WRITE=1`"; `wire::FLOOR`
  equals `contract/floor`; and every declaration of a kept text from the
  floor that differs from the current's has its adapter, else it fails
  naming the tool and the fixes, an adapter or the floor raised by a
  breaking move. And `fabrig-boundary`'s: a kept text the base holds
  never changes, the kept versions run from 1 to the greatest without
  a gap, and the floor moves by the rules above against the base
  (`floor.md`, 6).
- **Not stated at the opening:** a worker carries a call's name, tool,
  input and contract as bytes and numbers and reads none of them.

## 8. Jobs

- **A job kind's request and result are not converted between layouts.**
  The spec is `temper-worker-wire`'s `JobSpec`, converted as its other
  parts are (6). For a project's command the worker carries the
  request and result as file bytes (`../domain/worker/jobs.md`), but
  reads the close request's `version` and goal for its commit title.
  For temper's own prepare, the worker reads the request and writes
  the result itself. The request forms name their `version`; the
  result forms do not (`tooling.md`, 4 and 5).
- **A kind's meaning to a worker** (its checkout, its command's verb, its
  delivery) is fixed by its number: a change of it is a new kind
  number, which a worker declares in its hello only once it runs it, so
  the engine places that job on no worker of an older build
  (`../domain/worker/jobs.md`).
- **A kind's bounds are judged as its number is:** a job goes only to a
  worker whose hello declares its kind with a spec, a result and a
  deadline at least the engine's declaration of it, and waits, saying
  why, while none does (jig's `domain/engine/fleet.md`, 2, *A job goes
  only where a host runs it*). An engine whose declaration of a kind
  grows (a longer close deadline, a larger spec) places none of that
  kind on a worker of the build before until it restarts on the new
  build; `upgrade --check` names that worker (10.2).
- **A step a job cannot do without** is asked for in what the engine
  sends: a field or a variant of the job's spec or items that the
  engine sets, which a worker whose layout lacks it cannot keep, so the
  job is refused for that worker as *untranslated* and waits for another
  (jig's `protocol/channel/layouts.md`, 5.1), and `upgrade --check`
  names the worker (10.2); or it moves the kind's number. A step of the
  worker's own whose absence the job's command meets needs neither: main's
  counts (`../domain/worker/checkouts-prepare.md`, 5), without which the
  checked close breaks at once, judging nothing, and is tried again in its
  job's class (`tools/pre-pr/design.md`, 4.1), is one.
- **A close asks for its typed refusal in its spec:** `JobSpec`'s
  `refusal`, a `bool` defaulting to `false`, true on every close job the
  engine makes (`tooling.md`, 4.2). A worker given it sets
  `TEMPER_CLOSE_REFUSAL=3` in the command's environment. It is no new
  kind: the kind's checkout, verb and delivery stay as they are, and a
  worker whose layout lacks the field can't keep a spec that asks, so
  the engine sends it none and the job waits for a worker that can,
  spending no try (jig's `protocol/channel/layouts.md`, 5.1). A spec of
  an earlier engine, without it, reads as `false`, so a worker of this
  build runs that close as before. `JobSpec` also has `result: u32 = 512`,
  a droppable hint for the result bound in bytes. The engine sends 8,192
  on a close; a new worker gives the command the lesser of this and its
  own declared bound. A spec without `result` defaults to 512 on a new
  worker; a worker without the field still keeps the ask and writes at
  most 512. The requester's decision of 2026-11-27 keeps those workers
  placed while carrying more failure names on newer workers. The layout
  changes but the wire version does not.
- **`Items`' `rerun`,** a project's bound
  on the checks' reruns (`../domain/forge/reds.md`, 5.2), goes the same
  way as the refusal ask: set only for a project that states its `flakes`,
  kept by no worker whose layout lacks it, and none from an earlier engine, which
  leaves the worker's own figure. A new kind was turned down: every
  project's policy grants the job kinds its goals may make (the close
  job, job 4, `configuration/stock.md`), so a kind 5 would be refused
  until each deployment's settings granted it.
- **A close asks for its outdated finding in its spec too:** `JobSpec`'s
  `outdated`, a `bool` defaulting to `false` and marked `droppable`
  (skein's `data/codec/README.md`, 3), true on every close job the
  engine makes (`tooling.md`, 4.2). A worker given it sets
  `TEMPER_CLOSE_OUTDATED=3` in the command's environment. Unlike
  `refusal`, a worker whose layout lacks it keeps the spec, the field
  dropped by the Keep plan (skein's `data/codec/layouts.md`, 2), so
  that worker runs the close as before, its command not told, and a
  close it breaks is read as before: the worker says, by the layout it
  states at its opening, whether it carries the ask. Dropping it does
  not bypass the separate `find` and deadline requirements below. A
  spec of an earlier engine, without `outdated`, reads as `false`.
  Why droppable: where the worker can run the close, losing the ask
  costs only the catch-up its finding would have brought.
- **A merge or close job asks its worker to find its hook:** `JobSpec`'s
  `find`, a `bool` defaulting to `false`, and `fallback`, bytes of the
  `command`'s form defaulting to empty (`hooks.md`, 4.2 and 4.3). The
  engine sets `find` on every merge and close job it makes, its
  `command` the hook's path and its `fallback` the project's
  `merge_job` or `close_job`, or empty. As `refusal`, it is no new kind,
  and a worker whose layout lacks `find` can't keep a spec that sets it,
  so the engine sends it none and the job waits for a worker of this
  build, spending no try; a spec of an earlier engine reads `find`
  false and `fallback` empty, and a worker of this build runs its
  command as that engine expects.
- **A hook's ceiling rides in the items:** `Items`' `hook_timeout_max_seconds`,
  a `u32` defaulting to 3,600 and `droppable`, the project's
  `hook_timeout_max_seconds` on every run's items and every job's
  (`hooks.md`, 2.3). A worker whose layout lacks it drops it and reads
  no hook's `config.toml`, as before (`hooks.md`, 6); items of an
  earlier engine read 3,600, the setting's default.
- **The merge and close kinds' deadline** is the ceiling's greatest,
  86,400 seconds, in the engine's declaration and the worker's alike,
  and each merge or close job's deadline its project's ceiling
  (`hooks.md`, 2.3; `../domain/worker/jobs.md`, 2): a grown declaration,
  so an engine of this build places neither kind on a worker of an
  earlier build (above), as `find` alone would keep them off it.

## 9. Clients

- **The families:** `jig-client-wire` and `jig-public-wire`
  (`jig/crates/jig-client-protocol-wire`), `jig-engine-cursor`
  (`jig/crates/jig-protocol-engine-clients`), `temper-client-wire` and
  `temper-public-wire` (`temper/crates/temper-client-protocol-wire`),
  and the reference UI's `jig-client-view-records`
  (`jig/crates/jig-client-view`).
- **Their versions are frozen** (`../draft/layouts-both-ways.md`, 3),
  with no kept text, no patch, no bridge and no list of layouts: a
  layout is the family's fingerprint, and the gate refuses a landing
  that moves one of their versions, naming the family. The base is the
  gate's landing base (`floor.md`, 6), the family found by its header
  among the crate's `schema/*.schema`, a family of several files
  read as its generated code reads them; a family the base lacks
  passes (jig's `protocol/client/versions.md`, section 1). A focused test in each crate holds
  the version constants equal to the header (`VERSION`,
  `PUBLIC_VERSION`, `CLIENT_VERSION`).
- **An engine serves its one version and layout of each,** and answers
  another version *version not served*, naming the owner (jig's
  `protocol/README.md`, section 7); a client judges the layouts alone
  (jig's `protocol/client/versions.md`, section 1). temper declares its owner's
  client layout beside its version, to the clients' component and to
  each client it makes, and its public kinds' layout beside theirs: a
  client at another layout of temper's owner finds that owner not
  served at the hello, and a public read at another layout is answered
  not served.
- **A page refused reloads at most once:** when the link turns out of
  date, or one of temper's owners is not served, the browser's page
  reloads itself to load the engine's bundle only if it can read and
  write its marker in session storage and has not already marked a
  reload. If storage is unavailable, the marker cannot be written, or
  the page is still out of date after reloading, it shows the column
  (`../client/view.md`, section 1) without reloading. The TUI says to
  run the temper the engine runs; `temper-client` reports `client is
  out of date` for the link and `version not served` for a family's
  refusal. A kept request of another layout is dropped with a notice
  (jig's `protocol/client/small-store.md`, section 1).

## 10. `versions`, the stored wires and the workers' openings

- **Two tables:** `temper_engine_service::wires()` builds the display
  table for `versions`, in this order: jig's four
  (`jig_protocol_worker_wire::spoken()`), then `temper-worker-wire`,
  each its name and its layout, from its `Spoken`; no wire's version is
  read as a range. The host tools' contract is no wire of it: a text
  contract with versions of its own (7), shown apart with its floor
  and its current version. The store takes its own static `WIRES`;
  tests hold its names and versions equal to the `Spoken`s' names and
  frozen versions, in order, then `temper-host-tools` at the
  contract's current version. The engine's workers component builds
  its own list of `Spoken`s. The worker's link states the same from its
  crates (4).
- **`versions` prints,** after the families' lines (`binaries.md`,
  section 2), one line per wire in the table's order, then the host
  tools' contract:

  ```
  wire NAME FINGERPRINT
  contract temper-host-tools FLOOR CURRENT
  ```

  FINGERPRINT the wire's family fingerprint, 64 lowercase hex digits,
  once; FLOOR and CURRENT the contract's floor and version (7).
- **The store keeps the wires** the last start spoke (jig's
  `protocol/store/records.md`, 3, `Wires`): one `{name, version}` per
  wire of the table, its frozen version, and `temper-host-tools` at the
  contract's current version, its stored form as before.

### 10.1 The workers' openings kept

- **At each link opening the engine accepts,** once the worker's hello
  decodes, its workers component hands the service the worker's name,
  the wires that convert and its statement, the worker's texts and its
  hello's job kinds among it (jig's `protocol/channel/wires.md`, 3, *The
  statement handed up whole*). A figures-only opening, and a channel
  lost before its hello, hand up no statement (jig's
  `protocol/channel/figures.md`, 3). The service logs the link opening
  (section 4) and hands the root the name and the statement; the root
  commits them, `Write::Opening`, in a decision of its own, `at` its
  wall time (jig's `protocol/store/records.md`, 3, *The workers' openings*).
- **Since the engine's last start:** each start's wires commit clears
  them, and an opening the root hears before that commit is durable is
  committed once it is, so what the store keeps is each worker's last
  opening since the engine's last start.
- **Never read by the engine:** they are for `upgrade --check` and
  `upgrade --workers` (10.2), which read a stopped deployment or a
  checkpoint of a serving one, the state at one commit
  (`../draft/archive/redeploys.md`, 5.1; `checkpoints.md`, 2.1 and 3).

### 10.2 Whether the workers may stay

`upgrade --check` (`binaries.md`, section 2) judges each worker by its
kept opening (`../draft/archive/redeploys.md`, 5.1), after its jobs' and
restores' lines. `upgrade --workers` prints `rehearsal linked N files,
copied M` first, then the verdict, with no job or restore before it
(`binaries-box.md`, section 2). It can read the checkpoint at the
engine's settled stop, so that a redeploy judges again what may have
changed since its check (`checkpoints.md`, 2.1). Every line starts with
the binary's name and `upgrade:`, as the check's others do, and the
verdict does not change its exit code.

1. **The copy keeps no wires,** a store from before the row: `workers
   may stay: unknown; the copy keeps no wires`, and nothing more.
2. **The host tools' contract,** which the opening does not state
   (section 2): it must be kept, at a version this binary answers, from
   its floor to its current, since a run adopted across the restart
   goes on calling the tools of the contract it started with (section
   7). Else `the host tools' contract: kept V, this binary answers
   F..C` or `the host tools' contract is not kept by the copy`, and no
   worker may stay. A kept wire the table lacks is passed over.
3. **The copy's current workers,** read without a restore: temper's
   default workers (`Defaults.workers`, the box's `local` where it
   passes one: `cluster.md`, 1) with the copy's stored worker settings
   over them (jig's `protocol/store/opening-rehearsal.md`, 1, *The
   workers kept*), the withdrawn ones left out, in number order. Where
   a stored worker setting does not decode: `workers may stay: unknown;
   the copy's workers cannot be read`, and nothing more of this step or
   the next.
4. **Each worker the copy names,** in number order, by its kept
   opening, judged in three parts, each as this binary's engine would
   judge it with the room, the limits and the job kinds its start would
   take from the configuration:
   1. **Its opening,** by `jig_protocol_engine_workers::judge` (jig's
      `protocol/channel/wires.md`, 3) with this binary's worker wires and
      roots, deriving the plans from the kept texts. Where it refuses:
      `worker NAME may not stay: wires: TEXT` or `worker NAME may not
      stay: limits: TEXT`, TEXT the opening's refusal's words, and
      nothing more of that worker.
   2. **Its job kinds:** for each kind this binary's engine declares
      (`temper_engine_service_vocabulary::limits::jobs`, ascending), the
      kept hello's kinds judged by `jig_domain_fleet::job_short`, the
      function placement judges by (jig's `domain/engine/fleet.md`, 2).
      Each kind it falls short of: `worker NAME must restart: job kind K
      (VERB): WHAT, this binary's REQUIRED`, VERB the kind's
      (`../domain/worker/jobs.md`: `prepare`, `key`, `merge`, `close`),
      WHAT `it runs none` (with no `, this binary's` part), `its spec
      takes N bytes`, `its result takes N bytes` or `its deadline is N
      s`, REQUIRED the same unit: `worker s1 must restart: job kind 4
      (close): its deadline is 600 s, this binary's 1800 s`.
   3. **What this binary would send it:** each gap of its kept texts,
      by `jig_protocol_engine_workers::gaps` (jig's
      `protocol/channel/layouts.md`, 5.4), a part of what this binary's
      engine sends whose Keep plan may refuse a value: `worker NAME must
      restart: TEXT, sent in WHERE; this binary's WIRE is at HEX`, TEXT
      the words a job or an assignment refused for it as *untranslated*
      would carry (`the worker's layout cannot keep WIRE
      DECLARATION.PART: WHY`), HEX this binary's layout of WIRE, its
      first 8 bytes in lowercase hex as the opening's refusals give a
      layout, and WHERE by the gap's site: `frames to workers` for a
      frame's record, `assignments` for a charter, `resumed assignments`
      for a turn or a part, `assignments and jobs of kinds KINDS` for
      the items, `jobs of kinds KINDS` for a job's spec, `assignments,
      jobs of kinds KINDS, list reads and renewals` for a grant's value,
      `answers to asks` for an answer, KINDS this binary's declared kinds
      as `4 (close)`, joined by `, ` and a last ` and `.

   With none of these: `worker NAME may stay: opened AT`, AT the
   opening's time in UTC to the second, `YYYY-MM-DDTHH:MM:SSZ`.
   Without a judgeable opening:
   - an opening kept by a build before layouts, a row of the openings'
     family's version 1: `worker NAME unknown: opened before layouts`,
     and it may not stay;
   - one kept by a build before job kinds were kept, a row of its
     version 2: `worker NAME unknown: opened before its job kinds were
     kept`, and it may not stay;
   - with no opening kept: `worker NAME unknown: no opening kept since
     the engine's last start`, and it may not stay.

   A kept opening of a name the copy no longer names (withdrawn, or
   never a setting) is passed over.
5. **The verdict:** `workers may stay: yes` when step 2 passed and every
   worker named may stay, with none named too; else `workers may stay:
   no`.

- **Two judgements:** the opening's, by which a worker may connect at
  all: each of its wires at this binary's layout, or at one whose plans
  derive both ways it crosses, where a value its layout cannot keep is
  no reason, since only a value refuses (jig's
  `protocol/channel/layouts.md`, 5.1); and the work's, by which it may
  stay: it runs every job kind this binary's engine makes, at its
  bounds, and keeps every value of every part that engine sends. A
  worker the opening accepts but the work's judgement does not must
  restart, since its work would wait for another worker: a job of a
  kind it falls short of is never placed there, and one it cannot keep
  is refused there as *untranslated*. A part's Keep plan that refuses
  only some values (a field the worker lacks, sent at its default) is
  still a gap: the engine may send it otherwise. Its author marks a
  field the worker may lose `droppable`, which no plan refuses.
- **Why the work is judged** (draft-8, 2026-11-25): staging's verdict
  said `workers may stay: yes` for an engine whose close kind declared
  a 30-minute deadline over a worker whose hello declared it at 10
  minutes; the close job was claimed and never placed, and sat
  *claimed by a host* until the worker restarted on the new build. The
  missing `counts` step was not the cause: that build's checked close
  only warned without main's counts, where it now breaks (section 8).
- **The window is not judged here:** the whole rehearsal/tool time is
  a rehearsal line (`binaries.md`, section 2), and comparing the window
  with the workers' grace is the referee's of a redeploy, its
  operator's outside this repository (`../draft/archive/redeploys.md`, 7.5).

## 11. Testing

Focused and fuzzy alone; the drills run on the deployments after landing
(`../draft/archive/redeploys.md`, 7.5).

- **Each crate's `wire` module:** `FAMILY` and `VERSION` equal the
  header, and a frozen wire's `VERSION` its frozen number; `layout`
  equals a fingerprint the test computes from the text with its names
  changed.
- **A worker's table once:** what `worker_settings::prepare` keeps is
  the table jig's `kinds::schema` makes over its link's limits, and a
  worker made from it opens as one made by `Worker::new`, stating every
  wire's layout and text.
- **The opening and conversion** at jig's components and in its
  worker-link world (jig's `protocol/channel/layouts.md`, 8); temper's
  roots against its encoders (6); each root converted in its direction
  between `temper-worker-wire`'s current text and one derived from it, a
  field with a default on each side, arriving as itself, and an
  `Identity` whose field the worker lacks holding a value refused.
- **The host tools' contract and the clients' families:** their gates'
  mutations over a scratch copy. The contract: a declaration's input
  changed with its version unmoved, refused, and with it moved,
  passing; a floor passed without a breaking file, refused; an older
  version's changed tool without its adapter, refused. The clients'
  freeze (9): a family's field added with its version unmoved, passing;
  its version moved, refused, naming the family.
- **`versions`:** its lines for the table, whole, in the binaries' tests.
- **The openings kept** (10.1): the engine's workers component handing up
  a statement as the worker stated it, its texts among it; the store
  keeping one row per worker, replaced by its next opening and cleared
  by a start's wires, and the rehearsal answering them; the root
  committing an opening, and one heard before its start's wires after
  them.
- **The verdict** (10.2), over kept openings and wires made by hand:
  every worker opened at this binary's layouts, with this binary's job
  kinds, yes; one at a derived layout whose plans derive and keep every
  value, yes; one whose text gives no plan, one whose output differs,
  each named with the opening's words; one whose hello declared the
  close kind with a shorter deadline, and one declaring no close kind,
  each told to restart naming kind 4 (close) and the figures; one at an
  older layout of `temper-worker-wire` lacking a part the close job's
  items or spec carry, told to restart naming the part, `jobs of kinds`
  with `4 (close)` among them, and this binary's layout; one of the
  openings' version 1 and one of version 2, unknown; one with no
  opening, unknown; `temper-host-tools` kept at another version; the
  copy keeping no wires. And in the one-process world, a rehearsal of a
  deployment whose worker opened says it may stay, by name.
- **The close job kept** (section 8): in temper's engine, a worker whose
  hello declares the close kind with a shorter deadline, and one whose
  layout cannot keep the close job, each the only worker: a goal's
  close job stays due, never claimed, its task's wait naming the worker
  and what it lacks, and is placed once a worker that covers it says
  hello.

## 12. Open questions

- **The host tools' meaning.** Default: by its author's hand, as above;
  a description's wording judged by its reviewer.
- **An adapter's answer.** An adapter maps an old call to a current one,
  whose answer goes back as the current tool renders it. Default: an
  answer whose meaning changed for an old caller is a breaking move.
