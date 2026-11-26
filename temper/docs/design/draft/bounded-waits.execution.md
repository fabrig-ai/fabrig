# bounded-waits: execution

2026-11-24. How [bounded-waits.md](bounded-waits.md) gets built: one milestone of seven goals, the first alone, then four side by side, each in its own part of the code, then two that compose them. It designs nothing: each goal writes its design from the draft as it runs. Plans use the focused and fuzzy suites only, and no goal walks.

## 1. The milestone and its goals

One milestone, *every wait named, bounded and visible*. Its goals land into main one by one.

| Goal | Draft parts | Where | Needs |
|---|---|---|---|
| W1, the wait record: the type jig and temper share, its kinds, objects and *since*; the table of figures in the settings' limits, with the user's stock; old records read with defaults; a task's line and `task show` listing its waits; the record check at quiet points and the previous build's store test | 3, 4.1's table, 9's first bullet, 12's first and fourth | jig's core and vocabulary, temper's settings and client render | — |
| W2, jig's bounds and holds: each jig kind's bound and outcome; a question passed up stays one; overdue entries; a hold's words always naming cause, object and since; a placed hold's reason; a person task passed up; a sign-in's exchanges held only in flight (P117) | 4 for jig's kinds, 7's first bullet, 8 | jig's tasks: holds, escalations, proposals, person tasks; jig's engine clients | W1 |
| W3, jig's words and wakes: a wait on any task its party can read (P113's jig half); subscriptions counted alike (P86); an escalation waiting at a task woken by any change | 6.3, 6.4, 6.5 | jig's tasks: references, batches, subscriptions, wake policies | W1 |
| W4, temper's procedures: a change's steps, a landing's round and its awaited fact, a closer's step, a goal branch's making, a release's cleanup, outbox room (P124) and sign-off as wait records, each with its clock or figure and its hold's words; what a hold keeps; a stored charter checked against its room at start (P63's residual) | 3.2's temper kinds, 4 for them, 7's second bullet | temper's forge, landing and closer domains, engine start | W1 |
| W5, temper's relays: a held answer's notice to its sender and its blocker in `change show` (P85); nothing taken and dropped, at a change or a closer; a change's `after` on another goal's task (P113's temper half) | 6.1, 6.2, 6.3's `after` | temper's change and closer relays | W1 |
| W6, stuck: what can wake a parked task, dead wakes, the walk when a task parks; tries at each stuck park, then a hold naming the chain; `question_woken` retired (P43) | 5 | jig's tasks: runs and parking | W2, W3 |
| W7, the views: the project's waits read in ctl, as text, paged and JSON, and its page; a milestone's goals' waits; a landing's since and bound; ages and overdue in the inbox and Needs you | 9 | temper's client, jig's client | W2, W4, W5, W6 |

W2 to W5 run side by side once W1 has landed, each in its own part of the code, as the table's third column says. W6 waits for W2 and W3, which share jig's hold and wake paths with it. W7 comes last.

What each goal's design now carries, as W7 read it on 2026-11-26 with all seven landed or landing; the draft's status says what still lags:

- **W1:** the record, its kinds, *since*, bounds and floors, `jig/docs/design/domain/engine/waits.md`; a task's line and `task show`, `../client/ctl/results.md`, 1 and 2; the record check at quiet points, jig's `domain/testing-fakes.md`, 1.1 and 1.2; the previous build's store, `../testing/restarts.md`, 8.
- **W2:** jig's kinds' outcomes, a question passed up as one, overdue at the last holder, a hold's words and a placed hold's reason, a person task offered on: jig's `domain/tasks/lifecycle-holds.md`, 1 to 3, `messages.md`, 2, `proposals.md` and `domain/people/README.md`, 7; a sign-in's exchanges held only in flight, jig's `protocol/client/README.md`, 10.
- **W3:** a wait on any task its party can read, subscriptions counted alike, an escalation woken by any change: jig's `domain/tasks/messages.md`, 3 to 6; a change's `after`, `../domain/agent/tools-change.md`, 1.
- **W4:** a change's, a landing's and a closer's waits, room, a making's and a release's bounds, `../domain/forge/waits.md`; a landing's bounded reads, `../domain/landing-waits.md`, 3; a stored charter checked at start, `../domain/agent/room.md`, 4.
- **W5:** a held answer's notice and nothing taken and dropped, `../domain/forge/change/notices.md` and `questions.md`, 1.
- **W6:** stuck, `jig/docs/design/domain/tasks/stuck.md`.
- **W7:** the project's waits, jig's `client/domain/contract/reads.md`, 1.2, in ctl, `../client/ctl/tools-reads.md`, 1, and as a page, `../client/pages/waits.md`; a need's bound, jig's `client/domain/contract/documents.md`, 1, with its age and overdue mark in ctl and on Needs you; a milestone's goals' first waits, `../client/ctl/milestones.md`, 4, and `../client/pages/milestone.md`, 3; a landing's since and bound, and its wait's name on the public landing, `../domain/landing-waits.md`, 4.

## 2. Keeping this file and the status

Goals running side by side would conflict on every line they each edit here (the user, 2026-11-24). So only W7, the last, updates this file and the draft's status file, from the milestone's results. W1 to W6 leave both alone: each records the design it writes in the design documents it changes, with their status files, and says in its result what of the draft it carried. A goal's state is the milestone's on the deployment, not a column here.

## 3. After the milestone

- The team's watchers retire (`bounded-waits.md`, 10), by their owners, once the milestone has reached the reference deployment.
- The figures the status file lists as open are the user's: they may change after the first week's waits are seen.
- What the draft's status still lists under **Design lags** has no milestone yet: each is a goal of its own, or the next milestone's, to plan from that entry. The draft stays until the design carries it all (`../../../../DESIGN.md`, 4).
