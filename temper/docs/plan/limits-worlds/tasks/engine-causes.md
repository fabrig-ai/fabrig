+++
id = "engine-causes"
part = "engine"
kind = "feature"
needs = ["figures-before"]
files = [
  "temper/crates/**",
  "jig/crates/**",
  "smith/crates/**",
  "skein/crates/**",
  "temper/docs/plan/limits-worlds/figures/engine-causes.md",
]
effort = "high"
minutes = 180
+++

# The causes the worlds share, fixed where they are

## Goal

The real loop, the conformance world and the one-process world share an engine, a store, a fake forge and a fake LLM, so one cause may cost in all of them. Take `figures/figures-before.md`'s *Causes, largest first*, and fix those it gives to this task, largest first, where they are, until the next one's gain is under about 2% of a story. No behaviour changes.

Acceptance:

- For each cause you fix: its own commit or commits saying why, a focused test of its own in the package that holds it, and instructions before and after on the story where it was largest and on one other story that shared it, in `temper/docs/plan/limits-worlds/figures/engine-causes.md`, in `notes/measuring.md`'s form.
- For each cause on the list that's yours and you didn't fix: one line saying why, with its share. The stop rule is one reason. "It would change behaviour" and "it isn't small enough to be clearly this plan's" are others.
- The file ends with what you stopped at: the next cause, and its share.
- The focused tests of every package you changed and of its direct dependents pass. Where a change of meaning reaches a reader (a value a shared item returns, an order it keeps), that reader's tests have run too.
- Finding nothing of yours on the list is a result: report `Done:` saying so, with the figures that show it.

## Inputs

- `temper/docs/plan/limits-worlds/notes/common.md`, `temper/docs/plan/limits-worlds/notes/measuring.md`.
- `temper/docs/plan/limits-worlds/figures/figures-before.md`: your list, and your *before*.
- `temper/docs/plan/limits-worlds/inputs/request.md`, its second section and *For every task of this plan*.
- `skein/docs/foundation/testing-strategy.md`, 8 ("A slow test is understood, never hidden", "A test is cheaper by its algorithm first").
- The design section of the code each cause sits in, which you find from the crate's module header, and read before changing it.

## Owns and may change

Owns the code a cause sits in, among the crates in the header, and the figures file. It doesn't own `temper/tests/**`, `jig/tests/**`, `jig/testing/jig-conformance`, `jig/testing/jig-core-kit` or `skein/testing/skein-fork`: those are other tasks' (`notes/common.md`). In the other kits under `*/testing/**`, a cause the list names is yours, and you edit it only for that cause.

May also change:

- **Readers of what you change, in any tree:** call sites and a literal's new field, each edited only for that. Check cross-tree dependents with `heavy cargo check -p <dependent> --tests`, naming each.
- **Goldens, schemas and generated code** a stored format or protocol change makes stale, by their own tools. A stored format or protocol may change freely here, with no compatibility (the request).

Options, each decided:

- **The cause is in a crate another draft holds** (`notes/common.md`, *Beside this plan*): make the hunk as small as the cause lets you, and if a merge meets theirs, keep theirs.
- **The cause is in another task's crates:** `## Left`, with its figures.
- **It's unoptimized hashing, or another hot leaf a test can't avoid:** say so with its share, and leave it. A per-crate build override is the user's, not this plan's (`skein/docs/foundation/testing-strategy.md`, 8).
- **It looks like a cache:** first ask why the work repeats. A cache is the answer only where the work has to happen and its result can't change, and a cache in a step crate keeps within its domain's bounds and memory ceilings.
- **It's a poll where an event would do, in product code:** make the event, only if it shows no difference to what a person, a client or a peer sees. Otherwise list it.
- **The fix needs more than the cause's own module and its callers:** do the part that's clearly small, and list the rest under `## Left` with what it would save.
- **About three hours in, with causes left over 2%:** stop at a clean commit and list them with their shares. `close` carries them to the user.

## Steps

1. Read the list. For each cause of yours, largest first, read the code and the design section before changing anything.
2. Fix it. Write its focused test, and break the rule by hand once, as the implementer's text says.
3. Measure before and after, on the two stories, as `notes/measuring.md` says.
4. Repeat, then the checks in the implementer's text, then `python3 -I -B tools/citations/citations.py --base origin/main`, and commit and push.

## Report

Each cause with its figures, what you stopped at and why, what you left and whose it is, and your implementation and check minutes apart.

## Answers

2026-11-07 (temper/tiers). The first: figures-before completes and this task reruns from its ranked list. Don't take over the profiling. By fab's early-start rule this task runs again only once figures-before has landed, so the baseline will be on your base then.
