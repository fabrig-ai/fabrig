+++
id = "line-prices"
part = "view-prices"
kind = "feature"
needs = ["line-surface", "pending-words"]
files = [
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/limits.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
  "temper/tests/client/tests/suite/reading.rs",
  "temper/tests/client/src/fixtures.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 60
+++

# A long line draws whole, at the view's prices

## Goal

`line-surface` lets a line's words reach the page. Prove they draw whole on a chat's page and an agent's, in the browser's tree and the terminal's print, price what's still priced at `text`, and reproduce the request's run: 66 messages of 538 or 1,149 bytes read to their start.

Acceptance:

- `Limits::of` prices as `notes/contracts.md`, *The view's prices*, has it, and holds at the browser's, the terminal's and the tests' limits. `nodes` falls: a person's kept words no longer cost `lines × text`.
- A window of `Words` lines at `long_text` builds within the bounds, at both shells' limits, for plain words and for words that are nothing but newlines, a paragraph a line.
- Kept words totalling a thread's `bytes` build within the bounds. So do a request at `long_text` and a composer's value at `long_text`, on the one page.
- A line of 1,149 bytes and one of 10,660 draw whole, uncut, on a chat's page and an agent's: their whole words are the tree's text, and in the terminal's print. So does a heard or notified word on a tracked task's page.
- In `temper/tests/client`, a chat of 66 person's messages of 538 or 1,149 bytes is read to its start: every message seen whole, no "Couldn't load earlier messages". An agent's page reads the same through "Older".
- The clause `5.4, *A line of a conversation*` leaves `temper/docs/design/client/README.status.md`'s **Not yet**.

## Inputs

- `temper/docs/design/client/README.md`, 5.4, *A line of a conversation*.
- `jig/docs/design/client/domain/client.md`, section 10, *A window bounded by size* and *A person's words, in three states*.
- `temper/docs/plan/long-words/notes/contracts.md`, *Which bound holds what* and *The view's prices*: binding.
- `temper/docs/plan/long-words/notes/common.md`.

## Owns and may change

Owns the files in the header. `README.status.md` is shared: remove your line alone. May also change, these edits alone:

- `temper/crates/temper-client-view/src/pages/chat.rs`, `pages/chat_line.rs`, `pages/agent.rs`, `pages/task.rs`: only if a line is cut or priced wrongly there, the least that draws it whole, said in the report.
- `temper/crates/temper-client-shell-web/tests/suite/memory.rs`: a ceiling only if it moves, with its measurement before and after.

Options, each decided:

- `thread_limits`' `text` isn't yours: it stays until `composer-domain`. Price kept words by bytes now, whatever it says, since `pending-words` holds them to `bytes`.
- If `turns`' words would pass `u32` at a test's limits, the checked arithmetic returning `None` is the fault to fix: a client that won't start is never the fix. Report the figures.
- The world's default limits hold `long_text` at `CUE_BYTES`: take the browser's figures for this test, as `temper/tests/client/src/fixtures.rs` already does for `long_text` 131,072, and raise `people.words` in the same world if it stops a message below 1,149 bytes.
- 66 lines at the world's page size may take many reads: if the test passes `temper/tests/client`'s budget, keep the two lengths and cut the count to what fills past one window, and say so.

Not this task: what a failed read says, the composer.

## Steps

1. Red first: `a_window_full_of_long_words_builds`, `kept_words_at_the_threads_bytes_build`, `a_request_and_a_composer_at_long_text_build` in `tests/limits.rs`. The worlds' `sixty_six_long_messages_read_to_their_start` and `an_agents_page_reads_long_lines_through_older` in `reading.rs`. The `*_draws_whole` tests in `tests/chat.rs` and `tests/task.rs`, and the terminal's.
2. The prices.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-view -p temper-client-shell-terminal -p temper-client-shell-web`; the world's narrow tests. Commit and push.

## Report

As the implementer's text asks, with `Limits::of`'s `nodes` and `text_bytes` at the browser's limits, before and after, and any line the request's run showed that this doesn't.
