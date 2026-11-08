+++
id = "detail-row"
part = "detail"
kind = "feature"
needs = []
files = ["smith/crates/smith-host-protocol/src/tests.rs", "smith/docs/design/domain/host.status.md", "smith/docs/design/STATUS.md"]
effort = "medium"
minutes = 30
+++

# The host's kept detail holds a panic's message

## Goal

`skein-fail-stop` now closes a panic's report with its `panicked at` line
and message after the backtrace (task `report-end`, built beside this
one to `notes/contracts.md`, section 2). This task pins, in the host's
existing kept-detail table, that an agent's error output in that form,
its backtrace past the bound, leaves the line and the message in the
failed agent's detail, whatever the deliveries; and closes the design's
gap.

Acceptance:

- `a_hosts_standard_error_keeps_its_last_bytes_whatever_its_deliveries`
  (`smith/crates/smith-host-protocol/src/tests.rs`) becomes a table over
  rows of `(stderr, detail_bytes, expected detail)`, each run at every
  delivery cut it runs today (`[20, 1]`) and at 4,096, the stderr read's
  most:
  - the row it has today: `abcdefghijklmnopqrst`, bound 8, detail
    `mnopqrst`;
  - a panic's report in the contracts' form, built in the test from a
    thread name, a location, a message (`the burst panicked`) and a
    backtrace of numbered frame lines over 4,096 bytes, at bound 1,024:
    the detail is the report's last 1,024 bytes, and it ends with
    `thread 'main' panicked at <location>:\nthe burst panicked\n`.
- The row bites: shown failing once with `keep_tail` changed by hand to
  keep the first bytes in place of the last, then restored.
- The **Not yet** entry `4: a failed agent's detail keeping its panic's
  `panicked at` line and message …` is gone from
  `smith/docs/design/domain/host.status.md`, and the phrase `a failed
  agent's panic message kept in its detail (`domain/host.md`),` from
  `smith/docs/design/STATUS.md`'s summary.
- Clippy clean, fmt clean, `smith-host-protocol`'s tests green.

## Inputs

- `smith/docs/plan/subagent-burst/notes/common.md` and `notes/contracts.md`,
  section 2.
- `smith/docs/design/domain/host.md`, section 4, its bullet *What it
  hands its parent*, as this cut wrote it.
- `skein/docs/design/shell.md`, section 6, its last paragraph.

Departures from the implementer's text, and why:

- **Status files:** you delete the two pieces named above, since this
  task and `report-end` close them together; touch no other design text.
- **No `needs`:** the row's bytes are literal, built to the contracts
  note; `report-end`'s code is not called. Both land in the same plan.

## Owns and may change

Owns:

- `smith/crates/smith-host-protocol/src/tests.rs`: the test above and a
  helper beside it that builds the report.
- `smith/docs/design/domain/host.status.md` and
  `smith/docs/design/STATUS.md`: the two deletions.

Reader and neighbour files: none. No production code changes: the
process keeps the last `detail_bytes` bytes today
(`smith-host-protocol/src/process.rs`, `keep_tail`).

Pre-approved options:

- **The report's form differs** from what `report-end`'s branch writes
  (seen on the trunk if it has merged): the contracts note wins; build to
  it and say so in the report.

## Steps

1. Read the test and `process.rs`'s `keep_tail` and `tree_ended`.
2. Reshape the test into the table and add the row; run
   `heavy cargo nextest run -p smith-host-protocol --lib -E
   'test(=tests::a_hosts_standard_error_keeps_its_last_bytes_whatever_its_deliveries)'`
   (check the count), break `keep_tail` by hand, see the row fail,
   restore.
3. Delete the two status pieces.
4. Once, at the end: `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`;
   `heavy cargo clippy -p smith-host-protocol --all-targets -- -D
   warnings`; `heavy cargo nextest run -p smith-host-protocol`;
   `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit (`smith-host-protocol: a failed agent's detail keeps its
   panic's closing line`, the body saying why) and push.

## Report

As the implementer's text says: the first line, the commit pushed, the
tests run and their counts, the rule broken by hand and the failure
seen, and your implementation and check minutes apart.
