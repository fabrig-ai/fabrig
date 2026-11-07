+++
id = "names-by-email"
part = "look"
kind = "feature"
needs = ["facts-line"]
files = ["temper/crates/temper-client-view/src/accounts/words.rs", "temper/crates/temper-client-view/src/pages/accounts.rs", "temper/crates/temper-client-view/src/pages/accounts/rows.rs", "temper/crates/temper-client-view/src/pages/connect_model.rs", "temper/crates/temper-client-view/src/choice/picker.rs", "temper/crates/temper-client-view/src/tests/accounts.rs", "temper/crates/temper-client-view/src/tests/welcome.rs", "temper/crates/temper-client-view/src/tests/model_check.rs", "temper/crates/temper-client-view/src/tests/choice.rs", "temper/crates/temper-client-view/src/tests/connect_model.rs", "temper/crates/temper-client-tools/src/jig/render/document/accounts.rs", "temper/crates/temper-client-tools/src/tests/jig/accounts.rs", "temper/tools/gallery/src/worlds.rs", "temper/tools/gallery/src/worlds/picker.rs"]
effort = "high"
minutes = 110
+++

# temper names an account by its email

## Goal

The look request `ready-account-email`. Ready says "ChatGPT plan" alone today, and its gallery holds "ChatGPT plan · sam@example.com" through a display no real sign-in gives. Everything that says who an account is now reads one place, so Ready, `model-check-pane`'s who it is, the picker's heading for two accounts of a provider, the accounts page and an ending's words all read the email.

Acceptance:

- Section 7 of the contracts note, exactly, and no site of those below reads `display` or a source's name for who an account is but through `who`.
- `temper-client-view`, in the tables beside these lines' tests, a row each:
  - `tests/welcome.rs`: Ready, "ChatGPT plan · alice@example.com" from an email fact with an empty display; with no fact, the display as today; a program's, "Claude subscription · alice@example.com, signed in on this machine", and with no fact its source's name, as today.
  - `tests/model_check.rs`: the who-it-is detail reads the email.
  - `tests/choice.rs`: two accounts of a provider that would read alike say their emails; one with no email says its display.
  - `tests/accounts.rs`: the heading, "ChatGPT · alice@example.com · OpenAI · held by Alice", and `Made` and `Renewed`'s words name the email. An account with both an email and a display names the email.
  - `tests/connect_model.rs`: the Model step's status.
  - The terminal reads the same words: `tests/welcome.rs`'s `terminal_print` row holds Ready's email.
- `temper-client-tools`: `account list --json` gives each account `"email"`, a string or `null`, beside `"display"`, for the party's and the deployment's rows. A row of its table.
- The gallery's first-run world moves from a display to the fact: `FirstRun { step: 5..=9 }` has an empty display, its source's name as the other steps' are, and an email fact of `sam@example.com`. `tests.rs`'s Holds stay exactly as they are, and pass: they are the proof that this reads the fact. Its prints don't change; a print that does is a red.
- Each commit carries `Look-request: ready-account-email`, directly above the other trailers (`temper/docs/design/draft/look-requests.md`, 2 and 3). The landing's account comes with `email-close`.
- The focused and fuzzy tests of `temper-client-view`, `temper-client-domain`, `temper-client-tools`, `temper-client-shell-terminal` and `temper-client-gallery` pass.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Ready*'s model line, and 5.5, *An account*.
- `temper/docs/design/client/greenfield/data.js`, near `who:`, its lines for a plan and a subscription.
- `temper/docs/plan/account-email/notes/contracts.md`, 6 and 7.
- `temper/docs/plan/account-email/notes/common.md`.

## Owns and may change

Owns the header's files. The sites, on the trunk:

- `pages/accounts.rs`: `account_words`' program and signed-in arms; `heading`, which takes the email beside `display`; `account_heading`; `named_end`.
- `pages/accounts/rows.rs`: `own` and `deployment` pass the row's email through `draw`.
- `pages/connect_model.rs`, near line 942: the Model step's name.
- `choice/picker.rs`, near line 348: the clause for two headings alike.
- `pages/model_check.rs` reads through `account_words`, and needs no change: its `account.display` near line 181 stands in for a provider's name, and stays. `tests/model_check.rs` may change.

Readers: test helpers that build a `c::AccountLine` may gain an email argument where a table needs it.

Options, each pre-approved:

- The "Work" display in `temper/tools/gallery/src/worlds/picker.rs` stays: it has no email, and shows the fallback. If a second row with an email is the cheapest way to hold the picker's rule in the gallery, add it as a fact on that sample, and regenerate its print.
- A help page under `temper/docs/help/` or a `docs/help/reference` page that says what `account list --json` gives is yours, within its bound.
- `temper.css` needs no rule: an email takes the words' place.

## Steps

```rust
// accounts/words.rs
pub(crate) fn who<'a>(email: Option<&'a [u8]>, named: &'a [u8]) -> &'a [u8] {
    match email {
        Some(email) => email,
        None => named,
    }
}
// account_words
c::AccountKind::Program => (named(w, W::Subscription, plan_name), who(row.email(), source.name.as_deref().unwrap_or(row.display.bytes()))),
```

Each new test breaks its rule once by hand. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents, one command at a time. Commit and push.

## Report

As the implementer's text says. No render check: it follows landing.
