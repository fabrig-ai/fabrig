# design-economy: fewer concepts, and a design that says what the code can't

The code follows the design, so the design sets its size. This draft asks a design to name fewer concepts and to leave to the code what the code says better, first as a pilot in one area, run as temper goals from a template of their own. The reasons are in `notes.md`, 18.

## 1. In one page

- **The design sets the code's breadth.** Hand-written production holds about 2.9 characters for each of the design's (section 2). With the design as it is, a survey of the code's size found under 5% of production to cut without weakening a check. What remains is breadth: the concepts the design names, and the layers each one crosses. Smaller code needs a design that names fewer concepts.
- **The design is detailed partly because of how goals run.** A goal's designer writes the shapes its changes share into documents that outlast the goal, so that implementers working side by side agree (section 3). Once the code holds those shapes, the documents still repeat them.
- **What a design says:** concepts, rules, formats, failures, and why. It leaves to the code the types, fields and steps that no other area reads (section 4). A change to a design counts the concepts it adds and removes, and removes those with no behaviour of their own (section 5). An implementer says where a design makes it write the same thing twice (section 6).
- **A pilot first,** in one area, run as temper goals into main from a template of their own, so that every other goal runs as today (sections 7 and 8). It measures what fewer concepts save, and how much detail the implementer needs before its code gets worse.
- **Adoption is the user's,** area by area, once the pilot's figures are in (section 9).
- **Unchanged:** `skein/docs/foundation/programming-model.md`, `skein/docs/foundation/testing-strategy.md`, `DESIGN.md`, template 4 and its charters.

## 2. What the design holds

Characters other than whitespace, at main `2267794d41`:

- **Against the code:** the design documents (each tree's `docs/design/`, drafts and status files left out) hold 8,557,235 characters in 166,390 lines; hand-written production holds 25,108,122 in 919,382 lines. That is 2.9 to 1 by characters, and 5.5 by lines, since a design line holds 56 characters and a code line 27. By tree: hearth 1.2, skein 2.5, temper 2.8, smith 3.9, jig 4.2.
- **What it is made of:** prose 68%, lists 17%, tables 11%, code blocks 3%. It is a contract in words, not pseudo-code: of the 6,924 distinct names of public types, 1,307 appear in its code spans.
- **How it grows:** from 2026-11-01 to 2026-11-14 the Markdown under `docs/design/`, drafts and status files included, went from 7.0 to 13.0 MB, and the Rust from 99 to 148 MB.
- **How closely the code follows it:** the design's status files hold 665 **Not yet** entries and 282 **Departs**.
- **Where breadth shows,** at `cc3e59a47e`: 13,192 public types; a concept spelled in up to six layers (generated, domain, protocol and store, client domain, view, tools and forms); 759 of the client wire's type names defined again elsewhere.

## 3. Why the design is detailed

- **A contract between changes that run side by side.** The stock designer charter asks for "the shapes that cross changes (types, fields, signatures, wire forms) as binding contracts" in the design, delivered before the changes that cite them, and for "every part of the request designed, in documents that outlast the goal" (`temper/docs/charters/stock/designer.md`). Shapes two changes share have to be written somewhere while they run; written into lasting documents, they stay after the code holds them.
- **A guardrail for the implementer.** A goal designs on one model and codes on another, and the implementer takes the spec's shapes as its direction. Detail keeps it on the path. How much it needs has not been measured.
- **Nothing removes a concept.** Each draft brings its own vocabulary. A delivered design section binds, and later waves add compatible detail only. The implementer builds what is specified, and what the code taught goes in its report. Nothing in the loop asks whether a concept could go.

## 4. What a design says

### 4.1 What it keeps

- **Concepts,** each named once, with what it is for and its rules.
- **Invariants,** and the states and transitions of whatever has a life, as rules rather than code.
- **Stored formats and wires,** with their versions and upgrades: by their schema files where they have them, in words where they don't.
- **Failure:** the refusals, their order, and what a failure leaves behind.
- **Order and time:** what waits for what, deadlines, what runs at once.
- **Trust:** what is trusted, and who may do what.
- **The edges between areas:** what one area reads of another.
- **Why,** and the options turned down.

### 4.2 What it leaves to the code

- Types, fields and signatures that no other area reads, and the modules that hold them.
- Steps that the rules of 4.1 already fix.
- Layouts and numbering that no format, wire or other area reads.
- Lists the code holds: a crate's dependencies beyond what the crate graph sets, a module's functions.
- Restatements of another document, of the programming model, or of a schema.

### 4.3 Contracts between a goal's changes

The shapes that two changes of one goal share go in their specs, word for word the same, and bind for the goal. Once the changes land, the code holds them. The lasting design binds only what 4.1 keeps. This departs from the stock designer charter (section 8).

### 4.4 Documents

`DESIGN.md` holds as it stands: status files, citations, the size limit, one concept to a document. A lean document keeps the section numbers that others cite, and moves sections with `tools/citations/move.py`.

## 5. Fewer concepts

### 5.1 What a concept is

A name with behaviour or rules of its own: an entity, a state, a refusal, a record, a role. The same name in two layers is one subject seen from each (`DESIGN.md`, 5). An enum's variants are part of its concept, not concepts of their own.

### 5.2 The budget

- **Counted:** a change to a design lists, in its goal's note, the concepts it adds and the ones it removes, with its area's public types before and after.
- **Removed:** a concept with no behaviour of its own is merged into the one it copies, or goes. Examples: a record with another's members and no rule of its own; a layer's copy that adds no meaning, check or projection; two names for one thing.
- **Justified:** a new concept says why an existing one can't carry it.

### 5.3 What stays fixed

The programming model, whole. Each layer has its own entities (`skein/docs/foundation/programming-model.md`, 4.1), the types belong to the domain (4.4), siblings share no domain types (4.5), and the subset holds (10). Fewer concepts works inside it: fewer per layer, and fewer layers for a concept where a layer adds nothing of its own.

## 6. The way back from code to design

- **The implementer says so:** where the design or its spec makes it write the same thing twice, or a type with no behaviour of its own, it says so. When the simpler form crosses its change's edge, it asks. Otherwise it writes it under `## Simpler` in its report, naming the place and the simpler form.
- **The designer takes it:** among options that keep the design's shape, the designer, or the coordinator once the designer has parked, prefers the one with fewer concepts. It accepts a simplification that keeps the behaviour and leaves the lasting design true.
- **Detail on demand:** when a change's question, failure or result shows that the lean design left out something the change needed, the designer adds it, to the design if 4.1 keeps it and to the spec if not. The goal's note lists each addition: what was missing, what showed it, and what was added. That list measures how much the implementer needs.

## 7. The pilot

### 7.1 The area

**hearth's render domain:** the crate `hearth-domain-render` and its design, `hearth/docs/design/domain/render/`. A read-only survey of every tree picked it at main `e2acd5d5b7`. It was the one area of 8,000 to 40,000 production lines that no unfinished milestone of a draft touches, with 2 commits in the 7 days before.

- **Before:** 17,633 lines of hand-written production, 13,388 of tests in its crate, no generated code, 278 public types; 444,073 characters of production against 80,996 of design, 5.5 to 1, a leaner design than most. So the pilot tests fewer concepts more than less detail.
- **Pinned by:** its library tests; `hearth-render-world`'s goldens, laws, properties, oracle, memory and fuzzy cases; `hearth-builder-world`; and the 51 expected files under `hearth/tests/render/`, kept byte for byte.
- **Its edge:** no schema of its own. `hearth-domain-builder` re-exports the whole crate, and at least 101 type paths are used outside it.
- **Where the concepts repeat:** its pipeline gives one subject a record at each stage. A value has four (revision's, the composed one, the evaluated one, an operand), a task four, a file three, and an installed package's identity three. A design of four results (a revision's inputs, a target's settings, the admitted concrete entries, the specs with their typed differences) may need fewer of the records between them.
- **Not established:** read from the code alone, the survey proved 16 to 19 lines of cut, merging records whose members match and two copied blocks. The wider collapse is unpriced, since positions, removal, provenance and ownership differ between the stages. That is what the pilot tests: a small figure is an answer too.

### 7.2 What it changes

- **The design:** the area's design is rewritten under sections 4 and 5, with fewer concepts and the same behaviour. The designer writes the leanest design section 4 allows, and adds detail only on demand (6).
- **The code:** refactored to the new design.
- **Unchanged:** what other crates use of it, the behaviour its tests expect, its golden and expected files, and its status files' gaps. A type no other crate uses may go. A test of a type that goes is rewritten to check the same behaviour through what stays.

### 7.3 What is measured

Read from main before the goal (7.1) and after it lands:

- the area's hand-written production lines, with its tests and generated code apart;
- its public types;
- the characters of its design documents;
- the detail added on demand (6), with the changes' questions and refused deliveries;
- the goal's minutes and spend, from its record.

## 8. The pilot's charters

- **A template of its own:** a copy of template 4, "goal for a draft", with its models and bounds. Its goal agent and implementer are copies of charters 5 and 2 carrying the changes below over the stock's text and fabrig's lines (`temper/docs/charters/`). In fabrig's lines for the goal agent, a coding change names the pilot's implementer as its producer. Template 4 and charters 5 and 2 stay as they are.
- **The goal agent's `designer` part:**
  - "Your goal", item 1, reads: "**A lean design, eventually.** Every part of the request designed, in documents that outlast the goal, keeping what `temper/docs/design/draft/design-economy.md`, 4.1 keeps and leaving to the code what 4.2 leaves."
  - "Your design's rules", the wave's bullet, reads: "**A wave's design** comes before its changes: the sections the request asks for, as that draft's 4.1 keeps them, by path and section, with wire and stored forms. The shapes the wave's changes share (types, fields, signatures) go in their specs, word for word the same, binding for the goal (4.3)."
  - Two bullets join it: "**Fewer concepts:** your note lists the concepts the goal adds and removes, with the area's public types before and after (5.2)." And: "**Detail on demand:** add what a change showed the design lacked, and list each addition in your note (6)."
- **The goal agent's `coordinator` part,** for a question: "Decide" gains "Among options that keep the design's shape, prefer the one with fewer concepts. Accept an implementer's simplification that keeps the behaviour and leaves the lasting design true. List detail added on demand in your note (6)."
- **The implementer,** after "Not yours": "Where the design or your spec makes you write the same thing twice, or a type with no behaviour of its own, say so: as a question when the simpler form crosses your change's edge, otherwise under `## Simpler` in your report, naming the place and the simpler form."
- **How the template carries them is temper/dev's.** Today a set of charter files overlays only charters 5 and 2 (`temper/docs/design/client/ctl/overlays.md`, 2). So the copies are either literal text assembled from the stock's set, fabrig's lines and these changes, or a set that `settings apply` lets serve another charter.

## 9. Adoption

- **The user decides,** with the pilot's figures, whether its rules hold beyond the pilot.
- **If they do, area by area:** `DESIGN.md` gains sections 4 to 6 for the areas a list names, as `tools/citations/sized.txt` names the trees the size limit covers. fabrig's charter set takes section 8's changes, and the pilot's template goes. Each area's rewrite is a goal of its own, in the order the user picks.

## 10. Open questions

### 10.1 The pilot's success

Which figures count as success: how far production and public types fall, against how much detail had to be added on demand. The figures are the user's.

### 10.2 New code from a lean design

The pilot refactors an area that is already built. A second pilot could build a designed but unbuilt part from a lean design, to measure the implementer on work it has no code to read from.
