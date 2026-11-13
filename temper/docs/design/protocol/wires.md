# temper's wires

Provisional, 2026-11-12. The wires between temper's engine, its
workers, their agents and its clients: which they are, how each one's
version moves with its layout, what the engine and a worker state of
them at their opening, what a mismatch costs, and how `versions`
prints them. It carries `../draft/redeploys.md`, sections 3.1, 3.2,
3.3's interim, 3.4 and 8, for temper's parts. The mechanisms are
skein's (`data/codec/command.md`, the wire rule; `data/codec/README.md`,
section 9, fingerprints) and jig's (`protocol/channel/wires.md`, the
opening's statement and a mismatch's refusal); this document says what
temper's deployment puts through them.

## 1. In one page

- **Versions move with layouts, from now** (the user, 2026-11-12). Each
  wire between temper's parts is a family of a patch owner, marked a
  wire: a landing that changes its layout moves its version by one, keeps
  the old text and brings its forward patch, and the gate refuses one
  changed in place (3).
- **Each build's floor is its current version** (3.2): a build speaks one
  version of each wire and translates nothing, until bridges
  (`../draft/redeploys.md`, section 4).
- **The opening states them** (4): a worker states each wire's versions
  and layouts; the engine refuses one it does not speak alike, naming
  the wire, and the worker redials.
- **A mismatch at an assignment costs no try** (5): a worker that cannot
  read a charter, items or a transcript for their version refuses the
  claim by name, and the task claims again after a pause.
- **The host tools' contract is versioned whole** (6), its kept text
  beside its code; jobs have no wire of their own (7).
- **Clients' versions move with their layouts too** (8), with no kept
  text and no bridge; a page refused reloads once to the engine's
  bundle.
- **`versions` prints each wire's range and layouts** (9), and the store
  keeps the wires the engine's last start spoke and each worker's last
  opening since (9.1), by which `upgrade --check` says which workers may
  stay (9.2).

## 2. The wires

| Wire | Crate | Between | Stated at the opening |
|---|---|---|---|
| `jig-worker-wire` | `jig/crates/jig-protocol-worker-wire` | the engine and its workers: the channel's records, its kinds' table included (jig's `protocol/channel/wires.md`, 4) | by jig |
| `smith-channel` | `smith/crates/smith-channel` | a worker and its agents; the engine keeps or sees two of its pieces, a call's name and a delivery's evidence (`channel.md`, section 2) | by jig |
| `smith-charter` | `smith/crates/smith-charter` | the engine writes charters, which the worker and its agent read; the agent writes the result, which the engine reads | by jig |
| `smith-transcript` | `smith/crates/smith-transcript` | the agent writes turns; the engine keeps them and resumes a window on a worker | by jig |
| `temper-worker-wire` | `temper/crates/temper-protocol-worker-wire` | temper's payloads in jig's channel: the workspace items, the work, the preparation, a job's spec and a git identity's value (`channel.md`) | by temper, through jig |
| `temper-host-tools` | `temper/crates/temper-engine-smith` | the host tools the engine offers in a charter and answers when a run calls them (6) | no: the engine's alone |

- **Each wire's name is its family's,** as its schema's header gives it;
  the host tools' contract, which is no schema, is named
  `temper-host-tools`.
- **A wire's version is its current schema's header,** read through its
  crate's `wire::VERSION` (3.1).
- **The pieces of smith's channel the engine keeps** are versioned as
  `smith-channel` itself: a change of the channel moves the version the
  engine checks, though the engine reads only those pieces. A family of
  their own comes with their bridges (`../draft/redeploys.md`, 4.4).
- **Clients' wires** are section 8's.

## 3. Versions move with layouts

### 3.1 The rule

- **Each wire family is a patch owner's,** marked a wire in its crate's
  `schema/wires` (skein's `data/codec/command.md`, section 1): its kept
  texts are frozen from its first, so a landing that changes its layout
  in anything but names, comments, `boxed` and the text's layout moves
  its version by one, keeps the old text and brings its forward patch,
  written by `cargo run -p skein-codegen -- patch <crate>` as a stored
  family's is (skein's `data/codec/command.md`, 2). The old version's
  golden bytes are made from its kept text (skein's
  `data/codec/command.md`, 3).
- **The gate holds it:** `fabrig-boundary`'s `patches` test
  (`baseline.md`, 7) lists each wire crate as an owner, so its check
  refuses a wire changed in place (`Unpatched`, or `Version` with the
  verdict `Unmoved`), a kept text edited (`Changed`) and a kind's number
  or direction changed (jig's `protocol/channel/wires.md`, 4).
- **Each wire's baseline** is its crate's first `--baseline` run, at
  this design's layout: `jig-worker-wire` with its statement and its
  kinds' table (jig's `protocol/channel/wires.md`, 2 and 4) at the next
  version after the one the baseline found; `temper-worker-wire` with
  `Items` made `versioned` (5) at the next version; `smith-channel`,
  `smith-charter` and `smith-transcript` at their current texts,
  `smith-transcript` already an owner (`baseline.md`, 3) and now marked.
  Before it, each moves in place, as before.
- **Each wire crate says what it speaks,** in a module `wire`:

  ```rust
  pub mod wire {
      pub const FAMILY: &str;          // the family's name, as its header gives it
      pub const VERSION: u16;          // the current version, as its header gives it
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
  3.3): stored turns, calls' names and deliveries' evidence keep the
  versions they were written in. The reverse, the floor kept in the
  repository and the breaking declaration are the bridges'
  (`../draft/redeploys.md`, 4.1 and 4.5).

### 3.2 The floor

- **Each build's floor is its current version,** until bridges: it
  speaks one version of each wire, states it alone (4), and translates
  nothing.
- **So a wire moved is a redeploy of every part together**
  (`../draft/redeploys.md`, 7.3): a worker of another layout is refused
  at the opening by name (4), and `upgrade --check` says each worker
  that opened at the previous version may not stay, naming the wire
  (9.2).

## 4. At the opening

- **The statement is jig's** (jig's `protocol/channel/wires.md`, 2 and
  3): the worker's credential lists each wire it speaks, with its lowest
  version and each version's layout; the engine checks each wire it
  speaks against it, refuses with *wires* naming the first that fails,
  and keeps each wire's agreed version with the channel.
- **jig states its four:** `jig-worker-wire`, `smith-channel`,
  `smith-charter` and `smith-transcript`, from their crates' `wire`
  modules.
- **temper hands both components its own,** `temper-worker-wire`, in
  place of `PAYLOAD_VERSION`: the worker's link and the engine's workers
  component are each given temper's wires as `jig_protocol_worker_wire::Spoken`
  values (jig's `protocol/channel/wires.md`, 2) when they are made: the
  engine's from `temper_engine_service`'s table (9), the worker's from
  `temper_protocol_worker_wire::wire`, the same entry, since the
  worker's service links no engine crate. `temper-service`, which joins
  both, holds the two equal. Neither component knows what they cover.
- **Named in both logs,** in the shell's own lines:
  - the engine, at each worker's channel accepted: `worker NAME opened:
    jig-worker-wire V, smith-channel V, smith-charter V,
    smith-transcript V, temper-worker-wire V`, each wire's agreed
    version, in the statement's order;
  - the engine, at each opening it refuses: `worker NAME refused at its
    opening: REASON: TEXT`, or, refused by skein's machine before the
    credential is read, `a worker refused at its opening: version:
    TEXT` (skein's `net/channel.md`, 5.2);
  - the worker, at each refusal it hears: `the engine refused the
    opening: REASON: TEXT`, then its redial as after any refusal.

## 5. A mismatch costs no task

- **What a worker reads with a version:** a charter, by its first two
  bytes against `smith-charter`'s; each turn of a window resumed, by its
  own against `smith-transcript`'s; the workspace items, by their own
  against `temper-worker-wire`'s. `Items` is a `versioned` record from
  this design on (3.1), so its first two bytes are the family's version,
  as a charter's are.
- **Another version is refused, not failed:** the worker answers the
  assignment with jig's version refusal, naming the wire, the version
  the bytes carry and the range it reads (jig's
  `protocol/channel/wires.md`, 6). Bytes that carry the worker's version
  and still do not decode are invalid, as before.
- **The engine's log names it,** in place of `Invalid(Start(Other)):
  charter`: `task T run R attempt A refused: version mismatch: WIRE N,
  the worker reads L..H`. The task's retry notice says the same reason
  (jig's `protocol/channel/wires.md`, 6).

## 6. The host tools' contract

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
- **Held by two tests:** the crate's own, focused: the rendered text
  equals the current version's kept text; `SKEIN_CODEGEN_WRITE=1`, the
  tree's rewrite switch, writes it for review; a difference fails naming
  the fix, "the host tools' layout moved: move `wire::VERSION` to N+1
  and write its text with `SKEIN_CODEGEN_WRITE=1`". And `fabrig-boundary`'s: a kept text
  the base holds never changes, and the kept versions run from 1 to the
  greatest without a gap, the base read as the `patches` test reads it
  (`baseline.md`, 7).
- **Not stated at the opening:** a worker carries a call's name, tool and
  input as bytes and reads none of them. A run's charter naming its
  tools' version, and an engine answering every version from its floor,
  are the bridges' (`../draft/redeploys.md`, 4.2).

## 7. Jobs

- **A job kind's request and result are no wire between temper's
  builds.** The spec is `temper-worker-wire`'s `JobSpec`, under that
  wire's version; its request and result are the files the project's
  command reads and writes, which the worker carries as bytes and never
  reads (`../domain/worker/jobs.md`), versioned by their own `version`
  member between the engine and the project's tooling (`tooling.md`).
- **A kind's meaning to a worker** (its checkout, its command's verb, its
  delivery) is fixed by its number: a change of it is a new kind
  number, which a worker declares in its hello only once it runs it, so
  the engine places that job on no worker of an older build
  (`../domain/worker/jobs.md`).

## 8. Clients

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

## 9. `versions`, the stored wires and the workers' openings

- **One table,** `temper_engine_service::wires()`, in this order: jig's
  four (`jig_protocol_worker_wire::spoken()`), then `temper-worker-wire`,
  then `temper-host-tools`, each a `Spoken`. The engine's workers
  component takes its `temper-worker-wire` entry, and the worker's link
  states the same entry from its crate (4); the store and `versions`
  take the whole.
- **`versions` prints,** after the families' lines (`binaries.md`,
  section 2), for each wire in the table's order:

  ```
  wire NAME LOWEST HIGHEST
  layout NAME VERSION FINGERPRINT
  ```

  one `layout` line per version from LOWEST to HIGHEST, FINGERPRINT its
  64 lowercase hex digits. Each build's floor is its current (3.2), so
  LOWEST and HIGHEST are equal, and the floor is LOWEST. They replace
  the lines `wire NAME VERSION`.
- **The store keeps the wires** the last start spoke (jig's
  `protocol/store/records.md`, 3, `Wires`): one `{name, version}` per
  wire of the table, its version the highest.

### 9.1 The workers' openings kept

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
- **Never read by the engine:** they are for `upgrade --check` (9.2),
  which runs with the engine stopped (`../draft/redeploys.md`, 5.1).

### 9.2 Whether the workers may stay

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
   tools of the contract it started with (section 6). Else, for each,
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
  below it fails step 4's ranges with no rule of its own. While each
  build's floor is its current (3.2), a wire moved makes every worker of
  the previous version unable to stay, each named with its wire. Once
  floors fall below the current (`../draft/redeploys.md`, 3.3), the
  same function judges by the ranges.
- **The window is not judged here:** the patches' times are the
  rehearsal's lines (`binaries.md`, section 2), and comparing the window
  with the workers' grace is the redeploy's referee's
  (`../draft/redeploys.md`, 7.5).

## 10. Testing

Focused and fuzzy alone; the drills run on the deployments after landing
(`../draft/redeploys.md`, 7.5).

- **The rule:** skein's step tests and its story (skein's
  `data/codec/command.md`, section 9); the gate's mutations over scratch
  copies: a wire's field added with its version unmoved, a kind's
  variant moved or appended (jig's `protocol/channel/wires.md`, 7).
- **Each crate's `wire` module:** `FAMILY` and `VERSION` equal the
  header; `layout` equals a fingerprint the test computes from the text
  with its names changed.
- **The opening** (jig's `protocol/channel/wires.md`, 7): a worker
  stating every wire as the engine, accepted, its agreed versions
  logged; one naming a version with another layout, one with no version
  in common, and one missing a wire, each refused with *wires* naming
  it.
- **A mismatch:** in the one-process world (`../testing/README.md`, 2.5),
  a worker answering an assignment with a version refusal: the task
  claims again with no try spent, its notice naming the wire, and no run
  is placed on that channel while it stands.
- **The host tools' contract and the clients' families:** their gates'
  mutations over a scratch copy: a declaration's input changed with
  the version unmoved, a client family's field added with its version
  unmoved, each refused; with the version moved, passing.
- **`versions`:** its lines for the table, whole, in the binaries' tests.
- **The openings kept** (9.1): the engine's workers component handing up
  a statement as the worker stated it; the store keeping one row per
  worker, replaced by its next opening and cleared by a start's wires,
  and the rehearsal answering them; the root committing an opening, and
  one heard before its start's wires after them.
- **The verdict** (9.2), over kept openings and wires made by hand:
  every worker opened as this binary speaks, yes; one with another
  layout, one with no version in common, one whose output differs, each
  named with the opening's words; one with no opening, unknown; a fresh
  patch; `temper-host-tools` kept at another version; the copy keeping
  no wires. And in the one-process world, a rehearsal of a deployment
  whose worker opened says it may stay, by name.

## 11. Open questions

- **The credential across versions.** The worker encodes its credential
  in its current channel version, which the engine reads before it
  agrees one; once a build speaks more than one, the credential's
  layout must be one both read. Default: kept unchanged across the
  versions a floor spans, a change of it breaking.
- **The engine's statement to the worker.** The engine states nothing
  back: with each build's floor its current, an accepted channel means
  every wire at both sides' one version. With bridges, the worker must
  learn the agreed versions. Default: an engine record before the
  hello's answer.
- **The host tools' meaning.** Default: by its author's hand, as above;
  a description's wording judged by its reviewer.
