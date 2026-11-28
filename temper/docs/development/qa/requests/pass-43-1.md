# QA pass 43, request 1: 1 findings at c22df0a5b4bb

Fix each finding, each fix with a test in the lowest world that reproduces it, starting from the finding's lines. Deliver what the cases expected and nothing past it.

## 11. A change waiting at its step reads 'waiting on: Nothing awaited · since … · holds at …'

Severity: low

Surface: ctl

Page: task

Area: ctl task show

Landings:
- goal 15518: temper's changes, landings and closers end their waits

Setup: temper/docs/development/qa/setups/turn-landing/

Reproduction:

```text
forge push forgejo work/hello qa/turn-a "Write turn-a.txt" "turn-a.txt"
ctl change open --project Hello --repository work/hello --branch qa/turn-a --title "Write turn-a.txt" --budget 400 --key open-a
ctl change show 4
ctl task message 5 "Approve now" --key approve-a
ctl task show 5 --until ended --timeout 300s
ctl task show 4
```

Expected: A change waiting at a step of its procedure has its waiting on: line say, in words, that it waits, with since and its bound (client/ctl/results.md, 1: a held task's line prints held 'where Nothing awaited would read as if nothing were wrong'; landing 415: changes end their waits in words)

Seen: With its review approved and its pull request at ready, change 4's header reads 'Waiting for news', its 'waits: 1' lists 'step at ready: pull request #2 · since 2026-11-26 21:38 UTC · holds at 2026-11-26 22:08 UTC · hears you', yet its line reads 'waiting on: Nothing awaited · since 2026-11-26 21:38 UTC · holds at 2026-11-26 22:08 UTC': nothing awaited, with a since and a hold. --json has waits_on "nothing" beside wait {kind step, step 108} and nearest. Change 6 read the same. Same family as finding 9 (a goal waiting on its --after dependency reads Nothing awaited).

Evidence (kept beside the QA ledger):
- evidence/11/change-4-at-ready.txt

Earlier goals: none

## Recipes

- temper/docs/development/qa/setups/turn-a-started/
