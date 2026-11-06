+++
id = "live-whole"
part = "live-text"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-client/src/thread.rs",
  "jig/crates/jig-client/src/thread/model.rs",
  "jig/crates/jig-client/src/tests/thread.rs",
  "jig/crates/jig-client-domain/src/text.rs",
  "jig/crates/jig-client-domain/src/tests/text.rs",
  "jig/crates/jig-client-protocol-link/src/limits.rs",
  "jig/crates/jig-client-protocol-link/src/translate/documents/runs.rs",
  "jig/crates/jig-client-protocol-link/src/tests/stream.rs",
  "jig/crates/jig-client-protocol-link/src/tests/documents.rs",
  "jig/docs/design/client/domain/client.status.md",
]
effort = "high"
minutes = 60
+++

# jig's client carries a turn being written whole, past a field's bound

## Goal

Three things stop a long live answer on its way through jig's client. The link refuses one delivery past a field's `text`. The domain, on a delivery the engine marked cut, drops what it held though nothing is missing: the engine marks every delivery once a turn passes its 65,536 retained bytes, so past that bound the page would collapse to the newest piece. And the thread keeps only a moving tail. This task clears all three. temper's value comes later (`turn-prices`), so every reader keeps its bound for now.

Acceptance:

- The link takes a run's delivery up to `long_text`, within the wire's `run_text_text` ceiling (65,536), and refuses one past it. Other receiving `text` bounds don't move.
- In `jig-client-domain`'s `merge`, a delivery marked cut that begins within or at the end of what's held extends it, keeping `cut` set. A delivery past what's held, marked cut, takes its place, as today. A replay while `Behind` still takes its place.
- `thread::Limits` has `live`. A live turn keeps its text's first `live` bytes on a UTF-8 boundary, so a line only grows, and stays as it is past the bound. `cut` says what it says today.
- `Thread::run` bounds a live line by `live`, and a person's words stay bounded by `text`.
- Every reader sets `live` to the `text` it passes today, so nothing they draw changes, but a line past the bound now shows its start.
- The **Not yet** line beginning `8.3, 10:` is gone from `jig/docs/design/client/domain/client.status.md`.

## Inputs

- `jig/docs/design/client/domain/client.md`, 8.3, section 10, *Live, then committed*, and section 14, on `long_text`.
- `temper/docs/design/draft/chat-reading.status.md`, *Design departs*, 3, *Whole*.
- `temper/docs/plan/chat-reading-a/notes/common.md`.
- `temper/docs/plan/chat-reading-a/notes/contracts.md`, `jig_client::thread::Limits`.

## Owns and may change

Owns `jig_client::thread`, `jig-client-domain`'s `text.rs`, the link's `receiving` run-text bound and `run_text_fields`. The engine's `jig-domain-views` doesn't change: `run.cut` and its 65,536 stand.

May also change, these edits alone:

- The field `live`, on each `thread::Limits { .. }` literal: `jig/crates/jig-client-view/src/state/ui.rs`, `jig/crates/jig-client-view/src/state/page/routing.rs`, `jig/tests/client-protocol/tests/suite/client.rs`, `temper/crates/temper-client-domain/src/pages/chat.rs` (`thread_limits`), `temper/crates/temper-client-tools/src/jig/runner/reads.rs`, and any test literal the compiler names.
- A test in `jig-client-view` or `jig-client` that pins a live line's moving tail: its expected value, to its start. Say which in your report.

`turn-order` adds one line to the link's `limits.rs`, in the `TurnLimits` literal: on a merge, keep it.

Options, each pre-approved:

- **The domain's own bound:** `merge` keeps its tail at `turn_bytes` (`text_past_the_turn_s_bound_is_cut_and_marked` stands). A text past `turn_bytes` then has `from > 0`, and the thread's line shows the start of what's held. Leave it.
- **A delivery past `long_text` where the wire's ceiling is lower:** the ceiling wins. A client's `long_text` past it stays valid (`narrow` keeps the class's rule), as `document_text_bytes` does.
- **`LiveTurn`'s own fields:** add or keep what you need inside, but `Line::Provisional` keeps its shape.

## Steps

1. **Red first:**

   ```rust
   // jig/crates/jig-client-domain/src/tests/text.rs
   #[test]
   fn a_delivery_marked_cut_that_continues_what_is_held_extends_it()
   #[test]
   fn a_delivery_marked_cut_past_what_is_held_takes_its_place()

   // jig/crates/jig-client/src/tests/thread.rs
   #[test]
   fn a_live_line_past_a_fields_text_arrives_whole_within_its_bound()
   #[test]
   fn a_live_line_past_its_bound_keeps_its_start_on_a_character_boundary()
   #[test]
   fn a_persons_words_stay_bounded_by_text_beside_a_larger_live_bound()

   // jig/crates/jig-client-protocol-link/src/tests/stream.rs
   #[test]
   fn a_delivery_past_a_fields_text_is_received_within_long_text()
   #[test]
   fn a_delivery_past_long_text_is_refused()
   ```

2. **The link:** in `receiving`, `wire.run_text_text` moves from `narrow(client.text, ..)` to a `narrow(client.long_text, ..)`. `run_text_fields`' `text` is bounded by `long_text`; `live_call_fields` and the rest keep `text`.
3. **The domain:** `merge`'s test for taking a delivery's place asks whether it begins past what's held, marked cut or not (`value.cut && value.from > held.through`), and a contiguous one appends.
4. **The thread:** `live`, and the start kept in `live_turn`.
5. **A change of meaning:** run `heavy cargo nextest run -p jig-client-domain -p jig-client -p jig-client-protocol-link -p jig-client-view -p jig-client-kit`, `-p temper-client-domain -p temper-client-view -p temper-client-tools`, and `-p jig-client-protocol-world`.
6. Fmt, clippy on the packages you changed, commit and push.

## Report

As the implementer's text says, and: the tests whose expected value moved.

## Answers

2026-11-06. **May this task change `explicit_cut_and_reopened_origin_replace_retry_text_while_valid_suffixes_deduplicate` in `jig/crates/jig-client-view/src/tests/state/domain/conversations.rs`, which has a continuing cut discard the held text?** Yes, by its input alone. `jig/docs/design/client/domain/client.md`, 8.3, settles the rule: a delivery that begins within what's held extends it, marked cut or not; one past it, marked cut, takes its place. The test's delivery at byte 3 overlaps what's held, so it no longer shows a lost origin. Give it one:

- Leave the name, the message and every expected value as they are: `deXYZ`, `deXYZ++` and `boot` still hold, and "cut never joins a retained prefix across a lost origin" is true of a delivery past what's held.
- The cut delivery becomes `from = 8`, `through = 13`, past the 6 held. The next delivery becomes `from = 10`, `through = 15`. `stale` becomes `from = 8`, `through = 13`. The texts and the rest of the test stay.
- Don't take the expected values to `abcdefYZ`. The overlap is `a_delivery_marked_cut_that_continues_what_is_held_extends_it`'s, and this test would stop showing the replacement it is named for.

Any other test in `jig-client-view` or `jig-client` that fails because a continuing cut discards held text gets the same repair: name it in your report. A failure of any other kind is outside this answer.

Then run step 5's commands with `--no-fail-fast`: the first run left 5,836 selected tests unobserved, and they are unobserved until a run reaches them.

## Rebase (2026-11-06, step merge, foreman chat-reading-a/live-whole/foreman-2)

Rebase by combining both status deletions and preserving the incoming chat tests with live bounds.
- The merge conflicts in `jig/docs/design/client/domain/client.status.md` and `temper/crates/temper-client-view/src/tests/chat.rs`.
- In the status file, remove both completed **Not yet** entries: this task’s `8.3, 10:` line and `turn-order` commit `a11600d5fe`’s `10, 14:` line. Preserve the remaining entries.
- In `chat.rs`, preserve commit `5e99713ce1`’s tests and behavior, including `conversation_turn` and the rewritten `a_note_turn_draws_no_cut_words`. Add `live: TEXT` to `conversation_turn`’s new `thread::Limits` literal; retain this branch’s other `live` additions.
- Update `a_long_provisional_text_says_nothing_of_what_was_cut`: both drawing paths must show the retained 31 ASCII bytes `1234567890123456789012345678901`, rather than `é and more`. The 32-byte live bound cannot include the following two-byte character. Keep the assertions that neither “Earlier text omitted” nor “…” appears. Report this changed expectation.
- Check the automatic merge in `jig-client-protocol-link/src/limits.rs`: retain `outcome: client.text` from `a11600d5fe` and this task’s `long_text` receiving bound.
- Preserve the incoming draft-note handling in `temper-client-domain/src/pages/chat.rs` alongside this task’s `thread_limits.live`.
- Run the spec’s step 5 package checks with `--no-fail-fast`, then the required formatting and clippy checks.
