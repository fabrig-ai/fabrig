+++
id = "needs-frame"
part = "needs-frame"
kind = "feature"
needs = ["needs-session"]
files = [
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
]
effort = "high"
minutes = 80
+++

# The rail's count and link, and the toast

## Goal

The rail's and the tab bar's "Needs you" go to the open project's filter and count that project's
open needs from one standing watch. Any page draws the toast, with its act.

Acceptance:

- With a project's frame showing, one `Subject::Inbox` watch stands at `needs::handle()`, opened once,
  closed on sign-out, per `temper/docs/plan/ui-needs/notes/contracts.md`, *The standing count*.
- "Needs you", in `frame.rs`'s `entries`, goes to `Address::Inbox { filter: Filter { project:
  Some(project), kind: None, goal: None } }`. Its count is `Session::count`: none at zero, "Needs you
  3", and "Needs you 3+" where cut, the number, with its "+", in `look::NAV_COUNT`. A card marked
  later still counts; an answered one doesn't.
- `navigation` (the banner's, on no project's page) keeps `me.needs_total` and its link as they are.
- The toast is drawn last in the page by a new `frame::toast`, called from `root.rs` before the page's
  `builder.close()`: a `Role::Status` of a new token `look::TOAST` (`temper-toast`), its words a text,
  and for `Toasted::Answered` while `Session::held(object)` a button "Undo" binding
  `Act::Needs(needs::Act::Undo { object })`. It reads in the terminal's print too.
- The toast's CSS is the mock's `.toast`, under the marker: fixed, centred, bottom 28, z-index 50,
  `--gf-ink` ground, white, 14px, its button underlined; bottom 88 at the phone's width. It shows while
  drawn, with no transition of the mock's `.show`, since drawing is its state.
- All the tests below pass; the journeys' reprint names `welcome-playground` alone as stale
  (`journey-again` records it). If it names another journey, say so in your report: `journey-again`
  takes every journey you name.

## Inputs

- `temper/docs/plan/ui-needs/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5, *What the client keeps*, *The toast* and *The rail's
  count*; 5.3, *The rail*.
- `temper/docs/design/client/greenfield/mock.css`, `.toast` (416-418, and the phone's); `app.js`,
  `toast`, `navItems`, `tabbar`.
- `temper/docs/design/client/journeys.md`, 8: why a new watch stales a recording.

## Owns and may change

Owns `files`, and may change as follows:

- `temper-client-domain`: `src/pages.rs` (the session's `configure` at the end of `step`, as
  `pages.accounts.configure` is; the route of `Told::Changed` and `Told::Ended` for `needs::handle()`,
  which counts as shown; `link_changed`'s `Unwatch`), `src/tests/pages/needs.rs`.
- `temper-client-view`: `src/root.rs` (the one call), `src/look.rs` (`TOAST`, `NAMES`), `src/words.rs` (a
  word each: `NeedsYouCountCut`; `ToastAnswered`, "Answered: LABEL"; `ToastLater`; `ToastUndo`, "Undo";
  `AnsweredReply`, `Replied: “WORDS”`), `src/limits.rs` (derived counts, with their comments).
- `temper/crates/temper-client-shell-web/temper.css`, under `/* ==== ui: needs ==== */`.
- The gallery: where a state's rail shows a count, its world delivers the same rows to the standing
  watch: `temper/tools/gallery/src/worlds.rs` (`world_in_layout`) and `src/samples.rs` (`home_domain`'s
  `me.needs_total`, `Subject::Inbox` answer). `src/tests.rs`' rows that read a rail's count or its
  `href`. No new state.
- Readers of the opened watch or the rail's words: `temper-client-domain/src/tests/pages/{home,inbox}.rs`
  and any domain test asserting the requests of a start with a project; `temper-client-view/src/tests/`
  `{rail,frame}.rs` (`rail_and_tabbar_keep_the_order_addresses_current_states_and_count` sets
  `needs_total` 3, and now delivers rows; `navigation_shows_the_inbox_count_only_when_nonzero` stays);
  `temper/tests/client/tests/suite/chat.rs` (Link "Needs you"), `temper/tests/one-process`.
- The journeys' pages: `heavy cargo run -p temper-client-journeys` reprints; it exits 1 for the stale
  one, its print left; commit what it printed.
- `temper/docs/design/client/README.status.md`: take the toast and the standing watch out of its 3.5
  entry.

Options, each with its default:

- The watch's condition is the domain's: a project is open and the address isn't `Welcome`, which is
  what `frame::rail_project` draws on. If a state differs from the rail in a way a journey notices, make
  the domain's condition closer, and say so.
- `domain.watches` or a view bound too small for one more watch in a test's limits: raise the derived
  count, with its comment. It is not a budget.
- `Told::Ended` for the watch: `count` falls to zero; nothing retries beyond what the client does.

## Steps

1. Domain: the watch, `count`, its tests: opened once a project's frame shows and not before; not
   twice; no watch on `Welcome`; closed and reopened across sign-out; a row of another project isn't
   counted; an answered card leaves the count at once and a later one stays; cut.
2. View: the rail's link and count; `frame::toast`; the words; their tests, in both shells, in
   `src/tests/rail.rs` and a new `src/tests/toast.rs` (its `mod` line in `src/tests.rs`): the toast
   drawn last in the page, on Home and on Chats; Undo's binding; no button past the hold.
3. CSS, the gallery's deliveries, the readers.
4. `heavy cargo nextest run -p temper-client-domain --lib`; `-p temper-client-view --lib`; `-p
   temper-client-gallery`; `-p temper-client-world` (the crate at `temper/tests/client`); `-p
   temper-client-shell-web`; the reprint. fmt, clippy and the checks of the implementer's text. Commit
   and push.

## Report

As the implementer's text asks, and which journeys went stale, by the reprint's words.
