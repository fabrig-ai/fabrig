+++
id = "agent-line"
part = "agent-page"
kind = "feature"
needs = ["agent-call", "run-time"]
files = [
  "temper/crates/temper-client-view/src/pages/agent.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/lib.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 70
+++

# An agent's page runs the line, and has "Latest"

## Goal

An agent's task draws its calls as a chat does, but its line never says what is under way, and a long conversation has no way back to its end. Both are a chat's already: this task gives them to an agent's page, with no three dots, as its run's heading says it is live.

Acceptance, in `temper-client-view/src/tests/task.rs`:

- **While a run is out:** with `task::run` at `Run::Live` and no words being written, the conversation's last run has `look::WORK_RUNNING` and no time, and its description is the call under way, from `Task::current_call`: "checking template 2".
- **None ended:** a call under way with no run yet is the line alone, "Checking template 2", one row.
- **Words being written:** with a non-empty provisional line last, the run before it is settled and has its time, as on a chat's page.
- **No run out:** the last run is settled, with its time, and `current_call` isn't read.
- **No three dots:** no node has `look::TYPING` on an agent's page, running or not.
- **"Latest":** a button named "Latest", `look::LATEST`, `State::ToLogEnd`, right after the log, in a browser's words alone; bound to `Act::Latest` where the window doesn't reach its end, and unbound where it does. No such button in the terminal's words.
- **A chat's page is unchanged:** its tests pass as they are, `tests/chat.rs` and `tests/chat_line.rs` untouched by this task.
- **The look:** in `temper.css`, under `.temper-agent-page`, the pill stands over the log's foot while `[data-log-away]` is on the log, and is hidden otherwise. The rules at `.temper-agent-page .temper-thread > :not([role=log]):not(.temper-cards)` and `[role=log] ~ :not(.temper-cards):not(.temper-ended)` don't move or pad it. The running line's "…" has its rule already.
- **The gallery:** a state `agent-work`, per `temper/docs/design/client/browser.md`, 7.2: an agent's page, its last run running with a call under way, and "Latest" showing. At 1440, 1000 and 420 wide, in throwaway renders, the pill and the line read as `chat-work-reading`'s and `chat-work-running`'s do. The report says what differs and why.
- **The status:** the **Not yet** line "5.9, *The conversation*" is gone from `temper/docs/design/client/README.status.md`, and "7.2, `agent-work`" from `temper/docs/design/client/browser.status.md`.

## Inputs

- `temper/docs/design/client/README.md`, 5.9, *The conversation*; 3.5, *its calls*, *while it runs*.
- `temper/docs/design/client/browser.md`, 7.2.
- `temper/docs/plan/chat-reading-b/notes/common.md`.

## Owns and may change

Owns the files in the header, with small hunks: `chat-reading-c`'s `chat-page` is rewriting `agent.rs` and `pages/chat.rs`, and `reading-browser` `temper.css`.

```rust
// pages/chat.rs
pub(crate) enum Drawn<'a> {
    // ...as today, and:
    /// An agent's page while a run is out (client/README.md, 5.9).
    AgentRun { current: Option<&'a c::CurrentCall> },
}
// `Drawing` gains what says so; `conversation_with_times` computes `running` and
// `current` from it where it does from `proposals`.
pub(crate) fn latest(b: &mut Builder, thread: &Thread, words: &Words, bindings: &mut Bindings)

// pages/chats.rs: the tail of `working`, for both pages.
pub(crate) fn words_not_begun(thread: &jig_client::thread::Thread) -> bool

// pages/agent.rs: `Drawn::AgentRun` while `task::run(task)` is `Run::Live`, else `Drawn::Lines`;
// `super::chat::latest` right after the conversation.
```

Options, each decided:

- **Gallery:** `agent-work` is a sample at `Address::Task`, its chip's executor an agent's as at `worlds.rs`, near 193, its log moved as `lib.rs`, near 146, moves `chat-work-reading`'s. If its world needs more than about 60 lines, reuse `chat_work`'s turns, and say so.
- **`Drawing`'s field:** yours. The dots' branch stays under `proposals`, so an agent's page can't reach it.
- **If `chat-page` has reached your base,** `older` and `Drawn` may have moved: build on what's there, and say so in your report.

**A change of meaning runs its readers' tests:** run `temper-client-view`, `temper-client-domain`, `temper-client-gallery`, `temper-client-shell-terminal`, and `temper-client-world`'s `task_pages` suite.

Not this task: the domain, skein, design text but the two status lines, the journeys' prints.

## Steps

1. Red first: `an_agents_run_says_the_call_under_way`, `an_agents_call_with_none_ended_is_a_line`, `an_agents_written_words_settle_its_run`, `no_run_out_settles_an_agents_line`, `an_agents_page_has_no_dots`, `an_agents_page_has_latest_in_a_browser`, `latest_binds_where_the_window_let_its_end_go`, `an_agents_terminal_has_no_latest`.
2. `words_not_begun`; `Drawn::AgentRun`; `latest` on a thread; `agent.rs`.
3. `temper.css`; the gallery state; the throwaway renders, gone before the push.
4. Each new test bites: break its rule once, see it fail for that reason, restore.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal --all-targets -- -D warnings`; the focused tests of those. Commit and push.

## Report

As the implementer's text asks, with the renders' differences.

## Rebase (2026-11-06, step merge, foreman chat-reading-b/agent-line/foreman-1)

Rebase and combine the gallery sample counts, preserving both sides’ samples.
The merge conflicts in `temper/tools/gallery/src/samples.rs` at `pub const N`.
Task commit `d08531040c` raises N from 269 to 270 for `agent-work`.
Main’s gallery changes, incorporated through catch-up commit `b05474c92b`, raise N to 274.
Resolve N to 275 for the combined table.
Keep main’s five added samples and their domain setup, plus this task’s `agent-work` sample and its `chat_work_world` dispatch.
Preserve the automatically merged gallery, CSS and test changes.
Verify with the gallery tests `names_are_the_promised_ones` and `agent_work_runs_its_line_and_reads_away_from_the_end` through heavy, then complete the spec’s required checks.
