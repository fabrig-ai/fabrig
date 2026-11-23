# Layouts

How the store names each value's layout and reads it at the binary's,
so that an upgrade of derived changes rewrites nothing (temper's
`draft/store-shape.md`, sections 3 and 4). The map's values of the
registered families carry, at their head, an id into the store's
registry of layouts in place of their family's version; the open checks
that every layout the store holds converts to the binary's; each value
is converted as it is read, and written at the binary's layout.

- **Value types** (2): each registered family's owner says which root
  declaration, and which variant of a broad root, each range of its keys
  holds. A layout is a value type's, not a family's (skein's
  `data/codec/fingerprints.md`, 1.4).
- **The registry** (3): rows of the component's own, a layout per value
  type and id, with its fingerprint and the texts it was made with, and
  the texts themselves. It holds every layout a value of the map may be
  at.
- **Heads** (4): a registered value's first two bytes are its layout's
  id. The store writes the binary's id as a value enters skein-kv and
  converts a value at another layout as it leaves, so every decoder,
  jig's and the application's, reads the binary's layout with its
  family's version at the head, as it always has.
- **At open** (5): the registry read, a non-empty store without one
  refused; the binary's layouts registered; a plan made for
  every layout held, or the open refused before anything is written;
  layouts no value holds retired.
- **Written changes** (6) convert by their owner's written job at open
  (`jobs.md`, 4).

## 1. What is registered

- **The map's families:** jig's core, tasks, people, notes, accounts and
  settings families, and each family the application hands with value
  types (temper's `temper-store`), each listed in its owner crate's
  `schema/registered` (skein's `data/codec/command.md`, 1). From the
  registration on their versions stay, and no patch names them (skein's
  `data/codec/README.md`, section 5).
- **Outside the registry:** the component's own rows under its tag (the
  format and wires rows, transcripts' places and
  attempts, the workers' openings and the registry's own rows, 3), read
  before the registry or holding it; the secrets' store's families
  (`secrets.md`), read once at open from a store copied whole at each
  upgrade; and the transcripts' frames (`transcripts.md`, 6), already
  converted as they are read. These formats stay at their current versions
  (temper's `draft/store-floor.md`, section 5). A change to one designs
  how the deployment's store reaches it in the same landing; the frame
  patcher and its texts and patches stay (`transcripts.md`, 6).
- **The deployed store's layouts hold both rules.** Every layout the
  oldest store a build opens may hold, dev.fabrig.ai's (temper's
  `draft/store-floor.md`), plans to the binary's as 5.3 does or is a
  written job's (6); a format outside the registry keeps each
  declaration's layout at its version, or moves its version with its
  patch. Temper's guard holds both against the texts of the builds whose
  layouts that store holds (temper's `protocol/floor.md`, 6).
- **Held records** of any family the binary holds a text of, registered
  or not, are part of their holder's layout (skein's
  `data/codec/fingerprints.md`, 1.4): a core projection that holds a tasks
  `PhaseRecord` changes layout when `PhaseRecord` does, and is converted
  with it. A held record's head stays its family's version.
- **What lies in the map outside every registered range** (the
  installation's identifier at `01 04 03`, an application's rows no
  family of its names) is passed through as it is.

## 2. Value types

Each registered family's `KeptFamily` (`opening.md`, 3) names the text
its binary's generated code was made from and its value types:

```rust
pub struct KeptFamily {
    // ... as `opening.md`, 3
    pub current: &'static str,          // the binary's text of the family
    pub types: &'static [ValueType],    // its value types; empty for a family outside the registry
    pub retired: &'static [Retired],    // its value types no binary writes again
}
pub struct Retired {
    pub number: u16,                    // the retired value type's, never reused
    pub keys: &'static [KeyRange],      // the keys that held it, which hold nothing now
}
pub struct ValueType {
    pub number: u16,                    // the value type's, within its family: stable, never reused
    pub root: &'static str,             // its root declaration, as `current` spells it
    pub variant: Option<&'static str>,  // the leading enumeration's variant, for a broad root its keys tell apart
    pub keys: &'static [KeyRange],      // the keys that hold it
}
pub struct KeyRange {
    pub from: &'static [u8],            // inclusive; the application's relative to its tag
    pub to: &'static [u8],              // exclusive
}
```

- **One per kind of value its keys tell apart:** the tasks family's
  ended tasks (`01 01 06`) are one, its live tasks another; a family whose
  roots are records of their own (the settings') has one per root. A
  broad root whose keys do not tell its variants apart is one value type
  without a variant, the declaration whole (skein's
  `data/codec/fingerprints.md`, 1.4): the settings' `ApplicationState`, led by
  its `follows`, and the tasks' live and ended tasks, whose keys hold a
  task written before talk (`live`, `ended`) beside one written since
  (`live_talk`, `ended_talk`); a change to any variant of its root moves
  it. Its
  number names it in the registry's keys (3), so a declaration or a
  variant renamed (`was`, skein's `data/codec/README.md`, section 3)
  keeps its value type.
- **One root per range:** a range never holds two roots that only the
  values' bytes tell apart, since a layout names one root: the settings'
  charter versions (`01 00 08 03`) are one value type at
  `CharterVersion`, whose `overlay: option Overlay` holds what an
  overlaid version adds (`../../domain/engine/overlays.md`, 7).
- **Checked where the set is admitted,** at every open, beside
  `opening.md`, 3's checks, a failure refused `Set { owner }`: each
  number given once in its family; each root a declaration of `current`
  and each variant given only where that root is broad, naming one of
  its leading enumeration's variants (`Schema::root`); each range
  non-empty and the ranges of all the family's types and retired types
  ordered, disjoint, and together exactly the family's `keys`, so that
  each key the family holds resolves to one value type or one retired;
  a retired number not among the family's types; a family with types whose
  `current` does not parse, or that is not in the map (`KeptMap::Store`),
  or a `current` whose version is not the family's at the binary's
  level.
- **Resolved by key:** the store keeps the value types' ranges in one
  table ordered by `from`, the application's under its tag, and finds a
  key's by a binary search, as it finds a key's family for the census
  (`README.md`, 6.3).
- **Retired** (temper's `draft/store-floor.md`, section 2): a value
  type whose variant or root the family's text no longer declares may
  be declared retired with the ranges that held it, until every store
  at the floor has had its rows erased; none is declared now. The
  tasks family's 10 (its milestones) and `temper-store`'s 12 (its
  whole change records) are numbers never given again: the tasks family's `keys`
  leave out `01 01 0a`, and `temper-store`'s its relative `0c`, so a key
  there is no family's and passes through as any key outside the
  registered ranges (1). A retired type's ranges hold nothing: every open reads them at its check
  (5.3) and refuses a store where they hold a value, and a put to one is
  a writer's fault, the commit failing (`Failed`) as for a head that is
  not its family's version (4). Its registry rows are erased as 5.4
  says. The census keeps naming its ranges, each counting none.

## 3. The registry

A family of the component's, `jig-store-registry`, beside `openings`
in `jig-protocol-store`, at version 1, outside the registry itself (1):

```
family jig-store-registry 1

# A layout of a value type, by its id within the value type.
versioned record Layout {
    root: text 256                  # its root declaration, as its text spells it
    variant: option text 256        # its variant, for a broad root
    layout: fixed 32                # its fingerprint (skein's data/codec/fingerprints.md, 1.4)
    text: fixed 32                  # its family's text, by digest: a Text row's key
    held: list 16 fixed 32          # the texts of the families it holds, by digest
}

# A schema text a layout names.
versioned record Text { text: bytes 262144 }

# A job done (jobs.md, 2): the commit that ended it.
versioned record Job { ended: u64 }
```

| Key | Row |
|---|---|
| `00 06 <family> <number> <id>` | a `Layout`: the family's name as skein-kv's escaped bytes, the value type's number and the layout's id, each a big-endian `u16` |
| `00 07 <digest>` | a `Text`, by the SHA-256 (lib's) of its bytes, 32 bytes |
| `00 08 <owner> <name>` | a `Job` done (`jobs.md`, 2): the owner's byte in `Owner`'s order, the job's name as escaped bytes |

- **Every layout a value may be at** has its row, and every text a row
  names has its own: a value is never written at a layout the registry
  lacks, since the open registers the binary's before anything of the
  root's is written (5.2), and a layout's row goes only when no value
  holds it (5.4). A text goes with the last layout naming it.
- **Ids** are `u16`s within a value type. An empty store's first commit
  takes the family's version (5.2); a layout registered later takes one more than the
  greatest id its value type's rows hold. An id comes back only once its
  row is retired, when no value holds it (5.4).
- **Read at every open** by its current text, as the component's rows
  are, all of it: as many rows as layouts and texts, a few hundred at
  most.

## 4. Heads

- **What a head says:** in the map, a registered value's first two
  bytes are its layout's id within its value type, big-endian. The
  value's generated code reads and writes its family's version there, as
  ever; the store maps one to the other at its edges, so no decoder or
  encoder changes.
- **Writing:** every put to the map passes `commits::submit`; a put to
  a registered value type's key has its head replaced by the binary's id
  of that value type (5.2). Its head must be its family's version, as
  the generated code writes it: one that is not is a writer's fault, and
  the commit fails (`Failed`). An erase is as it was. `Write::Copy`
  reads its source through the read edge and puts it through the write
  edge, so a copy between two value types is at each one's layout.
- **Reading:** every value of the map passes one of three edges before
  anything decodes it: a page's rows (`pages::reader::load_rows`), a
  point get (`open::get_row`), and the restore's ended joins
  (`restore::reader::ended`). For a registered value type's key:
  - a head that is the binary's id has the family's version written in
    its place, the bytes otherwise as they are;
  - a head with a layout whose plan keeps (skein's `data/codec/layouts.md`,
    2) the same, its plan never run;
  - a head with another planned layout is converted by its plan into
    the binary's layout, its head the family's version (skein's
    `data/codec/layouts.md`, 3); a value the plan refuses
    (`Misconverted`) is handed on as it was stored, for its decoder to
    refuse as it would any malformed value;
  - a head the registry lacks for its value type, or a value too short
    for a head, is handed on as it is, likewise.
- **The deployment's header** is read through its registry plan before
  the registry's writes, then decoded by the binary's codec
  (`open::decode_header`), so the open can check its commit number.
- **What the census counts** is the stored head (`README.md`, 6.3): per
  keyspace, values per layout id.
- **Until the registry is active** (its rows committed, 5.2), neither
  edge maps anything. An empty store's first commit registers its
  layouts with ids equal to the families' versions.
- **The rehearsal** decodes every record of a registered family as a
  generic value of its layout's text, from its range's value type's root
  at that layout, where its head names a layout of the registry
  (`opening.md`, 5). A store holding values without a registry is
  refused before decoding (5.1).

## 5. At open

### 5.1 A store with no registry

Every store a build opens holds its registry: an empty store's first
commit writes it (5.2). A store that holds values and no `Layout`
row is refused before anything is written, `OlderBuild` with why
`Unregistered` (`opening.md`, 1.8): its values' heads are versions that
no row names, and no open seeds those values from texts in its code. A
value type that a store's registry has no row for, its root or variant
new since the registry's rows, is registered by 5.2, at its family's
version as its id, as for any value type with no row: no value of the
store can be at a layout of it, since no binary before it wrote a key
of its ranges.
The refusal follows the floor's check (`records.md`, 3). A store whose
binary's sets register nothing keeps no `Layout` row and is not refused
for it. Only an empty store's first commit creates a registry (5.2);
a later open registers new binary layouts in it.

### 5.2 The binary's layouts

For each value type, its layout in `current`, with the binary's texts of
the families it holds (each family's `current`): its row is the registry's of that
fingerprint, whose id is the binary's id; with none, a new row at the
next id (3) and its texts' rows, committed in the registry's commit (5.5). An
empty store's first commit (`README.md`, section 5) writes the binary's
layouts at their families' versions as ids, so its heads are its
families' versions and the edges map nothing until a later binary moves
a layout.

### 5.3 The check

Before anything is written, every open (whether it writes or not)
plans each layout the registry holds to its value type's binary layout
(skein's `data/codec/layouts.md`): from its row's text and held texts
and its root and variant, to `current`'s. A layout planned is kept in
memory for the read edge. A layout with no plan is refused, unless:

- a written job of the binary converts from it (`jobs.md`, 4): one
  naming it by fingerprint (`One`), pending or done, or, where its plan
  is `Underived`, one converting its value type's unplanned layouts
  (`Unplanned`), pending; or
- no value holds it: the census counts none at its id in each keyspace
  its value type's ranges reach, with no count past the census's heads
  there (`README.md`, 6.3), where it is retired (5.4). A value type a
  task log holds (`logs.md`, 2) has every layout held, since the census
  counts the map alone (`logs.md`, 8).

A layout row of a retired value type (2) is no layout to plan: the
check reads the type's ranges, and where they hold nothing the row is
retired (5.4); where they hold a value, the open is refused
`OpenRefusal::Layout` with why `Retired`, the id the first value's head
names and the values unknown. The ranges are read so at every open,
with or without a row, so a store holding a retired type's value never
opens.

The refusal, `OpenRefusal::Layout`, names the family, the value type
(its number, root and variant), the id, why (`Underived`'s, a text its
row names that is missing or does not parse, or a value type the binary
does not declare), and the values the census counts at that id, or that
it cannot tell (`opening.md`, 1.8). A layout row of a value type the
binary no longer declares is refused the same way while a value may
hold it, `Undeclared`, whether its family is one the binary declares or
not: the refusal names the family as the row's key spells it.

### 5.4 Retirement

A layout that is not its value type's binary layout and that no value
holds (5.3's census exception), or a retired value type's whose ranges
hold nothing (5.3), has its row erased by the next open that writes
(`opening.md`, 2), with each text no other row names. No open writes
for retirement alone. A log-held value type's layouts never retire by
the census (`logs.md`, 8).

### 5.5 When the open writes

The open writes before serving, in a new generation made by links
(`opening.md`, 2), when a binary layout is not registered or when a job
is pending (`jobs.md`, 2); then one commit (or as many as the store's
commit edge needs, `records.md`, 4) writes the binary's new layouts and
the retirements, before any job runs. An open
with none of these writes nothing, so a binary that changes no layout
and brings no job opens the store in place, as before (temper's
`draft/store-shape.md`, section 4, step 3: an older binary can read all
such a binary writes).

## 6. Written changes

A change no plan derives (a field computed from others, a variant
dropped, a type changed in meaning) is its owner's **written job**
(`jobs.md`, 4): a job that names the value type and the layout it
converts from, or each of its layouts no plan reaches, walks that value
type's keys, and is handed each value at such a layout as it is stored,
with that layout's texts, to put it again at the binary's. Its owner's function reads the old value by skein's
generic values (skein's `data/codec/run-time.md`, section 2) and builds
the new by its generated code. The check takes its layouts as converted
(5.3), and the next writing open retires them.

A written job also takes a layout that a plan reaches when the values
at that layout are not all the root it names. The read edge leaves
values at a written job's layouts to the job, never to a plan
(`jobs.md`, 4).

## 7. Bounds

- **In memory, at open:** the registry's rows and texts, and what the
  open's planning (5.1 to 5.4) reads, from the check of the binary's
  texts (2, and the third item below) to the end of the plans (5.3).
  Each is made once in that planning and kept to its
  end, so the registration and every row's check read the
  same one, whichever family or row needs it:
  - **A text parsed** where the planning first reads it, or taken from
    the shell's prepared schemas where one's whole text equals it
    (`opening.md`, 3): the binary's
    `current` of each family where the open checks it, kept as that
    check parsed or took it; a text the registry already
    holds where a row or a family holding it first needs it,
    its bytes checked against its digest before it is parsed or taken (5.3). No
    text is parsed twice in one planning, and a prepared one not at all.
  - **A layout computed,** the fingerprint of a root and variant in a
    text with its held texts (skein's `data/codec/fingerprints.md`, 1.4), once
    per those four: the registration's (5.2) is the
    one the check compares a row's persisted fingerprint against when
    the row names the same text, root, variant and held texts, in the
    same order; a row naming others has its own computed. Only a
    computed layout is kept: one that fails refuses where it does.
  - **A text's held texts,** the families its records hold and theirs
    (1), found once per text, from the binary or from the registry.

  These are dropped when the plans are made, and with the open's
  unfinished registry when it refuses: none outlives the planning that
  made it. An upgrading open plans again at its reopen after the copy
  (`opening.md`, 2) and parses and computes again there, over the copy's
  registry. The parsed texts cost at most every
  text the registry holds and the binary brings, parsed once: the texts
  the open already holds whole. Then one plan derived at a time (skein's
  `data/codec/layouts.md`, 5). While serving:
  per value type its key ranges, version, binary id, the ids whose plan
  keeps, and a plan per held layout that converts. Rows, texts and parsed
  schemas are dropped when the open ends, after its registry writes and
  written jobs; the drained pending queue keeps no spare capacity.
  Bounded beside the jobs' page and commit room at open
  (`README.md`, section 8); no map patch machine is held.
- **Per read:** a converted value is built in a `Buffer` that grows with
  the values the store converts, never past the store's value bound
  (`records.md`, 4), with one skein `layout::Workspace` beside it fitted
  at open to every plan the store holds. An open whose plans convert
  holds no buffer yet. The first read that converts makes one at the
  least power of two at least twice its stored value's length, at least
  4,096 bytes and at most the bound; a conversion that runs out of the
  buffer's room makes one twice as large, at most the bound, and
  converts again. So what the store holds is the room the largest value
  converted so far needed, not the value bound, which an application may
  size for far more than it holds, and a read allocates only when its
  value outgrows the buffer. A value that doesn't fit at the bound is
  left as it is read, as before. A value at the binary's layout, or at
  one whose plan keeps, is rewritten in its two head bytes alone.
- **The registry's rows** within skein-kv's value bound: a `Text` at
  most 262,144 bytes, refused `Set { owner }` where a `current` is longer.

## 8. Tests

- **Step tests,** in `jig-protocol-store`, over the test owners' sets
  (`opening.md`, 3), with registry texts and changed binary texts from
  `jig-test-layouts`: a non-empty store with no
  registry refused as 5.1 says, before anything is written; a field added
  with a default, read at the old layout and converted, written at the
  binary's id and read back; a broad root's variant unmoved while a
  sibling's record changes; a held record's change converting its
  holder; the census counting by id; each refusal of 5.3 and its words
  (`opening.md`, 1.8); retirement; an empty store's first commit.
- **Retired value types,** step tests: a set whose family declares one
  is admitted only with the ranges rule of 2; a store whose retired
  range holds a value is refused `Layout` with why `Retired` before
  anything is written; one whose registry holds a retired type's rows
  and whose range is empty has them erased by the open that writes; a
  put into a retired range fails its commit.
- **A new value type,** a step test: a store whose registry holds a
  fixture family's value types, opened by a binary whose `current`
  adds a value type in a new key range, opens with no refusal; the
  registration adds its row at the family's version, and a value put
  at its key reads back.
- **The open's parses and layouts** (7), step tests: through the
  registration and every row's check, each text the planning reads
  is parsed once, however many families hold it or rows name it;
  each binary layout is computed once, not again for a row naming
  the same inputs, while a row naming another text has its own; and
  no parsed text is held once the plans are made. The refusals and
  their order are unchanged: a stored text whose bytes do not give its
  digest, a persisted fingerprint its text does not give, a malformed
  declaration, a held depth past a plan's, each refused as before.
  Given prepared schemas, the planning parses no text they hold and
  each other text once.
- **Prepared and fresh opens** (`opening.md`, 3), step tests: over
  copies of one store, an open given prepared schemas and one given none
  answer the same, the same `Opened` or refusal and the same registry
  writes, and read the same values back: over a store at the binary's
  layouts; over a historical store whose registry holds older texts,
  its values converted; with the binary's text of a family altered from
  the prepared one; with a holder whose held family's layout changed;
  and with a stored text whose bytes do not give its digest, refused as
  before though a prepared schema holds the text it should be.
- **Dev's build's store,** step tests: a store whose registry holds the
  jig texts of `13efa77461`, the build dev.fabrig.ai runs, kept verbatim
  as fixtures, with values at those layouts: refused delegations
  (core's calls, whose `TaskBytes` hold the tasks' `TaskProblem`), ended
  tasks whose history and ended records are in their task logs, in more
  than one of skein-kv's log shards (`logs.md`, 1), and an activity frame
  near the store's page. Opened by the binary's sets, the open registers
  the binary's layouts in a new generation (5.5); the restore's live and
  census pages, each task's own rows and its ended record read back with
  every value at the binary's layout. The source generation is
  unchanged, and a second open writes nothing and reads the same.
- **The conversion buffer** (7), measured under skein's counting
  allocator in the store's world: a store whose plans convert holds no
  buffer at open; its first converted read makes one at its value's
  room; a value past the buffer's room grows it and is converted whole;
  the heap a seeded store holds after its open is the same whether its
  value bound is the fixture's or several times it.

## 9. Open questions

- **The secrets' store and the component's rows** stay outside the registry
  (1). Registering them would need the registry before the format row
  is read, and the secrets' store's own registry; their rows are few and
  rarely change.
- **A layout no census count can clear** (its value type sharing a
  keyspace with another, or past the census's heads) is never retired:
  a walk of its value type's ranges at a writing open could count it.
