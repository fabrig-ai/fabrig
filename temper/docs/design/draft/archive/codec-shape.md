# codec-shape: generated code that stays small, merges itself, and pins its bytes once

Draft, 2026-11-17, from a session with the user, for the store and every wire alike. The user's aim: "a good end story across the board (store and wire)", "eliminating at the root all debt/pain we have accumulated due to the wrong approaches taken in the past (conflicts, generated code, goldens, etc)". This draft is the code's half: what the generator writes, what is still written by hand, how it is tested, and how it merges. [../layouts-both-ways.md](../layouts-both-ways.md) is the data's half: how a value or a message at one layout is read or written at another, in both directions. The figures are the survey's, on main at `a525e539f7` (`temper/docs/development/codec-survey-2026-11-17.md`).

Where they differ, this draft wins over skein's codec design (`skein/docs/design/data/codec/README.md`, sections 1 and 4, and `skein/docs/design/data/codec/golden.md`, section 1; `skein/docs/design/data/codec/command.md`, 2), the subset (`skein/docs/foundation/programming-model.md`, 10.2 and 10.3) and jig's store records (`jig/docs/design/protocol/store/records.md`, 1), until its goals fold it into them.

## 1. In one page

**The story, across both drafts:**

- **A schema per family** says each record's fields and bounds. It is the only specification of the bytes.
- **A generated record** is the schema as a Rust type, with encode and decode generated on it. It is the one type that knows bytes.
- **A translation** maps each record to the domain's type and back. It is generated where the two have the same fields, and written by hand only where the meaning changes or the shape is the owner's own.
- **A layout is named by its fingerprint** everywhere: in the store and on every wire. A reader at another layout converts by a plan derived from two schema texts. The old text travels with the data or with the peer, so no build keeps old texts, and no version number moves ([../layouts-both-ways.md](../layouts-both-ways.md)).

**Today:**

- 5,158 lines of schema generate 492,896 lines of Rust and 21,239 lines of generated tests. Beside them are 33,823 golden files, 2.8 MB (survey, 1).
- About 57,000 lines of translation are written by hand around them (survey, 3). In the samples, much of it copies fields between types of the same shape, or converts a scalar on the way.
- Generated files collide whenever two goals touch the same family or wire. Merges with "293 paths left with conflicts" and "72 paths left with conflicts" were settled by hand (survey, 6).

**The ideal:**

- **Small generated code** (section 3). `skein-codec` gets the exception `skein-lib` already has: generic items, written once and tested hard. The generated code calls them. Records lose their parallel `Parts` types, constructors and accessors, and are checked where bytes are made or read. The aim is under half of today's lines, with the same bytes.
- **Generated translation** (section 4). A maps file beside the schema says which domain type each record mirrors, and names a converter for each field whose meaning changes. The generator writes the rest into the owner's translation crate. The record stays its own type, and the codec crates still depend on nothing but `skein-lib` and `skein-codec`.
- **One pin on the bytes** (section 5). skein's `every` reference schema, its golden files and its fingerprint digests pin the encoding for every family, since bytes are made the same way whatever the family. Per family, the drift, fuzzing and memory tests stay, and read no golden file. Every family's golden directory and every generated test go.
- **Merges that settle themselves** (section 6). Generated files hold nothing that every change touches: no counts, no file named by a version, no appended list, no committed fingerprint file. A generated file is never merged by hand. The merge job makes it again from the merged schema.

**Unchanged:** every byte of every encoding, the schemas and their grammar, fingerprints, the codec crates as crates of their own, the drift test, and the subset for code written by hand.

## 2. Today

From the survey, on main at `a525e539f7`:

| Protocol | Schema lines | Generated lines | Generated test lines | Golden files | Golden bytes |
|---|---:|---:|---:|---:|---:|
| Client and public wires | 1,951 | 228,686 | 9,041 | 13,212 | 555,573 |
| Stores | 2,049 | 210,793 | 8,814 | 6,778 | 123,153 |
| Worker wires | 305 | 21,811 | 998 | 6,984 | 83,789 |
| smith's channel and agent | 421 | 24,548 | 1,198 | 4,824 | 34,869 |
| Views, cursor, fixtures | 432 | 7,058 | 1,188 | 2,025 | 2,027,833 |
| **All** | **5,158** | **492,896** | **21,239** | **33,823** | **2,825,217** |

- **What a generated module is made of** (survey, 2). In jig's tasks store family, 42,813 lines:
  - bounds' arithmetic: 8,508 lines;
  - measure, encode and decode: 12,382;
  - types: 2,919. Their `Parts` twins, constructors, accessors and `into_parts`: 7,574;
  - the `Path` enum and its names: 3,285;
  - the rest is limits, checks and glue.

  jig's client wire module, 133,964 lines, has the same proportions.
- **Why it is that large:** generated code is plain Rust of the subset, with no generic items (`skein/docs/foundation/programming-model.md`, 10.3). So every list, option and record gets its own code, field by field.
- **The translation** (survey, 3), outside tests:
  - jig's store, 17,796 lines; temper's store, 10,429;
  - jig's client wire: 11,590 on the engine's end and 11,104 on the client's;
  - the worker wires and smith: about 6,000.

  The samples hold plain copies, scalar conversions (`Wall` to nanoseconds is the most common, in every owner), and the owner's own shapes: packing, joins, remapping and checks.
- **Who reads golden files** (survey, 4):
  - the drift tests, for inventory and bytes;
  - the memory tests, for a full row;
  - a few tests written by hand;
  - the patch machine's tests.

  The generated tests embed their bytes and read no file. `every` lacks defaults, secret holds, a held enumeration, grouped booleans and split functions. Fingerprints are pinned there as digest literals.
- **What collides** (survey, 6). Every schema change touches:
  - family-wide items: `LIMIT_NAMES: [&str; N]`, `values() -> [u32; N]`, `CEILINGS`, the `Path` enum and the split functions' numbering;
  - committed fingerprints: `jig/crates/jig-protocol-store/src/generated/layouts.rs`;
  - golden files.

  A version move also renames the wire's module (`v30.rs`) and appends to its `.layouts` list, its patches and its kept texts. Since 11-10, temper's client wire changed its version-named files in 38 commits, its golden files in 35 and its `.layouts` lists in 19.

## 3. The generated code

### 3.1 The exception

- **`skein-codec` joins `skein-lib`'s exception** (`skein/docs/foundation/programming-model.md`, 10.2): generic items, written once and tested hard. It defines one trait, `Record`, and the generic helpers that measure, encode, decode and bound any field of a schema's types: integers, text, bytes, options, lists, held records and pieces, nested to the schema's fixed depth.
- **Generated code implements `Record`** for each declaration, and calls the helpers. The implementations are generated, never written by hand, so the subset's rule against hand-written trait implementations still holds for every line a person writes.
- **Code written by hand keeps the subset unchanged:** the translation's hand-written part, the domains, the components.
- **The guarantees stay where they were** (`skein/docs/foundation/programming-model.md`, 10.1): bounded execution, checked arithmetic, no panic a peer can trigger, no hidden state. The workspace's lints run over the helpers and the generated code, with no allowances.
- **Why this is enough:** what the subset protects is how code behaves, and generics change only how it is written once. Reading generated code as plain Rust is worth little: its specification is the schema, and a refusal names the field (`Session.reasons`), which is all anyone reads.

### 3.2 What a record gets

- **One type, with public fields.** No `Parts` twin, constructor, accessors or `into_parts`.
- **Checked where bytes are made or read.** Encoding checks each field against the limits it is given and refuses with the field's path, as a constructor does today. Decoding checks as it reads, as today. A value past its limits may exist in memory, but it can't become bytes, and no bytes past them become a value.
- **Its codec,** as one call to a helper per field, in schema order, for measure, encode and decode.
- **Its bounds as data.** Bounds are numbers that need no typed value. The generator writes a table of each module's declarations, shapes and fields, and one walk in `skein-codec` computes every encoded and decoded bound from it, with checked arithmetic and a stack capped by the table's depth. The interface stays: `encoded_bound` and `decoded_bound` on each type.
- **Its paths as data.** A path is a declaration and a field, each by its position in the schema, with their names in a table, rather than an enum variant per field. A refusal still names `Record.field`.
- **Limits by name,** as today. The owner sets them in a struct with a field per bound, and the names are a slice, so no count is written.

### 3.3 Size, compile and the gate

- **The aim:** under half of today's lines for each module, every byte, bound and fingerprint the same.
- **Measured before it spreads.** Goal K3 converts jig's tasks store family alone and measures:
  - its lines;
  - the compile CPU of a schema change, and of a change that doesn't touch the schema;
  - the run time of its decoders, on the fuzzy suite's corpus.

  The other families follow when the module is at most half its size and neither compile is slower.
- **Why compile is measured, not assumed:** the compiler makes a copy of a generic helper for each type, so the machine code shrinks less than the source does. The certain gains are in what is committed and reviewed, and in the compiler's and clippy's front end. The gain at compile is K3's to show.

## 4. The translation

### 4.1 What there is to generate

The survey's three kinds of translation (survey, 3):

1. **A plain copy:** the same fields, the same meaning, nested records and enumerations included.
2. **A copy with a scalar converted:**
   - a time to its nanoseconds (`Wall`, `Duration`);
   - a bit set to its integer;
   - a token or an id to its number;
   - a length checked into a narrower integer.
3. **The owner's own shape:** a record split or joined, a list packed by its bytes, names remapped through a lookup, a check of the owner's.

The first two are mechanical, and the generator writes them. The third stays written by hand. This part's goals, K5 for the stores and layouts-both-ways' L5 for the wires, first count each kind exhaustively, owner by owner; the survey sampled.

### 4.2 The maps file

- **A file beside the schema:** `schema/<family>.maps`, kept by the owner that translates. The schema, its parser, its fingerprints and its layouts never read it, so a map changes no byte and no layout.
- **What it says, for each record and enumeration it maps:**
  - the domain type it mirrors, by its path;
  - a field or a variant whose name differs on the domain's side;
  - a converter for each field whose meaning changes, by name: `last via wall`, `reasons via reasons`;
  - `by hand`, for a record whose shape is the owner's own.
- **Everything else matches by name:** a field to the field of the same name, a nested record through its own map, an enumeration variant by variant.
- **A converter is a pair of functions** in the owner's translation crate: one from the domain's value to the record's, and one back, which may refuse. Converters for `skein-lib`'s own types (`Wall`, `Duration`, `Token`) are written once, in `skein-codec`, and every owner shares them.

### 4.3 What is generated, and where

- **Both directions:** a function from the domain's type to the record, and one from the record to the domain's type, which may refuse with the field's path.
- **Into the owner's translation crate,** under its `src/generated/` (for jig's store, `jig-protocol-store-records`). That crate already depends on both the domain and the records. The record crates gain no dependency, so they still compile beside the domains and are reused by every build whose schema didn't change.
- **Exhaustive both ways:** the generated code names every field of both types, with no rest pattern. A field added to the domain's type with no counterpart, or to the schema with no counterpart, fails the build, as it does today in the translation written by hand (`jig/docs/design/protocol/store/records.md`, 1). It fails in the generator's output instead, with the map's line to change.
- **Mixed families:** a family may map some records and leave others `by hand`. The hand-written translation calls the generated functions for what they cover.

### 4.4 Why the record stays

- **Independence:** the record is the schema's type and nothing else's. The domain may rename, split or reshape its types, and change only the map. The schema may gain a field with a default and change only the map. Neither moves the other.
- **Wires with two ends:** a client wire's record is the one type the engine and the client share, each translating it into its own vocabulary. Each end keeps its own maps file, in its own translation crate.
- **One kind of code:** the store, the wires and smith's records all translate the same way. There is no codec that encodes a domain's type directly in one place and a mirror in another.

## 5. Tests

### 5.1 One pin on the bytes

- **The encoding is made of parts.** A record's bytes are its fields' bytes in schema order. Each field's bytes follow from its kind alone (an integer of a width, text, bytes, an option, a list, a held record, pieces), never from the family it is in. So pinning every kind and every way of nesting them on one schema pins every family's bytes. The drift test proves that each family's committed code is the generator's output.
- **The pin is skein's `every`** (`skein/tests/codec`). Its schemas use every type of the language, nested. Its golden files hold their bytes, and its fingerprint tests hold their digests as literals (survey, 4). A change to the generator or to `skein-codec` that moves a byte or a fingerprint fails there.
- **Complete by a check:** a test of the generator lists the grammar's constructs and the output's shapes, and fails when `every` lacks one:
  - each type and modifier;
  - defaults;
  - held records and enumerations;
  - secret holds;
  - grouped booleans;
  - a record long enough that its functions split.

  The survey found the last five missing (survey, 4). Each family's drift test refuses a schema that uses a construct the check doesn't list, naming it, so a new construct can't land without its pin.

### 5.2 What each family keeps

- **Drift:** the generator run on the schema (and on the maps file, section 4) equals the committed code.
- **Fuzzing,** in the fuzzy suite: each top-level record's decoder over arbitrary bytes never panics, and what decodes encodes back to the same bytes (`skein/docs/design/data/codec/golden.md`, 1).
- **Memory:** decoding the largest value the ceilings allow stays within the bound the generator computed. The value is built at run time, from the schema, by `skein-codec`'s generic values (`skein/docs/design/data/codec/run-time.md`, 2). Today it is read from a golden file.
- **None of these reads a golden file.**

### 5.3 What goes

- **Every codec family's golden directory** but `every`'s: about 24,000 files across the stores, the wires and smith (survey, 1). The views' snapshots are left aside (below). The patches' golden directories, about 8,000 files more, go with the patches ([../layouts-both-ways.md](../layouts-both-ways.md), 9).
- **Every generated test,** 21,239 lines, and the generator's test emitter, with the drift test's inventory of golden files.
- **The hand-written tests that pin bytes from golden files** are rewritten to build their values, or go where they only repeat what drift and `every` hold:
  - the store's checkpoints, manifests and history records;
  - temper's registered layouts;
  - the client wires' settings and login fixtures;
  - smith's shared fixtures.

  The survey lists them (survey, 4). Each keeps what it checks of its own, such as a projection or a payload's translation.
- **Outside this draft:** golden files that pin something other than a codec's bytes. That covers the views' rendered pages, the terminal and the TUI, events, skein-kv's own files, recordings, and the one-process story's directories. They are snapshots of their own subjects (survey, 4).

## 6. Merges

### 6.1 Nothing that every change touches

- **No count:** no `[T; N]` in generated code. Family-wide lists are slices, and functions loop to their end.
- **No file named by a version:** a family's module is `src/generated/<family>.rs`, its tests' `tests/generated/` go (5.3), and its version is a constant inside it. Since versions no longer move ([../layouts-both-ways.md](../layouts-both-ways.md), 3), the constant doesn't change either.
- **No appended list:** the `.layouts` lists, the patches' tables and the kept texts go with versions ([../layouts-both-ways.md](../layouts-both-ways.md), 9).
- **No committed fingerprints:** a build computes its families' fingerprints from the schema texts it embeds, when it first needs them. `jig/crates/jig-protocol-store/src/generated/layouts.rs` goes, and so does the store's `Layouts` row, which no open reads (`jig/docs/design/protocol/store/records.md`, 3).
- **No numbered pieces:** helper calls keep functions short, so long functions no longer split into numbered parts that shift when a field is inserted. A function that must still split is split by declaration, never by number.
- **Schema order throughout:** every table and list in a module follows the schema's order. Two goals that change different records change different places in it.

### 6.2 Never merged by hand

- **The merge job makes them again** (`temper/docs/design/protocol/tooling.md`, 4.1). A family's generated files may conflict while its sources merged clean: the schema, the maps file, its converters and written converters. Then fabrig's merge job runs that family's drift test with `SKEIN_CODEGEN_WRITE=1` on the merged tree, and stages what it writes.
- **Where the sources conflict too,** the job leaves all of them. The resolution merges the sources by judgement, then writes the generated files again with the drift test. No generated file is merged by hand: `AGENTS.md` says so (`AGENTS.md`, 2).
- **Marked for tools:**
  - `.gitattributes` marks `src/generated/` as generated, so a forge folds its diffs;
  - the repository's `.ignore` names it, so searches skip it unless asked.

  A record's specification is its schema.

### 6.3 What a pair of goals meets

Two goals that change different records of one family conflict only where they edit the same lines of its schema or its maps file. Everything generated from them is made again. Two goals that change the same record conflict in the schema, where a judgement belongs.

## 7. What stays

- **The schemas,** their grammar and the generator, which reads them through `skein-codec`'s parser (`skein/docs/design/data/codec/run-time.md`, 1).
- **Every byte:** canonical encoding, one encoding per value, sized and bounded decoding, refusals by field.
- **Fingerprints** (`skein/docs/design/data/codec/fingerprints.md`, 1), computed rather than committed.
- **The codec crates as crates of their own,** depending only on `skein-lib` and `skein-codec` (`jig/docs/design/protocol/store/crates.md`, 2).
- **The drift test,** now over code and maps alone.
- **The subset,** for every line written by hand.

## 8. Later and open questions

- **A careful review for changes to the generator and `skein-codec`.** Such a change is rare and moves every family at once, so the gate should flag it for a reviewer. The user's word, for later.
- **Compile figures** for the helpers are K3's (3.3). If the gate's compile grows, the helpers' boundary moves, not the aim.
- **Snapshot golden files** outside the codecs (5.3) are a question of their own.
- **The component's own rows, the secrets' store and the host tools' contract** keep their fixed formats ([../layouts-both-ways.md](../layouts-both-ways.md), 10).
