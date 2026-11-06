+++
id = "sheet-state"
part = "sheet"
kind = "feature"
needs = ["dismiss-making"]
files = ["temper/crates/temper-client-domain/src/forge/sheet.rs", "temper/crates/temper-client-domain/src/forge.rs", "temper/crates/temper-client-domain/src/forge/family.rs", "temper/crates/temper-client-domain/src/forge/making.rs", "temper/crates/temper-client-domain/src/forge/words.rs", "temper/crates/temper-client-domain/src/route.rs", "temper/crates/temper-client-domain/src/tests/sheet.rs", "temper/crates/temper-client-domain/src/tests/making.rs"]
effort = "high"
minutes = 100
+++

## Goal

The domain's half of "Add repositories": the state, reads, marks, counts, remedies, lines and sends of `notes/contracts.md`, *The sheet's state*, in a new `forge::sheet`. It draws nothing, and the old page stands until `sheet-view`.

Acceptance:
- opening `Address::Adopt` opens a `Sheet`, its defaults by the forges' rows; leaving drops it and drops any answer for it;
- reads follow the contract: one out at a time; next on `NearEnd`; at once while a search has found less than it has left; a page with no rows ends it; an answer for a forge or installation no longer chosen dropped; at `PICKER_ROWS` stops;
- a new forge or installation clears the rows, picks and search; a search keeps the picks;
- `shown` marks and `count` as `forge.md`, 5.1 says them, including "In PROJECT", "Copying…", "private" and the numbers without a total before the last page;
- `remedy` holds the table of `forge.md`, 5.1, *Not listed*; none for an owned or managed Forgejo;
- `making::lines` reads the addresses line by line as the mock's and `parse_copies`' rules, each line its own `Line`; `UseIt` rewrites a line; `AdoptIt` moves to `Source::Forge` on the target forge, its row picked once read;
- `Send`: adoptions one at a time through the form's `Adoption`, home only with one; `MakeRepository` for each `Line::To` or the one name with `into` this project, `home` the first where the project has none; all kept or adopted, the address goes to `Repositories`; a refusal keeps the sheet, what is done off it, its words in `Sent::Refused`;
- `temper-client-domain`'s focused tests and `temper-client-tools`' pass.

## Inputs

- `temper/docs/design/client/README.md`, 4.4 and 4.5; `temper/docs/design/client/forge.md`, 5.1; `temper/docs/design/draft/repository-copies.md`, 5.3, 5.4 and 9.
- The request's `README.md`, *The sheet* and *The picker, with many repositories*, and its `adopt.js`, which behaves as the sheet is to: follow its rules, not its code.
- `temper/docs/plan/look-round-13/notes/common.md`; `notes/contracts.md`, *The sheet's state*.

## Owns and may change

- Owns the `files` above.
- May change: `forge/form.rs` to expose what `sheet` reads, keeping `temper-client-tools/src/forge.rs`' use of `Form` as it is: `Text::Forge`, `Typed::Forge` and `Broken::ForgeNumber` stay for the command line. `pages.rs`, the one call that routes a `MakeRepository` from the sheet with the project's scope, as `pages/home.rs`'s sheet's does. `forge/github.rs`' `Answer::Page` arm, where `sheet` now reads it. `temper-client-domain/src/tests.rs`'s helpers.
- Not here: drawing, its words' use in the view, and removing "Forge number": `sheet-view`'s.
- Option: if `Field`'s writes don't fit `find`, a `Box<[u8]>` with `written`, and say so.
- Option: if a row's `w::Reached` is awkward held raw, a domain `Reached` as `boundary.rs`' has it. The contract's other names stay.

## Steps

1. Module and contract's types; route open and leave; `family::reachable`; tests in a new `temper-client-domain/src/tests/sheet.rs` for each acceptance line, on a fake answer. A `Sheet` with 213 rows read over three pages, as the request's page 16.
2. `making::lines` with a table of the mock's lines: `github.com/octo/vsr`, `octo/vsr`, `github/octo/vsr` (`NotAHost`), `a` (`Bad`), `https://host.example/o/n.git`, a name already there (`Taken`). Reduce the table, before the end, to rows that can still fail on a rule.
3. `Send` in `forge.rs`, with the progress the form's `pending` holds.
4. `cargo fmt`, old forms, clippy, focused tests, commit with the trailer, push.

## Report

As the implementer's text has it.
