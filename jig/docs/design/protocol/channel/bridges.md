# The channel across versions

Provisional, 2026-11-13. How an engine and a worker of two builds talk
once each speaks a range of each wire's versions: what each side states
and the engine states back, the terms at an older agreed version, what
the channel's two components translate and how, and what cannot go
down. It carries temper's `draft/redeploys.md`, sections 3.2's terms,
3.3, 4.1, 4.3 and 4.4's pieces, for jig's channel. The bridges
themselves are skein's (`data/codec/bridges.md`); the statement and the
check of each wire are `wires.md`'s; what an application puts through
them is its own (temper's `protocol/wires.md`).

## 1. In one page

- **Each side speaks each wire from its floor to its current** (2), the
  floor kept in the wire crate's `schema/wires`. The credential is laid
  out alike in every version from the channel's floor, so the engine
  reads it before agreeing one.
- **The engine states back the agreed versions** (3), in its first
  record, `wires`, so a newer worker knows what to send.
- **Terms are the agreed version's** (4), computed from its kept text
  with the side's current limits.
- **Each side sends from its current down to the agreed version, and
  receives from the agreed version up to its current** (5): the newer
  side translates both ways, the older none. jig's records by
  `jig-worker-wire`'s bridge, smith's charter and result by
  `smith-charter`'s, turns by `smith-transcript`'s, the application's
  parts by its own; smith's pieces cross as written.
- **What cannot go down** (6): an assignment or a job is refused for its
  version by the engine's own component and placed on another worker,
  spending no try; what goes to a running attempt must go down whole,
  which a gate holds; a newer worker whose sends cannot go down whole
  closes its channel after the statement, so the engine goes first.

## 2. What each side speaks

- **jig's four, from their crates' floors:** `jig-worker-wire`,
  `smith-pieces`, `smith-charter` and `smith-transcript`, each a
  `Spoken` whose `lowest` is its crate's generated
  `kept::<family>::FLOOR` and whose `layouts` are its kept texts'
  fingerprints from the floor (skein's `data/codec/bridges.md`, 5.1,
  `layouts`):

  ```rust
  // jig_protocol_worker_wire
  impl Spoken {
      pub fn of(name: &'static str, texts: &[&str], floor: u16) -> Result<Spoken, BridgeError>;
  }
  ```

  `spoken()` builds the four so; `Spoken::current` stays for a wire
  that speaks one version.
- **`smith-pieces` in place of `smith-channel`:** the pieces of smith's
  channel the engine keeps or sees, a call's name and a delivery's
  evidence, are a family of their own (smith's `protocol/README.md`,
  section 8), stated at the opening; `smith-channel` itself is per-build,
  between a worker and its agents, and jig's opening no longer states
  it.
- **The channel's own range** is `jig-worker-wire`'s, floor to current:
  skein's opening agrees it (skein's `net/channel.md`, 5.2), each side's
  schema listing every version from the floor with its kinds' table and
  bounds (4).
- **The credential is the same in every version from the floor:** the
  `Credential` and `Wire` records are laid out alike in each kept text
  from `jig-worker-wire`'s floor, so the engine decodes a worker's
  credential, written in the worker's current version, with its own
  current codec. A change of either is breaking (skein's
  `data/codec/bridges.md`, 3), and a focused test holds it.

## 3. The engine's statement

- **A record of the engine's, the first it sends:**

  ```
  record AgreedWire { name: text 32 version: u16 }
  record Agreed { wires: list 16 AgreedWire }
  ```

  `FromEngine` gains the variant `wires: Agreed` after its last, so its
  kind is `0x018d` in `jig-worker-wire` 12, the version that adds it
  (`wires.md`, 4). It lists each wire the worker stated and the engine
  speaks, with its agreed version, in the statement's order
  (`wires.md`, 3).
- **Sent once, first:** once the channel is accepted and its terms
  checked, before any other record, on a channel whose agreed version
  has the kind. Nothing the engine sends precedes it.
- **The worker keeps it** with its link, and translates by it (5). A
  second, or a statement naming a wire the worker did not state or a
  version outside its range, is a link fault: the worker closes the
  channel and redials.
- **A worker that states one version of every wire** knows the agreed
  versions without a statement: they are the ones it states, which the
  engine accepted. It translates to them, from its current, statement
  or none. A production worker states its whole ranges; a world caps a
  worker at chosen versions so (temper's `protocol/wires.md`, 11).
- **A channel version without the kind** (below 12): the engine predates
  the statement and speaks one version of each wire. A worker that
  stated a range of any wire cannot know the engine's versions: it
  closes the channel at once, logging `the engine states no wires:
  restart the engine first`, and redials after its pause, as after a
  refusal.

## 4. Terms at the agreed version

- **Each version's table from its kept text:** for every version from
  the floor, the schema skein's channel takes (`kinds::schema`) lists
  that version's kinds: each variant of its `FromWorker` and
  `FromEngine`, its number `0x0100` or `0x0180` plus its tag, its
  direction, and the largest body of its record, which for the current
  version is the generated `encoded_bound` with the side's limits, as
  today, and for an older one the kept text's
  `most_bytes_within(record, LIMIT_NAMES, limits.values())` (skein's
  `data/codec/README.md`, 7.3). So a field the older version lacks counts
  nothing, and a body that fits once translated fits its terms.
- **Stated and checked at the agreed version,** as today (`wires.md`, 5):
  skein's opening states the agreed version's received kinds, and
  `terms_gap` checks the peer's against the agreed version's sent kinds.
- **The credential's bound** is the current one's, the same in every
  version (2).

## 5. Translation

### 5.1 The rule

- **Send: current to agreed. Receive: agreed to current.** Each
  component translates every record it encodes from its current version
  of each wire to the channel's agreed version, and every record it
  decodes from the agreed version to its current, by skein's bridges
  (`data/codec/bridges.md`, 5). The side whose current equals the agreed
  version translates nothing: the bridge hands the bytes back as they
  are.
- **Nested before outer:** the bytes of another owner inside a jig record
  (smith's, the application's) are translated first, each by its
  wire's bridge and agreed version, and the jig record holding them then
  as a whole; on receipt, the jig record first, then what it holds.
- **The bridges are the component's:** each component makes, when it is
  made, the bridges of jig's wires from their crates' generated
  `kept::<family>::bridge` with capacities from its limits, and takes
  the application's (8). They translate bytes and never decode smith's
  or the application's records into types, so the components still link
  no smith type (`../../domain/hosts/hub.md`, 8).

### 5.2 jig's records

- **By the kind's enumeration:** a record of kind `K` is translated as
  the value of `FromEngine` (or `FromWorker`) holding it: its tag, one
  byte, then the record's bytes, the encoding of that enumeration's
  value (skein's `data/codec/README.md`, section 3). Down, the bridge's
  result's first byte is the kind's tag in the agreed version, so its
  number is `0x0180` (or `0x0100`) plus it, and the rest is the body; up,
  the frame's kind gives the tag at the agreed version, the bridge gives
  the current one. So a kind moved, renamed or given a changed record
  translates by the same patches as its record, and a kind the agreed
  version lacks is refused (`Unheld`), never sent under another number.
- **The hello and what precedes the statement** translate by the
  channel's agreed version, which skein's ready event names, and need
  no statement.

### 5.3 smith's payloads

| Bytes | In | Its bridge | Down (send) | Up (receive) |
|---|---|---|---|---|
| a charter | `Assignment.charter` | `smith-charter`, root `Charter` | to the agreed version | to the current |
| a result | `Ended`'s outcome in `Answer` | `smith-charter`, root `RunResult` | to the agreed version | to the current |
| a turn | `Turn.body` | `smith-transcript`, root `Turn` | to the agreed version | from its own version to the current |
| a resumed window's turns | `Assignment.transcript`, each turn | `smith-transcript`, root `Turn` | as written when its version is at most the worker's agreed one; else down to it | from its own version to the current |
| a call's name | `Call.name`, `Relayed.name`, each answered call's name | `smith-pieces` | as written | as written |
| a delivery's evidence | `DeliveryAnswer.evidence`, each answered call's reply | `smith-pieces` | as written | as written |

- **A turn carries its own version,** its first two bytes, since the
  engine keeps turns as their agents wrote them (smith's
  `protocol/transcript.md`): one already in a version the reader reads,
  at or below its current, crosses as written and is read forward
  there. Nothing goes up only to come down again.
- **smith's pieces cross as written:** `smith-pieces`' floor is its
  current in every build, every move of it breaking (smith's
  `protocol/README.md`, section 8), so both sides of an accepted channel
  speak one version of it. A focused test holds its floor at its
  current.

### 5.4 The application's parts

- **By roots the application names** (8): each part a jig record carries
  for the application has, when the application names one, a wire and a
  declaration of that wire, and is translated by that wire's bridge and
  agreed version as smith's are; a part with none crosses as is.

  | `Carried` | Its jig field |
  |---|---|
  | `Items` | `Workspace.items`, in an assignment or a job |
  | `Left` | `Work.left` |
  | `Saved` | `Work.saved` |
  | `Preparation` | `Resource.resource` |
  | `JobSpec` | `Job.spec` |
  | `JobResult` | `JobFinished.result` |
  | `Grant` | `ApplicationValue.secret`, a grant's value, at an assignment, a job and each renewal |

- **A grant's value is translated at each send,** for its channel's
  agreed version, keeping the grant's number, generation and validity,
  which are jig's fields around it. Its bytes are a secret's: the bridge's
  buffers hold them only while it runs.

## 6. What cannot go down

### 6.1 An assignment or a job

- **The engine's component refuses it for its version** when a
  translation of the record or of anything it carries halts: it sends
  nothing, and answers the domain through its usual answer route, as an
  assignment that cannot encode is answered (`records.md`, 2), with a
  refusal of its own, *untranslated*, naming the wire, the version the
  engine sends (its current) and the range the worker stated: `version
  mismatch: WIRE N, the worker reads L..H`. Its diagnostic names the
  halt.
- **Its cost on the engine:** as a worker's version refusal
  (`wires.md`, 6): no failure; the task claims again after a pause, with
  no try spent, its retry notice saying the reason.
- **Placed elsewhere:** the fleet keeps the refused task off that channel
  until the channel is lost, while the channel takes every other run
  and job (`../../domain/engine/fleet.md`). With no other worker, the
  task waits, its notice saying why; the next worker back, during a
  rolling redeploy, takes it. Whether an assignment goes down depends on
  its values, not its versions alone (an appended option at none goes
  down), so trying it is the judge, once per task and channel.

### 6.2 To a running attempt

- **Everything sent to a run already on a worker** must go down whole:
  `message`, `relayed`, `turn_kept`, `turn_busy`, `acknowledge`,
  `cancel` and `grant`, with the host's own `sources`, `check`, `sign_out`
  and `read_list`, which cannot wait either. A focused test in
  `jig-protocol-worker-wire` holds, for every version from the floor to
  the one before the current, that each of these kinds has a variant
  there and that the bridge's reverse is total for its record (skein's
  `data/codec/bridges.md`, 5.3). A change that breaks it is breaking,
  and raises the floor.
- **One that halts anyway** is a bridge's bug: the component discards it,
  logs `cannot send KIND to worker NAME at jig-worker-wire N: HALT`, and
  marks the channel mismatched, so nothing more is placed there
  (`wires.md`, 6).
- **The application's** grant values are held so by its own test
  (temper's `protocol/wires.md`, 6).

### 6.3 A newer worker

- **Checked by the worker,** at the statement, against the ranges it
  stated: for each wire whose agreed version is below its current, the
  bridge's reverse must be total, down to the agreed version, for
  everything the worker sends on it: each `FromWorker` kind's record, with its variant present there;
  smith's `RunResult` and `Turn`; and the application's parts it sends
  (`Left`, `Saved`, `Preparation`, `JobResult`, `Items` of a job's
  answer where it has one).
- **One that is not** closes the channel at once, logging `the engine
  speaks WIRE N, below what this worker sends whole: restart the engine
  first`, naming each such wire, and redials after its pause. So the
  engine goes first (temper's `draft/redeploys.md`, 7.2).
- **A worker capped at single versions** (3) has stated no range and is
  not checked. A send its link cannot take down is handled as a record
  that cannot encode on its side: a call answered `call refused` to its
  run (`README.md`, section 5), any other record counted, logged with
  the halt, and discarded.

## 7. The worker's hand-out

- **It reads its current version:** the worker's link hands the hub a
  charter and a resumed window translated up to its current (5.3), so
  the hand-out (`../../domain/hosts/hub.md`, 8) decodes its one version
  as before.
- **A version refusal stays for bytes it cannot read:** a charter or a
  turn whose version is above the worker's current, which an engine of
  bridges never sends, still answers with the version refusal of
  `wires.md`, 6, naming the range the worker reads: its first kept
  version to its current.

## 8. The application's interface

```rust
// jig_protocol_worker_wire
pub enum Carried { Items, Left, Saved, Preparation, JobSpec, JobResult, Grant }
pub struct Root { pub carried: Carried, pub wire: &'static str, pub declaration: &'static str }
pub struct Application {
    pub spoken: List<Spoken>,  // its wires, stated at the opening, as before
    pub bridges: List<Bridge>, // one for each wire a root names
    pub roots: List<Root>,     // what each carried part holds; a part with none crosses as is
}
```

- **Both components take it** where they took the application's
  `List<Spoken>` (`wires.md`, 2). A root naming a wire with no bridge,
  or a declaration its current text lacks, refuses the component's
  construction (`Unusable`).
- **The engine keeps each channel's agreed versions** with its
  connection, as `wires.md`, 3 has it, and the worker's link the
  statement's.

## 9. Testing

Focused and fuzzy alone, at the components and the wire crate; a story
across versions is the application's (temper's `protocol/wires.md`, 11).

- **The wire crate:** each version's kinds' table from its kept text, its
  bounds by `most_bytes_within`; the credential laid out alike from the
  floor; the running attempt's kinds total from the floor (6.2);
  `smith-pieces`' floor its current; `Spoken::of` for jig's four.
- **The opening,** at both components, with a worker whose bridges speak
  the floor and the engine's the current: accepted, the statement sent
  first and kept; a worker stating one version of every wire on a
  channel version without the statement, accepted and translating
  nothing; one stating a range there, closing; a newer worker whose
  sends are not total down to the engine's versions, closing and naming
  them.
- **Translation:** each kind of record, both ways, at an older agreed
  version: what one side encodes the other decodes as its own current
  value; a charter, a result, a turn and a window's turns, and a part
  of each `Carried` with a root; a turn kept in a version the worker
  reads crossing as written.
- **What cannot go down:** an assignment whose charter holds what the
  agreed version cannot, refused *untranslated* with no frame sent, its
  task claiming again with no try spent and placed on another channel,
  then waiting with none; a message to a running attempt that halts,
  discarded and the channel marked mismatched.
- **Fuzzy:** drawn records of every kind, at a drawn agreed version from
  the floor, sent and received through both components' translations:
  each arrives as itself or is refused, never altered.

## 10. Open questions

- **Placing by agreed versions before trying.** The fleet learns a task
  cannot go to a channel from its first try (6.1). Default: try, since a
  value decides; a placement that asks the bridges first comes if tries
  prove costly.
