# reference-m5: questions, and holds passed up

Carry out milestone **M5** of `temper/docs/design/draft/reference-case.execution.md`. Its entry is the scope, quoted below. Section 2 there says how a milestone runs: design and code go through this plan as one pipeline, and its designers write the design into the trunk, family by family, with the specs that cite it (`DESIGN.md`).

In short: a change relaying its agents' questions through a foreman or straight up (N1), its bound, an answer held until a dependency lands, and holds passed up to the person. Out of scope: designers' charters (M3).

## The entry

### M5. Questions, and holds passed up

- **Executes:** `reference-code-production.md`, sections 8 (a change
  relays its agents' questions) and 9 (holds escalated to G and passed
  up to the person), and 13's wakes on them.
- **Design:** `agent.md`, 8.2 and `forge.md`, 8.1 (a change's questions;
  the Message family for its producer, repairs and resolutions);
  `agent.md`, 4.3.1 (`questions`); temper's `../protocol/configuration.md`,
  13.2 (the foreman's charter) and section 13 (the `questions` setting
  and its bound).
- **Builds:** temper's change procedure: the relay, a foreman per
  question, the bound, an answer held until a dependency lands; a
  change's open questions in `ctl`.
- **Configures:** the foreman's charter; the coordinator's charter:
  answer, amend, a new change, ask the person, holds; waking on each
  question at once (N8).
- **Old code:** `charter-tools-r6` (M3), for the read-only foreman;
  `holds/forge-step-admitted-12` (M1).
- **Walk:** steps 9, 11, 14, 15 and 18; 5's question once M3's walk
  stands.

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
