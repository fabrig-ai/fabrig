# Spot review of debt-14's smith part (2026-11-05, merge e3b575fc1f)

Read-only reviewer, using the user's quality criteria. Only git ran: every "no test would fail" and "no fixture
sends" is from reading. Line numbers are at the merge.

Verdict: growth mostly earned (+40 production, +311 tests; about 250 of the 311 are real fakes, fake-versus-protocol
tests, true moves and new byte pins; about 60 are scaffold or overbuild). No live correctness bug.

1. **Gap: the effort rule lost its pin.** `smith-local-service/src/tests.rs:1009` now injects a ready `EffortRefused(choice)`.
   The rule moved to `smith-local-domain/src/person.rs:362`, and no test names `RunInvalid::Effort`. Add a step test in
   `parent_tests.rs` asserting `EffortRefused(ChoiceNotice::Effort { .. })`, add a no-newline case to the terminal tests,
   delete this test and the `render_notice` seam at `service.rs:915`. Net about -5.
2. **Risk: the session fake is fitted to its rows.** `smith/tests/session/src/translate.rs:260`,
   `smith/tests/protocol-llm/tests/suite/world_fakes.rs:7`: `"edits":"x"`, `[]`, `[1]` decode to an empty `Edit`; `"all":1`
   becomes Once; a nested object inside an edit gives Missing; `offset`, `window`, `depth`, `glob`, `timeout_seconds` and
   `output_bytes` are dropped. `listed` (`smith/tests/tools/src/translate.rs:178`) keeps `.git/` paths and sorts unsorted
   output. No fixture sends these today. Panic on them (about 6 lines). `feedback_text` (lines 356-373) is too faithful:
   replace with `extend_from_slice` and drop the row at `world_fakes.rs:37`, -18.
3. `smith/tests/local/src/world.rs:1010` renders the notice and compares words: match `Notice::Answer(Said::Parked)`. `b"Run
   cancelled"` at :1020 looks dead. One `Notice` arm, about -11. `parent_tests.rs:808`, `:950` also accept `Notice::Turn`, so
   they can pass with no answer notice: narrow to `Notice::Answer(_)`.
4. Residue: `smith/tests/machine/tests/suite/stories.rs:522-524` (useless imports), `const ALL` and `exited` copied from
   `simulation.rs`, about -10. `smith-local-protocol/src/terminal.rs:475` restates a match: add `Ended(Failed)` to the test
   at :443, -9. The clips at `person.rs:220,223` can't fail any test: drop `cap`, or add a row.
5. Production: `terminal.rs:264` `append_text` and `:393` `append` are one UTF-8 loop: a prefix helper, about -14. `said_text`
   (:256) clones a box: return `&[u8]`. `smith-local-domain/src/domain.rs:89,95` are forwarders: re-export, about -8.
6. Docs overclaim: `smith/docs/design/testing.status.md:24` and `STATUS.md:18` count two domain worlds on protocol crates;
   `smith/tests/host/src/inline.rs:154` makes three. D018 closed on "all words in `terminal.rs`", but `service.rs:1005` and
   `smith-local-shell/src/front/headless.rs:104,130,142` still hold words. Comments at `terminal.rs:93,100,308` are stale.
