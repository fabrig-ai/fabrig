+++
id = "words"
part = "words"
kind = "feature"
needs = []
files = ["smith/crates/smith-local-domain/src/person.rs", "smith/crates/smith-local-domain/src/domain.rs", "smith/crates/smith-local-domain/src/agent.rs", "smith/crates/smith-local-domain/src/boundary.rs", "smith/crates/smith-local-domain/src/lib.rs", "smith/crates/smith-local-domain/src/agent_tests.rs", "smith/crates/smith-local-domain/src/parent_tests.rs", "smith/crates/smith-local-domain/src/tests.rs", "smith/crates/smith-local-protocol/src/terminal.rs", "smith/crates/smith-local-protocol/src/lib.rs", "smith/crates/smith-local-service/src/service.rs", "smith/crates/smith-local-service/src/service/program_check/pending.rs", "smith/crates/smith-local-service/src/tests.rs", "smith/crates/smith-local-shell/src/front/headless.rs", "smith/crates/smith-local-shell/src/local_host/words.rs", "smith/crates/smith-local-shell/src/local_host.rs", "smith/crates/smith-local-shell/src/local_host/credentials.rs", "smith/tests/local/src/world.rs", "smith/tests/local/src/person.rs", "smith/docs/design/domain/host.status.md", "smith/docs/design/STATUS.md", "smith/docs/development/debt/D018-domain-formats-text.md"]
effort = "high"
minutes = 120
+++

# The local domain holds none of the person's words

## Goal

`smith-local-domain` writes some of the person's words itself: a turn's lines, the tool verdicts, two answers and the
older-build sentences. The service then patches a refused effort's words over a notice. All of it becomes typed, and
every word moves into `smith-local-protocol/src/terminal.rs`. Not a byte a person sees changes, nor its order.

Acceptance:

- `git grep -nE 'Tool: |Tool finished|Tool failed|Change delivered|Chat parked|older build' --
  smith/crates/smith-local-domain/src` finds nothing.
- `git grep -nE '\b(effort_notice|result_text|answer_text|turn_text|tool_end|bounded_text|append_text)\b' --
  smith/crates/smith-local-domain smith/crates/smith-local-service smith/crates/smith-local-shell` finds nothing, and
  `git grep -n 'Request::Show' -- smith/crates smith/tests` finds nothing.
- The other literals in the domain's source stay, as not the person's words: `person` (a message's label, on the
  channel), the commit body and `Smith-Delivery: ` trailer in `delivery.rs`, `commit `, `files kept` and
  `conflict markers remain` (stored receipts and refusals), `reconcile.rs`'s stored diagnostic, `deliver` (a tool's name),
  and `expect` and `assert` messages. Your report names any other you leave.
- `heavy cargo nextest run -p smith-local-domain -p smith-local-protocol -p smith-local-service -p smith-local-shell -p
  smith-local-world -p smith-local-process-world`, and `-p smith --test end_to_end --features end-to-end`, pass.
  `smith-local-world`'s `world.shown()` assertions don't change.
- `heavy cargo check -p jig-test -p temper-service -p temper-playground --tests` passes: they name
  `smith_local_service::profile` alone, which this leaves.
- The rows of Steps, 1 pass unchanged in expected bytes from base to tip.
- Production net lines: at most +40 across the four crates, `smith-local-domain` down by 60 or more.
- `smith/docs/development/debt/D018-domain-formats-text.md` is gone.

## Inputs

- `temper/docs/plan/debt-14/notes/common.md`
- `smith/docs/development/debt/D018-domain-formats-text.md`
- `smith/docs/design/domain/host.md`, 8: its rule on the words, this branch's contract
- `smith/docs/design/shell.md`, 5.3: the older-build sentences, read by a test
- `smith/AGENTS.md`

## Owns and may change

- **Owns:** the files in the spec's `files` list, for the changes below.
- **May change:** `smith/docs/design/domain/host.status.md`: the **Not yet** entry for 8 goes. `smith/docs/design/STATUS.md`:
  the clause ", and the local domain composes person-facing text (`domain/host.md`)" goes, and nothing else, since the
  sentence's first half stays true. These two edits are yours, a departure from the implementer's text.
- **Not to touch:** `smith/docs/design/domain/host.md` and `shell.md`: they say what this builds. The service's own
  words (`service.rs`, `service/choices.rs`) and `smith_events::effort_words`: outside the item, which is the domain's
  words. `Terminal::show` stays for them. `crates/smith-local-domain/src/delivery*`, `reconcile.rs`.
- **Exceptions to the assertion rule, the cut's:**
  - Tests of `smith-local-domain` that assert `Request::Show` with `b"Chat parked"` or text (`parent_tests.rs`,
    `tests.rs`, `agent_tests.rs`) assert the typed value instead: `Notice::Answer(Said::Parked)`,
    `Notice::Answer(Said::Text(..))`. A domain's test can't reach the terminal. The bytes are held by the rows below and
    by `smith-local-world`.
  - `shown_requests` in `parent_tests.rs` counts `Notice::Turn` and `Notice::Answer` where it counted `Show`.
  - `person.rs`'s `turn_text` test splits in two: the lines in the domain, the bytes in the terminal.
  - `older_store_words_match_all_three_shell_contract_blockquotes_verbatim` moves to `terminal.rs`'s tests, unchanged but
    for its path to `shell.md` and calling `older_words`.
  - Every other assertion stays.
- **Decided by the cut, so don't ask:** `Request::Show`, `Older::words`, `Domain::result_text` and `Service::effort_notice`
  go, and `Domain::tool_verdict` changes its type. They are `pub` for smith's own crates and nothing outside smith names
  them. No stored or wire format holds a `Notice`.
- **Options:**
  - If a refused effort's words read differently in the local world, that is expected and no test reads it: it shows
    `choice_text`'s words as the service does, in place of `run refused: invalid: effort`. Say so in the report.
  - The effort is read when the answer settles, not when the shell drains. If a test shows the difference, stop and ask.
  - If `smith-local-world`'s memory ceiling moves, that's a finding: stop and ask, with the measured peak.

## Steps

1. **Pin first, at base, as the first commit.** `tool_end` and the effort's route have no test, and `turn_text`'s cut has
   one row. In the crates where they live now, add:

   - `turn_text` at small caps: a multi-byte character split at the cap, a short segment after a failed cut (it is
     still written: `append_text` doesn't latch), and a cut inside `\nTool: `.
   - `tool_end`: one `Finished` and one `Failed` row.
   - in `smith-local-service/src/tests.rs`: a run refused with `RunInvalid::Effort` writes `choice_text`'s bytes,
     with no newline.

   They pass at base. Their expected bytes don't change again: the rows move with the code in step 4.
2. **Types,** in `smith-local-domain/src/boundary.rs`, exported from `lib.rs`:

   ```rust
   pub enum Notice {
       // the variants as they are, and:
       /// What the person reads of an accepted turn, in block order.
       Turn { lines: Box<[TurnLine]> },
       /// What the run's answer said.
       Answer(Said),
   }
   pub enum Failure {
       // the variants as they are, and:
       /// The run refused the sent charter's effort.
       EffortRefused(crate::ChoiceNotice),
   }
   /// One line of a turn, as the terminal words it.
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub enum TurnLine {
       Text(Box<[u8]>),
       Tool { name: Box<[u8]> },
       Ended(ToolVerdict),
   }
   /// How a told tool result ended.
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum ToolVerdict {
       Finished,
       Failed,
   }
   /// What an answer said, whole: a headless run writes it uncapped.
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub enum Said {
       Text(Box<[u8]>),
       Change,
       Parked,
   }
   ```

   `Said` is not `host::Answer`, which isn't `Clone`.
3. **The domain.** Decisions stay, words go.

   - `person.rs`: `turn_text` becomes `turn_lines(turn, cap) -> Box<[TurnLine]>`, in block order. It skips empty text, so
     `lines.is_empty()` holds exactly when `shown.is_empty()` held: `show_bytes` is at least 64, so a first character
     always fits. It clips each text and name to `cap` bytes, raw: a segment writes at most its room, so what's cut never
     reaches the screen. `tool_end` becomes `tool_verdict(result) -> ToolVerdict`, its two lists unchanged.
   - `agent.rs`: `answer_text` becomes `said(answer) -> Said`. `Refused` and `Failed` give `Said::Text(Box::new([]))`, as
     they gave no bytes.
   - `domain.rs`: `Domain::tool_verdict` returns `ToolVerdict`, and `Domain::result_text` becomes
     `Domain::said(answer) -> Said`. `accept_turn` pushes `Notice::Turn` where it pushed `Show`, under the same
     `!live_text && !lines.is_empty()`, after `SaveTurn`.
   - `person.rs`'s `finish_answered`: the `Accepted | Parked` arm pushes `Notice::Answer`. The `Refused` arm pushes
     `Failure::EffortRefused(domain.refused_effort())` for `RunInvalid::Effort`, and `Failure::Refused` for the rest.
     `refused_effort` becomes `pub(crate)`.
   - `boundary.rs`: `Older::words` goes. `Request::Show` goes. `bounded_text` and `append_text` go.
   - `lib.rs`'s state table: its `Show` rows say `Turn`, `Answer` or the failure notice, as each is.
4. **The terminal,** `smith-local-protocol/src/terminal.rs`, exported from `lib.rs` as `notice_text` is:

   ```rust
   /// A tool verdict's line, with its newline.
   pub fn tool_verdict_text(verdict: local::ToolVerdict) -> &'static [u8];
   /// What an answer said, whole.
   pub fn said_text(said: &local::Said) -> Box<[u8]>;
   fn older_words(why: local::Older) -> [&'static str; 2];
   ```

   `notice_text`'s match gains three arms. `Notice::Turn`, `Notice::Answer` and `Failure::EffortRefused` each `return`
   before the final `newline`: today none of those bytes ends in one that isn't its own. `Turn` renders through
   `append_text`, moved verbatim from `person.rs`, into a `List` of `cap`, and not through `Shown`'s `append`, which
   reserves a byte and latches. A tool line stays three appends. `Answer` is `said_text`'s bytes through `append_text`
   at `cap`, as `bounded_text` did. `EffortRefused` is `choice_text(choice, chat, cap)`.

   The rows of step 1 for `turn_text` and `tool_end` move here, with the same bytes, through `notice_text` and
   `tool_verdict_text`.
5. **The service.** `service.rs`: `effort_notice` goes, and `notice`'s branch is `terminal.notice` alone. `handed`'s match
   and the last match take `Turn` and `Answer` with the arms that do nothing more: they aren't pushed to the shell, as
   `Show` wasn't. `self.local.headless()` still drops them. `Notice::Failure(_)` carries `EffortRefused` to the shell.
6. **The shell.** `local_host/words.rs`: the `effort_notice` lines go, and a refused effort falls through to `notice_text`.
   `front/headless.rs`: `tool_verdict_text(Domain::tool_verdict(result))`, `said_text(&Domain::said(answer))`.
   `local_host.rs` and `credentials.rs`: the compiler's arms, in the `unreachable!("only operator notices reach the
   shell")` lists.
7. **The worlds.** `tests/local/src/world.rs`: a new arm ahead of `Request::Notice`, for `Notice::Turn` and
   `Notice::Answer`, renders with `notice_text` and runs what `Request::Show`'s arm ran on those bytes. Neither reaches
   `self.notices` or `saw`, so no person's cue fires. `tests/local/src/person.rs`: the compiler's arm, `unreachable!`.
8. **Bite.** Show red, and restore: remove the early `return` for `Turn`, so a turn gains a newline; render `Turn` through
   `append` in place of `append_text`; swap the two verdicts' words; send `EffortRefused` through the final `newline`.
9. `host.status.md`, `STATUS.md`, D018's deletion. `cargo fmt`, the ratchet, clippy on the four crates and the two worlds,
   the tests above, `citations.py --all`. Commit and push.

## Report

What each bite printed. The literals left in the domain. Whether `world.shown()` needed anything. The peak of
`smith-local-world`'s memory test before and after. Net lines, production and tests apart.

## Answers

2026-11-05 (fabrig/debt, the request's author, through agents/loop). **Accept the checked patch.** The domain's -16 production lines stand against the cut's -60 estimate: that figure was the cut's, not a user budget, and the prescribed public types cost 37 lines. The one-expression repair in `smith/tests/local-process/src/referee.rs` is allowed: it's a call repair with no assertion changed.

## Rebase (2026-11-05, step merge, foreman debt-14/words/foreman-2)

resolve STATUS.md by retaining main’s benchmark update and removing only the local-domain wording clause
The merge conflicts in `smith/docs/design/STATUS.md`.
Main commit `006d8e7788` updates the date and expands the benchmark summary; branch commit `79caaada69` removes an adjacent clause.
Rebase onto the landing’s current integration tip.
Keep its `2026-11-05` date and benchmark wording, including `shell-cwd` version 3 and its bundle/baseline gaps.
Remove only `, and the local domain composes person-facing text (`domain/host.md`)`.
The resulting sentences must read: “Two domain worlds call protocol crates (`testing.md`). Two step crates drive skein's JSON tokenizer over a whole buffer by hand …”
Preserve all other status content and the task’s code changes.
