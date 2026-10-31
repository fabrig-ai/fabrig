# Code-quality review of skein: shared brief

Tree: `skein (at 3371027)`
(a fresh clone of github origin/main at 337102750). Read only; do not build or run tests.
Work from this clone only; never touch `~/src/fabrig/*`.

## What is being asked

Not a bug hunt. The user wants an assessment of **code quality** of production code
**and tests (tests are code and are judged the same way)** on these criteria:

1. **Cleanliness**: readable, no dead code, no noise, comments that earn their place.
2. **Organization**: module/file layout, where things live, file sizes, cohesion.
3. **Taste**: idiom, judgement, does it look like it was written by someone who cares.
4. **Naming**: precise, consistent across the area, matches the design's vocabulary.
5. **Primitives**: orthogonality and decoupling. Are the abstractions the right ones,
   each doing one thing, composable, with narrow interfaces? Leaky or overlapping concepts?
6. **Elegance**: does the shape of the code follow the shape of the problem?
7. **Economy**: could the code be smaller? Duplication, near-copies, boilerplate,
   over-long tests, fixtures retyped, parallel hierarchies. Estimate the lines that
   could go (e.g. "~600 of 2,100 lines in tests/x.rs").
8. **Complexity**: is it just complex enough for what it needs, and no more?
   Speculative generality, layers that only forward, state that could be derived.

## Calibrate to the house rules (important)

skein deliberately writes a small Rust: read `docs/foundation/programming-model.md`
sections 1, 3, 4, 5.4 and 10 (no std in step crates, no closures/traits/generics/dyn,
exhaustive matches with no `_`, no indexing, no `as`, checked arithmetic, one handler
per state×event cell, the `mem::replace` transition idiom, free functions over fields,
`Writer`/`Reader`, slabs and `Id<T>`) and `docs/foundation/testing-strategy.md`
section 10 (a test's shape: one behaviour per test, rows carry data, the setup the
assertion reads, a setup written once, named fixtures and sizes, a bound's two sides).

- Do **not** flag verbosity that the rules force (e.g. a long exhaustive match) as
  uneconomical. Judge economy *within* the rules: given these rules, is this the
  smallest clear form?
- Do flag where code breaks its own rules in ways that hurt quality (e.g. tests that
  retype setups, bare numbers, branching on row labels).
- If a rule itself costs a lot of code in your area, note it once, separately, with a
  concrete example and rough cost. Low priority.
- Comments in this repo use a particular plain prose style; don't flag the style itself.
- Test crates (`tests/*`, `testing/*`, `src/tests*`) are ordinary Rust (may unwrap,
  index, use closures) — judge them as ordinary Rust held to section 10.

## How to work

- You cannot read everything. Map the area first (module trees, file sizes, public
  surface, how tests are organized), then read closely where quality matters most:
  the core types and state machines, the biggest files, the test support/fixtures,
  and a sample of tests. Say what you read closely and what you only skimmed.
- Prefer **patterns** (a thing done 20 times) over one-off nits. A finding is worth
  reporting if fixing it would make the code noticeably smaller, simpler or clearer.
- Verify every finding against the source; quote file:line. No speculation reported
  as fact; mark uncertain items "(unsure)".
- Look across crate boundaries inside your area: duplicated helpers, overlapping
  types, a test harness reinvented per test crate.

## Report (your final message; ≤ ~1,800 words)

1. **Scope read**: closely / skimmed.
2. **Scores**: 1–5 for each of the 8 criteria, one line of justification each.
3. **Strengths**: 2–4 bullets (what to keep doing; be specific).
4. **Findings**, ranked by payoff, numbered. Each: title; criterion; evidence (file:line,
   several sites for a pattern); why it matters; concrete fix; estimated size effect.
5. **Cross-area suspicions**: things you noticed that likely recur outside your area
   or involve crates outside it (name them), unverified beyond your area.
6. **Rule costs** (optional, short).
