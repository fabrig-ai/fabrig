# The channel across layouts

Provisional, 2026-11-24. How an engine and a worker of two builds talk
when their layouts of a worker wire differ: the engine converts what it
receives from the worker's layout to its own and what it sends from its
own to the worker's, by plans derived from both sides' schema texts
(skein's `data/codec/layouts.md`), and the worker speaks its own layouts
alone. It carries temper's `draft/archive/layouts-both-ways.md`, sections 5, 6
and 7.3, for jig's channel. The opening that gives the engine the
worker's texts, and checks them, is `wires.md`'s; what an application
puts through the channel is its own (temper's `protocol/wires.md`).

## 1. In one page

- **The engine converts, the worker does not** (2). Every frame and
  every part of another owner's that crosses a channel is at the
  worker's layout on the wire. The worker, and the agent its build pins
  (temper's `draft/archive/redeploys.md`, 4.4), know nothing of other builds.
- **A wire at the engine's layout converts nothing** (2.3): one build's
  parts meet only this case, and no text is parsed for it.
- **Otherwise a plan for each value type, in each direction it
  crosses** (2.1, 2.2): Read from the worker's layout to the engine's,
  Keep from the engine's to the worker's, each skein's `Plan::between`.
- **Derived at the opening, kept by the worker's text** (3), so a fleet
  on one build derives once; one that does not derive refuses the
  opening by name (`wires.md`, 3).
- **Nested before outer** (4): a part of smith's or the application's
  is converted as its own value type, inside the jig record that
  carries it; the record is converted as its kind's enumeration.
- **What the worker's layout cannot keep is refused to its domain**
  before anything is written to the channel (5): an assignment or a job
  waits for another worker, spending no try; a message to a running
  attempt is discarded and its channel takes nothing more.
- **Written converters** (6) are their owner's, for what no plan
  derives: the same function a store's written job calls (skein's
  `data/codec/layouts.md`, 4). Where a value type's plan would run and
  a converter takes it, the engine holds the value, asks its owner
  through the service, and goes on with the answer.

## 2. What converts

### 2.1 The value types

Each wire names the value types its messages carry, as a store's family
does (`../store/layouts.md`, 2). A plan is derived for each value type
in each direction it crosses, and no other:

| Wire | Value type | Worker to engine, Read | Engine to worker, Keep |
|---|---|---|---|
| `jig-worker-wire` | `FromWorker`, the declaration whole | every frame the worker sends | — |
| `jig-worker-wire` | `FromEngine`, the declaration whole | — | every frame the engine sends |
| `smith-charter` | `Charter` | — | `Assignment.charter` |
| `smith-charter` | `RunResult` | an ended run's outcome, `Ended.outcome` in an `Answer` | — |
| `smith-transcript` | `Turn` | `Turn.body`, `Part.body` | each turn of `Assignment.transcript`, each body of `Assignment.parts` |
| the application's | each root's declaration (7) | `Left` (`Work.left`), `Saved` (`Work.saved`), `Preparation` (`Resource.resource`), `Guides` (`Guides.bytes`), `JobResult` (`JobFinished.result`), `Ask` (`Ask.bytes`) | `Items` (`Workspace.items`, in an assignment or a job), `JobSpec` (`Job.spec`), `Grant` (`ApplicationValue.secret`, at an assignment, a job, a `ReadList` and each renewal), `Answer` (`Asked.answer`'s bytes) |

- **A frame is its kind's enumeration's value:** its tag, one byte, then
  its body, the encoding of that value (skein's `data/codec/README.md`,
  section 3), converted by the plan of `FromWorker` or `FromEngine`
  planned whole, as a broad root without a variant (skein's
  `data/codec/layouts.md`, 1). Where the layouts differ, the plan maps
  kinds by name, including kinds moved or renamed (`was`). A kind one
  side lacks is a variant only one side has: refused per value, never
  sent under another number.
- **No held texts:** jig's records carry the others' parts as `bytes`,
  which jig's plans copy as they are, held to their bounds; each part is
  converted by its own plan (4).
- **Not converted:**
  - the credential, laid out alike in every build (`wires.md`, 2);
  - `smith-pieces`, a call's name and a delivery's evidence
    (`Call.name`, `Relayed.name`, `AnsweredCall.name`,
    `DeliveryAnswer.evidence`): the engine keeps them as bytes that
    name no layout (`../store/records.md`, the core's `CallRecord`), so
    they cross as written, and the opening requires the worker's
    layout of `smith-pieces` to be the engine's (`wires.md`, 3);
  - a part with no root (7), which crosses as is.

### 2.2 The modes

- **Read, up:** what the engine receives is used as its own; what the
  worker's layout has and the engine's lacks is skipped, and what the
  engine's has alone takes its default, or the default from the worker's
  tombstone (skein's `data/codec/layouts.md`, 2).
- **Keep, down:** what the engine sends must mean the same at the
  worker, which acts on it for the engine. What the engine's layout has
  and the worker's lacks is dropped only at the default the way back
  writes (from the engine's field or the worker's tombstone), or where
  the engine's text marks it `droppable`; anything else refuses the value (5).
- **The modes are temper's draft's** (`temper/docs/design/draft/archive/layouts-both-ways.md`,
  4.4): the engine receives through Read plans and sends through Keep
  plans, whichever of the two builds is newer.

### 2.3 The same layout

A wire whose stated layout, its family fingerprint (`wires.md`, 2), is
the engine's own converts nothing: its bytes cross as they are, no plan
is made, and the worker's text is not parsed. Swapping the names of
kinds of one shape leaves the layout unchanged (skein's
`data/codec/fingerprints.md`, 1), so nothing maps their tags: an old tag
now selects the other kind. Every part of one build, the in-process
worker of `temper-box serve` among them, meets this case alone. A wire
at another layout whose value types' plans all keep (skein's
`data/codec/layouts.md`, 2) converts by their heads alone.

## 3. At the opening

- **For each wire whose layout differs,** in the engine's order
  (`wires.md`, 3), after the worker's name and secret:
  1. the worker's text is parsed by skein's step parser, its size within
     the credential's bound;
  2. its header names the wire's family, and its family fingerprint is
     the layout the worker stated;
  3. a plan is derived for each value type of 2.1, in each direction it
     crosses, from the worker's text to the engine's for Read and from
     the engine's to the worker's for Keep; where none derives, a
     written entry (6) naming the value type and the worker's layout of
     it makes that direction's pass a written one.

  The first that fails refuses the opening with *wires*, its text as
  `wires.md`, 3 gives it. A value type with no plan, and no written
  entry (6), names the wire, the value type, the direction and the
  change, so the operator knows which worker to drain and restart on the
  new build (temper's `draft/archive/redeploys.md`, 6.1).
- **The worker's kinds' table** comes from its `jig-worker-wire` text
  (`wires.md`, 4), and the engine accepts the channel with it.
- **Kept by the worker's text:** the engine's workers component keeps,
  for each wire, the plans it derived from each worker text it met, the
  text compared byte for byte, at most `MAX_TEXTS`, 8, texts a wire, the
  one met longest ago dropped first. A plan depends on its two texts, not
  their fingerprints alone (skein's `data/codec/layouts.md`, 1), so a
  fleet on one build derives once, and a worker of another text derives
  its own. The engine parses its own texts when the service builds.
- **Each channel holds its plans** and one `Workspace` fitted to all of
  them (skein's `data/codec/layouts.md`, 3 and 4), so that no pass
  allocates; a workspace that cannot be fitted refuses the opening as a
  plan that does not derive, its change `too deep`.

## 4. Converting

### 4.1 What the engine sends

- **In order:** the record is encoded at the engine's layout, each part
  it carries (2.1) first converted down by its Keep plan, so its jig
  field holds the part at the worker's layout; then the record, as its
  `FromEngine` value, by that plan. The result's first byte is the
  kind's tag at the worker's layout, the frame's kind `0x0180` plus it,
  and the rest is the body.
- **A resumed window:** each turn of `Assignment.transcript` is
  converted on its own, in its length; one that refuses refuses the
  assignment (5.1).
- **A resumed run's parts** (`records.md`, 5): the engine drops the kept
  parts the worker's layout refuses and sends the rest; where the
  worker's layout has no `parts` field, an assignment with kept parts is
  refused.
- **Within bounds:** each conversion writes into a buffer bounded by the
  field that holds its result, or for a frame by the worker's largest
  body of that kind (`wires.md`, 4); a result past it refuses the value,
  as one the worker's layout cannot keep.
- **A kind the worker's layout lacks** is one the engine does not send.
  The kinds the design sends only where a channel has them (`restart.md`,
  `pause.md`, `figures.md`, `records.md`, 5's `part kept` and `part
  busy`, the usage reads of `records.md`) are sent where the worker's
  layout has a `FromEngine` variant of that name, and the engine's side
  asks its channel's plans, not a version.

### 4.2 What the engine receives

- **In order:** the frame's kind, less `0x0100`, is the tag at the
  worker's layout; the tag and the body, as a `FromWorker` value, are
  converted up by that plan, giving the engine's tag and body; the body
  is decoded by the engine's own codec, then each part it carries is
  converted up by its Read plan before the component hands anything to
  its owner.
- **Within bounds:** the engine's own largest body of the kind, and the
  field's bound for a part.
- **A kind the engine's layout lacks** is not among the engine's terms
  (`wires.md`, 5), so the worker does not send it: a kind the worker
  sends only where the engine takes it (`pause.md`'s `paused`,
  `records.md`, 5's `part`, the usage notices, `records.md`, 1's
  `guides`) is sent where the
  engine's terms list it, and a worker whose other kinds the engine's
  terms lack refuses the opening as a terms gap.

### 4.3 The worker

- **Its own layouts alone:** the worker's link encodes and decodes at
  its build's layouts, states its texts at the opening (`wires.md`, 2),
  and translates nothing. Its hand-out reads charters, windows and
  items at its own versions (`../../domain/hosts/hub.md`, 8).
- **What the engine takes** is its terms: the worker reads nothing of
  the engine's layouts but that.

## 5. Refusals

A plan's refusal names its part by the plan's source text, as skein's
`Plan::refusal` gives it: the engine's for what it sends and the worker's
for what it receives. Its kind is skein's `Refused`: `variant`, `field`,
`narrowed` or `absent`. A result past its bound is `too large`.

### 5.1 An assignment or a job

- **Refused for that worker** when a conversion of the record or of a
  part it carries refuses: the engine's component sends nothing and
  answers its domain through the answer route an assignment that cannot
  encode takes (`records.md`, 2), with *untranslated*, its text `the
  worker's layout cannot keep WIRE DECLARATION.PART: WHY`, which the
  task's retry notice gives.
- **Its cost:** no failure; the task claims again after a pause, with
  no try spent (`../../domain/tasks/lifecycle-holds.md`, 1), and the
  fleet keeps that task off that channel until the channel is lost, while
  the channel takes every other run and job
  (`../../domain/engine/fleet.md`). With no other worker, the task waits,
  its notice saying why. Whether an assignment goes down depends on its
  values, not on its layouts alone (an option appended at none goes
  down), so trying it is the judge, once per task and channel.

### 5.2 To a running attempt

- **Everything sent to a run already on a worker:** `message`,
  `relayed`, `turn_kept`, `turn_busy`, `part_kept`, `part_busy`,
  `acknowledge`, `cancel`, `grant` and `asked`, with the host's own
  `sources`, `check`, `sign_out` and `read_list`, which cannot wait. One
  whose conversion refuses is discarded, the engine logs `cannot send
  KIND to worker NAME: TEXT`, TEXT as 5.1's, and the channel is marked
  mismatched, so the fleet places nothing more there while what it
  hosts goes on (`wires.md`, 6).
- **A redeploy's order costs refusals, never correctness**
  (`temper/docs/design/draft/archive/layouts-both-ways.md`, 6.1): no gate holds these kinds total
  across layouts. A change that a running attempt's messages cannot
  carry to the previous build's workers is met by draining them
  (temper's `draft/archive/redeploys.md`, 6.1).

### 5.3 What the engine receives

- **A value its Read plan refuses,** or one past the engine's bound once
  converted, is a body of its kind that does not decode, handled as the
  component handles one, its diagnostic `cannot read KIND from worker
  NAME: WIRE DECLARATION.PART: WHY`. A part's likewise, as a part that
  does not decode.

### 5.4 What a worker's layouts cannot keep, listed

- **A gap** is a part of what the engine sends whose Keep plan, from the
  engine's text to the worker's, may refuse a value: each refusal of
  each Keep plan of 2.1's table (skein's `Plan::refusal`, by its index),
  for `FromEngine` and for each part the engine sends, in 2.1's order.
  A variant of `FromEngine` the engine sends only where the worker's
  layout has it (4.1, *A kind the worker's layout lacks*) is no gap. A
  wire at the engine's layout has none (2.3).
- **For the tool that rehearses an upgrade,** with the engine stopped
  (temper's `protocol/wires.md`, 10.2): a worker whose kept texts give
  gaps may connect, but some of what the engine sends it would be
  refused for it (5.1, 5.2). One function lists them, deriving the plans
  as `judge` does (`wires.md`, 3), its words those of 5.1 for each:

  ```rust
  // jig_protocol_worker_wire::layouts, named again by jig_protocol_engine_workers
  pub enum Site { Frame, Charter, Turn, Carried(Carried) } // Frame: a FromEngine record; Turn: a window's turn or a kept part's body
  pub struct Gap {
      pub wire: Box<str>,             // the family whose Keep plan refuses
      pub site: Site,
      pub why: skein_codec::layout::Refused,
      pub declaration: Box<str>,      // by the engine's text's names, as Plan::refusal gives them
      pub part: Box<str>,
  }
  impl Gap {
      /// `the worker's layout cannot keep WIRE DECLARATION.PART: WHY`, 5.1's words.
      pub fn words(&self) -> Box<[u8]>;
  }
  // jig_protocol_engine_workers
  pub fn gaps(
      spoken: &List<Spoken>, roots: &List<Root>, limits: &v22::Limits,
      wires: &List<Stated>,           // a statement's wires (`wires.md`, 3)
  ) -> Result<List<Gap>, Unfit>;      // Unfit: as `judge` refuses the same wires
  ```

## 6. Written converters

- **The rule:** a change of a wire's value type that no plan derives (a
  retype, a reshape) is converted by its owner's written converter,
  skein's `data/codec/layouts.md`, 4: one function for the value type,
  from one layout to another given both texts, the same one its store's
  written job calls for a stored value of that type (`../store/jobs.md`,
  4). It runs where the value type's plan would, either way, and one way
  may refuse. It goes once dev.fabrig.ai and every part it runs are past
  the layout that needed it (temper's `draft/archive/layouts-both-ways.md`,
  7.4). The common changes need none: a field added with a default, a
  field dropped with its tombstone (`dropped name: type = default`), a
  variant added, a bound widened (skein's `data/codec/layouts.md`, 2).
- **Declared by its wire's owner,** as data the opening reads (7): a
  written entry names the wire, the value type's declaration and the
  worker's layout of it (skein's `data/codec/fingerprints.md`, 1.4),
  and says that its owner's converter takes that value type between
  that layout and the engine's own, both ways. jig declares its wires'
  entries, smith's families' among them, and the application its own.
  None is declared today.
- **At the opening** (3): a direction whose plan does not derive, for a
  value type a written entry names at the worker's layout, is a written
  pass. It parses as any differing text does, is kept with the plans
  by the worker's text, and lists no gap (5.4), since which values it
  refuses is its owner's.
- **Where it runs, the value waits for its owner.** Step code calls no
  function an owner hands in, so the engine holds the value and asks:
  - the component holds the value where its pass would run (4.1, 4.2),
    with its wire, its direction's mode, both sides' parsed texts and
    roots (the worker's from its text, the engine's own) and the bound
    its result is held to (4.1, 4.2), and says a conversion waits;
  - the service lends the waiting value from the component, calls the
    converter of the wire's owner (jig's for jig's four wires,
    `jig_protocol_worker_wire::convert`, which hands smith's families to
    smith's crates once smith has a converter; the application's for its
    own), and answers the
    component with the bytes or the refusal, before it routes anything
    else of the component's;
  - one value waits at a time, the oldest first; while a channel's
    value waits, that channel reads nothing more, and its later sends
    wait behind it, in order, at most the channel's output frames
    (skein's `net/channel.md`, 10), one past them unsent as a full
    output's is;
  - with the answer, the conversion goes on as after a plan's pass: the
    next value of the frame, then the frame. A refusal is a plan's
    refusal (5), named by the value type, the converter's declaration
    and part, and its why; a channel gone while its value waits takes
    the answer as nothing.
- **Why held, not called:** the store hands an owner its written job's
  pages through the shell for the same reason (`../store/jobs.md`, 3);
  a converter is needed only between two builds whose layouts no plan
  joins, so a held value costs only a worker of another build, and only
  while the change is within the floor.

## 7. The application's interface

```rust
// jig_protocol_worker_wire
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spoken {
    pub name: &'static str,   // the family's name, its text's header
    pub version: u16,         // its header's version, frozen
    pub layout: [u8; 32],     // its text's family fingerprint (skein's data/codec/fingerprints.md, 1)
    pub text: &'static str,   // its current schema text, as its crate embeds it
}
pub enum Carried { Items, Left, Saved, Preparation, Guides, JobSpec, JobResult, Grant, Ask, Answer }
pub struct Root { pub carried: Carried, pub wire: &'static str, pub declaration: &'static str }
pub struct Written {          // a written entry (6)
    pub wire: &'static str,   // its wire's family, as its `Spoken` names it
    pub declaration: &'static str, // the value type's root: a `Root`'s declaration, or `FromWorker` or `FromEngine`
    pub layout: [u8; 32],     // the worker's layout of that value type (skein's data/codec/fingerprints.md, 1.4)
}
pub struct Application {
    pub spoken: List<Spoken>, // its wires, stated at the opening after jig's
    pub roots: List<Root>,    // what each carried part holds; a part with none crosses as is
    pub written: List<Written>, // the layouts its converter takes, of its own wires' value types
}
impl Application {
    /// An application whose parts cross unconverted: no root, no written entry.
    pub fn opaque(spoken: List<Spoken>) -> Application;
}
pub fn written() -> List<Written>; // jig's own entries, for its four wires: none today
pub fn convert(mode: Mode, from: &Side<'_>, to: &Side<'_>, value: &[u8], out: &mut Buffer)
    -> Result<(), Refusal>;   // jig's written converters (skein's data/codec/layouts.md, 4): none today

// jig_protocol_engine_workers::host
pub enum Output { /* ... */ Convert } // a value waits for its written converter (6)
pub struct Converting<'a> {
    pub wire: &'a str,        // the owner the service asks: jig's for jig's four, else the application's
    pub mode: Mode,
    pub from: Side<'a>,       // the worker's layout for Read, the engine's for Keep
    pub to: Side<'a>,
    pub value: &'a [u8],
    pub bound: u32,           // the most bytes the result may hold (4.1, 4.2)
}
impl Workers {
    pub fn converting(&self) -> Option<Converting<'_>>;   // the value that waits, lent
    pub fn converted(&mut self, now: Time, answer: Result<Box<[u8]>, Refusal>); // its owner's answer
}
```

- **A `Spoken` is its crate's current text:** `jig_protocol_worker_wire::spoken()`
  gives jig's four, `jig-worker-wire`, `smith-pieces`, `smith-charter`
  and `smith-transcript`, each from its crate's embedded text, their
  layouts computed from the texts when the service builds (temper's
  `testing/checks.md`, 1). An application gives its own the same way.
- **Both components take the application** where they did, and check
  its roots when they are made: a root naming a wire with no `Spoken`,
  or a declaration that wire's text lacks, refuses the component's
  construction (`Unusable`). After this check, the worker's link states
  `spoken` at the opening and uses no roots for conversion.
- **A part's direction is its `Carried`'s** (2.1); the engine derives
  that one direction's plan for it.
- **A written entry is checked as a root is:** one naming a wire with no
  `Spoken`, a declaration its text lacks, or a value type named twice
  at one layout refuses the component's construction (`Unusable`).
- **The service routes `Convert`** as the store's `Event::Job` is
  routed (`../store/jobs.md`, 3): by `Converting.wire`, to
  `jig_protocol_worker_wire::convert` for jig's four wires and to the
  application's own function for its, then `converted` with what it
  wrote, before it takes the component's next output.

## 8. Testing

Focused and fuzzy alone; a story through temper's parts is temper's
(`protocol/wires.md`, 11).

- **Another layout of each wire that converts,** in the tests: each such
  wire's current text, and a text derived from it by edits of skein's
  `data/codec/layouts.md`, 2 that plan both ways: a field with a default
  the derived text lacks, a field it alone has with a default, a kind
  each side alone has. The worker's side of a test at the derived layout
  is the current codec behind a shim converting each frame and part it
  sends by the Keep plan from its current text to the derived one, and
  each it receives by the Read plan back. An altered `smith-pieces`
  layout is refused at the opening (2.1).
- **At the wire crate:** every kind of `FromEngine` and `FromWorker`, and
  each part of 2.1, converted both ways between the current and the
  derived texts arrives as itself; each refusal of 5 with its text: a
  kind the worker lacks, a field it lacks holding a value, a narrowed
  bound, a result past its bound; each opening refusal of `wires.md`, 3
  with its text; a wire at the same layout converting nothing and
  parsing nothing; a second worker of the same text deriving nothing; a
  workspace fitted once allocating nothing per pass.
- **At the components and in the worker-link world:** an engine and a
  worker at derived layouts exchange every kind, both ways, through both
  components and the channel; an assignment the worker's layout cannot
  keep refused *untranslated* with no frame sent; a message to a running
  attempt that cannot be kept discarded and reported as a mismatch; a
  worker's optional kind not sent where the engine's terms lack it. In
  separate domain tests, an injected *untranslated* refusal checks that
  the task claims again with no try spent and is placed on another
  channel; an injected mismatch checks that the fleet marks the channel
  and puts nothing more there while its running attempt can finish.
- **Fuzzy:** drawn records of every kind, at the derived layout, through
  both components: each arrives as itself or is refused, never altered.
- **Written converters** (6), with one converter written for the tests,
  in `jig-test-layouts`, over a fixture family whose two texts retype a
  field (a number at one, its decimal text at the other): the same
  function the store's written-job test calls (`../store/jobs.md`, 6).
  At the wire crate: a written entry makes the pass where no plan
  derives, and the opening without it still refuses naming the change.
  In the worker-link world, a test application carrying that value type
  as its part, the worker at the older text: what the engine sends and
  receives crosses converted both ways, each value held and answered
  through the world's service; a value the converter refuses is refused
  as a plan's refusal is (5.1, 5.3); a channel's later sends wait behind
  a held one, in order; another channel goes on meanwhile.
- **The gaps** (5.4), at the wire crate: none at the same layout; none
  for a derived text that only adds; one for a field the derived text
  lacks, a kind the engine always sends that it lacks, and a narrowed
  bound, each with its site and 5.1's words; none for a kind the engine
  sends only where the worker has it.

## 9. Open questions

- **A plan's cost on a hot path** (`temper/docs/design/draft/archive/layouts-both-ways.md`, 10): a
  pass is one walk over the bytes with no allocation (skein's
  `data/codec/layouts.md`, 3), and the same layout costs nothing; it is
  not yet measured on dev.fabrig.ai.
- **`smith-pieces` converted** once the store keeps the layout of the
  pieces it holds.
- **Clients converted the same way** (`temper/docs/design/draft/archive/layouts-both-ways.md`, 10).
- **Names at the opening:** should it compare names as well as layouts,
  so a swap of names between kinds of one shape is visible (2.3)?
