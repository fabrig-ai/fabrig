+++
id = "foreman-charter"
part = "foreman"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/stock.rs", "temper/crates/temper-service/src/stock/foreman.txt", "temper/crates/temper-service/src/tests/stock.rs", "temper/crates/temper-service/src/tests/stock_brief.rs", "temper/crates/temper-init/template/temper.json", "temper/crates/temper-init/src/tests/documents.rs", "temper/docs/examples/owned.json", "temper/docs/help/charter.md", "temper/docs/design/protocol/configuration.status.md"]
effort = "medium"
minutes = 50
+++

# foreman-charter: the stock's charter 6, a foreman's

## Goal

The stock holds a fifth charter, charter 6, a foreman's: read-only as the reviewer's charter
4 is, its instructions held in the binary as `stock/foreman.txt`, and the stock's rules and
policy 1 delegate to it. `temper init`'s document and `docs/examples/owned.json` raise the
limits the box's work needs for it (configuration.md, 13.4). No change uses it yet: the
change procedure delegates its foremen in later tasks (contract C4).

Acceptance:

- `temper-service/src/stock.rs`: `fn foreman(llm) -> Charter`, built as `reviewer` is
  (`charter(6, 16, include_bytes!("stock/foreman.txt"), llm)` with `policy.modify = false` and
  `policy.agents = false`), added to `checked`'s charters after the reviewer, so the stock's
  charters are 1, 2, 4 and 6 in that order. Its wake policy is `WakePolicy::DEFAULT`, as the
  others'.
- `ceiling`'s delegation: `Executor::Charter(6)` after `Charter(4)`, so the rules and policy
  1 delegate to it.
- `stock/foreman.txt`, exactly (one paragraph, then the three verdicts, no trailing blank
  line beyond the file's newline):

  ```
  You are a change's foreman. A producer working on the change asked it the question in your brief. Settle it from the change's words, the design in your checkout and the change's pushed head, which you read with read_branch and read_file; read what the question needs and nothing else. You only read: never edit, push or run anything. Finish with a verdict:
  - answer: the change's words, the design or the draft settle the question; your text is the answer, naming the words or the design section by path and section that settle it.
  - finish: the work done holds and the rest is outside what the change owns; your text tells the producer what to finish with and what to list under Left.
  - escalate: nothing you read settles it, or it is a choice the draft leaves open; your text says why, for the change's requester. Never guess past what you read.
  ```

- `temper-init/template/temper.json`'s `engine.limits.overrides`: `authority.executors` and
  `tasks.executor_kinds` 8, `tasks.charters` 8, `tasks.contract_choices` 4 (the other
  members unchanged). `docs/examples/owned.json`: `authority.executors` and
  `tasks.executor_kinds` 8 and `tasks.charters` 8; `tasks.contract_choices` 4 too if small's
  own value is below 3.
- `docs/help/charter.md`: the paragraph naming the forge's two charters names three, the
  third "its foremen's, `charter 6`, which reads the change and the design and settles its
  producer's question, and edits, pushes and runs nothing"; the bullet "The reviewer's
  charter is `charter 4`, fixed by temper" adds "and the foreman's `charter 6`". (A stated
  departure from the implementer's text: this task edits that help page.)
- `configuration.status.md`: remove from the **Departs** entry for 13.4 the clause that
  `authority.executors` and `tasks.executor_kinds` are 6, not 8 (now 8); remove from the
  **Not yet** the clause "13.4: `tasks.charters` 8 in `temper init`'s document and
  `docs/examples/owned.json`", and from "13.2, 13.4: kind 3's number 3 and charter 6, the
  foreman's, with the rules' and policy 1's delegation to it;" only the words "and charter
  6, the foreman's, with the rules' and policy 1's delegation to it", leaving kind 3's
  number 3 (which the change procedure builds).

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.4 (the table's charter 6 and the
  rules rows; "Each charter's host families"; "The box's limits for that work") and 13.2
  ("The forge's charters are read when a change is made").
- `temper/docs/plan/reference-m5/notes/common.md`; `notes/contracts.md`, C4.

## Owns and may change

Owns the files in the header.

- Readers that count or list the stock's charters or executors, edited only for the new
  count or list: `temper-service/src/tests/stock.rs` (the floors table's rows for
  `authority.executors` "the stock's rules need 7 executors", `tasks.charters` "the stock
  offers 4 charters", `core.policy_bytes` and `core.run_bytes` at their measured values;
  the delegation lists near lines 574 and 596), `stock_brief.rs`, and, if they fail for the
  new charter alone, `temper/crates/temper/tests/support/first_repository.rs`,
  `temper/crates/temper/tests/suite/startup.rs` and `temper-init/src/tests/documents.rs`.
  Name each in the report.
- If raising `tasks.contract_choices` to 4 makes a startup check refuse a bound that depends
  on it (an inbox or record bytes floor), raise the dependent override as the check names
  it, by the least that passes, and say so; if that bound is jig's own and not an override,
  set `tasks.contract_choices` to 3 instead and say so.
- If a stock test asserts the host part's largest charter is the reviewer's, it stays so:
  the foreman's host part is the reviewer's with other instructions; a test that measures
  the largest takes whichever is larger.

## Steps

1. The charter, its text, the delegation; the limits in both documents; the help page; the
   status lines.
2. Tests in `temper-service/src/tests/stock.rs`, beside `the_reviewers_words_say_to_fail_when_it_cannot_review`:
   - `the_foremans_charter_reads_and_gives_a_verdict`: the stock's charter 6 exists, its
     instructions are `stock/foreman.txt`, `modify` and `agents` false, `inspect` true,
     `shell` false; the rules' and policy 1's delegation contain `Executor::Charter(6)`.
   - `the_foremans_words_name_the_three_verdicts`: its instructions contain `answer:`,
     `finish:` and `escalate:` and "never edit, push or run anything".
3. Narrow: `heavy cargo nextest run -p temper-service --lib -E 'test(/stock/)'` (check the
   count), then `heavy cargo nextest run -p temper-init`, `heavy cargo nextest run -p
   temper-docs`, and `heavy cargo nextest run -p temper --test suite -E 'test(/startup/)'`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-service -p temper-init --all-targets -- -D warnings`. Commit ("Add the stock's
   foreman charter") and push.

## Report

As the implementer's text says, with the measured floors the test's rows now hold.
