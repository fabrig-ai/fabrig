# reference-m10: plan

M10 of `temper/docs/design/draft/reference-case.execution.md`, the case: what is left of the
reference drafts once M1 to M9 are on main. `reference-code-production.md`, section 14 whole,
as one reference profile a box is set up with, its limits sized for a goal of a hundred
changes (N21); `.temper/pre-pr` revisited with numbers; the drafts' statuses left holding only
what this plan cannot close. Written by the cut as it releases; each batch brings it up to
date.

## The design this cut wrote

- `temper/docs/design/protocol/configuration.md`, 13.9: "Charter 5's plan done, and its
  landing's words" (the plan done with its goal branch's tier settled; the landing's words
  naming the draft); charter 2 with the shell, running its changed packages' lints and focused
  tests; template 4's citation (jig's 3.3). Section 11: the numbers on `.temper/pre-pr`
  (`temper/docs/development/pre-pr-numbers.md`) and what the case's walk measures. Its status:
  Not yet for charter 5's text and for kind 3's number 3; the pre-pr Departs kept, its reason
  the numbers.
- `temper/docs/design/domain/agent.md`, 4.3.1: a change's tasks bounded by
  `forge.change_policy.tasks`; section 10: a change produced by a job of any kind, an open
  question (`producer` and `merge` cover the reference). Its status: Not yet.
- `temper/docs/design/domain/forge.md`, 8.11: the next producer of a change into a goal's own
  branch merges the branch's tip first. Its status: Not yet.
- `temper/docs/design/client/tools.md`, 5.3: a template's `resources` in a settings file. Its
  status: Not yet.
- `temper/docs/design/protocol/configuration.md`, 13.9 (batch 3): "A triage delegate" (charter
  8, read-only; charter 5 may delegate a red's triage to it; designers' changes `"questions":
  "up"`); "The profile, whole": `docs/examples/reference/` as one profile (`limits.json`,
  `agent.json`, `landing.json`, `settings.json` and the repository's files), its table of
  limits sized for a hundred changes (N21), the settings sized to it, the rules' and policy
  1's delegation 1,024 tasks 4 deep; "Checked as it is read" extended; the landing rules'
  bullet brought current (kind 1's items since 13.6's second step); the charters' and
  executors' count moved to the table. 13.4: the false claim that jig's settings schema caps
  `tasks.result_bytes` at 512 removed. Its status: Not yet for the profile's files and charter
  5's routes, batches and triage.
- jig's `jig/docs/design/domain/connectors.status.md`: the Not yet for 6.2's room wait, which
  the design already had.
- Closed in the drafts' statuses: `reference-code-production.status.md`'s "12: the plan done
  with g's tier settled", "3, 5, 14: `change` and its fields" and "5, 7, 8: a producer's next
  run once its dependencies move", "4, 14: the profile's limits"; `reference-landing.status.md`'s
  "3, step 2: L's body naming the draft" and "6: a triage delegate" (it now has no gap);
  `reference-case.status.md`'s "11" (g's tip merged first) and "22" (the close-out waiting
  for g's tier).

## What the drafts' statuses still hold

- `reference-code-production.status.md`: the Departs on a red of g's tier waking G, decided
  by reference-m9's cut; it goes with the drafts' close-out, which folds them into the design.
- `reference-case.status.md`: N22's fields and N19, N10, N1 and N13: the design carries them
  since reference-m3 and reference-m5 landed; their retires, not yet run, remove them, and
  this plan's retire removes any left.
- `.temper/pre-pr`'s decision is configuration's own (section 11 and its status's Departs),
  not a draft's: the record so far does not decide it (`temper/docs/development/pre-pr-numbers.md`),
  the first edition stays, and the case's walk measures what decides it.

The standing rules are `notes/common.md`.

## Parts and tasks

| Part | Task | Needs | State |
|---|---|---|---|
| charter | `charter-close`: charter 5's close-out waits for its tier settled; its landing's words name the draft | | released |
| checks | `charter-checks`: charter 2 with the shell runs clippy and the changed packages' focused tests | | released |
| change-tasks | `change-tasks-limit`: `forge.change_policy.tasks`, 6 in small | | released |
| next-run | `next-run-tip`: a goal branch's change's next producer merges g's tip first | | released |
| template | `template-resources`: a settings file's template carries `resources` | | released |
| room | `room-wait` (jig): a procedure's batch refused for live room waits for it | | released |
| triager | `charter-triage`: charter 8; charter 5 delegates a triage, designers ask up | charter-close | released |
| profile | `profile-limits`: `limits.json`, `agent.json`, the settings sized, the admission and fit tests | charter-triage, charter-checks, change-tasks-limit, template-resources | released |

Slices: 1 (charter), 2 (checks, change-tasks, next-run, template, room), 3 (triager, profile:
the walk's, `walk = true`). One question, `case-workflows`: the case's repository's CI and tier
workflows, which fabrig does not carry; nothing waits on it.

## Critical path, as it stands

`template-resources` (90) or `change-tasks-limit` (75) → `profile-limits` (150): about four
implementer-hours to the walk's slice, `charter-close` → `charter-triage` (40) beside it.
`room-wait` (240) runs alongside and is the longest single task; the walk needs it only where
live tasks bind.

## Overlaps with the plans beside this one

- `temper/docs/design/protocol/configuration.md`: configuring-p2 (integration) rewrites
  section 13's opening, 13.1 and 13.7; this plan writes 13.9 (and, later, section 7 and 13.4's
  box limits), not theirs.
- `temper/docs/design/client/tools.md`, 5.3: configuring-p2 adds the settings file's
  project-policy item there; this plan adds one clause on a template's `resources`.
- `temper/docs/design/draft/*.status.md`: reference-m3's and reference-m5's retires, not yet
  run, close their own entries in the same three drafts' statuses (N22's fields, N19, N10, N1,
  N13 in `reference-case.status.md`); this plan removes only the entries it closes.
