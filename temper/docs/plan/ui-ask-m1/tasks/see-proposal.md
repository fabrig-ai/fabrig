+++
id = "see-proposal"
part = "see-proposal"
kind = "feature"
needs = ["see", "proposal-read", "see-forge"]
files = ["temper/crates/temper-engine-domain/src/route/see/proposal.rs", "temper/crates/temper-engine-domain/src/route/now.rs", "temper/crates/temper-engine-smith/src/see.rs", "temper/crates/temper-settings-form/src/proposal.rs", "temper/crates/temper-settings-form/src/proposal/**", "temper/crates/temper-settings-form/src/lib.rs", "temper/tests/engine/domain/tests/suite/configuring/asking_proposal.rs", "temper/tests/engine/domain/tests/suite/configuring.rs", "temper/crates/temper-engine-domain/src/route/see.rs", "temper/docs/design/domain/agent.status.md"]
effort = "high"
minutes = 150
+++

# ui-ask-m1: `see` reads a proposal of settings

## Goal

`see proposal tN/K` and `see proposal pN/K` read one proposal of settings through jig's
run proposal read (`proposal-read`), and answer it compactly, rendered by
`temper-settings-form` beside its revision text: waiting, its reason, items and review;
decided, its outcome. In the stubs `see` left for it; nothing of `route/host_route.rs`
or `temper-engine-smith/src/settings.rs` changes.

Acceptance:

- **The read** (`route/see/proposal.rs`, `see`'s stub `read`): the call's flight `see`'s
  `SeeFlight` reading `Proposal` (crate-private, as `see` made it), then `core::NamedAction::ReadProposal { proposer,
  number }` sent as `route/see.rs` sends a settings read (`Work::Core(core::Event::NamedAction
  { to, key, action })`). Its refusal reaches the renderer through `see`'s branch in
  `host_route::render` already.
- **The answer** (`route/now.rs`): `proposal-read`'s unreachable arm for
  `core::Now::ProposalRead { to, key: _, answer }` becomes a call of
  `route/see/proposal.rs`'s `answered`, `see`'s stub, which you fill: it emits `HostDelivery::See { to, line, from,
  answer: SeeAnswer::Proposal(..) }`, the line and offset the flight's, as
  `host_route::settings` emits its delivery; `ProposalRead`'s three answers become
  `SeeProposal`'s three.
- **The text** (`temper-settings-form`, a new module `proposal.rs`, exported; `see`'s
  stub `proposal_text` in `temper-engine-smith/src/see.rs` calls it):
  - translated as `settings_text` translates the other answers, through
    `jig_protocol_engine_clients::translate::objects::proposal` (a waiting proposal's
    document) and `translate::objects::ended` (a decided one's), and rendered from those
    wire records (`jig_client_protocol_wire::v19`), which `temper-settings-form` already
    reads;
  - waiting: a first line `proposal tN/K of settings, waiting, against revision R`
    (`pN/K` for a party's, as `render.rs`' `proposal_text` writes a key); `reason:` and
    the reason's lines, each indented two spaces; `items:` and a line each, `  KIND N:
    set`, `removed` or `reset`, named as a settings file names its kind; then `review at
    revision NEWEST:` and, a block an item, its setting, `now:` and `would be:`, each
    value whole as a settings file holds it, or `too large, B bytes` where the review
    cut it;
  - decided: a first line `proposal tN/K of settings, ` and `accepted as revision R`,
    `rejected: REASON` or `withdrawn`; then `reason:` and `items:` as above, the items'
    heads alone (a decided proposal keeps no values);
  - unknown: 8.4.1's form for a read that finds nothing, `see ADDRESS: not read: no such
    proposal of settings. This says nothing about the settings: see settings lists those
    waiting.`, an error;
  - the `see:` line 8.4.1's for `proposal`; the bound, the cut and `more` by `see`'s form.
- Rows, failing first:
  - `temper/tests/engine/domain/tests/suite/configuring/asking_proposal.rs`, new, a
    child of `configuring` (its one line `mod asking_proposal;` in `configuring.rs`,
    beside `see-forge`'s `mod asking_forge;`, away from the rows `propose` adds), one
    table test, `an_asking_chat_reads_a_proposal`, in the form of `see`'s
    `an_asking_chat_reads_each_address` (`super::` reaches the configuring world's
    fixture and helpers): the stock world; an owner's
    configuring chat proposes a charter change (`configuring.rs`' `propose` helper, as
    `a_rejected_proposal_is_revised` does), and an asking chat (template 5) reads
    `proposal tT/K` waiting (its first line, the reason's first line, `review at
    revision 1:`, the charter's new instructions under `would be:`); the owner rejects
    it (`World::decide`) and the asking chat reads it again (`rejected: ` and the
    reason); a second proposal accepted reads `accepted as revision 2`; `proposal
    tT/999` answers the not-read line; each within the bounds and ending as `see`'s rows
    do; the store's rows unchanged by the reads. A party's proposal (`pN/K`) as a row if
    the world makes one cheaply, else say so.
  - `configuring.rs`, `an_asking_chat_reads_each_address`: its row `proposal t1/1`,
    which answers not yet after `see`, leaves the table (the story above reads
    proposals); nothing else of the test changes (contracts, 9.6).
  - If building a wire record in `temper-settings-form`'s own tests is cheap, a row each
    for a value the review cut and a withdrawn proposal there (`proposal/tests.rs`);
    else the world's rows alone, and say so.
- **Status** (a departure from the implementer's text, which closes `docs/design/`: the
  status must say what is built): delete from `temper/docs/design/domain/agent.status.md`
  the **Not yet** "8.4.1: `proposal tN/K` and `pN/K` read nothing: ..." bullet, and
  nothing else.

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `notes/contracts.md`, sections 8, 9.1,
  9.2, 9.4, 9.5 and 9.6.
- `temper/docs/design/domain/agent.md`, 8.4.1 (the `proposal` row; "A refusal");
  `jig/docs/design/domain/engine.md`, 7.3 ("A run's proposal read"), 17.6 (a proposal of
  settings and its review); `temper/docs/design/client/tools.md`, 5.2 (`proposal show`,
  for what a proposal's reading holds).
- The code it follows: `temper-engine-smith/src/settings.rs`' `settings_text` (the
  translation and restore pattern); `temper-settings-form/src/render.rs`
  (`revision_text`, `proposal_text`, its value text); for words alone, never as a
  dependency, `temper-client-tools/src/jig/render/document/proposal.rs` (`text`,
  `history_text`).

## Owns and may change

- Owns the files in the header; in `temper-engine-smith/src/see.rs`, only
  `proposal_text` and its helpers (`see-forge` writes `landing_text` and `gates_text`
  there); in `temper-settings-form/src/lib.rs`, the module's line and its export.
- `route/see.rs` is `see`'s: of it, one line each, as contracts 9.6 allows and as a
  stated departure from one writer per file: a lint expectation `see` put on
  `SeeReading::Proposal` or on `proposal::answered` (which you now use) removed, and a
  `pub(crate)` this module needs. `configuring.rs`: the `mod asking_proposal;` line and
  the row's removal alone.
- Not yours: `route/host_route.rs`, `temper-engine-smith/src/settings.rs`.
  `temper-engine-smith` gains no dependency (it has `jig-protocol-engine-clients` and
  `temper-settings-form`); `temper-client-tools` stays out of the engine.
- Pre-approved: if the translators need models or limits `proposal_text` lacks, take
  them as `settings_text` takes its (`models(catalogue)`, `Limits::standard()`). If a
  decided party proposal's `reads::Ended` translates as a task's
  (`EndedValue::Proposal`), render both alike.

## Steps

1. The rows; run them; see them fail.
2. The read and its answer; the text.
3. `heavy cargo nextest run -p temper-settings-form --lib`; `-p temper-engine-smith
   --lib`; `-p temper-engine-domain --lib`; `-p temper-engine-domain-world --test suite
   -E 'test(/configuring::/)'`. Then `cargo fmt`, `python3 -I -B
   temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-settings-form -p
   temper-engine-smith -p temper-engine-domain -p temper-engine-domain-world
   --all-targets -- -D warnings`.
4. Commit and push.

## Report

As the implementer's text says; give `see proposal tT/K`'s waiting answer in the world,
whole.
