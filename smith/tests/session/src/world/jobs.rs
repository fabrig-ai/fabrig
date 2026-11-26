//! Yielded commands and their pending replies on the world's clock.
//! Contracts: smith/docs/design/domain/session.md, 10; smith/docs/design/domain/tools.md, 5.

use super::{Decoded, Delivery, Done, Duration, Key, Returned, Stats, Time, Token, World, agent, io, tools};
use agent::llm::{NotRun, Problem};
use std::collections::BTreeSet;
use tools::{Call, Effect, Outcome};
use tools::{JobAct, JobEnd, JobNumber};

/// A command whose fixture output is written whole at its drawn end.
pub(super) struct Command {
    pub(super) started: io::Started,
    pub(super) ends: Time,
    pub(super) deadline: Time,
}

impl Command {
    fn terminal_at(&self) -> Time {
        self.ends.min(self.deadline)
    }
}

/// The machine retains a finished job, with its unread output, until a reader arrives.
enum Process {
    Running(Command),
    Ended { end: JobEnd, output: Vec<u8> },
}

/// A held command keeps its original spawn token and at most one pending act.
pub(super) struct Job {
    pub(super) session: Token,
    process: Process,
    reply: Option<Reply>,
    event: Option<Key>,
}

struct Reply {
    at: Time,
    head: u32,
    tail: u32,
    writes: Option<bool>,
}

impl Job {
    pub(super) fn batch_effect(&self) -> Option<bool> {
        self.reply.as_ref().and_then(|reply| reply.writes)
    }

    fn finish(&mut self, at: Time, checkout: &mut skein_fake_checkout::Checkout) {
        let Process::Running(command) = &self.process else { return };
        if at < command.terminal_at() {
            return;
        }
        let (end, output) = if command.ends <= command.deadline {
            let process = &command.started.process;
            io::finish(checkout, process);
            let exit = match process.program.exit {
                skein_fake_checkout::Exit::Code(code) => tools::Exit::Code { code },
                skein_fake_checkout::Exit::Signal(signal) => tools::Exit::Signal { signal },
            };
            (JobEnd::Ended(exit), process.program.output.clone())
        } else {
            (JobEnd::Ended(tools::Exit::TimedOut), Vec::new())
        };
        self.process = Process::Ended { end, output };
    }

    fn stop(&mut self, end: JobEnd) {
        if matches!(self.process, Process::Running(_)) {
            self.process = Process::Ended { end, output: Vec::new() };
        }
    }
}

impl World {
    pub(super) fn hold_job(&mut self, owner: Token, session: Token, command: Command) -> Done {
        let held = self.jobs.values().filter(|job| job.session == session).count() + 1;
        assert!(held <= self.settings.agent.tools.jobs as usize, "the tools admit only their job cells");
        let session_seen = self.sessions.get_mut(&self.openers[&session]).expect("the job belongs to a live session");
        session_seen.held_peak = session_seen.held_peak.max(held);
        if command.ends > command.deadline {
            self.stats.op_timeouts += 1;
        }
        self.keep_job(owner, Job { session, process: Process::Running(command), event: None, reply: None });
        Done::Held { head: Box::default(), tail: Box::default(), dropped: 0 }
    }

    pub(super) fn job_act(&mut self, owner: Token, act: JobAct) {
        let Some(mut job) = self.jobs.remove(&owner) else {
            assert_eq!(act, JobAct::Hasten, "only a hasten can cross the job's terminal");
            assert!(self.owners.contains(&owner), "a crossed hasten names a former spawn");
            return;
        };
        job.finish(self.now, &mut self.checkout);
        match act {
            JobAct::Hasten => {
                if !self.op_cancel_lost.contains(&owner)
                    && let Some(reply) = &mut job.reply
                {
                    reply.at = self.now;
                }
            }
            JobAct::Read { head, tail, wait } => {
                let writes = (wait != Duration::from_nanos(u64::MAX)).then_some(false);
                if let Some(writes) = writes {
                    self.batched(job.session, writes);
                }
                assert!(job.reply.is_none(), "only one reader at a time");
                job.reply = Some(Reply { at: self.now.saturating_add(wait), head, tail, writes });
                self.stats.job_acts += 1;
            }
            JobAct::Stop { head, tail } => {
                self.batched(job.session, true);
                assert!(job.reply.is_none(), "stop has sole use of the job");
                job.stop(JobEnd::Stopped);
                job.reply = Some(Reply { at: self.now, head, tail, writes: Some(true) });
                self.stats.job_acts += 1;
            }
        }
        self.keep_job(owner, job);
    }

    pub(super) fn cancel_job(&mut self, owner: Token) -> bool {
        let Some(mut job) = self.jobs.remove(&owner) else { return false };
        job.finish(self.now, &mut self.checkout);
        self.closing.insert(job.session);
        let at = match &job.process {
            Process::Running(command) if self.op_cancel_lost.contains(&owner) => command.terminal_at(),
            Process::Running(command) if self.rng.chance(self.settings.cancels_lost) => {
                assert!(self.op_cancel_lost.insert(owner), "one race per cancel");
                self.stats.op_cancels_lost += 1;
                command.terminal_at()
            }
            Process::Running(_) => {
                job.stop(JobEnd::Cancelled);
                self.stats.op_cancels += 1;
                self.now
            }
            Process::Ended { .. } => self.now,
        };
        // A cancel always awaits the group's terminal, not the old polling wait.
        // A cancel crossing the initial Held answer can arrive without a cleanup read.
        let reply = job.reply.get_or_insert(Reply {
            at,
            head: self.settings.agent.tools.shell_head,
            tail: self.settings.agent.tools.shell_tail,
            writes: None,
        });
        reply.at = at;
        self.keep_job(owner, job);
        true
    }

    pub(super) fn job_tick(&mut self, owner: Token, at: Time) {
        let mut job = self.jobs.remove(&owner).expect("a scheduled job is retained");
        job.event = None;
        job.finish(at, &mut self.checkout);
        let (end, output) = match &job.process {
            Process::Running(_) => (JobEnd::Running, &[][..]),
            Process::Ended { end, output } => (*end, output.as_slice()),
        };
        if let Some(reply) = job.reply.take_if(|reply| at >= reply.at || end != JobEnd::Running) {
            let Done::Exited { head, tail, dropped, .. } = io::exited(None, output, reply.head, reply.tail) else {
                unreachable!("output capture returns Exited")
            };
            self.agent_stage.push(agent::Event::Done { owner, done: Done::Job { head, tail, dropped, end } });
            if end != JobEnd::Running {
                if self.op_cancel_lost.remove(&owner) {
                    self.stats.done_after_cancel += 1;
                }
                return;
            }
        }
        self.keep_job(owner, job);
    }

    fn keep_job(&mut self, owner: Token, mut job: Job) {
        if let Some(event) = job.event.take() {
            self.wire.withdraw(event).expect("the old job delivery is pending");
        }
        let next = match (&job.process, &job.reply) {
            (Process::Running(command), Some(reply)) => Some(command.terminal_at().min(reply.at)),
            (Process::Running(command), None) => Some(command.terminal_at()),
            (Process::Ended { .. }, Some(_)) => Some(self.now),
            (Process::Ended { .. }, None) => None,
        };
        job.event = next.map(|at| self.schedule(at, Delivery::Job { owner, at }));
        assert!(self.jobs.insert(owner, job).is_none(), "one job per spawn owner");
    }
}

pub(super) fn observe_answer(highest: &mut u32, ended: &mut BTreeSet<u32>, outcome: &Outcome) {
    match outcome {
        Outcome::Running { job, .. } => {
            *highest += 1;
            assert_eq!(job.0, *highest, "new jobs are numbered in their session's answer order");
        }
        Outcome::Job { job, end, .. } => {
            assert!(job.0 > 0 && job.0 <= *highest, "a job answer names an announced job");
            if *end != JobEnd::Running {
                observe_end(ended, *job, *end);
            }
        }
        _ => {}
    }
}

pub(super) fn observe_end(ended: &mut BTreeSet<u32>, job: JobNumber, end: JobEnd) {
    assert_ne!(end, JobEnd::Running, "a job terminal does not say running");
    assert!(ended.insert(job.0), "each job's end is told once, by answer or fact");
}

/// Walk one turn's ordered call/result pairs, once, before recording the turn.
#[derive(Default)]
pub(super) struct RefusalShape {
    position: u16,
    entrance: Option<u16>,
    suppressed_write: bool,
    counted: bool,
}

impl RefusalShape {
    pub(super) fn observe(&mut self, call: &Decoded, result: &Returned, stats: &mut Stats, bound: u32) {
        self.position += 1;
        let effect = match call {
            Decoded::Owned { call } => {
                if let Returned::Invalid { problem } = result {
                    match (call, problem) {
                        (Call::Job { job, .. }, Problem::UnknownJob { job: named }) => {
                            assert_eq!(job.0, *named, "an unknown job refusal names the requested job");
                            stats.unknown_jobs += 1;
                        }
                        (Call::Shell { after: tools::Yield::Asked(_), .. }, Problem::JobsFull { jobs }) => {
                            assert_eq!(*jobs, bound, "a full jobs refusal reports the session's bound");
                            stats.full_jobs += 1;
                        }
                        _ => panic!("unexpected tools refusal for {call:?}: {problem:?}"),
                    }
                    self.entrance.get_or_insert(self.position);
                }
                tools::effect(call)
            }
            Decoded::Delegated { effect, .. } => *effect,
            _ => return,
        };
        match (effect, result) {
            (Effect::Write, Returned::NotRun { why: NotRun::AfterFailedWrite { call } }) => {
                self.suppressed_write |= self.entrance == Some(*call);
            }
            (Effect::Read, Returned::Owned { .. } | Returned::Text { .. })
                if self.suppressed_write && !self.counted =>
            {
                stats.refused_write_read += 1;
                self.counted = true;
            }
            _ => {}
        }
    }
}
