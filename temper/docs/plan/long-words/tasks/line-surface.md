+++
id = "line-surface"
part = "line-words"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-client-protocol-wire/schema/client-v19.schema",
  "jig/crates/jig-client-protocol-wire/src/generated/**",
  "jig/crates/jig-client-protocol-wire/golden/**",
  "jig/crates/jig-client-protocol-link/src/limits.rs",
  "jig/crates/jig-client-protocol-link/src/translate/pages.rs",
  "jig/crates/jig-client-protocol-link/src/translate/pages/activity.rs",
  "jig/crates/jig-client-protocol-link/src/tests/pages.rs",
  "jig/crates/jig-client-protocol-link/src/tests/documents.rs",
  "jig/crates/jig-protocol-engine-clients/src/tests/pages.rs",
  "jig/tests/client-protocol/tests/suite/**",
  "jig/docs/design/client/domain/client.status.md",
]
effort = "high"
minutes = 45
+++

# A line's words are read to `long_text`

## Goal

A line holding words past a field's `text` fails its whole page today, so the chat never reads it. Make a line's words, and a `Message`'s and an answer's, `long_text` on the client, and let the engine encode any line a shipped profile's store took.

Acceptance:

- A page whose `Words` line, heard word or notified word holds `long_text` bytes decodes whole: `text` is well below it in the test's limits. One byte past `long_text` still fails the page. An ended object's question answer past `text` decodes too, as it shares `activity_line_fields`.
- `Message` and `AnswerQuestion` asks encode at `long_text` where a test's `text` is lower.
- `Word.words` and `ActivityWords.words` are `text 262144` in `client-v19`, regenerated, and the engine's `translate::activity` encodes a line of 262,144 bytes at its wire's ceilings.
- Through the real engine and native bytes, in `jig/tests/client-protocol`: a person's message past the client's `text`, within `long_text` and `people.words`, reads back whole in its task's activity.
- The clause `14: an activity line's words, a Message's and an answer's stop at text, and the wire's ceiling on a line's words is 65,536.` leaves `jig/docs/design/client/domain/client.status.md`'s **Not yet**.

## Inputs

- `jig/docs/design/client/domain/client.md`, section 14, `long_text`, and section 16, *Documents decoded whole*.
- `temper/docs/design/client/README.md`, 5.4, *A line of a conversation*.
- `temper/docs/plan/long-words/notes/contracts.md`, *Which bound holds what*, *The client wire*, *The link's bounds*: binding, its `Me` line aside, which is `me-words`'.
- `temper/docs/plan/long-words/notes/common.md`.

## Owns and may change

Owns the files in the header. May also change, these edits alone:

- `jig/crates/jig-client-protocol-wire/src/tests/fixtures.rs` and `jig/tests/client-wire/**`: a ceiling's literal, if a fixture or a fuzzy bound names one of the two members.
- `jig/testing/jig-client-kit/**` or `jig/testing/jig-client-protocol-kit/**`: a story's limits, if the world's story can't set `text` below `long_text` otherwise: a limit on `Story`'s constructor, no more.
- `jig/tests/client-protocol/tests/suite/main.rs`: the new test's `mod` line.

Options, each decided:

- If `receiving_inventory_bounds_keep_their_schema_ceilings` names a moved member, update its value to the new class's; add the four members to it if it holds the class's others.
- If a test pinned a line past `text` as refused, it now pins it past `long_text`, and your report names it.
- If the world's fixture engine sets `people.words` below your test's words, choose words within it: the point is past the client's `text`.

Not this task: `component.rs` and what a failed read ends as, `read-ends`'. `Me`, `me-words`'. Any temper file: the shells' constants don't move.

## Steps

1. Red first, in `tests/pages.rs`: `a_line_at_long_text_decodes_whole` (each of `Words`, `Heard`, `Notified`, at a limit with `text` 32 and `long_text` 128), `a_line_past_long_text_fails_its_page`, `an_ended_question_answer_past_text_decodes`, and `a_message_and_an_answer_encode_to_long_text`. Red shows the page refused at 33 bytes.
2. The schema's two numbers and the regeneration (`notes/common.md`).
3. `limits.rs`' classes, `translate/pages.rs`' `long_text`, `activity.rs`' two callers.
4. `jig-protocol-engine-clients`' `a_line_at_the_largest_profile_bound_encodes`, and the world's `a_message_past_text_reads_back_whole_through_the_engine`.
5. This changes what a shared bound means: besides the implementer's text's checks, run `heavy cargo nextest run -p jig-client -p jig-client-protocol-link -p jig-protocol-engine-clients -p jig-client-protocol-world -p temper-client-domain -p temper-client-tools`.
6. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-client-protocol-wire -p jig-client-protocol-link -p jig-protocol-engine-clients --all-targets -- -D warnings`. Commit and push.

## Report

As the implementer's text asks, with the regeneration's command, and the four wire and link bounds that moved, before and after.
