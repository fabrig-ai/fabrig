//! A party's batch loads ended places, then repeats admission on current state
//! (jig/docs/design/domain/tasks/README.md, section 4). Inputs and holdings have
//! already returned; connector-report retries enter this same judgement.
use crate::{
    Core, Event, Limits, Request,
    routing::{
        Work,
        flight::{Flight, Kind},
    },
};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Id, List, Queue, ReplyTo, Token};

/// Transient evidence belongs to the party, not to the batch's task creator.
#[derive(Hash, Debug, PartialEq, Eq)]
pub(crate) struct Admission {
    pub person: u64,
    pub project: u32,
    pub loaded: List<u64>,
    pub places: List<tasks::DependencyPlace>,
}
impl Admission {
    pub(crate) fn new(person: u64, project: u32, limits: &Limits) -> Self {
        // Each named dependency can also need its ended root's place.
        let capacity = limits
            .people
            .batch
            .checked_mul(limits.tasks.dependencies)
            .expect("bounded batch dependencies")
            .checked_mul(2)
            .expect("bounded batch roots");
        Self { person, project, loaded: List::with_capacity(capacity), places: List::with_capacity(capacity) }
    }

    #[expect(clippy::manual_find, reason = "step code scans bounded places without closures")]
    fn place(&self, number: u64) -> Option<&tasks::DependencyPlace> {
        for row in &self.places {
            if row.number == number {
                return Some(row);
            }
        }
        None
    }

    /// Recomputed after every load: a live dependency may have ended meanwhile.
    pub(crate) fn missing(&self, core: &Core, role: people::Role, batch: &[tasks::New]) -> Option<u64> {
        for member in batch {
            for &dependency in &member.dependencies {
                if in_batch(batch, dependency) || core.tasks.task(dependency).is_some() {
                    continue;
                }
                if !self.loaded.as_slice().contains(&dependency) {
                    return Some(dependency);
                }
                let Some(row) = self.place(dependency) else { continue };
                if row.project == self.project
                    && !core.reaches_project(role, self.project)
                    && row.requester != tasks::Party::Person(self.person)
                    && row.root != row.number
                    && core.tasks.task(row.root).is_none()
                    && !self.loaded.as_slice().contains(&row.root)
                {
                    return Some(row.root);
                }
            }
        }
        None
    }

    /// Stubs never prove reach: only live topology or a loaded place does.
    pub(crate) fn reached(&self, core: &Core, role: people::Role, depth: u32, dependency: u64) -> bool {
        let admission = tasks::CompoundAdmission::new(&core.tasks, 0);
        if core.tasks.task(dependency).is_some() {
            return core.reaches(
                crate::admission::compound::PartyTask {
                    person: self.person,
                    role: Some(role),
                    project: self.project,
                    task: dependency,
                },
                depth,
                &admission,
            );
        }
        let Some(row) = self.place(dependency) else { return false };
        if row.project != self.project {
            return false;
        }
        if core.reaches_project(role, self.project)
            || row.requester == tasks::Party::Person(self.person)
            || Core::person_tree(self.person, row.root, depth, &admission)
        {
            return true;
        }
        match self.place(row.root) {
            Some(root) => root.project == self.project && root.requester == tasks::Party::Person(self.person),
            None => false,
        }
    }
}

fn in_batch(batch: &[tasks::New], number: u64) -> bool {
    for member in batch {
        if member.number == number {
            return true;
        }
    }
    false
}

/// A whole uncommitted make, retained behind the held ended-row load.
#[derive(Hash, Debug)]
pub(crate) struct BatchRead {
    pub request: Token,
    pub creator: tasks::Party,
    pub maker: Option<u64>,
    pub batch: Box<[tasks::New]>,
}

pub(crate) fn admission(core: &Core, request: Token) -> Option<&Admission> {
    let Flight::Made { route, .. } = core.flights.get(Id::from_token(request))? else { return None };
    route.dependencies.as_ref()
}

/// One entry for party-created roots and party delegation: register Admission on
/// the make's continuation, independently of who will be the tasks' requester.
pub(crate) fn admit(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    mut event: tasks::Event,
) -> Result<tasks::Event, Queue<Request>> {
    let Some(request) = super::super::flight::request_token(&mut event) else { return Ok(event) };
    let tasks::Event::Make { batch, places, .. } = &mut event else { return Ok(event) };
    let Some(evidence) = admission(core, request) else { return Ok(event) };
    let Some(role) = core.people.role(evidence.person, evidence.project) else {
        terminal(core, work, request, people::Outcome::Refused(people::Refusal::Unknown));
        return Err(Queue::uncapped());
    };
    if !core.open_project(evidence.project) {
        terminal(core, work, request, people::Outcome::Refused(people::Refusal::Unknown));
        return Err(Queue::uncapped());
    }
    if let Some(next) = evidence.missing(core, role, batch) {
        let tasks::Event::Make { reply_to, creator, maker, batch, .. } = event else { unreachable!("make inspected") };
        let request = reply_to.into_token();
        let mut out = Queue::uncapped();
        crate::routing::run::results::wait_batch(
            core,
            work,
            &mut out,
            next,
            BatchRead { request, creator, maker, batch },
        );
        return Err(out);
    }
    for member in batch.as_ref() {
        for &dependency in &member.dependencies {
            if !in_batch(batch, dependency) && !evidence.reached(core, role, env.limits.tasks.depth, dependency) {
                terminal(core, work, request, people::Outcome::Refused(people::Refusal::Unknown));
                return Err(Queue::uncapped());
            }
        }
    }
    *places = evidence.places.as_slice().into();
    Ok(event)
}

pub(crate) fn loaded(
    core: &mut Core,
    work: &mut Queue<Work>,
    read: BatchRead,
    wanted: u64,
    row: Option<Box<tasks::TaskRecord>>,
    busy: bool,
) {
    let Some(evidence) = admission(core, read.request) else { return };
    if core.people.identity(evidence.person).is_none() {
        discard(core, read.request);
        return;
    }
    if busy {
        terminal(core, work, read.request, people::Outcome::Again(people::Again::Busy));
        return;
    }
    let Flight::Made { route, .. } = core.flights.get_mut(Id::from_token(read.request)).expect("pending make") else {
        unreachable!("make continuation checked")
    };
    let evidence = route.dependencies.as_mut().expect("party admission checked");
    evidence.loaded.push(wanted).expect("bounded batch places");
    if let Some(row) = row
        && row.number == wanted
    {
        let status = match row.phase {
            tasks::Phase::Ended(tasks::Ending::Done(_)) => Some(tasks::Status::Done),
            tasks::Phase::Ended(tasks::Ending::Failed { .. }) => Some(tasks::Status::Failed),
            tasks::Phase::Ended(tasks::Ending::Cancelled { .. }) => Some(tasks::Status::Cancelled),
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                None
            }
        };
        if let Some(status) = status {
            evidence
                .places
                .push(tasks::DependencyPlace {
                    number: row.number,
                    project: row.project,
                    requester: row.requester,
                    root: row.root,
                    depth: row.depth,
                    status,
                })
                .expect("bounded batch places");
        }
    }
    work.push(Work::Event(Event::Tasks(tasks::Event::Make {
        maker: read.maker,
        reply_to: ReplyTo::new(read.request),
        creator: read.creator,
        batch: read.batch,
        places: Box::new([]),
    })));
}

pub(crate) fn terminal(core: &mut Core, work: &mut Queue<Work>, request: Token, outcome: people::Outcome) {
    if let Some(Flight::Made { request, .. }) = core.land_kind(request, Kind::Task) {
        work.push(Work::Event(Event::People(people::Event::Decided { request, outcome })));
    }
}

pub(crate) fn discard(core: &mut Core, request: Token) {
    if let Some(Flight::Made { request, .. }) = core.land_kind(request, Kind::Task) {
        drop(core.land_kind(request, Kind::Party));
    }
}
