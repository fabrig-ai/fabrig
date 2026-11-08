+++
id = "begin-sections"
part = "begin"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
]
effort = "high"
minutes = 80
+++

# Home's Needs you and Chats stay out while To begin shows

## Goal

The request's task 1 (`temper/docs/plan/look-round-20/inputs/request.md`): the look request `to-begin-two-steps` (`4332a26265`) says Home's own Needs you and Chats sections are left out while the To begin card shows, "even with a chat started from another page". Today `temper-client-view`'s `pages/home.rs` leaves them out only while they are empty, so a need or a chat draws "Needs you · 1" and "Chats" under the card, in both shells and at every width. Leave them out whatever they hold, in both shells; the gallery's state that drew them under the card takes a name that says it doesn't.

Acceptance:

1. **The view** (`pages/home.rs`):
   - `main`: while `facts.to_begin()`, none of `needs`, `goals` and `chats` is drawn, whatever the inbox and the chats hold; once the card goes (its last step, a skip), they draw as on any Home, as today. The intro, the card, the Away block (`decided`), the foot and the sheets are unchanged.
   - `needs` and `chats` lose what only served To begin, since neither runs while it shows: `needs`' early return on `count == 0 && facts.to_begin()` and its `!facts.to_begin()` guards (on "Answer in a row", the others' rows and the phone block); `chats`' `beginning` parameter and its early return. No other behaviour of either changes.
2. **Tests,** rows of existing tables, each rule broken once by hand (put the old guard back, see the row fail):
   - `src/tests/home.rs`, `empty_sections_are_hidden_only_while_to_begin_shows` becomes the table `sections_are_hidden_while_to_begin_shows`: its rows are what the sections hold (empty, as today; a need, `PARSED`, and a chat, through the file's fixtures, `home_documents` with a `NeedsFixture` and its `chat` or a `ChatFixture`) by whether To begin shows (shown, or skipped with `SkipToBegin`), each row checked in `f::page_view` and `f::browser_page_view`: no `look::HOME_NEEDS` and no `look::HOME_CHATS` section while it shows, no "Needs you · 1" heading and no "All 1 chats" link; once skipped, both sections, with the need's card or the empty line, and the chat's row or "No chats yet.".
   - `beginning_keeps_only_the_card_and_no_phone_block_or_rows` pins the rule this task removes: delete it, its case a row of the table above.
   - Any other test of the file that drew a section under the card (a `beginning` fixture: `everyday(true, ..)`, `home_goals(.., true)`, `away_home(.., true, ..)`, `status_home(.., true, ..)`) keeps what else it checks and takes the sections' absence; name each in the report.
3. **The gallery** (`temper/tools/gallery`), by `temper/docs/design/client/browser.md`, 7.2's rows:
   - `home-needs-under-begin` is renamed `home-begin-with-needs`, its deliveries unchanged (one question, one chat, To begin showing), in every place `samples.rs` names it; `names.txt` takes the new name in its sorted place.
   - `tests.rs`, `each_state_holds_what_its_name_says`'s rows: `home-begin-with-needs` holds the heading "To begin", `Part::NoStyle` for `look::HOME_NEEDS` and `look::HOME_CHATS`, and `Part::Absent` for the heading "Needs you · 1" and the link "All 1 chats"; `home-begin-settings-left` keeps its parts and gains `Part::NoStyle { style: look::HOME_CHATS }`.
4. **Prints:** every journey reprinted (`notes/common.md`, *Prints*). `project-landing/01.html`, the one print with the card, draws no section under it today, so none is expected to move; a print that moves is named in the report with why.
5. **Status:** delete the **Not yet** entry in `temper/docs/design/client/README.status.md` naming `begin-sections`; the one in `browser.status.md` naming it; and the sentences that point at this gap: "The nonempty Home sections' suppression gap is `README.status.md`, 3.5's." in `browser.status.md`'s first paragraph and "; the nonempty Home sections' suppression remains `README.status.md`, 3.5's gap" in `journeys.status.md`'s first paragraph (the sentence before it ending at "drawing route."). No other design line.
6. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-view`, `-p temper-client-gallery`, `-p temper-client-journeys`, `-p temper-client-shell-terminal`, `-p temper-client-world` (the view's readers: what Home draws changes meaning); clippy on `temper-client-view` and `temper-client-gallery`.

One commit or more, each `Look-request: to-begin-two-steps`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-20/notes/common.md`.
- `temper/docs/plan/look-round-20/inputs/request.md`, task 1.
- `temper/docs/design/client/README.md`, 3.5, *Home*: the list under its first paragraph (*Needs you* and *Chats* are left out while To begin shows), then *Needs you* and *Chats*.
- `temper/docs/design/client/browser.md`, 7.2, the `home-begin-with-needs` and `home-begin-settings-left` rows.
- The request: `git show 4332a26265`, its message's *States the page does not show*, "Home's own Needs you and Chats sections".

## Owns and may change

Owns the header's files. Reader and neighbour files: the journeys' prints (`temper/docs/design/client/journeys/*/[0-9][0-9].html`), regenerated alone; the status lines of acceptance 5.

Options, each decided:

- **The table's need and chat need a fixture the file lacks:** build them from `home_documents` and `ChatFixture` as `home_goals` and `everyday` do; no new fixture file.
- **A `needs` helper becomes unused** once its To begin branches go (a word, a style): remove it where nothing else reads it, and name it; a word of `words.rs` it leaves unused stays, named in the report, since the vocabulary is regenerated by its own test.
- **The terminal's Home** reads the same `main`: no terminal-only change; a terminal golden that drew a section under the card is regenerated by its own tool and named.

## Steps

1. Read the request's message and the design sections of Inputs.
2. The view (acceptance 1); the table and the tests it touches (acceptance 2); `heavy cargo nextest run -p temper-client-view`.
3. The gallery (acceptance 3); `heavy cargo nextest run -p temper-client-gallery`.
4. The reprint (acceptance 4); the status lines (acceptance 5); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; clippy; the runs of acceptance 6; `python3 -I -B tools/citations/citations.py --base origin/main`.
5. Commit with `Look-request: to-begin-two-steps` and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| Home's Needs you left out while To begin shows, with a need | landed (`home.rs`, `main`); both shells, every width (the phone block is `needs`') |
| Home's Chats left out while To begin shows, with a chat started from another page | landed (`home.rs`, `main`) |
| Once the card goes, by its last step or a skip, both sections back | landed; the table's skipped rows |
| The gallery's `home-needs-under-begin` and `home-begin-settings-left` | `home-begin-with-needs`, neither section drawn; `home-begin-settings-left` with no Chats |
| The journeys' prints | reprinted; unchanged, or each change named |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4); compiling, prints and focused tests establish no rendered equality |

## Report

The implementer's report: the view's change, the table's rows and each rule broken by hand, the tests that took the sections' absence, the gallery's rename and rows, the prints, the options taken, the runs' counts, and the account as committed.
