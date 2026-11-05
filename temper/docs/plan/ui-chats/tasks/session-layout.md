+++
id = "session-layout"
part = "session-layout"
kind = "feature"
needs = ["chats-page"]
files = [
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/pages/agent.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/fixtures/task.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 85
+++

# One layout for every chat

## Goal

Every chat takes `look::SESSION`, with the shared bar and the one composer. A chat differs from a configuring chat only in
its path and, from `chat-byline`, its chooser. The conversation inside keeps today's look until `conversation-turns`.

Acceptance, the request's sections 3 and 5:

- `chat::main` has one branch: `MAIN`, `SESSION`; `session::top` with `Path::Settings` for a configuring chat and
  `Path::Chats` for the others, `‹` on a phone a link to `Chats`; `Heading::Chat`, its "TN · " hidden in the browser;
  `CONVO`, `THREAD`; `session_composer`, on every chat. `chat::composer` goes.
- **The bar's right part,** in this order: the byline, for now the line `configure.rs` printed for a configuring chat and
  `choice`'s words for a chat; the status, quiet: 12 px, `--gf-faint`, lower case by css, `.state-needs`' colour when it waits
  for the person (`STATE_NEEDS` when the token is `STATUS_YOURS` or `STATUS_WAITING`); Stop and Close, while allowed, each
  `.btn.quiet.small`; the spend, in the view and hidden in the browser.
- `crate::choice::composer(b, root, Some(task), ...)` sits above the session's composer for now; `chat-byline` moves it.
- **The context line** (`chat::context`): first in `THREAD`, in `.meta-line`'s look.
- **Words:** `MessagePlaceholder` "Reply to T4" and its name "Message"; `ChatSendHint`, on `Configure`'s too.
- **The send arrow:** `ICON.send` on ink, 36 px, radius 10, `--gf-line-2` while disabled, as `.composer button`. **Its
  column:** `.convo-foot > *` 720 px centred, on `Configure`'s too, sharing its edges with the turns.
- **Ended:** no composer, as today.
- **The rail stays on every chat;** at 760 px and under, rail and tab bar go, as `temper-session` already does.
- **Retired:** `look::CHAT` and `look::CHATBAR` become `AGENT_PAGE` and `TASKBAR`, as `notes/contracts.md`, and `chat.rs` reads
  neither, nor `ENTRY`, `COMPOSER_AREA`, `COMPOSER` or `HINT`. In `temper.css` `.temper-chat` and `.temper-chatbar` become
  `.temper-agent-page` and `.temper-taskbar`; every `:is(.temper-chat, .temper-session)` keeps both until
  `conversation-turns`. An agent's task's page renders exactly as before: compare `task-*` gallery pages.
- **Gallery:** `chat-solo-T10`, `chat-solo-T6`, `chat-reference-T1`, `chat-first-run-*`, `chat-closing`, `configure-chat` hold.
- `README.status.md`: 3.5, One layout for every chat, A chat's bar, and 5.9's token words leave the **Not yet**, but the
  chooser's (`chat-byline`).

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chats/inputs/request.md`, sections 3 and 5.
- `temper/docs/design/client/README.md`, 3.5, *One layout for every chat*, *A chat's bar*; 5.9, *The look*.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/tests/{delegation,choice,playground,playground_start,times,confirm}.rs`,
  `temper/crates/temper-client-shell-terminal/src/tests/**`, `temper/tools/gallery/src/**`: the tokens' new names, and what the
  layout makes stale. `src/tests/chat.rs` holds 98 tests: most read a token or a role, and change by a line each.
- `temper/tests/client/**`, `temper/tests/client-browser/**`, `temper/tests/one-process/**`, `temper/tests/real-loop/**`,
  `temper/crates/temper/tests/**`: a walker reading the chatbar's region, "Message T4…" or the old hint. **A change of meaning runs
  its readers' tests.**
- The terminal's goldens: the bar's parts reorder, and `Enter sends · Shift+Enter for a new line`. The journeys' prints.

Options, each pre-approved:

- The chat's region named by its title goes with the chatbar; if `tests/` finds a chat by that region, it reads the level-1
  heading.
- `chat::heading`, its parts, is `session::top`'s for `Heading::Chat`.
- Confirm's dialog (`crate::confirm::main`) keeps its place in the main.

## Steps

1. Rename the tokens and rules, mechanically, in its own commit.
2. `chat::main`, the bar, the composer. The rules under the marker.
3. Renders beside the mock's `chat.html?id=T3` in Hello, a configuring chat, a closed chat (`T5`), at 1440, 1000 and 420.
4. **Tests,** `tests/chat.rs`: the bar's order on a chat and on a configuring chat; the path each way; Stop and Close while
   allowed; `STATE_NEEDS`; placeholder and name; no composer when ended. Each bites. Gallery, goldens, prints.
5. **Check,** `notes/common.md`, clippy, push.

## Report

As `notes/common.md`, *Your report*.

## Rebase (2026-11-05, step merge, foreman ui-chats/session-layout/foreman-1)

rebase session-layout onto integration, preserving Changes additions and regenerating combined goldens
The merge conflicts with integration commit fcc47a6c5c (“Merge main Changes work while preserving Chats and NewChat”), merged by 8f55ab276d.
Resolve these four paths:
- `temper/crates/temper-client-view/src/words.rs`: retain integration’s Changes vocabulary, phrase IDs, ALL entries and byte allowances; retain this task’s “Reply to T” placeholder. Combine the segment changes: SEGMENTS becomes 2438, PHRASES stays 1402, ALL stays 806.
- `temper/tools/gallery/src/samples.rs`: retain integration’s four Changes samples and supporting behavior plus this task’s `chat-solo-T1`; N becomes 122. Preserve both additions in `temper/tools/gallery/names.txt` in sample order.
- `temper/crates/temper-client-shell-terminal/src/tests/golden/chat_delegation_running.txt`
- `temper/crates/temper-client-shell-terminal/src/tests/golden/chat_delegation_landed.txt`
Regenerate both goldens from the combined code using `SKEIN_CODEGEN_WRITE=1` and terminal test `tests::render::each_fixture_page_prints_as_its_golden_file`. They must include the Changes rail link alongside the session bar, nested conversation and shared composer; link numbers consequently shift.
Review the automatic merges in CSS, look tokens, choice, gallery tests and README.status.md to preserve both features.
Through heavy, with TEMPER_BUNDLE=none, run the vocabulary declaration test, gallery `names_are_the_promised_ones`, affected chat tests and terminal golden test. Inspect and report regenerated artifacts.
