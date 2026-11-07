+++
id = "configuring-learns"
part = "configuring"
kind = "feature"
needs = ["stock-models", "model-act"]
files = ["temper/crates/temper-service/src/stock/configuring.txt", "temper/crates/temper-engine-smith/src/settings.rs", "temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-settings-form/src/**", "temper/crates/temper-client-tools/src/jig/**", "temper/crates/temper-client-tools/src/settings.rs", "temper/crates/temper-client-view/src/cards/proposal.rs"]
effort = "high"
minutes = 110
+++

# The configuring chat can read and propose a model

## Goal

A change of a model is a proposal the configuring chat can make, as any setting's. Its instructions say so; its tools read a model whole and propose one, a removal being its withdrawal; the proposal's card and history say what changes in a person's terms. `temper ctl`'s settings file form takes the kind too.

Acceptance:

- `settings` with a member reads `model 4` whole, and `propose_settings`' schema's kind enum holds `model`; a proposal setting a model's prices and shown name round-trips through the settings form, a note-backed proposal included, and an accept makes the revision.
- `the_settings_form_round_trips_a_model`, `a_model_item_reads_as_what_it_changes` on the card: "GPT-6.1 Terra: on the picker", "off the picker", "new prices", "added", "withdrawn". The card's history row shows what it replaced.
- `temper-engine-smith/src/settings.rs`'s `models` passes the catalogue with its provider, shown name and picker, not `None` and `true`.
- `temper ctl settings` lists `model N`, by name, and reads and writes one as a file, in `temper-client-tools/src/jig/settings_file.rs`.
- `configuring.txt` says: you may propose a model's window, output, efforts, default effort, prices and shown name, whether the picker shows it, and its removal; a default of a role is its charter's order, and a person's switch is on Settings, Models; a task keeps the model it has; a model's a setting like the others, and the offer still bounds what you mention. Its words are the person's terms, as the file's are, and the file stays within its bound.
- `stock_brief.rs`'s figures that follow the longer instructions are regenerated, and a stock figure that changes for another reason is a question.
- Focused tests of each package changed pass. A real-model proof isn't supplied: say so.

## Inputs

- `temper/docs/design/domain/agent.md`, 8.1; `temper/docs/design/protocol/configuration.md`, 13.4, *A model of the catalogue*.
- `jig/docs/design/domain/engine.md`, 17.11.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files. `model-surface` gave these plain arms for the kind: this task reads them again.

Readers: `temper/crates/temper-client-view/src/words.rs` for the card's words; `temper/crates/temper-client-view/src/tests/cards.rs`, `temper/crates/temper-client-tools/src/tests/jig/settings.rs`, `temper/tests/client/tests/suite/{configuring_chat,settings_tools}.rs`, `temper/tests/engine/domain/tests/suite/configuring.rs`; goldens of `temper-client-tools/src/tests/golden/settings_*`, regenerated; `temper/docs/help/reference/settings-fields.md` by its writer.

Options, each pre-approved:

- The card's hunk in `cards/proposal.rs` beside `ui-proposal-card`'s: keep to `setting_base` and one `Proposed` arm, and regenerate its goldens at catch-up.
- A model item's summary with several fields changed says "new settings", naming the model.

## Steps

A test each, breaking its rule once by hand, as named in the acceptance. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents. Commit and push.

## Report

The instructions' lines added, and the figures they moved. That no real model has read them.

## Rebase (2026-11-07, step merge, foreman model-catalogue-2/configuring-learns/foreman-1)

rebase by preserving both catalogue fixture behaviors and recalculating combined stock brief figures
The merge conflicts in four paths; resolve them without choosing either side wholesale.
In `temper/tests/engine/domain/src/protocol.rs`, combine this task’s changes with `33bb531deb` (“Preserve catalogue updates in the joined client world”):
- Keep `models` for numbered execution positions, `catalogue` for whole settings-note decoding, and `catalogues` for queued client updates.
- On `PartyRequest::Catalogue`, clone into `self.catalogue`, rebuild execution models by borrowing entries, then push the owned catalogue into `self.catalogues`.
- Decode `SettingsNote` with `self.catalogue`; retain one scoped `too_many_lines` expectation.
Preserve the corresponding client forwarding loop in `temper/tests/client/src/front.rs`.
The other conflicts overlap `9e4f73d894` (“Tell settings readers which member their kind needs”), which lengthened the settings schemas:
- `temper/crates/temper-service/src/limits.rs`
- `temper/crates/temper-service/src/tests/limits.rs`
- `temper/crates/temper-service/src/tests/stock_brief.rs`
Retain both main’s explicit required-member descriptions, including model, and this task’s configuring instructions.
Recalculate the stock figures from the combined code using `every_stock_charter_admits_its_whole_maximal_brief`; neither side’s existing figures cover both changes.
Update charter 3’s measured row, the room assertion, `STANDARD_BRIEF`’s comment and `brief_bytes`, and the matching brief/journal expectations.
Verify with the focused stock admission, whole-spec receiving-bound and standard-profile tests, then the task’s model proposal tests, including note-backed acceptance.
Report the regenerated figures; keep budgets and bounds unchanged.
