+++
id = "row-tab"
part = "tab-root"
kind = "feature"
needs = ["sign-in-holds"]
files = ["temper/crates/temper-client-domain/src/pages/accounts/acts.rs", "temper/crates/temper-client-domain/src/sign_in_tab.rs", "temper/tests/client/tests/suite/accounts_page.rs"]
effort = "medium"
minutes = 25
+++

## Goal

The accounts page's row, which sends the same sign-in and holds its page today, opens and points the same tab, so that a person pressing it gets what a person on `ConnectModel` gets. Its page draws as it does: its link "Open the sign-in page" and its paste.

Acceptance, failing tests first, in `temper-client-world`:

- The row's press asks `Tab::Open { provider }` before any answer; the answer asks one `Tab::Point`; a refused answer asks `Tab::Close`. The page's address stays `/accounts`, and no `Leave` comes out.
- `Event::TabRefused` then the answer asks no `Point`, and the row still draws its link.
- `accounts_row_sign_in_offers_the_paste_and_observes_success` and the row's other sign-in tests pass unchanged.

## Inputs

- `temper/docs/plan/sign-in-new-tab/notes/contracts.md`, *The root*.
- `temper/docs/plan/sign-in-new-tab/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, `ConnectModel`, *The sign-in's tab*, its last sentence.

## Owns and may change

- Owns the files in the header. `Operation::SignIn` in `acts.rs` is where the row asks.
- The row's page has no way to be refused in its words: `ConnectModel`'s fact isn't its. `sign_in_tab::refused` clears `Domain::tab` for whichever page holds it, and sets a refusal only on `ConnectModel`.
- `sign_in_tab::told`'s "the page still holds that key" reads either form. If that wants a reader on `Pages`, add one `pub(crate)` method, keeping the `pages.rs` hunk small.
- No change to `temper-client-view`.

## Steps

As `sign-in-holds` did for `Act::SignIn`: the `Output` for the tab right before the send, the route setting `Domain::tab`.

Finish: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-world`. Commit and push.

## Report

As the implementer's text asks, with the rule you broke by hand and what failed.
