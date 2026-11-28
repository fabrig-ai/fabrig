# The wires a worker's opening states

Provisional, 2026-11-12, revised 2026-11-24 for layouts. What an engine
and a worker of two builds tell each other at their channel's opening:
the wires each speaks, by layout, with the worker's schema texts; the
engine's check of them before the hello; the kinds' table, kept in the
schema; terms; and what an assignment refused for a version costs. It
carries temper's `draft/archive/layouts-both-ways.md`, sections 3 and 5, and
`temper/docs/design/draft/archive/redeploys.md`, 3.4, for jig's channel. How the engine then
converts is `layouts.md`'s; an application names its own wires
(temper's `protocol/wires.md`).

## 1. In one page

- **Versions are frozen** (2): each worker wire's version is its
  schema's header and no longer moves; a layout is its family
  fingerprint, and the layout, not the version, says what a side
  speaks.
- **The worker states its wires** in its credential (2): for each, its
  name, its layout and its current text.
- **The engine checks each wire it speaks** (3), after the worker's name
  and secret: a wire missing, or one at another layout whose text does
  not parse, is not its layout, or gives a value type no plan, refuses
  the channel with *wires*, naming the wire, and the worker redials as
  after any refusal. A wire at the engine's layout is not parsed.
- **The engine accepts with the worker's kinds' table** (4), from the
  worker's `jig-worker-wire` text, so frames cross at the worker's
  layout and the engine converts them (`layouts.md`).
- **Terms are the worker's table's** (5), and the engine's list only the
  kinds its own layout reads.
- **A version refusal at an assignment** (6) stays a refusal: the task
  claims again after a pause, spending no try.

## 2. The statement

- **In the credential,** `jig-worker-wire`'s:

  ```
  record Wire { name: text 32 layout: fixed 32 text: bytes 65536 }
  record Credential { name: text 256 secret: secret 1024 wires: list 8 Wire output: u32 opening: Opening }
  ```

  A `Wire`'s `layout` is its text's family fingerprint (skein's
  `data/codec/fingerprints.md`, section 1), and `text` its current
  schema text, as its crate embeds it.
- **The credential is the opening's bootstrap:** every build lays out
  `Credential`, `Wire` and `Opening` alike, since the engine reads a
  worker's before it knows its layouts. A change to any of them is a
  change no plan carries, which restarts every part together at its
  redeploy (temper's `draft/archive/redeploys.md`, 7.3); a focused test holds
  their layout to a fingerprint written in the test.
- **Every text, every opening.** The worker cannot know before the
  engine answers which of its layouts differ, and the engine needs the
  worker's kinds' table before it accepts (4), so the credential carries
  each wire's text whatever the engine's layouts; it is a few tens of
  KB, sent once per connection.
- **What a side speaks,** as the components take it, is a `Spoken` per
  wire (`layouts.md`, 7): jig's four, `jig-worker-wire`, `smith-pieces`,
  `smith-charter` and `smith-transcript`, in that order, then the
  application's, each name once. The engine checks the same list from
  its own side.
- **A wire's version is frozen** at its header (temper's
  `draft/archive/layouts-both-ways.md`, 3): `jig-worker-wire` 22, `smith-pieces`
  1, `smith-charter` 5, `smith-transcript` 8, the application's at theirs
  (temper's `protocol/wires.md`, 3.1). A versioned record's head keeps
  that number, so no byte of one layout moves. A focused test in each
  wire's crate holds its header at its frozen number.
- **The channel's own version** is agreed by skein's opening (skein's
  `net/channel.md`, 5.2), each side's range `jig-worker-wire`'s one
  version.

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
  - its layout is the engine's: nothing more is checked, and nothing
    converts (`layouts.md`, 2.3);
  - `smith-pieces` at another layout: `wire smith-pieces: the worker's
    layout HEX is not the engine's HEX, and pieces cross as written`
    (`layouts.md`, 2.1);
  - its text does not parse: `wire NAME: the worker's text does not
    parse at line N`;
  - its text's header names another family: `wire NAME: the worker's
    text is of OTHER`;
  - its text is not its stated layout: `wire NAME: the worker's text is
    not its layout HEX`;
  - a value type of `layouts.md`, 2.1 with no plan in a direction it
    crosses: `wire NAME VALUE: no plan DIRECTION: CHANGE at
    DECLARATION.PART`, DIRECTION `from the worker` for Read and `to the
    worker` for Keep, CHANGE skein's `Why` in words (`no such root`, `no
    default`, `retyped`, `reshaped`, `variant dropped`, `payload changed`,
    `held on one side`, `pieces`, `too deep`), the part by the text skein
    names it in, and `.PART` left out where the change names a
    declaration alone;
  - a bound of the worker's `jig-worker-wire` text that the engine's
    limits do not give (4): `wire jig-worker-wire: the worker's text
    bounds DECLARATION by LIMIT, which the engine lacks`.

  HEX is a layout's first 8 bytes in lowercase hex. A worker that states
  one name twice is refused with *wires*, `wire NAME: stated twice`. A
  wire the worker states and the engine does not speak is passed over.
  Each text is cut to the refusal's 256 bytes.
- ***wires*** is jig's reason 257; the other reasons keep their numbers.
- **What the check keeps:** the plans it derived, with the channel's
  connection (`layouts.md`, 3), and the worker's kinds' table (4).
- **Refusals reach both owners' logs:** the engine's side hands its
  owner each refusal it sends at an opening, with its reason and text,
  and the worker's link hands its owner each refusal it hears, with its
  reason and text, before it redials.
- **The statement handed up whole** (temper's `draft/archive/redeploys.md`,
  5.1): with the channel accepted and the worker's hello decoded, the
  engine's side hands its owner what the worker stated, its wires as
  its credential gives them, in their order, its `io.output`, its terms
  as its channel's ready event gave them (5), and the job kinds its
  hello declares, in the hello's order (`../../domain/hosts/hub.md`, 9),
  so that the owner may keep the opening (`../store/records.md`, 3,
  *The workers' openings*), and which wires convert. A channel lost or
  refused before its hello decodes hands up none:

  ```rust
  // jig_protocol_engine_workers
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct Stated {                    // a wire as a worker states it
      pub name: Box<str>,
      pub layout: [u8; 32],
      pub text: Box<[u8]>,
  }
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct Statement {
      pub wires: List<Stated>,           // the credential's, in its order
      pub output: u32,                   // the credential's io.output
      pub terms: List<skein_channel::Term>, // the worker's, as its ready event gave them
      pub jobs: List<jig_domain_link::JobBound>, // its hello's job kinds, in their order
  }
  // Request::Opened and host::Output::Opened carry `statement: Statement`
  // beside `worker` and `converted`: the names of the wires the engine
  // speaks whose layout the worker's is not, in the engine's order.
  ```
- **The checks as one function,** which the opening runs and which a
  tool runs over a kept opening, with the engine stopped (temper's
  `protocol/wires.md`, 10.2): after the name and the secret, the wires
  (above), then the worker's `io.output` against the engine's room, then
  its terms against the largest body of each kind the engine sends at
  the worker's table (5):

  ```rust
  // jig_protocol_engine_workers
  pub enum Unfit {
      Wires(Box<[u8]>),                  // *wires*, with the text above
      Limits(Box<[u8]>),                 // skein's *limits*, with the opening's words for an output or a kind
  }
  pub fn judge(
      spoken: &List<Spoken>,             // what the engine speaks
      roots: &List<Root>,                // its application's roots (`layouts.md`, 7)
      room: u32,                         // the engine's io.output
      limits: &v22::Limits,              // the limits its kinds' bounds take
      statement: &Statement,
  ) -> Result<List<Box<str>>, Unfit>;    // the wires that convert, as `converted`
  ```

  The opening runs the same code at its moments: the wires and the
  output when the credential is read, the terms at the ready event, so
  its refusals and their order are as before. `judge` runs all three in
  that order over a whole statement, deriving the plans the opening
  would. Its texts are the opening's, word for word, so a kept opening
  judged by a new build says what that build's opening would refuse it
  with. It reads no `jobs`: the opening is judged before the hello, and
  a hello's kinds are the fleet's to judge (`../../domain/engine/fleet.md`,
  2, *A job goes only where a host runs it*).

## 4. The kinds' table in the schema

- **Two enumerations,** the schema's last declarations: `FromWorker`, the
  kinds the worker sends, the initiator's, and `FromEngine`, the kinds
  the engine sends, the responder's, each variant holding its kind's
  record.
- **A kind's number is its variant's place** at the worker's layout:
  `0x0100` plus its tag in the worker's `FromWorker`,
  `0x0180` plus its tag in the worker's `FromEngine`. Every frame on a
  channel is at the worker's layout (`layouts.md`, 4), so its numbers are
  the worker's, and the engine maps them by name.
- **The worker's table** is built by the engine from the worker's
  `jig-worker-wire` text, as each side builds its own from its own: each
  variant of `FromEngine` a kind the engine sends, each variant of
  `FromWorker` that the engine's `FromWorker` has by name a kind it
  receives, with the largest body of its record at the worker's layout,
  from the text's bounds within the engine's limits (skein's
  `data/codec/run-time.md`, 1.3, `bounds_within`). The engine accepts
  the channel with it (skein's `net/channel.md`, 5.2); a worker at the
  engine's layout is accepted with the engine's own table.
- **`KINDS` follows the schema:** a focused test reads the current
  schema text with `skein_codec::schema` and holds every rule of `KINDS`
  equal to its variant's number, direction and record, and every variant
  to a rule.
- **No frame is an enumeration's value on the wire:** the enumerations
  state the table; a body is its kind's record alone, and the
  conversion puts the tag before it (`layouts.md`, 2.1).

## 5. Terms

- **Each side states and checks terms from its channel's table**
  (`README.md`, 4.1): the worker from its own, the engine from the
  worker's (4), each the largest body of each kind it receives.
- **So the engine's terms list only what it reads:** a kind of the
  worker's that the engine's layout lacks is not in its table, and the
  worker sends it only where its design lets it go unsent: such a kind
  absent from the engine's terms is not sent (`layouts.md`, 4.2), and
  any other refuses the opening at the worker with skein's *limits*,
  naming it, as a terms gap.
- **A limit both sides must share** is stated beside the wires: the
  worker's `io.output` in the credential (`README.md`, 4.1).

## 6. A version mismatch at an assignment

- **The refusal,** among the channel's refusals:

  ```
  record Mismatch { wire: text 32 version: u16 lowest: u16 highest: u16 }
  enum Refusal { busy, invalid: InvalidValue, version: Mismatch }
  ```

  `version` is what the refused bytes carry. The worker writes the one
  version it reads of that wire, its own, as both `lowest` and
  `highest`, and the engine reads `highest` alone, as that version: the
  record keeps its layout, so a worker of any build is read alike.
- **Made by the worker:** the hand-out reading smith's payloads
  (`../../domain/hosts/hub.md`, 8) answers a charter whose first two
  bytes are another `smith-charter` version, or a resumed turn of
  another `smith-transcript` version, with this refusal in place of
  invalid; the application answers its own items so (temper's
  `protocol/wires.md`, section 5). The engine converts every one to the
  worker's own layout first (`layouts.md`, 4), so a refusal is a bug of
  the engine's conversion. Bytes of the worker's version that do not
  decode stay invalid.
- **Its cost on the engine:** no failure. The run ends refused, as a
  refusal before anything ran (`../../domain/tasks/lifecycle-holds.md`,
  section 1): the task claims again after a pause, with no bound and no
  try spent, its retry notice's reason `version mismatch: WIRE N, the
  worker reads V`. The window stays open.
- **The fleet places nothing more on that channel:** a channel that
  answered a version refusal, or whose engine could not send a running
  attempt what it holds (`layouts.md`, 5.2), is marked mismatched, and
  from then until it is lost the fleet places no run and no job on it,
  as on a draining one (`../../domain/engine/fleet.md`), while what it
  hosts goes on. The worker's next channel is checked again.
- **Told to the application** with the run's answer, as any refusal, so
  its log names the wire and both sides' versions.

## 7. Testing

- **The statement and the check,** at the engine's workers component: a
  worker stating every wire at the engine's layouts, accepted with the
  engine's own table, no text parsed; one at another layout of every
  wire, accepted with its own table, the wires that convert handed up;
  one missing a wire, stating a name twice, stating `smith-pieces` at
  another layout, sending a text that does not parse, one of another
  family, one that is not its layout, and one giving a value type no
  plan, each refused with *wires* and its text. At the worker's link: a
  credential carrying every text; a refusal's reason and text handed up,
  then a redial.
- **The credential's layout** held to its fingerprint (2), and each
  wire's version to its frozen number.
- **The kinds' table:** `KINDS` against the schema (4); the worker's
  table built from a derived text (`layouts.md`, 8): its numbers the
  derived text's, a kind the engine lacks left out of its receipts, its
  bounds within the engine's limits.
- **Terms:** the engine's from the worker's table; a worker's optional
  kind left unsent where the engine's terms lack it, and another kind
  absent refusing with *limits*.
- **`judge`** over a kept opening, word for word the opening's texts.
- **A mismatch:** the hand-out answering a charter and a turn of another
  version with the refusal, and one of its version that does not decode
  as invalid; the engine ending the run refused with no try spent and
  its notice's reason; the fleet placing nothing more on the channel,
  and placing again on the worker's next channel.

## 8. Open questions

- **Texts by need:** a worker could send its texts only for the wires
  whose layouts the engine lacks, at the cost of a second exchange
  before the engine accepts; every text in every credential is a few
  tens of KB once per connection, so it waits for a reason.
