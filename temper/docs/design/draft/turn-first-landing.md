# A turn landing that never runs, and goal start's model flags

Draft, 2026-11-13. Two faults seen on a deployment's first real landing
under `by: turn` (`../domain/landing.md`). Each wants its cause found, then
fixed, with a focused test that reproduces it first.

## 1. The first landing under turn never runs

- **Seen:** a project whose landing entry is `by: turn`, batch 1, with one
  gate. A goal's change was approved and delivered, and its pull request
  opened. The landing was made and has sat at `preparing`, round 1, no
  candidate, for over 40 minutes: its procedure task (`landing`, made with
  a budget of zero, `../domain/landing.md`, 4) is due to run, with no
  run since, no live delegate, and no prepare job ever delegated
  (`../domain/landing.md`, 5). The project's landings report `turn: free`.
  An engine restart only moved its attempt on; it stayed due. Nothing in
  the engine's or the worker's log names it.
- **Questions to answer first:** why a due procedure task with a budget of
  zero is never run (the scheduler, a missing grant, the procedure's own
  admission), and whether a project's configuration can cause it.
- **Wanted:** the landing's procedure runs as soon as it's due and
  delegates its round's prepare job; if something it needs is missing, it
  is held with words naming what, never left due and silent. A focused
  test makes a turn landing for a delivered change and sees its prepare
  job delegated.

## 2. goal start ignores --model, --effort and --sub-agent

- **Seen:** `temper ctl DIR goal start --project P --template T
  --as-allowed --model M --effort E --sub-agent S:EFFORT TEXT --key K`
  made a goal whose main model and sub-agent were the template's
  (`claude-opus-5-5/high`, `gpt-6.1-sol/high`), not the flags'.
- **Wanted:** the flags reach the goal, or, if a template or the party's
  allowance overrides them, the start says so and refuses or names what it
  used. A focused test starts a goal with each flag and reads them back
  with `task show`.

## 3. Out

- The landing's design; batches past 1; the gate's own workflow.

## 4. Tests

Cheap, focused and fuzzy suites only; walk = false. Run only the new and
affected tests by name.

## 5. Formats

If a fix needs a field no store keeps today, the store's version moves
and its patch comes with it: upgrades are supported (`redeploys.md`).
