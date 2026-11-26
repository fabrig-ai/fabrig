//! A round's follow-up is retried when its procedure step was taken (landing-waits.md, 3).
use super::{reserved_turn_landing, reserved_turn_landing_with_limits, task_event};
use crate::{Domain, Limits, Work, route::forge_route as route};
use jig_domain_tasks as tasks;
use skein_forge::api::{self, Answer, Error, Op, Read, Write};
use skein_lib::{Duration, Env, List, Queue, ReplyTo, Token, Wall};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_landing as policy;

const BASE: api::Commit = [4; 32];
const HEAD: api::Commit = [3; 32];
const GREEN: api::Commit = [11; 32];
const RED: api::Commit = [12; 32];
const TEST: &[u8] = b"test bin broken";
const STALE_PULL: u64 = 99;

fn has_ready(out: &Queue<forge::Request>) -> bool {
    for r in out {
        if let forge::Request::LandingReady { .. } = r {
            return true;
        }
    }
    false
}
fn has_handoff(out: &Queue<forge::Request>) -> bool {
    for r in out {
        if let forge::Request::LandingReady { .. } | forge::Request::LandingDecision { .. } = r {
            return true;
        }
    }
    false
}
fn member_at(row: &forge::LandingRow, branch: &[u8]) -> usize {
    for (i, m) in row.landing.members.iter().enumerate() {
        if m.branch.as_ref() == branch {
            return i;
        }
    }
    panic!("a member's branch")
}
fn merged_member(row: &forge::LandingRow, pull: u64) -> Option<api::Commit> {
    for m in &row.landing.members {
        if m.pull == pull {
            return m.candidate;
        }
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MainMove {
    Closing,
    Cleaned,
    Opened,
}

fn take_call(out: &mut Queue<forge::Request>) -> Option<(Token, Op)> {
    let mut call = None;
    while let Some(request) = out.pop() {
        if let forge::Request::Call { call: token, op, .. } = request {
            assert!(call.is_none());
            call = Some((token, op));
        }
    }
    call
}

fn open_pull(number: u64, head: Box<[u8]>, base: Box<[u8]>, commit: api::Commit) -> api::Pull {
    api::Pull {
        number,
        state: api::State::Open,
        head,
        base,
        commit,
        base_commit: Some(BASE),
        merged: None,
        mergeable: true,
        ci: api::Ci::Passed,
        queued: None,
        dequeued: None,
        size: None,
        reviewers: Box::new([]),
    }
}

struct Round {
    domain: Domain,
    env: Env<Limits>,
    task: u64,
    entry: u64,
    prefixes: [Option<api::Commit>; 4],
    pull: Option<api::Pull>,
    main: api::Commit,
    refuse_deletion: bool,
}
impl Round {
    fn restoring(domain: Domain, env: Env<Limits>, row: &forge::LandingRow) -> Self {
        let task = row.landing.task;
        let mut round =
            Self { domain, env, task, entry: 100, prefixes: [None; 4], pull: None, main: BASE, refuse_deletion: false };
        while round.domain.work.pop().is_some() {}
        round.domain.forge.restore_begin(round.env.wall);
        round
    }

    // The snapshots are at the search boundary, with the prepare's conflicts and
    // the selected search's blame retained. The top tests drive their earlier steps.
    fn cut(number: u16, kept: usize, whole: bool) -> Self {
        Self::cut_members(number, kept, whole, false)
    }
    fn cut_members(number: u16, kept: usize, whole: bool, three: bool) -> Self {
        let mut limits = crate::tests::resume_fixture::limits();
        limits.forge.landing_members = 4;
        limits.authority.segments = 8;
        let (domain, env, mut row) = reserved_turn_landing_with_limits(&limits);
        let mut r = Self::restoring(domain, env, &row);
        let template = row.landing.members[0].clone();
        let mut members = List::uncapped();
        for i in 0_usize..if three { 3 } else { 4 } {
            let mut m = template.clone();
            m.change = 7_u64.checked_add(i.try_into().unwrap()).unwrap();
            m.unit = i.try_into().unwrap();
            m.pull = 1_u64.checked_add(i.try_into().unwrap()).unwrap();
            m.head = Some(HEAD);
            m.base = Some(BASE);
            m.round = Some(number);
            m.branch = skein_lib::bytes::concat(&[
                b"temper/1/l",
                skein_lib::Decimal::of(r.task).as_bytes(),
                b"-",
                skein_lib::Decimal::of(i.try_into().unwrap()).as_bytes(),
            ]);
            m.candidate = if i < kept {
                Some(if kept == 2 && i == 0 { [10; 32] } else { GREEN })
            } else if (three && i == 2) || (!three && ((kept == 1 && i == 1) || (kept == 2 && i == 3))) {
                Some(RED)
            } else {
                None
            };
            m.combined = if m.candidate.is_some() { Some(policy::Combine::Merge) } else { None };
            r.prefixes[i] = m.candidate;
            m.left = if i < kept {
                None
            } else if m.candidate.is_some() {
                Some(policy::Leave::Blamed { gate: 0, run: 8, red: RED, green: GREEN })
            } else if kept == 2 || three {
                Some(policy::Leave::EarlierConflict)
            } else {
                Some(policy::Leave::Conflict)
            };
            m.leave_told = m.left.is_some();
            m.gates = Box::new([policy::GateProof {
                proof: policy::Proof::Unkeyed,
                posted: policy::Posted::Nothing,
                faults: 0,
            }]);
            let mut change = r.ready_change(m.change, m.pull);
            change.branch =
                skein_lib::bytes::concat(&[b"temper/c", skein_lib::Decimal::of(i.try_into().unwrap()).as_bytes()]);
            r.send(forge::Event::Restore { record: forge::Stored::Change(Box::new(change)) });
            members.push(m).unwrap();
        }
        row.landing.members = members.into_boxed();
        row.landing.preparation = number;
        row.landing.phase = policy::Phase::Proving;
        row.landing.round.pull = Some(STALE_PULL);
        row.landing.round.ending = Some(policy::RoundEnd::Cut);
        row.landing.round.changed = Some(if three {
            2
        } else if kept == 1 {
            1
        } else {
            3
        });
        row.landing.round.search = Some(policy::Search {
            green: kept.try_into().unwrap(),
            red: if three {
                3
            } else if kept == 1 {
                2
            } else {
                4
            },
            tests: if whole {
                policy::SearchTests::Whole
            } else {
                policy::SearchTests::ByTest { gates: Box::new([Some(Box::new([TEST.into()]))]), probed: Box::new([]) }
            },
        });
        r.seed_gates();
        r.install(row);
        assert_eq!(r.domain.forge.restore_fit_failure(), None);
        r
    }
    fn seed_gates(&mut self) {
        let gate = policy::Gate {
            name: b"gate".as_slice().into(),
            workflow: b"gate.yml".as_slice().into(),
            ignored: Box::new([]),
            blocking: true,
            base: false,
            report: None,
        };
        let repo = self.domain.forge.change(7).unwrap().repository;
        let after = self.domain.forge.after_landing(1).unwrap();
        assert!(self.domain.forge.project_landing(
            1,
            temper_engine_domain_forge_change::Way::Turn,
            Box::new([forge::RepositoryGates {
                repository: repo,
                gates: Box::new([gate.clone()]),
                tiers: Box::new([])
            }]),
            policy::Project::DEFAULT,
            after
        ));
        for (candidate, passed) in [(GREEN, true), (RED, false)] {
            let policy::Keyed::Key(key) = policy::key(
                policy::Repository { forge: repo.forge, repository: repo.repository },
                &gate.name,
                &policy::definition(&gate),
                &[],
                candidate,
                &[policy::Tree {
                    object: candidate,
                    entries: Box::new([policy::Entry {
                        name: b"file".as_slice().into(),
                        object: candidate,
                        mode: 0o100_644,
                        kind: policy::EntryKind::Blob,
                    }]),
                }],
                None,
                &policy::KeyLimits { entries: 32, path_bytes: 256 },
            ) else {
                panic!("key")
            };
            self.send(forge::Event::Restore {
                record: forge::Stored::Receipt {
                    key: forge::ReceiptKey {
                        project: 1,
                        repository: repo,
                        check: forge::Check::Gate(gate.name.clone()),
                        key,
                    },
                    receipt: forge::Receipt {
                        passed,
                        candidate,
                        base: Some(BASE),
                        ended: self.env.wall,
                        run: forge::ReceiptRun { id: 7, number: 8, url: Box::new([]) },
                        jobs: Box::new([]),
                        logs: Box::new([]),
                        report: forge::ReceiptReport::Unset,
                        failures: if passed {
                            None
                        } else {
                            Some(temper_engine_domain_forge_tiers::Failures::Tests(Box::new([TEST.into()])))
                        },
                    },
                },
            });
        }
    }
    fn stale() -> Self {
        let (domain, env, mut row) = reserved_turn_landing();
        let mut r = Self::restoring(domain, env, &row);
        r.prefixes[0] = Some(GREEN);
        r.main = [19; 32];
        row.landing.phase = policy::Phase::Turn { at: r.env.wall };
        row.landing.preparation = 1;
        row.landing.branch = b"temper/1/g99".as_slice().into();
        row.landing.round.pull = Some(STALE_PULL);
        row.landing.round.upstream = Some(BASE);
        let m = &mut row.landing.members[0];
        m.pull = 1;
        m.head = Some(HEAD);
        m.base = Some(BASE);
        m.candidate = Some(GREEN);
        m.combined = Some(policy::Combine::Squash);
        let mut change = r.ready_change(7, 1);
        change.base = row.landing.branch.clone();
        change.branch = b"temper/c0".as_slice().into();
        r.send(forge::Event::Restore { record: forge::Stored::Change(Box::new(change)) });
        r.send(forge::Event::Restore {
            record: forge::Stored::Hold(forge::Hold {
                name: forge::Name {
                    forge: 1,
                    repository: 1,
                    what: forge::What::Branch(Box::new([
                        b"temper".as_slice().into(),
                        b"1".as_slice().into(),
                        b"g99".as_slice().into(),
                    ])),
                },
                task: 99,
                writer: None,
                design_writers: Box::new([]),
                drift: None,
                beside: None,
            }),
        });
        r.env.limits.authority.segments = 8;
        r.install(row);
        r
    }
    fn ready_change(&self, task: u64, pull: u64) -> forge::ChangeRow {
        let mut change = self.domain.forge.change(7).unwrap().clone();
        change.task = task;
        change.change.task = task;
        change.change.state =
            temper_engine_domain_forge_change::State::Ready { head: HEAD, ready_since: self.env.wall };
        change.change.gates = Box::new([]);
        change.pull = Some(pull);
        change
    }
    fn install(&mut self, row: forge::LandingRow) {
        let mut top = None;
        for member in &row.landing.members {
            if member.candidate.is_some() {
                top = Some(member);
            }
        }
        let top = top.unwrap();
        self.pull = Some(open_pull(STALE_PULL, top.branch.clone(), row.landing.branch.clone(), top.candidate.unwrap()));
        self.send(forge::Event::Restore { record: forge::Stored::Landing(row) });
        self.send(forge::Event::Connection {
            forge: 1,
            state: forge::Connection::Connected { id: 1, login: b"bot".as_slice().into() },
        });
        self.send(forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        self.send(forge::Event::CatchUp);
    }
    fn send(&mut self, event: forge::Event) -> Queue<forge::Request> {
        let mut out = Queue::uncapped();
        forge::step(&mut self.domain.forge, &self.env.with(self.env.limits.forge), event, &mut out);
        out
    }
    #[expect(clippy::wildcard_enum_match_arm, reason = "the fixture rejects unrelated forge reads")]
    fn answer(&self, read: Read) -> Result<Answer, Error> {
        let row = self.domain.forge.landing(self.task).unwrap();
        match read {
            Read::Branch { branch } => {
                if branch == row.landing.branch {
                    return Ok(Answer::Commit(self.main));
                }
                if branch.as_ref() == b"main" {
                    return Ok(Answer::Commit(BASE));
                }
                for (i, m) in row.landing.members.iter().enumerate() {
                    if branch == m.branch {
                        return match self.prefixes[i] {
                            Some(prefix) => Ok(Answer::Commit(prefix)),
                            None => Err(Error::Missing),
                        };
                    }
                }
                Ok(Answer::Commit(HEAD))
            }
            Read::Pull { number, .. } => {
                if let Some(pull) = &self.pull
                    && pull.number == number
                {
                    return Ok(Answer::Pull(pull.clone()));
                }
                let mut pull = open_pull(number, Box::new([]), row.landing.branch.clone(), HEAD);
                if self.main == GREEN {
                    pull.merged = merged_member(row, number);
                }
                Ok(Answer::Pull(pull))
            }
            Read::PullFor { head, .. } => {
                if let Some(pull) = &self.pull
                    && pull.head == head
                {
                    Ok(Answer::Pull(pull.clone()))
                } else {
                    Err(Error::Missing)
                }
            }
            Read::Reviews { .. } => Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
            Read::Commits { before, after, .. } => Ok(Answer::Commits {
                // The outside tip contains neither the source head nor the prepared prefix.
                commits: if before == self.main && (after == HEAD || (self.main == [19; 32] && after == GREEN)) {
                    Box::new([api::CommitMessage { id: after, parents: Box::new([]), message: Box::new([]) }])
                } else {
                    Box::new([])
                },
                more: false,
            }),
            Read::Tree { object, .. } => Ok(Answer::Tree {
                entries: Box::new([api::TreeEntry {
                    name: b"file".as_slice().into(),
                    object,
                    mode: 0o100_644,
                    kind: api::TreeEntryKind::Blob,
                }]),
                more: false,
            }),
            Read::Items { .. } => Ok(Answer::Items { items: Box::new([]), more: false, cut: false, now: self.env.now }),
            Read::Permission { user } => Ok(Answer::Permission(api::UserPermission {
                user,
                login: b"bot".as_slice().into(),
                permission: api::Permission::Write,
            })),
            Read::Settings => Ok(Answer::Settings(
                self.domain.forge.repository(api::Repository { forge: 1, repository: 1 }).unwrap().settings.clone(),
            )),
            Read::Protection { .. } => Ok(Answer::Protection(None)),
            Read::File { .. } => Err(Error::Missing),
            other => panic!("unexpected round read {other:?}"),
        }
    }
    fn gather(&mut self) -> Queue<forge::Request> {
        self.env.now = self.env.now.saturating_add(Duration::from_secs(10));
        self.env.wall =
            Wall::from_nanos(self.env.wall.as_nanos().checked_add(Duration::from_secs(10).as_nanos()).unwrap());
        self.entry = self.entry.checked_add(1).unwrap();
        let context = self.domain.engine.core().tasks.run_context(self.task).expect("landing context");
        let preparation = route::landing_route::preparation(&self.domain, &context);
        let out = self.send(forge::Event::StepLanding {
            task: self.task,
            entry: self.entry,
            heard: forge::LandingHeard {
                job: None,
                outcomes: Box::new([]),
                dependencies: Box::new([]),
                held: Box::new([]),
                ended: Box::new([]),
                failure: Box::new([]),
                preparation: Some(preparation),
                released: self.domain.engine.core().tasks.task(self.task).unwrap().due_reason == tasks::Why::Released,
                cancelled: false,
                kept: Box::new([]),
            },
        });
        self.read_until_handoff(out)
    }
    fn decide(&mut self) -> Queue<forge::Request> {
        let out = self.gather();
        if !has_ready(&out) {
            return out;
        }
        let mut out = Queue::uncapped();
        self.domain.forge.decide_landing(&self.env.with(self.env.limits.forge), self.task, true, &mut out);
        self.read_until_handoff(out)
    }
    fn read_until_handoff(&mut self, mut out: Queue<forge::Request>) -> Queue<forge::Request> {
        for _ in 0_u32..128 {
            if has_handoff(&out) {
                return out;
            }
            for r in &out {
                if let forge::Request::Refused { .. } | forge::Request::ResourceRefused { .. } = r {
                    panic!("{out:?}");
                }
            }
            forge::resume(&mut self.domain.forge, &self.env.with(self.env.limits.forge), &mut out);
            if has_handoff(&out) {
                return out;
            }
            let (call, Op::Read(read)) = take_call(&mut out).expect("round reads") else {
                panic!("only reads before handoff")
            };
            let result = self.answer(read);
            out = self.answered(call, result);
            self.domain.forge.reclaim();
        }
        panic!("round decision");
    }
    fn step_core(&mut self, event: jig_domain_core::Event) {
        drop(jig_domain_core::step(self.domain.engine.core_mut(), &crate::limits::environment_core(&self.env), event));
    }
    fn take_step(&mut self) {
        let (_, _, step) = self.domain.engine.core().tasks.procedure_due(self.task).expect("due landing");
        task_event(
            &mut self.domain,
            &self.env,
            tasks::Event::Procedure {
                reads_project: false,
                reply_to: ReplyTo::new(Token::new(999)),
                task: self.task,
                step,
                read: None,
                decision: tasks::ProcedureDecision::new(tasks::ProcedureEnd::Wait),
            },
        );
        assert!(self.domain.engine.core().tasks.procedure_due(self.task).is_none());
    }
    fn wake(&mut self) {
        task_event(&mut self.domain, &self.env, tasks::Event::WakeProcedure { task: self.task });
        assert!(self.domain.engine.core().tasks.procedure_due(self.task).is_some());
    }
    fn route(&mut self, mut out: Queue<forge::Request>) {
        let mut decision = crate::Decision::decide(&mut self.domain.journal, &self.env.limits.journal);
        route::outputs(&mut self.domain, &self.env, &mut decision, &mut out);
        crate::decision::accept_routed(
            &mut self.domain.journal,
            &mut self.domain.engine.core_mut().counters,
            &self.env.limits.journal,
            decision,
            true,
        )
        .unwrap();
    }
    #[expect(clippy::wildcard_enum_match_arm, reason = "only the procedure handoff belongs to this fixture")]
    fn forward(&mut self, expected: policy::Decision) {
        let mut out = self.decide();
        let mut entry = None;
        let mut handoff = Queue::uncapped();
        while let Some(request) = out.pop() {
            if let forge::Request::LandingDecision { decision, entry: writing, .. } = &request {
                assert_eq!(*decision, expected);
                entry = *writing;
                handoff.push(request);
            }
        }
        assert_eq!(handoff.len(), 1);
        self.route(handoff);
        let mut started = false;
        while let Some(work) = self.domain.work.pop() {
            match work {
                Work::Core(event @ jig_domain_core::Event::ProcedureStep { .. }) => {
                    self.step_core(event);
                }
                Work::Core(jig_domain_core::Event::EffectStart { owner, .. }) => {
                    started = true;
                    assert!(self.domain.forge_effects.remove(&owner).is_some());
                    self.take_step();
                }
                Work::Tasks(tasks::Event::WakeProcedure { task }) => {
                    if task == self.task {
                        self.wake();
                    }
                }
                other => panic!("unexpected handoff {other:?}"),
            }
        }
        if let Some(entry) = entry {
            assert!(started, "the root carries the write");
            self.effect(entry);
            self.wake();
        }
    }
    // One pass; a prepare it routes is claimed as job `job` and ends done
    // with no answer the round keeps (landing-preparing.md, 2, case 4).
    #[expect(clippy::wildcard_enum_match_arm, reason = "a decision without effects hands over only its step")]
    fn unanswered(&mut self, job: u64) -> policy::Decision {
        use forge::history::{Job, JobOutcome, JobStep};
        if self.domain.engine.core().tasks.procedure_due(self.task).is_none() {
            self.wake();
        }
        let mut out = self.decide();
        let mut decided = None;
        let mut handoff = Queue::uncapped();
        while let Some(request) = out.pop() {
            if let forge::Request::LandingDecision { decision, .. } = &request {
                decided = Some(*decision);
                handoff.push(request);
            }
        }
        self.route(handoff);
        while let Some(work) = self.domain.work.pop() {
            match work {
                Work::Core(jig_domain_core::Event::ProcedureStep { decision, .. }) if !decision.batch.is_empty() => {
                    self.take_step();
                    for outcome in [None, Some(JobOutcome::Done)] {
                        let step = JobStep { task: job, attempt: 1, job: Job::Prepare, outcome, head: None };
                        self.send(forge::Event::JobHistory { task: self.task, job: step });
                    }
                }
                Work::Core(event @ jig_domain_core::Event::ProcedureStep { .. }) => {
                    self.step_core(event);
                }
                Work::Tasks(tasks::Event::WakeProcedure { task }) if task == self.task => self.wake(),
                other => panic!("unexpected handoff {other:?}"),
            }
        }
        decided.expect("a routed decision")
    }
    #[expect(clippy::wildcard_enum_match_arm, reason = "the fixture rejects writes outside round cleanup and landing")]
    fn effect(&mut self, entry: u64) {
        self.send(forge::Event::Committed { entry });
        let mut written = false;
        for _ in 0_u32..16 {
            let mut out = Queue::uncapped();
            forge::resume(&mut self.domain.forge, &self.env.with(self.env.limits.forge), &mut out);
            if written && self.domain.forge.describe_landing_effect(self.task, entry).is_none() {
                return;
            }
            let (call, op) = take_call(&mut out).expect("round effect");
            let result = match op {
                Op::Read(read) => self.answer(read),
                Op::Write(Write::DeleteBranch { .. }) if self.refuse_deletion => {
                    self.refused_deletion(call);
                    return;
                }
                Op::Write(write) => {
                    assert!(!written);
                    written = true;
                    Ok(match write {
                        Write::Close { .. } => {
                            self.pull = None;
                            Answer::Done
                        }
                        Write::DeleteBranch { branch } => {
                            let row = self.domain.forge.landing(self.task).unwrap();
                            let i = member_at(row, &branch);
                            self.prefixes[i] = None;
                            Answer::Done
                        }
                        Write::OpenPull { head, base, .. } => {
                            let row = self.domain.forge.landing(self.task).unwrap();
                            let i = member_at(row, &head);
                            self.pull = Some(open_pull(entry, head, base, self.prefixes[i].unwrap()));
                            Answer::Created(entry)
                        }
                        Write::Status { .. } => Answer::Done,
                        Write::Merge { head, how: api::MergeHow::FastForwardOnly, .. } => {
                            self.main = head;
                            self.pull.as_mut().unwrap().merged = Some(head);
                            Answer::Merged(head)
                        }
                        other => panic!("unexpected round effect {other:?}"),
                    })
                }
            };
            self.answered(call, result);
            self.domain.forge.reclaim();
        }
        panic!("round effect settles");
    }
    fn answered(&mut self, call: Token, result: Result<Answer, Error>) -> Queue<forge::Request> {
        self.send(forge::Event::Client(skein_forge_client::Event::Answered {
            call,
            free: 0,
            rate: None,
            cost: 1,
            result,
        }))
    }
    fn refused_deletion(&mut self, call: Token) {
        let mut out = self.answered(call, Err(Error::Forbidden));
        let mut outcome = Queue::uncapped();
        while let Some(request) = out.pop() {
            if let forge::Request::Outcome { .. } = request {
                outcome.push(request);
            }
        }
        assert_eq!(outcome.len(), 1, "the refused deletion answers its outbox entry");
        self.route(outcome);
        let Some(Work::Core(jig_domain_core::Event::EffectConnector(_))) = self.domain.work.pop() else {
            panic!("the root settles the connector's outbox entry without a hold")
        };
        assert!(self.domain.work.is_empty(), "the outbox answer did not ask for a second hold");
        self.domain.forge.reclaim();
    }

    fn finish_cut(&mut self, kept: usize) {
        use policy::Decision as D;
        let member = u16::try_from(kept.checked_sub(1).unwrap()).unwrap();
        self.forward(D::None);
        self.forward(D::Key { member, gate: 0, candidate: GREEN });
        self.forward(D::Post { member, gate: 0, posting: policy::Posting::Received });
        self.forward(D::Proved);
        self.forward(D::Merge { member, candidate: GREEN });
        for i in 0..kept {
            self.forward(D::Landed {
                member: i.try_into().unwrap(),
                prefix: if kept == 2 && i == 0 { [10; 32] } else { GREEN },
            });
        }
        self.forward(D::Finish);
    }
    #[expect(clippy::wildcard_enum_match_arm, reason = "only a wake and read bookkeeping may follow a refused handoff")]
    fn lost(&mut self, expected: policy::Decision) {
        let prior = self.domain.forge.landing(self.task).unwrap().clone();
        let out = self.gather();
        assert!(has_ready(&out));
        // The offered step is consumed before the follow-up's final read is routed.
        self.take_step();
        self.route(out);
        let mut woken = false;
        while let Some(work) = self.domain.work.pop() {
            match work {
                Work::Forge(event) => {
                    self.send(event);
                }
                Work::Core(jig_domain_core::Event::Marked { .. }) | Work::StartDone(_) => {}
                Work::Tasks(tasks::Event::WakeProcedure { task }) if task == self.task => {
                    woken = true;
                    self.wake();
                }
                other => panic!("an uncarried decision only wakes: {other:?}"),
            }
        }
        assert!(
            self.domain.forge.describe_landing_effect(self.task, self.entry).is_none(),
            "no uncarried outbox entry"
        );
        assert_eq!(self.domain.forge.landing(self.task), Some(&prior), "no committed decision or pending result");
        assert!(woken, "a dropped decision must be woken");
        self.forward(expected);
    }
}

#[test]
fn a_by_test_cut_after_two_conflicts_retries_the_kept_members_open() {
    let mut r = Round::cut(1, 1, false);
    r.forward(policy::Decision::Close);
    r.forward(policy::Decision::Delete { member: 1 });
    r.forward(policy::Decision::None);
    r.lost(policy::Decision::Open);
    r.finish_cut(1);
}
#[test]
fn a_second_round_cut_retries_the_member_below_the_blames_open() {
    let mut r = Round::cut(2, 1, false);
    r.forward(policy::Decision::Close);
    r.forward(policy::Decision::Delete { member: 1 });
    r.forward(policy::Decision::None);
    r.lost(policy::Decision::Open);
    r.finish_cut(1);
}
#[test]
fn a_stale_squash_turn_without_gates_retries_its_close() {
    let mut r = Round::stale();
    // Freshness ends the proved round before its close is decided.
    r.forward(policy::Decision::None);
    r.lost(policy::Decision::Close);
    r.forward(policy::Decision::Delete { member: 0 });
    r.forward(policy::Decision::Stale);
    r.forward(policy::Decision::Prepare { member: 0, head: HEAD, main: [19; 32] });
    assert_eq!(r.domain.forge.landing(r.task).unwrap().landing.round.upstream, Some(BASE));
}
#[test]
fn a_by_test_cut_after_an_earlier_conflict_retries_with_two_kept_members() {
    let mut r = Round::cut(1, 2, false);
    r.forward(policy::Decision::Close);
    r.forward(policy::Decision::Delete { member: 3 });
    r.forward(policy::Decision::None);
    r.lost(policy::Decision::Open);
    r.finish_cut(2);
}
#[test]
fn a_key_read_finishing_after_its_step_was_taken_wakes_the_landing() {
    let mut r = Round::cut(1, 1, false);
    r.forward(policy::Decision::Close);
    r.forward(policy::Decision::Delete { member: 1 });
    r.forward(policy::Decision::None);
    r.forward(policy::Decision::Open);
    r.forward(policy::Decision::None);
    let ready = r.gather();
    assert!(has_ready(&ready));
    let mut out = Queue::uncapped();
    r.domain.forge.decide_landing(&r.env.with(r.env.limits.forge), r.task, true, &mut out);
    assert!(!has_handoff(&out), "the key is still reading its tree");
    r.take_step();
    let out = r.read_until_handoff(out);
    let mut keyed = false;
    for request in &out {
        if let forge::Request::LandingDecision {
            decision: policy::Decision::Key { member: 0, gate: 0, candidate: GREEN },
            entry: None,
            ..
        } = request
        {
            keyed = true;
        }
    }
    assert!(keyed);
    r.route(out);
    let mut woken = false;
    while let Some(work) = r.domain.work.pop() {
        if let Work::Tasks(tasks::Event::WakeProcedure { task }) = work {
            assert_eq!(task, r.task);
            woken = true;
            r.wake();
        }
    }
    assert!(woken, "a late observation cannot take the only wake");
    r.forward(policy::Decision::Post { member: 0, gate: 0, posting: policy::Posting::Received });
}
#[test]
fn a_stale_cleanup_retries_its_transition_and_its_new_prepare() {
    let mut r = Round::stale();
    r.forward(policy::Decision::None);
    r.forward(policy::Decision::Close);
    r.forward(policy::Decision::Delete { member: 0 });
    r.lost(policy::Decision::Stale);
    r.lost(policy::Decision::Prepare { member: 0, head: HEAD, main: [19; 32] });
}
fn cut_on_a_stale_base(point: MainMove) {
    use policy::Decision as D;
    // Three members: the middle left on an earlier conflict, the top was blamed.
    let mut r = Round::cut_members(1, 1, false, true);
    if point == MainMove::Closing {
        r.main = [19; 32];
        r.lost(D::Close);
    } else {
        r.forward(D::Close);
    }
    r.forward(D::Delete { member: 2 });
    r.forward(D::None);
    if point == MainMove::Opened {
        r.forward(D::Open);
    }
    r.main = [19; 32];
    r.lost(D::None);
    assert_eq!(r.domain.forge.landing(r.task).unwrap().landing.round.ending, Some(policy::RoundEnd::Stale));
    r.forward(D::Delete { member: 0 });
    r.lost(D::Stale);
    r.lost(D::Prepare { member: 0, head: HEAD, main: [19; 32] });
    assert_eq!(r.domain.forge.landing(r.task).unwrap().landing.members[0].base, Some([19; 32]));
}
#[test]
fn main_moving_before_a_cuts_close_is_answered_retries_the_new_prepare() {
    cut_on_a_stale_base(MainMove::Closing);
}
#[test]
fn main_moving_after_cut_cleanup_retries_the_new_prepare() {
    cut_on_a_stale_base(MainMove::Cleaned);
}
#[test]
fn main_moving_after_the_kept_round_opens_retries_the_new_prepare() {
    cut_on_a_stale_base(MainMove::Opened);
}
#[test]
fn a_whole_cut_retries_the_green_turns_close() {
    let mut r = Round::cut(1, 1, true);
    r.lost(policy::Decision::Close);
    r.forward(policy::Decision::Delete { member: 1 });
    r.forward(policy::Decision::None);
    r.forward(policy::Decision::Open);
    r.finish_cut(1);
}
#[test]
fn an_ended_prepare_with_no_kept_answer_is_asked_again_once_then_held() {
    use policy::Decision as D;
    let mut r = Round::stale();
    r.domain.config.prepare_kind = Some(jig_domain_link::JobBound {
        kind: jig_domain_link::JobKind(1),
        spec: 16_384,
        result: 4_096,
        deadline: Duration::from_secs(60),
    });
    for decision in [D::None, D::Close, D::Delete { member: 0 }, D::Stale] {
        r.forward(decision);
    }
    let prepare = D::Prepare { member: 0, head: HEAD, main: [19; 32] };
    let mut prepares = List::with_capacity(12);
    let mut waited = false;
    for job in 100_u64..112 {
        let wall = r.env.wall;
        let decision = r.unanswered(job);
        if let D::Wait { until: Some(until), on: policy::Awaited::Prefixes } = decision {
            assert!(until > wall, "{decision:?}");
            waited |= prepares.len() == 1;
            continue;
        }
        // The root reads the ended job as absent: no answer its round keeps names its member.
        prepares.push(decision).unwrap();
        if decision == D::Hold(policy::Hold::Prepare) {
            break;
        }
    }
    assert_eq!(prepares.as_slice(), [prepare, prepare, D::Hold(policy::Hold::Prepare)]);
    assert!(waited, "the replacement waits a poll");
}

#[test]
fn a_refused_deletion_holds_once_and_one_release_retries_it() {
    use policy::Decision as D;
    let mut r = Round::stale();
    let Ok(Answer::Pull(mut pull)) = r.answer(Read::Pull { number: STALE_PULL, ci: false }) else {
        panic!("round pull")
    };
    pull.state = api::State::Closed;
    r.pull = Some(pull);
    r.forward(D::None);
    r.refuse_deletion = true;
    r.forward(D::Delete { member: 0 });
    r.forward(D::Hold(policy::Hold::Cleanup));
    let task = r.domain.engine.core().tasks.task(r.task).unwrap();
    let tasks::Phase::Held { why: tasks::Hold::Procedure, source: Some(source), .. } = &task.phase else {
        panic!("the round's procedure holds the task: {:?}", task.phase);
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("cleanup hold account") };
    assert!(words.starts_with(b"cleanup: delete branch"));
    r.step_core(jig_domain_core::Event::Tasks(tasks::Event::Release {
        reply_to: ReplyTo::new(Token::new(998)),
        by: tasks::Party::Person(1),
        operation: tasks::Release { task: r.task, again: None, message: None },
    }));
    r.refuse_deletion = false;
    r.forward(D::None);
    r.forward(D::Delete { member: 0 });
    r.forward(D::Stale);
}
