+++
id = "domain-configuring"
part = "domain-runs"
kind = "migration"
needs = []
files = ["temper/tests/engine/domain/tests/suite/configuring.rs", "temper/tests/engine/domain/tests/suite/configuring/**"]
effort = "high"
minutes = 300
+++

# domain-configuring: the engine's configuring stories run Ask temper

## Goal

The engine's domain world's configuring stories (`suite::configuring`) run charter 11
from template 5, reading through `see` and writing the chat's document through
`document`, with what each checks unchanged; the stories of the settings family's own
tools, which no stock charter will be declared, run a charter of the world's own. After
this task no test in `suite::configuring` starts template 3 or runs charter 3, so that
the task `stock` can remove them.

Acceptance:

- No `CONFIGURING` constant, no `world.chat(3, ..)`, no `charter == 3` and no call of
  the `settings`, `read_docs`, `read_file`, `read_branch` or `note` tool by a charter-11
  run in `temper/tests/engine/domain/tests/suite/configuring.rs` or
  `configuring/*.rs`; every story there that ran charter 3 runs charter 11 or the
  world's settings charter, by the table below.
- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/^configuring::/)'`
  passes, with as many tests as before but those the table deletes, and the run's CPU
  time no more than 10% above the base's (measure both; report the figures).
- The world's settings charter is the contract's (`notes/contracts.md`, *What stays*).

## Inputs

- `temper/docs/plan/ui-ask-drop/notes/common.md` and `notes/contracts.md` (binding).
- `temper/docs/design/domain/testing.md`, 6.4: the two rows "configuring by Ask temper"
  and "Ask temper's configuring reads".
- `temper/docs/design/domain/agent.md`, 8.4.1 (`see`'s addresses and parts), 8.4.2
  (`propose_settings`' asking form: inline items, `more`, approvals at the call), 8.4.3
  (`document`).
- `temper/docs/design/draft/ask-temper.md`, 5.1 and section 9.

## Owns and may change

- Owns `temper/tests/engine/domain/tests/suite/configuring.rs` and
  `temper/tests/engine/domain/tests/suite/configuring/` (`fixture.rs`, `asking_forge.rs`,
  `asking_proposal.rs`, and a new module there if the helpers want one, with its `mod`
  line).
- Not yours: `suite/settings.rs`, `suite/settings/**`, `suite/forge_routes/**` and
  `temper/tests/engine/domain/src/**` (the task `domain-settings` writes them in
  parallel). If a helper in `src/` must change, don't: copy what you need into
  `configuring/fixture.rs`, and say so in the report.
- Never `temper/crates/**` (charter 3 stays in the stock until `stock`).
- Options, pre-approved:
  - `the_configuring_agent_speaks_from_the_inventory_offer` asserts a sentence. If the
    sentence is `configuring.txt`'s, delete the test (its rule goes with charter 3); if
    it is the inventory's answer or a tool's description, assert it on charter 11's
    `see settings` answer. Say which.
  - A story whose subject is `see`'s own bound or parts (`an_asking_chat_reads_each_address`)
    keeps its assertions; its hidden read through charter 3 (`read_value` of policy 1
    near line 1213) becomes the world settings charter's read or the typed value from
    `world.driver`, whichever the comparison needs; default: the settings charter.
  - If a check needs a `see` address that 8.4.1 has no form for (an old revision's one
    setting, another project's scope), that check moves to the settings charter's story
    of the same subject; if none, say so in the report and keep the rest.

## Steps

1. **Fixture** (`configuring/fixture.rs`): replace `CONFIGURING` with
   `pub(super) const ASK_CHARTER: u32 = 11;` and `pub(super) const ASK_TEMPLATE: u32 = 5;`,
   the template where `world.chat(..)` takes one, the charter where a run's `charter` is
   asserted. Add `pub(super) const SETTINGS_CHARTER: u32 = 12;`,
   `pub(super) const SETTINGS_TEMPLATE: u32 = 12;` and
   `pub(super) fn settings_config() -> engine::Config`, built as `moving_config` builds
   its removable charter: `config()` plus a copy of stock charter 1 numbered 12, its
   `tools` `jig_domain_tasks::Tools(1920)`, `modify` and `agents` false, instructions
   `b"Read and propose settings."`; a copy of template 1 numbered 12, named `settings`,
   run by charter 12; `Executor::Charter(12)` offered by the rules' ceiling, policy 1's
   ceiling and each role that delegates, as `offer_removable` does. Keep `CHARTERS`,
   `EXECUTORS` and the other limits as they are unless a story fails for room; if it
   does, raise by the one it needs and say so.
2. **Readers:** keep `read_charter`, `read_value` and `read_whole` (the `settings`
   tool's offsets) for the settings charter's stories, renamed `settings_charter`,
   `settings_value` and `settings_whole`; add the asking readers beside them:

   ```rust
   /// A `see` answer whole, following `more: TOKEN` (agent.md, 8.4.1); the parts it took.
   pub(super) fn see_whole(driver: &mut Driver, run: &engine::Assignment, completion: u32, address: &str) -> (String, u32);
   /// A `see settings KIND N` answer's JSON value, its heading line dropped.
   pub(super) fn see_value(driver: &mut Driver, run: &engine::Assignment, completion: u32, address: &str) -> Value;
   pub(super) fn see_charter(driver: &mut Driver, run: &engine::Assignment, completion: u32, number: u32) -> Value;
   ```

   `see_whole` follows `an_asking_chat_reads_each_address`'s loop (the part up to
   `more: ` or the `see: ` lines, the token's `@offset`), asserting each part within
   `temper_engine_smith::SEE_PART` and the host reply bound, and that a read commits
   nothing.
3. **Stories, by class** (line numbers are today's):

   | Story | Runs | Change |
   |---|---|---|
   | `a_box_person_known_by_the_forges_link_approves_with_nothing_added` (229), `an_inventory_names_its_own_projects_people` (348), `an_inventory_names_the_payers_usable_claude_offer` (566), `an_inventory_says_when_no_account_can_run_a_model` (582), `settings_and_revision_reads_carry_no_model_offer` (592), `a_members_chat_reads_the_same_people` (622), `a_forge_person_requires_owned_mode_and_a_login` (638), `a_configuring_chat_discovers_the_forge_host_for_charter_documents` (654), `an_owners_configuring_chat_proposes_and_hears_it_accepted` (694), `a_proposal_selects_the_template_it_installs` (732), `a_project_with_no_ways_reads_at_their_stock` (834), `a_projects_stored_ways_read_as_stored` (857), `a_members_proposal_waits_for_an_owner` (899), `a_stale_proposal_is_withdrawn_read_again_and_proposed_afresh` (960), `a_rejected_proposal_is_revised` (983), `an_accepted_proposal_moves_running_work` (1016), `the_configuring_agent_reads_each_kind` (1067), `propose_settings_refuses_a_stale_revision_and_an_item_out_of_shape` (1231), `a_chats_fresh_proposal_replaces_its_own_that_waits` (1703), `a_chats_refused_fresh_proposal_leaves_its_own_waiting` (1772), `no_answer_a_configuring_run_hears_says_nothing_changed` (1800), the inline half of `a_member_reads_and_proposes_a_whole_model_inline_or_from_a_note` (1852) | charter 11, template 5 | reads by `see` (`settings`, `settings KIND N`, `settings project-ways`, `settings project-questions`, `revision N`, `revisions`, `help NAME`, `file OWNER/NAME PATH`); the document by `document`; proposals in the asking form, inline; a declaration assertion names `see`, `document` and `propose_settings`; `ask.txt` where the story read `configuring.txt`; names saying "configuring" say "asking" (`an_owners_asking_chat_...`, `the_asking_chat_reads_each_kind`, `no_answer_an_asking_run_hears_...`) |
   | `a_read_that_finds_nothing_says_it_was_not_read` (876), `settings_and_named_documents_use_the_host_answer_allowance` (1300), `settings_note_sources_refuse_moved_missing_private_and_malformed_values` (1640), `a_decided_note_proposal_replays_after_its_note_moves` (1665), the note half of `a_member_reads_and_proposes_a_whole_model_inline_or_from_a_note` | the settings charter, template 12, `settings_config()` | unchanged but for the charter, template and config; `a_read_that_finds_nothing_...` also gains a row for charter 11's `see settings charter 999`, answered not read |
   | `a_configuring_chat_proposes_large_settings_from_a_note` (1450) | both | split in two: `a_settings_charter_proposes_large_settings_from_a_note` (the settings charter, as today) and the asking half folded into `an_asking_chat_proposes_a_large_set_in_parts` as a row if it adds a case, else dropped; say which |
   | `direct_ways_are_checked_against_the_templates_the_decision_leaves` (776) | no run | the stock template it copies is 5, its references 3 become 5; custom 4 stays |
   | `the_configuring_agent_speaks_from_the_inventory_offer` (614) | | the option above |
   | `an_asking_chat_reads_each_address` (1120), `an_asking_chat_names_a_repository_as_the_inventory_does` (1938), `an_asking_chat_is_refused_an_unsatisfiable_approval_at_the_call` (2105) | charter 11 | their helper reads become `see_*`; 1213's comparison by the option above |
   | `an_asking_chat_proposes_a_large_set_in_parts` (1967) | charter 11; the settings charter at 2061 | charters read are 1, 2 and 4, not 1..=3; `Charter(3)` expected becomes `Charter(4)`; the run at ~2061 that is refused `more` ("more is not offered to this charter") is the settings charter's |
   | `a_chat_is_offered_no_settings_tools` (1265) | charter 1 | unchanged |
   | `configuring/asking_forge.rs`, `an_asking_chat_reads_the_forge` | charter 11 | the charter-3 `read_branch` at ~36, there only to learn the head, takes the head from the fixture's fake forge or a charter-1 chat's read |
   | `configuring/asking_proposal.rs`, `an_asking_chat_reads_a_proposal` | charter 11 | the producer chat at ~12 is template 5 |

4. Run the module's stories one by one while moving them
   (`heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(=configuring::NAME)'`),
   then the whole module once, then `cargo fmt`, `heavy cargo clippy -p
   temper-engine-domain-world --all-targets -- -D warnings`, and
   `python3 -I -B temper/scripts/old_forms.py`. A moved story needs no bite check (it
   moves); the new `see settings charter 999` row does: break `see`'s not-read answer's
   check by hand once, see the row fail, restore.
5. Commit ("Run the engine's configuring stories as Ask temper") and push.

## Report

The table above with each story's outcome (moved, split, deleted, unchanged) and its
new name; the option taken for each pre-approved choice; the module's test count and CPU
seconds before and after; any check that had no `see` form and where it went; the
helpers you copied from `src/`.

## Answers

2026-11-08 (temper/ui). A for the refusals. Charter 11 refuses an unsatisfiable approval at the call, so the story observes that refusal at propose_settings, keeping its assertions on eligibility, reasons, no settings change, the linked owner's acceptance and a maintainer's fresh proposal. Not B: there's no charter 12. For the nine instruction rules, don't drop what they guard: move it into Ask temper's instructions. Add this paragraph, as written, at the end of ask.txt's '## Configuring' section, before '## Your voice':

When a person asks to approve changes themselves, propose their own role: owner (0) for the project's owner. Read the inventory first. On a box, an owner whose account the repository's forge already knows needs nothing added. The box's own forge account is not a person, and the project, not the person, asks for the approval. Propose only a role the project has verified, after reading the project policy's current permission mapping. A forge admin can seed maintainer (1), never owner (0), so never promise that an admin will make a role-0 proposal work. After a repository is adopted again, read the inventory again and propose afresh.

Then the story's loop checks these fragments of ask.txt in place of the nine: 'propose their own role: owner (0)', 'Read the inventory first', 'needs nothing added', 'is not a person', 'the project, not the person, asks for the approval', 'current permission mapping', 'maintainer (1), never owner (0)', 'never promise that an admin', 'read the inventory again and propose afresh'. Keep the bound task's 16,384-byte check green; this adds about 700 bytes. Name the instruction change in the report.
