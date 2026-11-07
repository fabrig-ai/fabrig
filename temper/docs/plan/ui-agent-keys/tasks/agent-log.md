+++
id = "agent-log"
part = "log"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-view/src/pages/agent.rs", "temper/crates/temper-client-view/src/pages/chat.rs", "temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/tests/look.rs", "temper/crates/temper-client-view/src/tests/task.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/crates/temper-client-shell-web/task-pages.css", "temper/crates/temper-client-shell-web/tests/suite/switcher.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 60
+++

# A browser's agent page scrolls its log alone, its column inside it

## Goal

PgUp and PgDn act on the first log with something to scroll (`skein-ui-web/src/page.rs`, `key`). On an agent's page, a goal's untracked page among them, `.temper-thread` scrolls instead and the log has nothing to scroll, so the keys, "Latest" and the kept place do nothing. Make the log the one scroller in a browser, as on a chat's page, with the parts that relied on the thread's scrolling inside it. A terminal's tree keeps its order.

Why the parts move and not only the overflow rules: the brief, run heading, held card, outcome, result and plan are the thread's `flex: none` children, so with the overflow rules alone a long brief or result would run past the thread with nothing to scroll it, worst at 420 wide. A chat's log already holds its opening, lines and cards. The look stays as it is: the CSS below keeps each part's column, indent and gaps.

Acceptance:

- With `Shell::Browser`, the thread's children are the amendment's editor while it is open, then the log, then "Latest". The log holds, in order: brief, run heading, held card, the earlier read's status, the lines, and a new slot `look::AGENT_FOOT` holding the outcome, "Result" and "Plan".
- With `Shell::Terminal`, the parts stand around the log in today's order, the foot slot after the log.
- `task-pages.css` no longer gives the thread an overflow or the log `overflow: visible`; `temper.css` gives the agent's log `overflow-y: auto` and its thread `overflow: hidden`, as a chat's.
- The test lines in step 4 pass, and each bites.
- `skein/` is untouched. No browser, no render check, no new browser test (`temper/docs/plan/ui-agent-keys/notes/common.md`).

## Inputs

- `temper/docs/plan/ui-agent-keys/notes/common.md`.
- `temper/docs/design/client/README.md`, 5.9: *Its parts, in order* and *What scrolls*; 5.15, first bullet; section 11, *An agent's page opening at its end* (accepted: don't fix it here).
- Code: `temper-client-view/src/pages/agent.rs`, `main`; `pages/chat.rs`, `conversation_with_times` and `latest`; the chat's rules at `temper.css`, `#app > .temper-session .temper-thread`.

## Owns and may change

Owned: the files in this spec's header.

Readers you may change, only as said:

- `temper/crates/temper-client-view/src/tests/fixtures.rs` and `temper/tests/client/`: only under the options below.

Survey of who reads what moves, so you needn't repeat it:

- `chat::conversation` and `conversation_with_times` keep their signatures: `tests/fixtures.rs`, `tests/words.rs`, `tests/chat_line.rs` and the chat's page call them.
- Most view tests and the client world use `Shell::Terminal` (`tests/fixtures.rs` `page_view`, `temper/tests/client/src/fixtures.rs:193`), so their trees keep today's order, `tests/task.rs`'s `brief < run < log` and its log-text comparison with a chat among them.
- Browser-worded readers: `tests/task.rs`, `browser_page_view`; `temper-client-view/tests/suite/task_pages.rs`; `temper/tests/client/tests/suite/reading.rs`, `an_agents_page_reads_long_lines_through_older`; `temper/tools/gallery/src/tests.rs`, `agent_work_runs_its_line_and_reads_away_from_the_end` and `Holds`. They should pass unchanged.
- No script or view code reads `.temper-thread`; no journey holds an agent's page; the terminal's `agent_task.txt` golden comes from a hand-built fixture and doesn't change.

Options, each already approved:

- If a priced view bound trips on the foot's one node or the extra depth, raise that price where the page's bounds are computed (`temper/docs/design/client/README.md`, 5.4). If `task_pages.rs`'s fixed fixture envelope trips, never raise it: trim the failing case by one row, keeping what it checks. Say which in the report.
- If a Browser-worded `within(Log)` find now matches the brief's words, narrow the find to the line's words, keeping what it checks.
- `temper.css`: these edits stand where the old rules stand, not under a new marker block, since each replaces a rule in place. This departs from the request's marker rule on purpose.
- The rules `.temper-thread > button` and `> .temper-ended` stay as they stand: not this task.

## Steps

### 1. The view

In `pages/chat.rs`, split `conversation_with_times` without changing what it draws, and let `agent.rs` reach the key:

```rust
/// Opens the conversation's log, bound to read its edges (client/README.md, 5.15).
pub(crate) fn open_log(b: &mut Builder, thread: &Thread, words: &Words, bindings: &mut Bindings);

/// The log's lines, into a log its caller opened and closes.
pub(crate) fn conversation_lines(/* conversation_with_times's parameters */);

pub(crate) fn conversation_with_times(/* unchanged */) {
    open_log(b, thread, words, bindings);
    conversation_lines(/* ... */);
    b.close();
}
```

`open_log` takes the `b.open(Role::Log)`, name, bind and states lines; the `#[expect]` moves with the body. Make `fixed_key` `pub(super)`.

In `pages/agent.rs`, from the thread's opening:

```rust
b.open(Role::Slot);
b.style(look::THREAD);
super::task::amendment(b, root, words, bindings);
// A browser's log is the one scroller, holding the column (client/README.md, 5.9);
// a terminal prints the column around its log.
let scroller = words.shell == Shell::Browser;
if scroller { super::chat::open_log(b, &page.thread, words, bindings); }
brief(..); /* run heading */; /* controls unless paused */; super::chat::older(..);
if !scroller { super::chat::open_log(b, &page.thread, words, bindings); }
super::chat::conversation_lines(b, &page.thread, words, (rows, page.thread.limits.text), speakers, (drawn, None), bindings);
if !scroller { b.close(); }
b.open(Role::Slot);
b.style(look::AGENT_FOOT);
b.key(super::chat::fixed_key(56));
super::task::outcome(..); super::task::result(..); /* the plan, as it is */
b.close();
if scroller { b.close(); }
super::chat::latest(b, &page.thread, words, bindings);
b.close();
```

The foot's key keeps its identity beside the keyed lines as the status comes and goes; kind 56 is free. `latest` draws nothing on a terminal, so its move after the foot changes nothing there.

### 2. The look

Token 81 is the retired slot `temper-retired-task-danger`, with no constant. In `look.rs` rename that name to `b"temper-agent-foot"` and add, beside `AGENT_RESULT`:

```rust
/// What follows an agent task's lines: its outcome, result and plan.
pub const AGENT_FOOT: Style = Style { owner: owner(), token: 81 };
```

Add its row to `tests/look.rs`, `each_style_or_region_is_named_under_temper`.

### 3. The stylesheets

Literal, since no render check follows. Line numbers are at this task's base; find each by its selector.

`task-pages.css`:

| Line | Now | Becomes |
|---|---|---|
| 48-50 | the comment `One scrolling column...` and the two rules on `> .temper-thread` and `> .temper-thread > [role=log]` | removed |
| 51 | selector `.temper-agent-page .temper-thread > .temper-agent-brief` | `.temper-agent-page [role=log] > .temper-agent-brief` |
| 60 | selector `... .temper-thread > [role=log] ~ .temper-agent-result`, `width: min(1086.17px, calc(100% - 201.83px)); margin: 2.33px max(37.33px, calc((100% - 1213.33px) / 2)) 21px auto;` | selector `.temper-agent-page:has(> .temper-agent-lineage) .temper-agent-foot > .temper-agent-result`, `width: calc(100% - 127.17px); margin: 2.33px 0 21px auto;`, the rest as it is |
| 76 | same old selector, `width: calc(100% - 119px); margin-right: 18.67px;` | the new selector, `width: calc(100% - 81.67px);` |
| 80 | same old selector, `width: calc(100% - 37.33px);` | the new selector, `width: 100%;` |

`temper.css`:

| Line | Becomes |
|---|---|
| 223 | `.temper-agent-page .temper-thread { display: flex; flex-direction: column; flex: 1; min-height: 0; overflow: hidden; }` |
| 224 | `.temper-agent-page .temper-thread > [role=log] { position: relative; flex: 1; min-height: 0; width: 100%; padding: 21px max(37.33px, calc((100% - 1213.33px) / 2)) 35px; overflow-y: auto; overflow-x: hidden; scrollbar-width: thin; scrollbar-color: #d9d1c5 transparent; }` |
| 225 | removed |
| 226 | `.temper-agent-page .temper-thread > .temper-task-amend { flex: none; width: min(1213.33px, calc(100% - 74.67px)); max-height: 60%; margin: 21px auto 23.33px; overflow-y: auto; }` |
| 229-231 | comment `/* The parts after the lines, on the column of a message's words. */`, then `.temper-agent-page .temper-agent-foot:not(:empty) { padding-top: 35px; }` and `.temper-agent-page .temper-thread [role=log] > .temper-agent-foot > :not(.temper-agent-result) { padding-left: 127.17px; }` |
| 235 | `.temper-agent-page .temper-agent-foot > p, .temper-agent-page .temper-agent-foot > [role=status] { font-size: 12.83px; color: var(--temper-muted); }` |
| 333 | `.temper-agent-page .temper-thread > [role=log] { padding: 21px 18.67px 35px; }` |
| 334 | `.temper-agent-page .temper-thread > .temper-task-amend { width: calc(100% - 37.33px); }` |
| 343 | the foot's `:not(.temper-agent-result)` selector above, `{ padding-left: 81.67px; }` |
| 353 | the same selector, `{ padding-left: 0; }` |
| 1585 | removed |
| 1586 | the comment ends `...over the log's foot. */` |

Check by reasoning, not a browser: the log's padding already gives its content the old `min(1213.33px, 100% - 74.67px)` column, so the parts inside need no width. The earlier read's status now takes `.temper-thread-loading`'s own rule, which is what 3.5 asks: faint and centred.

### 4. The tests

Lines in existing tests; no new test.

`temper-client-shell-web/tests/suite/switcher.rs`, `scrolling_panes_contain_their_hidden_absolute_parts`:

```rust
const TASK_PAGES: &str = include_str!("../../task-pages.css");

for selector in [
    "#app > .temper-session .temper-thread > [role=log]",
    ".temper-agent-page .temper-thread > [role=log]",
    "#app > .temper-session > .temper-doc",
] {
    assert!(rule(selector).contains("position: relative;"), "{selector}");
    assert!(rule(selector).contains("overflow-y: auto;"), "{selector}");
}
// The log is what the page keys scroll; nothing around it scrolls the column (client/README.md, 5.9).
for thread in ["#app > .temper-session .temper-thread:has(> [role=log])", ".temper-agent-page .temper-thread"] {
    assert!(rule(thread).contains("overflow: hidden;"), "{thread}");
}
assert!(!TASK_PAGES.contains("overflow: visible"));
assert!(!TASK_PAGES.contains("> .temper-thread {"));
```

`temper-client-view/src/tests/task.rs`:

- `an_agents_page_has_latest_in_a_browser`: with `pill == log + size` already there, assert `log < brief && brief < run && run < foot && foot < pill`, by `styled` on `AGENT_BRIEF`, `AGENT_RUN` and `AGENT_FOOT`. Then press "Amend" (`press_button`) and assert the editor, `styled(.., look::TASK_AMEND)`, comes before the log.
- `an_agents_task_has_its_parts_in_order` (a terminal's): keep its `brief < run && run < log`, and assert the foot follows the log's subtree.
- `an_ended_agents_task_shows_its_result`: beside its terminal view, a `browser_page_view` of the same root, with "Result" inside the log's subtree.

Bite each: restore `task-pages.css` line 50, then line 49; drop `overflow-y: auto` from the agent's log; make `scroller` always false; make `scroller` always true for the terminal row. Report what each failure said.

### 5. The status

Remove the `**Not yet:** 5.9, *What scrolls*` entry from `temper/docs/design/client/README.status.md`, since this task closes it. The entry after it stays: no browser test scrolls this log.

### 6. Checks, commit, push

```
heavy cargo nextest run -p temper-client-shell-web --test suite -E 'test(scrolling_panes_contain_their_hidden_absolute_parts)'
heavy cargo nextest run -p temper-client-view --lib -E 'test(an_agents)'
```

Then once, per the implementer's text: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-shell-web --all-targets -- -D warnings`; the focused tests of `temper-client-view`, `temper-client-shell-web`, `temper-client-shell-terminal`, `temper-client-gallery` and `temper-client-world`, the last since it reads this page's log in a browser's words; `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

Beyond the implementer's text: which options you used; each bite and its failure; any rule you found relying on the thread's scrolling that this spec's table misses, and what you did with it; and that no browser ran, so the look and the keys wait for temper/qa's rewalk.
