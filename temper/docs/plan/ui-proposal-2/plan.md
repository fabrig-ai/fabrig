# ui-proposal-2: revising a proposal works from its own button, and what a real model's walk left

The request: `inputs/request.md`. Focused and fuzzy alone, three tasks; a plan is complete when it lands, and temper/ui looks after landing.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `replace` | `tasks/replace-own.md` | | A run's fresh proposal of settings replaces its own that waits, in one commit; the busy refusal's untrue words go |
| `chat` | `tasks/chat-revises.md` | `replace-own` | The configuring chat revises from "Change something": its words, the stories through the real engine, the help |
| `card` | `tasks/card-and-latest.md` | | A Goals-only card's rows and the read that lost them, the activity row's and the template's words, a refused Accept's words, Latest off a phone's controls |

Critical path: `replace-own`, then `chat-revises`. `card-and-latest` runs beside them from the start.

## What this plan decided

- **The mechanism: a fresh `propose_settings` replaces its own** (`jig/docs/design/domain/tasks.md`, section 9). The request left it to the cut, smallest first. A tool to withdraw looked smaller in jig, one arm of `named_tool_kind`, but moves `withdraw` out of the decide family for every run. It also leaves a moment with nothing pending, a card reading "Withdrawn" if the next call is refused, and a model that must keep a number across runs. Replacing changes one admission check and one function in `jig-domain-tasks`, gives no run a new power, and the person never loses the card they could accept.
- **What stays safe.** A refused fresh proposal leaves the old standing. Only settings replace settings, and only a task's own. The decider still decides. An Accept on a replaced card is refused and changes nothing, and the card ends on that refusal.
- **Accept's refusal is `Unknown`, and the client reads it.** jig writes no decision record for a task's withdrawn proposal, so a late decision is refused as unknown, not answered "decided first". Giving it a record would need a person as its `by`. The card treats that refusal, on a proposal the person just read, as withdrawn. This holds for a stale withdrawal too.
- **The busy refusal loses "or withdraw it" in both copies:** a run with no decide family can't, and the refusal doesn't know a run's families.
- **A card's application items come from the review, not the heads.** The trace found two losses: `proposal_heads` leaves them out, so `forge/ways.rs` asks nothing and `settings_lines` returns early; and that read asks under the proposal's number, where the forge keeps values under the review's `key`, as ctl reads them. So the existing rows only ever ran on hand-made documents. Carrying them in the heads would change `jig-domain-tasks`' stored `Setting`, which another plan is changing, for the sake of a decided card's rows.
- **So a decided card keeps the rows it read, and with none draws its heads alone,** and a person with no deployment role has no review, so no such rows: the forge's read takes that role already (`temper/docs/design/client/README.md`, 5.13).
- **Latest takes a row of its own at a phone's width,** over 760 px staying as the mock floats it. Anywhere it floats it can sit over something, and moving it to a corner still meets the buttons at 320 px. No focused world lays out a page, so its test holds the rules; temper/ui's look after landing presses it.
- **Every row of a card takes the card's type,** not only a template's: one row in another type would read as a fault. This departs from the mock's mono, recorded in `temper/docs/design/client/README.status.md`.
- **`an_owner_rejects_a_chats_pending_proposal_under_needs_you` has no `ignore` and runs,** where `agent.status.md` said it was ignored: the entry goes with the help page's false line.

## Stored formats and protocols

**None change.** A replaced proposal is an ordinary withdrawal: `ProposalState::Withdrawn`, a `ProposalWithdrawn` history row, a `Decided` activity line, its set erased as today. No new event, record, field or wire form, and no journey recording goes stale. temper/migration's plans have nothing to share here.

A meaning does change: a run's second proposal of settings is no longer busy.

## Overlaps with the drafts beside this one

Unavoidable, and whichever lands second catches up, as the request says:

- `model-catalogue-2/configuring-learns`: `configuring.txt`, and `temper-client-view/src/cards/proposal.rs` (`chat-revises`, `card-and-latest`).
- `look-round-13/look-proof`: `temper/docs/design/client/README.md`, its status, `temper/docs/help/**` (this cut's design, `chat-revises`).
- Other plans' `temper.css` and `temper/tools/gallery/**`: rules stand under their markers, and goldens are regenerated.

Avoided: `ui-model-next-turn/choice-next-turn`'s `jig-domain-tasks` `boundary.rs`, `control.rs`, `domain.rs`, `compound/staged.rs` and stored schema. `replace-own` keeps to `proposals.rs`, so it can start at once.

Inside the plan, `chat-revises` and `card-and-latest` both touch `temper/tests/client/tests/suite/configuring_chat.rs`, in different functions, and `temper/docs/design/client/README.status.md`, in different entries.

## Left outside the plan

- At the `settings.proposals` bound, a fresh proposal is refused rather than counting its replaced set as room: the old stands, nothing is lost, and the bound is rarely reached.
- A chat's `withdraw` stays offered and refused (`temper/docs/design/domain/agent.md`, section 10, *Which host tools a charter offers*).
- Whether a real model takes the new words is after landing's: one walk asks it for a different figure and for a model with no account.
