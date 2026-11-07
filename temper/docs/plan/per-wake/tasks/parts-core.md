+++
id = "parts-core"
part = "parts"
kind = "feature"
needs = ["surface"]
files = ["jig/crates/jig-domain-core/src/settings/check.rs", "jig/crates/jig-domain-core/src/routing/run/preparation.rs", "jig/crates/jig-domain-core/src/routing/run/parts.rs", "jig/crates/jig-domain-core/src/routing/run.rs", "jig/crates/jig-domain-core/src/tests/settings/parts.rs", "jig/crates/jig-domain-core/src/tests/runs/parts.rs", "jig/crates/jig-domain-core/src/tests/settings.rs", "jig/crates/jig-domain-core/src/tests/runs.rs", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 130
+++

# per-wake: the core checks a charter's parts and prepares a run from those its reasons meet

## Goal

The core's half of `jig/docs/design/domain/tasks.md`, 8.6 and `domain/engine.md`, 7.1: a
settings value or a configured charter whose parts are out of shape is refused; a run's
preparation chooses the parts its reasons meet, makes its instructions from them, offers
its start only the messages those parts cover, and gives its claim the kinds it is
offered. The tasks child's half (reasons, relays, the fence) is the offering task's, built
to the same fields in parallel.

Acceptance:

- **The check.** `charter_fault` (`src/settings/check.rs`), and with it every path that
  checks a charter (a settings change, the start's defaults, a restore's `restore_fit`),
  refuses: more parts than `limits.tasks.charter_parts` as `Fault::Count`; a part whose `on`
  is empty or not `is_known()` as `Fault::Shape`; parts on a charter whose
  `policy.resume` is true as `Fault::Shape`; and a charter whose `policy_owned_bytes` plus
  `parts_owned_bytes` passes `limits.run_bytes` as `Fault::Shape`, where the policy alone
  is checked today. A settings change refused so answers `Malformed::Settings { setting:
  Charter(n), fault, detail: None }` at its item, as an out-of-shape charter does today. No
  new refusal vocabulary (`notes/common.md`).
- **The run's instructions and offered kinds**, two functions in a new
  `src/routing/run/parts.rs`, by `contracts.md`, section 4:

  ```rust
  /// The instructions a run reads: the task's own, or its charter's followed by each part
  /// its reasons meet, in order, each after `\n\n` (domain/engine.md, 7.1, step 4).
  pub(crate) fn run_instructions(charter: &Charter, own: Option<&[u8]>, reasons: tasks::Reasons) -> Box<[u8]>;
  /// The kinds a run is offered beside the always-offered: every kind for a charter with no
  /// parts or a task with its own instructions, else the `on` of each part it reads,
  /// without `first` (domain/tasks.md, 8.6).
  pub(crate) fn offered_kinds(charter: &Charter, own: bool, reasons: tasks::Reasons) -> tasks::Reasons;
  ```
- **Preparation** (`src/routing/run/preparation.rs`): where `workspace_prepared` sets
  `policy.instructions` (the task's own instructions or the charter's) it takes
  `run_instructions(<the charter at the run's revision>, own, context.reasons)`; it computes
  `offered_kinds` there, filters the start's messages by `jig_domain_tasks::offered(word,
  offered)` before taking the first `run_inbox` of them (`start_messages`, so `start_proof`
  offers only those), and keeps `offered` with the prepared run until `route_claim_prepared`
  pushes `Event::Claim { reasons: context.reasons, offered, .. }` (the surface passed
  `Reasons::ALL`). A job's claim keeps the surface's values.
- **Tests, failing first:**
  - `src/tests/settings/parts.rs`, one table of charters `(name, parts, resume, expected
    fault)`: none (accepted); two well formed (accepted); nine against a limit of 8
    (`Count`); an empty `on`; bit 14; parts with `resume`; one part whose instructions pass
    the run bound (`Shape` each); run through the settings change and through the start's
    check, as `src/tests/settings/documents.rs` runs its table through both.
  - `src/tests/runs/parts.rs`, one table `(name, parts, own instructions, reasons, expected
    instructions, expected offered)`: no parts (`S`, `ALL`); parts `first: F` and `question:
    Q` with reasons `FIRST` (`S\n\nF`, `FIRST`'s `on` without `first`: empty), with
    `FIRST | QUESTION` (`S\n\nF\n\nQ`, `QUESTION`), with `QUESTION` (`S\n\nQ`, `QUESTION`),
    with `RESULT` (`S`, empty); a part on `question | result` read once for both; a task's
    own instructions (`own`, `ALL`).
  - one row in an existing preparation test (`src/tests/runs/` or `src/tests.rs`, wherever
    a prepared run's start messages are pinned): a charter with a `first` part, its task's
    inbox holding a person's words and a question, prepares a start carrying the words and
    not the question, and a claim whose `offered` lacks `QUESTION`.
  Break each rule by hand (drop the filter, read every part), see its rows fail, restore.
- Delete from `jig/docs/design/domain/engine.status.md` the sentence "7.1, 9: a run's
  preparation chooses no parts: ... no reasons or offered kinds to the tasks child." and
  nothing else (a departure from the implementer's text, which closes `docs/design/`: the
  status must say what is built when this lands).
- This changes what a shared item means (what a run reads and is offered): run the focused
  tests of `jig-domain-core`, the core world (`jig/tests/core`), `jig-test-domain` and
  `jig-protocol-config`, and `heavy cargo check -p temper-engine-domain --tests`.

## Inputs

- `temper/docs/plan/per-wake/notes/common.md`; `notes/contracts.md`, sections 1 to 4.
- `jig/docs/design/domain/tasks.md`, 8.6 (whole) and section 7 ("Instructions are replaced
  whole"); `jig/docs/design/domain/engine.md`, 7.1 (steps 2 to 4) and section 9;
  `jig/docs/design/protocol/configuration.md`, section 6 ("A charter's parts").

## Owns and may change

- Owns the files in the header; the mod lines for the new modules.
- Readers: a test elsewhere in `jig-domain-core`, `jig/tests/core` or `jig-test-domain` that
  pinned a run's instructions or start under a charter with parts (none exists before this
  plan): none expected.
- Pre-approved: if the charter at the run's revision is not at hand where `policy` is set,
  take it as `charter_of` does for that revision and say how; if `restore_fit` cannot use
  `charter_fault` for parts, check the four rules there too and say so.
- Not yours: the tasks child (the offering task), the LLM choice and what a task's document
  shows (the later task), the configuration's JSON (config), the store and the wire.

## Steps

1. The two tables and the preparation row, failing.
2. The check; `parts.rs`; the preparation's instructions, filter and claim.
3. The rows pass; the bites above.
4. The status sentence.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-domain-core --all-targets -- -D warnings`; the focused tests named above; the
   citations check.
6. Commit (why: a run reads the shared instructions and the parts for why it runs, and is
   offered what they cover); push your branch.

## Report

As the implementer's text says, and also: where the prepared run keeps `offered` until its
claim; any existing expectation changed.
