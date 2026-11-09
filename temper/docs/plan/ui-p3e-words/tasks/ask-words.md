+++
id = "ask-words"
part = "ask"
kind = "feature"
needs = ["words-kind"]
files = [
  "temper/crates/temper-service/src/stock/ask.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-engine-smith/src/see.rs",
  "temper/tests/engine/domain/tests/suite/configuring.rs",
  "temper/docs/design/domain/agent.status.md",
]
effort = "medium"
minutes = 60
+++

# Ask temper writes the words with what it proposes

## Goal

Ask temper's instructions ask it to write, with each proposal, the words the pages
show: a charter's words for each charter it proposes, and the project's words for
a way of the project's own. A configuring story shows `propose_settings` carrying
them: an asking chat's proposal of a charter, its words and the project's words,
accepted, and both read back.

Acceptance, by `temper/docs/design/domain/agent.md`, 8.4.2 (*Words with the
settings*, *Its tests*):

1. **The sentence,** exactly `temper/docs/plan/ui-p3e-words/notes/contracts.md`,
   section 9: in `temper/crates/temper-service/src/stock/ask.txt`'s
   *Configuring*, after the sentence ending "and what the change does to running
   work.", a space and "With it, write in their terms what the pages show:
   charter-words for each charter, what it does; project-words for the project's
   own way, a headline and a short lede." No other byte of the file changes. The
   file is then 8,052 bytes, within the 8,192 its test holds; if the trunk's
   `ask.txt` has grown so that it would pass 8,192, stop with `Question:`, naming
   the sizes. The sentence is the user's to review (`plan.md`): never reword it.
2. **Its assertion** (`temper-service/src/tests/stock.rs`, the Ask temper
   charter's test, about line 343): the *Configuring* part holds the sentence
   whole, as a `contains` beside the headings' list.
3. **`ADDRESSES`' comment** (`temper-engine-smith/src/see.rs`, about line 35)
   says what is true now: the line of addresses an unknown address answers with,
   `ask.txt`'s and the settings kinds' past it. Its value doesn't change.
4. **The story,** a row of the engine world's asking stories
   (`temper/tests/engine/domain/tests/suite/configuring.rs`, beside
   `an_owners_asking_chat_proposes_and_hears_it_accepted`, about line 630, on its
   fixture): an owner's Ask temper chat proposes, in one `propose_settings`, a
   charter item, that charter's `charter-words` and the project's
   `project-words`; the proposal's items are the three; accepted, `see settings
   charter-words N` and `see settings project-words` read back the words
   proposed. And a `charter-words` of 121 bytes refused at the call, nothing
   proposed.
5. **Bite:** propose without the `project-words` item and assert it read back,
   see the row fail; restore.
6. **Status:** delete the bullet naming `ask-words` in
   `temper/docs/design/domain/agent.status.md` (8.4.2) (`notes/common.md`, *Status
   files*).

## Inputs

- `temper/docs/design/domain/agent.md`, 8.1 and 8.4.2.
- `temper/docs/design/protocol/configuration.md`, 13.2 (*A setting's words*).
- `temper/docs/plan/ui-p3e-words/notes/contracts.md`, sections 1, 4 and 9.
- `temper/docs/plan/ui-p3e-words/notes/common.md`.

## Owns and may change

- Owns: the one sentence, its assertion, the comment, the story's row and the
  status bullet.
- **The stock first-request room** (`temper-service/src/tests/stock_brief.rs`,
  16,384 bytes on both request shapes) reads `ask.txt`: if it fails by the
  sentence, stop with `Question:`, naming the figures.
- `charter-lines` may run beside this task in `tests/stock.rs` and
  `configuring.rs`: keep to your rows.
- Not yours: the kinds' names, reads and form (`words-kind`, merged), the stock's
  lines (`charter-lines`).

## Steps

1. Read the inputs; `ask.txt`'s *Configuring*; the Ask temper charter's test;
   `an_owners_asking_chat_proposes_and_hears_it_accepted` and its helpers.
2. The sentence (`wc -c` the file after), its assertion, the comment.
3. The story's row; the bite.
4. Runs: `heavy cargo nextest run -p temper-service --lib -E
   'test(/stock/)'`; `-p temper-engine-domain-world --test suite -E
   'test(/configuring/)'`; then the focused tests of `-p temper-service`, `-p
   temper-engine-smith` and `-p temper-engine-domain-world`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-service -p temper-engine-smith --all-targets -- -D warnings`.
6. Commit (why, in the message); `python3 -I -B tools/citations/citations.py --base
   origin/main`; push.

## Report

As the implementer's text says, and: `ask.txt`'s size after; the story's proposal
items and what was read back; the rows and the bite.
