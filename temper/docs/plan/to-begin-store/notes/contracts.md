# to-begin-store: contracts

The shapes that cross tasks, binding. Numbered; never renumbered. The design they carry is
`temper/docs/design/protocol/store.md`, 2 and 3; `temper/docs/design/client/README.md`, 4.6 and 8;
`temper/docs/design/domain/forge.md`, 5.5.

## K1. The domain's row

In `temper-engine-domain-forge`, a new module `src/dismissed.rs`, its types exported from `lib.rs`:

```rust
/// What a party has dismissed (temper/docs/design/domain/forge.md, 5.5).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Dismissal {
    /// "Skip" on the project's "To begin".
    ToBegin { project: u32 },
}
```

- `Stored::Dismissed { party: u64, what: Dismissal }`, last of `Stored`;
  `Key::Dismissed { party: u64, what: Dismissal }`, last of `Key`, so `Key`'s order is the bytes'.
- `RecordKind::Dismissed`.
- `Limits::dismissed(&self) -> u32`, `bindings` times `judge_projects`, saturating: no new limit,
  no override, no line in `temper/docs/help/`.
- `Domain::dismissals(&self, party: u64) -> Box<[Dismissal]>`: the party's rows in key order.

## K2. The store's row

`temper-protocol-store`: `pub const DISMISSED: u8 = 34;` in `keys.rs`. The key is the family's byte,
`writer.u64(party)`, then `writer.tag(0).u64(u64::from(project))` for `ToBegin`; the decoder refuses
another tag, a project past `u32` and trailing bytes.

`schema/store-v11.schema`, in place, appended so that no tag moves:

```
# A person's dismissals (protocol/store.md, 2).
record ToBegin { project: u32 }
enum Dismissal { to_begin: ToBegin }
record Dismissed { party: u64 what: Dismissal }
enum Value { ... setting: SettingRow, dismissed: Dismissed }
```

## K3. The wire

`temper-client-protocol-wire/schema/client-wire-v1.schema`, in place, each new variant last of its
enum, so that no tag moves; the family's header stays 11:

```
# A person's dismissals (client/README.md, 4.6).
record ToBegin { project: u32 }
enum Dismissal { to_begin: ToBegin }
record Dismiss { what: Dismissal }
enum Request { ... replace: Replace, dismiss: Dismiss }
enum Outcome { ... end_refused: EndRefused, dismissed }
enum Watch { ... landings: LandingsSubject, dismissed }
versioned record Dismissals { dismissed: list 1024 Dismissal }
```

In its `lib.rs`:

```rust
/// A party's dismissals on the wire (client/README.md, 4.6).
pub const DISMISSED: u32 = 1024;
pub fn dismissals(record: &v1::Dismissals) -> Box<[u8]>;
pub fn decode_dismissals(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Dismissals>;
```

`limits()` sets the generated field for the list from `DISMISSED`.

## K4. The request, through the root

In `temper-engine-domain-forge`:

```rust
/// A dismissal's answer (temper/docs/design/domain/forge.md, 5.5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DismissAnswer { Dismissed, TooLarge }
pub enum Answered { ..., Dismiss(DismissAnswer) }
pub enum Event { ..., Dismiss { request: Token, token: Token, party: u64, what: Dismissal } }
/// The subject `dismissed`; outside the task numbers, as `landings_subject`'s are.
pub const DISMISSED_SUBJECT: u64 = 1 << 61;
```

A change marks it with the existing `Request::Marked { tasks: Box::new([DISMISSED_SUBJECT]) }`.

In `temper-engine-domain`, beside `Return`, since the forge's `Ask` has no party:

```rust
pub enum Application { Forge(forge::Ask), Return { .. }, Dismiss { what: forge::Dismissal } }
pub enum ApplicationAnswer { ..., Returning, Dismiss(forge::DismissAnswer) }
```

`applications::start` names `ApplicationScope::Project(project)`, `what`'s project, as the scope,
whatever scope the record came under. `applications::admitted` takes the core ask's `person` and
pushes `forge::Event::Dismiss { request, token, party: person, what }`.

`temper-service`'s `family.rs`: `Dismissal`'s two ways, `DismissAnswer::Dismissed` to
`w::Outcome::Dismissed`, `TooLarge` to `w::Outcome::ForgeRefused` with `w::ForgeReason::TooLarge`.

## K5. The document

```rust
pub enum Document { Change(Panel), Landing(LandingView), Landings(LandingsView), Dismissals(Box<[Dismissal]>) }
impl Domain {
    pub fn document(&self, subject: u64, scope: ApplicationScope, party: u64, now: Wall, limits: &Limits)
        -> Option<Document>;
}
```

`document` gains `party`: `ApplicationScope::Own` with `DISMISSED_SUBJECT` answers
`Document::Dismissals(self.dismissals(party))`, and `Own` with any other subject none.
`panels::document` takes the party from `core::Ask::Document`. `to_root`'s `w::Watch::Dismissed` is
`Watched::Application { scope: ApplicationScope::Own, connector: wire::FORGE, subject:
Token::new(forge::DISMISSED_SUBJECT) }`, whatever scope the record came under, as `forges`' is.

## K6. The client's facts

`temper_client_domain::forge::dismissed`, a new module:

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dismissal { ToBegin { project: u32 } }
pub enum State { Unknown, Rows(Box<[Dismissal]>), Ended }
#[must_use]
pub fn watch(limits: &super::Limits) -> Option<Watchable>;
/// The request "Skip" will send: of `what`'s project's scope.
#[must_use]
pub fn ask(what: Dismissal, limits: &super::Limits) -> Option<Ask>;
impl Side {
    /// `None` until the first document, or once the watch is refused or ended.
    #[must_use]
    pub fn dismissed(&self, what: Dismissal) -> Option<bool>;
}
```
