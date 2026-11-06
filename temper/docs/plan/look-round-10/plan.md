# look-round-10: the plan

From `inputs/request.md`: land the look request `close-chat` (`c52e34bd67`), by
`temper/docs/design/draft/look-requests.md`, sections 3 to 5. The trunk is `look-round-10/integration`.

The design is cut ahead of the tasks: `temper/docs/design/client/README.md`, 3.6, *The sheet* and *Closed ends on one
line*, with its lines in 3.5, 5.11; `README.status.md`'s new **Not yet**. Every commit of this plan carries `Look-request:
close-chat`.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `confirm-sheet` | | high, 170 | the domain's stay-until-answered, the words, one sheet for every confirmation, what it changes, the closed line, the rules, the gallery, the tests |
| 2 | `close-chat-proof` | 1 | high, 70 | the render check, the walk of each confirmation, the screenshots, the design text, the account |

## Critical path

`confirm-sheet`, `close-chat-proof`: ~240 implementer-minutes, then the trunk's end check.

## What the cut settled

- **The sheet stays until answered,** for "a confirmation that fails: the sheet stays". Today the press closes it and sends,
  so nothing could stay. `Confirm` gains `sent` and `failed`, one rule in `Pages::told`, the same for a chat, a task and a
  card, and a goal's failure goes to the sheet rather than the toast while it is open. What closing does is unchanged; only
  when the sheet goes moves. A change of meaning, so the task runs its readers' tests.
- **The words the request leaves open,** decided in the design's table: an agent's task's Stop ("Stop this task?", "Keep
  going", "Stop task"), Accept ("Accept this proposal?", "Not yet", "Accept"), Reject ("Reject this proposal?", "Keep it",
  "Reject"), Release ("Release this task?", "Keep it held", "Release"), Start over's lead ("A new configuring chat starts.
  This one stays in Chats."), and a live run with no grace ("The run under way stops now."), since "Close chat now" goes.
  Only Accept, Reject and Release open a card's sheet (`acts::step`); no other act of `ACTIONS` reaches one.
- **"N tasks below it":** jig's `Impact.tasks` counts every descendant, delegates included (`jig-domain-core`'s
  `documents/offers.rs`), so N is the live counts less the delegate rows the sheet lists, or they'd be said twice. A moved
  delegate's own tasks stay in N: the document has no count per delegate. No wire change.
- **"Closed at" everywhere an end line says "Closed":** `TimeEnded` and `TimeEndedReport` say what `TimeClosed` says, so one
  state doesn't read two ways. "Failed 21:05" and "Cancelled 21:05" keep their words, which the request doesn't name.
- **`.temper-look-backdrop` isn't landed:** skein opens a dialog with `show_modal`, so `::backdrop` is the real one.
- **The `.terminal-dialog` block goes only where no sheet reads it:** the start-goal, welcome and forge sheets lean on
  `dialog`'s frame, so that stays and the account names it. Those sheets are not `confirm.rs`'s and don't change.
- **jig is untouched:** `jig_client::words::decision` keeps its words for jig's own presentation; temper's `cards::decision`
  drops `dangerous` by the facts' public field. temper's sheet is the mark `facts::dangerous` asks for (jig's
  `client/domain/client.md`, 19). No stored format, protocol or public interface changes.
- **Two tasks,** one writer for the shared files; the proof needs the sheet's code and gallery states.

## Overlaps it could not avoid

`priority = true`; `confirm-sheet` goes first and owns these while it runs:

- `temper-client-view/src/words.rs`, `look.rs`, `temper-client-shell-web/temper.css`: `look-round-6`, `look-round-8`,
  `look-round-9`, `ui-owner-menu`, `ui-rename-focus`, `ui-walk-2`. Word and token ids may collide; a catch-up renumbers ours.
- `temper-client-view/src/pages/chat.rs`, `tests/chat.rs`: `look-round-6` (`reading`, `call-line`, `answer`) and
  `chat-reading-a` (`chat-live`). This plan keeps to `closed_status`, `act_status`, `hidden_line`.
- `temper-client-view/src/tests/needs.rs`: `ui-walk-2/needs-press`, a word's name only. `confirm.rs` itself has no other
  writer among the drafts beside this one. `temper-client-domain/src/pages.rs` and `pages/chat.rs`: `look-round-6`,
  `look-round-8`, `ui-walk-2/login-command`, in other functions.
- `temper/tools/gallery/**`: `look-round-6`, `look-round-8`, `ui-owner-menu`, `ui-walk-large`: regenerate on a catch-up.
- `README.status.md`: an entry of its own, beside theirs.
