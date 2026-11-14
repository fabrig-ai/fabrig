# design-economy E2: the count

2026-11-14. Milestone E3 of `temper/docs/design/draft/design-economy.execution.md`: the figures `temper/docs/design/draft/design-economy.md`, 7.3 asks of the pilot, goal 2893, at its two endpoints: `4d7dcc520e`, the commit its goal started from, and `355f28a06a`, its landing on main. No other commit touched the area between them, or since.

Counted by script from git at both endpoints, and from the goal's and every change's records: specs, reports, questions, answers and deliveries. Lines are Rust lines that are neither blank nor comments; characters are characters other than whitespace, outside comment lines in the code, as `temper/docs/design/draft/design-economy.md`, 7.1 counts them. Constructs of the Rust subset are counted by parsing with syn, not by searching text. Not checked: the code by building or running it, beyond the gate that landed it.

## 1. In one page

- **Size:** production fell 2.3% (407 lines), the design 13.8% (11,195 characters). Tests grew 68 lines. Nothing moved to another crate.
- **Concepts:** 23 struct and enum definitions went, and 20 type aliases came in their place, so the area's type names went from 341 to 338. Public type names fell from 278 to 251, every one of them by narrowing to the crate.
- **Guidance:** the implementers started from 33,937 characters of the lasting design their specs cite, a contracts document of 3,228 and `## Steps` of 549 in all. No detail was added on demand: no change asked a question, and none had a delivery refused.
- **Quality:** the code is shorter and less repetitive, but its largest cut breaks the programming model in step code (`skein/docs/foundation/programming-model.md`, 10.3), as the goal's contracts asked: generic types, function pointers and closures. Section 5.4 lists the review's findings by kind.
- **Cost:** 274 minutes. The goal agent spent 1,083.22 and its changes 323.56, in the record's unit. The design took 86 minutes and 72% of the goal agent's spend before its first change was cut. The changes took 26 minutes side by side, and the landing 157, held three times by a failed effect.
- **The figure:** in this area, fewer concepts shortened the design and narrowed the crate's edge, but bought few lines, because the records between the pipeline's stages differ in rules the code reads (position, provenance, ownership), as the survey warned (`temper/docs/design/draft/design-economy.md`, 7.1). The repair of the subset's breaches runs as goal 3045, which keeps the programming model whole, by the user's decision.

## 2. Size

| | Start `4d7dcc520e` | Landing `355f28a06a` | Change |
|---|---:|---:|---:|
| Production lines, `hearth-domain-render` | 17,666 | 17,259 | -407 (-2.3%) |
| Production characters | 445,687 | 437,660 | -8,027 (-1.8%) |
| Test lines, in its crate | 13,355 | 13,423 | +68 |
| Generated lines | 0 | 0 | 0 |
| Design, `README.md` | 49,441 | 39,717 | -9,724 (-19.7%) |
| Design, `spec.md` | 23,739 | 22,254 | -1,485 (-6.3%) |
| Design, `errors.md` | 8,122 | 8,136 | +14 |
| Design in all, contracts apart | 81,302 | 70,107 | -11,195 (-13.8%) |
| Production characters per design character | 5.5 | 6.2 | |

The contracts document held 3,228 characters at its fullest and was deleted by the goal's last change. E2's whole diff, by kind: the crate's production 59 files, +853 -1,227 lines; its tests 12 files, +125 -70; its design 3 files, +225 -1,134 (paragraphs rewrapped to a line each, so the line counts overstate the cut that the characters measure); the status files, the draft's execution and status files, and `.temper/known-reds.md`, a few lines each. No other crate or document changed, so no code was moved elsewhere to count as a cut.

## 3. Concepts

| | Start | Landing |
|---|---:|---:|
| Struct and enum definitions | 332 | 309 |
| Of them public | 278 | 230 |
| Type aliases | 9 | 29 |
| Of them public | 0 | 21 |
| Public type names, definitions and aliases | 278 | 251 |
| All type names | 341 | 338 |

- **In the design** (the goal's note): one name added, "admitted entries", the third of the four results the design now frames the pipeline by (`hearth/docs/design/domain/render/README.md`, 3); none removed. The module list and the history of the tool render replaces went.
- **In the code** (the note and the changes' reports): added `diff::Keyed<T>` and `diff::Changed<T>`. Removed as definitions: 9 keyed-change enums and 9 before-and-after records, now aliases of those two; `resolution::Triple`, and `diff::Listed`, now an alias of `resolution::Inst`; the repository tree's file records, merged into the declared installs' `Install` and `Make`; the collision checks' own claimant, merged into the entries'.
- **Narrowed, not removed:** 27 public types, and many functions, became the crate's own, since no code outside reads them.
- The aliases stand because the pilot kept every source outside the crate as it was (`temper/docs/design/draft/design-economy.md`, 7.2): each removed record lives on as a second name for a shared one.

## 4. Guidance and detail on demand

| What the implementers were given at the start | Characters |
|---|---:|
| The lasting design their specs cite: `README.md` 2.3, 3, 4.1, 5.1 and 5.4; `spec.md` 1 and 2.3, with their subsections | 33,937 |
| The contracts document, at its fullest | 3,228 |
| The specs' `## Steps`, six specs, 51 to 146 each | 549 |
| The specs whole, for scale: goal, acceptance, inputs, scope and report | 22,238 |

Detail on demand added nothing. The goal's note lists no addition. None of the six changes asked a question or had a delivery refused, so nothing reached the designer that the note would not show. Each implementer delivered once, in 5 to 15 minutes.

The `## Simpler` sections, read against section 6: each implementer looked for a record that copies another and found the rest carrying rules of their own (positions and nulls kept for diagnostics, provenance and ownership read by later checks, borrowed and owned trees, a first claimant that error provenance needs). One named a simpler form outside its reach: naming `Inst` everywhere instead of the `Listed` alias, which outside sources would have had to change.

## 5. Quality

### 5.1 Questions and refusals

None from the changes. The landing asked three questions, all about reds outside the goal's diff: one inherited from main and two smith end-to-end tests that passed alone. They went on `.temper/known-reds.md`, and the goal agent passed the third failed effect up to a person.

### 5.2 What got better

- Copies folded into one: the target-limit and composition errors, one walk that charges a held tree for two, the path-segment check, the part's name in one match instead of two, one lookup of a part in a spec, the known-resolution layer keys computed in one place, and one claimant record.
- The crate's edge says what outside code reads: 27 public types and many functions became the crate's own.
- Behaviour pinned throughout: the 51 expected files under `hearth/tests/render/` unchanged, the world's cases and the library tests' expectations kept.

### 5.3 The subset

Counted by syn in the crate's code outside its tests, at the start and at the landing: generic items 4 and 10, function pointers 3 and 5, closures 15 and 39. The goal added the public generic types `diff::Keyed<T>` and `diff::Changed<T>`, their generic impls, the generic `diff::keyed` and `diff::part` with function-pointer parameters, closures at each part's call, and two `#[expect(clippy::disallowed_methods)]` for `binary_search_by` (`revision.rs` held two before). All of them are out of step code (`skein/docs/foundation/programming-model.md`, 10.3). The contracts document fixed the two generic types and change 2921's `## Steps` asked for them, so the design is where this started. Clippy passes them, and no change ran a code review.

### 5.4 The review

One code review of E2's diff, findings by the kinds of `temper/docs/design/draft/design-economy.md`, 7.3:

| Kind | Count | Findings |
|---|---:|---|
| Repetition | 1 | `parts/secrets.rs`'s own keyed walk, `named`, stays beside `diff::keyed` |
| A concept with no behaviour of its own | 1 | 21 public aliases, each a second name for a shared record |
| A layer crossed for nothing | 1 | `Target` holds `Prepared` whole, so every part is reached through it: `target.prepared.installation.plans` |
| A piece in the wrong place | 3 | a key-ordered join of two lists, written as generics in the domain instead of in skein's lib; sorted lookups through `binary_search_by`, also lib's; production functions only tests call, kept behind `#[cfg(test)]` (`compose::compose`, `exports::evaluate`, `exports::conform`, `parts::config::rendered`) |
| An unclear name | 1 | `entry.value.value.path`: an admitted entry's `value` holds a record whose own field is `value` |

One more, of no listed kind: the repository tree's files now fill the declared installs' fields with fixed values (`owner` and `group` root, no position, no template source), a merge that holds as long as those values stay a rule of their own.

## 6. Cost

From the goal's record, in its unit:

| Phase | Minutes | Spend |
|---|---:|---:|
| Design, to the first change cut (09:12 to 10:38): reading the area with 9 sub-agents, then the contracts and the lean design | 86 | 777.48 |
| Changes, side by side, to the last merged into the goal's branch (to 11:04); the goal agent's share | 26 | 194.01 |
| The six changes' implementers | 5 to 15 each | 278.24 |
| Landing (11:09 to 13:46): three holds on a failed effect, the known reds, a pass up to a person | 157 | 111.73 and the landing's 45.32 |
| The whole goal | 274 | 1,406.78 |

For scale, three goals of the same week on template 4 (2699, 2811, 2833), each carrying a draft's milestone into new or changed code: their goal agents spent 854.65 to 936.69, and cut their first change 20 to 32 minutes after starting. E2's goal agent spent more and started its changes later, the extra time spent understanding 17,666 lines it was to refactor. One goal against three, of another kind of work: an indication, not a measure.

## 7. Reading the figures

Which figures count as success is the user's (`temper/docs/design/draft/design-economy.md`, 10.1). What they show:

- **A lean design reads as well and costs no more detail:** 14% shorter, and no implementer needed anything added.
- **Fewer concepts bought little code here.** The survey proved 16 to 19 lines; the goal cut 407, most of it by folding copies, and the largest part of that by the generics the model rules out. Kept inside the model, as goal 3045 does, some of it comes back as concrete records.
- **The designer is where the subset slipped.** A lean design's contracts name the shared shapes, and they named generic ones. The charters gain a line for it (`temper/docs/design/draft/design-economy.md`, 8).
- **The design did not cost less to write.** Most of the goal agent's spend went into reading the area before it wrote anything.
