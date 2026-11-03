+++
id = "apps-world"
part = "appsettings"
kind = "feature"
needs = ["apps-connector", "apps-start", "apps-store"]
files = ["jig/testing/jig-test-domain/src/routes.rs", "jig/testing/jig-test-domain/src/lib.rs", "jig/testing/jig-test-domain/src/tests/**", "jig/tests/core/tests/suite/application_settings.rs", "jig/tests/core/tests/suite/main.rs", "jig/tests/core/src/**", "jig/docs/design/domain/connectors.status.md"]
effort = "high"
minutes = 120
+++

# apps-world: an application's kind across the core and its connector

## Goal

jig's test root routes the core's settings asks to `jig-test-connector`'s kind and its
answers back, gives the connector its defaults at each start, and declares the kind to the
core when its configuration asks; a core-world story runs both real sides: changes judged
both ways and kept, a proposal kept, accepted and another withdrawn, and a restart finding a
changed default that joins the release's revision.

Acceptance:

- The story below passes in `jig-core-world`, with the restart oracle (no `broken()`
  finding); the core world's and `jig-test-domain`'s other tests pass unchanged (the root's
  routing changes meaning for the five asks: run them).
- The **Not yet** "15, 17: no connector owns a kind of the application's settings ..." of
  `jig/docs/design/domain/connectors.status.md` is removed.

## Inputs

- `jig/docs/design/domain/connectors.md`, sections 15 and 17.
- `jig/docs/design/domain/engine.md`, 17.5, 17.6, 17.7 (**A start**).
- `jig/docs/design/domain/root.md`, section 11 (the root routes, never judges).
- `temper/docs/plan/reference-m3/notes/contracts.md`, C1, C2.
- `temper/docs/plan/reference-m3/notes/common.md`.

## Owns and may change

- Owned: the `files` above.
- Neighbour: the `Config` literals `apps-surface` gave `kinds: Box::default()` in
  `jig-test-domain`'s fixtures and `jig/tests/core/src/oracle.rs`: the value only, where the
  story's configuration declares the kind.
- Options, pre-approved:
  - If the core world's restart oracle cannot see the connector's records across a
    restart, give the story's restart through the world's existing restart driver and
    say so; never mark it `broken()`.
  - If `jig-test-domain`'s `Config` has no place for the connector's `SettingsKind`, add
    `settings: Option<..>` beside its connector configuration (default), mapped to one
    `jig_domain_core::settings::Kind` in the core's `Config.kinds`.

## Steps

1. `routes.rs`: `route_effect_ask` maps `JudgeSettings`, `KeepSettings`,
   `KeepProposalSettings`, `DropProposalSettings` and `SaveSettingsDefaults` to the
   connector's events (the root keeps the core's `owner` token beside the connector's own,
   as `Judge` does), and its `SettingsJudged`/`SettingsDefaults` requests to
   `connector::Event::SettingsJudged`/`SettingsDefaults`; the start gives the connector its
   defaults before its restore (engine.md, section 6, step 2). The routing reads nothing
   of a value.
2. Story `jig/tests/core/tests/suite/application_settings.rs` (the world's drivers, its
   oracle on): the test connector owns kind 1, project-scoped, `charters` 1, with one
   default. An owner's change sets a value: accepted, kept, the settings document's state
   changed with its size; a change with a `!` value refused as judged with the words; a
   change to the same value refused as empty; a value naming a charter, then a change
   removing that charter refused as reference; a member's proposal kept, accepted (a new
   revision, the value read from the connector); another proposal withdrawn by that
   revision (dropped at the connector); a restart whose connector default changed: a
   release revision recording it, the connector's default saved.
3. Narrow tests: `heavy cargo nextest run -p jig-core-world --test suite -E
   'test(/application_settings/)'` (check the count), then `-p jig-test-domain --lib`.
   `cargo fmt`; `heavy cargo clippy --workspace --all-targets`. Edit the status file. Commit
   and push.

## Report

How the restart oracle saw the connector's records; any finding left.
