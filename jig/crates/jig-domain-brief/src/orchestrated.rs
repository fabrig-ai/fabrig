//! Brief gathering across owners (domain/engine/briefs.md, section 1). The brief
//! keeps the core's text, but a connector's section is only its number,
//! kind, fixed name, token and size. Its owner keeps the bytes through assignment.
//!
//! One deadline covers gathering and cuts; a settle point seven eighths of the
//! way there stops optional gathering to leave time for cuts (domain/engine/briefs.md,
//! section 1). A required section that does not arrive fails the brief; an optional
//! one leaves a missing marker. Abandoning a brief drops every connector token it
//! was handed.

use alloc::boxed::Box;
use jig_domain_contract::parts::{Failure, Fate};

use skein_lib::{Deadlines, Decimal, Env, List, Map, Queue, Time, Token, Writer, length::len_u64};

use crate::limits::Limits;
use crate::planned::{self, ConnectorAction, Core, Planned};

/// Why a section is absent from a completed brief.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Missing {
    /// Its owner could not gather it.
    Failed,
    /// Optional gathering passed the settle point, or gathering or cutting passed the deadline.
    Late,
    /// It could not fit within the brief's byte budget.
    Budget,
}

/// A completed section. The root takes connector bytes from their owners.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Placed {
    /// Text retained and cut by the brief itself.
    Core { kind: Core, text: Box<[u8]> },
    /// Optional core text that was left out.
    CoreMissing { kind: Core, why: Missing },
    /// An opaque connector section, still held by that connector.
    Connector { connector: u16, kind: u16, token: Token, size: u32 },
    /// An optional section omitted, with its reason.
    Missing { connector: u16, kind: u16, why: Missing },
}

/// Inputs to the brief's gathering machine.
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Event {
    /// Start one brief, with the core's words and connector section tokens.
    Plan { brief: Token, budget: u32, deadline: Time, sections: Box<[Planned]> },
    /// A connector gathered or cut a section and retains its bytes.
    Sized { brief: Token, connector: u16, section: Token, size: u32 },
    /// A connector could not gather or cut a section.
    Missing { brief: Token, connector: u16, section: Token, why: Failure },
    /// An amendment or cancellation withdrew this brief.
    Abandon { brief: Token },
}

/// Requests to the parent, which routes connector handoffs by number.
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    /// Gather one connector section under this read bound.
    Gather { connector: u16, brief: Token, section: Token, budget: u32, deadline: Time },
    /// Ask its owner to cut a retained section to this bound.
    CutTo { connector: u16, brief: Token, section: Token, size: u32 },
    /// Release a retained or outstanding connector section.
    Drop { connector: u16, brief: Token, section: Token },
    /// Take sections in order from their owners for the assignment.
    Complete { brief: Token, order: Box<[Placed]> },
    /// A required section could not be prepared.
    Failed { brief: Token, why: Missing, failure: Failure },
    /// Admission refused the brief.
    Refused { brief: Token, failure: Failure },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Status {
    Ready,
    Gathering,
    Cutting,
    Missing(Missing),
    Dropped(Missing),
}

#[derive(Hash, Debug)]
struct Active {
    sections: List<Planned>,
    status: List<Status>,
    allotted: List<u32>,
    budget: u32,
    waiting: u32,
    deadline: Time,
    settled: bool,
}

/// Bounded active briefs and their single timer each, for settling then expiry.
#[derive(Hash, Debug)]
pub struct Domain {
    briefs: Map<Token, Active>,
    deadlines: Deadlines<Token>,
}

impl Domain {
    /// Allocate room for the configured number of concurrent briefs.
    #[must_use]
    pub fn new(limits: &Limits) -> Domain {
        Domain { briefs: Map::with_capacity(limits.briefs), deadlines: Deadlines::with_capacity(limits.briefs) }
    }

    /// The earliest settle point or brief deadline.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.deadlines.next()
    }

    /// Whether any brief is still gathering or being cut.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.briefs.is_empty()
    }
}

/// Representability of the child limits (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    limits.briefs > 0
        && limits.sections > 0
        && limits.read_bytes > 0
        && limits.brief_bytes > 0
        && limits.gather.as_nanos() > 0
}

/// Advance one brief or accept a new one.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    match event {
        Event::Plan { brief, budget, deadline, sections } => begin(domain, env, brief, budget, deadline, sections, out),
        Event::Sized { brief, connector, section, size } => {
            update(domain, env, brief, connector, section, Ok(size), out);
        }
        Event::Missing { brief, connector, section, why } => {
            update(domain, env, brief, connector, section, Err(why), out);
        }
        Event::Abandon { brief } => {
            if let Some(active) = domain.briefs.remove(&brief) {
                domain.deadlines.cancel(brief);
                drop_tokens(brief, &active, out);
            }
        }
    }
}

/// Settle or expire the earliest due brief after the stage has taken today's inputs.
pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(brief) = domain.deadlines.expire(env.now) else { return };
    let Some(mut active) = domain.briefs.remove(&brief) else { return };
    if env.now < active.deadline && !active.settled {
        settle(brief, &mut active, out);
        active.settled = true;
        domain.deadlines.arm(brief, active.deadline).expect("admitted brief deadline");
        if active.waiting == 0 {
            allocate(domain, brief, active, false, out);
        } else {
            domain.briefs.insert(brief, active).expect("same active brief");
        }
        return;
    }
    let mut failure = None;
    let mut cutting = false;
    for index in 0..active.sections.len() {
        let status = active.status.get_mut(index).expect("section status");
        match status {
            Status::Gathering | Status::Cutting => {
                if *status == Status::Cutting {
                    cutting = true;
                }
                let section = active.sections.get(index).expect("section");
                if section.required() && failure.is_none() {
                    failure = Some(match status {
                        Status::Gathering => late(section),
                        Status::Cutting => uncut(section),
                        Status::Ready | Status::Missing(_) | Status::Dropped(_) => unreachable!("waiting section"),
                    });
                }
                *status = Status::Missing(Missing::Late);
            }
            Status::Ready | Status::Missing(_) | Status::Dropped(_) => {}
        }
    }
    active.waiting = 0;
    if let Some(failure) = failure {
        fail(brief, &active, Missing::Late, failure, out);
    } else if cutting {
        drop_missing(brief, &mut active, out);
        complete(brief, active, out);
    } else {
        allocate(domain, brief, active, true, out);
    }
}

fn settle(brief: Token, active: &mut Active, out: &mut Queue<Request>) {
    for index in 0..active.sections.len() {
        let section = active.sections.get(index).expect("section");
        let status = active.status.get_mut(index).expect("section status");
        match *status {
            Status::Gathering if !section.required() => {
                *status = Status::Missing(Missing::Late);
                active.waiting = active.waiting.checked_sub(1).expect("an outstanding section");
            }
            Status::Gathering | Status::Cutting | Status::Ready | Status::Missing(_) | Status::Dropped(_) => {}
        }
    }
    drop_missing(brief, active, out);
}

fn begin(
    domain: &mut Domain,
    env: &Env<Limits>,
    brief: Token,
    budget: u32,
    deadline: Time,
    sections: Box<[Planned]>,
    out: &mut Queue<Request>,
) {
    let too_many = permanent(b"brief", b"it has more sections than a brief may hold");
    let Ok(count) = u32::try_from(sections.len()) else {
        out.push(Request::Refused { brief, failure: too_many });
        return;
    };
    let failure = if domain.briefs.contains_key(&brief) || domain.briefs.len() == env.limits.briefs {
        Some(Failure { fate: Fate::Capacity, section: b"brief", words: Box::from(b"no brief slot is free".as_slice()) })
    } else if count > env.limits.sections {
        Some(too_many)
    } else if budget > env.limits.brief_bytes {
        Some(permanent(b"brief", b"its budget is past the limit"))
    } else if deadline <= env.now {
        Some(permanent(b"brief", b"its deadline has passed"))
    } else if !distinct(&sections) {
        Some(permanent(b"brief", b"a section is planned twice"))
    } else {
        None
    };
    if let Some(failure) = failure {
        out.push(Request::Refused { brief, failure });
        return;
    }
    for section in &sections {
        match section {
            Planned::Core { kind, text, .. } => {
                if text.len() > skein_lib::length::usize_of(env.limits.brief_bytes) {
                    let failure = permanent(planned::core_name(*kind), b"its text is past the brief's limit");
                    out.push(Request::Refused { brief, failure });
                    return;
                }
            }
            Planned::Connector { .. } => {}
        }
    }
    let span = u128::from(deadline.saturating_since(env.now).as_nanos());
    let nanos = span.saturating_mul(7).checked_div(8).expect("positive divisor");
    let settle_at = env
        .now
        .saturating_add(skein_lib::Duration::from_nanos(u64::try_from(nanos).expect("seven eighths of a span fits")));
    let mut active = Active {
        sections: List::with_capacity(count),
        status: List::with_capacity(count),
        allotted: List::with_capacity(count),
        budget,
        waiting: 0,
        deadline,
        settled: settle_at == deadline,
    };
    for section in sections {
        let status = match &section {
            Planned::Core { .. } => Status::Ready,
            Planned::Connector { connector, section } => {
                out.push(Request::Gather {
                    connector: *connector,
                    brief,
                    section: section.token,
                    budget: env.limits.read_bytes,
                    deadline,
                });
                active.waiting = active.waiting.checked_add(1).expect("bounded sections");
                Status::Gathering
            }
        };
        active.sections.push(section).expect("measured inventory");
        active.status.push(status).expect("measured inventory");
        active.allotted.push(0).expect("measured inventory");
    }
    if active.waiting == 0 {
        allocate(domain, brief, active, false, out);
    } else {
        domain.deadlines.arm(brief, settle_at).expect("admitted brief deadline");
        domain.briefs.insert(brief, active).expect("admitted brief room");
    }
}

fn distinct(sections: &[Planned]) -> bool {
    for (index, section) in sections.iter().enumerate() {
        let (connector, token) = match section {
            Planned::Connector { connector, section } => (*connector, section.token),
            Planned::Core { .. } => continue,
        };
        for previous in sections.get(..index).expect("prefix of inventory") {
            match previous {
                Planned::Connector { connector: owner, section } if *owner == connector && section.token == token => {
                    return false;
                }
                Planned::Connector { .. } | Planned::Core { .. } => {}
            }
        }
    }
    true
}

fn update(
    domain: &mut Domain,
    env: &Env<Limits>,
    brief: Token,
    connector: u16,
    section: Token,
    size: Result<u32, Failure>,
    out: &mut Queue<Request>,
) {
    let Some(mut active) = domain.briefs.remove(&brief) else { return };
    let Some(index) = locate(&active, connector, section) else {
        domain.briefs.insert(brief, active).expect("same active brief");
        return;
    };
    let status = active.status.get_mut(index).expect("section status");
    let cutting = match *status {
        Status::Gathering => false,
        Status::Cutting => true,
        Status::Ready | Status::Missing(_) | Status::Dropped(_) => {
            domain.briefs.insert(brief, active).expect("same active brief");
            return;
        }
    };
    active.waiting = active.waiting.checked_sub(1).expect("an outstanding section");
    let row = active.sections.get_mut(index).expect("section");
    match size {
        Ok(reported)
            if reported <= env.limits.read_bytes
                && (!cutting || reported <= *active.allotted.get(index).expect("cut allotment")) =>
        {
            match row {
                Planned::Connector { section, .. } => section.size = reported,
                Planned::Core { .. } => unreachable!("matched connector section"),
            }
            *status = Status::Ready;
        }
        Ok(_) | Err(_) => {
            let failure = match size {
                Ok(_) => permanent(section_name(row), b"answered past its bound"),
                Err(failure) => failure,
            };
            if row.required() {
                *status = Status::Missing(Missing::Failed);
                domain.deadlines.cancel(brief);
                fail(brief, &active, Missing::Failed, failure, out);
                return;
            }
            *status = Status::Dropped(Missing::Failed);
            drop_one(brief, row, out);
        }
    }
    if active.waiting == 0 {
        if cutting {
            domain.deadlines.cancel(brief);
            complete(brief, active, out);
        } else {
            allocate(domain, brief, active, false, out);
        }
    } else {
        domain.briefs.insert(brief, active).expect("same active brief");
    }
}

fn allocate(domain: &mut Domain, brief: Token, mut active: Active, expired: bool, out: &mut Queue<Request>) {
    drop_missing(brief, &mut active, out);
    let count = active.sections.len();
    let mut available = List::with_capacity(count);
    let mut positions = List::with_capacity(count);
    for (index, section) in active.sections.iter().enumerate() {
        let status = active.status.get(u32::try_from(index).expect("bounded index")).expect("section status");
        match status {
            Status::Ready => {
                available.push(section.clone()).expect("measured inventory");
                positions.push(u32::try_from(index).expect("bounded index")).expect("measured inventory");
            }
            Status::Missing(_) | Status::Dropped(_) => {}
            Status::Gathering | Status::Cutting => unreachable!("allocation follows handoffs"),
        }
    }
    let Some(plan) = planned::plan(available.as_slice(), active.budget) else {
        domain.deadlines.cancel(brief);
        fail(brief, &active, Missing::Budget, permanent(b"brief", b"its required sections do not fit its budget"), out);
        return;
    };
    for placement in &plan.order {
        let index = *positions.get(placement.index).expect("planned section");
        *active.allotted.get_mut(index).expect("planned section") = placement.size;
    }
    for index in 0..active.sections.len() {
        let section = active.sections.get(index).expect("section");
        let allotted = *active.allotted.get(index).expect("allotment");
        match section {
            Planned::Core { text, required, .. }
                if allotted < u32::try_from(text.len()).expect("admitted core text") =>
            {
                if *required && !can_cut(text, allotted) {
                    domain.deadlines.cancel(brief);
                    fail(
                        brief,
                        &active,
                        Missing::Budget,
                        permanent(section_name(section), b"does not fit the brief's budget"),
                        out,
                    );
                    return;
                }
            }
            Planned::Core { .. } | Planned::Connector { .. } => {}
        }
    }
    if expired {
        for action in &plan.connectors {
            match action {
                ConnectorAction::CutTo { connector, token, .. } => {
                    let index = locate(&active, *connector, *token).expect("planned connector token");
                    if active.sections.get(index).expect("section").required() {
                        domain.deadlines.cancel(brief);
                        fail(brief, &active, Missing::Late, uncut(active.sections.get(index).expect("section")), out);
                        return;
                    }
                }
                ConnectorAction::Drop { .. } => {}
            }
        }
    }
    for action in plan.connectors {
        match action {
            ConnectorAction::CutTo { connector, token, size } => {
                let index = locate(&active, connector, token).expect("planned connector token");
                let section = active.sections.get(index).expect("section");
                if expired || size == 0 {
                    if section.required() {
                        domain.deadlines.cancel(brief);
                        fail(
                            brief,
                            &active,
                            Missing::Budget,
                            permanent(section_name(section), b"does not fit the brief's budget"),
                            out,
                        );
                        return;
                    }
                    *active.status.get_mut(index).expect("section status") = Status::Dropped(Missing::Late);
                    out.push(Request::Drop { connector, brief, section: token });
                } else {
                    *active.status.get_mut(index).expect("section status") = Status::Cutting;
                    active.waiting = active.waiting.checked_add(1).expect("bounded sections");
                    out.push(Request::CutTo { connector, brief, section: token, size });
                }
            }
            ConnectorAction::Drop { connector, token } => {
                let index = locate(&active, connector, token).expect("planned connector token");
                *active.status.get_mut(index).expect("section status") = Status::Dropped(Missing::Budget);
                out.push(Request::Drop { connector, brief, section: token });
            }
        }
    }
    if active.waiting == 0 {
        domain.deadlines.cancel(brief);
        complete(brief, active, out);
    } else {
        domain.briefs.insert(brief, active).expect("same active brief");
    }
}

fn locate(active: &Active, connector: u16, token: Token) -> Option<u32> {
    for (index, section) in active.sections.iter().enumerate() {
        match section {
            Planned::Connector { connector: owner, section } if *owner == connector && section.token == token => {
                return u32::try_from(index).ok();
            }
            Planned::Connector { .. } | Planned::Core { .. } => {}
        }
    }
    None
}

fn drop_tokens(brief: Token, active: &Active, out: &mut Queue<Request>) {
    for (index, section) in active.sections.iter().enumerate() {
        let status = active.status.get(u32::try_from(index).expect("bounded index")).expect("section status");
        match status {
            Status::Dropped(_) => {}
            Status::Ready | Status::Gathering | Status::Cutting | Status::Missing(_) => drop_one(brief, section, out),
        }
    }
}

fn drop_one(brief: Token, section: &Planned, out: &mut Queue<Request>) {
    match section {
        Planned::Connector { connector, section } => {
            out.push(Request::Drop { connector: *connector, brief, section: section.token });
        }
        Planned::Core { .. } => {}
    }
}

fn drop_missing(brief: Token, active: &mut Active, out: &mut Queue<Request>) {
    for index in 0..active.sections.len() {
        let status = active.status.get_mut(index).expect("section status");
        match status {
            Status::Missing(why) => {
                let reason = *why;
                drop_one(brief, active.sections.get(index).expect("section"), out);
                *status = Status::Dropped(reason);
            }
            Status::Ready | Status::Gathering | Status::Cutting | Status::Dropped(_) => {}
        }
    }
}

fn section_name(section: &Planned) -> &'static [u8] {
    match section {
        Planned::Core { kind, .. } => planned::core_name(*kind),
        Planned::Connector { section, .. } => section.name,
    }
}

fn permanent(section: &'static [u8], words: &'static [u8]) -> Failure {
    Failure { fate: Fate::Permanent, section, words: Box::from(words) }
}

fn late(section: &Planned) -> Failure {
    Failure {
        fate: Fate::Infrastructure,
        section: section_name(section),
        words: Box::from(b"not gathered by its deadline".as_slice()),
    }
}

fn uncut(section: &Planned) -> Failure {
    Failure {
        fate: Fate::Infrastructure,
        section: section_name(section),
        words: Box::from(b"not cut by its deadline".as_slice()),
    }
}

fn fail(brief: Token, active: &Active, why: Missing, failure: Failure, out: &mut Queue<Request>) {
    drop_tokens(brief, active, out);
    out.push(Request::Failed { brief, why, failure });
}

fn complete(brief: Token, active: Active, out: &mut Queue<Request>) {
    let mut order = List::with_capacity(active.sections.len());
    for (index, section) in active.sections.into_iter().enumerate() {
        let index = u32::try_from(index).expect("bounded section count");
        let status = *active.status.get(index).expect("section status");
        let allotted = *active.allotted.get(index).expect("section allotment");
        let placed = match section.clone() {
            Planned::Core { kind, text, required, .. } => {
                if (allotted == 0 && !required && !text.is_empty()) || !can_cut(&text, allotted) {
                    Placed::CoreMissing { kind, why: Missing::Budget }
                } else {
                    Placed::Core { kind, text: cut_core(kind, text, allotted) }
                }
            }
            Planned::Connector { connector, section } => match status {
                Status::Ready => {
                    Placed::Connector { connector, kind: section.kind, token: section.token, size: section.size }
                }
                Status::Missing(why) | Status::Dropped(why) => Placed::Missing { connector, kind: section.kind, why },
                Status::Gathering | Status::Cutting => unreachable!("a brief completes after its handoffs"),
            },
        };
        order.push(placed).expect("one placement per section");
    }
    out.push(Request::Complete { brief, order: order.into_boxed() });
}

fn can_cut(text: &[u8], size: u32) -> bool {
    if text.len() <= skein_lib::length::usize_of(size) {
        return true;
    }
    let lost = len_u64(text.len());
    let marker = b"[".len().saturating_add(Decimal::of(lost).as_bytes().len()).saturating_add(b" bytes cut]\n".len());
    marker <= skein_lib::length::usize_of(size)
}

fn cut_core(kind: Core, text: Box<[u8]>, size: u32) -> Box<[u8]> {
    let room = skein_lib::length::usize_of(size);
    if text.len() <= room {
        return text;
    }
    let tail = match kind {
        Core::Attempts | Core::TranscriptTail => true,
        Core::Task
        | Core::Now
        | Core::Lineage
        | Core::Results
        | Core::Plan
        | Core::Calls
        | Core::Waiting
        | Core::NotesIndex
        | Core::Preferences => false,
    };
    let mut low = 0_usize;
    let mut high = text.len().min(room);
    for _ in 0..usize::BITS {
        if low >= high {
            break;
        }
        let mid = low.saturating_add(high.saturating_sub(low).div_ceil(2));
        let keep = edge(&text, mid, tail);
        let lost = text.len().saturating_sub(keep.len());
        let marker = usize::from(!tail && !keep.is_empty())
            .saturating_add(b"[".len())
            .saturating_add(Decimal::of(len_u64(lost)).as_bytes().len())
            .saturating_add(b" bytes cut]\n".len());
        if keep.len().saturating_add(marker) <= room {
            low = mid;
        } else {
            high = mid.saturating_sub(1);
        }
    }
    let keep = edge(&text, low, tail);
    let lost = text.len().saturating_sub(keep.len());
    let digits = Decimal::of(len_u64(lost));
    let marker = usize::from(!tail && !keep.is_empty())
        .saturating_add(b"[".len())
        .saturating_add(digits.as_bytes().len())
        .saturating_add(b" bytes cut]\n".len());
    let mut writer = Writer::new(keep.len().saturating_add(marker));
    if tail {
        writer.put(b"[").expect("measured marker");
        writer.put(digits.as_bytes()).expect("measured count");
        writer.put(b" bytes cut]\n").expect("measured marker");
        writer.put(keep).expect("measured retained text");
    } else {
        writer.put(keep).expect("measured retained text");
        if !keep.is_empty() {
            writer.put(b"\n").expect("measured break");
        }
        writer.put(b"[").expect("measured marker");
        writer.put(digits.as_bytes()).expect("measured count");
        writer.put(b" bytes cut]\n").expect("measured marker");
    }
    writer.finish()
}

fn edge(text: &[u8], wanted: usize, tail: bool) -> &[u8] {
    if tail {
        let mut start = text.len().saturating_sub(wanted);
        for _ in 0..3 {
            if continues(text.get(start)) {
                start = start.saturating_add(1);
            }
        }
        text.get(start..).expect("within text")
    } else {
        let mut end = wanted;
        for _ in 0..3 {
            if continues(text.get(end)) {
                end = end.saturating_sub(1);
            }
        }
        text.get(..end).expect("within text")
    }
}

fn continues(byte: Option<&u8>) -> bool {
    match byte {
        Some(byte) => byte & 0b1100_0000 == 0b1000_0000,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_domain_contract::parts::Section;
    use skein_lib::{Duration, Wall};

    const DEADLINE: Time = Time::ZERO.saturating_add(Duration::from_secs(1));

    fn env() -> Env<Limits> {
        let limits =
            Limits { briefs: 2, sections: 3, read_bytes: 256, brief_bytes: 400, gather: Duration::from_secs(30) };
        Env { now: Time::ZERO, wall: Wall::EPOCH, limits }
    }

    fn outputs(domain: &mut Domain, event: Event) -> Box<[Request]> {
        let env = env();
        let mut out = Queue::uncapped();
        step(domain, &env, event, &mut out);
        let mut list = List::with_capacity(out.len());
        for _ in 0..out.len() {
            list.push(out.pop().expect("counted output")).expect("bounded output");
        }
        list.into_boxed()
    }

    #[test]
    fn a_connector_section_is_only_a_token_and_size_until_completion() {
        let env = env();
        let mut domain = Domain::new(&env.limits);
        let brief = Token::new(4);
        let section = Token::new(9);
        assert_eq!(
            outputs(
                &mut domain,
                Event::Plan {
                    brief,
                    budget: 20,
                    deadline: DEADLINE,
                    sections: Box::new([
                        Planned::Core {
                            kind: Core::Task,
                            text: Box::from(&b"task"[..]),
                            limit: 20,
                            priority: 0,
                            required: true
                        },
                        Planned::Connector {
                            connector: 2,
                            section: Section {
                                name: b"section",
                                kind: 7,
                                token: section,
                                size: 0,
                                limit: 20,
                                priority: 1,
                                required: false,
                                document: None,
                            },
                        },
                    ]),
                }
            )
            .as_ref(),
            [Request::Gather { connector: 2, brief, section, budget: env.limits.read_bytes, deadline: DEADLINE }]
        );
        assert_eq!(
            outputs(&mut domain, Event::Sized { brief, connector: 2, section, size: 5 }).as_ref(),
            [Request::Complete {
                brief,
                order: Box::new([
                    Placed::Core { kind: Core::Task, text: Box::from(&b"task"[..]) },
                    Placed::Connector { connector: 2, kind: 7, token: section, size: 5 },
                ]),
            }]
        );
    }

    #[test]
    fn a_required_missing_section_fails_and_drops_every_token_once() {
        let limits = env().limits;
        let mut domain = Domain::new(&limits);
        let brief = Token::new(7);
        let first = Token::new(1);
        let second = Token::new(2);
        let sections = Box::new([
            Planned::Connector {
                connector: 3,
                section: Section {
                    name: b"section",
                    kind: 1,
                    token: first,
                    size: 0,
                    limit: 100,
                    priority: 0,
                    required: true,
                    document: None,
                },
            },
            Planned::Connector {
                connector: 3,
                section: Section {
                    name: b"section",
                    kind: 2,
                    token: second,
                    size: 0,
                    limit: 100,
                    priority: 1,
                    required: false,
                    document: None,
                },
            },
        ]);
        assert_eq!(
            outputs(&mut domain, Event::Plan { brief, budget: 100, deadline: DEADLINE, sections }).as_ref(),
            [
                Request::Gather { connector: 3, brief, section: first, budget: limits.read_bytes, deadline: DEADLINE },
                Request::Gather { connector: 3, brief, section: second, budget: limits.read_bytes, deadline: DEADLINE },
            ]
        );
        assert_eq!(
            outputs(
                &mut domain,
                Event::Missing { brief, connector: 3, section: first, why: permanent(b"section", b"unavailable") }
            )
            .as_ref(),
            [
                Request::Drop { connector: 3, brief, section: first },
                Request::Drop { connector: 3, brief, section: second },
                Request::Failed { brief, why: Missing::Failed, failure: permanent(b"section", b"unavailable") },
            ]
        );
        assert!(domain.is_idle());
    }

    #[test]
    fn an_optional_late_section_is_dropped_and_marked_missing() {
        let mut env = env();
        let mut domain = Domain::new(&env.limits);
        let brief = Token::new(7);
        let section = Token::new(8);
        let deadline = Time::ZERO.saturating_add(Duration::from_secs(1));
        outputs(
            &mut domain,
            Event::Plan {
                brief,
                budget: 100,
                deadline,
                sections: Box::new([Planned::Connector {
                    connector: 4,
                    section: Section {
                        name: b"section",
                        kind: 5,
                        token: section,
                        size: 0,
                        limit: 100,
                        priority: 0,
                        required: false,
                        document: None,
                    },
                }]),
            },
        );
        env.now = deadline;
        let mut out = Queue::uncapped();
        fire(&mut domain, &env, &mut out);
        assert_eq!(out.pop(), Some(Request::Drop { connector: 4, brief, section }));
        assert_eq!(
            out.pop(),
            Some(Request::Complete {
                brief,
                order: Box::new([Placed::Missing { connector: 4, kind: 5, why: Missing::Late }]),
            })
        );
        assert_eq!(out.pop(), None);
    }

    #[test]
    fn a_connector_cut_keeps_its_token_with_the_owner() {
        let mut domain = Domain::new(&env().limits);
        let brief = Token::new(7);
        let section = Token::new(8);
        outputs(
            &mut domain,
            Event::Plan {
                brief,
                budget: 40,
                deadline: DEADLINE,
                sections: Box::new([Planned::Connector {
                    connector: 4,
                    section: Section {
                        name: b"section",
                        kind: 5,
                        token: section,
                        size: 0,
                        limit: 40,
                        priority: 0,
                        required: true,
                        document: None,
                    },
                }]),
            },
        );
        assert_eq!(
            outputs(&mut domain, Event::Sized { brief, connector: 4, section, size: 50 }).as_ref(),
            [Request::CutTo { connector: 4, brief, section, size: 40 }]
        );
        assert_eq!(
            outputs(&mut domain, Event::Sized { brief, connector: 4, section, size: 38 }).as_ref(),
            [Request::Complete {
                brief,
                order: Box::new([Placed::Connector { connector: 4, kind: 5, token: section, size: 38 }]),
            }]
        );
    }

    const FIRST: u16 = 2;
    const SECOND: u16 = 3;
    const SHARED: Token = Token::new(9);
    const RUN: Token = Token::new(4);
    const SECTION_LIMIT: u32 = 8; // Each section keeps eight bytes at most.
    const READ_SIZE: u32 = 12; // Enough gathered bytes to require both cuts.
    const BUDGET: u32 = 16; // Both sections fit at their cut size.

    fn shared_section(connector: u16, kind: u16, required: bool) -> Planned {
        Planned::Connector {
            connector,
            section: Section {
                token: SHARED,
                kind,
                name: b"section",
                size: 0,
                limit: SECTION_LIMIT,
                priority: 0,
                required,
                document: None,
            },
        }
    }

    #[test]
    fn equal_tokens_are_gathered_cut_and_dropped_by_owner() {
        const FIRST_KIND: u16 = 1;
        const SECOND_KIND: u16 = 2;
        let mut domain = Domain::new(&env().limits);
        outputs(
            &mut domain,
            Event::Plan {
                brief: RUN,
                budget: BUDGET,
                deadline: DEADLINE,
                sections: Box::new([
                    shared_section(FIRST, FIRST_KIND, true),
                    shared_section(SECOND, SECOND_KIND, false),
                ]),
            },
        );
        let waiting =
            outputs(&mut domain, Event::Sized { brief: RUN, connector: SECOND, section: SHARED, size: READ_SIZE });
        let cuts =
            outputs(&mut domain, Event::Sized { brief: RUN, connector: FIRST, section: SHARED, size: READ_SIZE });
        let drop = outputs(
            &mut domain,
            Event::Missing {
                brief: RUN,
                connector: SECOND,
                section: SHARED,
                why: permanent(b"section", b"cut refused"),
            },
        );
        let complete =
            outputs(&mut domain, Event::Sized { brief: RUN, connector: FIRST, section: SHARED, size: SECTION_LIMIT });
        assert_eq!(waiting.as_ref(), []);
        assert_eq!(
            cuts.as_ref(),
            [
                Request::CutTo { connector: FIRST, brief: RUN, section: SHARED, size: SECTION_LIMIT },
                Request::CutTo { connector: SECOND, brief: RUN, section: SHARED, size: SECTION_LIMIT },
            ]
        );
        assert_eq!(drop.as_ref(), [Request::Drop { connector: SECOND, brief: RUN, section: SHARED }]);
        assert_eq!(
            complete.as_ref(),
            [Request::Complete {
                brief: RUN,
                order: Box::new([
                    Placed::Connector { connector: FIRST, kind: FIRST_KIND, token: SHARED, size: SECTION_LIMIT },
                    Placed::Missing { connector: SECOND, kind: SECOND_KIND, why: Missing::Failed },
                ]),
            }]
        );
    }

    #[test]
    fn a_zero_gather_window_is_refused() {
        assert!(!valid(&Limits { gather: Duration::ZERO, ..env().limits }));
    }

    #[test]
    fn the_smallest_positive_gather_window_is_valid() {
        const SMALLEST_WINDOW: Duration = Duration::from_nanos(1);
        assert!(valid(&Limits { gather: SMALLEST_WINDOW, ..env().limits }));
    }

    #[test]
    fn abandoning_gathering_drops_every_connector_token() {
        let mut domain = Domain::new(&env().limits);
        let brief = Token::new(7);
        let section = Token::new(8);
        outputs(
            &mut domain,
            Event::Plan {
                brief,
                budget: 40,
                deadline: DEADLINE,
                sections: Box::new([Planned::Connector {
                    connector: 4,
                    section: Section {
                        name: b"section",
                        kind: 5,
                        token: section,
                        size: 0,
                        limit: 40,
                        priority: 0,
                        required: true,
                        document: None,
                    },
                }]),
            },
        );
        assert_eq!(
            outputs(&mut domain, Event::Abandon { brief }).as_ref(),
            [Request::Drop { connector: 4, brief, section }]
        );
        assert!(domain.is_idle());
    }

    #[test]
    fn a_core_cut_says_how_many_utf8_bytes_it_lost() {
        let text = Box::from("aébcdefghijklmnopqrstuvwxyz".as_bytes());
        let cut = cut_core(Core::Task, text, 18);
        assert!(cut.len() <= 18);
        assert_eq!(cut.as_ref(), b"a\n[27 bytes cut]\n");
    }

    #[test]
    fn a_transcript_cut_keeps_its_end_after_the_cut_marker() {
        let text = Box::from(&b"abcdefghijklmnopqrstuvwxyyz"[..]);
        let cut = cut_core(Core::TranscriptTail, text, 18);
        assert!(cut.len() <= 18);
        assert!(cut.starts_with(b"["));
        assert!(cut.ends_with(b"yz"));
    }
}
