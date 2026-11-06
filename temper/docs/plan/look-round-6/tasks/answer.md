+++
id = "answer"
part = "answer"
kind = "feature"
needs = ["call-line"]
files = [
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/cards/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/delegation.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/golden/chat.txt",
  "temper/crates/temper-client-shell-terminal/src/tests/fixtures/chat.rs",
  "temper/tests/client/tests/live_text.rs",
  "temper/tests/client/tests/suite/live_text.rs",
  "temper/tests/real-loop/src/terminal.rs",
  "temper/tests/real-loop/tests/live_text.rs",
  "temper/tests/real-loop/tests/suite/terminal.rs",
  "temper/crates/temper/tests/support/person.rs",
  "temper/docs/design/client/journeys/**",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 70
+++

# The answer: no reasoning, no notices, and a small block where it is being written

## Goal

What a person reads of an answer is the answer. "Content in 3" and "2164 bytes" go, a conversation says nothing of what was cut, and "Writing..." gives way to a small block after the last word. The second of three tasks landing the look request `chat-work`.

Acceptance:

- `Block::Opaque` draws nothing in either conversation, and a message holding only opaque blocks draws no shell. `CardWord::OpaqueDialect` and `OpaqueBytes` are gone.
- `CardWord::TurnCut` is gone: `prefix`, `markdown`, `provisional`, `request_message`, `committed` and `committed_lines` say nothing of a cut, and `note_only` goes with its one reader.
- `Line::Provisional` draws "Temper" over its Markdown, then the end, by one function, `writing_end(b, words)`: in a browser an empty `Role::Text` with `WRITING_END`; in the terminal a `Role::Status` "▌". `CardWord::Provisional` is gone.
- The block shows after the last word in a browser, at 1440, 1000 and 420 wide, on a paragraph, a list's last item, a heading and a code block with no final newline, and alone where nothing is written yet. On a closed fenced code block it falls on the line after the last, deferred to `chat-reading-a` (see *Answers*). Once the turn commits, the words don't move and only the mark goes.
- Every rule of the overlay's *The answer* block is in `temper.css`, with the difference below, and `.temper-agent-page` draws the same end.
- The terminal's print shows the words, then `* ▌`, where it showed `* Writing...` above them. The status bar's "Writing" stays.
- This task's **Not yet** entry in `temper/docs/design/client/README.status.md` is gone, but for the closed fenced code block's, which names `chat-reading-a` as its owner.

## Inputs

- `temper/docs/design/client/README.md`, 3.5: *the agent's words*, *the words being written*, *While it works*.
- `temper/docs/design/draft/chat-reading.md`, section 3, for what's `chat-reading-a`'s: the whole text, a block read once, a marker that waits.
- `temper/docs/plan/look-round-6/notes/contracts.md`, binding; `temper/docs/plan/look-round-6/notes/common.md`.
- The request: `git show 3f707babb9`, its message's *The answer*, its `overlay.css`, and `02.html`, `03.html`.

## Owns and may change

Owns the files in the header.

Readers and neighbours:

- `temper/tests/real-loop/src/terminal.rs` (near 197, 501, 590, 592), `tests/suite/terminal.rs:9`, `tests/live_text.rs:11` and `:12`, and `temper/tests/client/tests/suite/live_text.rs:6`: "Writing..." as the mark of a turn being written becomes "▌", and its line `"    * ▌\n"`. Each test's meaning stays: a turn being written is marked, and a committed one isn't. An assertion on the mark's place above the words moves below them.
- `temper/crates/temper-client-shell-terminal/src/tests/fixtures/chat.rs:22` and `golden/chat.txt:17`: a synthetic page; the mark and words swap, as the view now draws them.
- `temper/crates/temper/tests/support/person.rs:249`: the step on "Earlier text omitted" goes. Keep the step on "delegate" beside it.
- `temper/crates/temper-client-view/src/tests/chat.rs` (near 98, 109, 395, 707, 1128, 1260, 2401, 2480) and `tests/delegation.rs` (near 738, 800, 897): each changes with its part. `long_provisional_text_says_that_earlier_text_was_omitted` becomes the opposite. A test whose only point was the notice goes, and the report says which. Lists of words that must be absent keep their words.
- `temper/docs/design/client/journeys/**`: reprinted where this changes them. Any gallery golden this moves is regenerated.

Options, each decided:

- **The block's place.** skein's reader closes its blocks, so the view can't put a span inside the last one. Default: the element stays for the fact and is `display: none`, and the overlay's declarations for `.temper-writing-end` move to `::after`:

  ```css
  #app > .temper-session .temper-writing-end { display: none; }
  #app > .temper-session .temper-writing-end:only-child { display: inline-block; /* the overlay's declarations */ }
  #app > .temper-session .temper-says:has(> .temper-writing-end) > :nth-last-child(2):not(ol, ul)::after,
  #app > .temper-session .temper-says:has(> .temper-writing-end) > :is(ol, ul):nth-last-child(2) > li:last-child::after { content: '' / ''; display: inline-block; /* the overlay's declarations */ }
  ```

  The report names it for the account as a difference. If `chat-reading-a` has by catch-up given skein's reader a way to write inside the last block, take it, and use the overlay's rule as it stands.
- If `chat-reading-a` has already removed a `TurnCut`, an opaque block or the 512-byte bound, keep theirs and do the rest. If its tests and these say the same, keep one.
- A text cut at the bound ending in "…" is `chat-reading-a`'s (`chat-reading.md`, section 4), as are a bold still being written, a marker that waits and an answer longer than `text`: the report names them as deferred.
- A numbered list's markers are `chat-reading-d`'s. The overlay's `ol` rules land here all the same, and the report names the state as deferred.

Not this task: the line (`chat_line.rs`), "Older", `Line::Left`, jig, skein.

## Steps

1. `pages/chat.rs`: the removals above. `has_content` no longer counts `Opaque`. `prefix` loses `b` and `words`; `markdown` loses its status.
2. `writing_end`, in `pages/chat.rs` beside `provisional`. `provisional` draws `P::Temper` in `WHO`, as a committed turn does.
3. `cards/words.rs`: the words go; a removed word leaves its number unused. `limits.rs`: anything only these used. `look.rs`, `tests/look.rs`: `WRITING_END`.
4. `temper.css`, under `/* ==== ui: chats ==== */`: the overlay's `ol`, `li::marker`, `.temper-writing-end` and `@keyframes chat-writing-end`, as above, and the agent page's copies.
5. Tests: `an_opaque_block_draws_nothing_and_no_shell`, `a_cut_text_says_nothing`, `the_words_being_written_end_in_the_mark`, `the_mark_goes_when_the_turn_commits_and_the_words_stay` (the same text nodes in order before and after), and the terminal's print. Each new rule bites.
6. Reprint and regenerate: `heavy cargo run -p temper-client-journeys`, goldens. Read each diff.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal --all-targets -- -D warnings`; the focused tests of those, `temper-client-journeys` and `temper-client-gallery`, and of `temper/tests/client` and `temper/tests/real-loop`'s terminal and live-text tests, which read the changed meaning. Commit with the trailer and push.

## Report

As the implementer's text asks, with these for the account: each overlay rule of *The answer*, landed or how it differs; each state of the message this task reaches and the test holding it; what's deferred to `chat-reading-a` and `chat-reading-d`.

## Answers

2026-11-06. **Question:** a closed fenced code block keeps its final newline in skein's reader, so `pre::after` draws the mark on the next line: authorise a skein end-marker hook, accept the next-line mark, or assign it to `chat-reading-a`?

**Answer: assign it to `chat-reading-a`; no skein change here.** The request says what needs the engine is theirs, drawn as honestly as the view can and named as deferred. `chat-reading-a/growing-reader` already gives `read_growing`, whose `end` runs inside the last word's block, and `chat-live`'s acceptance has the mark "in the line of the last word". A second reader change now would collide with that branch's rewrite of `markdown.rs`, for a state that lasts from a closing fence to the next word. The mark on the next line is honest, and the words don't move when it goes. The acceptance above is amended.

What the next run does, on the pushed `f5dd5a9405`:

- Reword the entry in `temper/docs/design/client/README.status.md`: **Not yet:** 3.5, *the words being written*: a closed fenced code block keeps its final newline in `skein-ui`'s reader, so the end's mark falls on the line after its last; `chat-reading-a`'s `chat-live` takes it, with `read_growing`.
- Change no code, CSS or test for this, and rerun no render: `renders/fenced-code-newline.json` is the proof of the deferred state, and the run's checks stand. Run the citations check, then push.
- Report `Done`, carrying over the previous report's overlay and state tables (`look-round-6/answer-1`), with the closed fence's row reading deferred, to `chat-reading-a`.
