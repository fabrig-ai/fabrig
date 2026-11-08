+++
id = "google-endings"
part = "endings"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/clients/google/pages.rs",
  "temper/crates/temper-service/src/clients/google/tests.rs",
  "temper/crates/temper-service/src/tests/sign_in.rs",
  "temper/docs/design/protocol/sign-in.status.md",
]
effort = "medium"
minutes = 25
+++

# The Google endings in the client's typographic apostrophe

## Goal

The request's task 6 (`temper/docs/plan/look-round-22/inputs/request.md`). The server's Google sign-in endings (`temper/crates/temper-service/src/clients/google/pages.rs`) say "didn't", "can't", "isn't", "hasn't", "couldn't" and "this account's address" with the straight apostrophe, which `page`'s escape turns into `&#39;`; the client's words use ’ (U+2019). Every ending's words take ’.

Acceptance, by `temper/docs/design/protocol/sign-in.md`, 4.3 (the table's words, and "Their words take the client's typographic apostrophe"):

1. **Every apostrophe of the endings' words is ’,** in `not_member` ("ADDRESS can’t sign in here"), `another` ("This isn’t the Google account…"), `unverified` ("Google hasn’t verified ADDRESS", and its fallback "this account’s address"), `cancelled`, `expired` and `unreachable` ("The sign-in didn’t finish"; "temper couldn’t reach Google…") and `busy` ("temper couldn’t sign you in just now"). A byte literal holds no ’: write each as a `&str` literal's bytes (`" can’t sign in here".as_bytes()`), or `\xe2\x80\x99` inside the byte literal; say which. `escape`'s arm for `b'\''` stays: it escapes an apostrophe of an address, an error or a `then`, which are not the endings' words. No other word, markup or byte of the pages changes.
2. **The row:** the existing table test `endings_draw_the_column_and_link_the_bundles_sheet` (`clients/google/tests.rs`), whose eight rows cover every ending, gains in its loop body `assert!(!text.contains("&#39;") && !text.contains('\''), "typographic apostrophes: {text}");` (its addresses and `then` hold none). No new test function.
3. **The tests reading the words:** `checked_nonmembers_and_unverified_addresses_mint_nothing` (its four expected headings, about lines 191 to 195), `mint_refused_at_the_sign_in_capacity_answers_busy` (about line 229) and `temper/crates/temper-service/src/tests/sign_in.rs`, about line 450 ("The sign-in didn&#39;t finish"), expect ’.
4. **Status:** delete the **Not yet** entry naming `google-endings` in `temper/docs/design/protocol/sign-in.status.md`. No other design line.
5. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-service`; `cargo clippy -p temper-service --all-targets -- -D warnings`.

One commit or more, each with the trailer `Look-request: outside-pages`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-22/notes/common.md`.
- `temper/docs/plan/look-round-22/inputs/request.md`, task 6.
- `temper/docs/design/protocol/sign-in.md`, 4.3, the opening paragraph and the endings' table.

## Owns and may change

Owns the endings' words in `pages.rs`, the table's new assertion and the expected words of the tests in acceptance 3, and the status entry. Not `page`'s markup, `escape`, the setup pages (`temper/crates/temper-service/src/setup/`), the client's words or any other file.

Options, each decided:

- **A test elsewhere expects a straight apostrophe of an ending** (found by `rg "&#39;" temper/crates/temper-service`, and the endings' words across `temper/`): change its expected words to ’, keeping what it checks, and name it in the report.
- **A bound in a test counts an ending's bytes:** each ’ is three bytes where `&#39;` was five; update the count and say so.

## Steps

1. Read the request's task 6 and `sign-in.md`, 4.3; change the words (acceptance 1).
2. The table's assertion (acceptance 2); bite it: restore one straight apostrophe, see the row fail on `typographic apostrophes`, and restore.
3. The tests' expected words (acceptance 3); the status entry (acceptance 4); the runs of acceptance 5; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| Every Google ending's words in the typographic apostrophe, ’ | landed (`clients/google/pages.rs`, seven endings) |
| The endings' table holds no straight or escaped apostrophe | landed (`endings_draw_the_column_and_link_the_bundles_sheet`) |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): focused tests establish no rendered equality |

## Report

The implementer's report: the literal form chosen, the tests changed, the bite and its failure, the runs' counts, and the account as committed.
