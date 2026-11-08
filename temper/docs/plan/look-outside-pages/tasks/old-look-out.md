+++
id = "old-look-out"
part = "old-look-out"
kind = "migration"
needs = ["outside-frame", "approval-column", "sign-in-column", "public-top", "boot-mark", "server-pages"]
files = ["temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/tests/look.rs", "temper/crates/temper-client-view/src/tests/terminal.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/crates/temper-client-shell-web/files/brand.svg", "temper/crates/temper-client-shell-web/tests/suite/main.rs", "temper/tools/gallery/src/tests.rs"]
effort = "high"
minutes = 90
+++

# look-outside-pages: the old look leaves for good, and two tests keep it out

## Goal

Once nothing draws them, the looks only the old frame and card used leave `look.rs`,
their rules leave `temper.css`, `files/brand.svg` goes, and the stylesheets' old accent,
the old mark's `#b8664e`, becomes the signal. Two focused tests keep the old look from
coming back.

Acceptance:

- `look.rs`: `BANNER` (0), `CORNER` (21), `UNAVAILABLE` (68), `MENU` (85), `SIGN_IN` (123),
  `APPROVE_ACTS` (606) and `APPROVE_PRIMARY` (607) are gone; each slot's name is
  `temper-reserved-N` (contracts, 1). `CARD`, `MAIN`, `BRAND`, `APPROVE`, `APPROVE_CODE`,
  `APPROVE_ASKED` stay.
- `temper.css`: no rule selects `temper-banner`, `temper-menu`, `temper-corner`,
  `temper-sign-in`, `temper-unavailable`, `temper-approve-acts` or
  `temper-approve-primary`; a selector list that names one of them among others loses that
  selector alone. The old approval block under `/* ==== tui: approval ==== */` goes
  with its marker, the outside marker's rules standing for it. Variables only those rules
  read (`--temper-banner-line`, `--temper-menu-ink`, `--temper-menu-shadow`,
  `--temper-brand-line`, and any other you find) go, by day and night.
- No rule reads `files/brand.svg`: the base `.temper-brand::before` draws `content: '✦';
  color: var(--gf-signal); font-weight: 400;` as the rail's, the welcome's and the top
  bar's brands already override it to; the base `.temper-brand` rule and its `::after`
  otherwise stay (the top bar's and the rail's brands build on them). `files/brand.svg` is
  deleted.
- `--temper-accent` and the light terminal's `--accent` are `#c2410c`.
- Two focused tests:
  - `temper-client-view`, `tests/look.rs`,
    `every_class_the_stylesheet_selects_is_a_look_name`: every `.temper-…` class selector
    in `temper.css` (comments stripped) is a name in `look::NAMES.styles` and not a
    `temper-reserved-N`.
  - `temper-client-shell-web`, `tests/suite/main.rs`,
    `no_source_of_the_bundle_or_the_servers_pages_names_the_old_mark`: no text file under
    the crate's directory (its stylesheets, `loading.html`, `loading.css`, `files/**` but
    the fonts, `playground/**`, `src/**`) and none of `temper-service`'s
    `src/clients/google/pages.rs`, `src/setup/routes.rs` and `src/clients/no-bundle.html`
    names `#b8664e`, in any case.
- `heavy cargo nextest run` passes for `-p temper-client-view`, `-p
  temper-client-shell-web`, `-p temper-client-gallery`, `-p temper-client-journeys`, `-p
  temper-client-shell-terminal`, `-p temper-client-world`; `heavy cargo check --tests`
  compiles `-p temper-client-tui`, `-p temper-client-browser-world`.

## Inputs

- `temper/docs/design/client/browser.md`, 2.5, *Outside a project* (*The old look is gone
  for good*) and the icons' paragraph of 2.5.
- `temper/docs/plan/look-outside-pages/notes/contracts.md`, 1;
  `temper/docs/plan/look-outside-pages/notes/common.md`.
- The request's message, *So that the old look is gone for good* (`git show -s
  43a18a5042`).

## Owns and may change

- Owns: the seven constants and their slots in `look.rs`; the old rules, variables and
  marker in `temper.css`; `files/brand.svg`; the two tests.
- May change, rows and assertions only:
  - `tests/look.rs`: `each_style_or_region_is_named_under_temper`'s five rows of the gone
    tokens (and its array's length), `account_rows_and_sign_ins_have_distinct_style_names`
    (it compares against `SIGN_IN`: compare the account rows against another kept token,
    or drop that half, and say which).
  - `tests/terminal.rs`, `the_terminal_draws_a_chats_pages_in_the_browser_alone`: `BANNER`
    becomes `TOP`.
  - `temper-client-shell-web`'s `tests/suite/main.rs`, `the_approval_code_reads_at_a_glance`:
    it reads the outside marker's `.temper-outside .temper-approve-code` rule (the mono
    face at 30 px, letter-spaced .14em, in the ink), no longer the old block's place.
  - `temper/tools/gallery/src/tests.rs`, `print_writes_every_page_and_the_bundle`: the
    bundled images' row without `brand.svg`.
- If any code still draws a gone token (the compiler says), it is a red of the task that
  owned that page: report it under `## Left` with its file, and stop there rather than
  redraw that page.
- Read, never change: the outside marker's rules (`outside-looks`'s and
  `outside-frame`'s); `files/menu.svg` (the rail's unfold still reads it).

## Steps

1. `look.rs`: remove the seven constants; their names to `temper-reserved-N`.
2. `temper.css`: remove the rules, the old approval block and its marker, the variables;
   the base brand `::before`; the two accents. Check with a throwaway count (untracked,
   deleted before the push) that every remaining `.temper-…` class is a look name.
3. Delete `files/brand.svg`.
4. The two tests and the rows above. Each test must bite: put one old rule back and see
   the first fail; put `#B8664E` into `loading.css` and see the second fail.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py` (`--write` only to lower);
   `heavy cargo clippy -p temper-client-view -p temper-client-shell-web --all-targets --
   -D warnings`; the focused runs and checks of Acceptance; `heavy cargo run -p
   temper-client-journeys` reprints, and the journeys' focused tests pass (no committed
   print names a gone class: if one does, reprint and say which); commit with the trailer
   (common.md), push.

## Report

As the implementer's text says. `## Account` rows: each look that left `look.rs`; their
rules; `brand.svg`; the accent; the class test (with the count of classes it checks); the
old mark's test.
