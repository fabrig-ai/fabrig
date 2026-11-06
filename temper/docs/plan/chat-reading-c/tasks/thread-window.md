+++
id = "thread-window"
part = "thread"
kind = "feature"
needs = ["beside-surface"]
files = [
  "jig/crates/jig-client/src/thread.rs",
  "jig/crates/jig-client/src/thread/**",
  "jig/crates/jig-client/src/tests/thread.rs",
  "jig/crates/jig-client/src/tests/fixtures/thread.rs",
  "jig/crates/jig-client-domain/src/contract/documents.rs",
  "jig/crates/jig-client-domain/src/tests/**",
  "jig/docs/design/client/domain/client.status.md",
]
effort = "high"
minutes = 90
+++

# A thread's window is bounded by size, and the lines it lets go come back

## Goal

Fix the user's journey at its root: a window lets go whole lines past its bytes, knows it let them go, and reads them again beside the line next to them, at either end. `Thread::newer` reads something at last. `Line::Left` goes. Failing tests first, here in `jig-client`, the lowest tier that shows it.

Acceptance:

- **Size:** a window of pages whose lines' weights sum past `bytes` holds at most `bytes` of them, whatever `lines` says when `lines` is `bytes / LINE_ROOM`. `DecodedActivityLine::weight` is what `client.md`, section 10 says, and a line of many small blocks weighs `LINE_ROOM` for each.
- **Either end:** reading older past the bound lets the newest whole lines go, and `edges.before` is `Edge::Dropped`; reading newer past it lets the oldest whole lines go, and `edges.after` is `Edge::Dropped`. Never all of the page just read: a page alone past `bytes` stays.
- **The `lines` bound** drops by the same rule and sets `Edge::Dropped` too, so a reference UI's short window behaves the same way.
- **Back:** `older` on `Edge::Dropped` reads `ActivityBeside { position: oldest held, side: Earlier }`, and `newer` on `Edge::Dropped` or `Edge::More(_)` reads `ActivityBeside { position: newest held, side: Later }`. A held line isn't repeated. A later page with no `after` makes `edges.before` `Edge::End`, and `moved` then reads any new tail. `loaded` holds the newest position read through `newer`.
- **The journey, at this tier:** page after page of `older` past the bound lets the end go; `newer` page after page brings every line back, in order, whole, none twice, to `Edge::End`.
- **`latest`** reads the list's end, and its page replaces the window, whatever it held, its edges the page's.
- **`moved`** reads no new tail while the window has let its newest go, and it still advances `committed`.
- **`lines()`** holds no `Words` or `Provisional` line while `edges.before` isn't `Edge::End`, and they come back when it is. Their state is kept: a word sent in the gap is still there, and a live turn still grows.
- **`read_failed`:** `failed()` records `End::After` for an older read, `End::Before` for a newer or latest one; the next read of that end clears it, and so does its page.
- `Line::Left` and `Thread.left` are gone. `Thread`'s module doc table and `client/domain/client.md`'s words agree.
- The clause beginning `10, A window bounded by size` is gone from `jig/docs/design/client/domain/client.status.md`.

## Inputs

- `jig/docs/design/client/domain/client.md`, section 7, and section 10, *A window bounded by size*, *The newest are never lost*, *A read that failed*.
- `jig/docs/design/client/domain/contract.md`, section 6, *A page beside a line*.
- `temper/docs/plan/chat-reading-c/notes/contracts.md`, *What didn't hold in the request*, *The activity read*, *The thread's window*, binding.
- `temper/docs/plan/chat-reading-c/notes/common.md`.

## Owns and may change

Owns the files in the header.

Readers and neighbours, these edits alone:

- **`Edge::Dropped`, forced arm:** `jig/crates/jig-client-view/src/state/windows.rs`, `state/page/conversations.rs`, `state/page/task.rs`: their windows never hold it, so an arm with `Edge::End`'s meaning. `temper/crates/temper-client-domain/src/pages/chat.rs`, `offers_older`: `Dropped` as `More(_)`.
- **`Line::Left` removed:** `temper/crates/temper-client-tools/src/jig/render/activity.rs`, `temper/crates/temper-client-view/src/pages/{chat,chats,chat_sheet}.rs`: the arm goes, and on `chat.rs` whatever it drew. If `look-round-6` hasn't landed, `Word::LinesLeft` stays, unread: it is that plan's to remove. A test that reads that line (`temper/crates/temper-client-view/src/tests/{chat,task,delegation}.rs`): it now finds nothing, or goes.
- **The tests' fixtures in `jig/crates/jig-client-view/src/tests/`, `jig/tests/client-protocol`:** a test pinning `Line::Left`, or an old `newer` on a `More(cursor)` read, for its expected value.
- **Exhaustive matches on `Reading` or `Read`,** none new but what the compiler forces.

Options, each decided:

- **`TurnWindow::slide` and `plan` stay,** `jig-client-view`'s windows share them. The fit by bytes is a function beside them: `TurnWindow::fit(&mut self, bytes: u64, end: End) -> u32`, which keeps rows read at `end`, or the thread's own loop, whichever reads better. The window's weight may be summed at each page: pages are rare.
- **`fresh`,** the staging of a new tail, stays by rows. If its pages pass `bytes` before meeting `known`, it stops as a page that doesn't fit does, and the window is replaced.
- **`weight`:** `Text`, `Bytes` and `Box<[u8]>` by length; numbers and flags count in the room. Keep it one walk of `ActivityLine`, `ArchivedTurn` and `TurnRead`.
- **The tests' `lines: 3`:** the tests of `jig/crates/jig-client/src/tests/thread.rs` keep their small `lines` where they pin paging; a new test of size sets `lines` to `bytes / LINE_ROOM`.
- **Where a temper test pins the 16 lines** (`temper-client-view`'s `older` tests, `temper/tests/client`): if `offers_older`'s rule still holds, it stays. Anything that can't hold until `chat-reads` and `chat-page` goes under `## Left`, named.

**A change of meaning runs its readers' tests:** `Edge`, `Thread::newer` and `lines()` mean something new. Run `jig-client-view`, `temper-client-domain`, `temper-client-view` and `temper-client-tools`.

Not this task: `pages::chat::LINES`, `thread_limits`, `fill`, any new act or drawing: `chat-reads` and `chat-page`. The store's forward walk (`beside-store`) needn't be there: this tier's pages are scripted.

## Steps

1. Red first, in `jig/crates/jig-client/src/tests/thread.rs`: `a_window_lets_whole_lines_go_past_its_bytes`, `reading_older_lets_the_newest_go_and_reading_newer_the_oldest`, `the_end_comes_back_after_enough_older`, `a_page_alone_past_the_bound_stays`, `older_past_a_dropped_edge_reads_beside_the_oldest_line`, `latest_replaces_the_window`, `a_window_that_let_its_end_go_reads_no_tail_and_draws_nothing_live`, `read_failed_says_which_end_until_it_is_read`, `a_line_weighs_its_bytes_and_its_room`.
2. `LINE_ROOM` and `weight`; `Edge::Dropped`; the thread.
3. The arms; `Line::Left`'s readers.
4. Each test bites: break its rule once, see it fail for that reason, restore.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and `temper/scripts/old_forms.py`; `heavy cargo clippy -p jig-client -p jig-client-domain -p jig-client-view --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-client -p jig-client-domain -p jig-client-view -p temper-client-domain -p temper-client-view -p temper-client-tools`, and `heavy cargo nextest run -p jig-client-protocol-world`. Commit and push.

## Report

As the implementer's text asks, with the weight of a typical turn against its text's bytes, and every test moved to `## Left`.
