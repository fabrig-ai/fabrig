# restore-stories: the restart findings

2026-11-08. `a_restore_leaves_out_a_deleted_projects_rule`, in
`temper/tests/engine/domain/tests/suite/settings/kinds.rs`, remains ignored
under this plan's missing-feature exception (`common.md`).

`each_kind_through_ctl_proposed_shown_accepted_and_restored`, in
`temper/tests/client/tests/suite/settings_tools.rs`, also remains ignored
under that exception. Both stories reach restore and acceptance; their
restart checks expose the missing default-state lifecycle described below.

The missing behavior is in `jig/docs/design/domain/engine.md`, 17.5 and
17.10: restoring an application setting that was not offered resets it;
the retained state still occupies its kind's slot, including after restart.

The story passes its restore, omitted-project, proposal and acceptance
assertions. Its restart oracle then reports that application kind 1,
scope 0, number 2 had `State { follows: Default, size: 0, charters: [] }`
before restart, and no row afterward. The first isolated run failed in
0.041 seconds. During a trial of a shared-project-default repair,
an explicit ignored-only run still failed for this reason in 0.026 seconds.

The cause is jig's `settings/application.rs`, `prepare_start`: it treats
every default-following state omitted from the connector's supplied
defaults as a removed default, erases the state and makes a release
revision. A temporary isolated regression in jig's
`src/tests/settings/restore.rs` reproduced the missing state directly
(0.016 seconds); no jig change is retained on this branch.

Skipping that erasure for zero-size states made the regression and this
story pass, but cannot distinguish the reset from a valid empty application
default with no charter references. Dropping that default must still make
the release revision required by section 17.5. Both cases have the same
stored `State` and an empty supplied-default list at restart. The proposed
shortcut was removed rather than suppressing that release change.

The retained application state or the connector's default-reporting
contract needs enough information to distinguish these cases. This task
forbids stored-format and protocol changes, so it leaves the story's
assertions intact, updates its ignore reason, and defers that feature.
Jig's eleven ignored world restore stories are unchanged.

The client story's missing behavior is in
`temper/docs/design/protocol/configuration.md`, 13.2 and 13.4, and
`jig/docs/design/domain/engine.md`, 17.5: the shared landing default must
retain each restored project's default-following setting state at restart.
Its oracle reports kind 2, scope 1, number 1 present before restart with
size 36 and absent afterward. The final explicit ignored-only run failed
in 0.118 seconds. The forge omits the shared kind-2 stock (scope 0) from
the defaults it reports to jig, whose kind 2 has project scope.

Reporting that shared default once per live project made the client story
pass, but introduced metadata for untouched projects at restart. A focused
run of the changed packages and their direct dependents then observed
59 failures among 5,157 tests: restart oracles saw new setting rows or
release revisions, and the resulting history changes erased retained
revision rows. That trial and its new test were removed. This needs the
whole project-default lifecycle, including projects made after first
start, rather than a projection added only at restart. The existing story
keeps its assertions, including all restored values and the project policy,
and names the missing feature in its ignore reason.

The client fixture's template grace is within the task bound, and its
review assertions follow `temper/docs/design/client/tools.md`, 5.2:
`revision show` prints a whole set value before `was:`, and a restored
application default is reviewed as `would be default`. Those corrections
remain so the ignored story reaches the missing feature directly.
