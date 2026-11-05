+++
id = "session-fake-strict"
part = "smith-review"
kind = "feature"
needs = []
files = ["smith/tests/session/src/translate.rs", "smith/tests/tools/src/translate.rs", "smith/tests/protocol-llm/tests/suite/world_fakes.rs"]
effort = "high"
minutes = 40
+++

# smith: the session and tools fakes panic on what no fixture sends

## Goal

`temper/docs/plan/debt-18/inputs/smith-debt14-review.md`, finding 2. A fake fitted to its rows must not quietly answer
for an input it doesn't model: it panics instead. Its render path, by contrast, is more faithful than any fixture asks.
The cut re-read `smith/tests/session/src/translate.rs` and the finding holds: `"edits":"x"`, `[]` and `[1]` decode to an
empty edit, `"all":1` to once, a nested object in an edit to a missing field, and the option fields are dropped.

Acceptance:

- `translate::decode` in `smith/tests/session/src/translate.rs` panics, with a message saying the finite fixture doesn't
  send it, on: `edits` that isn't an array of at least one object; `all` that is neither `true` nor `false`; an object
  nested inside an edit; a call with `offset`, `window`, `depth`, `glob`, `timeout_seconds` or `output_bytes`.
- `listed` in `smith/tests/tools/src/translate.rs` panics on a `.git` path and on output that isn't already in name order.
- `feedback_text` is gone: its two callers `extend_from_slice`. One `assert!` in the fake refuses a path holding a byte
  the protocol would escape (not UTF-8, or a control character other than `\n` and `\t`). The row
  `b"a\n\t\x01\x7f\xc2\x80\xff"` leaves `the_session_fake_renders_path_failures_like_the_protocol`.
- `the_session_fake_decodes_like_the_protocol` passes with its other rows untouched; every reader's tests pass.
- The branch's net lines fall by at least 10 (the review's estimate: -18, plus about 6).

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `temper/docs/plan/debt-18/inputs/smith-debt14-review.md`, finding 2
- `smith/docs/design/testing.md`, 2

## Owns and may change

- **Owns:** the three files in `files`, only these functions and that row.
- **Readers:** nothing else changes. The fakes' panics change what they mean, so their readers' tests run:
  `smith-session-world`, `smith-tools-world`, `smith-protocol-llm-world`, `smith-agent-world`, `smith-local-world`,
  `smith-run-world`, `smith-machine-world`.
- **If a panic fires on an existing fixture or row,** the review read that input wrongly: keep the fake's handling for
  that input, drop that one panic, and name the fixture in the report. Don't change a fixture to suit a panic.

## Steps

1. In `decode`'s `b"edit"` arm: walk `edits` as a non-empty array of objects, each `old`, `new` and optional boolean
   `all`; a check at the top of the tool match for the six option fields. About six lines: compact over thorough.

2. In `listed`: check the split paths before the loop (no path with a `.git` part, already sorted after `./` comes off),
   and keep the `BTreeMap`.

3. `feedback_text`'s two callers, the assert, and the row.

4. Make each panic fire once by hand (a row with `"edits":"x"`, `"all":1`, a nested object, an option field; an unsorted
   and a `.git` listing; a control byte in a path), see the message, restore. No `should_panic` test is wanted.

5. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-session-world -p smith-tools-world -p
   smith-protocol-llm-world --all-targets -- -D warnings`; `heavy cargo nextest run` with the seven packages above; the
   citations check. Commit and push.

## Report

The common note's items, each panic and what made it fire, and any you dropped because a fixture sends it.
