+++
id = "stock-words"
part = "words"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/stock/configuring.txt", "temper/crates/temper-service/src/tests/stock.rs"]
effort = "medium"
minutes = 25
+++

# The configuring agent's words

## Goal

The stock instructions speak the person's terms, never send anyone to a terminal, start who approves from the person, never report an unmade read as a setting, and say "nothing has changed" once. The tools the words name come from sibling tasks; this task touches words and their test alone.

Acceptance:

- `temper/crates/temper-service/src/stock/configuring.txt` is byte for byte `temper/docs/plan/configuring-words/notes/configuring.txt`.
- `the_configuring_agent_is_charter_3_and_template_3` holds the new length: 6386, from 4807.
- A new test `configuring_txt_speaks_the_persons_terms` in `temper-service/src/tests/stock.rs` holds the phrases below, present and absent.
- `configuring_txt_tells_it_how_project_works` and `assert_document_rule` pass unchanged.
- `heavy cargo nextest run -p temper-service --lib` passes, its stock, room and brief tests among them, and `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/configuring/)'` passes: it reads the same file.

## Inputs

- `temper/docs/plan/configuring-words/notes/common.md`
- `temper/docs/design/domain/agent.md`, 8.1, *Its instructions*

## Owns and may change

Owns `configuring.txt` and its tests in `temper-service/src/tests/stock.rs`.

**If a stock fit fails:** `every_stock_charter_admits_its_whole_maximal_brief` or `stock_charters_leave_at_least_2048_bytes` may fail for charter 3, since the words grow by 1,579 bytes. No limit or margin moves. Shorten the sentences that are new against the old text, keeping every phrase the new test holds, until it passes; set the length to what you end with; and report the shortfall you measured, the room each charter ended with and the final byte figure. If keeping those phrases can't fit, stop with a `Question:` and the measurements.

## Steps

1. `cp temper/docs/plan/configuring-words/notes/configuring.txt temper/crates/temper-service/src/stock/configuring.txt`.
2. Set the length in `the_configuring_agent_is_charter_3_and_template_3`.
3. Add the test, beside `configuring_txt_tells_it_how_project_works`:

   ```rust
   #[test]
   fn configuring_txt_speaks_the_persons_terms() {
       let words = include_str!("../stock/configuring.txt");
       for phrase in [
           "Speak the person's terms",
           "only in a proposal or its outcome, or when asked",
           "answer from \"How PROJECT works\"",
           "Never send them to a terminal, temper ctl or a command",
           "the owner's acceptance checks",
           "Say that nothing has changed once",
           "not in every reply",
           "you (ada on the forge), or someone else?",
           "read_permission names a forge user's login",
           "propose the rule naming the approver",
           "never report it as a setting",
           "each read with project and no number",
       ] {
           assert!(words.contains(phrase), "missing: {phrase}");
       }
       for phrase in ["(kind 7)", "(kind 6)", "establish eligible people first", "naming pages"] {
           assert!(!words.contains(phrase), "still says: {phrase}");
       }
   }
   ```
4. Break one phrase by hand, see the test fail, restore. `cargo fmt`, `heavy cargo clippy -p temper-service --all-targets -- -D warnings`, the tests above. Commit and push.

## Report

Your text's report, with the final byte figure of `configuring.txt` and each stock charter's room as `stock_charters_leave_at_least_2048_bytes` prints it.
