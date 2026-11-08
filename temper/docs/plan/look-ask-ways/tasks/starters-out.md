+++
id = "starters-out"
part = "removal"
kind = "migration"
needs = ["opening"]
files = ["temper/crates/temper-client-domain/src/pages.rs", "temper/crates/temper-client-domain/src/pages/chats.rs", "temper/crates/temper-client-domain/src/tests/pages/chats.rs", "temper/crates/temper-client-view/src/pages/reference.rs", "temper/crates/temper-client-view/src/tests/reference.rs", "temper/crates/temper-client-view/src/pages.rs", "temper/crates/temper-client-view/src/tests.rs", "temper/crates/temper-client-view/Cargo.toml"]
effort = "medium"
minutes = 80
+++

## Goal

Remove what only the old opening reached, now that `opening` has taken
its way in away: the starters' act, the reference's act, the page's
reference mode and the pane beside Ask temper's page
(`temper/docs/design/client/README.md`, 3.5, *temper's own way*: the
client neither compiles the page in nor draws it beside the page).

Acceptance:

- `temper-client-domain`: `pages::Act::Starter` and
  `pages::Act::ShowReference` are gone, with their routing in
  `pages.rs`, their arms in every exhaustive list, and `Chats::reference`
  with what reads it (`Act::Mode`'s branch for the Ask page, the hint's
  choice in the view). `Act::Mode` keeps its other branches.
- `temper-client-view`: `pages/reference.rs` and `src/tests/reference.rs`
  are gone, with their `mod` lines; the words and look tokens only they
  used go (tokens renamed `temper-reserved-N` in `NAMES`), and their
  `temper.css` rules where they stand (`.temper-doc-flow` and its parts,
  if nothing else draws them); `temper-docs` leaves the view's
  `Cargo.toml` if nothing else of the view names it.
- `temper-docs` keeps the page `how-temper-builds-temper` and its
  constant: the help pages and Ask temper's `see help` read it.
- The domain test `reference_opens_only_at_ask_without_a_request` goes;
  any other row that pressed a starter or the reference goes with it, or
  is changed to the nearest act the page still has, and named.
- `temper-client-domain`'s and `temper-client-view`'s focused tests, the
  gallery's and the client world's pass; `temper-client-tui` and
  `temper-client-shell-terminal` build their tests.

## Inputs

- `temper/docs/plan/look-ask-ways/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, *Ask temper's page* and
  *temper's own way* (this cut's revision).

## Owns and may change

- Owns the files above.
- Neighbour files, for the removal alone: every file of
  `temper-client-domain`, `temper-client-view`, `temper-client-tui`,
  `temper-client-shell-terminal`, `temper/tests/client` and
  `temper/tools/gallery` that names a removed item, each edited only to
  drop it (an arm, a `use`, a row); `src/words.rs` and `src/look.rs` for
  the words and tokens only the pane used; `temper.css` for the pane's
  rules.
- Not yours: `pages/ask_page.rs` but a line that still names a removed
  item; the `ways` state; `temper/docs/help/`.
- Options: if a removed word or token is drawn elsewhere, keep it and
  say where; if a gallery state names `ShowReference` still (it should
  not, after `opening`), drop that state and its prints with
  `names.txt` kept sorted, and say so.

## Steps

1. `rg -n 'ShowReference|Act::Starter|\.reference\b|reference::pane|HOW_TEMPER_BUILDS_TEMPER'
   temper/crates temper/tests temper/tools` and read each hit.
2. Remove them; let the compiler find the arms.
3. `heavy cargo nextest run -p temper-client-domain --lib`; `heavy cargo
   nextest run -p temper-client-view --lib`; `heavy cargo nextest run -p
   temper-client-gallery`; `heavy cargo nextest run -p
   temper-client-world --test suite -E 'test(/ask/) | test(/chat/)'`;
   `heavy cargo check -p temper-client-tui --tests`; `heavy cargo check
   -p temper-client-shell-terminal --tests`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py` (`--write` to
   lower what the removal lowers); `heavy cargo clippy -p
   temper-client-domain -p temper-client-view --all-targets -- -D
   warnings`. Commit with the trailer `Look-request: ask-temper-ways` and
   push.

## Report

As the implementer's text says; list each removed item and each
neighbour file edited.
