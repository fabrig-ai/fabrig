# reference-m7b: the merge job and catching up (M7b)

Carry out milestone **M7, its part M7b** of `temper/docs/design/draft/reference-case.execution.md`. Its entry is the scope, quoted below. Section 2 there says how a milestone runs: design and code go through this plan as one pipeline, and its designers write the design into the trunk, family by family, with the specs that cite it (`DESIGN.md`).

In short: M7b only: the merge job (N7) as a kind of job (jobs are on main from M7a), its three uses (a merge into g refused for a conflict, a catch-up change into g (N6), L's update when M2 has landed), and a role's authority naming a job kind in jig's configuration (M7a's retire found it missing). L's update needs M2: design it, and leave its wiring for when M2 lands if it has not.

## The entry

### M7. Jobs, the merge job, catching up

- **Executes:** `reference-code-production.md`, sections 6, step 5 (a
  merge into g refused for a conflict) and 11 (catching up);
  `reference-landing.md`, sections 3, step 6 (L's update in conflict)
  and 5.
- **Design:** jobs are designed and, by M7a, built (jig's
  `domain/core.md`, 3.2, `domain/tasks.md`, 5.3.1, `domain/engine.md`,
  7.5 and `domain/hosts.md`, 6.9; `worker.md`, 4.4); temper's worker
  declares no kind yet. The merge job is not
  designed: `landing.md`, 5.3, `forge.md`, 8.4 and section 10, temper's
  `../protocol/tooling.md`, sections 1 and 4, and
  `../protocol/configuration.md`, section 13 (`merge_job`); nor a
  catch-up change into g (`forge.md`, section 10 and 11.1), the merge
  job its producer (`agent.md`, 4.3.1).
- **Builds, in two parts:** M7a, jobs in jig and temper's worker, from
  M0, as already designed; M7b, the merge job and its three uses: a
  merge into g and a catch-up (after M1), L's update (after M2). M7b
  also lets a role's authority name a job kind in jig's configuration,
  which M7a's retire found missing (jig's `protocol/configuration.md`,
  section 3).
- **Configures:** `merge_job`, the project's kind that merges and
  recounts its counted baselines; the coordinator's charter:
  catch-ups, landing news paced at most once in 30 minutes (N23).
- **Old code:** temper `ladder-b01/f19-r4`, the resolving agent's words
  (the worker pushes the merge commit).
- **Walk:** steps 12, 16 and 21; `reference-landing.md`, section 9's
  first story (L's update conflicts, settled by the merge job and a
  resolution on g).

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
