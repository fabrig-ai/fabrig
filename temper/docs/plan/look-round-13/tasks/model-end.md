+++
id = "model-end"
part = "model"
kind = "feature"
needs = ["model-cards"]
files = ["temper/crates/temper-client-view/src/pages/accounts.rs", "temper/crates/temper-client-view/src/pages/accounts/rows.rs", "temper/crates/temper-client-domain/src/pages/accounts.rs", "temper/crates/temper-client-domain/src/pages/connect_model.rs", "temper/crates/temper-client-view/src/pages/connect_model.rs", "temper/crates/temper-client-view/src/tests/accounts.rs", "temper/crates/temper-client-domain/src/tests/pages/accounts.rs", "temper/tests/client/tests/suite/connect_model*.rs", "temper/tests/client/tests/suite/accounts_edges.rs", "temper/tools/gallery/**"]
effort = "high"
minutes = 75
+++

## Goal

The Ready step goes: an account made lands on `Accounts`, with its line, "Added your ChatGPT plan. temper’s agents now run on it." and a quiet "Start a chat" (pages 04, 08). Accounts with nothing connected is one way forward (page 01). An opened row says only what its summary doesn't (page 09).

Acceptance:
- on `ConnectModel`, a key kept, a program declared and checked, or a sign-in ended with an account lands on `Accounts`, the way's address replaced, the line under the section's head in `CONNECT_CONFIRM`; it goes when the person leaves the page or acts on it; a reload does not bring it back. Declined, refused and failed endings stay on the way's page with today's words, in its panel. The welcome keeps its own Ready;
- page 01: under "Your accounts · 0", `CONNECT_EMPTY`: "No model of yours yet." and the primary "Connect a model"; the section head's "+ Connect a model" and today's quiet band go in that state;
- an opened row draws no repeat of its name, models, "Ready" or "Pays for"; a plan's "Manage usage" stays in its row; every act stays;
- `ready_body`, `ready_address`, `ready_primary` and the words only they said are gone, unless the welcome still calls them; the `connect-model-*-ready` gallery names become the `Accounts` page's line, `accounts-added`;
- focused tests of `temper-client-view`, `temper-client-domain`, `temper-client-shell-terminal`, the client world's accounts and connect-model stories and `the_committed_journeys_are_their_print` pass.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, `Accounts`, and `ConnectModel`, *Ready's place* and *Its end*; 5.5, the made words.
- The request's `README.md` and pages 01, 04, 08, 09.
- `temper/docs/plan/look-round-13/notes/common.md`; `notes/contracts.md`, *Look tokens*.

## Owns and may change

- Owns the `files` above.
- May change: `temper-client-domain/src/pages.rs`, the address change after the end, the one call; tests walking Ready in `temper/tests/client/tests/suite/{sign_in,sign_in_paste,welcome_*}.rs` and `temper/crates/temper/tests/support/person.rs`, their words alone. `temper/tests/client-browser/tests/browser_all/browser_accounts.rs`: edit and compile only.
- Option, the line's state: `Accounts` form's new `added: Option<Added>`, `Added { provider, kind }`, set when `ConnectModel` moves, cleared on any act or leaving. If `Accounts`' form isn't open at the move, hold it in `Pages` and hand it over, and say so.
- A sign-in signed from an accounts page's row stays there as today and takes the same line.

## Steps

1. Domain with tests in `temper-client-domain/src/tests/pages/accounts.rs`: moved with the line; the line gone after an act; a reload's fresh page has none; a declined ending does not move.
2. View; `rows.rs`'s opened row; page 01.
3. Tests; goldens and gallery names kept sorted; journeys reprint.
4. Compare at 1440, 1000, 420 against pages 01, 04, 08, 09 by a throwaway driver.
5. `cargo fmt`, old forms, clippy, focused tests, commit with the trailer, push.

## Report

As the implementer's text has it, with the differences from the request's pages named.
