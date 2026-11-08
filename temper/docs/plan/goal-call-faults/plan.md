# goal-call-faults: the plan

Three faults a goal's coordinator met in its host calls on a real
deployment (`inputs/request.md`), each found in the code, its failing
row first in an existing table, and the design saying so (item 4) where
each is written. Design written by this cut:

- `temper/docs/design/domain/agent.md`, 4.3.1 (a batch's members) and
  4.3 (*A run's subscriptions take its task's room*), and its status;
- `temper/docs/design/domain/forge.md`, 7.2 (*Those are a goal's only
  topics from the core*), and its status;
- `temper/docs/design/domain/root.md`, 5.2 (the engine alone's page),
  and its status;
- `temper/docs/design/domain/testing.md`, row 5.4 (A hears no CI news
  of its changes' heads).

## Parts and tasks

| Part | Task | Request item | Crates (files) | Needs | Minutes |
|---|---|---|---|---|---|
| `change-batch` | `batch-options` | 1, 4 | temper-engine-smith (`calls.rs`, `Problem`), temper-service (`tools.rs`'s rendering) | | 40 |
| `subscribe-room` | `goal-ci` | 2, 4 | temper-engine-domain-forge (`topics.rs`, `restored`), temper-engine-domain (`forge_route.rs`), the engine, forge, protocol-forge and real-loop worlds | | 110 |
| `subscribe-room` | `busy-room` | 2, 4 | temper-engine-smith (`answers.rs`' `task_capacity`), the engine world's subscription-room table | `batch-options`, `goal-ci` | 45 |

Item 3 is the question `read-room` in `plan.toml`: no task.

## Decisions

- **Item 1, a batch's options: refused, not inherited.** The design
  already makes each change of `changes` an object of its own members
  and refuses `words` beside `changes`. Inheriting would need a rule per
  member (an outer `name`, `after`, `merge`, `repairs` or `branch` means
  nothing shared across a batch) and which value wins; a refusal at
  decoding makes no change, names the member and says where it goes
  (`host call member belongs in each change, not beside changes:
  landing`), so nothing given is dropped unheard. The first such member
  in the declaration's order is named. A new `Problem::Beside` carries
  it, since `Range` renders "out of range", which reads as a bad value
  and could lead a run to drop the member. The declaration's schema
  text is unchanged, so no host-part figure moves. An unknown member
  stays accepted in both forms, as today: outside the request.
- **Item 2, what made the goal busy to itself:** its own subscription
  room. jig refuses a subscription once the subscriber's row holds
  `tasks.subscriptions` (4 in `small` and `standard`), before it looks
  at the target, so `kind: task` and timers alike were refused; and
  temper renders that cause (`CapacityCause::Subscriptions`) as nothing,
  hence exactly `Refused: task 4 busy`. The row was full of the core's
  subscriptions for the goal: its landings topic and, from code no
  design asks for (forge.md 7.1 gives CI on a head to changes and tasks
  that subscribe; 7.2 gives goals landings), one `Topic::Ci` per live
  change's head. A goal with three changes pushed fills 4.
- **Item 2's fix: the code goes back to the design.** `goal-ci` stops
  the core's goal CI subscriptions and drops those a restart finds;
  `busy-room` names the room in the refusal. The run then has its room
  less its landing branches (one or two). Weighed and not taken:
  counting the core's topics apart from the run's in jig (a mark on
  each stored subscription, or a stored row up to twice the bound: a
  stored format and the store's bound, the user's), raising the figure
  (outside the request), and carrying a goal's CI news under its
  landings subscription (no format change, but it merges CI, landing
  and red news into one inbox hint, rewrites the CI bookkeeping in five
  places, and keeps news the design never asked for). The goal still
  hears each change's end and hold as its delegate's; a run that wants a
  head's CI subscribes to it. jig's own renderer drops the same cause
  for jig's applications (`jig-protocol-tools`' `render.rs`): left to a
  later plan, since a branch stays in its tree.
- **Item 3, the 128-byte page: a question.** The page is the
  deployment's `documents.text_bytes` (128, from a document made before
  `temper init` wrote 512 on 2026-11-05), which a process with no local
  agent, an engine alone, uses as its room because it is never told its
  workers' reply bound; the room a local agent gives (10,662 bytes at
  init's figures, 3 calls for 30.5 KB) landed on 2026-11-06 and is in
  `b4555e6785`. Going past it needs that bound at the engine: jig's
  worker link (a protocol change) or the engine's document (a
  configuration change), both question-first by the request. The
  design now says what the page was and why (`root.md`, 5.2).

## Overlaps

- None with the drafts beside this one: `temper-engine-smith`,
  `temper-engine-domain`, `temper-engine-domain-forge`, the engine
  worlds, the real loop and `temper-service/src/tools.rs` are on no
  other draft's branch (`tui-box-front/serve-sign-in`, merged, changed
  other files of `temper-service`).
- Inside the plan: `batch-options` and `busy-room` share
  `temper-engine-smith` (`answers.rs`, `tests.rs`), and `goal-ci` and
  `busy-room` share `forge_routes.rs`, so `busy-room` follows both.
  `agent.status.md` holds two of this plan's entries, apart from each
  other.

## Critical path

`goal-ci` (110) → `busy-room` (45), about 2.6 hours; `batch-options`
(40) beside `goal-ci`.
