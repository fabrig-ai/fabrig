+++
id = "accounts-tab"
part = "accounts-tab"
kind = "feature"
needs = ["forges-tab"]
files = [
  "temper/crates/temper-client-view/src/pages/accounts.rs",
  "temper/crates/temper-client-view/src/pages/accounts/rows.rs",
  "temper/crates/temper-client-view/src/pages/account_programs.rs",
  "temper/crates/temper-client-view/src/accounts/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/accounts.rs",
  "temper/crates/temper-client-view/src/tests/settings.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 130
+++

# Accounts, in the language

## Goal

`Accounts` under Settings' frame draws as `Forges` now does: a section, a quiet band, folded rows, `.setting` facts and `.btn` controls. Every fact, act, form and state it has today stays, and the old accounts page's rules leave `temper.css`.

Acceptance:

- The tab is `temper/docs/design/client/README.md`, 3.5, *The rows* and *The `Accounts` tab*: "Your accounts · N" with "+ Connect a model" in its head, a `.btn.small`, as today's act and condition; none, a quiet band, the tag "No account yet", "No model of yours yet.", and the primary "Connect a model", as today's condition; a row per account, its summary today's line (5.5, *An account's line*).
- Opened, a row draws today's states, sources, hosts, ranks, models, "Pays for", sign-in, paste, key, program logins, sign-out and remove with its confirmation, in every state, a state's words and acts first.
- "Deployment accounts", an owner's, is a section of the same rows. The playground's is a section of its own, as today. The foot is `settings::sign_out`, as built.
- With no project, the page takes the same column, sections and rows under its level-1 heading, "Accounts", and the tab's lede, with no kicker and no tabs (`settings::open_alone`). The frame's or that lede says what `Word::AccountLead` says, and that line leaves the section.
- `look::ACCOUNT_ROW` and every rule under `.temper-account-row` and `.temper-accounts-page` are gone from `temper.css`, `:is(.temper-accounts-page, .temper-adoption-page)` narrowing to `.temper-adoption-page`, which `flow-paths` removes. `look::SUBTLE_COUNT`, `look::HEADING`, `look::PAGEBAR`, `look::DETAILS` and `look::EMPTY` are no longer on these pages. The welcome's Model step and `ConnectModel` draw as they did.
- The proof, by `temper/docs/plan/ui-settings-tabs/notes/common.md`, *The proof*: the tab at the three widths; `accounts-in-settings`, `accounts-solo`, `accounts-reference`, `accounts-first-run-1`, with a row open in each state.
- `python3 -I -B tools/citations/citations.py --base origin/main` shows none broken. The tests pass and bite.

## Inputs

- `temper/docs/plan/ui-settings-tabs/notes/common.md`; `temper/docs/plan/ui-settings-tabs/notes/contracts.md`, *The language*.
- `temper/docs/design/client/README.md`, 3.5, *Settings* and *The accounts page*; 5.5, *An account's line*, *Each state*.
- `temper/crates/temper-client-view/src/forge/forges.rs` as `forges-tab` left it: the shape to follow.

## Owns and may change

Owns `files`. It builds on `settings::open_alone`, `section_head`, `quiet_band`, `button_to` and `look::SETTINGS_ROW`; a change they need is made in `pages/settings.rs` and named in the report, and How work runs' functions stay alone there. `look::ACCOUNTS_PAGE` stays, on the first section or its head, as `forges-tab` placed `look::FORGES`, with no rule.

Readers that may change, each only to follow a word, heading or token:

- `Your accounts` becomes `Your accounts · N`: `temper/crates/temper-client-view/src/tests/accounts.rs` (lines ~109, ~119, ~736), `temper/crates/temper-client-view/src/tests/playground.rs` (~281), `temper/tests/client/tests/suite/accounts_edges.rs` (~15), `temper/tests/client-browser/tests/browser_all/browser_accounts.rs` (~45). `rg 'Your accounts' temper` finds any since.
- `look::ACCOUNT_ROW`: `tests/accounts.rs`, to `look::SETTINGS_ROW`.
- `your_accounts_heading_row_holds_its_count_and_the_connect_link` follows the head, `look::SETTINGS_SECTION_HEAD`, its count now in the title.
- `forges_and_accounts_are_in_the_frame_with_a_project_and_as_they_were_without`, in `tests/settings.rs`: Accounts' half follows, as Forges' did.
- `Word::AccountLead`, `W::Empty` and any word nothing reads any more: its `words.rs` lines go.

Pre-approved:

- `account_programs.rs` and the key, paste and sign-in parts of `pages/accounts.rs` are shared with `ConnectModel` or the welcome only if `rg` shows so: keep what they draw and reach their look by a rule scoped under `.temper-settings-row`.
- A row's level-3 heading and its holder's line stay, as the first lines of its opened content, unstyled by `look::HEADING`.
- A `Region` from `rows.rs`' `pair` takes `look::SETTINGS_SETTING`; if a term and its value won't sit in two columns by a rule alone, `SETTINGS_FACT`, if `forges-tab` made it, or make it.
- A source's line with its state's token: the state's colour stays, on `--gf-*` values, where the rule is `.temper-settings-row`'s; a `--temper-*` rule that other pages need stays where it is, and this page simply doesn't reach it.

## Steps

1. **`pages/accounts.rs` `main`,** to the contract's shape, as `forge/forges.rs`' is. New words: `W::OwnCount`, "Your accounts · N"; `W::NoOwnTag`, "No account yet". `W::Own` stays, the title before the document is read. `connect`'s `W::ConnectAdd` and `W::Connect` take `settings::button_to`.
2. **`pages/accounts/rows.rs`:** `look::SETTINGS_ROW`; the opened content's order as it is; `look::DETAILS`' slot and `pair` to `.setting` rows.
3. **`temper.css`,** under `/* ==== ui: settings ==== */`: a source's line, its ranks, forms, a confirmation and a program's hosts, under `.temper-settings-row`, on the mock's values, and the old rules gone where they stand.
4. **Tests,** `tests/accounts.rs`:
   - `the_tab_draws_your_accounts_with_its_count_and_act_in_its_head`.
   - `no_account_is_a_quiet_band_and_an_unread_document_claims_nothing`.
   - `while_the_playground_is_read_the_tab_offers_no_connect`, if none says so now.
   - `deployment_accounts_and_the_playgrounds_are_sections_of_the_same_rows`.
   - `alone_it_keeps_its_heading_and_lede_with_no_tabs`.
   - `the_old_chrome_is_gone`, as Forges'.
5. **The gallery:** its `Holds` follow. Keep every `accounts-*` sample.
6. **The proof.**
7. **The status:** take out `temper/docs/design/client/README.status.md`'s 3.5, *The `Accounts` tab* line.
8. `cargo fmt`; old forms; `heavy cargo clippy -p temper-client-view -p temper-client-gallery --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal -p temper-client-shell-web`; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(account) or test(sign_in) or test(connect_model) or test(playground)'`; `heavy cargo check -p temper-client-browser-world --tests`; `heavy cargo check -p temper --tests`; `heavy cargo nextest run -p temper-client-journeys`, reprinting if it's stale (`notes/common.md`, *Journeys*), this task taking up any a task before it left; commit and push.

## Report

As the implementer's text says: which builder; each screenshot, by name, beside the mock's; each difference and why it stays; the rules removed from `temper.css`; each reader you changed.
