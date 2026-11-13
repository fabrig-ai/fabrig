//! Findings (temper/docs/design/domain/findings.md, 2, 6, 9; domain/areas.md, 3 to 7).
use crate::{
    Request,
    settings::{AreaOwner, ProjectAreas},
};
use alloc::boxed::Box;
use skein_lib::{List, Queue, Wall};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Holder {
    Task(u64),
    Party(u64),
    Role(u32),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Filer {
    Task(u64),
    Party(u64),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FindingKind {
    Qa,
    Flake,
    Red,
    Review,
    Feedback,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Severity {
    Low,
    Medium,
    High,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FindingState {
    New,
    Routed,
    Backlog,
    Dropped,
}
impl FindingState {
    #[must_use]
    pub const fn ended(self) -> bool {
        match self {
            Self::Routed | Self::Dropped => true,
            Self::New | Self::Backlog => false,
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Finding {
    pub project: u32,
    pub number: u32,
    pub kind: FindingKind,
    pub severity: Severity,
    pub area: Option<Box<[u8]>>,
    pub state: FindingState,
    pub title: Box<[u8]>,
    pub body: Box<[u8]>,
    pub filer: Filer,
    pub at: Wall,
    pub why: Option<Box<[u8]>>,
    pub hander: Option<Filer>,
    pub goal: Option<u64>,
    pub reason: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileFinding {
    pub project: u32,
    pub kind: FindingKind,
    pub severity: Severity,
    pub area: Option<Box<[u8]>>,
    pub title: Box<[u8]>,
    pub body: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct HandFinding {
    pub project: u32,
    pub number: u32,
    pub area: Box<[u8]>,
    pub why: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TriageFinding {
    pub project: u32,
    pub number: u32,
    pub act: Triage,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Triage {
    Route(u64),
    Backlog,
    Drop(Box<[u8]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FindingRefusal {
    NoArea(Box<[Box<[u8]>]>),
    Full(u32),
    NotYours(Holder),
    NoFinding,
    NoGoal,
    Ended,
    Words(Box<[u8]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FileFindingAnswer {
    Filed { number: u32 },
    Refused(FindingRefusal),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum HandFindingAnswer {
    Handed,
    Refused(FindingRefusal),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TriageFindingAnswer {
    Triaged,
    Refused(FindingRefusal),
}
/// The deciding snapshot kept for an agent's named finding write (domain/agent/tools.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FindingReply {
    pub project: u32,
    pub number: u32,
    pub area: Option<Box<[u8]>>,
    pub result: Result<FindingDone, FindingRefusal>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FindingDone {
    Filed(Holder),
    Handed(Holder),
    Routed(u64),
    Backlog,
    Dropped,
}
#[must_use]
pub fn reply_bytes(reply: &FindingReply) -> Option<u64> {
    let area = match &reply.area {
        Some(area) => skein_lib::len_u64(area.len()),
        None => 0,
    };
    let mut bytes = 0_u64;
    if let Err(why) = &reply.result {
        match why {
            FindingRefusal::NoArea(names) => {
                bytes = skein_lib::len_u64(names.len()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?;
                for name in names {
                    bytes = bytes.checked_add(skein_lib::len_u64(name.len()))?;
                }
            }
            FindingRefusal::Words(words) => bytes = skein_lib::len_u64(words.len()),
            FindingRefusal::Full(_)
            | FindingRefusal::NotYours(_)
            | FindingRefusal::NoFinding
            | FindingRefusal::NoGoal
            | FindingRefusal::Ended => {}
        }
    }
    area.checked_add(bytes)
}

/// The three-step rule; standing comes from the root's snapshot of the core.
#[must_use]
pub fn owner(
    area: Option<&[u8]>,
    areas: &ProjectAreas,
    triager: Option<AreaOwner>,
    stands: &[(AreaOwner, Holder)],
) -> Holder {
    for row in &areas.areas {
        if area == Some(row.name.as_ref())
            && let Some(holder) = standing(row.owner, stands)
        {
            return holder;
        }
    }
    if let Some(triager) = triager
        && let Some(holder) = standing(triager, stands)
    {
        return holder;
    }
    Holder::Role(0)
}
#[must_use]
pub fn standing(owner: AreaOwner, stands: &[(AreaOwner, Holder)]) -> Option<Holder> {
    if let AreaOwner::Role(role) = owner {
        return Some(Holder::Role(role));
    }
    for (candidate, holder) in stands {
        if *candidate == owner {
            return Some(*holder);
        }
    }
    None
}
/// Filters are part of the subject identity; each write marks every filter.
#[must_use]
pub fn findings_subject(project: u32, state: Option<FindingState>, mine: bool) -> u64 {
    let state = match state {
        None => 0,
        Some(FindingState::New) => 1,
        Some(FindingState::Routed) => 2,
        Some(FindingState::Backlog) => 3,
        Some(FindingState::Dropped) => 4,
    };
    (1 << 59) | (state << 33) | (u64::from(mine) << 32) | u64::from(project)
}
#[must_use]
pub fn findings_filter(subject: u64, project: u32) -> Option<(Option<FindingState>, bool)> {
    for state in [
        None,
        Some(FindingState::New),
        Some(FindingState::Routed),
        Some(FindingState::Backlog),
        Some(FindingState::Dropped),
    ] {
        for mine in [false, true] {
            if subject == findings_subject(project, state, mine) {
                return Some((state, mine));
            }
        }
    }
    None
}
pub fn mark_findings(project: u32, out: &mut Queue<Request>) {
    let mut subjects = List::with_capacity(10);
    for state in [
        None,
        Some(FindingState::New),
        Some(FindingState::Routed),
        Some(FindingState::Backlog),
        Some(FindingState::Dropped),
    ] {
        for mine in [false, true] {
            subjects.push(findings_subject(project, state, mine)).expect("ten finding subjects");
        }
    }
    out.push(Request::Marked { tasks: subjects.into_boxed() });
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FindingRow {
    pub number: u32,
    pub kind: FindingKind,
    pub severity: Severity,
    pub area: Option<Box<[u8]>>,
    pub state: FindingState,
    pub title: Box<[u8]>,
    pub holder: Holder,
    pub at: Wall,
    pub why: Option<Box<[u8]>>,
    pub hander: Option<Filer>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AreaRow {
    pub name: Box<[u8]>,
    pub words: Box<[u8]>,
    pub owner: AreaOwner,
    pub holder: Option<Holder>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Findings {
    pub project: u32,
    pub rows: Box<[FindingRow]>,
    pub areas: Box<[AreaRow]>,
    pub triager: Option<AreaOwner>,
    pub holder: Holder,
    pub cut: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FindingAnswer {
    pub row: FindingRow,
    pub body: Box<[u8]>,
    pub filer: Filer,
    pub goal: Option<u64>,
    pub reason: Option<Box<[u8]>>,
}
impl Finding {
    #[must_use]
    pub fn row(&self, holder: Holder) -> FindingRow {
        FindingRow {
            number: self.number,
            kind: self.kind,
            severity: self.severity,
            area: self.area.clone(),
            state: self.state,
            title: self.title.clone(),
            holder,
            at: self.at,
            why: self.why.clone(),
            hander: self.hander,
        }
    }
}
impl FindingRefusal {
    /// Human refusal with the request's address and, for `NoArea`, its asked name.
    #[must_use]
    pub fn words(&self, project: u32, number: u32, area: &[u8]) -> Box<[u8]> {
        use skein_lib::Decimal;
        let mut out = List::uncapped();
        match self {
            Self::NoArea(names) => {
                append(&mut out, b"no area ");
                append(&mut out, area);
                append(&mut out, b" in project ");
                append(&mut out, Decimal::of(u64::from(project)).as_bytes());
                append(&mut out, b"; its areas: ");
                for (at, name) in names.iter().enumerate() {
                    if at != 0 {
                        append(&mut out, b", ");
                    }
                    append(&mut out, name);
                }
            }
            Self::Full(new) => {
                append(&mut out, b"the project's findings are full: ");
                append(&mut out, Decimal::of(u64::from(*new)).as_bytes());
                append(&mut out, b" new");
            }
            Self::NotYours(holder) => {
                append(&mut out, b"finding ");
                append(&mut out, Decimal::of(u64::from(project)).as_bytes());
                append(&mut out, b"/");
                append(&mut out, Decimal::of(u64::from(number)).as_bytes());
                append(&mut out, b" is not yours: its owner is ");
                let (kind, number): (&[u8], u64) = match holder {
                    Holder::Task(n) => (b"task ", *n),
                    Holder::Party(n) => (b"party ", *n),
                    Holder::Role(n) => (b"role ", u64::from(*n)),
                };
                append(&mut out, kind);
                append(&mut out, Decimal::of(number).as_bytes());
            }
            Self::NoFinding => append(&mut out, b"no finding"),
            Self::NoGoal => append(&mut out, b"not a live goal of the project"),
            Self::Ended => append(&mut out, b"finding is already routed or dropped"),
            Self::Words(field) => append(&mut out, field),
        }
        out.into_boxed()
    }
}
fn append(out: &mut List<u8>, words: &[u8]) {
    for &byte in words {
        out.push(byte).expect("bounded finding refusal");
    }
}
/// A judged write is committed after the root admits the receiving subscription.
#[derive(Hash, Debug)]
pub struct FindingWrite {
    pub row: Finding,
    pub erase: Option<u32>,
}
#[must_use]
#[expect(clippy::disallowed_methods, reason = "the forge judges UTF-8 at its finding boundary")]
pub fn body_fits(body: &[u8]) -> bool {
    body.len() <= 4096 && core::str::from_utf8(body).is_ok()
}
#[must_use]
pub fn reason_fits(reason: &[u8]) -> bool {
    !reason.is_empty() && reason.len() <= 400 && body_fits(reason)
}
