+++
id = "terminal-redraw"
part = "terminal-redraw"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/terminal.rs",
  "temper/crates/temper-client-domain/src/tests/pages/terminal.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 50
+++

# ">_" draws the terminal in its press's step

## Goal

Pressing ">_" enters full screen, but the page keeps the ordinary chat until another input, since `terminal::Act::Enter` sets the look without moving the pages' shown count, and the view builds only when that count moves (`temper/docs/design/client/README.md`, 3 and 5.1, `build`). "Exit"'s press, `Theme` and `Font` as acts, and a list closed by Esc or by leaving full screen with no "/" in the box have the same cause. Make every terminal act, and a list's closing on any event, move the count, with a failing focused test first.

Acceptance:

1. A row-level check, written and run red before the fix: in `the_look_moves_by_its_acts`, each look act moves `crate::shown` of the domain, and a `ScreenLeft` that closes a list moves it. It fails on the trunk for `Enter`, `Leave`, `Theme`, `Font` and the list rows, and passes after.
2. `terminal::prepare` (or the functions it calls) moves `pages.shown` for every `Act::Terminal(_)` it resolves, and wherever it sets `pages.listing` from `Some` to `None`; `restore_look` moves it too. Nothing else changes: the requests each act pushes, the look, the notes and the box's words are as they are.
3. The `look-round-17` **Not yet** entry under 3.5 in `temper/docs/design/client/README.status.md` is deleted.
4. The last commit carries `Look-request: chat-terminal` and the account (below).
5. The focused tests of `temper-client-domain`, `temper-client-view`, `temper-client-shell-web`, `temper-client-shell-terminal`, `temper-client-journeys` and `temper-client-world` pass: the count's meaning changed (it moves more often), so its readers' tests run.

## Inputs

- `temper/docs/plan/look-round-17/notes/common.md`.
- `temper/docs/plan/look-round-17/inputs/request.md`, 1.
- `temper/docs/design/client/README.md`, 3 (its first paragraph: `shown`), 3.5, *A chat's terminal*, *Entered*, *Left* and *Keys*, and 5.1, `build`. This cut wrote *Entered*'s line on the shown count.
- The Escape landing's account, `git show 2a62e07db5`: its rows *State: page 01 >_ requests real full screen* and *State: page that is not a secure context* say the press draws the terminal, refused full screen or not.
- The code: `temper/crates/temper-client-domain/src/pages/terminal.rs` (`Look::act`, `terminal_act`, `prepare`, `restore_look`, `clear_terminal_words`, which moves the count today); `pages.rs`, `step` and `shown`; `route.rs`, `Event::ScreenLeft` and `Event::LookKept`; `temper/crates/temper-client-view/src/root.rs`, `build`, whose `stale(shown)` skips the build.

## Owns and may change

Owns the header's files. May also change `temper/crates/temper-client-domain/src/pages.rs` where `prepare`'s arms need a line (call sites only), and nothing in the view: the view builds on the count already.

Departs from the implementer's text: the task deletes its own **Not yet** entry in `README.status.md`, a design status file.

Options, each decided:

- **Where the count moves:** default, one place in `prepare`: the `P::Terminal(_)` arm moves it before `terminal_act`, and each arm that clears `listing` moves it when the list was open. If a helper reads better (`fn redraw(&mut self)` beside `clear_terminal_words`), use it and say so.
- **`clear_terminal_words` already moves it:** a terminal act that also clears the box moves the count twice; that is harmless (the view compares, never counts), and needs no guard.
- **A test elsewhere breaks** because it counted builds or patches after a terminal act: the new count is correct; repair that test's expectation, keeping what it checks otherwise, and name it in the report.

## Steps

1. In `temper/crates/temper-client-domain/src/tests/pages/terminal.rs`, `the_look_moves_by_its_acts`:
   - in the first loop, read `crate::shown(&domain)` before the `crate::step` of `Act::Terminal(act)` and assert it grew after, for every row;
   - in the second loop (`ScreenLeft` after words or a command), assert the count grew across the `ScreenLeft` step, for every row; those that leave no "/" in the box (the words "hello", and each command, which clears the box and opens its list or confirmation) fail today;
   - in the third loop, assert it grew across `LookKept` and across `ScreenLeft`.
   Run it (`heavy cargo nextest run -p temper-client-domain --lib -E 'test(=tests::pages::terminal::the_look_moves_by_its_acts)'`, the path as nextest names it) and see it fail on the rows the Goal names, for the count. That is the test's bite; say which rows failed in the report.
2. Make the count move as acceptance 2 says; run the test again, green.
3. Delete the **Not yet** entry; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain --all-targets -- -D warnings`; the focused tests of the packages acceptance 5 names, one `heavy` command at a time.
5. Commit with the account's table in the message and `Look-request: chat-terminal` above the other trailers; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| Page 01: ">_" draws full screen and the terminal at once | landed: where the count moves, and the test row |
| "Exit" as a press, and a list's Esc or full screen's leaving with no "/" typed | landed, the same cause and line |
| The Escape landing's account (`2a62e07db5`), its rows on page 01 and on a page that is not a secure context, says the press draws the terminal | differed: the press set the look and asked for full screen, but the page drew at the next change; it draws in the press's step now, refused full screen or not |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4); compiling and focused tests establish no rendered equality |

## Report

The implementer's report: the rows that failed before the fix and how, where the count moves, any test whose expectation changed, and the account as committed.
