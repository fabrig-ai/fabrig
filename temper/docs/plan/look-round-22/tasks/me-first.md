+++
id = "me-first"
part = "me"
kind = "feature"
needs = ["welcome-end"]
files = [
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/outside.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/crates/temper-client-view/src/tests/fixtures.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-shell-web/src/focus.rs",
  "temper/crates/temper-client-shell-web/tests/suite/host.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 90
+++

# While `me` is read, the top bar alone

## Goal

The request's task 2 (`temper/docs/plan/look-round-22/inputs/request.md`). Before `me` is read, `root::build` (`temper/crates/temper-client-view/src/root.rs`, about lines 110 to 363) draws the top bar and then the address's banners and main: `/chats` draws "Project · chats", its filters and "+ New chat", and `/chats/new` its composer, until "No project yet" settles about 940 ms later. No page body draws until `me` is read: the top bar alone. The web shell's heading focus, which looks for a new entry's main on the draw that opens it, waits for the main when there is none yet.

Acceptance, by `temper/docs/design/client/README.md`, 5.1, *while `me` is read*, and `temper/docs/design/client/browser.md`, 2.3, *A heading drawn after its page*:

1. **The rule's test, in `outside.rs`** beside `no_project`:

   ```rust
   /// `me` is being read where the page needs it: the page draws the top bar alone
   /// (temper/docs/design/client/README.md, 5.1).
   pub(crate) fn reading_me(root: &temper_client_domain::Domain, words: &Words, ways: &crate::sign_in::Ways) -> bool
   ```

   true when the link is neither signed out nor out of date (`crate::frame::unavailable(root).is_none()`), there is no `Authorize` page (`root.authorization()`) and no approval page (`root.approval()`), the sign-in page is not offered (`crate::sign_in::offered`), and `root.pages().me(root.client())` is `None`.
2. **`build`,** in both shells, with `let reading = crate::outside::reading_me(root, &view.forge_words, &view.sign_in);` computed beside `no_project`:
   - `terminal` is false while `reading`, so the browser's terminal look waits and the top bar draws;
   - neither the accounts' banner (`choice::banner`) nor the playground's (`frame::playground_banner`, both of its calls) draws while `reading`;
   - the main's chain draws nothing while `reading`: a branch after the `unavailable` one and before `no_project`'s, at every address, `Welcome` included;
   - the rail, the tab bar, the toast and the start sheet are as they are (the rail and the tab bar need `me` already).
3. **The web shell's heading focus** (`temper/crates/temper-client-shell-web/src/focus.rs`, `HeadingFocus`): on a new entry whose draw has neither a `main h1` nor a `main`, it waits for the main; on a later `finish` with no focus request, the wait still on, it does what a new entry's draw does (the heading, or the main with the heading pending). A `focusin` meanwhile, or a focus request, ends the wait, as it ends the heading's. `HeadingFocus::new` and `finish`'s signature stay; its state inside is the implementer's (an enum of what it waits for is the sketch's).
4. **The rows,** in the existing table test `the_top_bar_says_the_links_state_and_the_persons_name` (`temper/crates/temper-client-view/src/tests/frame.rs`, about line 7), no new test function:
   - its roots become `(root, reading)` pairs: `no_project_root()` false; `temper_client_domain::new(..)` false (its link starts signed out, `jig/crates/jig-client-domain/src/domain.rs`, about line 60, so it draws the "Signed out" column; if it reads otherwise, set the row by its link and say so); `signed_out()` and `out_of_date()` false; and three new rows, true, from a new fixture `pub fn reading_me_at(address: temper_client_domain::Address) -> temper_client_domain::Domain` in `tests/fixtures.rs` (unlinked, `StartUnlinked`, `Start` at the address with `With::Transport`, `Served`, and no `me`, as `tests/chat.rs`'s `no_me_no_line` builds its root): `Address::Chats { filter: ChatsFilter::All }`, `Address::NewChat { draft: false }` and `Address::Welcome { step: Step::Welcome }`;
   - for each row and both shells, beside what it checks now: a `Role::Main` is drawn exactly when the row is not `reading`; while `reading`, no `Role::Region` but the link region (`crate::frame::link_region()`), so no banner.
5. **Tests whose roots read no `me`** (this cut's survey), kept to what they check:
   - `tests/home.rs`, `home_unknown_and_ended_forges_draw_only_known_facts`, its second half (about lines 1296 to 1314, `unlinked` with no `me`): deliver `me` before it draws (`fixtures::deliver_me` with the `Me` open's watch, or the root from `fixtures::root_at` with the half's forge events); say which.
   - `tests/chat.rs`, `no_me_no_line` (about line 4806): its state, a composer drawn before `me`, no longer exists; delete it (the `NewChat` row of acceptance 4 covers the page before `me`).
   - `temper/crates/temper-client-shell-web/tests/suite/host.rs`, `a_start_at_a_stale_filter_shows_its_printed_address` (about line 23): its root reads no `me`; keep its `show` assertion and `!temper-filters`, and in place of `contains("temper-needs")` assert the top bar (`temper-top`) and no `<main`.
6. **Prints:** the journeys' marks are taken with `me` read (the survey found every committed page past it), so none should move: `heavy cargo nextest run -p temper-client-journeys` passes; if a print moves, reprint it (`notes/common.md`, *Prints*) and say which.
7. **Status:** delete the **Not yet** entries naming `me-first` in `temper/docs/design/client/README.status.md` and `temper/docs/design/client/browser.status.md`. No other design line.
8. Runs through `heavy`, counts in the report: `cargo nextest run` for `-p temper-client-view`, `-p temper-client-shell-web`, `-p temper-client-shell-terminal`, `-p temper-client-gallery`, `-p temper-client-world` and `-p temper-client-journeys` (this changes what every page draws before `me`: every reader's tests run); `cargo check --tests` for `-p temper-client-browser-world`, `-p temper-one-process-world` and `-p temper`; `cargo check -p temper-client-shell-web --target wasm32-unknown-unknown`; `cargo clippy -p temper-client-view -p temper-client-shell-web --all-targets -- -D warnings`, and `cargo clippy -p temper-client-shell-web --target wasm32-unknown-unknown -- -D warnings`.

One commit or more, each with the trailer `Look-request: outside-pages`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-22/notes/common.md`.
- `temper/docs/plan/look-round-22/inputs/request.md`, task 2.
- `temper/docs/design/client/README.md`, 5.1: *the top bar* ("Until `me` is read"), *while `me` is read*, *the main*, and *The pages not about the work*.
- `temper/docs/design/client/browser.md`, 2.3, *A heading drawn after its page*.
- The code: `root.rs`, `build`; `outside.rs`, `top` and `no_project` (as `welcome-end` left them); `frame.rs`, `rail_project` and `unavailable`; `sign_in.rs`, `offered`; `temper-client-shell-web/src/lib.rs`, `finish` (about line 351), which hands `HeadingFocus::finish` the heading and `new_page`.

## Owns and may change

Owns `reading_me`, `build`'s gate, `HeadingFocus`'s wait, the fixture, the rows and the tests of acceptance 5, and the status entries. Not `top` (as `welcome-end` left it), the pages' own code, the domain, the TUI's crates (they never call `build`), or the gallery's worlds (each answers the `Me` open).

May change, keeping what each checks for everything else: another test that draws a page with `me` unread and reads its main, a banner or the terminal look (found by its failure in the runs of acceptance 8): give it `me` as acceptance 5 does, or, where its state no longer exists, delete it; name each. A client-world story (`temper/tests/client`) that presses a page's control before `me` is read: wait first for what it presses (its `See`), and name it.

Options, each decided:

- **The terminal look while `me` is read:** add a row to the table of acceptance 4 with the terminal look on (the look record in `Start`'s `kept`, as `tests/terminal.rs` sets it) if a root reading `me` can carry it in a few lines; otherwise leave it to the code and say so.
- **Clippy objects to the empty branch:** restructure the chain so that it draws nothing while `reading` (an early `if !reading` around the address's pages, say), keeping the order of the branches before it; say how.
- **`lib.rs`'s title before the main** comes out empty and is set when the main comes: leave it, and say so.

## Steps

1. Read the request's task 2 and the Inputs; `reading_me` and `build`'s gate (acceptance 1 and 2).
2. The fixture and the rows (acceptance 4); bite them: draw the main while `reading` again, see the `Chats`, `NewChat` and `Welcome` rows fail on the main, and restore.
3. The tests of acceptance 5; `HeadingFocus`'s wait (acceptance 3), compiled for wasm32; its bite is the browser tier's, after landing, and the report says so.
4. The journeys' check (acceptance 6); the status entries (acceptance 7); the runs of acceptance 8; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`.
5. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| While `me` is read, the top bar alone: no banner, no main, at `/chats`, `/chats/new`, the welcome and every other address | landed (`root.rs`, `outside::reading_me`) |
| The browser's terminal look waits for `me` too | landed (`build`'s `terminal`) |
| A new entry with no main yet: its heading takes the focus when the main comes | landed (`focus.rs`), compiled; the browser tier's after landing |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): compiling, prints and focused tests establish no rendered equality |

## Report

The implementer's report: `reading_me` and the gate as written, `HeadingFocus`'s wait, the rows and the tests changed or deleted, the bite and its failure, the options taken, the runs' counts, and the account as committed.
