+++
id = "sheet-view"
part = "sheet"
kind = "feature"
needs = ["sheet-state", "list-near", "connect-look"]
files = ["temper/crates/temper-client-view/src/forge/sheet.rs", "temper/crates/temper-client-view/src/forge/page.rs", "temper/crates/temper-client-view/src/forge/bindings.rs", "temper/crates/temper-client-view/src/forge.rs", "temper/crates/temper-client-view/src/pages/settings.rs", "temper/crates/temper-client-view/src/root.rs", "temper/crates/temper-client-domain/src/forge/words.rs", "temper/crates/temper-client-view/src/tests/forge_page.rs", "temper/crates/temper-client-view/src/tests/sheet.rs", "temper/tests/client/tests/suite/adoption.rs", "temper/tools/gallery/**"]
effort = "high"
minutes = 120
+++

## Goal

The sheet, drawn: `Address::Adopt` is Settings' Repositories with the dialog "Add repositories to PROJECT" over it, on its first source, "Adopt" (pages 16 to 20). The Adopt page goes, and with it "Forge number". The other two sources are `sheet-copies`', and until it lands they draw their head and an empty panel.

Acceptance:
- pages 16 to 20 draw as the request's, by the tokens of `notes/contracts.md`, at 1440, 1000 and 420, the sheet a dialog and a bottom sheet on a phone; the list is a bound `Role::List` that `State::Continues` while `more`, `State::Busy` while a read is out, its rows `Role::Check`s named `OWNER/NAME`, a mark in `em`, disabled where the mark is;
- "Find by name" draws only for a long list, in a search landmark; the count as `forge.md`, 5.1; one account, one forge: their fields go;
- "Not listed" and "More" fold; "More"'s summary says its role and CI; "Make it the project's home" goes with more than one;
- the button counts, and sends;
- Settings' Repositories section's "Adopt" control says "Add repositories", in the head alone, and the empty band's is the same; `Word::Forge`'s "Forge number" and `Word::Title*` are gone, the page's `page()` is gone, and `outcome` stays for whoever calls it;
- the terminal prints the sheet as a dialog group, with its fields numbered and "more" reachable;
- every other link to `Adopt` (Home's three, Chats' band, the forge's row) still reaches it;
- focused tests of `temper-client-view`, `temper-client-domain`, `temper-client-shell-terminal`, `temper/tests/client`'s and `temper/tests/one-process`'s suites and `the_committed_journeys_are_their_print` pass.

## Inputs

- `temper/docs/design/client/README.md`, 4.4; `temper/docs/design/client/forge.md`, 5.1.
- The request's `README.md`, and pages 16 to 20.
- `temper/docs/plan/look-round-13/notes/common.md`; `notes/contracts.md`, *The sheet's state* and *What the view binds*.

## Owns and may change

- Owns the `files` above.
- May change, for the walk alone: every test that walks the Adopt page: `temper/tests/client/src/fixtures.rs`, `temper/tests/client/src/app.rs`, `temper/tests/one-process/src/{fixtures,world,stories}.rs`, `temper-client-view/src/tests/{settings,home,chats,frame,rail,fixtures}.rs`, `temper-client-domain/src/tests{.rs,/form.rs}`, and, compiled only, `temper/crates/temper/tests/support/{person,managed_client}.rs`, `temper/crates/temper/tests/process.rs`, `temper/tests/client-browser/**`.
- May change: `frame.rs`, `pages/home.rs`, `pages/chats.rs`, `forge/connect.rs`, words only for the link's name where the request gives one; `temper/tools/gallery/src/{samples,tests,worlds}.rs` and `worlds/github.rs`, `names.txt`: `adoption`, `adoption-filled`, `adoption-refused` become `sheet-adopt`, `sheet-searching`, `sheet-none`, `sheet-few`, `sheet-empty`, `names.txt` kept sorted.
- Keeps `Address::Adopt` and its path: the design has it as the sheet's address. `temper-client-tools/src/forge.rs` isn't touched.
- Option: a check's disabled mark: `State::Disabled` on `Role::Check`; if the builder refuses, say so and draw the row as text with the mark.
- Option: if a rule of the request's sheet needs an element the mapping doesn't make (`label`, `em`), adapt the selector in `temper.css` where it stands, keeping every declaration, and name it for the account.

## Steps

1. `forge/sheet.rs` and bindings; settings' call; root's `Adopt` arm; the page out.
2. Tests: `temper-client-view/src/tests/sheet.rs` for each acceptance line by the tree: counts, marks, the list's states, the fields hidden with one forge, its Cancel with Escape. Move what `tests/forge_page.rs` still checks of the form's fields. A world story in `temper/tests/client/tests/suite/adoption.rs`: open, read a page, search across pages, pick two, adopt both, land on Repositories; refused, the sheet stays.
3. Goldens and gallery; journeys reprint.
4. Compare at 1440, 1000, 420 against pages 16 to 20 by a throwaway driver; the render of an overflowing list scrolled reading on is the driver's too.
5. `cargo fmt`, old forms, clippy, focused tests, commit with the trailer, push.

## Report

As the implementer's text has it, with the differences from the request's pages named.

## Answers

2026-11-06 (temper/ui). The user's rule, 2026-11-06: a plan executed by fab runs focused and fuzzy alone, and is complete when it lands. Walks and render checks follow landing, and file findings back to fab. So drop this task's throwaway-driver comparison and screenshots: don't start a browser to compare. temper/ui runs the render check after the plan lands.

## Rebase (2026-11-07, step merge, foreman look-round-13/sheet-view/foreman-1)

merge the sheet vocabulary counts and gallery changes with integration’s model, forge and account changes
The merge failed in `temper/crates/temper-client-view/src/words.rs` and `temper/tools/gallery/src/samples.rs`.
Rebase onto the landing trunk; the inspected integration tip is `e7a6504afb`.
Relevant incoming commits are `50e7171ce1` (composer picker), `ce16672eec` (forge rows), `64f1d4af60` (Accounts completion), and their merge resolutions `3221fbbb41` and `0aaea51072`.

In `words.rs`, retain integration’s vocabulary and recalculate `PHRASES`, `SEGMENTS` and `ENGLISH_BYTES` for the combined code.
The sheet commit changes only those three constants in this file; choosing either side’s totals drops the other side’s accounting.

In `samples.rs`, combine the `Then` imports: remove `Filled`, `Refused` and `KeyReady`, matching their combined enum removals.
Keep `"sheet-few" => Some(b"More")` alongside integration’s model-picker disclosure arms.
Keep all five `sheet-*` samples and `adoption_sheet`; retain integration’s picker samples, forge sample and Accounts completion changes.
Recount `N` from the combined table; the inspected tips imply 295 entries.
Keep `names.txt` sorted and consistent with the table; regenerate any stale prints, goldens and counted baselines using their tools.

Validate the combined vocabulary through focused `temper-client-view` tests and the gallery’s `names_are_the_promised_ones` and `each_state_holds_what_its_name_says` tests, through `heavy`.
Run only focused and fuzzy suites; retain the task’s prohibition on browser comparisons and walks.

## Rebase (2026-11-07, step merge, foreman look-round-13/sheet-view/foreman-2)

combine the sheet and incoming proposal vocabulary counts in words.rs
Rebase onto the landing trunk; the inspected integration tip is `0470f4a7ee`.
The merge conflicts in `temper/crates/temper-client-view/src/words.rs`.
Incoming merge `96b7b3852e` brings proposal wording from `183c94f536`, adding three phrases, five segments and 55 English bytes.
Retain `WorkProposedChange`, `WorkProposedChangeCount` and `WorkProposingChange`, including their enum entries, IDs and English forms.
Preserve the sheet’s vocabulary additions and regenerate the combined declarations rather than choosing either side’s totals.
The inspected combined code implies `PHRASES = 1946`, `SEGMENTS = 3387`, `ENGLISH_BYTES = 47527`, and `ALL` length 1299.
Use `SKEIN_CODEGEN_WRITE=1` with the focused `temper-client-view` test `tests::accounts::english_phrase_declarations_match_the_vocabulary` through `heavy` to regenerate, then verify without write mode.
Keep the automatically merged CSS and gallery changes from both sides.
Regenerate any combined prints, goldens or counted baselines found stale by focused checks.
Run only focused and fuzzy suites; retain the prohibition on browser comparisons and walks.
