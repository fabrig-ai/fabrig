# temper's wires

Provisional, 2026-11-12, revised 2026-11-13. The wires between temper's
engine, its workers, their agents and its clients: which they are, how
each one's version moves with its layout, the floor each build speaks
from and what bridges them, what the engine and a worker state of them
at their opening, what a mismatch costs, how the host tools' contract
is answered across versions, and how `versions` prints them. It carries
`../draft/redeploys.md`, sections 3, 4 and 8, for temper's parts. The
mechanisms are skein's (`data/codec/command.md`, the wire rule;
`data/codec/bridges.md`, reverses, floors and bridges;
`data/codec/README.md`, section 9, fingerprints) and jig's
(`protocol/channel/wires.md`, the opening's statement and a mismatch's
refusal; `protocol/channel/bridges.md`, translation in the channel's
components); this document says what temper's deployment puts through
them.

## 1. In one page

- **Versions move with layouts** (the user, 2026-11-12). Each wire
  between temper's parts is a family of a patch owner, marked a wire: a
  landing that changes its layout moves its version by one, keeps the
  old text and brings its forward patch and its reverse, and the gate
  refuses one changed in place (3).
- **Each build speaks each wire from its floor** (3.2), kept beside the
  wire's name in its crate's `schema/wires`, and translates between the
  versions it speaks by the wire's bridge, in jig's channel components.
  A change no reverse carries is breaking, and raises the floor.
- **The opening states them** (4): a worker states each wire's versions
  and layouts; the engine refuses one it shares no layout with, naming
  the wire, and states back the agreed versions.
- **A mismatch at an assignment costs no try** (5): an assignment a
  worker cannot read, or that cannot go down to it, waits for one that
  can, spending no try.
- **temper's payloads are bridged as jig's are** (6): temper hands both
  components its bridge and names the declaration each part holds.
- **The host tools' contract is versioned whole** (7): a run's charter
  names its version, each call carries it, and the engine answers every
  version from its floor.
- **Clients' versions move with their layouts too** (9), with no kept
  text and no bridge; a page refused reloads once to the engine's
  bundle.
- **`versions` prints each wire's range and layouts** (10), and the store
  keeps the wires the engine's last start spoke and each worker's last
  opening since (10.1), by which `upgrade --check` says which workers may
  stay (10.2).

## 2. The wires

| Wire | Crate | Between | Stated at the opening |
|---|---|---|---|
| `jig-worker-wire` | `jig/crates/jig-protocol-worker-wire` | the engine and its workers: the channel's records, its kinds' table included (jig's `protocol/channel/wires.md`, 4) | by jig |
| `smith-pieces` | `smith/crates/smith-channel`, its second family | the pieces of smith's channel the engine keeps or sees: a call's name and a delivery's evidence (`channel.md`, section 2; smith's `protocol/README.md`, section 8) | by jig |
| `smith-charter` | `smith/crates/smith-charter` | the engine writes charters, which the worker and its agent read; the agent writes the result, which the engine reads | by jig |
| `smith-transcript` | `smith/crates/smith-transcript` | the agent writes turns; the engine keeps them and resumes a window on a worker | by jig |
| `temper-worker-wire` | `temper/crates/temper-protocol-worker-wire` | temper's payloads in jig's channel: the workspace items, the work, the preparation, a job's spec and a git identity's value (`channel.md`) | by temper, through jig |
| `temper-host-tools` | `temper/crates/temper-engine-smith` | the host tools the engine offers in a charter and answers when a run calls them (7) | no: the engine's alone |
| `smith-channel` | `smith/crates/smith-channel` | a worker and its agents, per-build (`../draft/redeploys.md`, 4.4) | no: smith's own opening, between a worker and its agent |

- **Each wire's name is its family's,** as its schema's header gives it;
  the host tools' contract, which is no schema, is named
  `temper-host-tools`.
- **A wire's version is its current schema's header,** read through its
  crate's `wire::VERSION` (3.1); `smith-pieces`' through
  `smith_channel::pieces::wire`.
- **`smith-channel` is per-build:** a worker pins its agent's program
  and configuration at its start, so the channel between them never
  spans two builds; its version moves with its layout and keeps its
  text, with no reverse, no floor below its current and no breaking
  declaration (`schema/wires`, `smith-channel build`). What the engine
  keeps of it is `smith-pieces`.
- **Clients' wires** are section 9's.

## 3. Versions move with layouts

### 3.1 The rule

- **Each wire family is a patch owner's,** marked a wire in its crate's
  `schema/wires` (skein's `data/codec/command.md`, section 1): its kept
  texts are frozen from its first, so a landing that changes its layout
  in anything but names, comments, `boxed` and the text's layout moves
  its version by one, keeps the old text and brings its forward patch,
  written by `cargo run -p skein-codegen -- patch <crate>` as a stored
  family's is (skein's `data/codec/command.md`, 2), with its reverse,
  derived or its author's, or its breaking declaration (skein's
  `data/codec/bridges.md`, 2 and 3). The old version's golden bytes are
  made from its kept text, and taken up and down by the patch's golden
  test (skein's `data/codec/bridges.md`, 6).
- **The gate holds it:** `fabrig-boundary`'s `patches` test
  (`baseline.md`, 7) lists each wire crate as an owner, so its check
  refuses a wire changed in place (`Unpatched`, or `Version` with the
  verdict `Unmoved`), a kept text edited (`Changed`), a kind's number
  or direction changed (jig's `protocol/channel/wires.md`, 4), a floor
  passed or lowered (`Floor`) and a reverse a floor needs that does not
  plan (`Unbridged`). Each owner's golden tests hold its reverses in the
  focused suite.
- **Each wire crate says what it speaks,** in a module `wire`:

  ```rust
  pub mod wire {
      pub const FAMILY: &str;          // the family's name, as its header gives it
      pub const VERSION: u16;          // the current version, as its header gives it
      pub const FLOOR: u16;            // its floor, the generated kept::<family>::FLOOR
      pub fn layout() -> [u8; 32];     // the current text's family fingerprint (skein's data/codec/README.md, 9)
  }
  ```

  `layout` parses the crate's current schema text, embedded by
  `include_str!`, with `skein_codec::schema::parse`, and returns its
  `fingerprints().family().digest()`. A focused test in each crate holds
  `FAMILY` and `VERSION` equal to the current text's header. Every
  writer and reader that names the version names `wire::VERSION`:
  `Charter::FORMAT_VERSION`, `Turn::FORMAT_VERSION`,
  `smith_domain_session::record::VERSION`, `kind::VERSION`,
  `KindRule::FORMAT_VERSION`, jig's `VERSION`, `PAYLOAD_VERSION`, and
  the version prefixes checked by hand (smith's `decode_charter` and
  `decode_transcript`) are it or derived from it, so a version moves in
  its header and its constant alone.
- **Kept texts and forward patches stay for good** (`../draft/redeploys.md`,
  3.3): stored turns and calls' names keep the versions they were
  written in, and a build reads every kept version, whatever its floor.

### 3.2 The floor

- **Kept in each crate's `schema/wires`** (skein's
  `data/codec/bridges.md`, 4): a build speaks, negotiates and translates
  down to every version from it to its current.
- **At the landing of this design's milestone** (R4,
  `../draft/redeploys.execution.md`): `jig-worker-wire` 12, its current
  13; `smith-charter` 4, its current 5; `smith-pieces`,
  `smith-transcript` and `temper-worker-wire` at their current. Each move
  this milestone makes keeps its floor at the version before, so its
  reverse is held from the start.
- **The redeploy of that landing is every part together**
  (`../draft/redeploys.md`, 7.3): a worker of the build before states
  `smith-channel` where the engine now asks `smith-pieces`, and is
  refused at its opening by name (`wire smith-pieces: the worker states
  none`); and a worker of this build meeting an engine of the build
  before is refused by it for the same wire, the other way.
- **From then on,** a landing that moves a wire keeps its floor at or
  below the version before, unless it declares the move breaking, and a
  floor rises alone once dev.fabrig.ai and staging.fabrig.ai have run
  past it for a week (`../draft/redeploys.md`, section 12).

## 4. At the opening

- **The statement is jig's** (jig's `protocol/channel/wires.md`, 2 and
  3): the worker's credential lists each wire it speaks, with its lowest
  version and each version's layout; the engine checks each wire it
  speaks against it, refuses with *wires* naming the first that fails,
  keeps each wire's agreed version with the channel, and states them
  back in its first record (jig's `protocol/channel/bridges.md`, 3).
- **jig states its four:** `jig-worker-wire`, `smith-pieces`,
  `smith-charter` and `smith-transcript`, from their crates' floors and
  kept texts.
- **temper hands both components its own** (6), `temper-worker-wire`,
  as an `Application` (jig's `protocol/channel/bridges.md`, 8): the
  engine's from `temper_engine_service`'s table (10), the worker's from
  `temper_protocol_worker_wire`, the same entry, since the worker's
  service links no engine crate. `temper-service`, which joins both,
  holds the two equal. Neither component knows what they cover.
- **Named in both logs,** in the shell's own lines:
  - the engine, at each worker's channel accepted: `worker NAME opened:
    jig-worker-wire V, smith-pieces V, smith-charter V,
    smith-transcript V, temper-worker-wire V`, each wire's agreed
    version, in the statement's order;
  - the engine, at each opening it refuses: `worker NAME refused at its
    opening: REASON: TEXT`, or, refused by skein's machine before the
    credential is read, `a worker refused at its opening: version:
    TEXT` (skein's `net/channel.md`, 5.2);
  - the worker, at each refusal it hears: `the engine refused the
    opening: REASON: TEXT`, then its redial as after any refusal;
  - the worker, at a channel it closes itself after the engine's
    statement or for its absence: jig's text (jig's
    `protocol/channel/bridges.md`, 3 and 6.3), then its redial.

## 5. A mismatch costs no task

- **What a worker reads with a version:** a charter, by its first two
  bytes against `smith-charter`'s; each turn of a window resumed, by its
  own against `smith-transcript`'s; the workspace items, by their own
  against `temper-worker-wire`'s. `Items` is a `versioned` record, so
  its first two bytes are the family's version, as a charter's are.
  jig's worker link translates each to the worker's current before the
  hand-out reads it (jig's `protocol/channel/bridges.md`, 7).
- **Another version is refused, not failed:** bytes the worker cannot
  read, above its current, answer the assignment with jig's version
  refusal, naming the wire, the version the bytes carry and the range
  it reads (jig's `protocol/channel/wires.md`, 6). Bytes that carry a
  version it reads and still do not decode are invalid, as before.
- **What cannot go down is refused at the engine,** as *untranslated*,
  before anything is sent: an assignment or a job whose records or
  payloads hold what the worker's agreed versions cannot, which waits
  for another worker (jig's `protocol/channel/bridges.md`, 6.1).
- **The engine's log names either,** in place of `Invalid(Start(Other)):
  charter`: `task T run R attempt A refused: version mismatch: WIRE N,
  the worker reads L..H`. The task's retry notice says the same reason
  (jig's `protocol/channel/wires.md`, 6).

## 6. temper's payloads across versions

- **temper's `Application`** (jig's `protocol/channel/bridges.md`, 8):
  its `Spoken` for `temper-worker-wire`, from its floor and kept texts;
  its bridge, `temper_protocol_worker_wire`'s generated
  `kept::temper_worker_wire::bridge`, with a capacity from the payload
  limits; and its roots:

  | `Carried` | Declaration of `temper-worker-wire` |
  |---|---|
  | `Items` | `Items` |
  | `Left` | `Left` |
  | `Saved` | `Saved` |
  | `Preparation` | `Preparation` |
  | `JobSpec` | `JobSpec` |
  | `JobResult` | none: the project's tooling's result file, carried unread (8) |
  | `Grant` | `Identity` |
  | `Ask` | `Ask` (`../domain/forge/reds.md`, 5.4) |
  | `Answer` | `Answer` |

  The declaration each part holds is the one temper's own encoder and
  decoder of it name; a focused test holds the table against them.
- **A git identity's value is translated at each send** (`channel.md`,
  section 6): the engine keeps the two newest generations encoded in its
  current version, as before, and jig's component takes the value down
  to each channel's agreed version at the assignment, the job and each
  renewal, keeping the grant's number, generation and validity.
- **Grants go down whole:** a grant's renewal goes to a running attempt,
  so `Identity`'s reverse is total from the floor (skein's
  `data/codec/bridges.md`, 5.3), which a focused test in
  `temper-protocol-worker-wire` holds; a change that breaks it is
  breaking.

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
  schema).
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
  (jig's `domain/hosts/hub.md`, 8): a call from a worker of an older
  build, or from a charter of version 4, carries 1.
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
  a gap, and the floor moves by the rules above against the base, read
  as the `patches` test reads it (`baseline.md`, 7).
- **Not stated at the opening:** a worker carries a call's name, tool,
  input and contract as bytes and numbers and reads none of them.

## 8. Jobs

- **A job kind's request and result are no wire between temper's
  builds.** The spec is `temper-worker-wire`'s `JobSpec`, under that
  wire's version and bridged as its other parts are (6); its request
  and result are the files the project's command reads and writes,
  which the worker carries as bytes and never reads
  (`../domain/worker/jobs.md`), versioned by their own `version` member
  between the engine and the project's tooling (`tooling.md`).
- **A kind's meaning to a worker** (its checkout, its command's verb, its
  delivery) is fixed by its number: a change of it is a new kind
  number, which a worker declares in its hello only once it runs it, so
  the engine places that job on no worker of an older build
  (`../domain/worker/jobs.md`).

## 9. Clients

- **The families:** `jig-client-wire` and `jig-public-wire`
  (`jig/crates/jig-client-protocol-wire`), `jig-engine-cursor`
  (`jig/crates/jig-protocol-engine-clients`), `temper-client-wire` and
  `temper-public-wire` (`temper/crates/temper-client-protocol-wire`).
- **Their versions move with their layouts** (`../draft/redeploys.md`,
  section 8), with no kept text, no patch and no bridge: a landing whose
  text of a family has another family fingerprint than the base's text
  of that family, at the same version, is refused by the gate, naming
  the family and the fix, "move its version". The base is read as the
  `patches` test reads it (`baseline.md`, 7), the family found by its
  header among the crate's `schema/*.schema`, a family of several files
  read as its generated code reads them. A family the base lacks, or
  one whose version moved, passes. A focused test in each crate holds
  the version constants equal to the header (`VERSION`,
  `PUBLIC_VERSION`, `CLIENT_VERSION`).
- **An engine serves its one version of each,** and refuses another with
  *version not served*, naming the ranges served (jig's
  `protocol/README.md`, section 7).
- **A page refused reloads, once:** when the link turns out of date, or
  one of temper's owners is not served, the browser's page reloads
  itself to load the engine's bundle. A page that reloaded for it and is
  out of date again shows the column (`../client/view.md`, section 1)
  and reloads no more. `temper-client` and the TUI say to upgrade, as
  they do. A kept request of another version is dropped with a notice
  (jig's `protocol/client/README.md`, section 7).

## 10. `versions`, the stored wires and the workers' openings

- **One table,** `temper_engine_service::wires()`, in this order: jig's
  four (`jig_protocol_worker_wire::spoken()`), then `temper-worker-wire`,
  then `temper-host-tools`, each a `Spoken` from its floor. The engine's
  workers component takes its `temper-worker-wire` entry, and the
  worker's link states the same entry from its crate (4); the store and
  `versions` take the whole.
- **`versions` prints,** after the families' lines (`binaries.md`,
  section 2), for each wire in the table's order:

  ```
  wire NAME LOWEST HIGHEST
  layout NAME VERSION FINGERPRINT
  ```

  one `layout` line per version from LOWEST to HIGHEST, FINGERPRINT its
  64 lowercase hex digits, LOWEST the floor.
- **The store keeps the wires** the last start spoke (jig's
  `protocol/store/records.md`, 3, `Wires`): one `{name, version}` per
  wire of the table, its version the highest.

### 10.1 The workers' openings kept

- **At each opening the engine accepts,** its workers component hands
  the service the worker's name, its agreed versions and its statement
  (jig's `protocol/channel/wires.md`, 3, *The statement handed up
  whole*). The service logs it as before (section 4) and hands the root
  the name and the statement; the root commits them, `Write::Opening`,
  in a decision of its own, `at` its wall time (jig's
  `protocol/store/records.md`, 3, *The workers' openings*).
- **Since the engine's last start:** each start's wires commit clears
  them, and an opening the root hears before that commit is durable is
  committed once it is, so what the store keeps is each worker's last
  opening since the engine's last start.
- **Never read by the engine:** they are for `upgrade --check` (10.2),
  which runs with the engine stopped (`../draft/redeploys.md`, 5.1).

### 10.2 Whether the workers may stay

`upgrade --check` (`binaries.md`, section 2) judges each worker by its
kept opening (`../draft/redeploys.md`, 5.1), after its patches' and
restores' lines. Every line starts with the binary's name and
`upgrade:`, as the check's others do, and none changes its exit code.

1. **A pending patch marked fresh:** no worker may stay, `a pending
   patch requires fresh workers; stop the workers`, then `workers may
   stay: no`, and nothing more.
2. **The copy keeps no wires,** a store from before the row: `workers
   may stay: unknown; the copy keeps no wires`, and nothing more.
3. **The engine's own wires,** those of the table the opening does not
   state (`temper-host-tools`, section 2): each must be kept, at a
   version within this binary's range of it, from its floor to its
   current, since a run adopted across the restart goes on calling the
   tools of the contract it started with (section 7). Else, for each,
   `wire NAME: kept V, this binary speaks L..H` or `wire NAME is not
   kept by the copy`, and no worker may stay. A kept wire the table
   lacks is passed over.
4. **Each worker the configuration names,** in its order, by its kept
   opening, judged by `jig_protocol_engine_workers::judge` (jig's
   `protocol/channel/wires.md`, 3) with this binary's table, `wires()`,
   and the room and kinds' bounds its engine's workers component would
   take from the configuration, as that engine's opening would judge it:
   - `worker NAME may stay: opened AT`, AT the opening's time in UTC
     to the second, `YYYY-MM-DDTHH:MM:SSZ`;
   - `worker NAME may not stay: wires: TEXT` or `worker NAME may not
     stay: limits: TEXT`, TEXT the opening's refusal's words;
   - with no opening kept: `worker NAME unknown: no opening kept since
     the engine's last start`, and it may not stay.

   A kept opening of a name the configuration no longer names is not
   judged.
5. **The verdict:** `workers may stay: yes` when step 3 passed and every
   worker named may stay, with none named too; else `workers may stay:
   no`.

- **One judgement, at every floor:** a breaking change raises its wire's
  floor to its own version (`../draft/redeploys.md`, 4.5), so a worker
  below it fails step 4's ranges with no rule of its own. A wire moved
  with its reverse keeps workers of the previous version within the
  range; a breaking move refuses them by name.
- **The window is not judged here:** the patches' times are the
  rehearsal's lines (`binaries.md`, section 2), and comparing the window
  with the workers' grace is the redeploy's referee's
  (`../draft/redeploys.md`, 7.5).

## 11. Testing

Focused and fuzzy alone; the drills run on the deployments after landing
(`../draft/redeploys.md`, 7.5).

- **The rule and the bridges:** skein's step tests and its story (skein's
  `data/codec/command.md`, section 9; `data/codec/bridges.md`, 7); each
  owner's golden tests, up and down; the gate's mutations over scratch
  copies: a wire's field added with its version unmoved, a kind's
  variant moved or appended (jig's `protocol/channel/wires.md`, 7).
- **Each crate's `wire` module:** `FAMILY` and `VERSION` equal the
  header; `FLOOR` the generated floor; `layout` equals a fingerprint the
  test computes from the text with its names changed.
- **The opening and translation** at jig's components (jig's
  `protocol/channel/bridges.md`, 9); temper's roots against its
  encoders (6); `Identity` total from the floor.
- **A story across versions,** in the one-process world
  (`../testing/README.md`, 2.5), with the worker made to speak its wires'
  floors alone and the engine its whole ranges: a goal through a
  landing, a close job, a chat resumed, a run adopted across an engine
  restart and sent a message, a call's answer and a renewed git
  identity after it, and a host tool called by that run; and an
  assignment that cannot go down to that worker, waiting with no try
  spent. But for `smith-charter`, which the worker speaks from the
  lowest version that carries the current host tools' contract: a
  charter of contract 2 or later cannot go down to version 4, whose
  reverse holds contract 1 alone (7), so its assignment is for a worker
  that reads version 5 (`../draft/redeploys.md`, 4.3). The run calls
  the current contract; an older contract's adapters are held by `temper-engine-smith`'s
  focused tests over a contract of two versions made in the test. The world's seam is each side's wires: the worker's link and
  the engine's component take the `Spoken` lists and bridges the world
  gives, so a test caps the worker at the floors with no other change.
- **The host tools' contract and the clients' families:** their gates'
  mutations over a scratch copy: a declaration's input changed with
  the version unmoved, a client family's field added with its version
  unmoved, each refused; with the version moved, passing; a floor
  passed without a breaking file, refused; an older version's changed
  tool without its adapter, refused.
- **`versions`:** its lines for the table, whole, in the binaries' tests.
- **The openings kept** (10.1): the engine's workers component handing up
  a statement as the worker stated it; the store keeping one row per
  worker, replaced by its next opening and cleared by a start's wires,
  and the rehearsal answering them; the root committing an opening, and
  one heard before its start's wires after them.
- **The verdict** (10.2), over kept openings and wires made by hand:
  every worker opened as this binary speaks, yes; one with another
  layout, one with no version in common, one whose output differs, each
  named with the opening's words; one with no opening, unknown; a fresh
  patch; `temper-host-tools` kept at another version; the copy keeping
  no wires. And in the one-process world, a rehearsal of a deployment
  whose worker opened says it may stay, by name.

## 12. Open questions

- **The host tools' meaning.** Default: by its author's hand, as above;
  a description's wording judged by its reviewer.
- **An adapter's answer.** An adapter maps an old call to a current one,
  whose answer goes back as the current tool renders it. Default: an
  answer whose meaning changed for an old caller is a breaking move.
