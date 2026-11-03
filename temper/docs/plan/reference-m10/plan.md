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
- jig's `jig/docs/design/domain/connectors.status.md`: the Not yet for 6.2's room wait, which
  the design already had.
- Closed in the drafts' statuses: `reference-code-production.status.md`'s "12: the plan done
  with g's tier settled", "3, 5, 14: `change` and its fields" and "5, 7, 8: a producer's next
  run once its dependencies move"; `reference-landing.status.md`'s "3, step 2: L's body naming
  the draft".

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

## Critical path, as it stands

`room-wait` (240) is the longest; the profile's batch (the reference's limits as one profile,
its turns and sizes, template 4's goal branch, the triage delegate) follows `charter-close`,
`charter-checks`, `change-tasks-limit` and `template-resources`, and is the walk's slice.

## Overlaps with the plans beside this one

- `temper/docs/design/protocol/configuration.md`: configuring-p2 (integration) rewrites
  section 13's opening, 13.1 and 13.7; this plan writes 13.9 (and, later, section 7 and 13.4's
  box limits), not theirs.
- `temper/docs/design/client/tools.md`, 5.3: configuring-p2 adds the settings file's
  project-policy item there; this plan adds one clause on a template's `resources`.
- `temper/docs/design/draft/*.status.md`: reference-m3's and reference-m5's retires, not yet
  run, close their own entries in the same three drafts' statuses (N22's fields, N19, N10, N1,
  N13 in `reference-case.status.md`); this plan removes only the entries it closes.
