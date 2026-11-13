# The wires a worker's opening states

Provisional, 2026-11-12. What an engine and a worker of two builds
agree at their channel's opening beside its version: the wires each
speaks, by version and layout, checked before the hello; the channel's
kinds' table, kept in its schema so that its version moves with it; and
what an assignment refused for a version costs. It carries temper's
`draft/redeploys.md`, 3.2 and 3.4, for jig's channel. The rule that
moves a wire's version is skein's (`data/codec/command.md`, 2, the wire
rule); an application names its own wires (temper's
`protocol/wires.md`).

## 1. In one page

- **The worker states its wires** in its credential (2): for each, its
  name, its lowest version and each version's layout fingerprint.
- **The engine checks each wire it speaks** (3), after the worker's name
  and secret: a wire missing, no version in common, or a version both
  name with different layouts refuses the channel with *wires*, naming
  the wire, and the worker redials as after any refusal.
- **Agreed per wire:** the highest version both speak, kept with the
  channel. Each build's floor is its current version until bridges, so
  today each wire's agreed version is both sides' one.
- **The kinds' table is the schema's** (4): two enumerations, one per
  direction, a kind's number its variant's place, so a kind's number,
  direction or record changed moves the channel's version.
- **Terms are the agreed version's** (5).
- **A version mismatch at an assignment is a refusal** (6): the task
  claims again after a pause, spending no try, its notice naming the
  wire, and the fleet places nothing more on that channel.

## 2. The statement

- **In the credential,** `jig-worker-wire`'s, in place of the
  application's payload version:

  ```
  record Wire { name: text 32 lowest: u16 layouts: list 16 fixed 32 }
  record Credential { name: text 256 secret: secret 1024 wires: list 16 Wire output: u32 }
  ```

  A `Wire` speaks the versions `lowest` to `lowest` plus its layouts'
  count less one, `layouts` each one's family fingerprint (skein's
  `data/codec/README.md`, section 9), lowest first.
- **What a side speaks,** as the components take it:

  ```rust
  // jig_protocol_worker_wire
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct Spoken {
      pub name: &'static str,      // the wire's family name
      pub lowest: u16,             // its floor
      pub layouts: List<[u8; 32]>, // each version's layout from lowest, at least one
  }
  impl Spoken {
      pub fn current(name: &'static str, version: u16, layout: [u8; 32]) -> Spoken; // floor = current
      pub fn highest(&self) -> u16;
      pub fn layout(&self, version: u16) -> Option<[u8; 32]>;
  }
  /// jig's own wires, in this order: jig-worker-wire, smith-channel,
  /// smith-charter, smith-transcript, each from its crate's `wire` module.
  pub fn spoken() -> List<Spoken>;
  ```

  Each build's floor is its current version (temper's
  `protocol/wires.md`, 3.2), so every `Spoken` is made by `current`
  until bridges.
- **jig states its four and the application's:** the worker's link and
  the engine's workers component are each given the application's wires,
  a `List<Spoken>`, when they are made, where they took its payload
  version; the credential lists jig's four, then the application's, each
  name once. The engine checks the same list from its own side.
- **The channel's own version** is still agreed by skein's opening
  (skein's `net/channel.md`, 5.2), each side's range from its schema;
  its layout is checked as any wire's, by the `jig-worker-wire` entry.

## 3. The engine's check

- **In order,** before its domain hears anything (`README.md`, 4.1):
  1. the name and the secret, or *unauthorized*;
  2. the wires, or *wires*;
  3. the worker's `io.output`, or skein's *limits*;
  4. room for one more channel, or *busy*.
- **The wires,** for each the engine speaks, in its own order, the
  first that fails refusing the channel:
  - the worker states none of that name: `wire NAME: the worker states
    none`;
  - no version in common: `wire NAME: the engine speaks L..H, the worker
    L..H`;
  - a version both speak with different layouts:
    `wire NAME version N: the engine's layout HEX, the worker's HEX`,
    each HEX the layout's first 8 bytes in lowercase hex.

  A worker that states one name twice, or a range past `u16`, is refused
  with *wires*, `wire NAME: stated twice` or `wire NAME: no such
  range`. A wire the worker states and the engine does not speak is
  passed over. Each text fits the refusal's 256 bytes, a name being at
  most 32.
- ***wires*** is jig's reason 257, in place of *application*; the other
  reasons keep their numbers.
- **Agreed per wire:** the highest version both speak. The engine's
  side keeps each wire's agreed version with the channel's connection,
  and hands them, by name in the statement's order, to its owner with
  the channel accepted, for its log (temper's `protocol/wires.md`,
  section 4).
- **Refusals reach both owners' logs:** the engine's side hands its
  owner each refusal it sends at an opening, with its reason and text,
  and the worker's link hands its owner each refusal it hears, with its
  reason and text, before it redials. skein's own refusal of a range
  with no version in common names both ranges (skein's
  `net/channel.md`, 5.2).
- **The statement handed up whole** (temper's `draft/redeploys.md`,
  5.1): with the channel accepted, beside the agreed versions, the
  engine's side hands its owner what the worker stated: its wires as
  its credential gives them, in their order, its `io.output`, and its
  terms as its channel's ready event gave them (5), so that the owner
  may keep the opening (`../store/records.md`, 3, *The workers'
  openings*):

  ```rust
  // jig_protocol_engine_workers
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct Stated {                    // a wire as a worker states it
      pub name: Box<str>,
      pub lowest: u16,
      pub layouts: List<[u8; 32]>,       // each version's layout from `lowest`
  }
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct Statement {
      pub wires: List<Stated>,           // the credential's, in its order
      pub output: u32,                   // the credential's io.output
      pub terms: List<skein_channel::Term>, // the worker's, as its ready event gave them
  }
  // Request::Opened and host::Output::Opened carry `statement: Statement`
  // beside `worker` and `wires` (the agreed versions).
  ```
- **The checks as one function,** which the opening runs and which a
  tool runs over a kept opening, with the engine stopped (temper's
  `protocol/wires.md`, 9.2): after the name and the secret, the wires
  (above), then the worker's `io.output` against the engine's room, then
  its terms against the largest body of each kind the engine sends, as
  the agreed `jig-worker-wire` version encodes it (5):

  ```rust
  // jig_protocol_engine_workers
  pub enum Unfit {
      Wires(Box<[u8]>),                  // *wires*, with the text above
      Limits(Box<[u8]>),                 // skein's *limits*, with the opening's words for an output or a kind
  }
  pub fn judge(
      spoken: &List<Spoken>,             // what the engine speaks, its table
      room: u32,                         // the engine's io.output
      schema: &skein_channel::Schema,    // its kinds' bounds, from its limits, as its component makes it
      statement: &Statement,
  ) -> Result<List<Agreed>, Unfit>;
  ```

  Its texts are the opening's, word for word, so a kept opening judged
  by a new build says what that build's opening would refuse it with.
  A floor below the current (temper's `draft/redeploys.md`, 3.3) is
  judged by the same function, through the ranges `Spoken` and the
  statement give.

## 4. The kinds' table in the schema

- **Two enumerations,** the schema's last declarations:
  - `FromWorker`, the kinds the worker sends, the initiator's: `hello`,
    `turn`, `call`, `answer`, `fact`, `unauthorized`, `limited`,
    `account_refused`, `logins`, `signed_out`, `list_read`, `draining`,
    `job_answer`;
  - `FromEngine`, the kinds the engine sends, the responder's:
    `assignment`, `message`, `relayed`, `turn_kept`, `turn_busy`,
    `acknowledge`, `cancel`, `grant`, `sources`, `check`, `sign_out`,
    `job`, `read_list`;

  each variant holding its kind's record (`message` holds `Inbound` and
  `grant` holds `Granted`, as today).
- **A kind's number is its variant's place:** `0x0100` plus its tag in
  `FromWorker`, `0x0180` plus its tag in `FromEngine`. So `job_answer`
  is `0x010c` and `read_list` `0x018c`, the numbers moving with the
  channel's version.
- **So the table moves the version:** a kind added, dropped, moved or
  given another record changes an enumeration, which the wire rule
  holds to a new version (skein's `data/codec/command.md`, 2).
- **`KINDS` follows the schema:** a focused test reads the current
  schema text with `skein_codec::schema` and holds every rule of `KINDS`
  equal to its variant's number, direction and record, and every variant
  to a rule.
- **No frame is an enumeration's value:** the enumerations state the
  table; a body is still its kind's record alone, as before.

## 5. Terms by the agreed version

- **Each side states and checks terms from the agreed version's table**
  (`README.md`, 4.1), the version the ready event names, not a constant:
  the largest body of each kind as that version encodes it. With one
  version spoken, the same as before.
- **A limit both sides must share** is stated beside the wires: the
  worker's `io.output` in the credential (`README.md`, 4.1).

## 6. A version mismatch at an assignment

- **The refusal,** appended to the channel's refusals:

  ```
  record Mismatch { wire: text 32 version: u16 lowest: u16 highest: u16 }
  enum Refusal { busy, invalid: InvalidValue, version: Mismatch }
  ```

  `version` is what the refused bytes carry; `lowest` and `highest` the
  range the worker reads of that wire. A change of the channel's layout,
  so it moves the channel's version, with its forward patch.
- **Made by the worker:** the hand-out reading smith's payloads
  (`../../domain/hosts/hub.md`, 8) answers a charter whose first two
  bytes are another `smith-charter` version, or a resumed turn of
  another `smith-transcript` version, with this refusal in place of
  invalid; the application answers its own items so (temper's
  `protocol/wires.md`, section 5). Bytes of the worker's version that do
  not decode stay invalid.
- **Its cost on the engine:** no failure. The run ends refused, as a
  refusal before anything ran (`../../domain/tasks/lifecycle.md`,
  section 6): the task claims again after a pause, with no bound and no
  try spent, its retry notice's reason `version mismatch: WIRE N, the
  worker reads L..H`. The window stays open.
- **The fleet places nothing more on that channel:** a channel that
  answered a version refusal is marked mismatched, and from then until
  it is lost the fleet places no run and no job on it, as on a draining
  one (`../../domain/engine/fleet.md`), while what it hosts goes on.
  The opening should have prevented it, so a mismatch is a bug of a
  bridge or of the gate, and the worker's next channel is checked again.
- **Told to the application** with the run's answer, as any refusal, so
  its log names the wire and both sides' versions.

## 7. Testing

- **The statement and the check,** at the engine's workers component: a
  worker stating every wire as the engine, accepted, its agreed versions
  handed up; one naming a version with another layout, one with no
  version in common, one missing a wire and one stating a name twice,
  each refused with *wires* and its text. At the worker's link: a
  refusal's reason and text handed up, then a redial.
- **The kinds' table:** `KINDS` against the schema (4); and the gate's
  mutations over a scratch copy of `jig-protocol-worker-wire`: a variant
  moved within `FromEngine`, one moved to `FromWorker`, one appended,
  each with the version unmoved, refused by the patch command's check.
- **Terms:** computed and checked from the agreed version.
- **A mismatch:** the hand-out answering a charter and a turn of another
  version with the refusal, and one of its version that does not decode
  as invalid; the engine ending the run refused with no try spent and
  its notice's reason; the fleet placing nothing more on the channel,
  and placing again on the worker's next channel.

## 8. Open questions

- **The engine's statement back.** The worker learns nothing of the
  engine's wires but its acceptance; enough while each wire's floor is
  its current. Default: an engine record of the agreed versions, sent
  with its terms, once bridges translate.
- **The credential across versions** (temper's `protocol/wires.md`,
  section 11).
