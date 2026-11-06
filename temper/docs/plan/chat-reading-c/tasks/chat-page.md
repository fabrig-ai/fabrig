+++
id = "chat-page"
part = "page"
kind = "feature"
needs = ["chat-reads", "view-window", "text-reading", "log-surface"]
files = [
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/chat_line.rs",
  "temper/crates/temper-client-view/src/pages/agent.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
  "temper/crates/temper-client-view/src/tests/delegation.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
]
effort = "high"
minutes = 90
+++

# The conversation's log reads by itself, and "Older" goes

## Goal

Bind the log, say when it continues and when it's busy, key its lines, and take away "Older". After this a chat in a browser is read by scrolling, and in the terminal by `up` and `down`.

**Your base must hold `look-round-6`:** this plan stands on it, and `fabrig/ui` has priority. Check first: `rg -n 'State::ToLogEnd' temper/crates/temper-client-view/src/pages/chat.rs`. If that finds nothing, change nothing and report `Question: look-round-6 has not reached my base; wait for the catch-up?`, recommending yes.

Acceptance, in `temper-client-view/src/tests/chat.rs` and `task.rs`:

- **No "Older":** no page holds a button named "Older", `Word::Older` is gone, and so is `pub const LINES` with its last reader. `Act::Older` stays, reached only by the log's act.
- **Bound:** the log is bound while `edges.after` isn't `End` or `edges.before` isn't `End`, and unbound at both ends. `Near` at its head decodes to `Act::Older`, at its foot to `Act::Newer`, on a chat's page and an agent's.
- **Continues:** `State::Continues` while `edges.before` isn't `End`, on both pages.
- **Busy:** `State::Busy` on the log while `thread.reading` is `Some` or `read_failed` is `Some`.
- **The end request:** the chat's log has `end_request(chat.to_end())`; an agent's has none.
- **Keys:** every direct child of the conversation's column is keyed (`line_key`) or stands before the first line, a line's key is the same for a provisional turn and its committed one, and a run of calls takes its first turn's. Reading an earlier page into a built view patches with inserts alone: no `Patch::Text` or `Patch::Remove` on a line already drawn. Reading past the bound removes the lines let go, and nothing else. This test bites: take the key off, and see it fail.
- **At the head:** the status while `Reading::Older` is out, as `look-round-6` has it. After a failed read of that end, `Word::LoadingEarlierFailed` in the same status and the same look. Nothing once `edges.after` is `End`, and nothing for a read at the foot.
- **"Latest"** stays as `look-round-6` has it, bound where `edges.before` isn't `End`, and `pages/chat.rs`'s doc words say what it does now.
- **In the terminal,** a chat's print holds `(up, down: more lines)` under the log's name while it's bound, and no "Older". The terminal's prints and fixtures (`temper-client-shell-terminal/src/tests/**`) are regenerated, and read: the report says what was read.

## Inputs

- `temper/docs/design/client/README.md`, 5.15, and 5.9.
- `skein/docs/design/ui.md`, 3.2 and section 6.
- `temper/docs/plan/chat-reading-c/notes/contracts.md`, *temper's view*, binding, and *A log's busy state*.
- `temper/docs/plan/chat-reading-c/notes/common.md`.

## Owns and may change

Owns the files in the header; in `temper-client-domain/src/pages/chat.rs`, `LINES`'s line alone.

May also change, these edits alone:

- `temper/crates/temper-client-view/src/tests/{look,cards,fixtures}.rs`, `temper/tools/gallery/src/**` where a state read "Older": for that alone. `temper/tools/gallery/src/tests.rs`'s prints regenerate.
- `temper/crates/temper-client-view/src/pages/chats.rs`: only if a forced arm needs it.
- `temper/tests/client/tests/suite/**`, a story that pressed "Older": it now acts `Near` on the log, through the world's person or `world.act`, whichever exists; `reading-worlds` writes the new stories.
- A word's new id takes an unused one above the occupied range (`temper-client-view/src/words.rs`), and `tests::accounts::english_phrase_declarations_match_the_vocabulary` regenerates with `SKEIN_CODEGEN_WRITE=1` through heavy, as `look-round-6`'s `reading` did. `temper.css` isn't touched: the failed status takes `THREAD_LOADING`'s rule.

Options, each decided:

- **`older`'s home:** `look-round-6` left `older` in `pages/chat.rs`, called from `agent.rs` too. It keeps its name, loses `offered`, and draws the status alone.
- **A request's first message and a day's line** have no key of their own unless a test shows them moved: they stand before the first line, or take their line's key with a kind apart.
- If a keyed child can't be told from its neighbours (two runs with no first turn), the next line's key with a kind apart.
- **`b.end_request`'s value** before a first send is 0: `Some(0)` makes no apply move, as a count that hasn't moved.

**A change of meaning runs its readers' tests:** run `temper-client-domain`, `temper-client-shell-terminal`, `temper-client-shell-web`, `temper-client-gallery`, and `temper/tests/client`'s `chat`, `task`, `live_text` and `delegation` suites.

Not this task: `temper.css`, skein, jig, new world stories, the browser story, the journeys' prints, design text.

## Steps

1. Red first: `no_page_offers_older`, `the_log_is_bound_while_it_can_read_on`, `near_decodes_to_older_and_newer`, `a_log_continues_while_the_window_let_its_end_go`, `a_log_is_busy_while_a_read_is_out_or_failed`, `sending_moves_the_logs_end_request`, `an_earlier_page_only_inserts`, `a_provisional_turn_and_its_committed_one_share_a_key`, `a_failed_read_says_so_at_the_head`.
2. `frame.rs`; `conversation_with_times`; `line_key` through the walk and `chat_line.rs`; `older`; the words; `agent.rs`.
3. `LINES` goes. Regenerate the terminal's prints, and read them.
4. Each test bites: break its rule once, see it fail for that reason, restore.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-domain -p temper-client-shell-terminal -p temper-client-gallery --all-targets -- -D warnings`; the focused tests of those and the suites above. Commit and push.

## Report

As the implementer's text asks, with what was read of the terminal's prints, and each story that moved.
