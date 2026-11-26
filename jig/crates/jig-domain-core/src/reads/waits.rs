//! A project's waits, scanned from live state (jig/docs/design/client/domain/contract/reads.md, 1.2).
use super::{Answer, Cursor, WaitPage, WaitPosition, WaitQuery, WaitRow, Why};
use crate::{Bound, Core, Limits, Wait, WaitObject, documents};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_domain_tasks::{EscalationHolder, Phase};
use skein_lib::{Env, List, Wall, length::usize_of};

pub(super) fn read(core: &Core, env: &Env<Limits>, party: u64, query: WaitQuery, after: Option<&Cursor>) -> Answer {
    if core.people.deleted(query.project) {
        return Answer::None(Why::Gone);
    }
    if !core.open_project(query.project) || core.people.role(party, query.project).is_none() {
        return Answer::None(Why::Unknown);
    }
    let (now, after) = match after {
        Some(Cursor::Waits { query: held, now, after }) if *held == query => (*now, Some(*after)),
        Some(_) => return Answer::None(Why::Malformed),
        None => (env.wall, None),
    };
    let limit = env.limits.documents.rows;
    let mut window = BTreeMap::new();
    let mut more = false;
    let filter = Filter::new(core, query, now);
    for candidate in core.tasks.view_tasks().as_ref() {
        if candidate.project != query.project {
            continue;
        }
        let row = core.tasks.task(candidate.number).expect("live task view");
        if let Phase::Ended(_) = row.phase {
            continue;
        }
        let mut order = 0_u32;
        for wait in core.inspection_wait_records(row) {
            let position = WaitPosition::of(&wait, now, order);
            order = order.checked_add(1).expect("bounded own waits");
            if let Some(after) = after
                && position <= after
            {
                continue;
            }
            if !filter.includes(&wait, query) {
                continue;
            }
            if window.len() == usize_of(limit) {
                more = true;
                let Some((last, _)) = window.last_key_value() else { continue };
                if position >= *last {
                    continue;
                }
                window.pop_last();
            }
            assert!(window.insert(position, wait).is_none(), "unique task and wait order");
        }
    }
    if more && window.is_empty() {
        return Answer::None(Why::Busy);
    }
    page(core, &env.limits, party, query, now, window, more)
}

fn page(
    core: &Core,
    limits: &Limits,
    party: u64,
    query: WaitQuery,
    now: Wall,
    window: BTreeMap<WaitPosition, Wait>,
    more: bool,
) -> Answer {
    let after = if more {
        let (&after, _) = window.last_key_value().expect("a continued page advances");
        Some(Cursor::Waits { query, now, after })
    } else {
        None
    };
    let mut rows = List::with_capacity(limits.documents.rows);
    for (position, wait) in window {
        let row = core.tasks.task(position.task).expect("window keeps live tasks");
        let chip = match documents::small::chip_with_core(core, limits, party, row) {
            Ok(chip) => chip,
            Err(documents::Missing::TooLarge) => return Answer::None(Why::Busy),
            Err(documents::Missing::Unknown | documents::Missing::NotAllowed | documents::Missing::Ended) => {
                return Answer::None(Why::Unknown);
            }
        };
        rows.push(WaitRow { chip, wait, position }).expect("bounded window");
    }
    Answer::Waits(Box::new(WaitPage { now, rows, after }))
}

struct Filter {
    role: Option<u32>,
    through: Option<Wall>,
}

impl Filter {
    fn new(core: &Core, query: WaitQuery, now: Wall) -> Self {
        let role = match query.party {
            Some(person) => core.people.role(person, query.project),
            None => None,
        };
        let role = match role {
            Some(role) => Some(role.number()),
            None => None,
        };
        let through = match query.due {
            Some(due) => Some(Wall::from_nanos(now.as_nanos().saturating_add(due))),
            None => None,
        };
        Self { role, through }
    }

    fn includes(&self, wait: &Wait, query: WaitQuery) -> bool {
        if let Some(kind) = query.kind
            && wait.kind != kind
        {
            return false;
        }
        if let Some(through) = self.through {
            match wait.bound {
                Bound::At { at, .. } if at <= through => {}
                Bound::At { .. } | Bound::None(_) => return false,
            }
        }
        let Some(party) = query.party else { return true };
        if let Some(holder) = wait.hears
            && self.held_by(holder, party, query.project)
        {
            return true;
        }
        match wait.object {
            WaitObject::Entry { at, .. } => self.held_by(at, party, query.project),
            WaitObject::Question { .. }
            | WaitObject::Task(_)
            | WaitObject::Resource { .. }
            | WaitObject::Run(_)
            | WaitObject::News { .. }
            | WaitObject::Procedure { .. } => false,
        }
    }

    fn held_by(&self, holder: EscalationHolder, party: u64, project: u32) -> bool {
        match holder {
            EscalationHolder::Person(person) => person == party,
            EscalationHolder::Role { project: scope, role } => scope == project && self.role == Some(role),
            EscalationHolder::Task(_) => false,
        }
    }
}
