# Fix two ways a goal stops on its own

Draft, 2026-11-14. On a deployment, on 11-14, goals stopped twice in ways nothing woke them from. Each needed a person. Both bugs are temper's.

## 1. A goal's run ends with relayed questions unanswered, and nothing wakes it (P43)

- **What happened:** Goal 2937 (tier-controls) received two producers' questions, relayed up by their changes:
  - 2943's, question 186113 (skein-codegen refuses the v24 `TierProbe`);
  - 2957's, question 186129.
  The goal's live run read both and answered a third (2961's, 186127). It ended at 11:22Z with a `wait`, leaving both unanswered, and stayed idle for an hour, with `questions: true` in its wake. A person's message at 12:20Z woke it, and it answered both within minutes.
- **Wanted:** a task whose run ends while a question addressed to it is still unanswered is due again. A question isn't settled by being read in a run: until it's answered, or the task says why it won't answer, it wakes the task, as its wake policy's `questions` says.
  - Find where a question that arrives during a live run is marked delivered or seen, so that the run's end doesn't wake the task again (jig's wake and inbox: `jig/crates/jig-domain-tasks/src/wake.rs`, `inbox.rs`).
  - Fix it there, or at the run's end.
  - If the design says otherwise, cite it, and say what keeps such a goal from stopping.

## 2. A repair with nothing to push holds its change for good (P44)

- **What happened, twice:**
  - Change 2951 (goal 2893, into main) failed CI. Its repair ran both failed tests alone, found that both pass, and pushed nothing. The change was then held with "An effect failed permanently" at repairing.
  - Change 2984 (goal 2945) failed CI with "starved or faulted with no failing test" twice. Its repair was told to finish `no_change`, and the change was held the same way.
  - Releasing the hold by hand let 2951 update and check again.
- **Wanted:** a repair that ends with no change (flakes, or a red that names no failing test) is not a permanent failure.
  - The change checks again: it reruns CI at the same head where the forge can, or updates onto its base and checks the new head.
  - It counts that against its repairs, so a red that keeps coming back still ends in a hold, by its bound, with words that say so.
  - Find the effect that failed and why it counts as permanent (temper's change domain: `temper/crates/temper-engine-domain-forge-change/`, the repairing step, and its route in `temper/crates/temper-engine-domain/src/route/forge_route/`).
  - `temper/docs/design/domain/forge/change/repairs.md` says how a runner's failure within `reruns` is run again. Make a repair's empty result fit that, and update the design where it's silent.

## 3. Out

- How the runner shares its machine with other work (operations).
- Holding answers to a change kept by its dependencies (`questions.md`, 1): that's a separate decision.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- For 1, a focused test in jig's tasks domain: a run that reads a question and ends without answering it leaves the task due.
- For 2, a focused story in temper's engine domain tests (`temper/tests/engine/domain/tests/suite/forge_routes/`):
  - a repair that delivers no change gets its change checked again, not held;
  - repeated empty repairs end in a hold at the repair bound.

Run only the new and affected tests, by name.
