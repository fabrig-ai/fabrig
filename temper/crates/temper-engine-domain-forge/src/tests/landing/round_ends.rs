//! Round cleanup leaves the next prepare or turn due (domain/landing-waits.md, 3).
use super::*;
use skein_forge::api::{self, Answer, Error, Op, Read, Write};
use skein_lib::List;

const TASK: u64 = 61;
const GREEN: api::Commit = [11; 32];
const RED: api::Commit = [12; 32];
const TEST: &[u8] = b"test bin broken";

fn member_at(row: &LandingRow, branch: &[u8]) -> usize {
    for (i, m) in row.landing.members.iter().enumerate() {
        if m.branch.as_ref() == branch {
            return i;
        }
    }
    panic!("a member's branch")
}
fn merged_member(row: &LandingRow, pull: u64) -> Option<api::Commit> {
    for m in &row.landing.members {
        if m.pull == pull {
            return m.candidate;
        }
    }
    None
}
fn has_entry(row: &LandingRow, entry: u64) -> bool {
    for e in &row.entries {
        if e.entry == entry {
            return true;
        }
    }
    false
}

struct World {
    d: Domain,
    env: Env<Limits>,
    prefixes: [Option<api::Commit>; 4],
    pull: Option<api::Pull>,
    main: api::Commit,
    entry: u64,
    by_test: bool,
}
impl World {
    fn new(round: u16, goal: bool, by_test: bool) -> Self {
        let mut e = env();
        e.limits.landing_members = 4;
        let mut d = domain_with_limits(&e.limits);
        d.restore_begin(Wall::EPOCH);
        let mut w = Self { d, env: e, prefixes: [None; 4], pull: None, main: fixtures::BASE, entry: 90, by_test };
        w.send(Event::Restore { record: Stored::Repository(adopted()) });
        let count = if goal { 1 } else { 4 };
        let mut members = List::uncapped();
        for n in 0..count {
            let mut change = fixtures::opening_row();
            change.repository = REPO;
            change.task = 51_u64.checked_add(n).unwrap();
            change.change.task = change.task;
            change.change.state =
                temper_engine_domain_forge_change::State::Ready { head: fixtures::HEAD, ready_since: w.env.wall };
            change.change.landing = None;
            change.pull = Some(1_u64.checked_add(n).unwrap());
            change.branch = alloc::format!("temper/c{n}").into_bytes().into_boxed_slice();
            if goal {
                change.base = b"temper/g".as_slice().into();
            }
            w.send(Event::Restore { record: Stored::Change(Box::new(change)) });
            let mut member = merge_landing_row(false).landing.members[0].clone();
            member.change = 51_u64.checked_add(n).unwrap();
            member.pull = 1_u64.checked_add(n).unwrap();
            member.unit = u16::try_from(n).unwrap();
            member.branch = alloc::format!("temper/1/l61-{n}").into_bytes().into_boxed_slice();
            member.head = None;
            member.base = None;
            member.candidate = None;
            member.combined = None;
            member.gates = if goal {
                Box::new([])
            } else {
                Box::new([policy::GateProof {
                    proof: policy::Proof::Unkeyed,
                    posted: policy::Posted::Nothing,
                    faults: 0,
                }])
            };
            members.push(member).unwrap();
        }
        let mut row = landing_row(TASK, policy::Phase::Preparing);
        row.landing.preparation = round.checked_sub(1).unwrap();
        row.landing.members = members.into_boxed();
        row.asks = alloc::vec![None; count.try_into().unwrap()].into_boxed_slice();
        if goal {
            row.landing.branch = b"temper/g".as_slice().into();
        }
        let gates: Box<[policy::Gate]> = if goal {
            Box::new([])
        } else {
            Box::new([policy::Gate {
                name: b"gate".as_slice().into(),
                workflow: b"gate.yml".as_slice().into(),
                ignored: Box::new([]),
                blocking: true,
                base: false,
                report: None,
            }])
        };
        assert!(w.d.project_landing(
            5,
            temper_engine_domain_forge_change::Way::Turn,
            Box::new([RepositoryGates { repository: REPO, gates, tiers: Box::new([]) }]),
            policy::Project::DEFAULT,
            AfterLanding {
                verdict: temper_engine_domain_forge_tiers::Settings {
                    retries: 2,
                    suspects: temper_engine_domain_forge_tiers::Suspects::Bisect
                },
                bound: Duration::from_secs(7200),
            }
        ));
        w.send(Event::Landing { row });
        w.send(Event::Connection { forge: 1, state: Connection::Connected { id: 7, login: b"bot".as_slice().into() } });
        w.send(Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        w.send(Event::ReadAfresh);
        w.send(Event::SettleOutbox);
        w
    }
    fn send(&mut self, event: Event) -> Queue<Request> {
        let mut out = Queue::uncapped();
        step(&mut self.d, &self.env, event, &mut out);
        out
    }
    #[expect(clippy::wildcard_enum_match_arm, reason = "the fixture rejects unrelated forge reads")]
    fn read(&self, read: Read) -> Result<Answer, Error> {
        let row = self.d.landing(TASK).unwrap();
        match read {
            Read::Branch { branch } => {
                if branch == row.landing.branch {
                    return Ok(Answer::Commit(self.main));
                }
                for (i, member) in row.landing.members.iter().enumerate() {
                    if branch == member.branch {
                        return match self.prefixes[i] {
                            Some(prefix) => Ok(Answer::Commit(prefix)),
                            None => Err(Error::Missing),
                        };
                    }
                }
                Ok(Answer::Commit(fixtures::HEAD))
            }
            Read::Pull { number, .. } => {
                if let Some(pull) = &self.pull
                    && pull.number == number
                {
                    return Ok(Answer::Pull(pull.clone()));
                }
                let mut pull = fixtures::open_pull();
                pull.number = number;
                if self.main == GREEN {
                    pull.merged = merged_member(row, number);
                }
                Ok(Answer::Pull(pull))
            }
            Read::PullFor { head, .. } => {
                if let Some(pull) = &self.pull {
                    for (i, member) in row.landing.members.iter().enumerate() {
                        if head == member.branch && self.prefixes[i] == Some(pull.commit) {
                            return Ok(Answer::Pull(pull.clone()));
                        }
                    }
                }
                Err(Error::Missing)
            }
            Read::Reviews { .. } => Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
            Read::Commits { before, after, .. } => Ok(Answer::Commits {
                commits: if after == fixtures::HEAD && before == self.main {
                    Box::new([api::CommitMessage { id: fixtures::HEAD, parents: Box::new([]), message: Box::new([]) }])
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
            other => panic!("unexpected read {other:?}"),
        }
    }
    fn seed_receipts(&mut self) {
        let gate = self.d.landing.gates(5, REPO)[0].clone();
        for (prefix, passed) in [(GREEN, true), (RED, false)] {
            seed_receipt(self, &gate, prefix, passed);
        }
    }
    #[expect(clippy::wildcard_enum_match_arm, reason = "the fixture retains only the landing decision and client call")]
    fn decide(&mut self, expected: policy::Decision) {
        self.env.now = self.env.now.saturating_add(Duration::from_secs(10));
        self.env.wall =
            Wall::from_nanos(self.env.wall.as_nanos().checked_add(Duration::from_secs(10).as_nanos()).unwrap());
        self.entry = self.entry.checked_add(1).unwrap();
        let entry = self.entry;
        let mut out = self.send(Event::StepLanding {
            task: TASK,
            entry,
            heard: preparing_heard(Some(policy::Preparation::Absent)),
        });
        for _ in 0_u32..128 {
            while let Some(request) = out.pop() {
                if let Request::LandingDecision { decision, entry: writing, .. } = request {
                    assert_eq!(
                        decision,
                        expected,
                        "round {} phase {:?}",
                        self.d.landing(TASK).unwrap().landing.preparation,
                        self.d.landing(TASK).unwrap().landing.phase
                    );
                    if let Some(entry) = writing {
                        self.effect(entry);
                    }
                    return;
                }
            }
            resume(&mut self.d, &self.env, &mut out);
            let mut call = None;
            while let Some(request) = out.pop() {
                match request {
                    Request::Call { call: token, op, .. } => {
                        assert!(call.is_none());
                        call = Some((token, op));
                    }
                    Request::LandingDecision { decision, entry: writing, .. } => {
                        assert_eq!(decision, expected);
                        if let Some(entry) = writing {
                            self.effect(entry);
                        }
                        return;
                    }
                    other => {
                        assert!(
                            !matches!(other, Request::Refused { .. } | Request::ResourceRefused { .. }),
                            "{other:?}"
                        );
                    }
                }
            }
            let (call, op) = call.expect("landing read");
            let Op::Read(read) = op else { panic!("uncommitted write") };
            let result = self.read(read);
            out = self.send(Event::Client(skein_forge_client::Event::Answered {
                call,
                free: 0,
                rate: None,
                cost: 1,
                result,
            }));
            self.d.reclaim();
        }
        panic!("landing decides");
    }
    #[expect(clippy::wildcard_enum_match_arm, reason = "the fixture rejects writes outside round cleanup and landing")]
    fn effect(&mut self, entry: u64) {
        let write = self.d.entries.get(&entry).unwrap().entry.effect.write.clone();
        self.send(Event::Committed { entry });
        for _ in 0_u32..16 {
            let mut out = Queue::uncapped();
            resume(&mut self.d, &self.env, &mut out);
            let mut call = None;
            while let Some(request) = out.pop() {
                if let Request::Call { call: token, op, .. } = request {
                    assert!(call.is_none());
                    call = Some((token, op));
                }
            }
            let (call, op) = call.expect("effect call");
            let result = match op {
                Op::Read(read) => self.read(read),
                Op::Write(sent) => {
                    assert_eq!(sent, write);
                    let answer = match &sent {
                        Write::OpenPull { head, .. } => {
                            let row = self.d.landing(TASK).unwrap();
                            let i = member_at(row, head);
                            let mut pull = fixtures::open_pull();
                            pull.number = entry;
                            pull.head = head.clone();
                            pull.base = row.landing.branch.clone();
                            pull.commit = self.prefixes[i].unwrap();
                            self.pull = Some(pull);
                            Answer::Created(entry)
                        }
                        Write::Close { .. } => {
                            self.pull = None;
                            Answer::Done
                        }
                        Write::DeleteBranch { branch } => {
                            let row = self.d.landing(TASK).unwrap();
                            let i = member_at(row, branch);
                            self.prefixes[i] = None;
                            Answer::Done
                        }
                        Write::Merge { head, how: api::MergeHow::FastForwardOnly, .. } => {
                            self.main = *head;
                            if let Some(pull) = &mut self.pull {
                                pull.merged = Some(*head);
                            }
                            Answer::Merged(*head)
                        }
                        Write::Status { .. } => Answer::Done,
                        other => panic!("unexpected effect {other:?}"),
                    };
                    self.send(Event::Client(skein_forge_client::Event::Answered {
                        call,
                        free: 0,
                        rate: None,
                        cost: 1,
                        result: Ok(answer),
                    }));
                    self.d.reclaim();
                    if !has_entry(self.d.landing(TASK).unwrap(), entry) {
                        return;
                    }
                    continue;
                }
            };
            self.send(Event::Client(skein_forge_client::Event::Answered {
                call,
                free: 0,
                rate: None,
                cost: 1,
                result,
            }));
            self.d.reclaim();
            if !has_entry(self.d.landing(TASK).unwrap(), entry) {
                return;
            }
        }
        panic!("effect answered");
    }
    fn prepared(&mut self, goal: bool) {
        self.decide(policy::Decision::Prepare { member: 0, head: fixtures::HEAD, main: fixtures::BASE });
        self.prefixes = if goal { [Some(GREEN), None, None, None] } else { [Some(GREEN), Some(RED), None, None] };
        let report: &[u8] = if goal {
            br#"{"combined":[{"member":1,"by":"squash"}]}"#
        } else {
            br#"{"combined":[{"member":1,"by":"merge"},{"member":2,"by":"merge"}],"conflicts":[{"member":3,"with":"branch","files":["file"]},{"member":4,"with":"branch","files":["file"]}]}"#
        };
        self.d.landing.rows.get_mut(&TASK).unwrap().landing.round.failure = report.into();
    }
}

fn cut_to_kept_prefix(round: u16, by_test: bool) {
    use policy::{Decision as D, Leave, Posting};
    let mut w = World::new(round, false, by_test);
    w.prepared(false);
    w.seed_receipts();
    w.decide(D::Leave { member: 2, why: Leave::Conflict });
    w.decide(D::Leave { member: 3, why: Leave::Conflict });
    w.decide(D::Open);
    w.decide(D::None);
    w.decide(D::Key { member: 1, gate: 0, candidate: RED });
    w.decide(D::None);
    w.decide(D::Post { member: 1, gate: 0, posting: Posting::Ran { passed: false } });
    w.decide(D::Key { member: 0, gate: 0, candidate: GREEN });
    if !by_test {
        w.decide(D::Post { member: 0, gate: 0, posting: Posting::Received });
    }
    w.decide(D::None);
    w.decide(D::Leave { member: 1, why: Leave::Blamed { gate: 0, run: 8, red: RED, green: GREEN } });
    w.decide(D::Close);
    w.decide(D::Delete { member: 1 });
    w.decide(D::None);
    assert_eq!(
        w.d.landing(TASK).unwrap().landing.preparation,
        if by_test { round.checked_add(1).unwrap() } else { round }
    );
    w.decide(D::Open);
    w.decide(D::None);
    if by_test {
        w.decide(D::Post { member: 0, gate: 0, posting: Posting::Received });
    }
    w.decide(D::Proved);
    w.decide(D::Merge { member: 0, candidate: GREEN });
    w.decide(D::Landed { member: 0, prefix: GREEN });
    w.decide(D::Finish);
}

fn seed_receipt(w: &mut World, gate: &policy::Gate, prefix: api::Commit, passed: bool) {
    let tree = policy::Tree {
        object: prefix,
        entries: Box::new([policy::Entry {
            name: b"file".as_slice().into(),
            object: prefix,
            mode: 0o100_644,
            kind: policy::EntryKind::Blob,
        }]),
    };
    let policy::Keyed::Key(key) = policy::key(
        policy::Repository { forge: 1, repository: 2 },
        &gate.name,
        &policy::definition(gate),
        &[],
        prefix,
        &[tree],
        None,
        &policy::KeyLimits { entries: 32, path_bytes: 256 },
    ) else {
        panic!("key")
    };
    w.d.landing
        .receipts
        .insert(
            ReceiptKey { project: 5, repository: REPO, check: Check::Gate(gate.name.clone()), key },
            Receipt {
                passed,
                run: ReceiptRun { id: 7, number: 8, url: Box::new([]) },
                candidate: prefix,
                ended: w.env.wall,
                base: Some(fixtures::BASE),
                jobs: Box::new([]),
                logs: Box::new([]),
                report: ReceiptReport::Unset,
                failures: if passed || !w.by_test {
                    None
                } else {
                    Some(temper_engine_domain_forge_tiers::Failures::Tests(Box::new([TEST.into()])))
                },
            },
        )
        .unwrap();
}

#[test]
fn an_earlier_conflict_and_a_by_test_cut_open_and_land_two_kept_members() {
    use policy::{Decision as D, Leave, Posting};
    let mut w = World::new(1, false, true);
    // Three prepared members' pull request includes three complete commit lines.
    w.env.limits.client.op_bytes = 1024;
    let fail = w.d.landing.gates(5, REPO)[0].clone();
    let mut pass = fail.clone();
    pass.name = b"candidate".as_slice().into();
    let after = w.d.after_landing(5).unwrap();
    assert!(w.d.project_landing(
        5,
        temper_engine_domain_forge_change::Way::Turn,
        Box::new([RepositoryGates {
            repository: REPO,
            gates: Box::new([pass.clone(), fail.clone()]),
            tiers: Box::new([])
        }]),
        policy::Project::DEFAULT,
        after
    ));
    for member in &mut w.d.landing.rows.get_mut(&TASK).unwrap().landing.members {
        member.gates = Box::new(
            [policy::GateProof { proof: policy::Proof::Unkeyed, posted: policy::Posted::Nothing, faults: 0 }; 2],
        );
    }
    w.decide(D::Prepare { member: 0, head: fixtures::HEAD, main: fixtures::BASE });
    w.prefixes = [Some([10; 32]), Some(GREEN), None, Some(RED)];
    w.d.landing.rows.get_mut(&TASK).unwrap().landing.round.failure = br#"{"combined":[{"member":1,"by":"merge"},{"member":2,"by":"merge"},{"member":4,"by":"merge"}],"conflicts":[{"member":3,"with":"member","files":["file"]}]}"#.as_slice().into();
    for (gate, prefix, passed) in
        [(&pass, RED, true), (&fail, RED, false), (&pass, GREEN, true), (&fail, GREEN, true), (&fail, [10; 32], true)]
    {
        seed_receipt(&mut w, gate, prefix, passed);
    }
    w.decide(D::Leave { member: 2, why: Leave::EarlierConflict });
    w.decide(D::Open);
    w.decide(D::None);
    w.decide(D::Key { member: 3, gate: 0, candidate: RED });
    w.decide(D::Post { member: 3, gate: 0, posting: Posting::Received });
    w.decide(D::Key { member: 3, gate: 1, candidate: RED });
    w.decide(D::None);
    w.decide(D::Post { member: 3, gate: 1, posting: Posting::Ran { passed: false } });
    w.decide(D::Key { member: 0, gate: 1, candidate: [10; 32] });
    w.decide(D::None);
    w.decide(D::Key { member: 1, gate: 1, candidate: GREEN });
    w.decide(D::None);
    w.decide(D::Leave { member: 3, why: Leave::Blamed { gate: 1, run: 8, red: RED, green: GREEN } });
    w.decide(D::Close);
    w.decide(D::Delete { member: 3 });
    w.decide(D::None);
    assert_eq!(w.d.landing(TASK).unwrap().landing.preparation, 2);
    w.decide(D::Open);
    w.decide(D::None);
    w.decide(D::Key { member: 1, gate: 0, candidate: GREEN });
    w.decide(D::Post { member: 1, gate: 0, posting: Posting::Received });
    w.decide(D::Post { member: 1, gate: 1, posting: Posting::Received });
    w.decide(D::Proved);
    w.decide(D::Merge { member: 1, candidate: GREEN });
    w.decide(D::Landed { member: 0, prefix: [10; 32] });
    w.decide(D::Landed { member: 1, prefix: GREEN });
    w.decide(D::Finish);
}

#[test]
fn a_by_test_cut_after_two_conflicts_opens_and_lands_the_kept_prefix() {
    cut_to_kept_prefix(1, true);
}
#[test]
fn a_later_by_test_cut_opens_and_lands_the_member_below_the_blame() {
    cut_to_kept_prefix(2, true);
}
#[test]
fn a_whole_cut_after_two_conflicts_opens_the_green_turn() {
    cut_to_kept_prefix(1, false);
}
#[test]
fn a_stale_squash_turn_without_gates_cleans_up_and_prepares_again() {
    use policy::Decision as D;
    let mut w = World::new(1, true, false);
    w.prepared(true);
    w.decide(D::Open);
    w.decide(D::None);
    w.decide(D::Proved);
    w.main = [19; 32];
    w.decide(D::None);
    w.decide(D::Close);
    w.decide(D::Delete { member: 0 });
    w.decide(D::Stale);
    w.decide(D::Prepare { member: 0, head: fixtures::HEAD, main: [19; 32] });
    assert_eq!(w.d.landing(TASK).unwrap().landing.preparation, 2);
}
