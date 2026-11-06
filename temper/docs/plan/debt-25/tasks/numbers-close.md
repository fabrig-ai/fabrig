+++
id = "numbers-close"
part = "numbers"
kind = "docs"
needs = ["numbers-testing", "numbers-client", "numbers-accounts", "numbers-people", "numbers-tasks", "numbers-core"]
files = ["jig/docs/development/debt/D035-bare-numbers-in-worlds.md"]
effort = "medium"
minutes = 10
+++

# D035 goes

## Goal

Measure what the six tasks left, and pay the item if the count and a reading both say so.

Acceptance:

- `notes/numbers.md`'s Measure reads at most ten, and each survivor is explained in your report. It should read 2.
- Reading across the worlds, a party, project, task or connector named per file is gone, but for numbers the six tasks' reports kept with a reason: `rg -n 'const (OWNER|CONNECTOR|PROJECT|PARTY): ' jig/tests jig/testing`. List what's left and why each stays.
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md` is deleted. `python3 -I -B tools/citations/citations.py --all` reads 0 after you commit, and `python3 -I -B tools/debt/debt.py | rg D035` reads nothing under jig.
- **If a number or repeat remains that no report explains,** fix it if it's one file and under 20 lines, and keep the item open with its rows rewritten if it's more. Say which in your report's first line.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/numbers.md`

## Owns and may change

- **Owns:** the file in the header, plus the small fix above. No build is needed unless you make that fix.

## Steps

1. The counts, then the deletion. Commit, run the citations tool and push.

## Report

The common note's items; both counts; each number left and why; across D035's tasks, the test lines at this plan's base and now.
