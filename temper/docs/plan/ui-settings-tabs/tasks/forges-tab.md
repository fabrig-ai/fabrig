+++
id = "forges-tab"
part = "forges-tab"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/forge/forges.rs",
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/forges.rs",
  "temper/crates/temper-client-view/src/tests/settings.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/forge.status.md",
]
effort = "high"
minutes = 120
+++

# Forges, in the language

## Goal

`Forges` under Settings' frame draws in the greenfield mock's language: a section, a quiet band, folded rows with a hairline and a chevron, `.setting` facts and `.btn` controls. Every fact, act and state it has today stays. The helpers and the row's token that `accounts-tab` builds on exist.

Acceptance:

- The tab is `temper/docs/design/client/README.md`, 3.5, *The rows* and *The `Forges` tab*, and `temper/docs/design/client/forge.md`, 5.4's *The Forges page*: "Your forges · N" with "+ Add a forge" in its head, a `.btn.small`, not while the playground runs; none, a quiet band; a row per forge; the playground's section and row.
- With no project, the page takes the same column, sections and rows under its level-1 heading, "Forges", and the tab's lede, with no kicker and no tabs (`settings::open_alone`).
- Opened, a row draws today's facts and acts in every state: setting up with its steps, failed, stopped, connecting, replacing, removing with its confirmation, GitHub's App and installations, a first repository making or failed; "Open Forgejo ↗", the owned forge's password's words, `Adopt` as today.
- `ForgesDescription`'s line leaves the page, the frame's lede saying it. A row's second line starts "Forge N · KIND", with a capital (`Word::ForgeSummary`).
- `look::FORGE_ROW` and its rules are gone. No rule under the `--temper-*` tokens reaches this tab, and the welcome and `ConnectForge` draw as they did.
- How work runs and Repositories draw as they did: `settings` and `repositories*` in the gallery are unchanged but for the primary's rule the contract moves.
- The proof, by `temper/docs/plan/ui-settings-tabs/notes/common.md`, *The proof*: the tab at the three widths; `forges-in-settings`, `forges-first-run-7`, `forges-github-install`, with a row open in each state above.
- `python3 -I -B tools/citations/citations.py --base origin/main` shows none broken. The tests pass and bite.

## Inputs

- `temper/docs/plan/ui-settings-tabs/notes/common.md`; `temper/docs/plan/ui-settings-tabs/notes/contracts.md`, *The language*.
- `temper/docs/design/client/README.md`, 3.5, *Settings*. `temper/docs/design/client/forge.md`, 5.4, its first bullets. `temper/docs/design/client/browser.md`, 2.5, *Settings' look*.
- `temper/docs/design/client/greenfield/mock.css`: `.lines`, `.setting`, `.section-head`, `.band`, `.quiet-band`, `.btn`, `.text`, `.switcher > summary::after`.

## Owns and may change

Owns `files`. In `pages/settings.rs`, only the contract's helpers, `repository_section`'s use of them, and `button_link`'s primary: `how-work-runs` is rewriting the rest. In `temper.css`, this plan's rules go under `/* ==== ui: settings ==== */`; `:is(.temper-forge-row, .temper-account-row)` narrows to `.temper-account-row`, which `accounts-tab` removes, and `.temper-forge-row`'s other rules go. In `tests/settings.rs`, `forges_and_accounts_are_in_the_frame_with_a_project_and_as_they_were_without` splits: Forges' half follows this task, and Accounts' half stays as it was for `accounts-tab`.

Readers that may change, each only to follow a word or a token:

- `Your forges N` becomes `Your forges · N`: `temper/crates/temper-client-view/src/tests/forges.rs`, `temper/tools/gallery/src/tests.rs` (its two `Holds`).
- `forge N · KIND` becomes `Forge N · KIND`, `Word::ForgeSummary`'s words alone: `rg 'forge [0-9] · (Forgejo|GitHub)' temper/crates/temper-client-view temper/tests temper/tools` for the tests and worlds that say it. "forge N" elsewhere (ctl's, `temper-client-tools`' goldens) is another word and stays.
- `look::FORGE_ROW`: `tests/forges.rs`, line ~561.
- `ForgesDescription`: its `words.rs` lines go if nothing else reads them.

Pre-approved:

- If `forge/connect.rs` needs a line to take a look a scoped rule can't reach, make it conditional on the row's place and name it in the report; the welcome's and `ConnectForge`'s drawing stay as built.
- If a fact's term and value don't sit in `.setting`'s two columns by a rule alone, give the slot one more token, `SETTINGS_FACT`, under the marker.
- A `flow::card` in a row (GitHub's install card) loses its card by a scoped rule; its words and act stay.
- Golden or gallery fallout that follows from the words above is regenerated, not hand-edited.

## Steps

1. **The contract's helpers,** in `pages/settings.rs`: `open_alone`, `section_head`, `quiet_band`, `button_to`; `repository_section` draws through them, its tree unchanged but for what the primary's rule needs.
2. **`look::SETTINGS_ROW`,** and the rules: the row, its summary, the chevron, its opened content, its controls and fields; at 760 px and under, as the mock's `.setting` does.
3. **`forge/forges.rs` `main`,** to the contract's shape:

   ```rust
   b.open(Role::Main);
   b.style(look::MAIN);
   b.style(look::SETTINGS);
   match project {
       Some(project) => settings::open(b, root, w, bindings, project, Tab::Forges),
       None => settings::open_alone(b, &w.say(Word::ForgesTitle, &[]), &w.say(Word::SettingsForgesLede, &[])),
   }
   // the section, its head, band or SETTINGS_LINES, then the playground's section
   b.close();
   b.close();
   ```

   `look::FORGES` stays, on the first section. `personal`'s facts (`account`, `access`, `owned`, the repository's) take `SETTINGS_SETTING`. New words: `Word::NoForgeTag`, "No forge yet". `Word::YourForges` says "Your forges · N".
4. **Tests,** `temper/crates/temper-client-view/src/tests/forges.rs`, `tests/settings.rs`:
   - `the_forges_page_says_its_words_and_folds_each_row` follows: its words in order, in the tab and alone.
   - `the_tab_draws_its_section_head_and_its_act_in_it`: with rows; while the playground runs, no act.
   - `no_forge_is_a_quiet_band_and_an_unread_watch_claims_nothing`.
   - `each_forge_is_a_folded_row_with_its_facts_as_settings`.
   - `alone_it_keeps_its_heading_and_lede_with_no_tabs`.
   - `the_old_chrome_is_gone`: no `CARD`, `PAGEBAR`, `HEADING` or `SUBTLE_COUNT` in the page, in every sample state the view tests reach.
5. **The gallery:** `Holds` follow. Keep every `forges-*` sample.
6. **The proof.**
7. **The status:** take out `temper/docs/design/client/README.status.md`'s 3.5, *The `Forges` tab*, *The rows* line, and `temper/docs/design/client/forge.status.md`'s 5.4 line on `Forges`' tab and rows.
8. `cargo fmt`; old forms; `heavy cargo clippy -p temper-client-view -p temper-client-gallery --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal -p temper-client-shell-web`; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(forges) or test(connect_forge) or test(forge_replace) or test(github_connect)'`; `heavy cargo check -p temper --tests`. Commit and push.

## Report

As the implementer's text says: which builder; each screenshot, by name, beside the mock's; each difference and why it stays; the rules removed from `temper.css`; each reader you changed.

## Rebase (2026-11-05, step merge, foreman ui-settings-tabs/forges-tab/foreman-1)

resolve the duplicate citation correction using the integration version
The merge conflicts only in `temper/docs/design/client/README.md`, section 3.5, *The rows*.
Task commit `2efb9c7a03` and integration commit `931d44c2af` independently correct the same citation.
Resolve that line to integration’s wording:
`act, form and state of 4.8, 5.5 and `forge.md`, 5.4 stays, and only its`
This preserves local sections 4.8 and 5.5 and the external forge section 5.4.
Keep the Forges implementation and status changes; no implementation repair is indicated by this refusal.
Run `python3 -I -B tools/citations/citations.py --base origin/main` after resolving.
