+++
id = "accounts-expiring"
part = "accounts"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-view/src/pages/accounts/rows.rs", "temper/crates/temper-client-view/src/accounts/words.rs", "temper/crates/temper-client-view/src/tests/accounts.rs"]
effort = "high"
minutes = 70
+++

## Goal

On Accounts, an account whose deciding source reads as expiring offers
its renewal under the account's line, as a signed-out one does, so that
Only you's "Sign in again" lands where the act is
(`temper/docs/design/client/README.md`, 5.5, *Each state*'s two
expiring rows and the paragraph under the table).

Acceptance:

- `pages/accounts/rows.rs`'s `notice`, for its source with `expiring`
  true and a state other than `SignInAgain`, draws as
  `notes/contracts.md`, section 4 has it: a sign-in, the paragraph
  "Expiring: sign in again before it ends." and `signin(..)`'s "Continue
  with PLAN" ("Continue with ChatGPT"); a program's login, the same
  paragraph, its command line (`connect_model::command_line`, the
  source's name) and "Check again" (`Operation::CheckProgram { source }`),
  each once. A signed-out source, a limited one and a usable source not
  expiring draw as they do today; the tag "Expiring" stays as `state`
  draws it.
- The new words are `accounts/words.rs`'s, each its own `Word` with a
  stable number, as its neighbours are.
- `src/tests/accounts.rs`'s `each_account_state_draws_its_words_and_state_controls`
  gains its rows, the table's row shape widened to name the source's kind
  and `expiring` if it needs them: a ChatGPT sign-in `Granted`, usable
  and expiring, "Expiring: sign in again before it ends." with the button
  "Continue with ChatGPT"; a program's login `Granted`, usable and
  expiring, the same words, its command and the button "Check again";
  and a sign-in `SignInAgain` and expiring, "Signed out of ChatGPT." and
  no expiring words. Its existing rows keep their expectations.
- The rule bites: draw the expiring paragraph for a usable source not
  expiring once, see a row of the table fail, restore; the report names
  it.
- Delete `temper/docs/design/client/README.status.md`'s **Not yet** entry
  `5.5, *Each state*, expiring`.

## Inputs

- `temper/docs/plan/ui-n3a-only/notes/common.md`;
  `temper/docs/plan/ui-n3a-only/notes/contracts.md`, section 4.
- `temper/docs/design/client/README.md`, 5.5: *Each state* (its table's
  rows "needs sign-in", "needs sign-in, a program's, on a host" and the
  two expiring ones, and the paragraph after the table), *A sign-in*.
- Patterns: `pages/accounts/rows.rs:516` (`notice`), `:460` (`signin`),
  `:599` (a program's `CheckAgain`), `:701` (`command_line`'s use);
  `src/tests/accounts.rs:1617` (the table).

## Owns and may change

- **Owns:** the files in this spec's header, and the status entry above.
- **May change:** a view test outside `tests/accounts.rs` whose expected
  Accounts row has an expiring deciding source, its expectation only;
  the gallery's `accounts-*` prints and the journeys' prints if a state
  of theirs holds an expiring source, by their own tools.
- **Not yours:** `pages/needs.rs`, `pages/only_you.rs`, `words.rs`,
  `look.rs` and `temper.css`, which `only-region` writes.
- **Options, pre-approved:** if `notice`'s program branch already draws
  "Check again" for the source (`needs_check`), draw it once, not twice.
  If the paragraph needs a style, use the one the signed-out paragraph
  (`W::SignedOutPlan`) uses.

## Steps

1. The expiring branch in `notice`; its words.
2. The table's rows; break the rule once.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-view --all-targets -- -D warnings`; `heavy
   cargo nextest run -p temper-client-view --lib`; `heavy cargo check -p
   temper-client-gallery --tests`. Commit and push.

## Report

As the implementer's text asks; name any print regenerated.
