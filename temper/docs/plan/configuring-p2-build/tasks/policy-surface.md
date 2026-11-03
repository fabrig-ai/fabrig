+++
id = "policy-surface"
part = "policy-item"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-people/src/settings.rs", "jig/crates/jig-domain-core/src/settings/change.rs", "jig/crates/jig-domain-core/src/tests/settings/project_policy.rs", "jig/crates/jig-domain-core/src/tests/settings.rs"]
effort = "medium"
minutes = 60
+++

# policy-surface: a project's policy, named as a setting

## Goal

jig's `Setting` names a project's own policy as an item of a change (contract B1), and the
core refuses it at the change's entrance, `SettingFault::Shape`, until `policy-change` gives it
its check and commit. Every match the new variant reaches gets the compiler-forced arm below;
no behaviour of any existing change, proposal, start, read or route moves.

Acceptance:

- `jig_domain_people::Setting::ProjectPolicy { project: u32 }` exists, last of the variants,
  documented in a line citing `jig/docs/design/domain/engine.md`, 17.2.
- A `ChangeSettings` and a `Propose` naming it are refused `Shape`, naming the item, by
  `jig_domain_core::settings::change::change_shape`; the unit test below shows both.
- `heavy cargo check --workspace --all-targets` is clean; the focused tests of
  `jig-domain-people` and `jig-domain-core` pass, with the new test.

## Inputs

- `jig/docs/design/domain/engine.md`, 17.2 (a project's own policy as an item) and 17.6.
- `jig/docs/design/domain/people.md`, 5.2.
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B1 (binding names), and
  `temper/docs/plan/configuring-p2/notes/contracts.md`, P1.
- `temper/docs/plan/configuring-p2-build/notes/common.md`.
- As the pattern this follows: `temper/docs/plan/reference-m3/tasks/apps-surface.md`, which
  added `Setting::Application` the same way (read its arm table; do not redo its work).

## Owns and may change

- Owned: `jig/crates/jig-domain-people/src/settings.rs` (the variant);
  `jig/crates/jig-domain-core/src/settings/change.rs` (`change_shape`'s refusal); the new
  unit test file `jig/crates/jig-domain-core/src/tests/settings/project_policy.rs` and its
  `mod project_policy;` line in the settings tests' module.
- Neighbour files, **compiler-forced arms only**, in jig and in temper (temper's are the
  stated cross-tree allowance for a required variant: each edited in that arm alone), each
  arm as this table says, nothing else of the file changed:

| Where | The arm |
|---|---|
| `jig-domain-core/src/settings/change.rs` `change_shape`, the first match | `Setting::ProjectPolicy { .. } => return Err(Failure { setting: item.setting, fault: Fault::Shape })` with the line `// A project's policy is refused until policy-change gives it a check (engine.md, 17.2).` |
| `settings/application.rs` `is_application`, `settings/check.rs` `shape`'s `Value::Policy` arm and every other `matches!`-like boolean match of the core's settings | `Setting::ProjectPolicy { .. }` joins the `false` arm |
| any other match on `jig_domain_people::Setting` in `jig-domain-core` (settings, documents, reads, admission, routing, acts, the `#[cfg(test)]` fixtures) | `Setting::ProjectPolicy { .. } => unreachable!("a project's policy: policy-change")` |
| in `jig-protocol-store` (codecs, keys) | `unreachable!("a project's policy: policy-store")` |
| in `jig-protocol-engine-clients`, `jig-client-protocol-link`, `jig-client-view`, `jig-client-domain`, jig's testing crates and examples | `unreachable!("a project's policy: policy-wire")` |
| in temper (`temper-client-tools`, `temper-settings-form`, `temper-engine-smith`, `temper-service`, any other the compiler names) | `unreachable!("a project's policy: policy-ctl")` |

  A match the arm would make non-exhaustive in a way the table does not cover (a `_` arm that
  now catches the new variant silently, a `matches!` whose meaning changes): leave it, and
  list it in the report with its file and line.
- Options, pre-approved:
  - If a fixed byte price in a test (a document's or envelope's worst case) fails only
    because `Setting` grew, correct that expectation and its comment alone, the comment naming
    the type that grew and by how much; list each. A production bound, limit or budget that
    fails is not covered: stop and ask.
  - If a sorted golden or a test's order moves because the variant sorts last, keep it last
    and say so in the report.
  - If a match in a crate outside the table's rows needs an arm, give it
    `unreachable!("a project's policy: policy-wire")` and list it.

## Steps

1. `jig-domain-people/src/settings.rs`: `Setting` gains, after `Application { .. }`,
   ```rust
   /// A project's own policy, an item of its scope (jig/docs/design/domain/engine.md, 17.2).
   ProjectPolicy { project: u32 },
   ```
   The type's derives stay as they are.
2. `change_shape`'s arm (the table's first row), before the duplicate check, so a second
   `ProjectPolicy` never reaches it.
3. The compiler-forced arms, found with `heavy cargo check --workspace --all-targets`.
4. Test `jig-domain-core/src/tests/settings/project_policy.rs`, in the style of the
   settings tests beside it (`application.rs`, `templates.rs`): one test,
   `a_project_policy_item_is_refused_its_shape_until_its_check`, that starts the core from
   the crate's test fixture with one live project, sends a party's `ChangeSettings` with the
   one item `Setting::ProjectPolicy { project }` set to the fixture's policy 1's value, and a
   `Propose` with the same item, and asserts each is refused malformed with
   `SettingFault::Shape` naming that item, and that the current settings and the history are
   as they were (no revision made).
5. Narrow tests: `heavy cargo nextest run -p jig-domain-people --lib`;
   `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/settings/)'` (check the
   count); `heavy cargo check --workspace --all-targets`. Then `cargo fmt`; `heavy cargo
   clippy -p jig-domain-people -p jig-domain-core --all-targets -- -D warnings` and the same
   for each other package you gave an arm; the focused tests of `jig-domain-people` and
   `jig-domain-core`. Commit and push.

## Report

The arms added, by file, as a list (file, function, body); any match left as `## Owns and may
change` says; any test expectation corrected and why; whether the variant's order moved
anything.

## Answers

2026-11-03. **May `settings/application.rs`' `projects` collect the item's project instead of
`unreachable!`?** Yes. Give that one function the arm
`Setting::ProjectPolicy { project } => project`, and leave `routing/settings/dispatch.rs`'s
order as it is. A project's policy is an item of its project's scope (jig's
`domain/engine.md`, 17.2; contract P1, "Who"), so this arm is the design's own and not a
placeholder. It is the arm `policy-change` plans to write (its step 1), and this task now
writes it first. With it, the people child checks that the party owns the project, and
`change_shape` then refuses the item `Shape`, as acceptance asks. Every other arm stays as the
table says. If another match on the path to the `Shape` refusal (direct change or proposal)
panics the same way, give it the design's value for the item in the same way (its scope is the
project; a boolean is `false`) and list it in the report. Never reorder routing.
