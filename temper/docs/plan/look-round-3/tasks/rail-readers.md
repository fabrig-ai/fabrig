+++
id = "rail-readers"
part = "rail-readers"
kind = "migration"
needs = ["rail"]
files = [
  "temper/tests/client/**",
  "temper/tests/client-browser/**",
  "temper/tests/one-process/**",
  "temper/tests/real-loop/**",
  "temper/crates/temper/tests/**",
  "temper/crates/temper-client-shell-web/src/host_tests.rs",
]
effort = "medium"
minutes = 110
+++

# The test worlds walk the rail

## Goal

The worlds that walked the banner walk the rail: a person reaches Forges, Accounts, the adoption
form and Sign out through Settings, the inbox through "Needs you". Each test keeps its subject.

Acceptance:

- Every test the survey below names passes where its tier runs in the gate, and compiles elsewhere.
- No test asserts less than it did. A control a test presses is found through the page that now
  holds it, in the helpers below where one exists.
- `docs/help` says what the client now does.
- Every commit carries the request's trailer (`notes/common.md`).

## Inputs

- `temper/docs/plan/look-round-3/notes/common.md`.
- `temper/docs/design/client/README.md`, 5.3, *The rail*, and 3.5, *Settings*.

## Owns and may change

Owns, as `files` lists them, for controls and their walks alone.

May also change, departing from the implementer's text:

- `temper/docs/help/project.md`, 33 and 36, `temper/docs/help/playground.md`, 37 and 38,
  `temper/docs/help/forge.md`, 65: "banner" gives way to the rail or Settings. These are compiled
  into tests: run `temper-docs`'s.
- `temper/docs/design/client/journeys.md`, 206: the banner's "Menu" holds on a page that is no
  project's.

The survey, to start from:

- **Helpers:** `tests/client/src/fixtures.rs`, 290 to 296 (`project`, `adopt`);
  `tests/one-process/src/stories.rs`, 48 and 49; `crates/temper/tests/support/person.rs`, 167, 693,
  703, 776; `tests/one-process/src/world.rs`, 509, 525.
- **Inbox:** `tests/client/tests/suite/{question.rs:279,inbox.rs:38,chat.rs:323}`.
- **Accounts:** `suite/accounts_edges.rs` 84, 89, `suite/welcome_m3.rs` 67,
  `tests/client-browser/tests/browser_all/browser_accounts.rs` 43,
  `temper-client-shell-web/src/host_tests.rs` 235, 239, `tests/real-loop/tests/suite/sign_in.rs` 22.
- **Forges:** `suite/forges_edges.rs` 837 to 841 (the dots), 1005, 1014, 1105,
  `suite/first_run_hooks.rs` 133, `crates/temper/tests/process.rs` 86 to 91.
- **Adopt:** `suite/adoption.rs` 93, 134, `suite/welcome_m3.rs` 110.
- **Sign out:** `browser_all/browser_public.rs` 136, `browser_attacks.rs` 20, `browser_sign_in.rs` 38.
- **The page lists:** `suite/forges_edges.rs` 1062, `tests/one-process/src/m15/pages.rs` 15 gain
  `Home` and `Settings` if they mean every page.
- **Survive,** so check, don't edit: `Live` and `Offline` by status, `Chats` by link.

Options, each pre-approved:

- A test with no project keeps the banner: leave it.
- A browser test that presses a project's row by its exact name reads it once `switcher-rows` lands;
  that task updates it.
- `type_on`, which takes the last match, may now meet the rail's and a page's "Chats": name the
  control in the helper.

## Steps

1. `heavy cargo check -p <each package> --tests`, then each package's tests, from the helpers out.
2. A test that fails for a reason beyond a control's place is a question, with the test's name.
3. **Check,** `notes/common.md`: `heavy cargo nextest run` on each package you changed. Clippy on the
   same.

## Report

As `notes/common.md`, Your report: each helper changed, and any test you could not run here.
