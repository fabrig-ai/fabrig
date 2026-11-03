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

The cross-task shapes are `notes/contracts.md` (C1 to C8); the standing rules,
`notes/common.md`.

## Tasks so far

Released (needs in brackets):

- relay: `relay-types` [] (C1, C2's pure types and the bound); `relay-surface` [relay-types]
  (the row and store, parameter 14, the hold past the bound on the wire).
- call: `questions-call` [] (the member decoded; foreman and up refused until the relay).
- foreman: `foreman-charter` [] (the stock's charter 6 and the box's limits).

Next batch (being written): `relay-policy` [relay-types] (the pure relay step, C7);
`questions-panel` [relay-surface] (C8); `relay-up` [relay-surface, relay-policy,
questions-call] (the root's relay, straight up, held answers, the bound);
`relay-foreman` [relay-up, foreman-charter]; then `relay-story`, `questions-help` and
`reference-relay`.

## Critical path, as it stands

`relay-types` → `relay-surface` → `relay-up` → `relay-foreman` → `relay-story`, about six
hours of implementer time.

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
