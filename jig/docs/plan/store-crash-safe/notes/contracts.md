# store-crash-safe: contracts

The shapes that cross tasks, binding for every spec that cites them
(`jig/docs/design/domain/engine.md`, 6).

## 1. The start's link refusal (`link-refusal`)

In `jig/crates/jig-domain-core/src/start.rs`, beside `Failure`, exported
as `jig_domain_core::start::Linked`:

```rust
pub enum Failure {
    // ... the eight variants as they are ...
    /// A record names one the start did not load (engine.md, 6).
    Link { record: Key, linked: Linked },
}

/// What a record links to that the start could not find (engine.md, 6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Linked {
    /// A stored record, by its key.
    Record(Key),
    /// A charter at a revision the held settings lack.
    Charter { charter: u32, revision: u32 },
    /// A project with no policy.
    Project(u32),
    /// A connector this configuration lacks.
    Connector(u16),
}
```

- **One refusal, the first.** The start keeps its first failure
  (`start_refuse`); a link found missing refuses as `Failure::Link` and
  the script answers `Action::Refused`, never a panic, never a commit,
  never a sweep.
- **Every link the start checks:** the core's
  (`start/links.rs`, `start/projections.rs`'s link asserts,
  `start/script.rs`'s census position and `start.missing`), and each
  child's, through the child's own restore failure, which the core maps
  in `start_collect_failure`:
  - `jig_domain_people::RestoreCause::Link { target: people::Key }`, for
    `domain/start.rs`'s `link` (a row naming a person it lacks) and its
    identity collision (`record` the second person, `target` the first's
    `Person` key);
  - `jig_domain_tasks::RestoreCause::Link { target: tasks::Key }` and
    `RestoreCause::Charter { charter: u32, revision: u32 }`, for
    `stored.rs`'s `links` and `live_link`.
  The core maps a child's `Link` to `Failure::Link { record, linked:
  Linked::Record(target) }` in the core's `Key`, and tasks' `Charter`
  to `Linked::Charter`.
- **The words** (temper's shell, `start_failure`): the panic's words, as
  a reason: `{record:?} links missing {linked}`, where `linked` reads
  `{key:?}` for a record, `charter {charter} at revision {revision}`,
  `project {project}`, `connector {connector}`. So the deployment's line
  would read `temper serve: restart refused during <step>:
  Tasks(Live(199)) links missing People(Person(102))`.

## 2. What the rows promise (`crash-rows`, `writer-rows`)

- **A kill** at any file operation of a decision's commit, any number of
  kills in a row, starts again to `Phase::Open`, with neither
  `Action::Refused` nor a panic, and the store's rows those of the last
  acknowledged commit or of that and one whole uncertain one
  (`jig/docs/design/protocol/store.md`, 14).
- **A person named from outside the core** that the people child does
  not hold is refused where it is admitted and never written; one it
  holds is written and linked at the next start
  (`jig/docs/design/domain/engine.md`, 5.1).
