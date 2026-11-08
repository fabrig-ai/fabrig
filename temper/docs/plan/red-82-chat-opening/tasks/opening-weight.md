+++
id = "opening-weight"
part = "chat-opening"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-domain/src/pages/chat.rs", "temper/crates/temper-client-domain/src/tests/pages/chat.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 50
+++

## Goal

A chat's opening reads follow the older cursor until the history's end
**or** until a screen and a half is held, whichever comes first, the
window's bounds and every other stop of `1e6e2388ee` still applying
(`temper/docs/design/client/README.md`, 5.15, *Opening history*). The
domain measures no screen, so a screen and a half is a weight: the held
lines weighing 2,880 bytes or more by jig's `row.weight()`. A short chat
still reaches its opening question with no scroll (finding-465); a long
one stops with its older cursor kept, so a browser's reading back loads
earlier pages and says "Loading earlier messages" again (red-82).

Acceptance:

- `temper-client-domain`'s `pages/chat.rs` holds the weight as a named
  constant, and `Chat::accept_activity` stops the opening's reads once the
  lines held after a page weigh it or more. Nothing else of the opening
  changes: the history's end, the window's row and byte bounds, a lasting
  failure, a cursor that does not advance, the empty pages' read budget,
  the announced tail's read and the transient retry all hold as now.
- The domain's focused tests hold the rule in one table test (below): a
  short chat whose reply ended fast reaches its opening; a first page one
  byte under the weight reads on; a long chat whose first page weighs the
  weight stops, its older cursor kept, and the person's `Act::Older` reads
  from that cursor.
- Every test `1e6e2388ee` added or changed passes, in the domain, the TUI
  and the joint client world (Steps, 4).
- The **Not yet** entry 5.15, *Opening history*, in
  `temper/docs/design/client/README.status.md` ("initial reads don't stop
  once a screen and a half is held …") is removed, its lines alone. This
  departs from the implementer's text, which closes `docs/design/` to
  you: the entry is this task's.

## Inputs

- `temper/docs/design/client/README.md`, 5.15: the bullets *The log is
  bound* and *Opening history*, the rule this task builds.
- jig's `jig/docs/design/client/domain/client.md`, section 10: a line's
  weight (`LINE_ROOM` for it and each message, block, object and part,
  with its bytes), what `row.weight()` returns.
- `git show 1e6e2388ee -- temper/crates/temper-client-domain`: the
  opening's reads as they stand, and the tests that pin their other stops.
- `temper/docs/plan/red-82-chat-opening/inputs/request.md`.
- `temper/docs/plan/red-82-chat-opening/notes/common.md`.

## Owns and may change

- Owns the files in `files`.
- Reader files, assertion edits only: `temper/tests/client/tests/suite/reading.rs`
  and `temper/tests/client/tests/suite/tui.rs`. If a test there asserts
  that a chat opens with more than the weight now holds, change that
  assertion to the rule (the opening holds at least the weight, or the
  whole history) and name it in the report; its other steps stay.
- Pre-approved: if a domain test from `1e6e2388ee` or before fails
  because its pages now weigh past the constant, keep the rule it pins:
  lighten its pages, or have the person's `Act::Older` read on where its
  opening read did; name which in the report. Never loosen what it
  asserts of its own rule.
- Not yours: `temper/tests/client-browser/**` (the browser story and its
  fixture), `temper/docs/help/**` (`tui.md`'s "A short chat reads back to
  its opening automatically" stays true), `README.md` itself, any other
  crate.

## Steps

1. **The constant,** in `pages/chat.rs`, beside `thread_limits`:

   ```rust
   /// A screen and a half as a chat's opening reads hold it: the held lines'
   /// weight, a screen and a half of an 80 by 24 terminal
   /// (temper/docs/design/client/README.md, 5.15).
   pub(crate) const OPENING_WEIGHT: u64 = 2_880;
   ```

2. **The stop,** in `Chat::accept_activity`. Its tail already sums the
   held lines' `weight` after `self.thread.page(page)`. The opening's
   follow (`self.initial_pages > 0`) reads on only while that weight is
   under `OPENING_WEIGHT`; the arm for an empty page with an older cursor
   outside the opening (`!filling && empty`) keeps its rule. A page that
   meets the weight is nonempty, so the existing tail sets
   `initial_pages` to 0 and the read ends with `thread.reading` none and
   the window's edges as the page left them: the older cursor stays for
   `Thread::older`. The shape inside is yours; the stop is one condition
   on the weight already computed, not a second pass over the rows.

3. **The table test,** in `src/tests/pages/chat.rs`: turn
   `a_fast_completed_chat_reads_back_to_its_opening` into
   `a_chat_opens_to_its_opening_or_a_screen_and_a_half`, one loop over
   rows, each opening a chat as that test does (`f::opened`, the task
   delivered, `Told::Changed` on the document) and answering its first
   read with a completed page (`completed_activity`, given a text or a
   padding row so its weight can be set) whose `before` is a cursor:

   | row | the first page's held weight | then |
   |---|---|---|
   | short | the fast reply as today | reads `before: Some(cursor)`; the opening page (`before: None`) reaches `End::After`, holds the request's words, and nothing more is read |
   | under | `OPENING_WEIGHT - 1` | reads `before: Some(cursor)` |
   | long | `OPENING_WEIGHT` | nothing is read (`out` empty), `thread.reading` is none, `thread.lines.edges` are the first page's, not at `End::After`; `chat_act(.., Act::Older, ..)` then asks `Read::Activity { task: f::TASK, before: Some(cursor) }` |

   The test sums `row.weight()` over the page it built and asserts it is
   the row's target, so a fixture change can't move a row off its
   boundary. Break the rule once by hand (drop the weight condition; then
   make it `<=`), see `long` and then `under` fail for that reason,
   restore, and name both in the report.

4. **Narrow runs, then the readers.** This task changes what a chat's
   opening reads, not a signature, so it runs the tests of the packages
   that read that, and narrows the implementer's text's "direct
   dependents" run to them:
   - `heavy cargo nextest run -p temper-client-domain --lib -E 'test(/pages::chat::/)'`
   - `heavy cargo nextest run -p temper-client-tui --lib -E 'test(/chat/)'`
   - `heavy cargo nextest run -p temper-client-shell-tui --lib`
   - `heavy cargo nextest run -p temper-client-world --test suite -E 'test(/^reading::/) | test(/^tui::/) | test(/^open_chat::/) | test(/^chat::/)'`

   `tui::a_tuis_fast_completed_chats_keep_the_opening_said_row` and
   `tui::a_tuis_launchpad_asks_temper_and_draws_the_chat` must pass:
   their chats are short. Check each run's count; one that selected
   nothing proves nothing.

5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-domain --all-targets -- -D warnings`; commit
   saying why (the opening's reads keep the screen-and-a-half rule, so a
   long chat reads back with "Loading earlier messages" again); push.

## Report

As the implementer's text has it, and: the rows' weights and how the
test sets them; the rule broken for each bite and the failure seen; each
reader test you edited or each 1e6e2388ee test you reshaped, and how;
the counts of the runs in step 4. The browser story is not run here: say
so; after landing runs it.
