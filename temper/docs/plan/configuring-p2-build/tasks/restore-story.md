+++
id = "restore-story"
part = "restore"
kind = "feature"
needs = ["restore-request", "restore-forge", "restore-connector", "restore-ctl", "policy-story"]
files = ["jig/tests/core/tests/suite/settings_restore.rs", "jig/tests/core/tests/suite/main.rs", "jig/tests/core/tests/settings/support.rs", "temper/tests/engine/domain/tests/suite/settings/kinds.rs", "temper/tests/engine/domain/src/kinds.rs", "temper/tests/client/tests/suite/settings_tools.rs", "temper/tests/client/src/front.rs", "temper/tests/client/src/fixtures.rs", "jig/docs/design/domain/engine.status.md", "jig/docs/design/domain/connectors.status.md", "temper/docs/design/client/tools.status.md"]
effort = "high"
minutes = 120
+++

# restore-story: restoring a version, told end to end (the walk)

## Goal

The gate's worlds tell the walk's story (jig's `domain/engine.md`, 17.10; `domain/
connectors.md`, 17; temper's `client/tools.md`, 5.2 and 5.3): a revision's settings restored
as a proposal, through jig's core on its real store, an application's kinds through
`jig-test-connector`, temper's kinds through the root and the forge, and every kind through
temper ctl joined to the engine root. It closes the status entries the earlier tasks left.

Acceptance:

- jig's core world, a new `jig/tests/core/tests/suite/settings_restore.rs` (the old branch's
  stories, quoted in Inputs, with this design's additions), each one thread, one loop, its
  seed replaying, through `settings_support`'s helpers and a `restore` helper beside
  `propose`:
  - `restoring_a_revision_proposes_what_later_revisions_replaced`: revision 2 sets charter 1
    and removes template 2; revision 3 sets charter 1 again and resets template 2; `Restore {
    revision: 2 }` answers `Outcome::Restored { proposal, to: 0, left_out: [] }`; the review's
    items: charter 1 *set* to revision 2's value, template 2 *removed*; the reason begins
    `restores revision 2: `; accepted, the settings are as revision 2 left them.
  - `a_restore_reads_every_page_of_the_history_after_r`: more revisions after R than one
    `revisions` page holds (the page bound lowered in the story's own limits); the proposal
    holds the earliest replaced value of each setting, whatever page held it.
  - `a_restore_of_a_forgotten_revision_names_the_oldest_kept`: `start(2)`, four revisions;
    restoring 1 refused `people::Refusal::Forgotten { oldest }`, `oldest` the history's.
  - `a_restore_of_the_newest_is_empty`: refused `Malformed::SettingsUnchanged`.
  - `a_revision_made_while_a_restore_loads_refuses_it_stale`: a change committed while the
    restore's pages load (their load answers held back, per the option); refused
    `Refusal::Stale { newest }`, nothing kept.
  - `a_restore_past_the_whole_set_is_refused_too_large`: settings added since R past a whole
    set, or two projects' policies set since R; refused `Malformed::Settings { fault:
    SettingFault::Count, .. }` naming the first item past the bound.
  - `a_restore_proposes_a_project_policy_it_replaced`: project 1's policy set after R;
    restored, accepted by the owner of both, the project's policy is R's again.
  - `a_restore_of_an_applications_kind_takes_its_connectors_kept_value`: with
    `application_settings.rs`' kind (`jig-test-connector`), revisions 2 and 3 set its value;
    restoring 2 proposes it *set*, the connector keeping the proposal's item from the value it
    kept as replaced by 3; accepted, `world.domain.setting(..)` reads revision 2's value.
  - `a_restore_leaves_out_and_names_a_deleted_projects_items`: a project's application item
    and its policy changed after R, the project deleted; the answer's `left_out` names both,
    the proposal holds neither.
  - `forgetting_a_revision_erases_what_its_items_replaced_at_the_connector`: the history
    erases revision 2's record; the connector's replaced rows of revision 2 are gone, revision
    3's kept.
  - `a_pending_restore_survives_a_restart_and_is_accepted`: a restore's proposal pending,
    `restart`; its review the same, and its acceptance commits.
- temper's domain world (`suite/settings/kinds.rs`, helpers in `src/kinds.rs`):
  - `a_landing_rule_and_the_forges_charter_changed_then_restored`: revision 2 sets landing rule
    2 and forge charter 1 to 2, revision 3 changes both again; restoring 2 makes a proposal the
    forge keeps from its `SettingSlot::Replaced(3)` rows (`kept(.., SettingSlot::Proposal(key))`
    holds revision 2's rule and charter); accepted, the `Change` rows hold revision 2's values
    and the next merge is judged by revision 2's rule, as
    `a_landing_rule_changed_reaches_the_next_merge` judges one.
  - `a_restore_leaves_out_a_deleted_projects_rule`: a project's rule (kind 2) changed after R,
    the project deleted (as `a_deleted_project_loses_its_rules` deletes one); `left_out` names
    `Setting::Application { kind: 2, scope: P, number }`.
  - `forgetting_erases_the_forges_replaced_rows`: with the world's small history, the rows of a
    revision past it are erased, the newer kept.
- temper ctl joined to the engine root (`temper/tests/client/tests/suite/settings_tools.rs`,
  `policy-story`'s file), with temper's kinds configured:
  `each_kind_through_ctl_proposed_shown_accepted_and_restored`: one settings file setting
  charter 1, template 1, policy 1, the rules, landing rule 1, forge charter 1 and project 1's
  policy, each from `settings show ... --json` with one member changed; `settings propose`,
  `proposal show` (every item, each value whole), `proposal decide ... accept` (revision 2);
  `revision show 2` (each item with its `was:` line); `revision restore 1 TEXT --key K`
  (`proposed restoring revision 1 as proposal p1/N to role 0`); `proposal show` and `proposal
  decide ... accept` (revision 3); then each `settings show ... --json` and `project show 1`
  print revision 1's values again. And `revision restore` of a forgotten revision exits 2 with
  tools.md's words.
- Status, each **Not yet** text removed whole where an earlier task has not:
  - `jig/docs/design/domain/engine.status.md`: "17.5, 17.10: a connector keeps no value its
    items replaced, beside its changes (`jig-test-connector`'s `settings.rs`, `keep`)." and
    what is left of "17.10: restoring a version: no `Restore` request, and a revision's record
    keeps no item's replaced value (settings-v1's `Revision` and `Item`)."
  - `jig/docs/design/domain/connectors.status.md`: "17: what its items replaced, for a
    restore: no ask hands a connector a restore's items, nor erases what a forgotten
    revision's items replaced, and `jig-test-connector` keeps no replaced value
    (`jig-domain-core/src/effects/connector.rs`, `Ask`)."
  - `temper/docs/design/client/tools.status.md`: "5.3, beside its acts above: `revision
    restore`'s items of temper's kinds and of a project's policy."
- The narrow runs below pass with their counts, then the three worlds' focused suites.

## Inputs

- `jig/docs/design/domain/engine.md`, 17.5 ("History"), 17.7, 17.10; `jig/docs/design/domain/
  connectors.md`, 17; `jig/docs/design/domain/people.md`, 5.1 and 5.2; jig's
  `jig/docs/design/domain/testing.md`.
- temper's `temper/docs/design/protocol/configuration.md`, 13.2; `temper/docs/design/client/
  tools.md`, 5.2, 5.3 and 11.2; `temper/docs/design/domain/testing.md`, 6.3 and 6.4.
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B2 to B5; `notes/common.md`.
- The specs this builds on: `restore-request.md`, `restore-connector.md`, `restore-forge.md`,
  `restore-ctl.md`, `policy-story.md` (its `settings_tools.rs` and `give_with_file`).
- Old stories, jig's `configuring-revisions/restore` (unreachable from your slot, quoted from
  `temper/docs/plan/configuring-p2/notes/next-cut.md`), in `jig/tests/core/tests/
  settings_proposals.rs`: `restoring_a_revision_proposes_what_later_revisions_replaced`,
  `a_restore_reads_every_page_of_the_history_after_r`,
  `a_restore_of_a_forgotten_revision_names_the_oldest_kept`, `a_restore_of_the_newest_is_empty`,
  `a_revision_made_while_a_restore_loads_refuses_it_stale`,
  `a_restore_past_the_whole_set_is_refused_too_large`. They departed from 17.10: no
  application's items (`left_out` always empty), no project's policy, no deleted project's
  items, too large counted on `Set` items alone; these stories hold all four.

## Owns and may change

- Owned: the new `settings_restore.rs` and its `mod` line; the stories added to
  `temper/tests/engine/domain/tests/suite/settings/kinds.rs` and
  `temper/tests/client/tests/suite/settings_tools.rs`; the status texts above.
- Neighbour files, helpers alone, no existing helper's behaviour changed:
  `jig/tests/core/tests/settings/support.rs` (`restore(world, signed, key, revision, reason)
  -> people::Reply`, `restored(&reply) -> (u64, Box<[settings::Setting]>)`);
  `temper/tests/engine/domain/src/kinds.rs` (`restore(sign_in, revision, key) ->
  engine::Event`, a `PartyInput::Ask` with `people::Ask::Restore`, and `restored` reading
  the answer from `driver.delivered`); `temper/tests/client/src/front.rs` (one constructor,
  `Joint::with_kinds(seed, limits)`, `Joint::new` with the engine configured as
  `temper_engine_domain_world::kinds::configuration` configures it) and
  `temper/tests/client/src/fixtures.rs` (`kinds_world(seed)`).
- Options, pre-approved:
  - **Holding a page's load** (the stale story): if the core world can deliver a change's
    request before the store answers the restore's page loads (the world's own event order, or
    a store fault it already schedules), use that; if it answers the loads within the step
    that asks them, so that no change can come between, keep the case to `restore-request`'s
    unit test, drop it here (ten stories) and say so.
  - **The page bound** (every page): lower `revisions`' page rows in the story's own limits;
    if no limit sets it, make as many revisions as one default page holds plus one, and say how
    long the story takes.
  - **The client world's kinds:** if `Joint::with_kinds` needs more than the configuration
    (the forge's stock, a landing default), give it what `kinds::configuration` gives the
    domain world, nothing else; if the joined world cannot carry temper's kinds without
    changing a shared constructor, tell temper's kinds through ctl in the domain world's
    stories alone, keep the ctl story to jig's kinds and a project's policy, and say so.
  - **Why not the one-process world:** temper's `domain/testing.md`, 6.4 gives the one-process
    world no new settings story, and tools.md 11.4 keeps settings proposals and decisions to
    the step tests and the core and joined worlds: the ctl story is the joined client
    world's, as 11.2 makes it ("Each area adds its stories").
  - If a story finds the code departs from the design, keep the story as the design says, mark
    it `#[ignore = "..."]` naming the departure, and report it (never change the design to
    fit). If a story needs larger limits, raise them in its own setup, never a world's
    defaults.
  - **Status:** an entry an earlier task already removed is left as it is; an entry whose
    gap a story shows still open stays, and the report says which.

## Steps

1. The helpers named above.
2. The core world's stories, one at a time: `heavy cargo nextest run -p jig-core-world --test
   suite -E 'test(/settings_restore::/)'` (check the count: eleven).
3. The domain world's: `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/settings::kinds::/)'` (check the count: the six there plus three).
4. The joined world's: `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(/settings_tools::/)'` (check the count: two).
5. The status texts; `python3 -I -B tools/citations/citations.py --base origin/main`.
6. The worlds' focused suites: `heavy cargo nextest run -p jig-core-world --test suite -E
   'test(/settings/)'`, `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/settings/)'`, `heavy cargo nextest run -p temper-client-world --test suite`; and
   `jig/tests/test-connector`'s package's suite, which the connector's records reach.
7. `cargo fmt`; `heavy cargo clippy -p jig-core-world -p temper-engine-domain-world -p
   temper-client-world --all-targets -- -D warnings`. Commit ("Tell restoring a version end
   to end") and push.

## Report

The implementer's text's report, plus: the options taken (the held load, the page bound, the
client world's kinds); any story ignored, and why; the status texts removed and any left; each
story's time from the run's log.
