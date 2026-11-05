+++
id = "stock-sentence"
part = "stock-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/stock/chat.txt",
  "temper/crates/temper-service/src/limits.rs",
  "temper/crates/temper-service/src/tests/limits.rs",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
]
effort = "medium"
minutes = 30
+++

# A goal writes its questions in the line form

## Goal

The stock chat's goal paragraph tells a goal to write a question in the line form. The sentence
spends charter 1's host part, so the figures the stock's tests hold follow it, and the stock brief
still fits whole.

Acceptance:

- The goal's paragraph of `temper/crates/temper-service/src/stock/chat.txt` ends with this sentence,
  after one space, and no other byte of the file changes:

  `Write a question to your requester as one line asking it, a blank line, what they need to know, then a line starting Saw: with what you saw, and a line starting Choice: for each option, the one you recommend ending (recommended).`

- That is 230 bytes with its space, so: in `src/tests/stock_brief.rs`,
  `every_stock_charter_admits_its_whole_maximal_brief` holds `(1, false, 38_030, 362_105)` and
  `(1, true, 35_940, 364_195)`, its other rows unmoved, and
  `the_stock_brief_with_a_whole_spec_fits_every_charter_whole` holds `room` 362,105, with `MARGIN`
  and `FRAMING` unmoved. `src/limits.rs`' `STANDARD_BRIEF.brief_bytes` is 360,045, and its comment
  says 362,105. `src/tests/limits.rs` expects 360,045 in both places. If a test measures a figure
  other than these, the measure stands: use it, and say so in the report.
- A new test in `src/tests/stock.rs` pins that the goal's paragraph holds the sentence and that the
  chat's paragraph and the advisor's don't.

## Inputs

- `temper/docs/plan/ui-needs/notes/common.md`.
- `temper/docs/design/domain/agent.md`, 8.2, the bullet *A question's line form*.
- `temper/docs/design/protocol/configuration.md`, 13.4, *Each charter's host families*. This plan's
  cut already wrote the design's figure (360,045, in `configuration.md`, section 11, and its status).

## Owns and may change

Owns the five files in `files`. May also change `temper/docs/design/domain/agent.status.md`, to take
"the stock chat's sentence" out of its 8.2 entry and leave the parser's.

Readers, surveyed:

- `temper/crates/temper-engine-smith/src/tests/brief_fit.rs` makes its own charter and holds no stock
  figure: change nothing unless it goes red.
- `temper/tests/engine/domain/tests/suite/settings/b2.rs` reads `chat.txt` by `include_bytes!`: no
  edit.
- `temper/crates/temper/tests/fixtures/cluster/settings.json` is a cluster's own settings, with its
  own copy of the words, compared with nothing: leave it. If a test does compare it, regenerate that
  line and say so.
- `journal.result_bytes` is `STANDARD_BRIEF.brief_bytes` by construction.

## Steps

1. Edit `chat.txt`, the figures and the comment.
2. Add the test; check it bites by taking the sentence out once.
3. **A change of meaning runs its readers' tests:** the brief's bound moved, so run `temper-service`'s
   `stock` and `limits` tests, `heavy cargo nextest run -p temper-engine-smith --lib -E
   'test(brief_fit)'`, and `heavy cargo nextest run -p temper-engine-domain-world -E
   'test(settings)'`.
4. fmt, clippy and the checks of the implementer's text. Commit and push.

## Report

As the implementer's text asks, with the figures as measured, and whether the stock brief fits whole.
