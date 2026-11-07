# look-round-13: the shapes that cross tasks

Binding. A task builds to these and needs no other task's text.

## Look tokens (`connect-look`)

`temper-client-view/src/look.rs` gains one `Style` per class, in `NAMES`, the class `temper-connect-X` for `CONNECT_X`:
`CONNECT` (the class `temper-connect`), `CONNECT_CARDS`, `CONNECT_CARD`, `CONNECT_DEFAULT`, `CONNECT_MUTED`, `CONNECT_YES`,
`CONNECT_ACTS`, `CONNECT_PRIMARY`, `CONNECT_SECONDARY`, `CONNECT_QUIET`, `CONNECT_MORE`, `CONNECT_ROWS`, `CONNECT_ROW`,
`CONNECT_PANEL`, `CONNECT_PAIR`, `CONNECT_ONE`, `CONNECT_TITLE`, `CONNECT_CMD`, `CONNECT_WAITING`, `CONNECT_HOW`,
`CONNECT_STEPS`, `CONNECT_LIVE`, `CONNECT_LATER`, `CONNECT_EMPTY`, `CONNECT_CONFIRM`, `CONNECT_LIST`, `CONNECT_BOUNDED`,
`CONNECT_NONE`, `CONNECT_COUNT`, `CONNECT_SHEET`, `CONNECT_SHEET_HEAD`, `CONNECT_SHEET_BODY`, `CONNECT_SHEET_FOOT`,
`CONNECT_SOURCE`, `CONNECT_SOURCES`, `CONNECT_SAYS`, `CONNECT_ADDRESSES`, `CONNECT_ALERTS`, `CONNECT_FAILED`.

## `DismissMaking` (`dismiss-making`)

- Wire, `temper-client-protocol-wire/schema/client-wire-v1.schema`: `record DismissMaking { forge: u16 org: bytes 1024 name: bytes 1024 project: option u32 }`, the project the failed row's `into` names, read by the client from the row on the `forges` watch; `Request` gains `dismiss_making: DismissMaking`; the wire's `MakeRefusal` gains `not_failed`, and so does the store's (`temper-protocol-store`, where a kept answer holds it).
- Engine, `temper-engine-domain-forge`: `Ask::DismissMaking { forge: u16, org: Box<[u8]>, name: Box<[u8]>, project: Option<u32> }`, answered with `MakeAnswer` (`Kept`, `Refused(MakeRefusal::NotFailed)`, or `Refused(MakeRefusal::NotOwner)`, by role or for a row whose `into` is another project's). The scope is the request's `project`, or the deployment with none, so a repeat of a key has the same scope and jig replays it.
- Client, `temper_client_domain::forge::dismiss_making(forge: u16, org: &[u8], name: &[u8], project: Option<u32>, limits: &w::Limits) -> Option<jig_client::Family>`, beside `family.rs`'s `make_repository`.

## A bound list's `Near` (`list-near`)

`skein_ui::binding::takes(Role::List, _, &Act::Near { edge: LogEdge::Foot })` is true; a `Role::List` bound and marked `State::Continues` reports `Near { edge: LogEdge::Foot }` by `ui-web.md`, 7, and `State::Busy` holds it as a log's does. Nothing else about a list changes, and an unbound list is untouched.

## The sheet's state (`sheet-state`)

`temper_client_domain::forge::sheet`, a new module beside `form.rs`. Everything the view draws is a field or a function here. `Side` holds `sheet: Option<Sheet>`, opened with `Address::Adopt { project }`, beside `form`.

```rust
pub const PICKER_ROWS: usize = 2000;
pub enum Source { Forge, Empty, Copy }
pub struct Pick { pub id: u64, pub owner_id: u64, pub owner: Box<[u8]>, pub name: Box<[u8]> }
pub enum Reading { Idle, Asked { key: Token, page: u32 }, Refused(Refused) }
pub enum Refused { NotConnected, Busy, NoAnswer, AdminMissing, NotAdmin { name: Box<[u8]> }, Role, Words(Box<[u8]>) }
pub struct Sheet {
    pub project: u32,
    pub source: Source,
    pub forge: Option<u16>,            // the Adopt source's; lowest listed connected, at open
    pub installation: Option<u64>,     // GitHub's; the first accepted
    pub target: Option<u16>,           // Empty's and Copy's; lowest owned or managed
    pub find: Field,                   // the search, `form::Field`
    pub rows: List<w::Reached>,        // rows read, in the forge's order, at most PICKER_ROWS
    pub page: u32,                     // the last page read, 0 for none
    pub more: bool,
    pub reading: Reading,
    pub picks: List<Pick>,             // survive a search; cleared by a new forge or installation
    pub not_listed: (Field, Field),    // owner, name
    pub org: Field,
    pub name: Field,
    pub addresses: Field,
    pub sent: Sent,
}
pub enum Sent { Idle, Adopting { left: List<Pick> }, Making { left: u32 }, Refused(Box<[u8]>) }

pub enum Act {
    Source(Source), Forge(u16), Installation(u64),
    Find(Box<[u8]>), Pick { id: u64, on: bool }, NearEnd,
    NotListed { owner: Option<Box<[u8]>>, name: Option<Box<[u8]>> },
    Org(Box<[u8]>), Name(Box<[u8]>), Addresses(Box<[u8]>), Target(u16),
    UseIt { line: Box<[u8]> }, AdoptIt { org: Box<[u8]>, name: Box<[u8]> },
    Send,
}

/// The rows the search leaves, with each one's mark. Case-insensitive substring of OWNER/NAME.
pub fn shown(sheet: &Sheet, project: u32, rows: &State) -> List<Shown<'_>>;
pub struct Shown<'a> { pub row: &'a w::Reached, pub mark: Mark, pub picked: bool }
pub enum Mark { None, Private, InProject, Copying }  // `InProject` and `Copying` are disabled
/// `Count::Hidden` with a forge of few (8 or fewer read and none more).
pub enum Count { Hidden, SoFar(u32), Found { found: u32, of: u32 }, Searching(u32), All(u32), Suspended, Full }
pub fn count(sheet: &Sheet, ...) -> Count;
/// Whether the find field and bounded list show: more than 8 read, or more to read.
pub const fn long(sheet: &Sheet) -> bool;
/// What "Not listed" says (`forge.md`, 5.1), and whether it holds the button back.
pub fn remedy(sheet: &Sheet, row: &Row) -> Option<Remedy>;
pub enum Remedy { Grant { owner: Box<[u8]>, name: Box<[u8]>, page: Box<[u8]> }, NotInstalled { owner: Box<[u8]> }, Collaborator { login: Box<[u8]>, owner: Box<[u8]>, name: Box<[u8]> } }
/// Each non-empty line of the addresses. `forge::making::lines(text, has: &dyn Fn(&[u8], &[u8]) -> bool)`.
pub enum Line { To { address: Box<[u8]>, org: Box<[u8]>, name: Box<[u8]> }, Bad { raw: Box<[u8]> }, NotAHost { raw: Box<[u8]>, line: Box<[u8]> }, Taken { raw: Box<[u8]>, org: Box<[u8]>, name: Box<[u8]> } }
/// What the button says and whether it sends: `Button::Adopt { count, first }`, `Make { full }`, `Copy { count, first }`.
pub fn button(sheet: &Sheet, ...) -> (Button, bool);
```

The implementer may reshape what is private, and add what the sketch misses; the names above bind `sheet-view`, `sheet-copies` and `sheet-rows`.

Reads: `forge::family::reachable(forge, installation, page, limits)` beside `make_repository`, as `temper-client-tools/src/forges/repositories.rs` builds it. Read page 1 when the Adopt source opens or its forge or installation is chosen, and when Copy opens, of the target forge. Read the next at `NearEnd` with no search, and at once on each answer while a search is typed and more follow. One read out at a time. A page with no rows sets `more` false. Past `PICKER_ROWS` stop reading; `Count::Full`.

## What the view binds (`sheet-view`)

`temper-client-view/src/forge/bindings.rs`'s `Meaning` gains `Sheet(sheet::Act)`, and takes `Act::Near { edge: Foot }` on the list as `NearEnd`, `Input` on a field and `Toggle` on a check. `crate::forge::sheet::draw(b, root, w, bindings, forge, project)` in a new `temper-client-view/src/forge/sheet.rs`, called from `pages/settings.rs`'s `repositories` when the address is `Adopt`. Its Cancel, with `Shortcut::ESCAPE`, goes to `Address::Repositories { project }`.
