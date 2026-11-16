# one-owner: a concept's values, defined once

2026-11-14. A second pilot after `design-economy.md`'s: fewer layers for a concept where a layer adds nothing of its own (`design-economy.md`, 5.3), which the first pilot couldn't reach, since it kept every source outside its crate as it was (`design-economy.md`, 7.2). It runs from the first pilot's template and is counted the way its count was (`temper/docs/development/debt/reviews/2026-11-14-design-economy-e2.md`). The reasons are the user's (`notes.md`, 18).

## 1. In one page

- **The rule:** a concept's values are defined once, by the domain that decides them. Another layer keeps a type of its own for them only where it adds a meaning, a check or a projection (section 2).
- **Where a machine names a domain's values, a vocabulary:** a crate below the domain that holds them, which a machine may name. The model gains the role (section 4). The code already splits such crates off for compile time, in a wider form.
- **The pilot:** hearth's render records, written three times (`hearth/docs/development/debt/D013-records-three-times.md`): render's own, the client records' copies, and the builder's translations between them (section 3). One goal, from `design-economy.md`'s template 8, with two more bullets in its charter (sections 5 and 7).
- **Scored** by what a change touches and how clearly the code reads, with size and cost as side figures (section 6). Which figures count as success is the user's, read from the count.
- **Unchanged:** the behaviour, what the client prints, and the tests that pin them. Outside hearth, nothing but the model's text and the workspace entries the new crate needs.

## 2. The rule

- **One owner per concept.** A concept's values are defined once, by the domain that decides them. A layer that names them uses the owner's values, and code that only copies one type into another goes, with the tests that only check the copy.
- **What a layer keeps:** a type of its own, or a translation, wherever it adds something:
  - an envelope: a variant or field only that layer has, such as the client records' `Outcome::Unavailable` and `StoreLimit::Layers`, which the builder's `src/client.rs` fills;
  - a projection: typed source names printed as bytes, or a filter kept printable with its expected arity;
  - a compilation or a refusal: configuration's declarations, compiled into render's with invalid expressions refused and `bless` left out;
  - a position or a source, kept for a diagnostic.

  Each stays, with the tests that check it, and the goal's note lists it with its reason.
- **A format's version lives in the code that writes it.** The client's JSON lines have a version of their own (`hearth/docs/design/client/README.md`, 3.2). Their printer maps the owner's values to that version's form, so no second set of types holds the version. Whether a new version bridges the previous one or refuses it follows that wire's rule: a client's wire moves its version unbridged (`temper/docs/charters/fabrig/common.md`).
- **The rest of the model holds:** each layer has its own entities (`skein/docs/foundation/programming-model.md`, 4.1), the domain parses nothing, a machine decides nothing about the domain, siblings share no domain types (4.5), and the subset holds (10).

## 3. The area

At main `2e457d3f00`:

- **Render's records:** render's parts, changes and errors in `hearth-domain-render`; their copies in `hearth-client-protocol-records`, a machine that prints them as tables and as the client's JSON lines; and `hearth-protocol-builder`'s translations between them, in the 25 files under its `src/translate/`, 2,772 lines that are neither blank nor comments. Their sizes, for scale: the records crate's 162 struct and enum names share 140 short names with render's, some of them unrelated (`Event`, `Limits`, `Form`), so this is a survey's figure, not a count of copies, which section 6.2 makes. The builder's tests hold 177 `_back` functions, 13 of them in `src/tests/fixtures/`: some invert the translations that go, others check boundaries that stay (`src/tests/translate/instructions.rs`).
- **Then configuration:** `hearth-protocol-config`, a machine, parses configuration into records of its own, and the builder's `translate/declarations.rs` and `translate/documents.rs` carry them into render's. Compiling and refusing are a translation's own work (section 2), so only a record copied whole goes.
- **The builder's other translations,** to the template engine, mmdebstrap and the store, speak those tools' vocabularies and stay, unless one only copies render's values, as mmdebstrap's `Component` may.

## 4. The model's change

The goal's first change writes it into `skein/docs/foundation/programming-model.md`, in these words or ones that say the same in the model's voice:

- **Section 1,** the layers' bullet: "A layer may be several crates: a domain with child domains (4.5), a vocabulary of a domain's values (4.6), a crate per protocol machine."
- **Section 4's table** gains a row below the domain's: `vocabulary`, depending on "lib, and other vocabularies", for "a domain's values, for the crates that name them (4.6)".
- **Section 4,** the machines' bullet: "**Machines depend on lib only,** and on the vocabularies of the values they encode or decode (4.6), not on io and not on each other".
- **10.2,** step code's list: "Step code (io, the protocol machines, vocabularies, and a service's protocol layer, domain and `iterate`) uses:".
- **A new 4.6, Vocabularies:**
  - "A domain's values that other crates name (its records, changes, errors and their limits) may live in a vocabulary: a crate of their own, below the domain, that depends on lib and on other vocabularies only. It is step code (10)."
  - "**It holds values and their own operations:** their definitions, the inherent methods that read or name a value (an accessor, a name, an ordering), and derived trait impls. Nothing it exports holds a domain's state, steps it or decides for it. A method that reads a domain's limits or chooses an outcome stays in the domain, as a free function."
  - "**They are still the domain's types** (4.4). The domain re-exports its vocabulary along the same module paths, so a caller builds through either crate."
  - "**Who names it:** the domain and its parents, a protocol layer, and a machine that encodes or decodes those values, which then holds no copy of them. A sibling domain doesn't (4.5)."
  - "**A copy needs a reason.** A layer keeps a type of its own for a domain's values only where it adds a meaning, a check or a projection. A format's version lives in the code that writes and reads it, not in a second set of types."
  - "**A crate split off for compile time** may be wider: some depend on whole child domains or export decisions. It is a vocabulary only where it meets this section, and a machine names no other."

## 5. What changes

- **The model:** section 4, in the goal's first change.
- **The design,** lean, as `design-economy.md`, 4 and 6 ask:
  - hearth's crate graph gains render's vocabulary (`hearth/docs/design/README.md`, 4.2);
  - the client's output prints render's values (`hearth/docs/design/client/README.md`, 3.2);
  - render's design, where it names the records;
  - D013's status and measure, as the goal leaves them.
- **The code:**
  - **The cut's closure first:** which of render's items move, the `pub(crate)` items they reach, and which methods decide and so become free functions in render. `EntryChange::pair`, for one, reads the call's size limits and decides on a hunk request (`hearth-domain-render/src/parts/config.rs`). Public module paths stay, through render's re-exports, as `jig-domain-core-vocabulary`'s do (`jig/docs/design/domain/engine/README.md`, 3), so `hearth-domain-builder`'s re-export of render still serves.
  - **The move:** the values the records copy, to `hearth-domain-render-vocabulary`, with their own operations.
  - **The printer:** the records crate prints the vocabulary's values with concrete free functions. Its inherent printers, such as `impl Data` (`hearth-client-protocol-records/src/parts/data.rs`), can't stay on another crate's types, and the vocabulary takes no JSON dependency. It keeps what render doesn't own and what section 2 lets a layer keep.
  - **The copies go:** the builder's translations that only copy, and the `_back` tests that only invert them. The other translations and their tests stay.
  - **Then configuration,** where a record is copied whole.
- **Unchanged:**
  - the observable behaviour;
  - what `hearth render` and `hearth diff` print, tables and JSON lines, byte for byte, pinned by tests: where no test pins a record's printed form, the goal adds one before it changes the record;
  - the 51 expected files under `hearth/tests/render/`, byte for byte, never blessed;
  - `hearth-render-world` and `hearth-builder-world`, with their cases, expectations and oracles;
  - outside hearth, everything but the model's text and the root `Cargo.toml` and `Cargo.lock` entries that register the vocabulary and its dependents.
- **The charters:** section 7.

## 6. What is measured

At the goal's two endpoints, the commit it starts from and its landing, by the same script, parsing the source.

### 6.1 What a change touches

- **The records:** render's `Component` (a package's archive component), `DataChange` (a data part's change) and `Collision` (a collision error), and each type paired with one of them at the start (6.2).
- **A site** is a `struct` or `enum` item that defines one of them, not a `use` or an alias; or a function or method whose body names one of its fields or variants one by one: a struct literal, a pattern that names fields, a match arm on a variant (through `Self` too), or a field read. A function counts once for each record it names, however often it names it.
- **Names resolve** through `use`, aliases and `Self`. Production and test code are counted apart.
- **Reported:** the sites at the start, listed. At the landing, each is mapped through moves and renames as kept, moved or removed, and new sites are listed as added. A site that only moved counts as neither a gain nor a loss.

### 6.2 Copies

- **Pairs:** a render type and a records or configuration type that one of the builder's translation functions builds from the other, listed at the start; at the landing, the pairs left, each with its reason from the goal's note.
- **The translation's lines:** the files under `hearth-protocol-builder/src/translate/` and `src/translate.rs`, lines that are neither blank nor comments.
- **`_back` functions:** in all of the builder's `src/tests/`, those that invert a pair apart from the rest.

### 6.3 Quality and the rest

- One code review of the goal's diff, by `design-economy.md`, 7.3's kinds, saying whether the code reads more clearly than before, with examples.
- Side figures: production and test lines by crate, the design's characters, and the goal's minutes and spend.

## 7. The charters

Template 8's goal charter, charter 17, gains two bullets in its `designer` part, under "Your design's rules". Charter 18 and the rest of template 8 stay as they are.

- The subset's bullet, as `design-economy.md`, 8 gives it, which has lagged since that draft's E1.
- "**One owner per concept:** a layer keeps a type of its own for a concept only where it adds a meaning, a check or a projection; otherwise it names the owner's values, from the owner's vocabulary where a machine names them, and a format's version lives in the code that writes it (`temper/docs/design/draft/one-owner.md`, 2). Your note lists each second type you keep, with its reason."

The user adopted both bullets into fabrig's charter 5 on 2026-11-15, the second citing `skein/docs/foundation/programming-model.md`, 4.6. Template 8's charter is then built from charter 5, and charter 18 retires, so template 8's coding changes run charter 2.

## 8. Open questions

- **The steward's records:** hearth's client design has the steward answer with these records, from M3 (`hearth/docs/design/client/README.md`, 3.2). With section 4, the steward's protocol layer names render's vocabulary, not its domain. Whether the steward's own domain needs them is M3's design.
- **Adoption:** the user adopted the rule into fabrig's charter set on 2026-11-15 (section 7), so new work follows it. Converting existing areas to it is the user's, area by area, as for `design-economy.md`, 9. jig's contract written by hand (`jig/docs/development/debt/D007-contract-written-by-hand.md`) plans generated mirrors; under the rule its codecs would name the owner's values instead.
