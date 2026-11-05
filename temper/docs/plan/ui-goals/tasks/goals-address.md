+++
id = "goals-address"
part = "goals-address"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/lib.rs",
  "temper/crates/temper-client-domain/src/tests/address.rs",
  "temper/crates/temper-client-view/src/pages/goals.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
]
effort = "medium"
minutes = 60
+++

# The Goals address

## Goal

`Address::Goals { project, filter }` exists, parses and prints, and opens in the project's frame with a level-1
heading "Goals" and nothing else. The domain's page and the view's rows follow.

Acceptance:

- `Address::Goals` and `GoalsFilter` as `temper/docs/plan/ui-goals/notes/contracts.md`, The address, gives them.
- Every path of `temper/docs/design/client/browser.md`, 2.3 round-trips; an unread `state` is dropped, the address
  replaced with the printed one.
- A project's Goals opens in its frame: the rail shows, its project is the address's, and choosing another project
  keeps the page (`Goals` with that project and the filter All), as `Home`, `Settings` and `Configure` do.
- No rail entry yet, and no reads.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/browser.md`, 2.3; `temper/docs/design/client/README.md`, 3.5, *Goals*, and 5.3.

## Owns and may change

Owns, as `files` lists them.

May also change, each one arm or line for the variant, and only that:

- `temper-client-domain/src`: `pages.rs` (at `:520` keeps the page across a project switch; `:776`, `:1028`
  `address_project`, in its `Some(*project)` arm; `:1213`), `pages/connect_model.rs`, `pages/switcher.rs`,
  `route.rs`, `departure.rs`.
- `temper-client-view/src`: `choice.rs`, `pages/connect_model.rs`, `frame.rs` (its six arms, each current on nothing
  here), `tests/rail.rs`, `tests/fixtures.rs`.
- `temper/tools/gallery/src/samples.rs`, the two arms.

Options, each pre-approved:

- If an arm's right answer isn't plain from `Configure`'s or `Home`'s neighbour, take `Home`'s.
- `root.rs`'s `if let` chain falls through silently: add the arm for `Address::Goals`; it's a line no compiler asks for.

## Steps

1. **`address.rs`:** the variant, `GoalsFilter`, the print and parse (`/projects/N/goals`; the doc comment on `print`
   gains it). `lib.rs` re-exports it.
2. **The arms,** by the compiler's list. `Goals` joins `Home` in what is a project's page.
3. **`pages/goals.rs` of the view:** `pub(crate) fn main` as the contract, opening `Role::Main` with `look::MAIN`
   and a level-1 heading of `Word::Goals`; the module cites `README.md`, 3.5. A new word as `notes/common.md` says.
4. **Tests,** `tests/address.rs`: the page table gains it; `goals_filter_is_its_query` as `:334`'s; an unread `state`.
   In the view, `tests/frame.rs`: `goals_opens_in_its_projects_frame`.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
