+++
id = "new-chat-playground"
part = "new-chat-playground"
kind = "feature"
needs = ["start-walks"]
files = [
  "temper/crates/temper-client-view/src/pages/new_chat.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/tests/playground.rs",
  "temper/crates/temper-client-view/src/tests/playground_start.rs",
  "temper/crates/temper-client-view/src/tests/new_chat.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/tests/**",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
]
effort = "high"
minutes = 70
+++

# The playground's start moves to NewChat

## Goal

The first run's playground starts on `NewChat`: everything that opened the Chats page for it opens `NewChat`, and the
frame draws its banner.

Acceptance, the request's section 2, *The playground*, and `temper/docs/design/client/README.md`, 4.8, 5.1, 5.6:

- `route.rs`' `playground` shows `Address::NewChat`, and so does anything else that opened `Chats` for the playground.
  `chats::playground_start` moves to `pages/new_chat.rs`, and answers at `NewChat` alone.
- On `NewChat`, above the `META_LINE`'s place: the status ("Ready", "Starting", "Not started") as a quiet line; the
  context line, OWNER/NAME alone; the failure and "Try again". Under the `META_LINE`, the suggestions, "Suggested
  requests", in `STARTERS`, each button `.btn.small`, disabled as today. While the playground's start is drawn, the
  byline and the composer are as on any `NewChat`.
- `pages::chats::main` draws no playground part.
- **The banner:** `root.rs` draws `frame::playground_banner` before the main on `Chats` and `NewChat` while the playground
  runs or is leaving, and on every other page while it is leaving, as `chats.rs` no longer does. A session main fills
  what the banner leaves, with no page scroll.
- **Words:** `5.6`'s, in `new_chat.rs`.
- **Gallery:** `chat-new-first-run-1`, the playground's start; `chats-first-run-1` is now the first run's Chats, an empty
  list, and `chats-first-run-5` the Chats after the first plan connects. The gallery's tests that read the three
  suggested requests read `chat-new-first-run-1`.
- A chat started from a suggestion opens with `Request::Replace`, as a typed one.
- `README.status.md`'s **Not yet** entry for 4.8 and 5.6 goes.

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chats/inputs/request.md`, section 2.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/tests/{frame,welcome,connect_forge,chats}.rs`: a test of the banner or the start
  whose address moves.
- `temper/crates/temper-client-view/src/frame.rs`: the banner's condition if you put it there.
- `temper/tests/client/tests/suite/playground.rs`, `welcome*.rs`, `first_run_hooks.rs`; `temper/tests/one-process/**`,
  `temper/tests/client-browser/**`, `temper/crates/temper-playground/**`, `temper/crates/temper-client-shell-web/src/playground/**`:
  a step that sees the playground's start on the Chats page, or reads the first run's words there.
- The terminal's `first_run.txt` and other goldens; the journeys' prints.

Options, each pre-approved:

- `Word::ProjectPane`, `Word::Ready` and the like go if nothing reads them.
- `Welcome`'s whole state stays `Chats`: while a playground runs it opens `Chats`. If a test expects the start there, it
  now goes to `NewChat` by its link.
- `welcome-playground`'s recording may go stale; leave it to `playground-recording`.

## Steps

1. The route, the view's move, the banner.
2. The rules; a render of `chat-new-first-run-1` at 1440, 1000 and 420, no mock to match: the mock has no playground, so
   match `.starter` and `.meta-line`.
3. **Tests:** the start's states at `NewChat` (`tests/playground_start.rs`); none at `Chats`; the banner's three cases
   (running on `Chats`, running on `Chat`, leaving on `Chat`); a suggestion's act. Each bites. Gallery, goldens, prints.
4. **Check,** `notes/common.md`, and `temper-playground`'s tests, clippy, push. Of the browser tier, the playground's
   story needs its own bundle: `heavy cargo check --tests -p temper-client-browser-world`, and say it ran no further.

## Report

As `notes/common.md`, *Your report*.
