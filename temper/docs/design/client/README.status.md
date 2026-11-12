# README: status

2026-11-11. Where temper's code lags or departs from [README.md](README.md).

- **Departs:** 2, *The crate graph*: `temper-client-domain` also depends on
  `temper-protocol-records`, because the forge's side encodes and decodes a
  pasted bundle by it (`forge/forges.rs`), on `skein-ui` for its phrasebooks
  (`words.rs`) and on `skein-json` to read a call's input (`pages/calls.rs`);
  the graph names none of them. Through it the
  deployment's client links the records crate (`../protocol/binaries.status.md`).

## Gaps and departures

- **Not yet:** `ui-ask-m1`'s `proposal-read` part was never designed,
  according to the retirement brief. Its landed run proposal read needs
  design coverage reconciliation against `jig/docs/design/domain/engine/executors.md`,
  3; this is a design gap, not an absent implementation.

- **Departs:** 3.6, *Closed ends on one line*: a requester-closed chat hides
  every "Decision made" activity line, where the section and the request
  `close-chat` of 2026-11-06 hide the closing's alone, because `Activity::Decided`
  carries only a revision and cannot identify the closing decision (section 5).
  This keeps the quiet end line, at the cost of hiding earlier decision markers too.

- **Design departs:** 3.6, *The sheet*, from the user's request of 2026-11-06 to land
  `close-chat`, which asks no change of behaviour: a Release pressed on an uncertain
  effect before either choice closes the sheet and sends nothing, by the section's
  rule, which that plan's cut wrote (`temper-client-domain`'s `acts.rs`, `send`;
  `a_press_that_sends_nothing_closes_the_sheet`). The view draws both choices alike,
  reading no `again`, so nothing says one was pressed. A Reject pressed with no reason
  keeps its sheet open and says nothing, where `close-chat` has a confirmation that
  fails say why above its acts: no ask went, and the reason's box is on the card behind
  the sheet.

- **Not yet:** 3.6, *The sheet*: that request has each confirmation walked. A browser
  on a fake box pressed Close, Start over, a goal's Close, a Release both ways, a goal's Pause,
  and Accept and Reject on a chat's proposal of settings from *Needs you* and *Home*,
  and Accept from the chat itself; Reject from *Needs you* included a reason. Escape and
  the keeping act on Close, Start over, a goal's Close, Accept and Reject were also pressed
  by hand at each width, which no test keeps; a press while the page updates, on a Release
  alone. The client's world holds the chat's Accept and Reject's ask, with no browser
  (`temper/tests/client/tests/suite/chat_accept.rs`, a chat accepting and rejecting its own
  proposal).
  An agent's task's Close is drawn at each width by its gallery state, the walk of the
  request `ui-walk-3` of 2026-11-06 having tried the fake box for it first, and no walk
  on record presses it.

T1's repair, goal 761, carries the launchpad's brief, Where you left off
and Needs you across the person's projects, a walked need's reason,
need ages, dated left-off rows and the blank row under the header
(`tui/pages/launchpad.md`). Its walk follows landing (`README.md`, 5);
Stop asking stays on the web (`../draft/surfaces.status.md`, 11).
