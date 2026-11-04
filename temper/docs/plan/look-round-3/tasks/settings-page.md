+++
id = "settings-page"
part = "settings-page"
kind = "feature"
needs = ["project-addresses"]
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "medium"
minutes = 50
+++

# The Settings page holds Forges, Accounts, adoption and Sign out

## Goal

`Settings { project }` becomes the page the request sends Forges, Accounts and Sign out to, so that
the rail can drop them: its heading, three links with their marks, "Sign out". The banner keeps
them until the `rail` task.

Acceptance:

- The page: level-1 "Settings for PROJECT"; a list of links, "Forges", "Accounts", "Adopt a
  repository" (to `Adopt { project }`), the two first with the dots and words the banner's entries
  carry; then "Sign out", which acts as the banner's does; "Sign out did not reach the engine."
  when `sign_out_lost`.
- `frame.rs` draws each mark through one function, shared with the banner's entries.
- Both shells; the terminal numbers the links and "Sign out".
- Every commit carries the request's trailer (`notes/common.md`).

## Inputs

- `temper/docs/plan/look-round-3/notes/common.md`, `notes/contracts.md` (Numbers).
- `temper/docs/design/client/README.md`, 3.5, *Settings*, 5.3 (the dots' states), 6.6.

## Owns and may change

Owns, as `files` lists them; in `temper.css` a block "The project's settings" at the end, and
nothing else.

May also change:

- `temper/crates/temper-client-view/src/frame.rs`: `forges_entry` and the accounts' entry split into
  `pub(crate)` functions the two pages call. No change to what the banner draws.
- `temper/crates/temper-client-view/src/{words.rs,look.rs}`: new words and tokens as
  `notes/contracts.md` numbers them, beside `Rename` and at the list's end.
- `temper/tools/gallery/src/{samples.rs,tests.rs}`, `temper/tools/gallery/names.txt`: one state,
  `settings`, after `inbox`'s, if the world opens it; its `Holds` row; if the world can't, leave
  them and say so.
- `temper/docs/design/client/README.status.md`: the *Settings* part of its **Not yet** entry.

Option, pre-approved: if a link's dot needs a word the banner has private, make it `pub(crate)`.

## Steps

1. `frame.rs`: the two entries as `forges_link` and `accounts_link`, `pub(crate)`, each opening the
   link, its text, its dot; the banner calls them, its tests unchanged.
2. `pages/home.rs::settings`: the sketch's shape:

   ```rust
   b.open(Role::Main); b.style(look::MAIN); b.style(look::SETTINGS);
   // heading, then a List of Items, each a link, then the Button.
   ```

3. A plain `.temper-settings` block: the list unbulleted, its items spaced as the accounts' page,
   no new colours. Nothing else in the sheet moves.
4. Tests in `tests/home.rs`: the page's order, the marks in both states, "Sign out" giving
   `Act::SignOut`, the lost sign-out's alert, the terminal's print through `skein_ui_text::render`.
5. **Check,** `notes/common.md`, then `heavy cargo nextest run -p temper-client-view -p
   temper-client-gallery -p temper-client-shell-terminal`, clippy on the same, then push.

## Report

As `notes/common.md`, Your report.
