+++
id = "project-templates"
part = "project-templates"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-core/src/lib.rs",
  "jig/crates/jig-domain-core/src/documents/small.rs",
  "jig/crates/jig-domain-core/src/tests/settings/templates.rs",
]
effort = "medium"
minutes = 25
+++

# jig: the templates a project offers, public

## Goal

temper's root judges kind 7's `way` against the templates a project offers, and only jig can say which:
`Core::template_overrides` is the crate's. Give the root one public read, the one `me` already computes.

Acceptance:

- `Core::project_templates` is public and `documents/small.rs` calls it where it merges today, so that
  `me`'s templates and this read can't differ.
- A step test: a project with no override reads the current templates; a project whose override retires
  a template reads without it; an unknown project reads the current templates.
- jig's `me` tests still pass.

## Inputs

- `temper/docs/plan/ui-settings/notes/common.md`.
- `jig/docs/design/domain/engine.md`, 17.2 (a project's templates).

## Owns and may change

Owns `files`. Nothing else: this branch stays in jig.

## Steps

1. In `jig-domain-core/src/lib.rs`, on `impl Core`, beside what reads `template_overrides`:

   ```rust
   /// The templates `project` offers: the current ones under its overrides, as `me` delivers them
   /// (jig/docs/design/domain/engine.md, 17.2).
   #[must_use]
   pub fn project_templates(&self, project: u32) -> Box<[settings::Template]>
   ```

   Its body is what `documents/small.rs` does at its `current_merged` call; the name of the template
   type is whatever `current_merged` returns.
2. `documents/small.rs` calls it.
3. The test in `src/tests/settings/templates.rs`, by that file's fixtures.
4. `cargo fmt`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-domain-core`. Commit and push.

## Report

As the implementer's text says.
