# The store's loads: the shapes their changes share

While the goal that moves the core's loads onto the store's port runs
(`temper/docs/design/draft/jig-domain-api.execution.md`, J4), the shapes its
changes share and no other area reads (`temper/docs/design/draft/design-economy.md`,
4.3). They bind as `vocabulary.md`, 2.4 and `decisions.md`, 1.3 do. The goal's
last change deletes this document and its status.

## 1. The core's held load

- **`Held::Store(engine::store::Out)`,** in `jig-domain-core-vocabulary`'s `Held`:
  the one request by which the core asks for a load. The engine gives it to the
  root as `Out::Held(Peer::Store(out))`, at its place among the core's requests
  (`vocabulary.md`, 2.1). No root, kit or world names it; the core's own tests
  may.
- **`In::Store`** reaches the core through an entrance of the core's that the
  engine calls, one decision with the step's own wrapping (its routes drained,
  its stopping and settling gates), never through a variant of `Event`.

## 2. The load table

- **One table of the core's loads in flight,** keyed by each load's token
  (`vocabulary.md`, 2.4, *One token per load*), each entry saying what its load
  is for, as one variant of an enum of the core's: the first change's for a
  read, then one each for records and proposals, windows and notes, each added
  by its own change with its arm where an answer is dispatched.
- **Room:** one function of the core's, written by the first change and called
  by the others, saying whether a load of the core's own work other than a read
  may be sent: fewer than `Limits::load_slots` such loads in flight, a window's
  later pages counting in the load its first took (`decisions.md`, 1.3).
- **`Core`'s fields don't grow** (`temper/docs/design/draft/jig-domain-api.md`,
  16, count 10): the table lives in an existing field of `Core`, which may
  become the part that groups the core's loads, its reads among them, or in
  `Engine`.

## 3. What the family changes share

- **`engine::store`:** the first change writes `Out`, `In`, `Answer`, `Rows` and
  `Failure` whole, and `Range::Read` alone; each later change adds its own
  ranges (`Record` and `Proposal`; `Window`; `Notes`).
- **Each root's carriers,** a held output carrying `store::Out` to its store and
  an input carrying `store::In` back, are the first change's, named in its
  report; the others use them.
- **temper's read table** (`temper-engine-domain`'s `archive.rs`): each change
  takes out its own kinds of read; the one that, once it has merged the goal's
  branch, finds none left removes the table, its module if nothing else is in
  it, and that module's line in `temper/docs/design/domain/root.md`, 2.
