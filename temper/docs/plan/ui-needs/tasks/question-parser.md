+++
id = "question-parser"
part = "question-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/question.rs",
  "temper/crates/temper-client-domain/src/tests/question.rs",
]
effort = "high"
minutes = 25
+++

# A question's words, parsed

## Goal

A pure function in `temper-client-domain` that reads a question's words in the line form, for Home
and Needs you to draw (later tasks). Nothing reads it yet.

Acceptance:

- `question::parse` and `question::Parsed::primary` exist as
  `temper/docs/plan/ui-needs/notes/contracts.md` has them, and hold every rule of the design's
  *A question's line form*.
- Their step tests pass, each named in the report, and each bites.

## Inputs

- `temper/docs/plan/ui-needs/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/domain/agent.md`, 8.2, the bullet *A question's line form*.

## Owns and may change

Owns the two files in `files`. May also change `temper/crates/temper-client-domain/src/lib.rs` (the
`pub mod question;` line) and `temper/crates/temper-client-domain/src/tests.rs` (the `mod question;`
line). `temper-client-domain` is a step crate: the strict subset holds
(`AGENTS.md`, 2), and it reads no clock and no state.

Options, each with its default:

- A line's trailing ASCII whitespace, a `\r` among it, is no part of the line.
- The bound of a choice's words is `temper_client_protocol_wire::CUE_BYTES`, as `Act::CardWords`
  (`src/acts.rs`) takes an answer's words; the crate already names that dependency.
- `Parsed` owns its bytes, as the sketch has it; don't return borrows.

## Steps

1. Write `src/question.rs`, its module comment citing the design section.
2. Write `src/tests/question.rs`, at the least:
   - the design's example: the question, its context, its saw, two choices, the first recommended
     without its ending, and `primary()` `Some(0)`;
   - a `Choice: ` line in the middle of the context, then ordinary words at the end: no choice, so
     the fallback, the line among the context;
   - a `Choice: ` line in the context above real trailing lines stays context;
   - blank lines among and after the trailing lines are passed over;
   - no choice marked: `primary()` `Some(0)`; the second marked: `Some(1)`; two marked: the first
     recommended, the second not, and neither ending in its words;
   - `Saw: ` lines with no `Choice: `: the fallback, the `Saw: ` line among the context, `saw` `None`;
   - two `Saw: ` lines among the trailing lines, joined by a line break, in order;
   - `cut` true over the whole example: the fallback, no choices, no saw;
   - the fallback: two paragraphs, the first the question, the rest the context; one paragraph of
     two lines, all of it the question; `primary()` `None`;
   - words that are only `Choice: A`: the first line is the question, no choices;
   - a `Choice: ` with no words, and one past `CUE_BYTES`, are no choices; where that leaves none,
     the fallback;
   - empty words: all fields empty, no panic;
   - every prefix of the example, cut at each byte, parses without a panic, the bytes not valid
     UTF-8 among them.
3. `heavy cargo nextest run -p temper-client-domain --lib -E 'test(question)'`, then fmt, clippy and
   the checks of the implementer's text. Commit and push.

## Report

As the implementer's text asks.
