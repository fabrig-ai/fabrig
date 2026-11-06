+++
id = "chat-reads"
part = "domain"
kind = "feature"
needs = ["thread-window"]
files = [
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/acts.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/pages/task.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/task.rs",
]
effort = "high"
minutes = 60
+++

# A chat's page reads both ways, and a person sent to the end gets there

## Goal

The domain's half of the user's journey: `Newer` is read, `Latest` and sending go to the end and read it again where the window let it go, and the window takes all of `document_bytes`. Nothing here draws: `chat-page` binds the log.

**Your base must hold `look-round-6`:** this plan stands on it, and `fabrig/ui` has priority. Check first: `rg -n 'State::ToLogEnd' temper/crates/temper-client-view/src/pages/chat.rs`. If that finds nothing, change nothing and report `Question: look-round-6 has not reached my base; wait for the catch-up?`, recommending yes.

Acceptance, in `temper-client-domain/src/tests/pages/chat.rs`, the lowest tier that shows each:

- **`newer` is read:** `Act::Newer` on a chat's page asks `thread.newer()` through its activity handle. So does an agent's task's (`Shape::Agent`), and on other shapes it does nothing, as `Older` does.
- **The end stays reachable,** the user's journey: enough `Act::Older` past `document_bytes` lets the end go, and `Act::Newer` read page after page brings it back to a window that reaches its end.
- **`to_end`:** `Act::Latest` and `Act::Send` each move it. Where `thread.lines.edges` doesn't reach `End::Before`, each also asks `thread.latest()`; where it does, no read leaves.
- **The window:** `thread_limits` sets `lines` to `document_bytes / LINE_ROOM`, saturating a `u32`, and a chat's window holds more than 16 short lines.
- **No `fill`:** `room`, `fill` and `offers_older` are gone. A first page with rows asks no further read. A page with no rows and an older cursor reads on at once, on either page's shape.
- A failed read leaves `thread.read_failed` set and the page's other state as it was, and a later `Older` reads again.

## Inputs

- `temper/docs/design/client/README.md`, 5.15.
- `jig/docs/design/client/domain/client.md`, section 10.
- `temper/docs/plan/chat-reading-c/notes/contracts.md`, *What didn't hold in the request*, *The thread's window* and *temper's domain*, binding.
- `temper/docs/plan/chat-reading-c/notes/common.md`.

## Owns and may change

Owns the files in the header.

May also change, these edits alone:

- An exhaustive match on `temper_client_domain::pages::Act`, in any file of the tree, for `Newer`'s forced arm: `temper-client-view/src/frame.rs` among them.
- `temper-client-view/src/pages/chat.rs`: `older`'s `offered` argument, as it loses `offers_older`: the caller passes `!thread.lines.edges.reaches(End::After)`, what an agent's page passes. "Older" stays drawn until `chat-page` removes it.
- A test anywhere that reads `offers_older` or pins `fill`'s read: `temper-client-view/src/tests/{chat,task,delegation}.rs`, `temper/tests/client/tests/suite/{task,chat}.rs`: for that reading alone. A world story that wanted `fill`'s second page now presses "Older", which stays.
- `temper-client-shell-terminal/src/client.rs`: the diagnostics' `Reading` arms, if any are forced.

Options, each decided:

- **`LINES`:** `pub const LINES` stays for `view-window` and `chat-page`, its readers, unless no one reads it on your base. The last reader's task deletes it.
- **`awaits_first_read`** stays as it is.
- If `Act::Send`'s refused or empty path can tell it sent nothing, `to_end` doesn't move there. A person who sends goes to the end; one whose send is refused stays.

**A change of meaning runs its readers' tests:** run `temper-client-view`, `temper-client-shell-terminal`, `temper-client-tools`, and `temper/tests/client`'s `chat`, `task` and `live_text` suites.

Not this task: `jig_client::thread`; any drawing; `Latest`'s binding.

## Steps

1. Red first: `newer_is_read_on_a_chats_page`, `the_end_comes_back_after_enough_older`, `latest_and_sending_go_to_the_end_and_read_it_where_it_went`, `a_chats_window_holds_more_than_sixteen_lines`, `an_empty_page_with_more_reads_on`, `no_second_read_after_a_first_page`.
2. The acts; `to_end`; `thread_limits`; `fill`'s remains.
3. Each test bites: break its rule once, see it fail for that reason, restore.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-view --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-tools`; `temper/tests/client`'s suites above. Commit and push.

## Report

As the implementer's text asks, with each world story that moved.
