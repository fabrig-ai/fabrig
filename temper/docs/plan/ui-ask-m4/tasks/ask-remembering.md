+++
id = "ask-remembering"
part = "remember"
kind = "feature"
needs = ["remember-surface"]
files = ["temper/crates/temper-service/src/stock/ask.txt", "temper/crates/temper-service/src/tests/stock.rs", "temper/crates/temper-service/src/limits.rs"]
effort = "medium"
minutes = 45
+++

## Goal

Put the *Remembering* part of Ask temper's instructions into `ask.txt`,
word for word as `temper/docs/design/draft/ask-temper.md`, section 8 gives
it, between *Showing* and *Your voice*, and hold the instructions to that
draft's 10 KB (`temper/docs/design/domain/agent.md`, 8.4, *Its
instructions*). The user reviews these words before landing: change none.

Acceptance:

- `temper/crates/temper-service/src/stock/ask.txt` holds, after the
  *Showing* part and before `## Your voice`, the draft's `## Remembering`
  heading, an empty line, its one paragraph exactly as the draft's
  section 8 has it (its `…` the Unicode ellipsis, its quotes straight), and
  an empty line; nothing else of the file changes.
- `ask_temper_is_charter_11_and_template_5` (`src/tests/stock.rs`): its
  heading list gains `## Remembering` before `## Your voice`; the assertion
  that the file has no Remembering part becomes one that its *Remembering*
  part, split from `## Remembering\n\n` to `## Your voice`, equals the
  draft's paragraph and its newline; *Showing*'s split ends at
  `## Remembering`; its bound becomes `words.len() <= 10_240`, its message
  "Ask temper's instructions fit ask-temper.md, 4.1's 10 KB".
- `asks_first_request_fits_both_shapes` (`src/tests/ask_bound.rs`) passes
  within 16,384 bytes with `remember` declared and these words.
- The engine domain world's whole-file check of the instructions
  (`temper/tests/engine/domain/tests/suite/configuring.rs:633`,
  `an_owners_asking_chat_proposes_and_hears_it_accepted`) and every other
  test that pins `ask.txt`'s words (this cut's survey:
  `temper-engine-smith/src/tests/see.rs:434`,
  `temper/tests/engine/domain/tests/suite/forge_routes/goal_delivery.rs:112`
  and `:203`, `stock.rs:1815`) pass unchanged.
- Delete the **Not yet** entries `8.4: ask.txt has no *Remembering* part`
  in `temper/docs/design/domain/agent.status.md` and `13.4: charter 11's
  ask.txt has no *Remembering* part` in
  `temper/docs/design/protocol/configuration.status.md`.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`.
- `temper/docs/design/draft/ask-temper.md`, section 8, its `## Remembering`
  paragraph, and 4.1's table (instructions, 10 KB);
  `temper/docs/design/domain/agent.md`, 8.4, *Its instructions*.
- `temper/crates/temper-service/src/tests/stock.rs`, lines 333 to 400.

## Owns and may change

- **Owns:** the two files in this spec's header.
- **Options, pre-approved:** none on the words. If the first request
  passes 16,384 bytes, report `Question:` with the parts the test prints
  on both shapes, the words in place and pushed.

## Steps

1. The paragraph; the stock test's rows and bound.
2. Break the words once (drop its last sentence) to see the row fail, and
   restore.
3. `heavy cargo nextest run -p temper-service --lib -E
   'test(/stock|ask_bound/)'`; `heavy cargo nextest run -p
   temper-engine-domain-world -E 'test(/asking|goal_delivery/)'`;
   `heavy cargo nextest run -p temper-engine-smith --lib -E 'test(/see/)'`;
   `heavy cargo clippy -p temper-service --all-targets -- -D warnings`.
   Commit and push.

## Report

As the implementer's text asks; give `ask.txt`'s size and the first
request's parts on both shapes.

## Answers

2026-11-09. **Charter 11 needs 20,795 assignment bytes against
`root.journal.run_bytes`' 20,480: raise the bound, or defer Remembering?**
Raise the bound; the words stay as they are. `run_bytes` sizes the root's
carrier for a stored run policy so that it holds the stock charters. It is
not one of the user's budgets or timeouts, and `9311793a79` raised it the
same way, from 16,384, when the stock goals grew. In
`temper/crates/temper-service/src/limits.rs`, `ROOT.journal.run_bytes`
becomes `24_576`, the next 4 KiB step above the need. Its comment names
what it holds now: the stock's largest run policy, charter 11's 20,795
bytes. Make the same edit to the value's other statements, each named in
the report: `temper/docs/help/reference/limits.md` (regenerate it if a
generator writes it), `temper/docs/design/protocol/configuration.md`,
13.8's "the box's 20,480", and `temper/docs/examples/reference/README.md`'s
"the box's bound of 20,480 bytes" (the README is read by tests: run them,
and the reference room's 65,536 stays). If the stock's check then names
another run-bytes floor in a world's, the playground's or a test's
document (`core.run_bytes`), raise it the same way, to the next 4 KiB above
what the check prints, and name it. Raise no other bound. If the first
request still passes 16,384 bytes, the spec's `Question:` stands.

2026-11-09. **`temper/tests/client/src/front.rs`'s stored-charter read
allowance, `loads.transcript_bytes = 8_870`, now fails
`asking_chat::the_page_starts_the_asking_chat` on restart: raise it, or
defer?** Raise it. "Raise no other bound" kept the box's and the reference
room's bounds still; this allowance is a fixture's measurement of the stock
charter 11 as stored, the same family as the run-bytes floors that answer
let grow, and the restart oracle checks the fixture still restores it. Set
it to the stored charter 11's payload bytes as measured with these words,
not inferred (9,470 is a guess: the stored form may not grow byte for byte
with `ask.txt`); keep the value exact, as it stands now, and its comment
naming the new figure. Name it in the report. Then run
`heavy cargo nextest run -p temper-client-world -E
'test(/asking_chat/)'` and the spec's other runs. If the test still fails
with the allowance at the measured size, or another product bound in
`limits.rs` is named, report `Question:` with the failure; raise nothing
else.
