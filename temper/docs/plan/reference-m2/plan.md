# reference-m2: plan

Milestone M2 of `temper/docs/design/draft/reference-case.execution.md`: the goal lands. L, a
change into main whose branch is g, its requester's held branch (N3): `change`'s `"branch":
"mine"`; a move of g by G's other delegates is L's new head, never drift; g deleted as G
closes once a change landed it, kept otherwise; the coordinator's charter lands the goal and
handles L's holds; main's rule asks CI alone. Written by the cut as it releases; each batch
brings it up to date.

## Design written by this cut

The lead wrote it (one writer keeps up: four sections' additions):

- `temper/docs/design/domain/agent.md`, 4.3.1: `branch` (`"mine"`), its refusals.
- `temper/docs/design/domain/forge.md`, 8.1 (the branch parameter's third form; "A change on
  its requester's held branch"), 8.5 ("On its requester's held branch"), 11 (such a change
  deletes nothing) and 11.1 (landed whole by it).
- `temper/docs/design/domain/after-landing.md`, section 6 (the goal's landing holds none).
- Status: **Not yet** entries in `forge.status.md` (8.1, 8.5, 11), `agent.status.md` (4.3.1)
  and `after-landing.status.md` (6); the drafts' **Design lags** closed:
  `reference-landing.status.md`'s "1, 2, 3: L on its requester's held branch", and the
  requester's branch in `reference-code-production.status.md`'s first entry.

forge.md 11's deletion of g once a change landed it was already designed (M1); its **Not
yet** is task `goal-release`'s.

## Parts and tasks

| Part | Tasks (needs) |
|---|---|
| mine | `mine-call` [] (the call's shape, C1); `mine-procedure` [] (the procedure's new head, C3); `mine-start` [mine-call] (the root's `mine`, C4, and L's first story: g landed into main whole); `mine-forge` [mine-procedure, goal-release] (the forge's side, C5); `mine-help` [mine-call] (the change page's `branch`, within its 8,192-byte bound); `mine-story-moves` [mine-start, mine-forge, goal-release] (a change into g is L's new head; drift by another identity; a stale read; G's close deleting or keeping g); `mine-story-held` [mine-story-moves] (`reference-landing.md`, section 9's second story) |
| release | `goal-release` [] (C2) |
| reference | `reference-landing-config` [] (`notes/settings.md`) |

The cross-task shapes are `notes/contracts.md` (C1 to C5); the standing rules,
`notes/common.md`.

Every task of the milestone is released (two batches). The old branch
`o/forgejo-land-drift-world`'s idea (a branch read sent before a move and answered after it
holds nothing) is carried into `mine-story-moves` for g.

## Slices

1. release; 2. mine and reference, **the walk** (`walk = true`): the case's steps 23, 24 and
   28, L living to cleared (M8 has landed), and `reference-landing.md`, section 9's second
   story.

## Decisions

- **`mine` is the requester's goal branch,** not any branch it holds: the only held branch a
  requester has that a change could land is its own goal branch (11.1).
- **No store change:** the change on a held branch is spec parameter 11 (jig keeps the spec);
  the forge tells it by the branch's hold being another task's. g's landing is read from the
  change rows the forge keeps (a landed change on g whose merged head is g's last known tip):
  no new read, field or effect.
- **The new head is the procedure's,** one rule in `temper-engine-domain-forge-change`
  behind `Facts.requester_branch`; the forge supplies the fact and routes drift on g to L
  too.
- **Three repairs** are the stock's `forge.change_policy.repairs`; **L's CI, the full
  gate,** is the repository's CI at L's head, which main's rule asks (`ci: true`): nothing
  to build; the walk's repository supplies its CI.

## Order decided by the lead

One writer per crate at a time: `goal-release` then `mine-forge` in
`temper-engine-domain-forge`; `mine-call` then `mine-start` in the root's `forge_route.rs`;
the two stories one after the other, since both may fix what they find in the plan's code;
`mine-procedure` alone in `temper-engine-domain-forge-change` (its literal-only lines in the
forge and landing crates run beside `goal-release`: field-only hunks).

## Critical path, as it stands

`mine-procedure` (75) -> `mine-forge` (90) -> `mine-story-moves` (110) -> `mine-story-held`
(90), about 365 implementer-minutes; beside it, as long, `mine-call` (45) -> `mine-start`
(120) -> the same two stories. `goal-release`, `reference-landing-config` and `mine-help` are
off the path.

## Overlaps with other plans

- forgejo-connect (integration) changes `temper-engine-domain-forge`'s `domain.rs` and
  `boundary.rs` and the root's `forge_route.rs`: `goal-release`, `mine-forge` and
  `mine-start` keep their hunks to the functions their specs name.
- The wave-2 plans (reference-m3, -m5, -m6, -m7b) add their own members to `ChangeCall` and
  the `change` decoder and schema (`mine-call`), and their own steps to the change procedure
  (`mine-procedure`, `mine-start`, `mine-forge`): unavoidable, each adds beside the others.
