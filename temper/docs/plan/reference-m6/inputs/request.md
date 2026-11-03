# reference-m6: early starts

Carry out milestone **M6** of `temper/docs/design/draft/reference-case.execution.md`. Its entry is the scope, quoted below. Section 2 there says how a milestone runs: design and code go through this plan as one pipeline, and its designers write the design into the trunk, family by family, with the specs that cite it (`DESIGN.md`).

In short: a kept start: a producer starting from g's tip with its dependencies' pushed heads (N2), its once rule, its refusal on a conflict, and pushing as it goes (N14).

## The entry

### M6. Early starts

- **Executes:** `reference-code-production.md`, section 7, and section
  6, step 1's kept start.
- **Design:** `worker.md`, 4.1 and section 5 (a checkout from g's tip
  with its dependencies' pushed heads merged in); `forge.md`, section 8
  (the early start, its once rule, its refusal on a conflict, an
  amendment refused for dropping a dependency started from); `landing.md`,
  4.1 (a dependent started kept); jig's `domain/connectors.md`, 6.1 (a
  procedure that starts kept).
- **Builds:** jig's kept start; temper's change procedure and the
  worker's checkout.
- **Configures:** the coding agent's charter: push as it goes (N14).
- **Walk:** steps 8, 10, 13 and 17; 4's designers kept on S once M3's
  walk stands.

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
