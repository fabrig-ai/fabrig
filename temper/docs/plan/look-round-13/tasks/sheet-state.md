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

## Answers

2026-11-06 (temper/ui). Neither as asked: no schema change, and no invented fact. temper/ui answers. For a GitHub account with an installation, whatever its selection, the Not listed remedy is one line that's true either way: 'Check OWNER/NAME exists, and grant it to temper's App on GitHub.', linked to the App's installation page by its slug if the client holds that (github.com/apps/SLUG/installations/new, which asks for the account itself), and with no link if not. Emit that in place of Remedy::Grant's two cases, and don't add installation fields. Change forge.md 5.1's row to match, and note in its status file that GitHub's repository selection mode isn't carried, so the two cases aren't told apart. Keep NotInstalled and connected Forgejo's collaborator remedy as you have them. Focused and fuzzy alone.

## Rebase (2026-11-06, step merge, foreman look-round-13/sheet-state/foreman-1)

resolve family.rs by preserving sheet pagination and the updated dismissal scope.
The merge conflicts in `temper/crates/temper-client-domain/src/forge/family.rs`.
The landing trunk contains `73b7f0b8a0541e2608ec3b8ae4088007dd69d0ec` (“Keep making dismissal admission stable after erasure”).
Keep this task’s `reachable(forge, installation, page, limits)` builder.
Keep the trunk’s `dismiss_making` signature with `into: Option<u32>` and pass `into` into `w::DismissMakingParts`.
Keep the trunk’s `dismiss_making_builds_a_bounded_forge_request` test, covering `None`, `Some(3)`, decoded scope and oversized fields.
The scope preserves keyed replay after the failed row disappears (`temper/docs/design/client/README.md`, 4.6).
Retain the other automatically merged changes.
After resolving, run the focused dismissal test and sheet tests through heavy, then the task’s required focused checks.
