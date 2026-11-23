# Contracts of the builder's checks at the gate

2026-11-23. The shapes that the changes moving hearth's builder and
resolver checks to the gate share while that goal runs
(`temper/docs/design/draft/design-economy.md`, 4.3). The lasting design
is `README.md`, 2.5 and 2.7; this document goes with the goal's last
change.

## 1. A later revision

The builder world's scenario, `hearth_builder_world::fixtures::Scenario`
in `hearth/tests/builder/src/fixtures/worlds.rs`, gains one member:

- **`later: Option<&'a [File]>`**, `None` in `fixtures::thin()` and so in
  every scenario built from it. Where it is `Some(files)`, the world
  commits `files`, whole, as `HEAD`'s tree over the commit of the
  scenario's own `files`, and `refs/heads/before` names that first
  commit. A path of `files` that `later` lacks is gone from `HEAD`.
  Where it is `None`, a `Call::Diff` keeps the edit
  `Shared::for_scenario` infers for it today, and every other call sees
  `files` alone at `HEAD`.

Two helpers in `hearth_builder_world::fixtures` build on it:

- **`fn diffing<'a>(files: &'a [File], later: &'a [File]) -> Scenario<'a>`:**
  `Call::Diff { resolve: false, repository: GIT_DIR, from: b"before",
  to: b"HEAD", targets: empty }` in `Form::Text`, with `later:
  Some(later)` and the rest as `thin()`. A caller sets its targets,
  `resolve`, form or `laid` with struct update.
- **`fn edited(files: &[File], changed: &[File], removed: &[&str]) -> Vec<File>`:**
  `files` with each of `changed` replacing the file of its path or
  appended where none has it, then each path of `removed` dropped: the
  later tree of a scratch repository's `Repository::next(changed,
  removed)`.

## 2. Open questions

None.
