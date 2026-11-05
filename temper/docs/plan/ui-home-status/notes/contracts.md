# ui-home-status: the run's binding shapes

What crosses tasks. A task builds to these; a name here is never another task's to change.

## The wire (`temper-client-protocol-wire/schema/client-wire-v1.schema`)

```
record StatusSubject { project: u32 }
enum Watch { ..., status: StatusSubject }
record StatusBrief { sentence: bytes 200 lede: bytes 600 at: u64 landed: u64 }
versioned record ProjectStatus { project: u32 landed: u64 last_landed: option u64 brief: option StatusBrief previous: option u64 seen: u64 queued: u32 red: bool branch: bytes 1024 }
record Looked { project: u32 }
enum Request { ..., looked: Looked }
enum Outcome { ..., looked }
```

Times are nanoseconds since the epoch, as `ChangesLanded.at` is. Each enum's new arm goes last. In
`temper-client-protocol-wire/src/lib.rs`, beside `changes_panel` and `decode_changes_panel`:

```rust
pub fn project_status(record: &v1::ProjectStatus) -> Box<[u8]>;
pub fn decode_project_status(bytes: &[u8], limits: &v1::Limits) -> Option<v1::ProjectStatus>;
```

## The store (`temper-protocol-store`)

Byte 36 `PROJECT_FACTS`, byte 37 `LOOKED` in `keys.rs`. `ProjectFacts`'s key is the project; `Looked`'s is the
party, then the project.

```
record StatusBrief { sentence: bytes 200 lede: bytes 600 at: u64 landed: u64 }
record StatusRunning { task: option u64 landed: u64 since: u64 }
record ProjectFacts { project: u32 landed: u64 last_landed: u64 brief: option StatusBrief running: option StatusRunning }
record Looked { party: u64 project: u32 previous: option u64 latest: u64 seen: u64 next_seen: u64 }
enum Value { ..., project_facts: ProjectFacts, looked: Looked }
```

## The forge (`temper-engine-domain-forge/src/status.rs`, a new module beside `dismissed.rs`)

```rust
/// A new sitting past this (temper/docs/design/domain/forge.md, 5.6).
pub const SITTING: Duration = Duration::from_secs(1800);
pub const BRIEF_SENTENCE_BYTES: usize = 200;
pub const BRIEF_LEDE_BYTES: usize = 600;

pub struct Brief { pub sentence: Box<[u8]>, pub lede: Box<[u8]>, pub at: Wall, pub landed: u64 }
/// A status run in flight (temper/docs/design/domain/forge.md, 5.7); `task` none until the root has made it.
pub struct StatusRunning { pub task: Option<u64>, pub landed: u64, pub since: Wall }
pub struct ProjectFacts { pub project: u32, pub landed: u64, pub last_landed: Wall, pub brief: Option<Brief>, pub running: Option<StatusRunning> }
pub struct Looked { pub party: u64, pub project: u32, pub previous: Option<Wall>, pub latest: Wall, pub seen: u64, pub next_seen: u64 }
pub enum LookedAnswer { Looked, TooLarge }
pub struct StatusView {
    pub project: u32, pub landed: u64, pub last_landed: Option<Wall>, pub brief: Option<Brief>,
    pub previous: Option<Wall>, pub seen: u64, pub queued: u32, pub red: bool, pub branch: Box<[u8]>,
}
/// The subject `status`, beside `changes_subject`.
pub fn status_subject(project: u32) -> u64 { (1 << 62) | (1 << 34) | u64::from(project) }
```

Each derives `Clone, PartialEq, Eq, Debug`, and is exported from the crate as `Dismissal` is. In the crate's
vocabulary:

```rust
Stored::ProjectFacts(ProjectFacts)      Stored::Looked(Looked)
Key::ProjectFacts { project: u32 }      Key::Looked { party: u64, project: u32 }
Event::Looked { request: Token, token: Token, party: u64, project: u32 }
Answered::Looked(LookedAnswer)
Document::Status(StatusView)
```

## The root (`temper-engine-domain`)

`Application::Looked { project: u32 }`, scoped as `Dismiss` is, and `ApplicationAnswer::Looked(forge::LookedAnswer)`.

## The client (`temper-client-domain`)

`forge/status.rs`, beside `forge/changes.rs`:

```rust
pub fn watch(project: u32, limits: &Limits) -> Option<Watchable>;
pub fn ask(project: u32, limits: &Limits) -> Option<Ask>;
pub fn document(record: &Family, limits: Limits) -> Option<w::ProjectStatus>;
pub(crate) enum Answer { Looked, Unknown, Refused }
pub(crate) fn answer(outcome: &jig_client::Outcome, limits: Limits) -> Answer;
```

`pages/home.rs`'s `Facts` gains `pub status: Status`:

```rust
pub enum Status {
    /// Not delivered, the page's look not yet answered, or its watch refused or ended.
    Unknown,
    Shown(Box<w::ProjectStatus>),
}
impl Facts {
    /// `landed - seen`, 0 until shown.
    pub fn landed_since(&self) -> u64;
    /// The brief, where its `landed` is above `seen`.
    pub fn brief(&self) -> Option<&w::StatusBrief>;
}
```

## The view (`temper-client-view`)

`pages/home.rs`'s one function for the intro, which `ui-away` extends:

```rust
struct Intro { kicker: Box<[u8]>, headline: Box<[u8]>, lede: Option<Box<[u8]>>, byline: Option<Box<[u8]>> }
fn intro_words(w: &Words, name: &[u8], facts: &Facts, times: &mut Times) -> Intro;
```

`TimeForm::Looked`, the kicker's span, in `words.rs`. `pages::changes`' line takes its facts, for Home and Changes:

```rust
pub(crate) struct LandingFacts<'a> { pub queued: u64, pub last: Option<Wall>, pub branch: Option<&'a [u8]>, pub red: &'a [&'a [u8]] }
pub(crate) fn landing_line(facts: &LandingFacts, words: &Words, times: &mut Times) -> Box<[LinePart]>;
```
