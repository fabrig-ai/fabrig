# reference-m1: the goal's own branch

Carry out milestone **M1** of `temper/docs/design/draft/reference-case.execution.md`. Its entry in section 3 is the scope: Executes, both Design bullets, Builds, Configures, Old code and Walk. Its section 2 says how a milestone runs: its design and its code go through this plan as one pipeline, and its designers write the design into the trunk, family by family, with the specs that cite it (the repository's `DESIGN.md`).

## The scope, in short

- **The goal's own branch g** (`temper/docs/design/draft/notes.md`, N3), from `reference-code-production.md`, sections 1, 3, 4, 6 and 13. The landing half (L, `mine`, g's deletion on release) is M2's, not this plan's.
- **Two families of design:**
  - jig's: a template naming *its own goal branch*; the effects of a hold's taking and release; a held resource a landing queue orders; writer slots taken by what a run writes; a charter's default wake policy (N23); a result's bound past the settings schema's cap.
  - temper's: g as the goal's own, a change from outside G's subtree refused; `change`'s landing branch, dependencies and batch (N22 in part); checking and updating by the landing branch's rule (N4); a result carrying the producer's report (N15).
- **Builds:** the landing rules as settings of temper's kinds, out of `Config`'s `landing` member, so the rule for goal branches stands beside main's. This is designed (`temper/docs/design/domain/root.md`, 5.5, 5.6 and 6; `forge.md`, 8.9; `protocol/configuration.md`, 13.2 and 13.3) and not built.
- **Configures:** the entry's charters, template and rules, in their first editions.
- **Out of scope:**
  - M2: L into main;
  - M3: designers as changes, LLM choices per role, the forge's charters as settings;
  - M5: questions;
  - M6: early starts;
  - M7: the merge job.
  
  Build `change`'s parameters so that each of those milestones adds its own, without designing them here.

## Size

This is the widest milestone so far: two trees of design and the settings. One cut is 90 minutes at most, so release early, family by family. If the whole cannot be designed in one cut, design and release what holds together. End with a `Question:` naming what is left, rather than squeezing it.

## Old code, for reference only

The entry's Old code lists four branches, in hel8's mirrors at `~/src/fabrig/attic/<tree>/mirror.git` (`git -C ... show <branch>:<path>`); the assessment is in hel8's `~/src/fabrig/coord/old-repos-value/`. They are a reference, never a plan. Whether to use one, what of it and how is the designers' and implementers' choice, case by case: adapting old code may cost more than writing it fresh. Implementers cannot reach the attic, so what a designer takes goes into the specs or notes.

One branch names a bug the walk may meet: temper `holds/forge-step-admitted-12` (a change held for effects, then released, is left with no producer). Check whether main still has it before relying on it.

## Walk

fabrig/qa walks M1 as a rung once it lands: steps 1, 3 (G specifies its changes itself), 7 and 19 of `reference-case.md`, then the goal's end with g kept, unlanded. Mark the slice that makes this story run `walk = true`.
