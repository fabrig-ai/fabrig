# QA pass 40, request 1: 1 findings at f6108d2f965e

Fix each finding, each fix with a test in the lowest world that reproduces it, starting from the finding's lines. Deliver what the cases expected and nothing past it.

## 9. task show's waiting on: line says Nothing awaited for a goal waiting on its --after dependency

Severity: low

Surface: ctl

Page: waits

Area: ctl task show

Landings:
- goal 15521: One list of every wait in a project, nearest bound first

Setup: fresh

Reproduction:

```text
ctl project make Hello --key w-hello
ctl goal start --project Hello --charter 2 --deadline 2026-12-30T12:00Z --budget 1 --key w-g1 "S1 \xc2\xb7 history\x0a\x0aThe first."
ctl goal start --project Hello --after 1 --charter 2 --deadline 2026-12-30T12:00Z --budget 1 --key w-g2 "S2 \xc2\xb7 indexes\x0a\x0aThe second."
ctl task show 2
```

Expected: S2's line gives its first wait, the dependency on task 1 (S1), with its since and bound, as bounded-waits.md 9 promises ('A task's line gives its first wait with its since and its nearest bound'), matching the waits read's line '2 S2 · indexes · dependency task 1 · since 2026-11-26 08:08 UTC · holds at 2026-12-30 12:00 UTC · hears you'

Seen: 'waiting on: Nothing awaited · since 2026-11-26 08:08 UTC · holds at 2026-12-30 12:00 UTC': the line says nothing is awaited yet gives the dependency wait's since and bound; the waits: 1 line below names 'dependency task 1', and --json has waits_on "nothing" beside wait {kind dependency, object task 1}. The since and bound do match the waits line.

Evidence (kept beside the QA ledger):
- evidence/9/task-show-2.txt
- evidence/9/waits.txt
- evidence/9/task-show-2.json.txt

Earlier goals: none

## Recipes

- temper/docs/development/qa/setups/waits-hello/
