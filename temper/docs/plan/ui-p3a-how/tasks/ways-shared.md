+++
id = "ways-shared"
part = "page"
kind = "migration"
needs = []
files = ["temper/crates/temper-client-view/src/pages/ways.rs", "temper/crates/temper-client-view/src/pages/ask_page.rs"]
effort = "medium"
minutes = 40
+++

# The ways' tabs, card and act, shared by Ask temper's page and How work runs

## Goal

Move what Ask temper's page draws of the stock ways, its tabs with the axis line,
a stop's card, a way's act and its refusal, into `temper-client-view`'s new
`pages/ways.rs` (`notes/contracts.md`, C4), so that How work runs (`how-page`)
draws them by the same code. A move: Ask temper's page draws exactly what it drew.

Acceptance:

1. `pages/ways.rs` holds C4's `tabs`, `card`, `act` and `refused`, each the code
   `ask_page.rs` has today for it; `ask_page.rs`'s `opening` and `foot` call them
   and keep the rest (the lede, the panel, the path's drawing, the legend).
2. No tree changes: the focused tests of `temper-client-view` pass unchanged,
   `tests/ask.rs`, `tests/toast.rs` and `tests/paths.rs` among them, and so do
   `temper-client-gallery`'s; clippy clean.

## Inputs

- `temper/docs/plan/ui-p3a-how/notes/common.md`; `notes/contracts.md`, C4.
- `temper/docs/design/client/README.md`, 3.5, *Ask temper's page* (the ways'
  tabs, the shown way, its act, a stop's card), and 5.13, *The page* (How work
  runs draws the same tabs, card and act).
- The code: `temper-client-view`'s `pages/ask_page.rs` (`opening`, `card`, `foot`,
  `text`), `pages.rs`'s module list.

## Owns and may change

Owns: `temper-client-view`'s new `src/pages/ways.rs` and `src/pages/ask_page.rs`.

Reader and neighbour files, and what may change there: `src/pages.rs`, the line
`pub(crate) mod ways;` in its alphabetical place. Nothing else.

Pre-approved options:

- **If `foot`'s legend and act** read better split otherwise than C4's `act`
  (the act slot alone): keep C4's signature, and say so.
- **If a helper** both files need (`text`, `paragraph`) is better shared: move it
  to `pages/ways.rs` as `pub(super)`, and say so.

## Steps

1. Move the four pieces into `pages/ways.rs` with C4's signatures and a module
   comment citing `README.md`, 3.5 and 5.13; call them from `ask_page.rs`.
2. No new test: a move. Check as `notes/common.md` says, for
   `temper-client-view` and `temper-client-gallery`. Commit and push: the message
   says the ways' tabs, card and act move to one module for both pages.

## Report

Follow the implementer's text. Also say which helpers moved, and each test
package's CPU time.
