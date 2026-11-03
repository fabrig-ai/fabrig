# reference-m5: plan

Milestone M5 of `temper/docs/design/draft/reference-case.execution.md`: a change relays its
agents' questions through a foreman of its own or straight to its requester (N1), within a
bound, an answer held until a dependency lands, its open questions in `ctl`; and the
reference's coordinator answers, amends, makes a new change, asks its person and decides
holds, waking on each question at once (N8, already its wake policy). Written by the cut as
it releases; each batch brings it up to date.

## The design, written by this cut

- `temper/docs/design/domain/forge.md`, 8.1 (a change's questions, a parameter) and the new
  8.11 (the relay: its agents may ask, taken at a step, one at a time, the bound, a foreman
  per question, passed up, the answer relayed down, held until its dependencies land, an
  asker that ends first, its record and panel); 8.8 (its panel's questions).
- `temper/docs/design/domain/agent.md`, 4.3.1 (`questions`) and section 8, 8.2 (a change
  asks only as it is made to; the foreman's row).
- `temper/docs/design/protocol/configuration.md`, 13.2 (kind 3's number 3, the foremen's
  charter; kind 6, a project's questions), 13.4 (the stock's charter 6, a foreman's; the
  box's limits), 13.9 (the reference: coordinator, coding agent, foreman, role 0).
- `temper/docs/design/protocol/store.md`, section 2 (family 12's questions);
  `temper/docs/design/client/README.md`, 4.6, `client/forge.md`, 9.2 and `client/tools.md`,
  5.5 (the panel's questions in `change show`).
- Their status files' **Not yet** entries; the draft's status loses its "8, 14: questions"
  **Design lags** entry.

## Families and parts

| Family | Parts | Design |
|---|---|---|
| relay | `relay` | `domain/forge.md` 8.11 |
| call | `call` | `domain/agent.md` 4.3.1 |
| foreman | `foreman` | `protocol/configuration.md` 13.4 |
| panel | `panel` | `client/forge.md` 9.2, `client/README.md` 4.6, `client/tools.md` 5.5 |
| reference | `reference` | `protocol/configuration.md` 13.9; `docs/help/` |

The cross-task shapes are `notes/contracts.md` (C1 to C8); the standing rules,
`notes/common.md`.

## Tasks so far

Released (needs in brackets):

- relay: `relay-types` [] (C1, C2's pure types and the bound); `relay-surface` [relay-types]
  (the row and store, parameter 14, the hold past the bound on the wire); `relay-policy`
  [relay-types] (the pure relay step, C7); `relay-up` [relay-surface, relay-policy,
  questions-call] (the root's relay, straight up, held answers, the bound; C2's revision 2,
  the question's words on the record); `relay-foreman` [relay-up, foreman-charter] (a
  foreman per question, C5).
- call: `questions-call` [] (the member decoded; foreman and up refused until the relay).
- foreman: `foreman-charter` [] (the stock's charter 6 and the box's limits).
- panel: `questions-panel` [relay-surface] (C8).
- relay, the story: `relay-story` [relay-foreman, questions-panel], the meeting point: a
  goal's changes into g ask, a foreman settles or escalates, the goal answers, an answer
  waits for a dependency, a change past its bound holds and is released; `relay-stall`
  [relay-foreman], a follow-up the cut found after releasing `relay-up`: a change waiting
  on an agent's open question does not stall (`domain/forge.md`, 8.4).
- reference: `reference-relay` [relay-foreman, foreman-charter] (`notes/settings.md`'s
  texts: the coordinator, the coding agent, the foreman's charter 6); `questions-help`
  [relay-foreman, questions-panel] (`docs/help/task.md`, `change.md`).

Every task is released; the plan is whole.

## Slices

1. relay, call, foreman: the relay runs on the domain world, `relay-story` its story.
2. panel: `change show` prints a change's questions.
3. reference, **the walk** (`walk = true`): the case's steps 9, 11, 14, 15 and 18 through
   `temper ctl`, with the reference's settings applied and the box's rules and policy 1
   given charters 5 and 6, role 0 and the control family (13.9); step 5's question once
   M3's walk stands. `temper init`'s document sets `tasks.batch` 1, which bounds a goal's
   batch of changes and a change step's messages: the walk's document sizes it. The box's
   stored rules and policy 1 predate charter 6, so the walk adds it beside charter 5; and
   while the stock's delegation also names procedures 3 and 4 (`configuration.status.md`'s
   **Departs** for 13.4), charters 5 and 6 with role 0 make nine executors, one past the 8
   `authority.executors` and `tasks.executor_kinds` the document gives: the walk's document
   sets 9 until that departure is closed.

## Decisions

- **The foreman's charter is the stock's charter 6,** fixed by the binary for kind 3's
  number 3 as the review's is at charter 4, since kind 3 is no setting until temper's kinds
  are (reference-m3); 5 is the reference's coordinator. The reference overrides charter 6's
  value with its own rules and model. `temper init`'s document gives `tasks.charters` 8,
  `authority.executors` and `tasks.executor_kinds` 8 (13.4's figure, closing a Departs) and
  `tasks.contract_choices` 4 (a foreman's three choices; a person task's options).
- **The bound is a limit,** `forge.change_policy.questions` (4), beside `repairs`, since a
  change's bounds are the deployment's `change_policy` today (`forge.status.md`, 8.1).
- **A project's questions are kind 6,** designed and not built: until temper's kinds are
  settings, a change whose maker names no route asks no one, so the reference's
  coordinator names `"questions": "foreman"` on each change.
- **One question at a time** per change, in the order taken; a step's messages stay within
  `tasks.batch`, the rest at the next step at once (contract C7's `room` and `more`).
- **The record keeps a question's words** (C2, revision 2): jig offers a procedure no reader
  of a question's words by number.
- **An independent review after release** (03:03 to 03:18) found the relay's edge cases
  that the released specs did not settle: inputs lost on a step short of room, a refused
  step's row change, a late foreman's result, foreman text past the message bound, the
  foreman's step bookkeeping and the world's sizing, the change's kept for a held answer,
  `taken` on any release. They are `notes/contracts.md`'s C2 and C7 revision 3 (binding
  `relay-up`, which owns `relay.rs` and supersedes `relay-policy`'s rules where they differ)
  and C5 and C6 revision 2 (binding `relay-foreman` and `relay-story`). It found the
  queue's own `kept` needs nothing: an agent's question arises only while the change
  produces, repairs or resolves, out of its queue.
- **Holds passed up** need no mechanism: jig's escalation reaches the coordinator, whose
  `decide` passes it to its person; the coordinator's charter says when (13.9).

## Critical path, as it stands

`relay-types` (35) → `relay-surface` (80) → `relay-up` (120) → `relay-foreman` (100) →
`relay-story`, about six hours of implementer time; `relay-policy` and `questions-call` run
beside `relay-surface`, `questions-panel` beside `relay-up`.

## Overlaps with the plans beside this one

- `temper-engine-smith/src/calls.rs`, `temper-engine-domain/src/boundary.rs` and
  `route/forge_route.rs` (`change_batch`, `chat_change`, `start_change`): reference-m2 and
  reference-m3 add `change`'s members and parameters there too. This plan's members sit
  beside theirs; its parameter is 14, leaving 11 to 13 to them (contract C3).
- `temper-engine-domain-forge-change`, `temper-engine-domain-forge` and the store's
  `ChangeRow` (v11 in place): reference-m3 adds a producer's charter, likely on the row.
- `temper/docs/examples/reference/settings.json`: every wave-2 plan edits the coordinator's
  instructions, one JSON line. This plan's `reference-relay` goes last, after the code, so
  it meets the others' edits by catch-up.
- Design: `domain/agent.md` 4.3.1, `domain/forge.md` 8.1 and `protocol/configuration.md`
  13.2 and 13.9, each a compatible addition of this plan's own part.
- Within this plan, `questions-call` and `relay-surface` both edit `route/forge_route.rs` in
  the first wave, in different functions (`change_batch`; `start_change` and
  `hold_change_decision`).
