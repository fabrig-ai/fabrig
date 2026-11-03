# reference-m6: plan

M6 of `temper/docs/design/draft/reference-case.execution.md`, early starts (N2, N14): a
change depending on another change into the same landing branch starts kept, and its
producer starts early from the landing branch's tip with its dependencies' pushed heads
merged in, once, refused on a conflict; an amendment dropping a dependency started from is
refused.

## The design this cut wrote

- `temper/docs/design/domain/forge.md`, 8.11 (new: early starts), and a pointer in 8.2
  step 1; `agent.md`, section 8 (the producing agent's row: started early, `deliver` mid-run).
- `temper/docs/design/domain/worker.md`, 4.1 and section 5 (a base with heads merged in,
  its conflict).
- `temper/docs/design/protocol/channel.md`, sections 4 and 5 (`Merged`, `conflicted`).
- jig's `domain/connectors.md`, 6.1 and 6.3 (the declaration given to the core; a
  dependency started from), `domain/tasks.md`, section 7 (*Started from*), `protocol/store.md`,
  5.1 (the mark stored).
- Status files: **Not yet** entries in forge, worker, channel (temper) and connectors, tasks,
  store (jig); the drafts' entries for early starts closed in
  `reference-code-production.status.md` (7) and `reference-case.status.md` (4, 8, 11, 13, 17).
- `landing.md`, 4.1 already says the change procedure starts kept; its status's entry is
  closed by `early-route`.

## Parts and tasks

| Part | Task | Needs | Owns, in short | Minutes |
|---|---|---|---|---|
| kept-jig | `kept-start` | | jig-domain-tasks, the core's `Config` | 90 |
| kept-jig | `kept-started-from` | kept-start | jig-domain-tasks, the core's `ProcedureStep`, the tasks family's record | 100 |
| checkout | `checkout-heads` | | the checkout, the worker domain, the worker wire, the service's translation, the engine's `Preparation` | 120 |
| checkout | `checkout-heads-engine` | checkout-heads | the forge's `items::Start`, the engine's `ForgeStart`, `run_workspace` | 45 |
| early | `early-step` | | the change procedure's pure step, the stored `Change.early` | 100 |
| early | `early-route` | kept-start, kept-started-from, checkout-heads-engine, early-step | the root and forge connector: kept start declared, `pending`, marks, starts, the conflict's route, dependents' wakes; the first story | 150 |
| early | `early-stories` | early-route | the root's stories of the case's steps 8, 10, 13 and 17 and the refused amendment | 90 |
| charter | `charter-push` | | a test pinning the reference charter's "push as you go" | 20 |

Released in the first push: `kept-start`, `checkout-heads`, `early-step`, `charter-push`.
Released in the second: `kept-started-from`, `checkout-heads-engine`, `early-route`,
`early-stories`. The plan is whole: no task is left to write.

## Critical path

`checkout-heads` (120) → `checkout-heads-engine` (45) → `early-route` (150) →
`early-stories` (90), about 405 implementer-minutes; `kept-start` → `kept-started-from`
(190) joins it at `early-route`.

## Overlaps with other plans

- Wave 2's plans beside this one (reference-m2, -m3, -m5, -m7b) change the change procedure
  (`temper-engine-domain-forge-change`), the forge connector, the root's `forge_route.rs` and
  temper's store schema: `early-step` and `early-route` cannot avoid them, and keep their
  hunks to their own functions and fields.
- `forge.md`: this plan adds 8.11 and one clause of 8.2 step 1; M2, M3 and M5 add to 8.1. If
  another plan also adds a subsection after 8.10, the second to land renumbers its own.
- `agent.md`, section 8's producing row: M5 may edit the same table row (its "Messages in"
  column); a line conflict at catch-up keeps both edits.
- jig's `jig-domain-tasks` and `jig-domain-core`: reference-m3 (a batch's LLM choices) and
  M5 may touch them; `kept-start` and `kept-started-from` keep to admission, readiness,
  amendment and the step's marks.
- The coding agent's charter: M1 put "push as you go" in it already; this plan only pins it.

## Walk

Slice 3 is walked (`walk = true`): the case's steps 8, 10, 13 and 17 through `temper ctl`
(a dependent change kept on its dependency's pushed head, its producer started early, done
while kept, merged after; the once rule; a conflict refusing the early start). Step 4's
designers kept on S waits for M3's walk.
