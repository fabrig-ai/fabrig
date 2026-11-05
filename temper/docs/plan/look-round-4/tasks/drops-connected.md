+++
id = "drops-connected"
part = "drops-connected"
kind = "feature"
needs = ["more-accounts"]
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-domain/src/pages/switcher.rs",
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "medium"
minutes = 90
+++

# Home drops Connected, and its account

## Goal

Land `home-drops-connected` whole: Connected goes from every Home, with what only it used, then its
render check and account. Every commit carries `Look-request: home-drops-connected`; the last holds
the table.

Acceptance:

- `git diff 0e816f6749 2c6f5a5146 -- temper/docs/design/client/journeys/overlay.css` is applied:
  Connected's block of `temper.css` goes.
- **Removed from the view:** `connected`, `line` and the `sign_in` parameter of `pages/home.rs`'s
  `main`; `Word::HomeConnected`, `HomeLogin`, `HomeRepository`, `HomeModel`, `HomeProject`,
  `HomeProjectRepository`, `HomeProjectNone`; `look::HOME_CONNECTED`, `HOME_LINES`, `HOME_MUTED`,
  whose names become `temper-reserved-210` to `-212`.
- **Rename:** only the switcher's: `Act::RenameHere`, `Switcher::renaming_here`, the field, its arms
  in `switcher.rs`, the `!renaming_here()` conditions of `frame.rs`, and `rename_form` private.
- **What stays reachable,** each by a test or one already there: Forges and Accounts from Settings,
  the lede's link while To begin shows, "Rename PROJECT" from the switcher.
- **The gallery's** `HOLDS` rows name no Connected.
- **The account,** by `temper/docs/plan/look-round-4/notes/account.md`: `project-at-work/01.html`
  and `project-after-skip/01.html` from `2c6f5a5146` against `home-at-work` and `home-after-skip`;
  `project-switcher/01.html` over its switcher as the last round compared it, which keeps its Home's
  column; `~/src/fabrig/look-round-4/home-drops-connected/`. Rows that differ or defer, at the least:
  "Open Forgejo ↗" reached from Settings through Forges, a step more than the request says, the
  Settings page not being designed; the model's line has no other place yet.
- `README.status.md`: Connected goes from its *3.5, Home* entry, and the entry goes if nothing's left.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/account.md`.
- `temper/docs/design/client/README.md`, 3.5, *Settings*, 5.3.
- `git show --format=%B -s 2c6f5a5146`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/{words.rs,limits.rs}`, `src/tests/{fixtures.rs,look.rs,rail.rs}`.
- `temper/crates/temper-client-domain/src/tests/**`: what `RenameHere`'s removal forces.
- `temper/tools/gallery/src/{samples.rs,tests.rs}`.
- `temper/docs/design/client/README.status.md`, `temper/docs/design/client/mockups/STATUS.md`.

Options, each pre-approved:

- `home_rename_stays_in_connected_and_is_only_offered_to_owners` goes with its part; the rail's
  `rail.rs` test covers an owner's rename; add a member's there if none does.
- `configure-address` lands beside you and adds an arm to `switcher.rs`'s block you remove: take your
  removal, its arm goes with the block.

## Steps

1. The removals, then the compiler, then `rg` for each name.
2. The render check, the account.
3. **Check,** `notes/common.md`: `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-terminal -p temper-client-gallery -p
   temper-client-world`, clippy, `python3 -I -B temper/scripts/old_forms.py`, the citations, then the
   last commit, its table in its message, and push.

## Report

The table, the render check's result, the directory and the builder.
