# QA pass 1, request 1: 1 findings at 04d309dfe96e

Fix each finding, each fix with a test in the lowest world that reproduces it, starting from the finding's lines. Deliver what the cases expected and nothing past it.

## 1. A milestone takes one goal: a second `goal start --milestone M` is refused 'Authority missing'

Severity: high

Surface: ctl

Page: milestone

Area: temper/draft/qa

Landings:
- goal 4793: Milestones M2 · temper's milestone
- goal 4793: Milestones M2 · temper's milestone

Setup: fresh

Reproduction:

```text
ctl project make Hello --key qa-hello-project
ctl milestone new --project Hello "Second" --as-allowed --key qa-ms-2
ctl goal start --project Hello --milestone 1 "T1 \xc2\xb7 first\x0a\x0aThe first." --charter 2 --deadline 2026-11-18T12:00Z --as-allowed --key qa-ms2-g1
ctl goal start --project Hello --milestone 1 "T2 \xc2\xb7 second\x0a\x0aThe second, no wait." --charter 2 --deadline 2026-11-18T12:00Z --as-allowed --key qa-ms2-g2
```

Expected: the second goal started in the milestone, as goal start starts any goal, its number printed (milestones.md, 1 and 6: a milestone is a set of goals; `goal start --milestone M` starts a goal in it)

Seen: `goal start: Authority missing`, exit 2, for the second goal of each of two milestones, with --after 2 and without it; the first goal of each started ('done: task 2, goal', 'done: task 5, goal'), and a goal with --after alone, in no milestone, started too ('done: task 3, goal'). So milestone show lists one goal, '0 / 1 done'.

Evidence (kept beside the QA ledger):

Earlier goals: none
