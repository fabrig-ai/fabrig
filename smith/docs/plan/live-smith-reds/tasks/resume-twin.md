+++
id = "resume-twin"
part = "resume-twin"
kind = "feature"
needs = []
files = ["smith/tests/local-process/src/world.rs", "smith/tests/local-process/src/world/**", "smith/tests/local-process/tests/suite/simulation.rs"]
effort = "high"
minutes = 100
+++
# The second-run resume story, run against the fakes

## Goal

`smith::live a_second_run_resumes_the_chat_from_its_files` (`smith/tests/end-to-end/live.rs`) is
red after landing: an exec exits non-zero within ~6 s for both runs together, and which one is
unknown until `live-report` lands. The story runs ChatGPT twice on one chat: first a report-form
run that is told to remember a code, then, resumed from the chat's files, a change-form run with
`--deliver` that writes the code into a file. Two of its paths have no offline test:

1. **A report chat resumed as a delivered change.** The local-process world resumes a chat from
   its files only with the same contract (`simulation.rs`
   `a_chat_resumes_from_its_files_across_runs_of_the_binary`); the live story changes the
   contract between runs (report, then change with checks and delivery).
2. **Provider reasoning replayed across runs.** On the same endpoint the resumed history carries
   the first run's opaque reasoning items (OpenAI's `reasoning` with `encrypted_content`), which
   the provider refuses unless they come back verbatim; within one run this works (the ChatGPT
   commit story passes), across runs from the chat's files nothing checks it.

Add both as local-process tests, the lowest world that runs the binary across runs. If either is
red, the cause is smith's and is fixed below with a unit test; if both are green, say so: the
live report names the cause after landing.

Acceptance:

- `simulation.rs` `a_report_chat_resumes_as_a_delivered_change`: a `World::new` report run, then
  a `World::changed` run on the same `Files`; the second exits 0, delivers its change (as
  `World::changed`'s own tests check), and its first provider query carries the first run's
  `finish` call (as `a_chat_resumes_from_its_files_across_runs_of_the_binary` checks).
- `simulation.rs` `a_resumed_chat_replays_its_reasoning_verbatim`: the first run's provider
  answer holds one `api::Line::Opaque` reasoning item, OpenAI's shape
  (`{"type":"reasoning","id":"rs_live_twin","encrypted_content":"opaque-twin","summary":[]}`),
  before its `finish` call; the second run's first query holds an `api::Part::Opaque` with those
  bytes, unchanged, before the first run's `finish` call.
- A `World` constructor (or a `Behaviour` field) for the reasoning answer, beside `World::new`;
  existing constructors keep their behaviour.
- fmt, clippy and the focused tests below are green; any fix has its unit test.

## Inputs

- `smith/docs/plan/live-smith-reds/notes/common.md`.
- `smith/docs/design/domain/session.md`, 3 (turns and transcripts: what a turn keeps of a
  provider's opaque blocks); `smith/docs/design/domain/run.md`, 6 (resuming);
  `smith/docs/design/protocol/transcript.md`, on opaque blocks; `skein/docs/design/fake-llm.md`,
  the `Opaque` line and part (sections 2-5).
- `smith/tests/local-process/src/world.rs` (`World::new`, `World::changed`, `configured`,
  `Behaviour`, `crossing_scripts` as an example of scripting turns with `api::Line`).

## Owns and may change

- The files in this spec's header.
- Pre-approved:
  - If `World::changed` cannot start on another run's `Files` (it sets up the checkout first),
    give it the carried chat as `configured` already copies `chat/`, keep the checkout's setup,
    and say what you changed.
  - If test 1 is red because the resumed run refuses a contract unlike the saved one, read
    `smith/docs/design/domain/run.md`, 6 and `smith/docs/design/shell.md` on `--chat`: if the design allows the
    change, fix smith in the one crate that refuses it (`smith-local-domain` or
    `smith-domain-session`, most likely) with a unit test; if the design forbids it, stop and ask
    (the live story would then be the thing that is wrong).
  - If test 2 is red (the bytes are dropped, reordered or altered), fix it in the one crate that
    changes them (`smith-transcript`, `smith-domain-session` or `smith-protocol-translate`) with
    a unit test there; if the cause is in skein (`skein-llm`'s replay) reproduce it in skein's own
    tests first and fix it there (`AGENTS.md`, 6: the lowest tier). Changes beyond one crate, or
    to a stored transcript form, are a question.
  - If the fake provider in this world speaks a dialect whose opaque parts are not OpenAI's
    reasoning items, keep the bytes above anyway (the fake keeps them verbatim) and say so.

## Steps

1. Write test 1; `heavy cargo nextest run -p smith-local-process-world --test suite -E 'test(=simulation::a_report_chat_resumes_as_a_delivered_change)'`
   (check the path with `--list` if it selects nothing); note red or green.
2. Add the reasoning constructor and test 2; run it the same way; note red or green.
3. Fix any red as Pre-approved; rerun.
4. `cargo fmt`; `heavy cargo clippy -p smith-local-process-world --all-targets -- -D warnings`
   (and any fixed crate); `heavy cargo nextest run -p smith-local-process-world`; the fixed
   crate's tests and its direct dependents' (`heavy cargo check -p <dependent> --tests` across
   trees for a public change).
5. Commit, push.

## Report

Each test's first result (red or green, what it saw), any fix and its crate, the tests run
(selected, passed), and implementation and check minutes apart.
