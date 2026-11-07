# machine-costs: figures

Instruction counts are Callgrind `Ir` totals for one test selected by exact name
through nextest, in the same unoptimized build on the same x86-64 builder.
The runner used `NEXTEST_DOUBLE_SPAWN=0`. Listing processes are excluded.
TLS handshakes draw entropy, so their instruction counts can vary slightly.

The baseline serial focused run had 387 `hand` tests and 26 `machine` tests.
Its costliest hand test was `stale_full_after_deleted_is_discarded` (38 ms);
the selected Forgejo median was `a_failed_step_after_more_steps_than_inner_is_found`
(10 ms, the middle of 96 Forgejo tests). The machine median was
`a_returned_answer_is_never_offered_again` (16 ms); its baseline profile was
collected by restoring the pristine base after identifying that median. The
prescribed start and GitHub routing tests are included too. These wall times
select representatives; the comparisons below use instructions.

| Row | Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---|---:|---:|---|---|
| `suite` `machine` | `machine::owner_close_wins_or_follows_each_terminal` | 1,947,965,463 | 919,345,951 | Quiet ticks repeatedly allocate peer keys, inspect requests and check deadlines; settling is 97.44% of the baseline | Stop after sixteen quiet ticks, retaining the 1,000-tick bound and both existing exit conditions; all 76 worlds stay |
| `suite` `machine` | `machine::a_returned_answer_is_never_offered_again` | 41,177,459 | 35,374,946 | Three owner actions settle even when their work has ended; settling costs 5,934,129 Ir | Quiet termination reduces settling to 97,254 Ir; every delivery and close assertion stays |
| `suite` `machine` | `machine::the_start_reaches_ready_from_the_recorded_answers` | 9,841,209 | 9,851,503 | Stream delivery and recorded-response processing through `run_for`, rather than `settle` | Stays: `run_for` keeps its clock-driving behaviour; tick progress bookkeeping adds 0.10% |
| `suite` `hand` | `hand::github::github_installation_loss::stale_full_after_deleted_is_discarded` | 33,219,510 | 30,917,000 | Two RSA signings cost 22,621,385 Ir; draining adds 7,353,732 Ir | Remove key-list copies and halve the quiet tail; signing stays |
| `suite` `hand` | `hand::forgejo::contents::a_failed_step_after_more_steps_than_inner_is_found` | 16,144,095 | 13,167,685 | Draining costs 12,840,858 Ir, including 1,053,650 Ir copying peer keys; stream tracing and allocation accounting also recur | Visit peers in place, gather events locally, and stop after sixteen quiet rounds |
| `suite` `hand` | `hand::github::github_calls::repository_account_selects_its_accepted_installation` | 5,473,654 | 4,471,441 | Draining costs 2,359,104 Ir, including 187 peer-key copies costing 333,282 Ir | No peer-key list on rounds or stream requests; empty worlds return immediately; draining falls to 1,353,517 Ir |
| `suite` `machine` | `machine::settlement_counts_fragments_before_a_header_line_is_complete` | unknown | unknown | New regression, not profiled: a header needs more fragments than the quiet threshold before a complete-line demand can answer | Counts trace growth as progress; without it the test fails at `world.script_done()`, and passes after restoration |
| `temper-engine-protocol-forge-world::suite` `protocol` | `protocol::github_reaches_ready_through_its_accepted_installations` | 57,923,886 | 51,058,287 | Its installation action settles 1,000 ticks despite finishing its same-time work | Shared quiet termination reduces that action to 177 ticks; the reader also costs less |

The close test calls `tick` from `settle` 176,286 times before and 10,609
after. No world, seed, assertion, fixture count, timer or budget changed.
The focused package has 523 passing tests (522 before, plus the regression),
and its nine fuzzy tests pass. The temper reader's 32 focused tests pass.

## Found and not fixed

- RSA signing: the costliest hand test calls `sign_rs256` twice, once from
  `limits::access` to judge the key and once from `Jwt::bearer`. Their combined
  22,621,385 Ir are 73.17% of its final profile. Avoiding the first needs a
  judge that parses without signing. Keeping a parsed key would depart from
  `skein/docs/design/tls.md`, 4: a key is parsed at each signing and never kept
  by skein-tls. Both operations stay as the task requires.
- Wire tracing and allocator accounting are shared work owned elsewhere. In
  the final Forgejo median, `Wire::answer` costs 2,745,941 Ir and `Wire::take`
  1,291,648 Ir, including real byte delivery as well as trace formatting;
  `core::fmt::write` across the test costs 3,115,557 Ir. Counting allocator
  allocation and deallocation cost 2,577,962 and 2,190,174 Ir respectively.
  Removing those costs needs the shared trace/accounting work; these inclusive
  counts overlap and must not be added as independent savings.
- Real TLS work stays in every secure world. The close test still spends
  397,066,112 Ir in rustls `ConnectionCore::process_msg`. Sharing settings
  would not remove these handshakes. Repeated server configuration and trust
  construction cost about 17.33 million Ir in the baseline: even removing
  all of it estimates only `17.33 / 1947.97 * 0.279 = 0.0025` CPU s.
- Listener settlement costs 15,287,984 Ir in the baseline close test, or
  about 0.0022 CPU s by the same rule; repeated `locate_script` construction
  costs 1,106,458 Ir and `deadline_rows` 55,232 Ir. None reaches 0.05 CPU s,
  so the listener's quiet count and per-row fixtures stay.
- The next owned hand cause is request inspection: the Forgejo median's
  `request_lines` costs 88,629 Ir after the change. Applying its baseline
  share even to the whole 1.92 CPU s hand row gives about
  `88629 / 16144095 * 1.92 = 0.0105` CPU s. The selected GitHub test's
  `request_bytes` costs 2,283 Ir, estimating 0.0008 CPU s across that row.
  These are below the stop rule. The waiting-refusal prefix and both
  twenty-call scenarios keep their counts; no profile puts their reduction
  above the stop rule.

## Tests that went

None.

## Counts

None. The close test retains `35 * 2 + 1 + 5 = 76` worlds. Quiet-round
termination is harness scheduling, not a reduction in scenario rows.
