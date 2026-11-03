# reference-m2: the goal lands

Carry out milestone **M2** of `temper/docs/design/draft/reference-case.execution.md`. Its entry is the scope, quoted below. Section 2 there says how a milestone runs: design and code go through this plan as one pipeline, and its designers write the design into the trunk, family by family, with the specs that cite it (`DESIGN.md`).

In short: L, the goal's landing: a change into main whose branch is g, its requester's held branch (N3), with g deleted as G's hold is released once landed. Out of scope: main's tiers (M8, on main), reds to the goal (M9).

## The entry

### M2. The goal lands

- **Executes:** `reference-landing.md`, sections 1 to 4 and 7 (L, its
  way into main, what moves g while L lives, its bounds, main's queue,
  a small change); `reference-code-production.md`, section 12 (ready to
  land, L made).
- **Design:** `forge.md`, 8.1 (a change on its requester's held branch)
  and 8.5 (a move by its requester's other delegates is its new head,
  not drift); `forge.md`, section 11 and `after-landing.md`, section 6
  (g deleted as its goal closes if a change landed it, kept otherwise);
  `agent.md`, 4.3.1 (`mine`).
- **Builds:** temper's change procedure on a held branch, and g's
  deletion as G's hold is released.
- **Configures:** the coordinator's charter: L, its escalations, changes
  into g while L is held; the landing rule for main; L's CI, the full
  gate; three repairs.
- **Old code:** temper `o/forgejo-land-drift-world`, a test idea for
  `forge.md`, 8.5.
- **Walk:** steps 23, 24 and 28, L ending at landed until M8; and
  `reference-landing.md`, section 9's second story (L held past its
  repairs, released after a change into g).

## What runs beside it

Wave 2 runs side by side: M2, M3, M5, M6 and M7b, each a plan of its own (`reference-m2`, `-m3`, `-m5`, `-m6`, `-m7b`), with `configuring-p1` and later `configuring-p2`. Several of them add to the same sections (`reference-case.execution.md`, section 4):
- `change`'s parameters in temper's `domain/agent.md`, 4.3.1 (M2 `mine`, M3 a producer's charter and LLM choices, M5 `questions`, M7b the merge job as a producer);
- `domain/forge.md`, 8.1 (M2, M3, M5);
- `protocol/configuration.md`, section 13 (M3, M5, M7b);
- the coordinator's charter (all).

Each adds its own part and leaves the others' alone: a compatible addition, never a rewrite. A designer meeting another plan's text there builds on it as it lands on main and reaches this trunk by catch-up. M1 (the goal's own branch: `change`'s landing branch, dependencies and batch; `forge.md` 2, 8.2, 11 and 11.1; jig's `tasks.md` 6 and 8.3) is on main: build on it.

A budget or timeout is the user's (skein's `docs/foundation/testing-strategy.md`, 8.1): a role never raises one.

## Old code, for reference only

The entry's Old code, if any, lives in hel8's mirrors `~/src/fabrig/attic/<tree>/mirror.git` (`git -C ... show <branch>:<path>`); the assessment is in hel8's `~/src/fabrig/coord/old-repos-value/`. It is a reference, never a plan. Whether to use it, what of it and how is the designers' and implementers' choice, case by case: adapting old code may cost more than writing it fresh. Implementers cannot reach the attic, so whatever a designer takes goes into the specs or notes.

## Size

One cut is at most 90 minutes. Release early, family by family. If the whole cannot be designed in one cut, release what holds together and end with a `Question:` naming the rest.

## Walk

fabrig/qa judges and walks it once it lands, if there is something to walk through `temper ctl`. Mark the slice that makes the entry's story run `walk = true`.
