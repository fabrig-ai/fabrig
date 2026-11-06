+++
id = "sheet-story"
part = "sheet"
kind = "feature"
needs = ["sheet-rows", "model-end", "forge-rows"]
files = ["temper/tests/client-browser/tests/browser_all/browser_connect_flows.rs", "temper/tests/client-browser/tests/browser_all/main.rs", "temper/tests/client-browser/tests/browser_all/browser_accounts.rs", "temper/tests/client-browser/tests/browser_all/browser_pages.rs", "temper/tests/client-browser/src/lib.rs"]
effort = "medium"
minutes = 30
+++

## Goal

A browser story for what only a browser shows, written and compiled, not run: the sheet's list read on by scrolling, the sheet at a phone's width a sheet from the bottom, and a connect flow landing on its row. The earlier stories these flows changed compile and read right.

Acceptance:
- `browser_connect_flows.rs` has a story: on fake backends with a forge of many repositories, open "Add repositories", scroll the list, see the count grow; search; pick; adopt; land on Repositories. A second at 420 wide: the sheet is at the bottom, its button shows. A third: connect a model, land on Accounts with its line;
- `heavy cargo check -p temper-client-browser-world --tests` passes. **It is not run**, and neither is anything else of the browser tier (`notes/common.md`, *Runs*): the report says so;
- each browser story that walked a removed part (a "Choose" card, "All ways", Ready, the Adopt page) reads as the page now does: found by reading, not by running.

## Inputs

- `temper/docs/design/client/README.md`, 4.4; `temper/docs/design/client/forge.md`, 5.1.
- `temper/docs/plan/look-round-13/notes/common.md`.

## Owns and may change

- Owns the `files` above.
- May change: other files under `temper/tests/client-browser/`, for what the compile or a removed part needs.
- Option: seeding a forge of many repositories: reuse how the fake backends seed one. If none does, a story of two pages' worth on the fake forge's own git store, through its existing test support, or drop to a page's worth and say so.

## Steps

1. Read `temper/tests/client-browser/tests/browser_all/browser_accounts.rs` and `browser_pages.rs` against the pages as they now stand; fix.
2. Write the stories.
3. `cargo fmt`, clippy on the package, the compile check, commit with the trailer, push.

## Report

As the implementer's text has it, saying what was compiled and not run.
