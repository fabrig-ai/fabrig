+++
id = "terminal-browser"
part = "terminal-browser"
kind = "feature"
needs = ["terminal-commands", "web-screen"]
files = [
  "temper/crates/temper-client-domain/src/boundary.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/pages/terminal.rs",
  "temper/crates/temper-client-domain/src/tests/pages/terminal.rs",
  "temper/crates/temper-client-shell-web/src/host.rs",
  "temper/crates/temper-client-shell-web/src/lib.rs",
  "temper/crates/temper-client-shell-web/src/records.rs",
  "temper/crates/temper-client-shell-web/src/host_tests.rs",
]
effort = "high"
minutes = 110
+++

# A chat's terminal: kept per browser, and full screen with Esc its own

## Goal

Land the look request `chat-terminal-esc`, and `chat-terminal`'s *the mode is remembered per browser*: ">_" asks for full screen and the browser keeps the look; "Exit" leaves both; in full screen Chromium gives a short Esc to the terminal, which closes a list, and a long one leaves full screen; where the browser keeps Esc (Firefox, Safari), leaving full screen closes the list, and the terminal stays. The look's record is kept in skein's kept store, read at start, so another tab and a reload open in it. The shapes are `notes/contracts.md`, 1, 2 and 7, binding.

Acceptance, by the tests below. No browser runs, here or in any step:

1. `Event::LookKept`, `Event::ScreenLeft`, `Request::KeepLook` and `Request::Screen` exist as contract 7 has them, and `Act::Terminal`'s acts give the requests it says, in the step of the act.
2. `Event::ScreenLeft` does what `Act::Terminal(Dismiss)` does: a list closed, a confirmation drawn as a list cancelled, the box's "/" words cleared; nothing on a page with none.
3. `records::encode_look` and `decode_look` write and read the three bytes of `temper/docs/design/client/README.md`, 3.5, *Its look, kept per browser*; any other value decodes as none.
4. The host gives `Event::LookKept` at start where the kept store holds a `look` record that decodes, and none otherwise; `Request::KeepLook` becomes `Duties::keep_look`, `Request::Screen` `Duties::screen`, and `Report::Screen(Left)` `Event::ScreenLeft`.
5. The shell reads the kept store at start, writes `keep_look` through `Store::kept`, outside the transport's whole, and carries `screen` by `skein_ui_web::screen::web::Screen` in the act's turn, before the transport, as it carries a tab; its reports reach the host through the inbox.
6. A reload opens the terminal again, not in full screen: a start with the record gives the look and asks no full screen.
7. `temper-client-shell-web` passes clippy for `wasm32-unknown-unknown` with warnings denied.

## Inputs

- `temper/docs/plan/look-round-16/notes/common.md`: the standing rules, the requests and how to read them.
- `temper/docs/plan/look-round-16/notes/contracts.md`, 1, 2, 6 and 7: binding.
- `temper/docs/design/client/README.md`, 3.5, *A chat's terminal*: *Its look, kept per browser*, *Entered*, *Left*, *Keys*; 7, *A sign-in's address in the small store*, the record this one is made as.
- `skein/docs/design/ui-web.md`, 10, *The kept store*; 11, *A person's gesture*, *Full screen*.
- The request `chat-terminal-esc`: its message and `terminal.js`'s `fullscreenchange` handler.

## Owns and may change

Owns the header's files.

Reader edits allowed:

- `temper-client-domain/src/pages.rs`: the arm that takes `Event::ScreenLeft` to `Dismiss`'s work and `Event::LookKept` to the look, and the requests on `Act::Terminal`, a few lines, if `route.rs` can't reach them; another draft's task (`terminal-cards`) changes `pages.rs` beside you, so keep to those lines.
- An exhaustive `match` on `Event`, `Request`, `Report` or `Duties` elsewhere in temper (`temper-client-shell-web/src/journey/*`, the journeys' replay host, the gallery's driver, `temper/tests/client`'s world): the arm or field the compiler asks, the replay and the gallery doing nothing with the screen and keeping no look.
- `temper/crates/temper-client-shell-web/src/host_tests*.rs` beside `host_tests.rs`: rows where the start's records are tested.

Options, each decided:

- **Where the codec lives:** `records.rs`, beside the sign-in's, as contract 7 says; the domain sees only `Look`.
- **A write refused:** nothing on the page; the shell drops it.
- **Two tabs:** each reads the record at its own start; neither is told of the other's write (skein's `ui-web.md`, 10).
- **Full screen asked outside an act's turn:** never; the requests come only from `Act::Terminal`'s acts, which a person's press makes.

Leave alone: the view (`temper-client-view`), `temper.css`, the gallery, skein (`web-screen` gave what this needs).

## Steps

### 1. The domain

`boundary.rs` gains contract 7's events and requests; `pages/terminal.rs`'s acts ask them; `route.rs` takes `LookKept` and `ScreenLeft` to the pages.

### 2. The shell

`records.rs` the look's key and codec; `host.rs` the report, the duties and the start's record; `lib.rs` the kept store, the `Screen` and its inbox wrap, and `carry`'s full screen beside the tab.

### 3. The tests

- **Rows of what exists:** `the_look_moves_by_its_acts` (`tests/pages/terminal.rs`) checks each act's requests: `KeepLook` with the new look, and `Screen` for `Enter` and `Leave`; and gains rows for `LookKept` and `ScreenLeft` with a listing open and with none.
- **New, shell:** `host_tests.rs`, `the_look_is_kept_and_full_screen_carried`: a table of a start with a kept record (`"100"`, `"011"`, `"1"`, `"abc"`, none) and the look it gives; a `KeepLook` and its duty's bytes; a `Screen` and its duty; `Report::Screen` of each kind and the event or none.
- `records.rs`'s codec: rows in the same table, or beside the sign-in's codec test where one loops.

Break each rule once by hand (a record's length, `Enter`'s screen request, `Left`'s event), run the test, see it fail for that reason, restore, and name each in the report.

### 4. The checks

One command at a time:

- `heavy cargo nextest run -p temper-client-domain --lib -E 'test(=NAME)'` and `heavy cargo nextest run -p temper-client-shell-web --lib -E 'test(=NAME)'` for each test you added or changed.
- `cargo fmt`, then `python3 -I -B temper/scripts/old_forms.py`.
- `heavy cargo clippy -p temper-client-domain -p temper-client-shell-web --all-targets -- -D warnings`, and `heavy cargo clippy --target wasm32-unknown-unknown -p temper-client-domain -p temper-client-shell-web -- -D warnings`.
- `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-gallery -p temper-client-journeys -p temper-client-world`: the journeys' replay reads the same host.
- `python3 -I -B tools/citations/citations.py --base origin/main`.

### 5. Commits

- **Trailers:** the kept look's commits `Look-request: chat-terminal`; full screen's and Esc's `Look-request: chat-terminal-esc`. One line above the other trailers.
- Push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report, and, for the account `terminal-close` writes:

- what each browser does with Esc, as the code now has it, against the request's message;
- each rule broken and the failure seen;
- each reader file you edited beyond the header's.
