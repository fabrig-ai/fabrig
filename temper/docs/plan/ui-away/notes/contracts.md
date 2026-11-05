# ui-away: the run's binding shapes

What crosses tasks. A task builds to these; a name here is never another task's to change. The design is
`temper/docs/design/domain/forge.md`, 5.8 and 5.9; `protocol/store.md`, 2 and 3; `client/README.md`, 3.5 (*Away*),
4.6, 5.12 and section 8; `domain/agent.md`, 4.3; `protocol/configuration.md`, 13.4.

## The wire (`temper-client-protocol-wire/schema/client-wire-v1.schema`)

Each enum's new arm goes last. Times are nanoseconds since the epoch, as `ChangesLanded.at` is.

```
record SetAway { project: u32 on: bool until: option u64 kinds: u8 }
record Overturn { project: u32 number: u32 }
enum Request { ..., set_away: SetAway, overturn: Overturn }
record DecidedSubject { project: u32 }
enum Watch { ..., away, decided: DecidedSubject }
record AwayRow { project: u32 since: u64 until: option u64 kinds: u8 landed: u64 off: option u64 }
versioned record Aways { rows: list 1024 AwayRow }
record Decision { number: u32 task: u64 at: u64 question: bytes 400 took: bytes 120 why: bytes 400 overturned: bool }
versioned record Decisions { project: u32 rows: list 64 Decision }
enum OverturnRefusal { already, unknown }
record OverturnRefused { reason: OverturnRefusal }
enum Outcome { ..., away, overturned, overturn_refused: OverturnRefused }
```

In `temper-client-protocol-wire/src/lib.rs`, beside `project_status` and `decode_project_status`:

```rust
pub fn aways(record: &v1::Aways) -> Box<[u8]>;
pub fn decode_aways(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Aways>;
pub fn decisions(record: &v1::Decisions) -> Box<[u8]>;
pub fn decode_decisions(bytes: &[u8], limits: &v1::Limits) -> Option<v1::Decisions>;
```

A request whose `kinds` is above 15 fails the request validation.

## The store (`temper-protocol-store`)

Byte 38 `AWAY`, byte 39 `DECIDED` in `keys.rs`, or the next free where another plan took them. `Away`'s key is the
party, then the project; `Decided`'s is the project, then the number.

```
record Away { party: u64 project: u32 since: u64 until: option u64 kinds: u8 landed: u64 off: option u64 }
record Decided { project: u32 number: u32 party: u64 task: u64 at: u64 question: bytes 400 options: list 6 bytes 120 took: u8 why: bytes 400 overturned: bool }
enum Value { ..., away: Away, decided: Decided }
record CallDecided { took: option bytes 120 }
enum CallPayload { ..., decided: CallDecided }
```

`CallPayload`'s arm is `decided-tool`'s; the rest is `away-forge`'s.

## The forge (`temper-engine-domain-forge/src/away.rs`, a new module beside `dismissed.rs` and `status.rs`)

```rust
/// The subject `away`, one number, each watcher's own rows.
pub const AWAY_SUBJECT: u64 = 1 << 60;
pub const DECIDED_PER_PROJECT: usize = 64;
pub const DECIDED_QUESTION_BYTES: usize = 400;
pub const DECIDED_OPTION_BYTES: usize = 120;
pub const DECIDED_WHY_BYTES: usize = 400;
pub const DECIDED_OPTIONS: core::ops::RangeInclusive<usize> = 2..=6;
/// The subject `decided`, beside `status_subject`.
pub fn decided_subject(project: u32) -> u64 { (1 << 62) | (1 << 35) | u64::from(project) }

pub enum DecisionKind { Implementation, Retry, Wording, Dependency, Other }
impl DecisionKind {
    /// Its bit in a row's `kinds`: 1, 2, 4, 8; `Other` has none.
    pub fn bit(self) -> Option<u8>;
}
pub struct Away { pub party: u64, pub project: u32, pub since: Wall, pub until: Option<Wall>, pub kinds: u8, pub landed: u64, pub off: Option<Wall> }
impl Away {
    /// `off` none, and `until` none or after `wall` (temper/docs/design/domain/forge.md, 5.8).
    pub fn on_at(&self, wall: Wall) -> bool;
}
pub struct Decided { pub project: u32, pub number: u32, pub party: u64, pub task: u64, pub at: Wall, pub question: Box<[u8]>, pub options: Box<[Box<[u8]>]>, pub took: u8, pub why: Box<[u8]>, pub overturned: bool }
/// What the tool's call says, its bounds checked at decoding.
pub struct DecidedCall { pub question: Box<[u8]>, pub options: Box<[Box<[u8]>]>, pub took: u8, pub why: Box<[u8]>, pub kind: DecisionKind }
pub enum SetAwayAnswer { Away, TooLarge }
pub enum OverturnAnswer { Overturned, Already, Unknown }
pub struct DecisionView { pub number: u32, pub task: u64, pub at: Wall, pub question: Box<[u8]>, pub took: Box<[u8]>, pub why: Box<[u8]>, pub overturned: bool }
pub struct AwaysView { pub rows: Box<[Away]> }
pub struct DecisionsView { pub project: u32, pub rows: Box<[DecisionView]> }
```

Each derives `Clone, PartialEq, Eq, Debug`, and `Copy` where it can, and is exported from the crate as `Dismissal` is.
In the crate's vocabulary:

```rust
Stored::Away(Away)    Stored::Decided(Decided)
Key::Away { party: u64, project: u32 }    Key::Decided { project: u32, number: u32 }
Event::SetAway { request: Token, token: Token, party: u64, project: u32, on: bool, until: Option<Wall>, kinds: u8 }
Event::Overturn { request: Token, token: Token, party: u64, project: u32, number: u32 }
/// `to` and `key` carried as `Request::Adopted`'s `reply_to` is, and answered in the same decision.
Event::Decided { to: ReplyTo, key: NamedCall, party: u64, project: u32, task: u64, call: DecidedCall }
Request::Decided { to: ReplyTo, key: NamedCall, took: Option<Box<[u8]>> }
Answered::SetAway(SetAwayAnswer)    Answered::Overturn(OverturnAnswer)
Document::Aways(AwaysView)    Document::Decisions(DecisionsView)
```

## The root and the tool (`temper-engine-domain`, `temper-engine-smith`)

```rust
Application::SetAway { project: u32, on: bool, until: Option<Wall>, kinds: u8 }   // scoped as Dismiss
Application::Overturn { project: u32, number: u32 }                               // scoped as Dismiss
ApplicationAnswer::SetAway(forge::SetAwayAnswer)    ApplicationAnswer::Overturn(forge::OverturnAnswer)
Tool::Decided(forge::DecidedCall)
CallAnswer::Decided { took: Option<Box<[u8]>> }    // a named call's: forge::CallPayload::Decided { took }
```

## The client (`temper-client-domain`)

`forge/away.rs`, shaped as `forge/dismissed.rs`, a watch of the party's own:

```rust
pub const DEFAULT_KINDS: u8 = 7;
pub struct AwayRow { pub project: u32, pub since: Wall, pub until: Option<Wall>, pub kinds: u8, pub landed: u64, pub off: Option<Wall> }
impl AwayRow {
    pub fn on_at(&self, wall: Wall) -> bool;
    /// When its spell ended: `off`, or `until` once past; none while on.
    pub fn ended(&self, wall: Wall) -> Option<Wall>;
}
/// The first wall after `wall` whose hours in the zone read 08:00.
pub fn next_eight(wall: Wall, zone_minutes: i32) -> Wall;
pub fn watch(limits: &Limits) -> Option<Watchable>;
pub fn ask(project: u32, on: bool, until: Option<Wall>, kinds: u8, limits: &Limits) -> Option<Ask>;
impl Side { pub fn away(&self, project: u32) -> Option<AwayRow>; }
```

`forge/decided.rs`, shaped as `forge/status.rs`, a page's watch of the project's:

```rust
pub fn watch(project: u32, limits: &Limits) -> Option<Watchable>;
pub fn overturn(project: u32, number: u32, limits: &Limits) -> Option<Ask>;
pub fn document(record: &Family, limits: Limits) -> Option<w::Decisions>;
```

`domain.rs`:

```rust
impl Domain {
    /// Minutes east of universal time: the shell's input (temper/docs/design/client/README.md, 5.12).
    pub fn zone_minutes(&self) -> i32;
    pub fn away_on(&self, project: u32, wall: Wall) -> bool;
    /// The project has a goal, the party's row is on at `wall`, or `Ways.away`.
    pub fn away_shown(&self, project: u32, wall: Wall) -> bool;
}
```

`pages/away.rs`, the switch's acts, reached as `pages::Act::Away(away::Act)`:

```rust
pub enum Act {
    Toggle { project: u32 },
    Kinds { project: u32, kinds: u8 },
    Until { project: u32, eight: bool },
    /// "Limits": opens Settings at the section (client/README.md, 3.5, *Away*).
    Limits { project: u32 },
}
```

Needs' `Toasted` gains: `AwayOn { project: u32, until: Option<Wall> }`, `AwayOff`, `AwayRefused`, `Overturned`,
`OverturnEnded`, `OverturnAlready`. `pages/home.rs`'s `Facts` gains `pub away: Option<AwayForm>`, none while To begin
shows or with no row showing:

```rust
pub struct AwayForm {
    /// `since` and its end; none with the party's row undelivered.
    pub span: Option<(Wall, Option<Wall>)>,
    /// `landed` less the row's `landed`; none with either undelivered.
    pub landed: Option<u64>,
    /// Every row showing, past the 7 drawn.
    pub showing: u32,
    /// The newest 7 showing, oldest first.
    pub rows: Box<[DecisionLine]>,
    /// When the oldest row stops showing.
    pub expires: Wall,
}
pub struct DecisionLine { pub number: u32, pub task: u64, pub title: Option<Box<[u8]>>, pub at: Wall, pub question: Box<[u8]>, pub took: Box<[u8]>, pub why: Box<[u8]>, pub overturned: bool, pub state: DecisionTask }
pub enum DecisionTask { Unknown, Live, Done(Wall), Ended(Wall) }
```

## The view (`temper-client-view`)

`look.rs`: `AWAY`, `AWAY_CHIP`, `AWAY_TRACK`, `DECIDED`, `DECISION`, `DECISION_TOOK`, `DECISION_WHY`, `DECISION_LATE`,
`DECISION_OVERTURNED`, named `temper-away`, `temper-away-chip`, `temper-away-track`, `temper-decided`,
`temper-decision`, `temper-decision-took`, `temper-decision-why`, `temper-decision-late`,
`temper-decision-overturned`. `frame::rail` takes `&mut Times`. `pages/home.rs`' `intro_words` takes the away form as it
does the rest of `Facts`.

## The goal's words

A goal started with "Always ask" holds the line `Asks: always` in its words, exactly. The root's rail, the charter's
sentence and the goal page's "How it runs" read that line; nothing stores it apart.
