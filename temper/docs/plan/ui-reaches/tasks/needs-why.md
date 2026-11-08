+++
id = "needs-why"
part = "client"
kind = "feature"
needs = ["away-client", "decided-client"]
files = [
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-domain/src/forge/ways.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/tests/pages/needs.rs",
  "temper/crates/temper-client-domain/src/tests/forge_ways.rs",
  "temper/crates/temper-client-view/src/pages/needs.rs",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/pages/inbox.rs",
  "temper/crates/temper-client-view/src/pages/decided.rs",
  "temper/crates/temper-client-view/src/shared/mod.rs",
  "temper/crates/temper-client-view/src/shared/reach.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/needs.rs",
  "temper/crates/temper-client-view/src/tests/inbox.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/**",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/needs.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
  "temper/docs/design/client/journeys/**",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 140
+++

# Why each need reached the person

## Goal

Every need says why it reached the person, in one line under its question, on *Needs you*, on Home's card and in the terminal's inbox: a question by its asking task's asked row and that row's reach, a question no call judged as asked directly, and proposals, held tasks and funds by their kind. The reason is one domain function every shell calls, so the TUI's launchpad can say it later (`temper/docs/design/client/README.md`, 3.5, *Needs you*, *Why it reached you*). Stop asking is `stop-asking`'s.

Acceptance:

1. **`pages::needs::why`** (`temper/crates/temper-client-domain/src/pages/needs.rs`, sketch below), by the card's kind:
   - a question by `message` (`ReadValue::Question`): its asking task is `from`'s where that is `Requester::Task(n)`, else the question's `task.task`; its time `since`;
   - a person task in words (`crate::question::in_words`): its asking task is its `requester` where that is `Requester::Task(n)`; its time `since`, or with none the newest row;
   - the row: the newest of `Pages::decisions(project)`'s `asked` rows (`decided-client`) whose `task` is the asking task and whose `at` is at or before the time; its reach by `reach_from`; a row whose reach is decided, or fails to decode, counts as none;
   - `Why::Reach` for a row; `Why::Direct` for none with the document `Delivered` and not `cut`; `None` for none with the document cut, `Ended`, `Unknown` or unwatched;
   - `Why::Proposal`, `Why::Held` and `Why::Funds` for those cards (a held task whose hold is `HoldReason::Budget` is funds, as `ask::read` has it); `None` for an approval and any other person task.
2. **The card waits:** `pages::needs::ready(root, need)` is false while a question's project's document is `Unknown` (watched, not yet delivered); the browser's *Needs you* column stays empty while its shown need is not ready, as it stays for the inbox, and Home's first card is left out until ready, its region's head drawn. The terminal's list does not wait.
3. **Kind 8 of the shown card's project** (`temper/crates/temper-client-domain/src/forge/ways.rs`): a second classes slot, read as `away-client`'s is, for the project of the need *Needs you* shows where it is not the open project, dropped when it is not; `progress` takes that project (from `route.rs`, beside `root.pages().project()`), and `Side::classes(project)` answers either slot.
4. **The words,** `shared::why(root, w, need) -> Option<Box<[u8]>>` in `temper/crates/temper-client-view/src/shared/reach.rs`, re-exported from `shared`: "Reached you because CLAUSE, which CLASS." for a reach, and the fixed lines for `Direct`, `Proposal`, `Held` and `Funds`, exactly as 3.5's *Why it reached you* words them, `under`'s words and set names from `root.forge().classes(project)`, "one of PROJECT’s path sets" where unread. `shared::allowed`'s callers (the decided region, `temper-client-view/src/pages/decided.rs`) now pass `root.forge().classes(row.project)` in place of `None`.
5. **Drawn** in `temper-why` (a new look name, `look::WHY`), a paragraph under the question line: *Needs you*'s card (`pages/needs.rs`, `card_main`), Home's first card (`pages/home.rs`, `ask`), and each needing entry of the terminal's inbox under its words (`pages/inbox.rs`, `entry`). Its rule under `/* ==== ui: needs ==== */` in `temper/crates/temper-client-shell-web/temper.css`: 14px, `--gf-ink-2`, its top margin 8px less than the card's gap.
6. **Rows:** one new table test in the domain for `why` and `ready`, beside `src/tests/pages/needs.rs`'s session tests (a new behaviour; each case its row): a delegate's question matched to its newest asked row at or before `since`, a later row of the same task ignored, another task's row ignored, a question with no `from` by its own task, a person task in words by its requester, no row with the document delivered (`Direct`), with it cut (`None`), unwatched or ended (`None`), unknown (`None`, not ready), a row with a decided reach (`Direct`), proposal, held, funds and an approval; in `src/tests/forge_ways.rs`, the second slot's rows (read for another project's shown card, dropped when the card goes, none for the open project's); in the view, rows of existing tests only: `src/tests/needs.rs`'s `example_parts_are_in_order_and_choices_keep_their_names_and_acts` (the line after the question) and `empty_column_until_delivered_and_terminal_keeps_its_list` (waiting for the document), a words row for each class and reason (`kind` with each class, `set` named and unread, `under` with words, `always`, `Direct`, the three fixed lines) in the nearest table of `src/tests/needs.rs`; `src/tests/inbox.rs`'s `cards_are_grouped_in_lists_with_kinds_and_named_project_paths` the terminal's line; `src/tests/home.rs` Home's card waiting and drawing its line; `src/tests/look.rs` the new name.
7. **The gallery** (rows only): state `needs-why` (`temper/docs/design/client/browser.md`, 7.2), its domain built in `temper/tools/gallery/src/needs.rs` with a delivered `decided` document holding the question's asked row, an `implementation` reach at `Ask`; in `samples.rs`, `names.txt` and `each_state_holds_what_its_name_says`. The existing needs states keep drawing (their documents delivered or their line absent): any whose tree moves is named.
8. **Generated and recorded files:** terminal goldens and journey prints that now show a reason line are regenerated (`SKEIN_CODEGEN_WRITE=1` for the goldens' test; `temper-client-journeys again` for prints), each named with why it moved.
9. **Runs,** counts in the report: `heavy cargo nextest run` for `-p temper-client-domain`, `-p temper-client-view`, `-p temper-client-gallery`, `-p temper-client-world`, `-p temper-client-tui`, `-p temper-client-shell-terminal` and `-p temper-client-journeys`; `heavy cargo check --tests -p temper-client-browser-world -p temper-one-process-world -p temper`; `heavy cargo check -p temper-client-shell-web --target wasm32-unknown-unknown`; clippy on the packages changed, and on `temper-client-shell-web` for wasm32.
10. **Status:** delete the **Not yet** entries naming `ui-reaches/needs-why` in `temper/docs/design/client/README.status.md` and `browser.status.md`. The TUI's entry (milestone T1) stays.

## Inputs

- `temper/docs/plan/ui-reaches/notes/common.md`, all of it.
- `temper/docs/plan/ui-reaches/notes/contracts.md`, section 1 (`Reach`, `Reason`, `Class`, `Kind::fixed`, `Reach::decided`, `Classes::set`) and section 3 (`reach_from`); section 4 (`Asked`).
- `temper/docs/design/client/README.md`, 3.5: *Needs you* (the card's parts, *Why it reached you*, *A press stays under the finger* in *What the client keeps*), *A card that needs the person* (the terminal's), **Decided for you** (*Its documents*); 5.13, *What the client holds* (kind 8 of the shown card's project).
- `temper/docs/design/client/browser.md`, 7.2 (`needs-why`).
- `temper/docs/design/domain/forge.md`, 5.9 (asked rows) and 5.10 (the judgement's reasons).
- The code: `temper-client-domain`'s `pages/needs.rs` (`Need`, `Session::shown`), `pages/inbox.rs` (`Card`, `question_words`), `question.rs` (`in_words`), `pages/decided.rs` (`decided-client`'s `decisions`, `Document`), `forge/ways.rs` (`away-client`'s classes slot, `progress`), `route.rs` (`ways::progress`'s call); the view's `pages/ask.rs` (`read`, `Kind`), `pages/needs.rs`, `pages/home.rs` (`ask`), `pages/inbox.rs` (`entry`, `body`), `shared/reach.rs` (`object`, `allowed`). Survey 3 (`card kinds and the task that asked`) found the fields above: `Question.from`, `Question.since`, `PersonTask.requester`, `PersonTask.since`.

## Owns and may change

Owns `why`, `ready`, the second classes slot, `shared::why` and its words, the line's drawing in the three places, the CSS rule, the rows and the state, the regenerated goldens and prints, and the status entries. Not Stop asking (`stop-asking`), not the TUI's crates (its line is temper/tui's, with milestone T1).

Sketch, binding at its outer layer:

```rust
// temper/crates/temper-client-domain/src/pages/needs.rs
/// Why a need reached the person (temper/docs/design/client/README.md, 3.5, Needs you, Why it reached you).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Why { Reach { project: u32, reach: temper_protocol_reaches::Reach }, Direct, Proposal, Held, Funds }
/// None where the need has no line, or its document does not say.
#[must_use] pub fn why(root: &crate::Domain, need: Need<'_>) -> Option<Why>;
/// False while a question's project's `decided` document is watched and not yet delivered.
#[must_use] pub fn ready(root: &crate::Domain, need: Need<'_>) -> bool;

// temper/crates/temper-client-view/src/shared/reach.rs
/// A need's reason line, every shell's (README 3.5, Needs you, Why it reached you).
#[must_use] pub fn why(root: &temper_client_domain::Domain, w: &Words, need: temper_client_domain::pages::needs::Need<'_>) -> Option<Box<[u8]>>;
```

May change, each only as said:

- `temper/crates/temper-client-view/src/pages/ask.rs`: a helper the line needs from `Asked` (its kind), no change to what it draws.
- A test or fixture elsewhere that calls `forge::ways::progress` (found by the compiler): the new argument, `None` where it has no shown card, named in the report.
- A client-world story (`temper/tests/client`) that presses a question's choice on *Needs you* before its project's `decided` document is delivered: pump until the card draws, keeping what it checks, and name it.

Options, each decided:

- **A question's `from` is a person or the deployment:** its asking task is the question's own task, as acceptance 1 says; no other source.
- **The terminal's list for a card of an unwatched project:** no line, as for an unknown document; say how many fixture cards that leaves without one.
- **The domain's table needs a fixture `decided` document:** build it with the wire's `Decisions::new` in the test's fixtures beside the inbox's, no new helper module.

## Steps

1. Read the Inputs. `why` and `ready` and their table (acceptance 1, 2, 6's domain rows); bite: drop the `at` bound, see the later-row case fail, restore.
2. The second classes slot and its rows (acceptance 3).
3. The words and the three drawings, the CSS rule, the view's rows (acceptance 4, 5, 6); bite: draw Home's card before `ready`, see its row fail, restore.
4. The gallery's state; the goldens and prints (acceptance 7, 8); the runs of acceptance 9; the status entries; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; commit; `python3 -I -B tools/citations/citations.py --base origin/main`; push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report: `why`'s matching as written, `ready` and where each page waits, the second slot, the words, the rows and their bites, the options taken, the goldens and prints regenerated (each with why it moved), and the runs' counts.
