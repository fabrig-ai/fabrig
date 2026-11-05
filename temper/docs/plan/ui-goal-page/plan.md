# ui-goal-page: the plan

From `inputs/request.md`: a goal's page, its acts and Start goal, drawn as the greenfield mock's `goal.html`. The design is cut ahead of the
tasks: `temper/docs/design/client/README.md`, 3.5, 3.6, 5.8, 5.11, 5.14, 10 and 11; `browser.md`, 2.3, 2.5 and 7.2; jig's
`client/domain/contract.md`, 5.1.

## How it sits on the plans before it

The request starts this plan after `ui-goals` and `ui-needs` have landed. Neither has, and the trunk was made from `ui-goals`' alone. Two things let the work
start anyway:

- **`after = ["ui-goals", "ui-needs"]`,** added by the cut beside fab's own options: this trunk carries their unlanded commits, and the option holds its end check and
  landing until they've landed. Fab's trunk was made as the option's text describes, without the option.
- **`needs-base`** merges `ui-needs`' integration into this trunk, so that what the page builds on (the parsed card, `pages/ask.rs`, the toast) is there.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `needs-base` | | high, 75 | `ui-needs`' merged code on this trunk |
| 2 | `made` | | medium, 60 | `Task.made`, from the record through the wire to the client |
| 3 | `goal-reads` | 1 | high, 90 | the domain's `pages/goal.rs`: its report, its line, its inbox, its pure decisions |
| 4 | `goal-look` | 1 | medium, 50 | the tokens and the mock's rules |
| 5 | `goal-header` | 2, 3, 4 | high, 90 | the view's `pages/goal.rs`: header and report, not yet reached |
| 6 | `goal-side` | 5 | high, 90 | Tasks, Landing, How it runs, Decided |
| 7 | `goal-bands` | 6 | high, 60 | the needs, as bands |
| 8 | `goal-route` | 7 | high, 90 | the hand-over; the old page, its tests and tokens gone; `golden/task.txt` |
| 9 | `goal-worlds` | 8 | high, 90 | the joint worlds on the new page |
| 10 | `goal-gallery` | 8 | high, 90 | the read-only samples; the old four moved |
| 11 | `goal-acts` | 9, 10 | high, 90 | Amend, Pause, Resume, Stop, the toast's kinds, their samples |
| 12 | `goal-composer` | 11 | high, 75 | `.sent`, the composer, `goal-sending` |
| 13 | `start-domain` | 11 | high, 75 | the sheet's state, its three ends |
| 14 | `start-sheet` | 12, 13 | high, 90 | the sheet, Goals' link, its samples |
| 15 | `plan-filter-out` | 14 | medium, 60 | `PlanFilter` gone from the address |
| 16 | `goal-browser` | 15 | high, 75 | the browser tier's story; the fake box's walk reaches the page |
| 17 | `goal-proof` | 16 | high, 90 | every state beside the mock, a box walk, the statuses |

## Critical path

`needs-base`, `goal-reads`, `goal-header`, `goal-side`, `goal-bands`, `goal-route`, `goal-worlds` (or `goal-gallery`), `goal-acts`, `goal-composer`, `start-sheet`,
`plan-filter-out`, `goal-browser`, `goal-proof`: ~1,105 implementer-minutes, then the trunk's end check. The request asks for one order, since `temper.css`, the gallery and
the goldens conflict; the cut runs side by side only what touches none of each other's files: `needs-base` and `made`, `goal-reads` and `goal-look`, `goal-worlds` and
`goal-gallery`, `goal-composer` and `start-domain`.

## What the cut settled

- **A goal's counts,** "6 of 9 tasks", true past a page, come from its own task line, since `delegates_done` and `delegates_total` are on the line alone and the request
  keeps jig to `made`. No key selects one task, so the page reads `Read::Tasks` by its project, requester, label and tracked, following cursors, at most 8 pages; a goal not found
  counts what the page holds. A party with more goals than that sees the page's own count, and only then.
- **The inbox runs in `pages.inbox`,** as Home's and Needs you's do, so that `ui-needs`' acts answer unchanged; `Pages::cards` gives its cards on a tracked page, whose own
  `task.cards` stay empty.
- **Plan rows' rules stay in 5.8,** moved after the goal's page: 5.9 and 5.10 cite them, and renumbering is barred. `task-held` and `task-releasing` move to an agent's task's plan,
  since a goal draws no held rows.
- **A live goal with no tasks** says the planning heading rather than "0 of 0 tasks landed", as `goals::row` already does for needs.
- **`goal-header` to `goal-bands` build a page nothing reaches** behind a scoped `dead_code` expectation: the hand-over changes 100 tests' page at once, and is its own task.
- **`goal-worlds` and `goal-gallery` go red between `goal-route` and themselves,** named in `goal-route`'s `## Left`.
- **Not built, as the request says, and named in `start-sheet`'s options:** `ui-away`'s fieldset and "follows the away switch", `ui-settings`' default way, the Chats area's Drafts link and
  "Start goal…". Each is a one-line option if its plan is on the base by then; otherwise a goal always asks, and the highest-numbered way is the default.
- **Stored formats and protocols change in place:** the client's wire gains `Task.made` (`jig/docs/design/protocol/README.md`, 7); no deployment to keep.

## Overlaps with the drafts beside this one

- `needs-base` meets `ui-goals`' and `ui-needs`' lines: the two conflict in about twenty files, one to three hunks each. Whichever lands second meets the same conflicts on its catch-up,
  and this trunk's next catch-up settles any difference between the two resolutions.
- `temper.css`, `look.rs`, `words.rs`, and the gallery's `names.txt`, `samples.rs`, `tests.rs`: `ui-chats`, `ui-settings`, `ui-welcome` hold open tasks on them. This plan keeps to its marker
  and its own `pages/goal.rs` and `pages/start_goal.rs`, and regenerates on a catch-up (`notes/common.md`).
- `Address::Task`'s `filter`: `plan-filter-out` removes a field every plan names (about 110 sites here). A plan landing after it meets a compile red on its catch-up, a one-line edit each; it runs late so as to meet the fewest.
- `pages/task.rs`, `tests/task.rs`: `ui-chats/session-layout` edits them for an agent's task. `goal-route` removes other lines; they should merge.
- `temper-client-domain/src/pages.rs`, `route.rs`: `ui-settings` and `ui-welcome` touch them; only `goal-reads`, `goal-acts` and `start-domain` do here, for acts and routing.
- `temper-fake/src/backends.rs`: only `goal-browser`, late, and only if its option is taken.

## Dropped

Nothing of the request. The chat's "Start goal…" isn't built here, as the request says: it's the Chat area's.
