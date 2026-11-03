# reference-m3: designers and the cut

Carry out milestone **M3** of `temper/docs/design/draft/reference-case.execution.md`. Its entry is the scope, quoted below. Section 2 there says how a milestone runs: design and code go through this plan as one pipeline, and its designers write the design into the trunk, family by family, with the specs that cite it (`DESIGN.md`).

In short: designers as changes' producers (N13), an LLM choice per role (N19), temper's kinds as settings on jig's application settings (the landing rules moved here from M1, with the forge's charters), and M4's sub-agents made reachable from temper. A charter's documents come from `configuring-p1` (`configuring.execution.md`, P1): M3's charters naming documents wait for it to land, so release them last.

## The entry

### M3. Designers and the cut

- **Executes:** `reference-code-production.md`, sections 2 and 5 (the
  cut, designers as changes, a family's batch as its design lands, a
  spec, a revision of the design) and 12 (the close-out).
- **Design:** `agent.md`, 4.3.1 and `forge.md`, 8.1 and 8.9 (`change`
  naming its producer's charter and an LLM choice per role); temper's
  `../protocol/configuration.md`, 13.2 (the forge's charters); jig's
  `domain/tasks.md`, section 4, jig's `protocol/configuration.md`, 3.1
  and jig's `domain/engine.md`, 17.4 (a batch's LLM choices checked
  against its charter's models); jig's `domain/core.md`, 7.1 (a plan's
  text in the repository's design).
- **Builds:** temper's `change` and change procedure (a producer's
  charter, an LLM choice per role); jig's check of them; temper's
  kinds as settings, on jig's application settings (jig's
  `domain/engine.md`, 17.5 to 17.7), built once for both: the landing
  rules, out of `Config`'s `landing` member (`../domain/root.md`, 5.5,
  5.6 and 6; `forge.md`, 8.9; temper's `../protocol/configuration.md`,
  13.2 and 13.3), and the forge's charters, out of `Config`'s
  `forge_charter`, so a change's producer and foreman each take a
  charter of their own (`../domain/root.md`, 6; `forge.md`, 8.1 and 8.9;
  temper's `../protocol/configuration.md`, 13.2), moved here from M1's
  cut (the user, 2026-11-02); M4's sub-agents
  reachable from temper (fabrig/qa, 11-02): the box's smith profile
  declares one conversation, so every `sub_agent` is refused as too
  many, and `ctl`'s LLM choice for a task names no sub-agents.
- **Configures:** the designer's charter; the coordinator's charter:
  the cut, the families' batches, revisions, the close-out; each
  change's LLM choices, gpt-6.1-sol on smith's direct backend (N20);
  sub-agents in both charters, at Opus 5.5 and high.
- **A charter's documents** are built by `configuring.execution.md`'s
  P1 (plan `configuring-p1`, started 2026-11-02), not here: M3's
  charters naming documents (`AGENTS.md`, the design rules) wait for P1
  to land.
- **Old code:** jig's and temper's `configuring-revisions/charter-tools-r6`
  (the tool families a charter offers); `forge-kinds` (M1).
- **Walk:** steps 1 to 6 without questions or early starts (each
  designer starts once S has landed), and 22; with M2, the goal then
  lands. M4's walk folds in here (fabrig/qa): a Claude Code chat asks
  two inspect-only sub-agents to read two parts of the repository side
  by side; both run at their own LLM choice, inspect only, seen through
  `ctl`, each child's session ended on its budget.

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
