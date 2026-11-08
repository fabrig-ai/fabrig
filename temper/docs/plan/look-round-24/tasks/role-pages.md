+++
id = "role-pages"
part = "links"
kind = "feature"
needs = ["ways-wire"]
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/roles.rs",
  "temper/crates/temper-client-domain/src/tests/address.rs",
  "temper/crates/temper-client-domain/src/tests/pages/project_pages.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/pages/roles.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/settings.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 120
+++

# A role's page and a rule's page

## Goal

The client's half of the request's task 6 (`temper/docs/plan/look-round-24/inputs/request.md`): "Open the reviewer ›" is to open the reviewer's charter, and no page shows one, a gate's or a rule. Two pages in Settings' frame, `Role { project, charter }` and `Rule { project, kind, number }`, each drawn from the forge's read `View` of `charter N` or `rule K/N`, the same read and card a reply's view draws. This task makes the addresses and the pages; `role-links` points the cards' links at them.

Acceptance, by `temper/docs/design/client/README.md`, 3.5, the pages' table's rows `Role` and `Rule` and *A role's or a rule's page*, and `temper/docs/design/client/browser.md`, 2.3 (the two paths), as this cut wrote them, and `temper/docs/plan/look-round-24/notes/contracts.md`, *The client*:

1. **The addresses** (`temper-client-domain/src/address.rs`): `Address::Role { project: u32, charter: u32 }` and `Address::Rule { project: u32, kind: u32, number: u32 }`, after `Builders`, and the codec both ways: `/projects/<project>/settings/roles/<charter>` and `/projects/<project>/settings/rules/<kind>/<number>`, `kind` 1 or 2 alone, `charter` and `number` naturals (no zero), any other no address, as the codec refuses its other bad paths. The doc comment listing the paths (about line 111) names them.
2. **The page's state** (`temper-client-domain/src/pages/roles.rs`, a new module, modelled on `pages/builders.rs`): `pub struct Roles` holding the read in flight or answered, `configure`d when a `Role` or `Rule` address opens and left as Builders' is; it reads `w::Read::View(ViewRead { project, address })` with `address` `charter N` or `rule K/N` (the note's words), as `pages/chat/views.rs`'s `reading` builds it, once per address shown, and again when the session's `settings` watch names a newer revision, as a reply's view is read (`README.md`, 5.16, *Read when shown*). `pub fn read(&self, ...) -> Read<'_>` with `pub enum Read<'a> { Reading, Drawn(&'a w::View), Unreadable }`. `Pages` gains `roles` and `pub fn roles(&self) -> Option<&roles::Roles>`, beside `builders()`. A project switch keeps the page's kind (`pages.rs`, about line 923: a `Role` or `Rule` goes to `Settings { project }` of the new project, since a charter or a rule numbered there is another).
3. **The view** (`temper-client-view/src/pages/roles.rs`, a new module, modelled on `pages/builders.rs` and the reply's charter card in `views.rs`): `pub fn main(builder, root, words, pages, project)` draws in Settings' frame, How work runs' tab current: on a phone "‹", a link named "How work runs" to `Settings { project }`; a level-1 heading, the card's `name` with its first letter a capital ("The reviewer", "The project's rule 1"); the card's `who` in a muted paragraph; its facts as the reply's card draws them (`views.rs`'s function for a charter's card, reused, not copied: make it `pub(crate)` if it is private). While `Reading`, a muted paragraph "Reading role N…" or "Reading rule K/N…"; for `Unreadable`, a `not_drawn` answer or a refusal, a muted paragraph "This role can't be read now: charter N." or "This rule can't be read now: rule K/N.". No act. `root.rs` dispatches `Role` and `Rule` to it beside `Builders` (about line 301). New words go in `words.rs` beside Settings' (`Word::Role*`), in the view's phrasebook as its other words are.
4. **Compiler-forced arms** for the two variants, each edited for them alone, in any file of temper's tree: `temper-client-domain/src/pages.rs` (a settings page, its frame and rail, as `Builders`); `temper-client-view` (`frame.rs`, `root.rs`, `pages/connect_model.rs`, `pages/terminal/commands.rs`, `pages/settings.rs`'s tabs: How work runs current for both); the TUI (`temper-client-tui/src/view.rs`, `view/launchpad.rs`: as `Builders`); the gallery (`temper/tools/gallery/src/samples.rs`, as `Builders`; no new sample). Name each in the report.
5. **The rows, no new test function where a table takes them:**
   - the codec's table (`temper-client-domain/src/tests/address.rs`, about line 28): `(Address::Role { project: 3, charter: 4 }, b"/projects/3/settings/roles/4")`, `(Address::Rule { project: 3, kind: 2, number: 1 }, b"/projects/3/settings/rules/2/1")`, `(Address::Rule { project: 0, kind: 1, number: 7 }, ...)`; and in its refusals, `/projects/3/settings/roles/0`, `/projects/3/settings/rules/3/1`, `/projects/3/settings/rules/2`;
   - the project pages' table (`tests/pages/project_pages.rs`): opening `Role` requests the `View` read of `charter 4` in project 3, `Rule` of `rule 2/1`; an answer is `Drawn`; a refusal `Unreadable`;
   - the view: Settings' page test (`temper-client-view/src/tests/settings.rs`) gains rows for `Role` (a charter card answered: heading "The reviewer", `who`, its four facts as terms; while reading, "Reading role 4…") and `Rule` (a rule card answered, as `ways-wire`'s `views.rs` row's; unreadable, its line). If neither file holds a table these rows fit, add one test function per crate, named in the report, its rows these.
6. **Status:** delete the **Not yet** entries naming `role-pages` in `temper/docs/design/client/README.status.md` (3.5, near its end) and `temper/docs/design/client/browser.status.md` (2.3, at its end). No other design line.
7. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-domain`, `-p temper-client-view`, `-p temper-client-shell-web`, `-p temper-client-shell-terminal`, `-p temper-client-gallery`, `-p temper-client-world`; `cargo check -p temper-client-tui --tests`; clippy on every package changed. The journeys print no Settings page this task adds; run `heavy cargo nextest run -p temper-client-journeys` once.

One commit or more, each with the trailer `Look-request: ask-temper-ways`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-24/notes/common.md`, `temper/docs/plan/look-round-24/notes/contracts.md`.
- `temper/docs/plan/look-round-24/inputs/request.md`, task 6.
- `temper/docs/design/client/README.md`, 3.5, the pages' table, *Settings*, *A role's or a rule's page*; 4.6, *A view, the read `View`*; 5.16, *Read when shown*, *Drawn*.
- `temper/docs/design/client/browser.md`, 2.3, the addresses' table.
- Models: `temper-client-domain/src/pages/builders.rs`, `temper-client-view/src/pages/builders.rs`, `temper-client-domain/src/pages/chat/views.rs` (`reading`, `answer`), `temper-client-view/src/views.rs` (the charter card).

## Owns and may change

Owns the files above and the forced arms of acceptance 4. Not `pages/ask_page.rs` or `tests/ask.rs` (`role-links`), not `views.rs` but a function's visibility, not `paths/*` (`path-scale`), not the wire.

Options, each decided:

- **The settings watch's revision is not at hand** where the page reads: read once per address opened, and say so in the report as left; the design's re-read on a newer revision is then a **Not yet** the report names, for temper/ui.
- **The shell-web's address tests** list every page's path (`rg -n 'settings/builders' temper/crates/temper-client-shell-web`): add the two, keeping what they check.
- **A gallery test lists every address** (`samples.rs`, about lines 1313 and 1351): the two join `Builders`' arm; no sample.

## Steps

1. Read the Inputs and the models; the addresses and their codec rows (acceptance 1, the first row of 5); bite: accept kind 3, see its refusal row fail, restore.
2. The page's state and its rows (2); the view and its rows (3); the forced arms (4), compiling with `heavy cargo check -p PKG --tests`; bite: read `charter N` for a `Rule`, see the rule's read row fail, restore.
3. The status entries (6), the runs (7); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main` after committing.
4. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| A role's page, `/projects/P/settings/roles/N`, from `charter N` | landed (`temper-client-domain`, `temper-client-view`, `pages/roles.rs`) |
| A rule's page, `/projects/P/settings/rules/K/N`, from `rule K/N` | landed (the same modules); the view's words are `rule-view`'s, this plan's |
| A card's links to them | `role-links`, this plan's |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): no gallery state draws these pages |

## Report

The implementer's report: the addresses, the page's state and its read as written, the view's words, every forced arm by file, the rows and the bites with their failures, the runs' counts, and the account as committed.
