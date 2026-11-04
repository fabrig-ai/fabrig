# repository-copies-1: the shapes that cross tasks

Binding. Each is the run's call; a task that needs another shape asks.

## skein

**S1, the dialect** (`skein_forge::forgejo::request::Request`, forge-setup.md, 4.3):

```rust
/// Copies a public repository by Forgejo's migration, answered once whole.
Migrate { at: Repo, from: Box<[u8]> },
/// Deletes a repository, answering without a document.
DeleteRepository { at: Repo },
```

`Migrate`'s answer is `Document::CreatedRepository`; `DeleteRepository` has no document.

**S2, the plan** (`skein_forge_setup`, forge-setup.md, 4.10):

```rust
// host::Plan
Copy { forge: u16, org: Box<[u8]>, repository: Box<[u8]>, from: Box<[u8]>, files: Box<[File]> },
// reconcile::Outcome
Created { id: NonZeroU64, branch: Box<[u8]> },      // `branch` is new, the repository's default
Copied { id: NonZeroU64, branch: Box<[u8]> },
// reconcile::Why
Taken, Empty, TimedOut,
// item::Item
Copying,                                            // the one byte 1
// host::Event
Said { run: Token, words: Box<[u8]> },
// limits::Limits
pub copy_deadline: Duration,
// given::Place
CopyDeadline, CopyFrom,
// given
pub fn check_copy(limits: &Limits, string_bytes: u32, plan: &crate::host::Plan) -> Result<(), Refused>;
```

- `Plan::Copy` writes (`Plan::writes`), and `Reachable`'s slot beside it is untouched.
- A `Read(Item::Copying)` is answered with the kept bytes or none; a `Write` of it is answered
  `Written` once kept. The owner clears it when a run stops at `Taken`.
- `Event::Said` comes at most once per run, before that run's `Event::Stop`; the owner holds the words
  until the stop. Only a refused `Migrate` says any, and never a 401's.
- `Plan::Copy`'s `Copying` is run by the owner, never in its credential records.

**S3, the fake's domain** (`skein_fake_forge_domain`, fake-forge.md, 2.6):

```rust
// api::Write
Copy { owner: Box<[u8]>, name: Box<[u8]>, fill: Fill },   // called on the source
DeleteRepository,                                          // called on the repository
// api
pub enum Fill { Whole, Slow(Duration), Cut }
// api::Answer
Copied { repository: u32, default_branch: Box<[u8]> },
// api::Refusal or its equivalent
Exists,
```

**S4, the catalogue** (`skein_fake_forge_protocol`, fake-forge.md, 4):

```rust
pub struct Source { pub address: Box<[u8]>, pub repository: Box<[u8]>, pub private: bool, pub fill: Fill }
// Config
pub sources: Box<[Source]>,
```

`repository` is the domain's full name; an address's host ends `.invalid`, or `valid` refuses it.

**S5, the throwaway** (`skein_throwaway_forgejo`, fake-forge.md, 6):

```rust
// Settings
pub local_migrations: bool,
// Instance
pub fn kill(&self) -> Result<(), Failure>;
pub fn restart(&self) -> Result<(), Failure>;
pub fn exchange_patient(&self, outgoing: &Outgoing<'_>, patience: Duration) -> Result<Exchange, Failure>;
```

## temper

The design is `temper/docs/design/domain/forge.md`, 4.5; `temper/docs/design/protocol/setup.md`, 6.9;
`temper/docs/design/protocol/store.md`, 2 and 3; `temper/docs/design/client/README.md`, 4.6 and 8.
**`Making` is a name taken twice already** (`temper_engine_domain_forge::Making`, the wire's `enum
Making`): only the store family and the `Stored` and `Key` variants take it. Every type says
`RepositoryMaking`, `MakingState`, `MakeFrom` and so on.

**T1, the row** (`temper-engine-domain-forge`, new `src/making.rs`, exported from `lib.rs`):

```rust
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MakingKey { pub forge: u16, pub org: Box<[u8]>, pub name: Box<[u8]> }
pub enum MakeFrom { Empty, Copy { address: Box<[u8]> } }
pub struct MakeInto { pub project: u32, pub home: bool, pub host: Box<[u8]> }
pub enum MakeStep { Admin, Org, Repository, File, Adoption }
pub enum MakeFailure { Taken, Empty, TimedOut, Refused { status: u16 }, NotAllowed, Other }
pub struct MakingFailed { pub step: MakeStep, pub reason: MakeFailure, pub words: Box<[u8]> }
pub enum MakingState {
    Waiting,
    Making,
    Made { id: u64, branch: Box<[u8]> },
    Adopting { id: u64, branch: Box<[u8]> },
    Failed(MakingFailed),
}
pub struct RepositoryMaking {
    pub key: MakingKey,
    pub from: MakeFrom,
    pub into: Option<MakeInto>,
    pub asked: Wall,
    pub copying: bool,
    pub state: MakingState,
}
pub enum MakeRefusal { NoSuchForge, NotOffered, NotOwner, Invalid, HomeHeld, Full }
```

- `Stored::Making(RepositoryMaking)`, `Key::Making(MakingKey)`, `RecordKind::Making`, last of each.
  `to-begin-store` adds `Dismissed` at the same place: **`Dismissed` stays before `Making`**, so `Key`'s
  order is the bytes'.
- `Limits::makings: u32`, the name `forge.makings`, default 32, at least 1, at most 64.

**T2, the store's row** (`temper-protocol-store`): `pub const MAKING: u8 = 35;`. The key is the
family's byte, `writer.u64(u64::from(forge))`, then `org` and `name`, each one byte string; `largest`
grows with it. `schema/store-v11.schema`, in place, appended so that no tag moves; `STORE_VERSION`
unchanged. `ApplicationOutcome` gains `MakeKept` and `MakeRefused(MakeRefusal)`, appended.

**T3, the wire** (`temper-client-protocol-wire/schema/client-wire-v1.schema`, in place, each new
variant last of its enum, the header's 11 unchanged; `to-begin-store` adds `dismiss` at the same places,
and whichever lands second follows the first):

```
record MakeCopy { address: bytes 512 }
enum MakeFrom { empty, copy: MakeCopy }
record MakeInto { project: u32 home: bool }
record MakeRepository { forge: u16 org: bytes 1024 name: bytes 1024 from: MakeFrom into: option MakeInto }
enum Request { ... make_repository: MakeRepository }
enum MakeRefusal { no_such_forge, not_offered, not_owner, invalid, home_held, full }
record MakeRefused { why: MakeRefusal }
enum Outcome { ... make_kept, make_refused: MakeRefused }
record Made { id: u64 branch: bytes 1024 }
enum MakeStep { admin, org, repository, file, adoption }
record Refused { status: u16 }
enum MakeFailure { taken, empty, timed_out, refused: Refused, not_allowed, other }
record MakingFailed { step: MakeStep reason: MakeFailure words: bytes 256 }
enum MakingState { waiting, making, made: Made, adopting: Made, failed: MakingFailed }
record RepositoryMaking { org: bytes 1024 name: bytes 1024 from: MakeFrom into: option MakeInto state: MakingState }
record ForgeRow { ... read_only: bool makings: list 64 RepositoryMaking }
```

`pub const MAKINGS: u32 = 64;` in `lib.rs`, set in `limits()` as `forge_row_repositories` is. If a
generated type's name collides (`Refused`, `Made`), prefix it `Making`, and say so in the report.

**T4, the address** (`temper-client-protocol-wire`, new `src/source.rs`, `pub mod source`; both domains
depend on this crate and neither on the other):

```rust
pub struct Source<'a> { pub host: &'a [u8], pub owner: &'a [u8], pub name: &'a [u8] }
pub enum SourceRefusal { Scheme, Segments, Host, Name }
pub fn parse(address: &[u8]) -> Result<Source<'_>, SourceRefusal>;
pub fn address(source: &Source<'_>) -> Box<[u8]>;   // https://HOST/OWNER/NAME
pub fn org(name: &[u8]) -> bool;                    // skein's given::login's grammar
pub fn repository(name: &[u8]) -> bool;             // 1 to 100 of [A-Za-z0-9._-], neither . nor ..
```

`parse` takes the draft's 5.4: `HOST/OWNER/NAME`, with or without `https://` before and `.git` after;
HOST a DNS name with an optional port; OWNER and NAME 1 to 100 bytes of `[A-Za-z0-9._-]`, neither `.`
nor `..`. `http://`, `ssh://` and `git@` are `Scheme`; more segments or fewer, `Segments`. A
source's own name is the copy's unless `--as` says otherwise, so an owner that is no `org` needs `--as`.

**T5, across the connector's boundary** (`temper-engine-domain-forge`):

```rust
// Ask
MakeRepository { forge: u16, org: Box<[u8]>, name: Box<[u8]>, from: MakeFrom, into: Option<MakeInto> },
// Answered
Make(MakeAnswer),          // MakeAnswer::{Kept, Refused(MakeRefusal)}
// Event, from the service
MakingStarted { key: MakingKey },
MakingCopying { key: MakingKey },
MakingMade { key: MakingKey, id: u64, branch: Box<[u8]> },
MakingStopped { key: MakingKey, failed: MakingFailed },
MakingAdopt { key: MakingKey },
// Event, from the root
MakingAsked { request: Token, adoption: Adoption, repository: skein_forge::api::Repository },
MakingActed { key: MakingKey, outcome: Option<MakingFailed> },   // None: adopted
MakingAgain { key: MakingKey },                                  // jig was busy; the pull asks again
// Request
MakingCopied { key: MakingKey },                                 // the mark is kept
MakingAct { key: MakingKey, act: [u8; 16], adoption: Adoption, repository: skein_forge::api::Repository },
// Domain
pub fn making(&self, key: &MakingKey) -> Option<&RepositoryMaking>;
pub fn next_making(&self, forge: u16) -> Option<&RepositoryMaking>;   // the oldest Waiting, by asked then key
pub fn making_due(&self, forge: u16) -> Option<MakingKey>;            // Made or Adopting, no act in flight
pub fn makings(&self, forge: u16) -> impl Iterator<Item = &RepositoryMaking>;   // in key order
```

The `Row` of `Domain::forges` gains `makings: Box<[RepositoryMaking]>`.

**T6, the act's key:** the first 16 bytes of the SHA-256 of `b"temper making"`, the forge as two
bytes big-endian, the org's and the name's lengths and bytes, and `asked`'s nanoseconds.
