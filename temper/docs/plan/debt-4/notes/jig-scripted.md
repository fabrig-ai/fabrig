# debt-4: the rules the scripted engine copies

For the `jig-scripted-*` tasks, which pay `jig/docs/development/debt/D028-scripted-engine-reimplements-core.md`.
From a survey of the fake at the plan's base; `E` is `jig/testing/jig-client-scripted-engine/src`.

## What the cut read

- **The fake links no core.** `jig/tests/boundary/src/tests/graph.rs`, `scripted_engine_links_only_the_client_codecs_and_foundation`,
  bars it, and the core's rules read `tasks::TaskRecord`, which the fake does not hold. So "taken from
  jig's own code" means the client crates' code. Do not widen that test: stop and ask first.
- **Paid when names three rules,** inbox order, phase classification and requester choice, and the
  item's text adds `queries.rs`'s `order`. Query admission and subtree counts are outside it, and
  `jig/docs/design/client/domain/testing.status.md`'s **Departs** records them.
- **The stories stay where they are.** The item would move scripted-only stories into
  `tests/client/src/stories/shared`. The survey read all of them: each needs a document `Front`
  cannot seed, and a `Facts` variant for one such document costs more than the story pays. Only two
  dead fixtures go.

## The rules

| Rule | At base | Verdict | Task |
|---|---|---|---|
| Phase to class, ended set | `E/queries.rs`, `selects`; `E/watches.rs`, `ended`; `line`'s `ended_at` | removed: `Phase::class` | `jig-scripted-phase` |
| Two count matches | `E/queries.rs`, `line`; `E/watches.rs`, `fold` | one function | `jig-scripted-phase` |
| Inbox order | `E/reads.rs`, `inbox_order` | removed: rows as seeded; a shared story holds the seed | `jig-scripted-inbox` |
| Task line order | `E/queries.rs`, `order` | stays: the fake computes lines live; shared stories hold it | `jig-scripted-order` |
| A member's own chats | `E/queries.rs`, `selects`, `query.requester` | shared story | `jig-scripted-requester` |
| An escalation record's requester | `E/requests/decisions.rs`, `requester: by` | fixed: the walk `jig-domain-tasks`' `context` does; shared story | `jig-scripted-requester` |
| A task holder's standing | `E/requests.rs`, `recipient`'s `Recipient::Task` | fixed for escalations, as `escalation_standing` says | `jig-scripted-requester` |

## How a shared story runs

A story is `pub fn name<F: Front<Test>>(world: &mut World<Test, F>)` in
`jig/tests/client/src/stories/shared`, one line in `person_story_suite!`
(`b_runner.rs`) naming its `Facts`, which `b_seed.rs` installs on both sides: `scripted` and
`core`. That one line runs it on the scripted and joint fronts (`tests/b_shared.rs`) and the protocol
front (`jig/tests/client-protocol/tests/suite/shared.rs`). A new `Facts` variant adds its arms to
`b_seed.rs`, `b_scenarios.rs` and `b_config.rs`; the compiler lists them. The joint front's limits are
in `tests/b_shared.rs`: `documents.rows = 2`. A story that needs more changes its `Facts`' limits in
`b_config.rs::setup`, not that line. `Seed::Conversation` seeds parties 1 and 3 on project 1 only.

A story reads an order from the tree, as `the_conversations_section_lists_live_chats_by_last_activity`
does (`b_conversations.rs`).

If the protocol front cannot run a new story for a reason D029 names (one identity), mark that one
`ignore = "..."` through the suite macro's attribute lists, and say so in your report; the scripted
and joint fronts are what **Paid when** asks.

## What a task runs

`cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/`; `heavy cargo clippy -p
jig-client-scripted-engine -p jig-client-world -p jig-client-protocol-world --all-targets -- -D
warnings`, adding the crates your spec owns; `heavy cargo nextest run -p jig-client-scripted-engine
-p jig-client-world -p jig-client-protocol-world`, adding `-p jig-ops-world` when a fixture of
`jig_client_world::stories` changes. temper reads
`jig_client_world::stories::{frame, task, project, deployment, conversations::chat}`: rename none
of them. Budgets are the user's: a shared story adds its seconds to the tier, and over budget is red.
