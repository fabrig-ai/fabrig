# How temper builds temper

Kind: way

How fabrig builds temper, with temper: one way among many. Start from it, from something smaller, or from your own words.

## 1. In one picture

1. **Design draft:** you write it
2. **The cut:** designers, early parts first
3. **Coding agents:** parts in parallel
4. **Goal’s branch:** checked, caught up from main
5. **Merge queue:** the goal lands whole
6. **main:** QA walks it

## 2. A pipeline that keeps growing

You write a design draft and start a goal from it. Designers cut it into parts and send the first ones out at once, so coding agents start while the larger parts are still being designed; a part whose dependency is still being made starts early, on that dependency’s pushed work.

## 3. In parallel, kept together

- Coding agents make parts in parallel, each in a slot on a worker. Add a worker or a slot to go faster; each worker keeps its checkouts cached.
- Every part lands into the goal’s own branch, whose tests run along the way; main’s landings come in by catch-ups, so a goal never drifts far.
- Questions go up only as far as they must: the goal’s agent, then you.

## 4. Landing many

Parts land into their goals’ branches many an hour; a goal done lands into main whole, through the merge queue, each candidate proved once. After each landing, QA agents try what a person can now do on the newest main, and file what they find.

## 5. temper itself, and yours

| | temper itself | Yours to set |
|---|---|---|
| **Plans** | A goal per draft, which designers cut into parts, the first parts out at once | From one chat per change to goals with plans |
| **Models** | Opus 5.5 to design and coordinate, gpt-6.1-sol to code | Any plan or model, for each role |
| **Workers** | Several machines, several slots each | The deployment’s configuration |
| **Branches** | A branch per goal, caught up from main | From straight to main to a branch per goal |
| **Landing** | Each part into its goal’s branch, many an hour; the goal into main whole | From one change at a time to whole goals |
| **QA** | Agents walk main after each landing | Later |
| **Questions** | The goal’s agent, then you | From everything to you to only what agents cannot settle |

Each row is a setting, unless it says otherwise. Tell the chat what to keep, and what to change.
