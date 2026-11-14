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
  `data/codec/README.md`, 9.4).
- **The registry** (3): rows of the component's own, a layout per value
  type and id, with its fingerprint and the texts it was made with, and
  the texts themselves. It holds every layout a value of the map may be
  at.
- **Heads** (4): a registered value's first two bytes are its layout's
  id. The store writes the binary's id as a value enters skein-kv and
  converts a value at another layout as it leaves, so every decoder,
  jig's and the application's, reads the binary's layout with its
  family's version at the head, as it always has.
- **At open** (5): the registry read, or seeded from the kept texts with
  no value rewritten; the binary's layouts registered; a plan made for
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
- **Patched as before:** the component's own rows under its tag (the
  format row, the layouts and wires rows, transcripts' places and
  attempts, the workers' openings and the registry's own rows, 3), read
  before the registry or holding it; the secrets' store's families
  (`secrets.md`), read once at open from a store copied whole at each
  upgrade; and the transcripts' frames (`transcripts.md`, 6), already
  converted as they are read. Each keeps its version and its patches.
- **Held records** of any family the binary holds a text of, registered
  or not, are part of their holder's layout (skein's
  `data/codec/README.md`, 9.4): a core projection that holds a tasks
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
    pub current: &'static str,          // the binary's text of the family; a patched one's latest kept text
    pub types: &'static [ValueType],    // its value types; empty for a family the store patches
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
  milestones (`01 01 0a`) are one, its live tasks another; a family whose
  roots are records of their own (the settings') has one per root. A
  broad root whose keys do not tell its variants apart is one value type
  without a variant, the declaration whole (skein's
  `data/codec/README.md`, 9.4): the settings' `ApplicationState`, led by
  its `follows`, and the tasks' live and ended tasks, whose keys hold a
  task written before talk (`live`, `ended`) beside one written since
  (`live_talk`, `ended_talk`); a change to any variant of its root moves
  it. Its
  number names it in the registry's keys (3), so a declaration or a
  variant renamed (`was`, skein's `data/codec/README.md`, section 3)
  keeps its value type.
- **One root per range:** a range never holds two roots that only the
  values' bytes tell apart, since a layout names one root. The settings'
  charter versions (`01 00 08 03`) held two, a version without an
  overlay as a `CharterVersion` and one with an overlay as an
  `OverlaidCharterVersion`, decoded by trial at the same keys
  (`../../domain/engine/overlays.md`, 7). They are one value type at
  `CharterVersion`, whose declaration gains `overlay: option Overlay`,
  and jig's written job `charter-versions` (6) puts each stored value
  again as that one root; the binary then declares no
  `OverlaidCharterVersion`.
- **Checked where the set is admitted,** at every open, beside
  `opening.md`, 3's checks, a failure refused `Set` with patch 0: each
  number given once in its family; each root a declaration of `current`
  and each variant given only where that root is broad, naming one of
  its leading enumeration's variants (`Schema::root`); each range
  non-empty and the ranges of all the family's types ordered, disjoint,
  and together exactly the family's `keys`, so that each key the family
  holds resolves to one value type; a family with types whose
  `current` does not parse, or that is not in the map (`KeptMap::Store`),
  or a `current` whose version is not the family's at the binary's
  level.
- **Resolved by key:** the store keeps the value types' ranges in one
  table ordered by `from`, the application's under its tag, and finds a
  key's by a binary search, as it finds a key's family for the census
  (`README.md`, 6.3).

## 3. The registry

A family of the component's, `jig-store-registry`, beside `openings`
in `jig-protocol-store`, new at its first version and kept afresh with
no patch (skein's `data/codec/command.md`, 1), so the component's
version and its patch set stay as they are:

```
family jig-store-registry 1

# A layout of a value type, by its id within the value type.
versioned record Layout {
    root: text 256                  # its root declaration, as its text spells it
    variant: option text 256        # its variant, for a broad root
    layout: fixed 32                # its fingerprint (skein's data/codec/README.md, 9.4)
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
- **Ids** are `u16`s within a value type. The seed takes the family's
  version (5.1); a layout registered later takes one more than the
  greatest id its value type's rows hold. An id comes back only once its
  row is retired, when no value holds it (5.4).
- **Read at every open** by its own kept text, as the component's rows
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
- **What the census counts** is the stored head (`README.md`, 6.3): per
  keyspace, values per layout id.
- **Until the registry is active** (5.1: its seed or its first rows
  committed) neither edge maps anything: patches walk a store behind as
  before.
- **The rehearsal** decodes every record of a registered family in a
  seeded store as a generic value of its layout's text, from its range's
  value type's root at that layout, where its head names a layout of the
  registry; a store not yet seeded as before (`opening.md`, 5).

## 5. At open

### 5.1 The seed

A store with no `Layout` row, once its pending patches have run, holds
every registered value at the version of its family that the binary's
levels give it (`records.md`, 3), which is what its head says. Its seed
is a `Layout` row per value type of every registered family, at id that
version, made from the family's kept text at that version (`KeptFamily::texts`)
with the held families' kept texts at their versions; and a `Text` row
for each text named. No value is read or rewritten: one commit of a row
per value type, and a row per text.

### 5.2 The binary's layouts

For each value type, its layout in `current`, with the binary's texts of
the families it holds (each family's `current`): its row is the registry's of that
fingerprint, whose id is the binary's id; with none, a new row at the
next id (3) and its texts' rows, committed with the seed or alone. An
empty store's first commit (`README.md`, section 5) writes the binary's
layouts at their families' versions as ids, so its heads are its
families' versions and the edges map nothing until a later binary moves
a layout.

### 5.3 The check

Before anything is written, every open (whether it writes or not)
plans each layout the registry holds, or will hold once its pending
patches and its seed are made, to its value type's binary layout
(skein's `data/codec/layouts.md`): from its row's text and held texts
and its root and variant, to `current`'s. A layout planned is kept in
memory for the read edge. A layout with no plan is refused, unless:

- a written job of the binary converts from it (`jobs.md`, 4), pending
  or done; or
- no value holds it: the census counts none at its id in each keyspace
  its value type's ranges reach, with no count past the census's heads
  there (`README.md`, 6.3), where it is retired (5.4).

The refusal, `OpenRefusal::Layout`, names the family, the value type
(its number, root and variant), the id, why (`Underived`'s, a text its
row names that is missing or does not parse, or a value type the binary
does not declare), and the values the census counts at that id, or that
it cannot tell (`opening.md`, 1.8). A layout row of a value type the
binary no longer declares is refused the same way while a value may
hold it.

### 5.4 Retirement

A layout that is not its value type's binary layout and that no value
holds (5.3's second exception) has its row erased by the next open that
writes (`opening.md`, 2), with each text no other row names. No open
writes for retirement alone.

### 5.5 When the open writes

The open writes before serving, in a new generation made by links
(`opening.md`, 2), when a patch is pending, when the registry is not
seeded, when a binary layout is not registered, or when a job is
pending (`jobs.md`, 2); then, after the patches, one commit (or as many
as the store's commit edge needs, `records.md`, 4) writes the seed, the
binary's new layouts and the retirements, before any job runs. An open
with none of these writes nothing, so a binary that changes no layout
and brings no job opens the store in place, as before (temper's
`draft/store-shape.md`, section 4, step 3: an older binary can read all
such a binary writes).

## 6. Written changes

A change no plan derives (a field computed from others, a variant
dropped, a type changed in meaning) is its owner's **written job**
(`jobs.md`, 4): a job that names the value type and the layout it
converts from, walks that value type's keys, and is handed each value at
that layout as it is stored, with that layout's texts, to put it again
at the binary's. Its owner's function reads the old value by skein's
generic values (skein's `data/codec/README.md`, section 8) and builds
the new by its generated code. The check takes the layout as converted
(5.3), and the next writing open retires it.

A written job also takes a layout that a plan reaches when the values
at that layout are not all the root it names: the settings' charter
versions at their seeded layout (2). Its job, `charter-versions`,
converts from `CharterVersion`'s layout in the settings' last kept text.
Its function reads each value as that text's `CharterVersion` or,
failing that, its `OverlaidCharterVersion`, as the codec did by trial,
and puts the value again as the binary's `CharterVersion`. The read edge
leaves values at a written job's `from` layout to the job, never to a
plan (`jobs.md`, 4).

## 7. Bounds

- **In memory, at open:** the registry's rows, each text parsed once
  (skein's `MAX_TEXT` each), and a plan per held layout (skein's
  `data/codec/layouts.md`, 5); kept while the store serves, counted in
  `README.md`, section 8's worst case beside the patch machines, which
  no serving open holds.
- **Per read:** a converted value is built in a `Buffer` at the store's
  value bound (`records.md`, 4), with one skein `layout::Workspace`
  beside it fitted at open to every plan the store holds, so a read
  allocates nothing; a value at the binary's layout, or at
  one whose plan keeps, is rewritten in its two head bytes alone.
- **The registry's rows** within skein-kv's value bound: a `Text` at
  most 262,144 bytes, refused `Set` with patch 0 where a `current` or a
  kept text the seed needs is longer.

## 8. Tests

- **Step tests,** in `jig-protocol-store`, over the test owners' sets
  (`opening.md`, 3), whose fixture families gain value types and a
  `current` past their last kept text: a store at the last kept text
  seeded with no value rewritten, its rows as 5.1 says; a field added
  with a default, read at the old layout and converted, written at the
  binary's id and read back; a broad root's variant unmoved while a
  sibling's record changes; a held record's change converting its
  holder; the census counting by id; each refusal of 5.3 and its words
  (`opening.md`, 1.8); retirement; an empty store's first commit.
- **The real families,** in `jig-protocol-store` and
  `temper-protocol-store`: every value type's layout in its family's
  latest kept text derives to `current` (skein's `data/codec/layouts.md`),
  so a landing that changes a registered schema with no default or
  written job fails in its owner's focused suite, not at a deployment's
  open.

## 9. Open questions

- **The secrets' store and the component's rows** keep their patches
  (1). Registering them would need the registry before the format row
  is read, and the secrets' store's own registry; their rows are few and
  rarely change.
- **A layout no census count can clear** (its value type sharing a
  keyspace with another, or past the census's heads) is never retired:
  a walk of its value type's ranges at a writing open could count it.
