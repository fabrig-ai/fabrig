+++
id = "proposal-read"
part = "see"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-core/src/vocabulary.rs", "jig/crates/jig-domain-core/src/reads/settings.rs", "jig/crates/jig-domain-core/src/reads.rs", "jig/crates/jig-domain-core/src/reads/rows.rs", "jig/crates/jig-domain-core/src/reads/vocabulary.rs", "jig/crates/jig-domain-core/src/documents/objects.rs", "jig/crates/jig-domain-core/src/routing/run/calls.rs", "jig/crates/jig-domain-core/src/lib.rs", "jig/crates/jig-protocol-tools/src/decode.rs", "jig/crates/jig-protocol-tools/src/render.rs", "jig/crates/jig-protocol-tools/src/tests/settings.rs", "jig/tests/core/tests/suite/settings_reads.rs", "jig/tests/core/tests/settings/support.rs", "jig/docs/design/domain/engine.status.md", "temper/crates/temper-engine-domain/src/route/now.rs"]
effort = "high"
minutes = 150
+++

# ui-ask-m1: a run reads one proposal of settings

## Goal

jig's core gains a run's read of one proposal of settings: a task's, of the reading
task's project, or a party's, answered with what a party's proposal read gives, its
reason, items, review and outcome, and decided by the `settings` family as the run's
settings reads are. Nothing in temper asks for one yet; `see-proposal` does.

Acceptance:

- `NamedAction::ReadProposal`, `Now::ProposalRead` and `ProposalRead`, as
  `temper/docs/plan/ui-ask-m1/notes/contracts.md`, 9.4 gives them, each with a one-line
  doc comment citing `engine.md`, 7.3; `ProposalRead` exported from the crate root beside
  `SettingsAnswer`.
- **Its path:** `routing/run/calls.rs` takes `ReadProposal` where it takes
  `NamedAction::Settings`, on the unrecorded path, its family `ToolKind::Settings`; the
  read (in `reads/settings.rs`, beside `run`) checks the family with
  `authorize_tool(key, ToolKind::Settings)` first, refused as a settings read is
  (`Now::EffectAnswer`), then, with no party (a run has none):
  - **waiting:** a task's proposal (`Proposer::Task(N)`) whose task is live, holds
    proposal K pending, is of the reading task's project and proposes settings; or a
    party's (`Proposer::Party(N)`) pending settings proposal K: answered at once,
    `ProposalRead::Waiting`, the document a party's live read builds
    (`documents/objects.rs`, `proposal`, its review from `settings_review`), with no
    acts, since a run takes none;
  - **decided, withdrawn or replaced:** loaded as a party's stored read loads it (the
    point read of `Scope::Proposal`: a task's history row, a party's `DecidedProposal`),
    answered `ProposalRead::Decided` with the `reads::Ended` a party's read gives, the
    row's visibility the run's (below), not a party's;
  - **the run's visibility,** in place of a party's: the proposal is a settings one; a
    task's is of the reading task's project; checked when the read starts and again when
    its row loads, with the family and the reading run's current attempt (as
    `run_task`'s reads recheck theirs);
  - **otherwise** (no such proposal, not settings, another project's task, a row gone)
    `ProposalRead::Unknown`; a load refused busy `CallPart::Unavailable`, as a settings
    read's is.
  The answer leaves as `Now::ProposalRead { to, key, answer }`.
- **jig's tool kit** (`jig-protocol-tools`): `decode.rs` decodes contracts 9.4's two JSON
  forms of the `settings` tool into `NamedAction::ReadProposal`; `render.rs` renders the
  answer in 9.4's one line; jig's services that answer `Now::SettingsAnswer`
  (`jig-test-service`, `jig-core-kit`: the compiler names them) answer `Now::ProposalRead`
  alike.
- **temper's one arm, a stated departure** (the implementer's text allows a type's forced
  arms in your own tree; this task carries one in temper's, as the lead's rule for a
  required field does): `temper/crates/temper-engine-domain/src/route/now.rs` gains
  `core::Now::ProposalRead { .. } => unreachable!("...")`, worded as the notes arm
  beside it is: temper sends no `ReadProposal` until `see-proposal`, which replaces it.
  Nothing else of temper changes: run `heavy cargo check -p temper-engine-domain --tests`
  and `-p temper-service --tests`; if another temper match on `Now` or `NamedAction`
  fails to compile, give it the same arm and say so.
- Rows, failing first:
  - `jig/tests/core/tests/suite/settings_reads.rs`, beside
    `a_run_reads_the_inventory_a_setting_a_revision_and_a_history_page`, one new test,
    `a_run_reads_a_proposal_waiting_and_decided`: the fixture's pending party proposal,
    `{"kind":"proposal","party":OWNER,"number":K}`, equal to the party's
    `reads::Read::Proposal { proposer: Party(OWNER), proposal: K }` document but for its
    `acts`; a run's own task proposal (proposed as `suite/settings_proposals.rs`'s
    `run_propose` does; move that helper into `settings/support.rs` if you share it)
    read waiting, then rejected by the owner and read again, `Decided` equal to the
    party's `Answer::Ended`; `Unknown` for a number nobody holds and for a task's
    proposal read from another project's run, and for a party's proposal that is not
    settings if the fixture makes one cheaply (else say so); and a run without the
    settings family refused, in the existing refusal's shape.
  - `jig-protocol-tools/src/tests/settings.rs`,
    `settings_reads_decode_through_the_reading_tool_declaration` (or a table beside it,
    since its rows decode to `NamedAction::Settings`): the two forms, and one naming both
    `task` and `party` refused.
- `jig/docs/design/domain/engine.status.md`: delete the **Not yet** "7.3: a run's
  `settings` family reads no proposal ..." bullet, and nothing else (a departure from the
  implementer's text: the status must say what is built).

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `notes/contracts.md`, 9.4 and 9.5.
- `jig/docs/design/domain/engine.md`, 7.3 (the `settings` row; "A run's reads are not
  recorded"; "A run's proposal read"); `jig/docs/design/client/domain/contract.md`,
  section 6 (a party's proposal read); `jig/docs/design/domain/tasks.md`, section 9.
- The code it follows: `routing/run/calls.rs` (the unrecorded path of
  `NamedAction::Settings`); `reads/settings.rs` (`run`, `answer`, `run_loaded`,
  `revision_read`'s load); `reads.rs` (`start`, `start_jig`, `loaded`, `Scope::Proposal`);
  `reads/rows.rs` (`scope_allowed`, `from_store`'s proposal rows); `reads/vocabulary.rs`
  (`row_visible`); `documents/objects.rs` (`proposal`, `party_proposal_visible`).

## Owns and may change

- Owns the files in the header. jig's other readers of `NamedAction` and `Now` (the
  compiler names them) gain the arm, each only for that.
- Refactoring the live document's builder so that a run's read reuses it with no party
  (no acts, the run's visibility) is yours; a party's read must answer byte for byte as
  today: `jig-domain-core --lib` and `jig-core-world`'s `settings_` and `proposal`
  tests hold it.
- No wire or stored schema changes (contracts, 9.4).
- Pre-approved: if a task's decided proposal can't be found by `(task, K)` without the
  task's live record (`proposal_rows` is the owner's), answer an ended task's proposal
  `Unknown`, and say so. If the store's point read can't carry the settings destination,
  add a destination of your naming beside `Destination::Settings`, and say so.

## Steps

1. The rows; run them; see them fail (or not compile).
2. The action, the answer, the path, the live builder without a party, the stored load
   with the run's visibility; the tool kit; temper's arm.
3. `heavy cargo nextest run -p jig-domain-core --lib`; `-p jig-protocol-tools --lib`; `-p
   jig-core-world --test suite -E 'test(/settings_/) | test(/proposal/)'`; the two
   temper checks above. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-core -p
   jig-protocol-tools -p temper-engine-domain --all-targets -- -D warnings`.
4. Commit and push.

## Report

As the implementer's text says; name where the run's visibility is checked at the start
and at the load.
