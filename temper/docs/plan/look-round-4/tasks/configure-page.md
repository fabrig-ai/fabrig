+++
id = "configure-page"
part = "configure-page"
kind = "feature"
needs = ["configure-address", "drops-connected"]
files = [
  "temper/crates/temper-client-view/src/pages/configure.rs",
  "temper/crates/temper-client-view/src/tests/configure.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
]
effort = "high"
minutes = 150
+++

# The configuring chat's page

## Goal

"Tell temper how you work" opens `Configure { project }`, the request's page, in the project's frame,
and no offer starts a chat at once any more. Every commit carries `Look-request:
configure-chat-in-settings`.

Acceptance:

- Every rule of `git diff 2c6f5a5146 53ba7876f8 -- temper/docs/design/client/journeys/overlay.css` is
  in `temper.css`, its block and `@media` as they stand, `send.svg`'s `url()` the stylesheet's own
  path (`temper.css` already reads it). Tokens for each class the page draws: `temper-session`,
  `-session-top`, `-session-foot`, `-session-composer`, `-session-hint`, `temper-convo`,
  `temper-crumbs`, `temper-back`, `temper-trail`, `temper-byline`, `temper-starter`,
  `temper-starters`; `look::THREAD` exists.
- The markup is `git show 53ba7876f8:temper/docs/design/client/journeys/configure-chat/01.html`'s
  `main`, names, classes and order; the words, `temper/docs/design/client/README.md`, 3.5, *The
  configuring chat's page*, each state of it.
- **The top bar** is one function, `pub(crate) fn top(...)`, `configuring-chat` reads it: the path,
  the level-1 heading, the line of what the runs use.
- **The line:** the first usable account's kind (`first_usable_kind`), " · ", the first model `me`
  serves for the project, as Connected's line chose them; either left out with no such fact.
- **The starters:** `Act::Starter { words }`, each its button, disabled while `Chats.start` is out.
- **The composer:** the text area, `Act::ChatWords`, Enter submits as on the chats page; "Send",
  `Act::StartChat`, disabled while the text is empty or the start is out; refused, the chats page's
  refusal under it, the words kept.
- **The offers:** Home's "Start" and the chats page's offer are `Act::Go { Address::Configure {
  project } }`; the chats page's is still a button. `Act::FirstChat`, `first_chat` and
  `FIRST_CHAT_WORDS` go. Home's "Start" isn't disabled by a pending start any more.
- **The rail:** "Settings" is current on `Configure`, in the rail and the tab bar.
- **With no such project in `me`:** what `Home` says.
- **The terminal:** the heading, the agent's line, "Ways to begin" with its numbered starters, the
  text area and "Send"; the path's links, numbered as links are.
- **Gallery:** `configure` (template 3 listed), `configure-typing`, `configure-sending`,
  `configure-refused`.
- `README.status.md`: the first half of *3.5, The first chat, The configuring chat's page* goes.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5, *The first chat*, *The configuring chat's page*; 5.3.
- `git show --format=%B -s 53ba7876f8`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/pages.rs`, `src/tests.rs`: the modules' lines.
- `temper/crates/temper-client-view/src/{words.rs,limits.rs}`, `src/tests/{fixtures.rs,look.rs,rail.rs}`.
- `temper/crates/temper-client-view/src/pages/{home.rs,chats.rs}`, `src/tests/{home.rs,chats.rs}`: the
  offers, and tests that pressed them.
- `temper/crates/temper-client-domain/src/{pages.rs,acts.rs}`, `src/pages/{chats.rs,chat.rs,task.rs}`,
  `src/tests/pages/{chats.rs,home.rs}`: what `Act::FirstChat`'s removal forces, a test of it moving to
  the starters or going with it.
- `temper/tools/gallery/src/worlds.rs`: template 3 in the states' `me`, those states alone.
- `temper/docs/design/client/browser.md`, 7.2, `temper/docs/design/client/README.status.md`.

Options, each pre-approved:

- `header.temper-session-top`: the role skein-ui draws as `header` with no banner's landmark clash;
  if there is none, a slot with the class, and say so.
- Focus at opening: as skein-ui offers it; if it offers none, leave it, and `account-configure` names
  it deferred, to a skein-ui draft.
- `autofocus`, `rows="1"`: what the view can set.
- `Send`'s words are the chat's `Send`; its look is the stylesheet's.

## Steps

1. Rules and tokens, `pages/configure.rs`, the arm in `root.rs`.
2. The offers, the removals.
3. **Tests,** `tests/configure.rs`: the page's node order against the request's in both shells where
   they differ; each act's event; each state; the phone's parts in the tree; a project that isn't
   the person's. Each bites.
4. **Gallery,** render at 1440, 1366, 1000 and 420.
5. **Check,** `notes/common.md`: `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-terminal -p temper-client-gallery -p
   temper-client-journeys -p temper-client-world`, clippy, push.

## Report

As `notes/common.md`, Your report, the overlay's rules by a list.
