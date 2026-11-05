+++
id = "new-chat-page"
part = "new-chat-page"
kind = "feature"
needs = ["new-chat-address"]
files = [
  "temper/crates/temper-client-view/src/pages/new_chat.rs",
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/pages/configure.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/new_chat.rs",
  "temper/crates/temper-client-domain/src/pages/chats.rs",
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
]
effort = "high"
minutes = 80
+++

# A new chat's page

## Goal

`NewChat` is drawn as the mock's `chatPage` draws `id=new`, on the session's layout: the path, "New chat", the
`.meta-line`, the composer whose send arrow is the act. `configure.rs`' `top`, `foot` and `hint` become the shared
`pages/session.rs`. The playground stays where it is until `new-chat-playground`, and `Chats` keeps its composer
until `start-walks`.

Acceptance, the request's section 2 but its playground paragraph:

- `pages/session.rs` and `pages/new_chat.rs`, as `notes/contracts.md`. `Configure` and a configuring chat read
  `session::top` with `Path::Settings`, their prints and pages unchanged. `SESSION_RIGHT` holds their byline and
  `SESSION_ACTIONS`' parts; `SESSION_ACTIONS` goes.
- **The bar:** "PROJECT / Chats /" and the level-1 "New chat"; on a phone "‹", a link named "Chats"; with no project,
  "Chats /" alone. Its byline, a `BYLINE` line of `choice::new_chat_model`'s words (`chat-byline` puts the chooser
  behind it).
- **The thread:** a centred `META_LINE`, `NewChatIn` with OWNER/NAME from `pages::home::repository`, else
  `NewChatNoRepository` with the project's name.
- **The composer,** `session::foot`, named `NewChatWords`: a text area with that name and placeholder, focused as the
  page opens but while the switcher renames, `Act::ChatWords`, submitting on Enter while `ready_to_start`; "Send",
  `Act::StartChat`, disabled while the words are empty or `!ready_to_start`; `session::hint`. No "Start" button.
- **Sent:** while out, the arrow is disabled and the words stay. Made, `Chat { task }` opens with
  `Request::Replace`, in place of this address, from `NewChat`; from `Configure` as it does today. Refused, jig's
  words under the composer, kept.
- **No usable model:** `choice::composer(b, root, None, ...)` stays above the composer for now, which says "No model
  yet · Connect a model" (`chat-byline` splits it).
- **The look:** `.session-*`, `.thread`, `.meta-line`, `.composer`, `.composer-hint`, `.convo-foot` as the mock,
  under the marker; `ICON.send` as `.composer button`; at 760 px and under, as the mock's and `temper-session`'s.
- **Gallery:** `chat-new`, `chat-new-no-repository`; `chat-new`'s text area is focused. `configure`, `configure-chat`
  and `configure-chats-tag` hold.
- `pages::chats::main` no longer draws at `NewChat`. `ready_to_start` moves to `pages/new_chat.rs`, `Chats` reading it
  there while it keeps its composer.

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chats/inputs/request.md`, section 2.
- `temper/docs/design/client/README.md`, 3.5, *A new chat*, *The configuring chat's page*.
- `git show cea9fdb5d2 -- temper/docs/design/client/journeys/overlay.css`, what `look-round-5` lands on these
  rules: match its column (a scrollbar's gutter on both edges, at most 720 wide, centred) where you draw it, so that
  the catch-up is small. If it has landed on the trunk, keep its rules.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/pages/chat.rs`, `pages/home.rs`: calls to `configure::top`, `foot`, `hint`.
- `temper/crates/temper-client-view/src/pages/chats.rs`: the arm and `ready_to_start`'s move.
- `temper/crates/temper-client-view/src/tests/{chat,chats,configure,frame,look,rail,choice,welcome,playground,playground_start}.rs`,
  `src/tests.rs`: a test that sat at `NewChat` expecting the old page moves to `tests/new_chat.rs`. A test of the
  playground's start at `Address::NewChat` moves to `Address::Chats { .. }` for now.
- `temper/tests/client/**`, `temper/tests/client-browser/**`, `temper/tests/one-process/**`,
  `temper/tests/real-loop/**`, `temper/crates/temper/tests/support/**`: a walker that follows "+ New chat": its field's
  name and "Send". Those that start on `Chats` are `start-walks`'.

Options, each pre-approved:

- A person's project not in `me`: what `Configure` does, `Home`'s words.
- The main's `SESSION` beside `MAIN`, as today; the text area as `configure.rs`' composer. `rows`, autofocus and
  placeholders as skein-ui offers them.

## Steps

1. `session.rs`, moving the three functions; the readers' lines. The tokens and words.
2. `new_chat.rs`, the arm in `root.rs`. The replace on made.
3. The rules under the marker; the renders against the mock's `chat.html?id=new` at 1440, 1000 and 420.
4. **Tests,** `tests/new_chat.rs`: the node order; each word with and without a repository; focus; disabled while
   empty, not ready and out; the act's event; refused words kept; a made chat replacing the address; the path's links.
   Each bites. Gallery, goldens, prints (`notes/common.md`).
5. **Check,** `notes/common.md`, clippy, push.

## Report

As `notes/common.md`, *Your report*.
