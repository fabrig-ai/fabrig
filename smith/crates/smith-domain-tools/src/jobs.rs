//! A kit's job cells, reserved by yielding flights or held by numbered jobs,
//! and the reads awaiting their unread output.
//! Original spawn tokens stay live until IO settles; cancelled terminals and
//! bounded unread output stay until read or close. Provider ids are never known.
//! Same-job reads reach the machine one at a time.
//! Contract: smith/docs/design/domain/tools.md, section 5; skein/docs/foundation/programming-model.md, sections 5.2 and 6.3.

use crate::command::{Kept, SelectedTimeout};
use crate::domain::Domain;
use crate::facts::{self, FactKind};
use crate::flight::{self, Flight};
use crate::kit::{self, Kit};
use crate::{CallInfo, Done, JobAct, JobEnd, JobNumber, JobProblem, Limits, Outcome, Request};
use alloc::boxed::Box;
use skein_lib::{Env, Id, Map, Queue, ReplyTo, Set, Time, Token};

#[derive(Debug)]
pub(crate) struct Jobs {
    reserved: Set<Id<Flight>>,
    held: Map<u32, Job>,
    reads: Map<u64, JobRead>,
    highest: u32,
    next_read: u64,
    cancelling: bool,
}

#[derive(Debug)]
pub(crate) struct Job {
    owner: Id<Flight>,
    reader: Option<u64>,
    call: Token,
    started: Time,
    state: State,
}

#[derive(Debug)]
enum State {
    Running,
    Cancelling,
    Ended { head: Box<[u8]>, tail: Box<[u8]>, dropped: u64, end: JobEnd, at: Time },
}

#[derive(Debug)]
pub(crate) struct JobRead {
    job: JobNumber,
    reply_to: ReplyTo,
    observation: Option<CallInfo>,
    timed: SelectedTimeout,
    kept: Kept,
    deadline: Time,
    stop: bool,
    hastened: bool,
    started: Option<Time>,
    interrupted: Option<skein_lib::Duration>,
}

impl Jobs {
    pub(crate) fn new(limits: &Limits, highest: u32) -> Self {
        Self {
            reserved: Set::with_capacity(limits.jobs),
            held: Map::with_capacity(limits.jobs),
            reads: Map::with_capacity(limits.calls),
            highest,
            next_read: 0,
            cancelling: false,
        }
    }

    pub(crate) fn full(&self) -> bool {
        let used = self.held.len().checked_add(self.reserved.len()).expect("the kit's job cells fit u32");
        used >= self.held.capacity()
    }

    pub(crate) fn reserve(&mut self, owner: Id<Flight>) {
        assert!(!self.full(), "the kit admitted this yielding command within its job bound: {owner:?}");
        let fresh = self.reserved.insert(owner).expect("the yielding command's reserved cell fits the kit's job table");
        assert!(fresh, "a yielding command reserves its cell once: {owner:?}");
    }

    pub(crate) fn release_reservation(&mut self, owner: Id<Flight>) {
        let reserved = self.reserved.remove(&owner);
        assert!(reserved, "the yielding command still owns its reserved job cell: {owner:?}");
    }

    /// Reservations still belong to ordinary flights, which the kit waits for separately.
    pub(crate) fn no_held_jobs(&self) -> bool {
        self.held.is_empty()
    }

    pub(crate) fn assert_drained(&self) {
        assert!(self.reserved.is_empty(), "a closed kit holds no yielding command's reservation");
        assert!(self.held.is_empty(), "a closed kit holds no numbered job");
        assert!(self.reads.is_empty(), "a closed kit holds no job read's place");
    }

    pub(crate) fn calls(&self) -> u32 {
        self.reads.len()
    }

    pub(crate) fn hold(&mut self, owner: Id<Flight>, observation: Option<CallInfo>, started: Time) -> JobNumber {
        self.release_reservation(owner);
        self.highest = self.highest.checked_add(1).expect("a conversation's job numbers fit u32");
        let number = JobNumber(self.highest);
        let call = match observation {
            Some(call) => call.owner,
            None => owner.token(),
        };
        let previous = self
            .held
            .insert(number.0, Job { owner, reader: None, call, started, state: State::Running })
            .expect("the yielding command's reservation fits the held job table");
        assert!(previous.is_none(), "a new job's number names no held job: {number:?}");
        number
    }

    fn release(&mut self, number: JobNumber, owner: Id<Flight>) -> Job {
        let job = self.held.remove(&number.0).expect("the job still holds the cell it releases");
        job.assert_owner(number, owner);
        assert!(job.reader.is_none(), "a job releases its cell only after its active read: {number:?}");
        job
    }

    fn admit_read(&mut self, mut read: JobRead, now: Time) -> u64 {
        self.next_read = self.next_read.checked_add(1).expect("bounded activation calls fit u64");
        let key = self.next_read;
        read.started = Some(now);
        let previous = self.reads.insert(key, read).expect("the kit admitted this job read within its call room");
        assert!(previous.is_none(), "a new read's key names no pending read: {key}");
        key
    }

    fn release_read(&mut self, number: JobNumber, key: u64) -> JobRead {
        let read = self.reads.remove(&key).expect("the job read still holds the place it releases");
        assert_eq!(read.job, number, "the read's place belongs to this job: read {key}");
        read
    }

    pub(crate) fn cut(&mut self, now: Time) {
        for index in 0..self.reads.len() {
            let key = *self.reads.iter().nth(usize::try_from(index).expect("bounded read")).expect("retained read").0;
            let read = self.reads.get_mut(&key).expect("retained read");
            if let Some(started) = read.started {
                read.interrupted = Some(now.saturating_since(started));
            }
        }
    }

    pub(crate) fn close(
        &mut self,
        session: Token,
        facts: &mut facts::Facts,
        limits: &Limits,
        out: &mut Queue<Request>,
    ) {
        let numbers = self.numbers();
        for number in &numbers {
            let job = self.held.get_mut(number).expect("retained job");
            match &job.state {
                State::Ended { .. } => {
                    let owner = job.owner;
                    let number = JobNumber(*number);
                    let job = self.release(number, owner);
                    tell(session, number, &job, facts);
                }
                State::Running | State::Cancelling => cancel_one(job, limits, out),
            }
        }
    }

    fn numbers(&self) -> skein_lib::List<u32> {
        let mut numbers = skein_lib::List::with_capacity(self.held.capacity());
        for (number, _) in &self.held {
            numbers.push(*number).expect("bounded jobs");
        }
        numbers
    }
}

impl Job {
    fn assert_owner(&self, number: JobNumber, owner: Id<Flight>) {
        assert_eq!(self.owner, owner, "the job's cell belongs to its original command: {number:?}");
    }
}

/// Enqueue a read; another read of this number must consume its answer first.
pub(crate) fn call(domain: &mut Domain, env: &Env<Limits>, id: Id<Kit>, read: JobRead, out: &mut Queue<Request>) {
    let kit = domain.kits.get_mut(id).expect("the session owns its kit");
    let job = read.job;
    let problem = if kit.jobs.held.contains_key(&job.0) { None } else { Some(JobProblem::Unknown { job }) };
    if let Some(problem) = problem {
        out.push(Request::Invalid { to: read.reply_to, problem });
        return;
    }
    if let Err(outcome) = kit::ready(kit, read.deadline, env) {
        domain.facts.push_call(
            facts::answered(
                kit::session(kit),
                crate::Tool::Job,
                &outcome,
                facts::Observation { omitted_bytes: 0, clamped: false },
            ),
            read.observation,
        );
        out.push(Request::Answer { to: read.reply_to, outcome });
        return;
    }
    domain.facts.push_call(FactKind::Started { session: kit::session(kit), tool: crate::Tool::Job }, read.observation);
    let key = kit.jobs.admit_read(read, env.now);
    let held = kit.jobs.held.get(&job.0).expect("an admitted read names a held job");
    match &held.state {
        State::Ended { .. } => {
            let owner = held.owner;
            let read = kit.jobs.release_read(job, key);
            let ended = kit.jobs.release(job, owner);
            match ended.state {
                State::Ended { head, tail, dropped, end, .. } => {
                    answer(kit::session(kit), read, &head, &tail, dropped, end, &mut domain.facts, out);
                }
                State::Running | State::Cancelling => unreachable!("the read releases a settled job's cell"),
            }
        }
        State::Running | State::Cancelling => advance(&mut kit.jobs, job, env.now, &env.limits, out),
    }
}

pub(crate) fn read(
    job: JobNumber,
    reply_to: ReplyTo,
    observation: Option<CallInfo>,
    timed: SelectedTimeout,
    kept: Kept,
    deadline: Time,
    stop: bool,
) -> JobRead {
    JobRead {
        job,
        reply_to,
        observation,
        timed,
        kept,
        deadline,
        stop,
        hastened: false,
        started: None,
        interrupted: None,
    }
}

/// The session names a pending reply; a queued read keeps its shortened wait.
pub(crate) fn end_wait(domain: &mut Domain, kit: Token, call: Token, out: &mut Queue<Request>) {
    let kit = domain.kits.get_mut(Id::from_token(kit)).expect("a session owns its kit");
    let mut found = None;
    for (key, read) in &kit.jobs.reads {
        if read.reply_to == ReplyTo::new(call) && !read.stop && !read.hastened {
            found = Some(*key);
            break;
        }
    }
    if let Some(key) = found {
        let read = kit.jobs.reads.get_mut(&key).expect("a pending read owns this reply");
        read.hastened = true;
        let job = kit.jobs.held.get(&read.job.0).expect("a read names a held job");
        if job.reader == Some(key) {
            out.push(Request::Job { owner: job.owner.token(), act: JobAct::Hasten });
        }
    }
}

fn advance(jobs: &mut Jobs, number: JobNumber, now: Time, limits: &Limits, out: &mut Queue<Request>) {
    let job = jobs.held.get_mut(&number.0).expect("a queued read names a held job");
    if job.reader.is_some() || is_cancelling(&job.state) {
        return;
    }
    for (key, read) in &jobs.reads {
        if read.job == number {
            job.reader = Some(*key);
            let wait = if read.hastened {
                skein_lib::Duration::ZERO
            } else {
                read.timed.timeout.min(read.deadline.saturating_since(now))
            };
            out.push(Request::Job {
                owner: job.owner.token(),
                act: if read.stop {
                    JobAct::Stop { head: limits.shell_head, tail: limits.shell_tail }
                } else {
                    JobAct::Read { head: limits.shell_head, tail: limits.shell_tail, wait }
                },
            });
            return;
        }
    }
}

/// A held owner's next terminal settles its active read, keeping the owner if
/// still running. Close also receives a terminal when no read was outstanding.
pub(crate) fn done(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Id<Flight>,
    number: JobNumber,
    done: Done,
    out: &mut Queue<Request>,
) {
    let kit_id = flight::kit(domain.flights.get(owner).expect("a held owner lives until its end"));
    let kit = domain.kits.get_mut(kit_id).expect("a kit owns every held command");
    let (head, tail, dropped, end) = match done {
        Done::Job { head, tail, dropped, end } => (head, tail, dropped, end),
        Done::Loaded { .. }
        | Done::Listed { .. }
        | Done::Removed
        | Done::NotEmpty
        | Done::Stored { .. }
        | Done::Conflict { .. }
        | Done::Exited { .. }
        | Done::CommandCancelled { .. }
        | Done::Held { .. }
        | Done::Found { .. }
        | Done::Missing
        | Done::NotFile
        | Done::Linked
        | Done::NotDirectory
        | Done::TooLarge { .. }
        | Done::Escapes
        | Done::Failed { .. }
        | Done::TimedOut
        | Done::Cancelled => unreachable!("a held command answers a job act"),
    };
    let session = kit::session(kit);
    let job = kit.jobs.held.get_mut(&number.0).expect("the original owner still holds its job cell");
    job.assert_owner(number, owner);
    let ended = match end {
        JobEnd::Running => false,
        JobEnd::Ended(_) | JobEnd::Stopped | JobEnd::Cancelled => true,
    };
    if ended && job.reader.is_none() {
        for (key, read) in &kit.jobs.reads {
            if read.job == number {
                job.reader = Some(*key);
                break;
            }
        }
    }
    if let Some(reader) = job.reader.take() {
        let call = job.call;
        let started = job.started;
        let read = kit.jobs.release_read(number, reader);
        if read.interrupted.is_some() {
            domain.facts.push(FactKind::JobEnded { session, job: number, owner: call, end, unread_bytes: 0, started });
        }
        answer(session, read, &head, &tail, dropped, end, &mut domain.facts, out);
    } else {
        assert!(ended, "only cancellation answers an unpolled job");
        job.state = State::Ended { head, tail, dropped, end, at: env.now };
        domain.flights.retire(owner);
        if kit::closing(kit) {
            let job = kit.jobs.release(number, owner);
            tell(session, number, &job, &mut domain.facts);
        }
        cancelled(kit, out);
        kit::settled(&mut domain.kits, &mut domain.facts, kit_id, out);
        return;
    }
    if ended {
        kit.jobs.release(number, owner);
        // Calls queued behind an end now name a job whose end was answered.
        let mut queued = skein_lib::List::with_capacity(env.limits.calls);
        for (key, read) in &kit.jobs.reads {
            if read.job == number {
                queued.push(*key).expect("at most calls reads await output");
            }
        }
        for key in &queued {
            let read = kit.jobs.release_read(number, *key);
            out.push(Request::Invalid { to: read.reply_to, problem: JobProblem::Unknown { job: number } });
        }
        domain.flights.retire(owner);
        cancelled(kit, out);
        kit::settled(&mut domain.kits, &mut domain.facts, kit_id, out);
    } else if !is_cancelling(&kit.jobs.held.get(&number.0).expect("running job retained").state) {
        advance(&mut kit.jobs, number, env.now, &env.limits, out);
    }
}

/// Cancel every live held owner; retained terminals still count until read.
pub(crate) fn cancel(domain: &mut Domain, kit: Token, limits: &Limits, out: &mut Queue<Request>) {
    let kit = domain.kits.get_mut(Id::from_token(kit)).expect("a session owns its kit");
    assert!(!kit.jobs.cancelling, "one cancellation request at a time");
    kit.jobs.cancelling = true;
    for number in &kit.jobs.numbers() {
        cancel_one(kit.jobs.held.get_mut(number).expect("retained job"), limits, out);
    }
    cancelled(kit, out);
}

fn cancel_one(job: &mut Job, limits: &Limits, out: &mut Queue<Request>) {
    match job.state {
        State::Running => {
            if job.reader.is_none() {
                // An unpolled held owner still owes a cancellation terminal.
                // Wait for that terminal without a polling timeout winning first.
                out.push(Request::Job {
                    owner: job.owner.token(),
                    act: JobAct::Read {
                        head: limits.shell_head,
                        tail: limits.shell_tail,
                        wait: skein_lib::Duration::from_nanos(u64::MAX),
                    },
                });
            }
            job.state = State::Cancelling;
            out.push(Request::CancelIo { owner: job.owner.token() });
        }
        State::Cancelling | State::Ended { .. } => {}
    }
}

fn is_cancelling(state: &State) -> bool {
    match state {
        State::Cancelling => true,
        State::Running | State::Ended { .. } => false,
    }
}

fn cancelled(kit: &mut Kit, out: &mut Queue<Request>) {
    if !kit.jobs.cancelling {
        return;
    }
    for (_, job) in &kit.jobs.held {
        if is_cancelling(&job.state) {
            return;
        }
    }
    kit.jobs.cancelling = false;
    out.push(Request::JobsCancelled { session: kit::session(kit) });
}

fn tell(session: Token, number: JobNumber, job: &Job, facts: &mut facts::Facts) {
    match &job.state {
        State::Ended { head, tail, dropped, end, at } => {
            let unread_bytes = skein_lib::len_u64(head.len())
                .checked_add(skein_lib::len_u64(tail.len()))
                .expect("bounded kept bytes")
                .checked_add(*dropped)
                .expect("machine's total output fits u64");
            facts.push_at(
                *at,
                FactKind::JobEnded {
                    session,
                    job: number,
                    owner: job.call,
                    end: *end,
                    unread_bytes,
                    started: job.started,
                },
            );
        }
        State::Running | State::Cancelling => unreachable!("only settled jobs have an unread terminal"),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "a job answer carries its four terminal fields beside its reply and observation sinks"
)]
fn answer(
    session: Token,
    read: JobRead,
    head: &[u8],
    tail: &[u8],
    dropped: u64,
    end: JobEnd,
    facts: &mut facts::Facts,
    out: &mut Queue<Request>,
) {
    let total = head.len().checked_add(tail.len()).expect("bounded output");
    let first = usize::try_from(read.kept.head).expect("head fits").min(if dropped > 0 { head.len() } else { total });
    let last = usize::try_from(read.kept.tail).expect("tail fits").min(if dropped > 0 {
        tail.len()
    } else {
        total.checked_sub(first).expect("head within kept output")
    });
    let mut leading = skein_lib::List::with_capacity(u32::try_from(first).expect("bounded head"));
    let mut trailing = skein_lib::List::with_capacity(u32::try_from(last).expect("bounded tail"));
    for (at, byte) in head.iter().chain(tail.iter()).enumerate() {
        if at < first {
            leading.push(*byte).expect("selected head room");
        }
        if at >= total.checked_sub(last).expect("tail within kept output") {
            trailing.push(*byte).expect("selected tail room");
        }
    }
    let dropped = dropped
        .checked_add(
            u64::try_from(
                total.checked_sub(first).expect("head within total").checked_sub(last).expect("tail within total"),
            )
            .expect("bounded capture"),
        )
        .expect("bounded total");
    let head = leading.into_boxed();
    let tail = trailing.into_boxed();
    let outcome = match read.interrupted {
        Some(ran) => Outcome::Interrupted { ran: Some(ran), head, last: tail, dropped },
        None => {
            Outcome::Job { job: read.job, head, tail, dropped, end, wait: read.timed.clamp, output: read.kept.clamp }
        }
    };
    facts.push_call(
        facts::answered(session, crate::Tool::Job, &outcome, facts::Observation { omitted_bytes: 0, clamped: false }),
        read.observation,
    );
    out.push(Request::Answer { to: read.reply_to, outcome });
}
