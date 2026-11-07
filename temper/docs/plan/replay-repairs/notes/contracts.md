# replay-repairs: the shapes that cross tasks

The run's binding calls. Each task builds to its part here; none needs another's code but where its spec's `needs` says.

## `Setting`'s order (`setting-order`, then `restart-story`)

`jig/crates/jig-domain-people/src/settings.rs`'s `Setting` derives its order from its variants', and that order must be the store's key order (`jig/docs/design/protocol/store.md`, 4.1). The store's settings key tags are written out and stay as they are (`jig/crates/jig-protocol-store/src/keys/settings.rs`, `setting_write`): Charter 0, Template 1, Policy 2, Rules 3, Application 4, ProjectPolicy 5, Model 6. So the enum becomes, in this order, with no variant's fields changed:

```rust
pub enum Setting {
    Charter(u32),
    Template(u32),
    Policy(u32),
    Rules,
    /// Scope 0 is the deployment; otherwise it is a project number.
    Application { kind: u32, scope: u32, number: u32 },
    /// A project's own policy, an item of its scope (jig/docs/design/domain/engine.md, 17.2).
    ProjectPolicy { project: u32 },
    Model(u32),
}
```

No stored byte moves; what moves is every in-memory order keyed by a setting: a model's setting now follows an application's and a project's policy, in the settings inventory (`documents::Settings::settings`), a revision's items, `Current::offered` and the like. Readers' assertions of such an order follow it.

## A turn read alone (`turn-whole`)

`jig-client-protocol-link`'s `Receiving` gains a second transcript limits value beside `archived_turn`, for `translate::turn::archive` (a `Read::Turn` answer) alone:

```rust
// jig/crates/jig-client-protocol-link/src/limits.rs
struct Receiving {
    // ...
    archived_turn: jig_protocol_transcript::Limits, // as now: a page's archived turns (`translate::pages::carried_turn_fields`)
    turn_read: jig_protocol_transcript::Limits,     // new: a turn read alone, decoded whole
}
// turn_read = Limits::new(bytes, TurnLimits {
//     messages: /* smith's transcript ceiling on a turn's messages (smith::CEILINGS) */,
//     blocks: /* smith's transcript ceiling on a message's blocks */,
//     text: /* bytes * 2, checked */,
//     outcome: client.text, // as now
// })
// where bytes is the archive's byte bound, as now: wire.owned_record.min(receiving.turn_carried_bytes)
pub(crate) fn turn_read(&self) -> &jig_protocol_transcript::Limits
```

so nothing of an admitted turn read alone is cut, while a page's turns keep `turn_text` (`jig/docs/design/client/domain/client.md`, 10 and 14). No wire, contract or stored type changes.

## A failed check's feedback (`check-named`)

`smith-domain-run`'s `Returned::ChecksFailed` gains the check's path and, for a timeout, the time it was given; `Ran` stays as it is:

```rust
ChecksFailed {
    /// First failed repository in workspace order, or none for a workspace-wide failure.
    repository: Box<[u8]>,
    /// The check's path in that repository: the charter's convention (`.smith/check` unless it says otherwise).
    check: Box<[u8]>,
    /// The time the check was given: `check_timeout`, or what was left of the call's deadline when that came first.
    given: Duration,
    /// Checks' process terminal, including its bounded diagnostic tail.
    ran: Ran,
},
```

rendered `checks-failed repository=R check=PATH exit=E given_ms=N cut=C tail=T`, `given_ms` written for every exit (the bound it ran under), in that order. The feedback is text; no wire or stored format changes.

## A files-omitted notice (`review-files`)

A pull section whose `Files` read answers `TooLarge` appends, within its byte budget:

```text
[files omitted from page N: past the forge client's limit; read_pull_files from page N+1 lists the rest, and read_file reads a path at the head or the base]
```

and completes as the comparison-omitted case does (`temper/docs/design/domain/forge.md`, 8.4).
