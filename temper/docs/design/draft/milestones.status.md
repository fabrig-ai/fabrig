# milestones: status

2026-11-16. Where the design lags or departs from [milestones.md](milestones.md).

Its jig part is designed and built: jig's gathering, a party's `Delegate` beneath a task and a party's batch naming an ended dependency (jig's `domain/tasks/gatherings.md`; `domain/tasks/README.md`, section 4). Sections 3, 6, 7 and 8 are designed, milestone M2 of [home-for-a-person.execution.md](home-for-a-person.execution.md): a milestone's template, identity and reads (`../domain/milestones.md`), and `temper ctl`'s tools (`../client/ctl/milestones.md`). Sections 4 and 5 follow, as M3.

- **Design lags:**
  - 4: `../client/pages/work.md`, 1's `goals::state` has no state for a goal that waits on a dependency or on a hold, and no page has a milestone's state or headline.
  - 5: `../client/pages/home.md`, 1, `../client/pages/work.md`, 1, `../client/pages/needs-you.md`, 1 and `../client/pages/goal.md`, 2 draw no milestones; the goal page links any task requester as a chat.
- **Design departs:**
  - 7: a milestone's executor is jig's gathering, a core procedure (jig's `domain/tasks/gatherings.md`, section 2), not a procedure of temper's: what it does names no system, and jig keeps the promise that it ends only once its maker has said whole and every goal in it has ended. Its maker's word, whole, is jig's `Conclude`; it ends done with a report counting its goals done, failed and cancelled, from which the pages say complete or ended (3). Decided by M1's designer, 2026-11-16, keeping the draft's intent.
  - 6: a goal's `--after` naming an ended task is jig's rule for any party's batch, a `Create`'s or a `Delegate`'s (jig's `domain/tasks/README.md`, section 4), so temper's `goal start` passes it through and judges nothing itself.
  - 3: a goal in a milestone asks its maker as a goal a chat calls asks its chat's person, with `message` to its requester, which jig hands to the milestone's requester (jig's `domain/tasks/gatherings.md`, section 4), not by a person task as a goal a person started does: its requester is the milestone, a task, and the question reaches the maker's Needs you all the same (`../domain/milestones.md`, section 5). Decided by M2's designer, goal 4793, 2026-11-16, keeping the draft's intent.
  - 8: a goal's milestone is no read of its own: a goal joins at its start as the milestone's direct delegate, so its task line's requester names it, joined with the project's milestones (`../domain/milestones.md`, section 7). Its last landing is the forge's, kept in a row per milestone from the landings it records, at most 256 rows a project, so a milestone whose row was dropped reads none (`../domain/milestones.md`, section 8). Decided by M2's designer, goal 4793, 2026-11-16.
  - 6: `milestone list` and `milestone show` print a milestone's facts, totals and goals, its state `live`, `complete`, `ended` or `closed`; the pages' state words and headline (4) come with them. `milestone new` takes `--priority` and `--deadline` as `goal start` does (`../client/ctl/milestones.md`). Decided by M2's designer, goal 4793, 2026-11-16.
- **Open:** 11, each question with its default.
