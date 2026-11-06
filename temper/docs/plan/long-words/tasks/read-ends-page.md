+++
id = "read-ends-page"
part = "read-ends-page"
kind = "feature"
needs = ["read-ends", "line-prices"]
files = [
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/task.rs",
  "temper/tests/client/tests/suite/reading.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 40
+++

# A page says its read ended, and its other lines and acts go on

## Goal

With `read-ends` in, an undecodable page ends its read. Hold that on a chat's page and an agent's, through the real link: the read stops, the page says "Couldn't load earlier messages", and "Latest", the composer and the lines held still work. Say it too where a chat's first read failed, which says nothing today.

Acceptance:

- A chat's or an agent's page whose earlier page holds a line past the client's `long_text` says "Couldn't load earlier messages", and asks nothing more with no person's move: none in 120 s of the world's time. "Loading earlier messages…" goes.
- Its other lines stay, a message sent after it still goes, and a press of "Latest" on a chat sends its read at once, where it sent none within 3 s.
- A person's next move near the head asks again, once (`a_failed_head_read_says_so_and_the_next_near_retries` holds).
- A lost exchange is still asked again on activity's waits.
- `chat::older` says the failure when a chat's first read ended with no line to show: `read_failed` the foot's and the window holding no line. With lines held, a failed foot read says nothing, as 5.15 has it. An agent's page, which shares `older`, holds the same.
- The clause `5.15, *An answer the client cannot decode*` leaves `temper/docs/design/client/README.status.md`'s **Not yet**.

## Inputs

- `temper/docs/design/client/README.md`, 5.15, *While a read is out*.
- `jig/docs/design/client/domain/client.md`, section 16, *An answer that came and does not decode ends its read*.
- `temper/docs/plan/long-words/notes/contracts.md`, *Reads*: binding.
- `temper/docs/plan/long-words/notes/common.md`.

## Owns and may change

Owns the files in the header. `README.status.md` is shared: remove your line alone. May also change, these edits alone:

- `temper/crates/temper-client-domain/src/pages/chat.rs`, `pages/task.rs`, `pages/activity.rs`: only if a test here shows the domain doing other than `Asking::answered` and `Thread::failed` on a lasting end, the least that does, said in the report. By reading, they already do.
- `temper/tests/client/src/fixtures.rs`, `temper/tests/client/src/front.rs`: the means for a client whose `long_text` is below a line the world's store took: a limit on `reading_client`, no more.

Options, each decided:

- The case `(Edge::End, Some(End::Before), false)`, if `a_failed_read_says_so_at_the_head` gains it with lines held, stays false; the new row is `(Edge::End, Some(End::Before), no lines, true)`.
- Words: `Word::LoadingEarlierFailed`, as they stand. No new word.
- Don't reach for a raw undecodable body in the world: a line past the client's own bound is the request's case, and `read-ends`' tests hold the rest.

Not this task: `Asking`'s waits and `ReadEnd::transient`, which stay. A failed read at the foot with lines held, which says nothing: it stays as 5.15 has it, and your report says so, for temper/ui.

## Steps

1. Red first: the world's `a_line_past_what_the_client_takes_ends_its_read_and_says_so` and `an_agents_page_says_so_and_keeps_older`, in `reading.rs`. The view's `a_failed_first_read_says_so`. The domain's `an_ended_read_leaves_latest_free` in `tests/pages/chat.rs`, and `a_lasting_activity_end_is_not_asked_again_on_an_agents_page` in `tests/pages/task.rs`; if they pass red, the domain was already right: say so.
2. `older`.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-domain --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-view -p temper-client-domain`; the world's narrow tests. Commit and push.

## Report

As the implementer's text asks, with what each red showed.
