+++
id = "asked-sentence"
part = "cards"
kind = "feature"
needs = ["ways-wire"]
files = [
  "temper/crates/temper-engine-domain-ways/src/words.rs",
  "temper/crates/temper-engine-domain-ways/src/lib.rs",
  "temper/crates/temper-engine-domain-ways/src/path/cards.rs",
  "temper/crates/temper-engine-domain-ways/src/tests.rs",
  "temper/crates/temper-engine-domain-ways/src/tests/derivation.rs",
  "temper/crates/temper-engine-domain-ways/src/tests/invariants.rs",
  "temper/crates/temper-engine-domain/src/route/ways/agents.rs",
  "temper/crates/temper-engine-domain/src/route/ways/view/charter.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/tools/gallery/src/worlds/ways.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 60
+++

# A card shows its charter's first sentence whole

## Goal

The request's task 5 (`temper/docs/plan/look-round-24/inputs/request.md`). The reviewer's card says "Asked to: “Review the change at the named head: read its pull request, the files it changes and its checks, and judge them against wh”": `first_sentence` cuts at 128 bytes (`VALUE`), and the card cuts again at 122 inside its quotes, mid-word, with no "…". An agent's "Asked to" now holds 184 bytes with its quotes (`ASKED`, from `ways-wire`), its first sentence whole within them, and past them cut at the last word's end that fits, with "…". Every other fact keeps 128 bytes, cut the same way.

Acceptance, by `temper/docs/design/domain/forge.md`, 8.13, *A stop's card*, and `temper/docs/design/client/README.md`, 4.6, *A path* (`Fact`'s `value`), as this cut wrote them, and `temper/docs/plan/look-round-24/notes/contracts.md`, *The engine*:

1. **`words.rs`:** `pub fn cut_on_word(value: &[u8], bound: usize) -> Box<[u8]>`: `value` whole when its length is at most `bound`; otherwise its longest prefix ending just before an ASCII space (the spaces before the cut trimmed) whose length plus 3 is at most `bound`, then "…" (U+2026); with no space in reach, the longest prefix at a character's end that leaves room for "…", then "…"; a `bound` under 3 gives the empty value. Always valid UTF-8 for valid UTF-8 in, at most `bound` bytes. `first_sentence(instructions: &[u8], bound: usize) -> Box<[u8]>`: the first sentence as today (through the first ". " or ".\n", trimmed), then `cut_on_word(sentence, bound)`. `lib.rs` exports both.
2. **The stop's card** (`path/cards.rs`): `agent_card`'s "Asked to" is “ + `cut_on_word(&agent.asked, ASKED - "“”".len())` + ”, its fact's value bounded by `ASKED`, not `VALUE`; `fact` bounds every other value with `cut_on_word(value, VALUE)` in place of the character cut (who and name keep `text`). `append` keeps its own bound.
3. **The readers of `first_sentence`:** the agents' inputs (`temper-engine-domain/src/route/ways/agents.rs`, about line 62) take `first_sentence(&policy.instructions, ASKED - "“”".len())`; the charter view's "Asked to" (`route/ways/view/charter.rs`, about line 39) takes `first_sentence(&policy.instructions, ASKED)` and is built there as a `ways::Fact` whose value is that, unclipped by `view.rs`'s `fact`, so that `view.rs` does not change in this task (`rule-view` owns it).
4. **The rows, no new test function** (`temper-engine-domain-ways`):
   - `sentence_stops_trims_and_cuts_at_a_character_end` (`src/tests.rs`, about line 310) is renamed `sentence_stops_trims_and_cuts_at_a_word_end`, calls `first_sentence(input, bound)` and keeps today's first five rows at `VALUE`; its other rows become: the stock reviewer's first sentence (`temper-service/src/stock/review.txt`'s, 152 bytes) whole at `ASKED - 6`; the same sentence at bound 64 giving "Review the change at the named head: read its pull request,…" (62 bytes: the longest prefix ending before a space whose length plus 3 fits); one word of 200 bytes at `ASKED` giving its first 181 bytes and "…"; a `"x".repeat(VALUE - 1) + "érest"` at `VALUE` giving no split "é"; bound 2 giving the empty value.
   - The derivation's bound sweep (`tests/derivation.rs`, about lines 526 to 554) keeps its `VALUE` and `VALUE + 7` sweeps for every fact but "Asked to", and asserts "Asked to" within `ASKED` and ending "…”" past it; the hard-coded `"x" + "€".repeat(128)` row expects the character-end cut with "…" that `cut_on_word` gives (no space in reach).
   - The invariants (`tests/invariants.rs`, about line 16) hold every fact's value within `VALUE`, but "Asked to" within `ASKED`, valid UTF-8; the fuzzy suite that calls them runs.
5. **The worst ways document still fits:** `the_worst_small_ways_document_fits_its_decision` (`temper-service/src/tests/family.rs`, about line 3398) fills "Asked to" to `ASKED` and the other facts as today; it passes, and the report gives the encoded size it measures against the allowance (about 59,000 bytes against 65,536, by the cut's arithmetic).
6. **The gallery's copy of the cut** (`temper/tools/gallery/src/worlds/ways.rs`, `agent`, about line 176, whose comment says the derivation reserves both quotes inside 128 bytes) quotes its sentence whole when it fits `184 - 6` bytes, as the engine now does; its comment says so. Its worlds' sentences are the stock's, so the reviewer's card in `ask-ways-card` shows the whole sentence.
7. **Status:** delete the **Not yet** entry naming `asked-sentence` in `temper/docs/design/domain/forge.status.md` (8.13, its first entry). No other design line.
8. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-engine-domain-ways`, `--profile fuzzy -p temper-engine-domain-ways`, `-p temper-engine-domain`, `-p temper-service`, `-p temper-client-gallery`; `cargo check -p temper-engine-domain-world --tests`; clippy on the packages changed.

One commit or more, each with the trailer `Look-request: ask-temper-ways`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-24/notes/common.md`, `temper/docs/plan/look-round-24/notes/contracts.md`.
- `temper/docs/plan/look-round-24/inputs/request.md`, task 5.
- `temper/docs/design/domain/forge.md`, 8.13, *A stop's card*.
- `temper/docs/design/client/README.md`, 4.6, *A path* and *A project's ways* (the worst document fits one decision's delivery in the smallest profile).
- A word cut with "…" already written: `temper/crates/temper-engine-domain-forge-issues/src/text.rs`, `title` (about line 8); reuse its approach, not its crate.

## Owns and may change

Owns the files above. Reader files, call sites only: any other caller of `first_sentence` (`rg -n 'first_sentence' --type rust`) passes `VALUE` where it shows a fact of 128 bytes, `ASKED` where it shows "Asked to". Not `view.rs` (`rule-view`), not `derive.rs`'s CI card (`ci-workflows`), not the wire (`ways-wire` lands it), not the client.

Options, each decided:

- **The worst-case test fails** past the allowance: it is a red of this cut's arithmetic; lower `ASKED` in `path.rs` to the largest multiple of 8 that fits and holds the reviewer's sentence (158 bytes quoted), say so in the report, and leave the wire at 184.
- **A client test pins a cut "Asked to"** (`rg -n 'judge them against' temper`): it now reads the sentence whole; change it, keeping what else it checks, and name it.

## Steps

1. Read the Inputs, `words.rs`, `cards.rs`, the callers and the tests named above.
2. `cut_on_word` and `first_sentence` (acceptance 1) with their rows (first row of 4); bite: drop the trim of the space before "…", see the 64-byte row fail, restore.
3. The card and the readers (2, 3), the derivation's and invariants' rows (4), the service's worst case (5), the gallery (6); bite: bound "Asked to" by `VALUE` again, see the reviewer's row fail, restore.
4. The status entry (7), the runs (8); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main` after committing.
5. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| A card's "Asked to": its charter's first sentence whole (the reviewer's) | landed (`temper-engine-domain-ways`, `cards.rs`, `ASKED`) |
| Past the bound, cut on a word with "…" | landed (`words.rs`, `cut_on_word`, every fact) |
| The worst ways document in the smallest profile | fits: N bytes of 65,536 (the service's test) |
| The charter view's "Asked to" | landed: the same sentence, within `ASKED` |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4) |

## Report

The implementer's report: the functions as written, the rows and the bites with their failures, the worst document's size, the runs' counts, and the account as committed.
