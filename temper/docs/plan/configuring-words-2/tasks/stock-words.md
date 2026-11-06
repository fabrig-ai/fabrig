+++
id = "stock-words"
part = "words"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/stock/configuring.txt", "temper/crates/temper-service/src/stock/chat.txt", "temper/crates/temper-service/src/tests/stock.rs", "temper/crates/temper-service/src/tests/stock_brief.rs", "temper/crates/temper-engine-smith/src/calls.rs", "temper/docs/help/reference/host-tools.md", "temper/docs/design/domain/agent.status.md"]
effort = "medium"
minutes = 40
+++

# The configuring agent's words hold across runs

## Goal

The stock says nothing has changed from one place, has no placeholder a run can't fill, names no operator, says who adds a person and who adopts again, and says how a pending proposal goes. `plan.md` says why each sentence is what it is.

Acceptance:

- `temper/crates/temper-service/src/stock/configuring.txt` is byte for byte `temper/docs/plan/configuring-words-2/notes/configuring.txt`, 5,642 bytes, but for what `notes/common.md` says of a sibling's sentences.
- `chat.txt` says `_Draft, with the user._`, its one change.
- `propose_settings`' description ends `Keep your document separate.`, and `docs/help/reference/host-tools.md` is regenerated from it.
- The tests below fail on the old words and pass on the new, and `heavy cargo nextest run -p temper-service --lib`, `heavy cargo nextest run -p temper-engine-smith --lib` and `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/configuring/)'` pass. `the_page_host_tools_is_what_the_declarations_render`, in package `temper`, passes.
- `temper/docs/design/domain/agent.status.md` has lost the two **Not yet** entries that name `stock-words`, and no other line.

## Inputs

- `temper/docs/plan/configuring-words-2/notes/common.md`, `temper/docs/plan/configuring-words-2/plan.md`
- `temper/docs/design/domain/agent.md`, 8.1, *Its instructions*, and 8.3

## Owns and may change

- Owns `configuring.txt`, the one line of `chat.txt`, the configuring and document tests in `temper-service/src/tests/stock.rs`, and the status entries above: a departure from your text, which closes `docs/design/`.
- May change, figures only: `the_configuring_agent_is_charter_3_and_template_3`'s length, and in `stock_brief.rs` the rows of `every_stock_charter_admits_its_whole_maximal_brief`, the `room` and `MARGIN` of `the_stock_brief_with_a_whole_spec_fits_both_receiving_bounds`.
- May change in `temper-engine-smith/src/calls.rs`: `propose_settings`' description alone.

**The figures, by reading; set what they measure.** `chat.txt` falls by 6 bytes, so charter 1's rows and `room` rise by 6 and `MARGIN` goes from 279 to 285, `brief_bytes` staying 356,270. Charter 3 falls by 31 bytes of words and 4 of description, so its row is about `(3, false, 42_077, 358_058)`. Its check needs charter 3's non-brief at or under charter 1's, 42,108: that's the room there is.

**Options:**

- If `model-catalogue-1/configuring-told` or finding-400 has landed on your base and charter 3 is past that room: shorten what those sentences and ours repeat from a tool's schema or `settings-fields`, say what you cut, and move no limit.
- `temper-engine-domain-world`'s `configuring` suite reads the words, and its phrase `adopt again, reread inventory and propose afresh` still holds. If another phrase there breaks, change that phrase alone, and name it.

## Steps

1. Write the tests first, in `stock.rs`, and see them fail for the reason:

   ```rust
   #[test]
   fn configuring_txt_says_nothing_has_changed_in_one_place() {
       let words = include_str!("../stock/configuring.txt");
       assert_eq!(words.matches("nothing has changed").count(), 1);
       assert!(words.contains("only when you first give a proposal's number"));
       assert!(words.contains("Never before a proposal; never to close a reply."));
       assert!(words.contains("Say in one sentence what changed in it"));
   }

   #[test]
   fn stock_words_hold_no_placeholder_a_run_cannot_fill() {
       for words in [include_str!("../stock/configuring.txt"), include_str!("../stock/chat.txt")] {
           assert!(!words.contains("PROJECT"));
           assert!(!words.contains("DATE"));
       }
       assert!(include_str!("../stock/configuring.txt").contains("titled \"How this project works\""));
   }

   #[test]
   fn configuring_txt_says_who_does_what_in_the_persons_words() {
       let words = include_str!("../stock/configuring.txt");
       assert_eq!(words.matches("operator").count(), 1, "only the word it must not say");
       assert!(words.contains("Not \"stock settings\" or \"operator\""));
       let approvers = words.lines().find(|line| line.starts_with("Approvers:")).expect("approval instructions");
       for phrase in [
           "Whenever someone is to approve or join, before or after acceptance",
           "say all this, and who does each",
           "as Write is not enough",
           "Adopting is theirs, no one else's: Settings, Repositories, Adopt a repository",
       ] {
           assert!(approvers.contains(phrase), "missing: {phrase}");
       }
   }

   #[test]
   fn configuring_txt_says_how_a_pending_proposal_goes() {
       let words = include_str!("../stock/configuring.txt");
       assert!(words.contains("You can't withdraw or replace it while it waits."));
       assert!(words.contains("an owner rejects it under Needs you, with a reason you hear; then propose afresh"));
   }
   ```

2. Copy the note over `configuring.txt`, edit `chat.txt`'s line and the tool's description. Regenerate `host-tools.md` with `SKEIN_CODEGEN_WRITE=1` on that test, `the_page_host_tools_is_what_the_declarations_render`, as it reads.
3. Bring the tests that read the old words up:
   - `configuring_txt_tells_it_how_project_works`: the title's phrase.
   - `configuring_txt_starts_from_temper_own_way`: `In your document, keep temper's own values`.
   - `configuring_txt_speaks_the_persons_terms`: `answer from your document`, for the old `answer from "How PROJECT works"`; drop `Say that nothing has changed once` and `not in every reply`.
   - `assert_document_rule`: `muted line _Draft, with the user._`. Its `Say in one sentence what changed; do not repeat the document in your reply` becomes two phrases, `Say in one sentence what changed` and `do not repeat the document in your reply`. `Use join true to continue the line…` leaves it for a phrase in the chat's own test, since charter 3's copy now has it from `note`'s schema.
4. Set the figures to what they measure. `cargo fmt`, clippy for `temper-service` and `temper-engine-smith`, the tests above. Commit and push.

## Report

Your text's report, with `configuring.txt`'s final bytes, every row and `MARGIN` as measured, and any sentence you changed or cut.
