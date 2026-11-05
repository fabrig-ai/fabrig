+++
id = "tool-lines"
part = "tool-lines"
kind = "feature"
needs = ["conversation-turns"]
files = [
  "temper/crates/temper-client-view/src/cards/delegation.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/confirm.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/delegation.rs",
  "temper/crates/temper-client-domain/src/pages/delegation.rs",
  "temper/crates/temper-client-domain/src/tests/**",
  "temper/crates/temper-client-shell-web/files/pen.svg",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
]
effort = "high"
minutes = 85
+++

# Tool lines and cards in the conversation

## Goal

A call, a delegated change and a goal are one line each, with the pen icon; a card in the conversation takes `.proposal`.

Acceptance, the request's section 4, *Tool lines* and *Cards in the conversation*, and `temper/docs/design/client/README.md`, 5.7, *In
the browser it is one line*:

- `ICON.pen` as `temper-client-shell-web/files/pen.svg`, drawn as `send.svg` is, its colour `--gf-faint`, 14 px.
- **The disclosure of other calls** keeps its name and description ("2 tool calls", "read ×2"), one `.turn-tool` line: pen, name
  in `--gf-muted`, description in `--gf-faint`; open, its work lines indented 24 px.
- **A delegated change:** in the browser one `.turn-tool` line: pen, "Change" (`DelegationLineChange`, browser alone), its title in
  `--gf-ink-2`, its state lower case, its result's words once it has ended, then a link. Meta line and steps are hidden in the browser
  and stay for the terminal. The link's name stays "Task N"; its text is `DelegationOpen` in the browser and "Task N ↗" in the
  terminal, underlined as `.turn-tool button`. A held change's state is "held" in `--gf-signal`.
- **A goal:** `delegation::Facts` gains `goal: bool`, a delegate that is tracked (an agent's executor with `task.priority.is_some()`, as
  `pages/task.rs` decides `Shape::Tracked`, or its `goal` call). The same line with "Started goal" (`DelegationLineGoal`), its title in `.file` mono (`TOOL_FILE`), its state, "Open".
- A card on a task's page, which reads `delegation::status` and `said`, is unchanged.
- **Cards in the conversation** (a proposal, a held task, a question's answer), on a chat's page, take `.proposal`: white, radius 12, its
  shadow, 20 px 22 px; a kind's title in `.proposal-title`; `.proposal-split`, `.proposal-state`; first button `.btn.primary.small`, the
  rest `.btn.quiet.small`. The inbox's cards don't change.
- **Gallery:** `chat-change-running`, `chat-change-landed`, `chat-goal`. `chat-first-run-2-T1`, `-3-T1` and `chat-solo-T10` hold.
- The remaining entries for these in `README.status.md` and `browser.status.md`'s 2.5 and 7.2 go.

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chats/inputs/request.md`, section 4.
- `temper/docs/design/client/README.md`, 5.7; `temper/crates/temper-client-domain/src/pages/delegation.rs`, `pages/task.rs`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/tests/{chat,cards,confirm,task,inbox}.rs`,
  `temper/crates/temper-client-shell-terminal/src/tests/**`: what the line makes stale.
- `temper/crates/temper-client-view/src/pages/{chat,task,inbox}.rs`: calls alone. `temper/tests/client/**`,
  `temper/tests/client-browser/**`, `temper/tests/one-process/**`: a walker in the browser that presses "Task N ↗" by its text: by its name
  it holds. **A change of meaning runs its readers' tests**: `delegation::facts`' are `temper-client-view`'s.
- The terminal's `chat_delegation_*.txt`, whose prints stay as they are but for what you name in your report; the journeys' prints.

Options, each pre-approved:

- `look::TASK_CARD` and `look::CARD` keep carrying the node; `TURN_TOOL` replaces `TASK_CARD` on a chat's page only if it reads
  better, in both shells.
- A goal the delegate's document doesn't yet say is tracked is a change's or a task's line until it does.
- A kind other than a change or a goal (report, verdict, job, approval) takes the same line with its kind's word, in lower case.

## Steps

1. The pen, the fold's line.
2. A delegate's line, `Facts.goal`, the words by shell; the cards' rules.
3. Renders beside the mock's `chat.html?id=T4` and `T3` at 1440, 1000 and 420; running, landed, held, goal, a proposal, a held task.
4. **Tests,** `tests/delegation.rs`: the browser's and the terminal's text of the link and one name; "Change", "Started goal" in the
   browser alone; `Facts.goal` both ways, in the domain's tests; the held token. Each bites. Gallery, goldens, prints.
5. **Check,** `notes/common.md`, clippy, push.

## Report

As `notes/common.md`, *Your report*.
