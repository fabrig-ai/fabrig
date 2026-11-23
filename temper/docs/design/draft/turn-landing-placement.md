# turn_landing's two-goal story is placed by what it checks

Draft, 2026-11-17, on the user's word. Goal 5217 added `turn_landing::two_goals_land_in_one_green_turn` to the real loop's focused stories on 11-16 (4180548b36, "Prove turn-landing scripts with a ctl-released two-change batch"). Alone it takes about 13 s. On 11-17 it failed a landing gate (run 1237): the referee found "temper judge overdue" at 19.2 s of the story's 20 s patience. It then passed alone. The earlier repair story's flake had the same shape: a long story running out of patience under the gate's load, with no fault.

`archive/focused-pyramid.md` sets a real-loop story's budget at 2 s, and the real loop's focused stories at 15 s together. It places each slow test only by what it checks: a behaviour goes in the lowest world that owns it (`temper/docs/design/testing/real-loop.md`), and cost never decides the tier. K20 placed the real loop's slow stories that way. This story came after K20.

## 1. Wanted

- The story is placed as K20 placed the others:
  - What it checks of the landing turn, two goals' changes forming one turn and landing on one green gate, moves to the lowest world that owns it: the engine domain world's landing and turn stories (`temper/tests/engine/domain/tests/suite/forge_routes/`, near the turn-landing stories).
  - What only real processes can show stays in the real loop, within the 2 s story budget: the ctl-released batch reaching the engine.
- `temper/docs/design/testing/real-loop.md` names the story's path, as K20's entries do. `archive/focused-pyramid.status.md` records the placement.
- No patience, budget or figure changes.

## 2. Out

- The other real-loop stories.

## 3. Tests

Cheap: focused and fuzzy suites only; walk = false.
- The moved checks pass in the engine domain world, by name.
- What stays in the real loop runs within 2 s alone, by name.

Run only the new and affected tests, by name.
