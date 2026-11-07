+++
id = "client"
part = "client"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-domain/src/address.rs", "temper/crates/temper-client-domain/src/pages.rs", "temper/crates/temper-client-domain/src/pages/chats.rs", "temper/crates/temper-client-domain/src/route.rs", "temper/crates/temper-client-domain/src/tests/address.rs", "temper/crates/temper-client-domain/src/tests/pages/chats.rs", "temper/crates/temper-client-view/src/pages/settings.rs", "temper/crates/temper-client-view/src/pages/configure.rs", "temper/crates/temper-client-view/src/root.rs", "temper/crates/temper-client-view/src/frame.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/tests/settings.rs", "temper/crates/temper-client-view/src/tests/configure.rs", "temper/crates/temper-client-view/src/tests/fixtures.rs", "temper/crates/temper-client-shell-terminal/src/tests/golden/**", "temper/crates/temper-client-shell-web/temper.css", "temper/tools/gallery/**", "temper/docs/design/client/journeys/chat-draft/**", "temper/tests/client/tests/suite/configuring_chat.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 180
+++

# ui-ask-m1: "Ask temper" on Settings, and its page

## Goal

Settings' How work runs draws an "Ask temper" row under "Configure temper": "Start" to
a new page, `Ask { project }`, or "Go on" to the person's newest open Ask temper chat.
The page is the configuring chat's page with its own words and no starters, and its
"Send" starts a chat from template 5. Charter 11 and template 5 are another task's (the
stock); this task knows template 5 by its number, as it knows template 3.

Acceptance:

- `Address::Ask { project }`, `/projects/<project>/settings/ask`, printed and parsed as
  `notes/contracts.md`, section 3, gives it, under `Settings` in the frame's path as
  `Configure` is; a row in `tests/address.rs`'s round-trip table.
- `ASKING_TEMPLATE: u32 = 5` beside `CONFIGURING_TEMPLATE` (`pages/chats.rs`).
  `Chats` gains, beside `open_configuring` and `configure_chat`, the newest open chat
  whose executor is template 5's configured charter in the project, and a start from
  template 5 with the person's words and its template's own models. Unlike Configure,
  there is no fallback to template 1: with no template 5 in `me`, nothing starts.
- The view (`pages/settings.rs`): the row of `temper/docs/design/client/README.md`, 5.13
  (*Ask temper*), drawn with Configure's row's styles, after it in the same section,
  its "Start" a primary `button_link` to `Address::Ask`, or "Go on" to the open chat;
  not drawn while `me` lists no template 5 among the project's templates.
- The page (`pages/configure.rs`, sharing its builder with Configure's page rather than
  copying it: a parameter or a small enum of your naming): 3.5, *Ask temper's page*,
  with `notes/contracts.md`, section 3's words exactly, no list, note, starters or
  temper's own way card; its picker from template 5's `resolved_default`. With no
  template 5, "Send" is disabled and the composer says "Ask temper isn't set up in this
  project."
- Rows in the existing tests, none new but where named:
  - `temper-client-view/src/tests/settings.rs`,
    `start_goes_to_configure_and_go_on_to_the_open_chat`: Ask temper's Start's href and
    act, its Go on to an open Ask chat, and no row with no template 5, in both shapes
    the test already draws.
  - `temper-client-view/src/tests/configure.rs`: Ask's page: its title, heading, lede,
    composer and placeholder, and that no "Ways to begin" or starter is drawn.
  - `temper-client-domain/src/tests/pages/chats.rs`,
    `configure_and_new_chat_start_their_template_and_open_the_chat`: a start from Ask
    sends template 5 and names no repository; with no template 5, no `Send`.
    `settings_selects_the_newest_open_configuring_chat`: rows for Ask's newest open chat
    beside configuring's, one never taken for the other.
  - The fixtures' `me` (`temper-client-view/src/tests/fixtures.rs`, `configuring_me`)
    gains template 5 run by its own charter number, kept apart from template 3's.
- Regenerated, by their own tools: the terminal golden
  `temper-client-shell-terminal/src/tests/golden/settings.txt` (its later controls'
  numbers move), by `src/tests/render.rs` with `SKEIN_CODEGEN_WRITE=1`; the gallery
  (`temper/tools/gallery`: an `ask` sample of the page, `names.txt` kept sorted; the
  `settings-chat-open` row's `Part::Absent { role: Role::Link, name: "Start" }` narrowed
  to the Configure row, since Ask's Start now stands beside Go on, keeping what it
  checks for Configure); the journey `chat-draft`'s prints (`01.html`, `02.html`) by
  `temper-client-journeys`, and its recording by `temper-client-journeys again
  chat-draft` only if the reprint shows it stale.
- `temper/tests/client/tests/suite/configuring_chat.rs`: one row or story beside
  `the_page_starts_the_configuring_chat`, in its joined world, starting from Ask's page
  and finding a chat of template 5 with no repository, if the world's `me` can offer a
  template 5 by its own settings; otherwise skip it and say so (the stock's template 5
  lands in another task).
- `temper/docs/design/client/README.status.md`: delete the **Not yet** "3.5, *Ask
  temper's page*, and 5.13, *Ask temper*: ... `--template 5`." and nothing else (a
  departure from the implementer's text: the status must say what is built). The
  **Design departs** entry beside it stays.

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `notes/contracts.md`, sections 2 and 3.
- `temper/docs/design/client/README.md`, 3.5 (the route table's `Ask { project }` row;
  *The configuring chat's page*; *Ask temper's page*) and 5.13 (*Configure temper*,
  *Ask temper*).
- `temper/docs/design/client/greenfield/README.md`: the target, which draws neither row;
  the departure is recorded in `README.status.md`.

## Owns and may change

- Owns the files in the header.
- Readers: every `match` on `temper_client_domain::Address` the compiler names, in any
  crate (`temper-client-view`'s pages and tests, `temper-client-domain`'s pages and
  route, `temper-client-shell-web`, `temper-client-shell-terminal`, the gallery), gains
  `Ask` where it has `Configure`, doing for `Ask` what it does for `Configure` (the
  same frame, rail and back behaviour), and nothing else.
- `temper.css`: none is expected (the row and page take Configure's classes); a rule the
  page needs goes under `/* ==== ui: settings ==== */`.
- Help: `temper/docs/help/chat.md` and `settings.md` name Configure temper; add one
  sentence for Ask temper to `settings.md` where it names Configure, within its 12,288
  bytes, and name it in the report.
- Pre-approved: if `Chats::configure_chat` takes its template from a constant in a way
  that a second template needs reshaping, give it the template as a parameter; if the
  configuring page's words are matched by tests through `Word` variants, add Ask's as
  variants of their own.

## Steps

1. The rows above; run them; see them fail.
2. The address, the domain's chats, the view's row and page, the words.
3. Regenerate the golden, the gallery and the journey prints.
4. Narrow tests: `heavy cargo nextest run -p temper-client-domain --lib`, `-p
   temper-client-view --lib`, `-p temper-client-shell-terminal --lib`, `-p
   temper-client-gallery`, `-p temper-client-journeys`, and the joined world's test by
   name (`-p temper-client-world` or the package of `temper/tests/client`). Then
   `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p
   temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p
   temper-client-shell-web -p temper-client-gallery --all-targets -- -D warnings`, and
   `heavy cargo check -p temper-client-shell-web --tests` for the browser tier's
   stories, compiled, not run.
5. Commit and push.

## Report

As the implementer's text says; list each file regenerated and what moved in it.
