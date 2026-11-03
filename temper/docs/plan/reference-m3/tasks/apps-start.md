+++
id = "apps-start"
part = "appsettings"
kind = "feature"
needs = ["apps-proposals"]
files = ["jig/crates/jig-domain-core/src/settings/restart.rs", "jig/crates/jig-domain-core/src/settings/application.rs", "jig/crates/jig-domain-core/src/start/settings.rs", "jig/crates/jig-domain-core/src/start/first.rs", "jig/crates/jig-domain-core/src/effects.rs", "jig/crates/jig-domain-core/src/admission/deletion.rs", "jig/crates/jig-domain-core/src/tests/settings/application_start.rs", "jig/crates/jig-domain-core/src/tests/settings.rs", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 110
+++

# apps-start: an application's defaults at each start, and its states restored

## Goal

At each start the core waits for each kind-owning connector's `SettingsDefaults`, restores
its stored states of an application's settings, refuses a start whose stored states or
defaults break their kinds, makes the release's revision when a changed default no change
is over changes the current settings, and tells each such connector `SaveSettingsDefaults`
in that decision (or with no revision when none was needed). A project's deletion erases the
core's states of its project-scoped settings.

Acceptance:

- The unit tests below pass; `jig-domain-core`'s focused tests and `jig/tests/core`'s
  start-settings and first-start stories pass unchanged (a change of meaning of a start:
  run them).
- The **Not yet** of `jig/docs/design/domain/engine.status.md` loses its last part, "an
  application's kinds in ... 17.7: jig's settings are closed (...), so an application
  declares no kind of its own, and no connector judges or keeps one." whole.

## Inputs

- `jig/docs/design/domain/engine.md`, 17.5 (**At each start**, **Charters a value
  names**), 17.7 (**A start**), section 6, steps 1 and 2.
- `jig/docs/design/domain/connectors.md`, section 17 (**Its defaults at each start**).
- `jig/docs/design/domain/people.md`, 5.2 (a project's deletion).
- `temper/docs/plan/reference-m3/notes/contracts.md`, C2 (its start and `ReleaseProject`
  bullets).
- `temper/docs/plan/reference-m3/notes/common.md`.

## Owns and may change

- Owned: the `files` above; `crate::start::Failure` (in `jig-domain-core/src/start*`) gains
  the arms below, with the compiler-forced arms where it is matched (a root's wording of a
  start refusal: `jig-test-domain`, `jig/examples/ops`, temper's root, the arm alone).
- Options, pre-approved:
  - If a connector's `SettingsDefaults` can arrive before the core's settings restore
    begins (the root may ask its connectors in any order), hold it in the start's state
    and apply it once the settings are read (default); if the core's start has no state
    to hold it, ask.
  - At a first start (no commits), the defaults' states join revision 1, no release
    revision; `SaveSettingsDefaults { revision: Some(1) }` (default).

## Steps

1. **Waiting:** a start with declared kinds waits, before its release decision, for one
   `connector::Event::SettingsDefaults` from each connector a kind names (the set of
   `Kind.connector`). The core's states of settings following their default take their
   `size` and `charters` from it; a `DefaultState` of a kind not declared, a scope not its
   kind's, past `per_scope`, or with more `charters` than its kind declares refuses the
   start: `start::Failure::Application { kind, scope, number, why: ApplicationWhy }`,
   `ApplicationWhy { Undeclared, Scope, Count, Shape }`.
2. **Restored states:** `Record::Application` restores into `Current.applications` before
   the defaults arrive; a stored state of a kind no longer declared, or past its kind's
   `per_scope`, refuses the start the same way (`Undeclared`, `Count`).
3. **Reference:** the start's whole check (17.1) holds the charters every state names,
   defaults' and changes': a charter not offered refuses the start as *reference*, naming
   the application's setting.
4. **Release:** a `DefaultState { differs: true, .. }` whose setting follows its default
   counts as a changed default, as a changed jig default does in
   `finish_restored_settings`; the release's revision records each in `applications` as a
   `SizedEdit::Set` (or a `Remove` when the default is gone: a state following a default the
   connector no longer reports is erased, and counts too). In the decision that commits
   the start's settings, each kind-owning connector gets
   `Ask::Effect(connector::Ask::SaveSettingsDefaults { revision })`, `Some` of the release's
   (or first start's) revision, `None` when no revision was made.
5. **Deletion:** the decision that erases a deleted project (`admission/deletion.rs`, with
   `ReleaseProject`) erases the core's states of that project's scope (their
   `Key::Changed` records deleted). The connector erases its own on `ReleaseProject`.
6. **Tests** (`src/tests/settings/application_start.rs`, the scripted connector of
   `apps-change`): a first start with one kind's defaults (states sized, revision 1, one
   `SaveSettingsDefaults(Some(1))`); a restart with no difference (no revision,
   `SaveSettingsDefaults(None)`); a restart with a changed default no change is over (a
   release revision recording it, and its save); one under a change (no revision); a
   stored state of an undeclared kind refused; a default naming an unoffered charter
   refused as *reference*; a project's deletion erasing its states.
7. Narrow tests: `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/settings/)'`
   and `-E 'test(/start/)'`; `heavy cargo nextest run -p jig-core-world --test suite -E
   'test(/start_settings|first_start/)'` (check the counts). `cargo fmt`; `heavy cargo
   clippy --workspace --all-targets`. Edit the status entry. Commit and push.

## Report

The options taken; where the defaults wait in the start; the start refusal's arms.
