+++
id = "call-line"
part = "call-line"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/pages/chat_line.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/chat_line.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
  "temper/crates/temper-client-view/src/tests/delegation.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-domain/src/pages/calls.rs",
  "temper/crates/temper-client-domain/src/tests/pages/calls.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/Cargo.toml",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/**",
  "temper/tests/client/tests/live_text.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/journeys/**",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 110
+++

# A run of calls on one line, and its open state

## Goal

A person reading a chat sees what the agent did in one short muted line, "Read 7 docs, checked settings 3×  29 s", which opens to a row a call, across the turns that made them. It replaces `folded_calls`, `fold_bytes` and the words `ToolCalls` and `ToolRepeated`, on a chat's page and an agent task's conversation. This is the first of three tasks landing the look request `chat-work`.

Acceptance:

- A run is every call between two of the agent's texts, across turns; a text, a person's words, a card, a document's write or any other drawn line ends it, and a hidden line doesn't. Calls with cards and a document's write aren't counted, as `other_call` holds today, and they stay where they fall.
- A turn holding only run calls draws no region of its own; each run is one `Region` with `TURN`, then the markup in the contract.
- Name, description, time, failure, running state and rows are as `temper/docs/design/client/README.md`, 3.5, *its calls*, says, each in a test.
- The three dots yield to a running run (3.5, *While it works*). The call under way comes from `Chat::current_call`; an agent's page has no running state.
- On an agent's page (5.9) a run's results are its rows' outcomes (a call's `outcome`, else its `Block::Result`'s summary), a message holding only those draws no shell, and a result that anchors a delegate's card still does.
- Every rule of the overlay's *The line* block is in `temper.css`, selectors and declarations as the overlay has them; `.temper-agent-page` draws the same line and rows under its own selectors. Rules only `folded_calls` used are gone.
- `Word::ToolCalls`, `Word::ToolRepeated`, `Word::ToolSeparator` if nothing else reads it, `limits::fold_bytes`, `folded_calls` and `other_call`'s callers outside the run are gone, with old-form counts only falling.
- This task's **Not yet** entry in `temper/docs/design/client/README.status.md` is gone.
- At 1440, 1000 and 420 wide, closed and open, in throwaway renders of a gallery state or reprinted page: the line reads as the request's `03.html` does. The report says what differs and why.

## Inputs

- `temper/docs/design/client/README.md`, 3.5: *its calls*, *A call's line*, *While it works*, and *The tool line*; 5.9, its conversation; section 11's *An agent's conversation drawn as a chat's*.
- `temper/docs/design/draft/chat-reading.md`, section 2.
- `temper/docs/plan/look-round-6/notes/contracts.md`, binding; `temper/docs/plan/look-round-6/notes/common.md`.
- The request: `git show 3f707babb9`, its message's *The line* and *States the pages do not show*, its `overlay.css`, and `01.html`, `03.html`.

## Owns and may change

Owns the files in the header. `chat_line.rs`, `calls.rs` and their test modules are new.

Readers and neighbours:

- `temper/crates/temper-client-shell-terminal/src/tests/golden/chat_delegation_landed.txt` and any other golden naming "N tool calls": regenerated, each read for the terminal's meaning first. A line prints as "NAME: DESCRIPTION", rows under it.
- `temper/tests/client/tests/live_text.rs:99`: `Find::named(Role::Disclosure, b"1 tool call")` becomes the line's name in this test's words.
- `temper/tools/gallery/src/tests.rs`: the `Part::Described` and `Part::Absent` rows naming tool calls (near lines 1767, 2033, 2047, 2290, 2291) take the new names and descriptions. Any tool the gallery's worlds call that the table doesn't word is its name, counted.
- `temper/crates/temper-client-view/src/tests/delegation.rs` and `tests/task.rs`: `assert_call_fold` becomes `assert_run_line`; each reader names the new words. `temper/crates/temper/tests/support/person.rs` only if a step there stops holding.
- `temper/docs/design/client/journeys/**`: reprinted (`chat-draft`'s 21 and 22 show a fold today).

Options, each decided:

- If `Line::Turn`'s `turn` doesn't outlive the walk's iteration, `Run` copies what it needs of each call instead of borrowing. Keep it bounded by the window.
- If `limits.rs`'s node count no longer covers rows (a paragraph, a code text, a plain text and an out text a call), raise that count from `receiving.rows`, and say so. Text is bounded by the 256- and 160-byte cuts, on a character's boundary.
- If `chat-reading-a` or `chat-reading` (b) has changed `chat.rs` by catch-up, keep theirs, and the line stays in `chat_line.rs`.
- A summary's flex `gap` from `details.temper-turn-tool`'s existing rules may need a landing rule for the comma before "1 failed": `details.temper-work-failed > summary > [id^="skein-ui-description-"]::before { content: ', ' / ''; margin-left: -6px; }` is the default.
- The hand-made page's rows for a call under way carry an outcome: a call under way has none here, a difference the report names.

Not this task: `Opaque`, `TurnCut`, `Provisional`, `markdown`, "Older", `Line::Left` (tasks `answer` and `reading`); any jig or skein file; `current_call`'s facts beyond what the chat page holds.

## Steps

1. `temper_client_domain::pages::calls`, as the contract has it, `skein-json` added to the crate's manifest. `touched` reads `read_docs`'s `topic`, `search`, `from`; `settings`' `setting { kind, number }`, `history`; any other tool's first text or number member (text as it is, number as "KEY N"), bounded. `first_line` stops at a newline. `Chat::current_call` reads `current_run`. Tests in `tests/pages/calls.rs`, each tool's shapes and a cut input.
2. `pages/chat_line.rs`, citing `temper/docs/design/client/README.md`, 3.5, at module level:

   ```rust
   pub(crate) struct Run<'a> { /* the calls so far, their results' summaries, first and last turn's Wall */ }
   impl<'a> Run<'a> {
       pub(crate) fn with_capacity(calls: u32) -> Self;
       pub(crate) fn call(&mut self, call: &'a c::Call, at: Wall);
       pub(crate) fn result(&mut self, call: u64, summary: &'a c::Text);
       pub(crate) fn is_empty(&self) -> bool;
       /// Draws its turn and empties; nothing when empty and with nothing under way.
       pub(crate) fn flush(&mut self, b: &mut Builder, under_way: Option<&c::CurrentCall>, running: bool, words: &Words);
   }
   fn phrases(calls: &[..], words: &Words) -> Box<[u8]>;      // the table, one function
   fn alone(call: &c::Call, words: &Words) -> Box<[u8]>;      // a run of one call
   fn under_way(call: &c::CurrentCall, words: &Words) -> Box<[u8]>;
   fn time(first: Wall, last: Wall, words: &Words) -> Option<Box<[u8]>>; // keeps its own function: chat-reading settles a turn's start
   fn row(b: &mut Builder, tool: &[u8], touched: &Touched, outcome: Option<&[u8]>, failed: bool, words: &Words);
   ```

   Words go in `words.rs` beside the others: `WorkReadDocs`, `WorkReadPage`, `WorkSearched`, `WorkCheckedSettings` and its counted form, `WorkChecked`, `WorkReading`, `WorkSearching`, `WorkChecking`, `WorkInventory`, `WorkHistory`, `WorkFromLine`, `WorkFailedCount`, `WorkFailedOut`, `WorkSeconds`, `WorkMinutes`, `WorkRepeated` ("NAME ×N"). A removed word leaves its number unused.
3. `pages/chat.rs`: the walk holds a `Run`. `Line::Turn` no longer gets a region from the walk: `committed` and `committed_lines` walk blocks in order, flush before each text, card, write's line and proposal card, and open each region after flushing. Every other drawn line flushes first, the day's line and clock included (a turn's clock and day go with the first text or card it draws). `chat_sheet::before` flushes when it would draw. At the walk's end the last run flushes before `chat_sheet::before(.., None, ..)`, with `running` from `chats::working` on a chat page.
4. `pages/chats.rs`, `working`: it keeps its meaning for the status; the walk draws the dots only if no run was left running.
5. `look.rs` and `tests/look.rs`, the contract's tokens for this task. `limits.rs`: `fold_bytes` goes.
6. `temper.css`, under `/* ==== ui: chats ==== */`: the overlay's *The line* rules as they stand there (its image as `temper.css` names files, `url(files/doc.svg)`), the failed colour on `temper-work-failed`'s description and its row's out, and the agent page's own copies beside `.temper-agent-page details.temper-work`.
7. Tests, in `tests/chat_line.rs` and readers: `a_run_spans_turns_and_ends_at_a_text`, `read_docs_counts_topics_not_calls`, `one_call_alone_says_what_it_touched`, `a_tool_with_no_words_is_its_name_counted`, `a_card_and_a_write_are_not_counted_and_end_a_run`, `a_failed_call_is_said_on_the_line_and_its_row`, `a_run_of_one_turn_has_no_time`, `a_running_run_says_the_call_under_way_and_takes_the_dots`, `a_call_under_way_with_none_ended_is_a_line`, `a_row_never_holds_a_results_body`, `an_agents_page_folds_results_into_rows`, and a terminal print test. Each new rule bites.
8. Reprint and regenerate: `heavy cargo run -p temper-client-journeys`, the gallery's and terminal's goldens. Read each diff; it should hold only this line.
9. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-domain -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-gallery --all-targets -- -D warnings`; the focused tests of those, `temper-client-journeys` and `temper/tests/client`. Commit with the trailer, in small commits, and push.

## Report

As the implementer's text asks, with these for the account: each overlay rule of *The line*, landed or how it differs; each state of the message this task reaches and the test holding it; what the view lacks a fact for (a run of one turn's time; an agent's page's running state; any tool beyond the table), named as deferred to `chat-reading`.

## Rebase (2026-11-06, step merge, foreman look-round-6/call-line/foreman-1)

resolve README.status.md by preserving answer/reading gaps and removing only the call-line gap
The merge failed in `temper/docs/design/client/README.status.md`.
Integration commit `30cd77f708` adds two adjacent **Not yet** entries for the agent's words/While it works and Reading back/loading earlier messages.
Task commit `be939c434f` removes the following **Not yet** entry for *its calls*, *A call's line*, *The tool line*, and 5.9.
Rebase onto the landing target; retain both added entries and remove only the completed call-line entry. Preserve all remaining status text.
The log reports automatic merges for the other paths; it reports no test or gate failure requiring another change.
