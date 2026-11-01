+++
id = "bx-unforge"
part = "fs"
kind = "migration"
needs = ["bx-story-playground", "bx-story-reset-cut", "bx-story-setup-cut", "bx-story-github", "sv-startup-refuse"]
files = ["temper/crates/temper/src/**", "temper/crates/temper-fake/src/demo.rs", "temper/crates/temper-fake/src/steps.rs", "temper/crates/temper-fake/src/tests/**", "temper/crates/temper-forge-setup/src/check.rs", "temper/crates/temper-forge-setup/src/step.rs", "temper/crates/temper-forge-setup/src/tests/**", "temper/crates/temper/tests/suite/**"]
effort = "medium"
minutes = 75
+++

## Goal

Once the document's forges are refused (`sv-startup-refuse`), the box, the binary and the
fakes keep no reader of them: groups come only from `box/groups/N`, `temper forge ... N`
and `temper fake forge N` read only their group file, and today's demo shape goes.

Acceptance:
- `box_command::run_configuration` builds no group from the document: `Steps.asked` holds
  only the recorded groups loaded from their files; `temper_fake::steps` and
  `temper_forge_setup::steps` take no `Setup` (their whole-list forms go, or keep only
  their always steps: the fake backends' first steps and serve), and the demo's group
  built at the start goes.
- `forge.rs`, `fake.rs`, `fake/check.rs` read forge N's entry from `box/groups/N` alone
  (absent: `forge N has no settings: box/groups/N`); `checked.setup` has no reader left in
  the binary.
- `temper_fake::demo` reads T4's shape alone (`forge` an entry body, `project` an object);
  `Demo::forge()` and `project()` go.
- `temper_forge_setup::check` keeps what the box and `temper forge` still need from the
  document (the setup limits' overrides, `fetch_bytes`) and reads no `application.setup`.
- Every test of the default profile in `temper`, `temper-fake`, `temper-forge-setup` and
  `temper-box` passes; old-form counts fall where forms went (`--write`).

## Inputs

- Draft `temper/docs/design/draft/playground-reset.md`, section 3.
- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T3, T4.
- bx-compose's and bx-doc's reports (their lists of readers of `checked.setup` and of
  today's demo shape); `sv-startup-refuse`'s report.

## Owns and may change

- Owns: the files in `files`.
- Options, pre-approved:
  - `temper forge reconcile N` (a managed forge's admin token named in the setup list) and
    `temper forge sign-in N`'s read of the setup entry: if nothing runs them any more
    (`rg -n '"reconcile"|b"reconcile"|sign-in' temper/crates`), remove `reconcile`; keep
    `sign-in`, reading its entry from `box/groups/N`. Help pages are the lead's: name each
    page that needs a change in the report.
  - A suite test that tested a document's setup entry now refused is deleted when
    `sv-startup-refuse` did not already turn it into a refusal test; list each.

## Steps

1. The readers, then their tests;
   `heavy cargo check -p temper -p temper-fake -p temper-forge-setup --all-targets`.
2. `heavy cargo nextest run -p temper-forge-setup`, `-p temper-fake`, `-p temper-box`,
   `-p temper` (default profile), one at a time.
3. `python3 -I -B scripts/old_forms.py --write` in `temper/` if forms went (never raise);
   `cargo fmt`; `heavy cargo clippy -p temper -p temper-fake -p temper-forge-setup --all-targets`.
   Commit and push.

## Report

Each reader removed, and each help page that names a removed command or member.
