+++
id = "restore-kept"
part = "restore-kept"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/effects/connector.rs", "jig/crates/jig-domain-core/src/settings/application.rs", "jig/crates/jig-domain-core/src/tests/settings/application_start.rs", "jig/testing/jig-test-connector/src/boundary.rs", "jig/testing/jig-test-connector/src/settings.rs", "jig/testing/jig-test-connector/src/tests/settings.rs", "jig/testing/jig-test-domain/src/routes.rs", "temper/crates/temper-engine-domain-forge/src/kinds.rs", "temper/crates/temper-engine-domain-forge/src/tests/kinds.rs", "temper/crates/temper-engine-domain/src/restart.rs", "temper/tests/engine/domain/tests/suite/settings/kinds.rs", "temper/tests/client/tests/suite/settings_tools.rs", "jig/docs/design/domain/engine.status.md", "jig/docs/design/domain/connectors.status.md", "temper/docs/design/protocol/configuration.status.md", "temper/docs/design/client/tools.status.md"]
effort = "medium"
minutes = 150
+++

## Goal

A setting a restore resets to a default its connector does not report
loses its retained state at the next start: jig's start erases every
default-following state the connectors' defaults leave out, with a
removal in the release's revision. The design now has each connector
name the defaults it stored that the application no longer gives, and
the core erases only a state following one of those. Build that
contract on both sides, so a reset's state survives a restart, and the
two restore stories run again.

Acceptance:

- `jig_domain_core::effects::connector::Event::SettingsDefaults` carries
  the dropped names beside the defaults (sketch below), and
  `settings::application::prepare_start` removes, erases and records as
  a release removal only a `Follows::Default` state that some
  connector's `dropped` names. A `Follows::Default` state that no reply
  reports or names keeps its state, size and slot, with no edit and no
  write.
- jig-domain-core's lib tests (`tests::settings::application_start`):
  - `a_default_no_longer_reported_is_erased_in_a_release_revision`
    becomes a default its connector names as dropped, with the same
    assertions; rename it `a_dropped_default_is_erased_in_a_release_revision`.
  - new: `a_reset_to_a_default_no_connector_reports_keeps_its_state_at_restart`:
    a setting of an application kind changed, then reset (it follows a
    default the connector never reported), then a start whose
    connector reports no default for it and drops none: its state is
    kept (follows its default, same size), no release revision is made
    for it, and nothing erases its record.
  - new: `a_dropped_name_with_a_changed_state_keeps_the_change`: a
    dropped name whose setting is changed leaves the change as it is.
  - `restart_without_a_default_difference_keeps_the_revision_and_saves_without_one`
    keeps passing.
- `jig-test-connector` reports a stored default its configuration no
  longer gives in `dropped`, never again as a zero-sized default with
  `differs` set (`src/settings.rs`, `defaults`).
- temper's forge names its dropped defaults:
  `temper_engine_domain_forge::Domain::dropped_settings_defaults`
  (sketch), read by `temper-engine-domain`'s `restart.rs`,
  `give_settings_defaults`, into the event. A forge test holds it.
- These lose their `#[ignore]` and pass, each alone first, then with
  their packages:
  - `temper/tests/engine/domain/tests/suite/settings/kinds.rs`,
    `a_restore_leaves_out_a_deleted_projects_rule` (package
    `temper-engine-domain-world`, binary `suite`, `settings::kinds::a_restore_leaves_out_a_deleted_projects_rule`).
  - `temper/tests/client/tests/suite/settings_tools.rs`,
    `each_kind_through_ctl_proposed_shown_accepted_and_restored`
    (package `temper-client-world`, binary `suite`,
    `settings_tools::each_kind_through_ctl_proposed_shown_accepted_and_restored`).
  Both reached the restart in their drivers' teardown (the engine
  domain world's `Driver::drop`, `close_erasing`, its cold restart and
  row compare, `tests/engine/domain/src/oracle.rs`), where the start
  erased the reset's state and made revision 4. That is the assertion
  that failed; their own assertions passed.
- The focused tests of jig-domain-core, jig-test-connector,
  jig-test-domain, jig-core-world, temper-engine-domain-forge,
  temper-engine-domain, temper-engine-domain-world and
  temper-client-world pass. This changes what a start keeps, so every
  package that reads `SettingsDefaults` runs (common notes, *a change of
  meaning*).
- The status entries below are gone.

## Inputs

- `temper/docs/plan/ignored-misc/notes/common.md`: every rule there
  holds.
- `jig/docs/design/domain/engine.md`, 17.5 (*At each start*: the new
  sentences on a default a connector names as no longer given, and on a
  setting following a default no connector reports or names), 17.7 and
  17.10.
- `jig/docs/design/domain/connectors.md`, section 17 (*Its defaults at
  each start*: naming each default it stored that the application no
  longer gives).
- `temper/docs/design/protocol/configuration.md`, 13.2 and 13.4 (*A
  project's rule 1 is given to the forge*: a default for no one scope,
  of which the forge tells jig nothing).
- `jig/docs/design/domain/engine.status.md`, its 17.5, 17.7, 17.10
  entry, which says where the code is now.

## Owns and may change

- Owns: the files in the header.
- Sketch, binding the outer layer:

  ```rust
  // jig/crates/jig-domain-core/src/effects/connector.rs
  /// A default a connector stored that the application no longer gives
  /// (jig/docs/design/domain/engine.md, 17.5; connectors.md, section 17).
  #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
  pub struct DroppedDefault {
      pub kind: u32,
      pub scope: u32,
      pub number: u32,
  }
  pub enum Event {
      // ...
      SettingsDefaults {
          connector: u16,
          defaults: Box<[DefaultState]>,
          dropped: Box<[DroppedDefault]>,
      },
      // ...
  }

  // jig/testing/jig-test-connector/src/boundary.rs
  SettingsDefaults { defaults: Box<[crate::SettingDefault]>, dropped: Box<[(u32, u32, u32)]> },

  // temper/crates/temper-engine-domain-forge/src/kinds.rs, beside give_settings_defaults
  /// The stored defaults the stock last given no longer holds, kind 2's
  /// every-project entries (scope 0) left out as give_settings_defaults
  /// leaves them out (temper/docs/design/protocol/configuration.md, 13.4).
  #[must_use]
  pub fn dropped_settings_defaults(&self) -> Box<[(u32, u32, u32)]>;
  ```

  The forge's method reads its `SettingSlot::Default` rows whose
  `setting_default(kind, scope, number)` is none once
  `give_settings_defaults` has taken the new stock: the rows
  `save_settings_defaults` would erase. It is called after
  `give_settings_defaults` and before anything saves. The core keeps
  each reply's `dropped` with its defaults until `prepare_start`; how
  `Starting::Waiting`'s replies hold them is yours.
- Reader files, call sites and literals only: every construction or
  pattern of `Event::SettingsDefaults` or `Request::SettingsDefaults`
  that the compiler names (among them `jig-domain-core/src/vocabulary.rs`,
  `jig-test-domain/src/admission.rs` and `src/lib.rs`,
  `temper-engine-domain/src/tests/kinds.rs`, and the core world's
  `jig/tests/core/**` fixtures). A test that builds the event gives
  `dropped: Box::new([])` unless it is about a dropped default.
- Options, each pre-approved:
  - If a core or core-world test relied on a dropped default being
    reported as a zero-sized default (the test connector's old way),
    give it the name in `dropped` instead and keep its assertions; say
    which in the report.
  - If a name appears in both a reply's `defaults` and its `dropped`,
    the core panics with a message naming the connector: the connector
    never sends both.
  - If either restore story still fails after the fix at a later
    point, read its failure. A cause in the same mechanism is yours.
    Another cause keeps that test's `#[ignore]`, its reason naming the
    design section it misses, and the report says so first.
- Not yours: any other ignored test, among them the core world's
  restore stories (`jig/tests/core/tests/suite/settings_restore.rs`),
  which stay ignored for their own reasons. No stored format changes:
  no schema under `jig-protocol-store` or `temper-protocol-store`.
- Status entries, in these words:
  - `jig/docs/design/domain/engine.status.md`: the bullet that begins
    `- **Not yet:** 17.5, 17.7, 17.10: a setting reset to a default its`
    is removed whole.
  - `jig/docs/design/domain/connectors.status.md`: the last bullet,
    which begins `- **Not yet:** 17: naming, at each start, the defaults a connector`,
    is removed whole.
  - `temper/docs/design/protocol/configuration.status.md`: in the long
    **Not yet** paragraph, the sentences from `13.2, 13.4: a setting a
    restore resets to a default the forge does not report to jig` to
    `... which the request \`ignored-engine\` of 2026-11-08 left out.`
    are removed, and the sentence after them, `The forge's own tests of
    what it keeps and restores run (...)`, too. Nothing else on that
    line changes.
  - `temper/docs/design/client/tools.status.md`, 11.2: the clause from
    `the joined story of a settings file of each kind proposed, shown,
    accepted` to `...\`../protocol/configuration.status.md\`);` is
    removed, so the sentence goes on with `the joint component
    correlates forge setting reads, ...`. Fix the punctuation around it
    and nothing else.
  - Each file's date line becomes the day you commit (`date -u +%F`).

## Steps

1. Run the two ignored stories alone with `--run-ignored only` to see
   their failure at the teardown's restart, and keep the logs out of the
   branch.
2. jig: the event's field, the core's `prepare_start`, the
   `jig-test-connector` and `jig-test-domain` sides. The core tests
   above, each made to bite (break the dropped check once, see the new
   test fail for it, restore).
3. temper: `dropped_settings_defaults` with its test in
   `temper-engine-domain-forge/src/tests/kinds.rs`
   (`dropped_defaults_name_the_stored_ones_the_stock_no_longer_gives`:
   a stored default gone from the stock is named; one still given and
   kind 2's scope 0 entry are not), and `restart.rs`'s event.
4. Remove the two `#[ignore]`s. Run each story alone, then its package.
5. The status entries.
6. Once: `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy` on
   every package you changed, `--all-targets -- -D warnings`; the focused
   tests of the packages in the acceptance.
7. Commit, run the citations check, push.

## Report

The implementer's report, with: the teardown failure each story showed
before, and that it passes after, with its time; the core's tests and
what each broke; the readers you edited; any option taken.
