# ui-changes: the shapes that cross tasks

Binding. The words and the meaning are `temper/docs/design/client/README.md`, 3.5 (*Changes*) and 4.6,
`temper/docs/design/client/forge.md`, section 3, and `temper/docs/design/domain/after-landing.md`, section
10. A task's sketch inside these shapes is its own.

## The wire: `temper-client-protocol-wire/schema/client-wire-v1.schema` (`changes-wire`)

```
record ChangesSubject { project: u32 }
enum Watch { change: ChangeSubject, forges, landing: LandingSubject, landings: LandingsSubject, dismissed, changes: ChangesSubject }
record ChangesQueued { task: u64 title: bytes 256 ready_since: u64 checks_since: option u64 }
record ChangesQueue { repository: bytes 2048 branch: bytes 1024 red: bool changes: list 32 ChangesQueued }
record ChangesLanded { task: u64 title: bytes 256 at: u64 }
versioned record ChangesPanel { project: u32 landing: bytes 1024 queues: list 32 ChangesQueue more: u32 landed: list 32 ChangesLanded landed_more: u32 }
```

Times are nanoseconds since the epoch, as `skein_lib::Wall::as_nanos` gives them. In
`temper-client-protocol-wire`'s `lib.rs`, beside `landings_panel`:

```rust
pub fn changes_panel(record: &v1::ChangesPanel) -> Box<[u8]>;
pub fn decode_changes_panel(bytes: &[u8], limits: &v1::Limits) -> Option<v1::ChangesPanel>;
```

A queue with `changes` empty is a branch that is red with nothing ready: it draws no section. The
changes queued are every queue's `changes` and `more`.

## The store: family 30 (`landing-log`)

`temper-protocol-store/schema/store-v11.schema`, `Value` gaining `landing_log: LandingLog` last:

```
record LoggedChange { task: u64 pull: option u64 title: bytes 256 }
record LoggedLanding { commit: fixed 32 branch: bytes 4096 at: u64 changes: list 64 LoggedChange }
record LandingLog { repository: Repository landings: list 256 LoggedLanding }
```

Its key: the tag `LANDING_LOG`, byte 30, then the repository, as `Key::Verification` writes it.

In `temper-engine-domain-forge`, its `boundary.rs` and a new module `landing_log.rs`:

```rust
/// Family 30's ring, oldest first (temper/docs/design/domain/after-landing.md, 10).
pub struct LandingLog { pub repository: skein_forge::api::Repository, pub landings: Box<[LoggedLanding]> }
pub struct LoggedLanding { pub commit: skein_forge::api::Commit, pub branch: Box<[u8]>, pub at: skein_lib::Wall, pub changes: Box<[LoggedChange]> }
pub struct LoggedChange { pub task: u64, pub pull: Option<u64>, pub title: Box<[u8]> }

pub const LOGGED_TITLE_BYTES: usize = 256;

Stored::LandingLog(LandingLog)
Key::LandingLog { repository: skein_forge::api::Repository }
RecordKind::LandingLog

impl Domain {
    /// A repository's ring, oldest first.
    pub fn landing_log(&self, repository: skein_forge::api::Repository) -> Option<&LandingLog>;
}
```

The domain keeps the rows as `landing_logs: Map<Repository, LandingLog>`, capacity `repositories`.

## The limits

`temper-engine-domain-forge`'s `Limits`, each a name of `temper-service`'s `limits/names.rs`, the same
in `small` and `standard`:

| Field | Name | Value | `valid` refuses | Task |
|---|---|---|---|---|
| `landings_kept` | `forge.landings_kept` | 16 | 0, past 256 | `landing-log` |
| `changes_rows` | `forge.changes_rows` | 32 | 0, past 32 | `changes-view` |
| `changes_landed` | `forge.changes_landed` | 8 | 0, past 32 | `changes-view` |

## The view (`changes-view`)

In `temper-engine-domain-forge/src/views.rs`, its third project subject and its `Document` arm:

```rust
#[must_use]
pub fn changes_subject(project: u32) -> u64 { (1 << 62) | (1 << 33) | u64::from(project) }

Document::Changes(ChangesView)

pub struct ChangesView {
    pub project: u32,
    pub landing: Box<[u8]>,
    pub queues: Box<[ChangesQueue]>,
    pub more: u32,
    pub landed: Box<[ChangesLanded]>,
    pub landed_more: u32,
}
pub struct ChangesQueue { pub repository: Box<[u8]>, pub branch: Box<[u8]>, pub red: bool, pub changes: Box<[ChangesQueued]> }
pub struct ChangesQueued { pub task: u64, pub title: Box<[u8]>, pub ready_since: skein_lib::Wall, pub checks_since: Option<skein_lib::Wall> }
pub struct ChangesLanded { pub task: u64, pub title: Box<[u8]>, pub at: skein_lib::Wall }
```

## The client (`changes-page`)

`temper_client_domain::Address::Changes { project: u32 }`, at `/projects/<project>/changes`.
