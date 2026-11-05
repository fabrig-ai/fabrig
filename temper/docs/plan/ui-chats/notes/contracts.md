# ui-chats: the run's binding calls

The shapes that cross tasks. A task builds to this note; a sketch here binds names and signatures, and
the bodies are the implementer's. Design: `temper/docs/design/client/README.md`, 3.5, 5.5, 5.6, 5.7, 5.9;
`temper/docs/design/client/browser.md`, 2.3, 2.5.

## Addresses

```rust
// temper-client-domain/src/address.rs
pub enum Address { /* ... */ Chats { filter: ChatsFilter }, NewChat, /* ... */ }
```

`NewChat` prints as `/chats/new` and reads back; a query on it is dropped, the address replaced. `/chats/`
and a number stay `Chat { task }`. `/chats?at=start` reads as `/chats`: a field of another name, dropped and
replaced.

`NewChat` opens `pages.chats` as `Chats` does, with the page's watches. A move between `Chats` and `NewChat`
keeps that state whole: its words, the closed chats it read.

## The session's module

`configure.rs`' `top`, `foot` and `hint` move to a new `temper-client-view/src/pages/session.rs`:

```rust
/// Where a session's path leads: a configuring chat's through Settings, every other's through Chats.
pub(crate) enum Path { Chats, Settings }
/// Opens the session's bar: the path and the heading. Its caller adds the right part and closes it.
pub(crate) fn top(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings,
                  project: Option<u32>, path: Path, heading: Heading<'_>);
/// A page's heading is its words; a chat's keeps "TN · TITLE" as its name, and a browser shows the title.
pub(crate) enum Heading<'a> { Words(&'a [u8]), Chat { whole: &'a [u8], title: &'a [u8] } }
/// Opens `SESSION_RIGHT`, the bar's right part; its caller closes it.
pub(crate) fn right(b: &mut Builder);
pub(crate) fn foot(b: &mut Builder, name: &[u8]);
pub(crate) fn hint(b: &mut Builder, w: &Words);
```

`SESSION_RIGHT` (`temper-session-right`, the mock's `.session-top .right`) is a flex row, so that the next
plan can add its mode group and "Start goal…" after the byline. It replaces `SESSION_ACTIONS` and holds the
byline, on `Configure` too.

`pages/new_chat.rs`:
`pub(crate) fn main(b, root, words, bindings, forge, text_limit, times)`, as `chats::main`'s arguments are.

## The chooser

`temper-client-view/src/choice.rs`:

```rust
/// The bar's byline: a disclosure named "Change model", described by the byline's words, while the chooser
/// is offered; otherwise a `BYLINE` paragraph of the same words (client/README.md, 3.5).
pub(crate) fn byline(b: &mut Builder, root: &Domain, task: Option<&c::Task>, w: &Words, bindings: &mut Bindings);
/// What sits above a session's composer: no usable model's line, or the chooser's feedback.
pub(crate) fn above_composer(b: &mut Builder, root: &Domain, task: Option<&c::Task>, w: &Words,
                             bindings: &mut Bindings, times: &mut Times);
```

`composer` stays for `pages/agent.rs` alone. The byline's words are "T4 · " on a task, then account, " · ",
model: the composer's line with its "Using " and its "Manage usage" left out; "Manage usage" is the panel's
last control. The disclosure carries `State::FoldsAway` from skein-ui, `BYLINE` its token.

## The conversation

- The scroller is the log: skein keeps a log's end in view (`skein/docs/design/ui-web.md`, 7) and acts on
  the `[role=log]` element. So on a chat, `[role=log]` is the mock's `.thread`, inside `THREAD`, and its one
  child is a slot `THREAD_INNER` (`temper-thread-inner`), the mock's `.thread-inner`.
- "Older", the turns, the unanchored cards, the end's line and the acts' status are all inside
  `THREAD_INNER`, in today's order, so that everything scrolls together as the mock's does.
- An agent's task's page keeps its log as it is: `Drawn::Lines`, with no `THREAD_INNER`.

## Tokens

| Token | Class | The mock's |
|---|---|---|
| `CHATS_PAGE` | `temper-chats-page` | `.main` and `.col` |
| `CHATS_INTRO`, `CHATS_KICKER`, `CHATS_LEDE` | `temper-chats-intro`, `-kicker`, `-lede` | `.intro`, `.kicker`, `.lede` |
| `CHATS_BAND`, `CHATS_TAG` | `temper-chats-band`, `-tag` | `.band.quiet-band`, `.tag` |
| `CHATS_SECTION`, `CHATS_SECTION_HEAD` | `temper-chats-section`, `-section-head` | `.section`, `.section-head` |
| `CHATS_ACT`, `CHATS_MORE` | `temper-chats-act`, `-more` | `.btn.small`, `.btn.quiet.small` |
| `CHATS_ROWS`, `CHATS_ROW`, `CHATS_ROW_ENDED` | `temper-chats-rows`, `-row`, `-row-ended` | `.goal-rows`, `.goal-row`, `.goal-row.ended` |
| `CHATS_TOP`, `CHATS_LINE` | `temper-chats-top`, `-line` | `.top.mono`, `.line` |
| `STATE_NEEDS` | `temper-state-needs` | `.state-needs` |
| `SESSION_RIGHT` | `temper-session-right` | `.session-top .right` |
| `META_LINE` | `temper-meta-line` | `.meta-line` |
| `THREAD_INNER` | `temper-thread-inner` | `.thread-inner` |
| `TYPING` | `temper-typing` | `.typing` |
| `TURN_TOOL`, `TOOL_FILE` | `temper-turn-tool`, `temper-tool-file` | `.turn-tool`, `.turn-tool .file` |
| `BY_DELEGATE` | `temper-by-delegate` | `.turn-agent`, its "TN" kept |

`FILTERS` and `EMPTY` stay, with new rules under `.temper-chats-page`. `STARTERS` serves the playground's
suggestions as it does the configuring page's.

An agent's task's page keeps its look: `look::CHAT` becomes `AGENT_PAGE` (`temper-agent-page`) and
`look::CHATBAR` becomes `TASKBAR` (`temper-taskbar`), every rule of `temper.css` following. The request
retires `CHAT` and `CHATBAR` from chats, and `pages/agent.rs` is outside it. `ENTRY`, `COMPOSER_AREA`,
`COMPOSER` and `HINT` stay for that page alone. `SIDEBAR` goes.

## Words

| Word | English |
|---|---|
| `ChatsKicker` | "PROJECT · chats" |
| `ChatsHeadlineIn` | "Talk to an agent about OWNER/NAME." |
| `ChatsHeadline` | "Talk to an agent." |
| `ChatsLede` | "A chat answers a question or changes the code." |
| `ChatsNoRepositoryLine` | "Chats in PROJECT talk and plan, but change no code until a repository is in." |
| `ChatsNone` | "None." |
| `NewChatIn` | "Say what you want; the agent works in OWNER/NAME." |
| `NewChatNoRepository` | "Say what you want. PROJECT has no repository yet, so the agent can talk and plan but not change code." |
| `NewChatWords` | "What do you want done?" |
| `MessagePlaceholder` | "Reply to T{}" |
| `ChatSendHint` | "Enter sends · Shift+Enter for a new line" |
| `DelegationOpen` | "Open" |
| `DelegationLineChange` | "Change" |
| `DelegationLineGoal` | "Started goal" |
| `ChatWorking` | "Writing" |

"No repository yet" is `NoRepositoryYet`, "Adopt a repository" `ForgeAdopt`, "+ New chat" `NewChatLink`,
"New chat" `NewChat`. A word that no page uses any more goes. `PHRASES`, `SEGMENTS` and `ALL` in `words.rs` are
exact, not ceilings: `english_phrase_declarations_match_the_vocabulary` checks them, and `SKEIN_CODEGEN_WRITE=1`
rewrites `ALL`.

## Reading the repository

`temper_client_domain::pages::home::repository(root, project)` becomes `pub`; "OWNER/NAME" is its
`Repository::In { owner, name }`, and anything else is "none" for the words above. The band's condition stays
`repository_context`'s: `forge::listed::without_repository(...) == Some(true)`.

## The gallery

`chat-byline-open` needs an open disclosure, which the page part holds and the print does not. A `Sample`
gets a way to open one disclosure in its print, as `page()` already opens a dialog with `.replace`.
