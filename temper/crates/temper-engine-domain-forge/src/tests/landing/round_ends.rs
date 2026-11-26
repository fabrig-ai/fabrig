//! Round cleanup leaves the next prepare or turn due (domain/landing-waits.md, 3).
use super::*;
use jig_domain_core_vocabulary::waits::{AtBound, Reported, ReportedBound, Thing, WaitKind, WaitObject};
use skein_forge::api::{self, Answer, Error, Op, Read, Write};
use skein_lib::List;
use temper_client_protocol_wire::waits::{LANDING_ANCESTRY, LANDING_OPEN, LANDING_READ, THING_TASK};

const TASK: u64 = 61;
const CLEANUP_PULL: u64 = 92;
const OTHER_HOLDER: u64 = 77;
const CLOSE_REFUSED: &[u8] = b"cleanup: close item: forge refused permission; resource org/repo #92";
const DELETE_REFUSED: &[u8] =
    b"cleanup: delete branch: forge refused permission; resource org/repo branch temper/1/l61-0";
const CLEANUP_HOLD: policy::Decision = policy::Decision::Hold(policy::Hold::Cleanup);
const GREEN: api::Commit = [11; 32];
const RED: api::Commit = [12; 32];
const FRESH: api::Commit = [20; 32];
const TEST: &[u8] = b"test bin broken";
const SOURCE_ABOVE: &[u8] = b"temper/c1";
const MOVED: api::Commit = [21; 32];
const INCIDENT_PAGES: u32 = 3;
const LONG_PAGES: u32 = CLIENT.rows + 1;
const ROWS: u64 = 8; // CLIENT.rows
const STALL: Duration = Duration::from_secs(15);

#[derive(Clone, Copy)]
enum PrefixHistory {
    Unscripted,
    Refused,
    Complete(u32),
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MainMove {
    Closing,
    Cleaned,
    Opened,
}

struct World {
    d: Domain,
    env: Env<Limits>,
    prefixes: [Option<api::Commit>; 4],
    pull: Option<api::Pull>,
    closed_pull: Option<u64>,
    refuse_cleanup: bool,
    refusals: List<Option<Box<[u8]>>>,
    main: api::Commit,
    entry: u64,
    by_test: bool,
    main_move: Option<MainMove>,
    moved: bool,
    history: PrefixHistory,
    prefix_pages: List<u32>,
    read_log: List<LandingReadNote>,
    wait_report: Option<Box<[Reported]>>,
    held_news: List<u16>,
    refused_pull: Option<u64>,
    comparisons: List<(api::Commit, api::Commit)>,
    open: Open,
}

#[derive(Clone, Copy)]
enum Open {
    Answer,
    Withhold,
    Sent(Token),
}
impl World {
    fn new(round: u16, goal: bool, by_test: bool) -> Self {
        Self::with_members(round, goal, by_test, if goal { 1 } else { 4 })
    }
    fn with_members(round: u16, goal: bool, by_test: bool, count: u64) -> Self {
        let mut e = env();
        e.limits.landing_members = 4;
        let mut d = domain_with_limits(&e.limits);
        d.restore_begin(Wall::EPOCH);
        let mut w = Self {
            d,
            env: e,
            prefixes: [None; 4],
            pull: None,
            closed_pull: None,
            refuse_cleanup: false,
            refusals: List::uncapped(),
            main: fixtures::BASE,
            entry: 90,
            by_test,
            main_move: None,
            moved: false,
            history: PrefixHistory::Unscripted,
            prefix_pages: List::uncapped(),
            read_log: List::uncapped(),
            wait_report: None,
            held_news: List::uncapped(),
            refused_pull: None,
            comparisons: List::uncapped(),
            open: Open::Answer,
        };
        w.send(Event::Restore { record: Stored::Repository(adopted()) });
        let mut members = List::uncapped();
        for n in 0..count {
            let mut change = fixtures::opening_row();
            change.repository = REPO;
            change.task = 51_u64.checked_add(n).unwrap();
            change.change.task = change.task;
            change.change.state =
                temper_engine_domain_forge_change::State::Ready { head: fixtures::HEAD, ready_since: w.env.wall };
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
                wait: temper_engine_domain_forge_change::Wait::Cleared,
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
        w.send(Event::CatchUp);

        w
    }
    fn move_main(&mut self, at: MainMove) {
        if self.main_move == Some(at) {
            self.main = [19; 32];
            self.main_move = None;
        }
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
                Ok(Answer::Commit(if self.moved && branch.as_ref() == SOURCE_ABOVE { MOVED } else { fixtures::HEAD }))
            }
            Read::Pull { number, .. } => {
                if self.refused_pull == Some(number) {
                    return Err(Error::Forbidden);
                }
                if let Some(pull) = &self.pull
                    && pull.number == number
                    && self.closed_pull != Some(number)
                {
                    return Ok(Answer::Pull(pull.clone()));
                }
                let mut pull = fixtures::open_pull();
                pull.number = number;
                if self.closed_pull == Some(number) {
                    pull.state = api::State::Closed;
                }
                if self.main == GREEN || self.main == FRESH {
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
            Read::Commits { before, after: GREEN, page } if before == self.main => match self.history {
                PrefixHistory::Unscripted => Ok(Answer::Commits {
                    commits: if self.main == [19; 32] {
                        Box::new([api::CommitMessage { id: GREEN, parents: Box::new([]), message: Box::new([]) }])
                    } else {
                        Box::new([])
                    },
                    more: false,
                }),
                PrefixHistory::Refused if page == 2 => {
                    Err(Error::TooLarge(api::Excess::at_least(api::Bound::Rows, ROWS, ROWS + 1)))
                }
                PrefixHistory::Refused => Ok(history_page(page, INCIDENT_PAGES)),
                PrefixHistory::Complete(pages) => Ok(history_page(page, pages)),
            },
            Read::Commits { .. } if row.landing.round.branch.is_empty() => {
                Ok(Answer::Commits { commits: Box::new([]), more: true })
            }
            Read::Compare { before, after } => {
                Ok(Answer::Compare { before, after, changed: 0, ahead: 1, behind: 0, files: Box::new([]) })
            }
            Read::Commits { before, after, .. } => Ok(Answer::Commits {
                // The outside tip contains neither the source head nor the prepared prefix.
                commits: if before == self.main && (after == fixtures::HEAD || after == MOVED) {
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
            Read::Settings => Ok(Answer::Settings(adopted().settings)),
            Read::Protection { .. } => Ok(Answer::Protection(None)),
            Read::File { .. } => Err(Error::Missing),
            other => panic!("unexpected read {other:?}"),
        }
    }
    fn seed_receipts(&mut self) {
        let gate = self.d.landing.gates(5, REPO)[0].clone();
        for (prefix, passed) in [(GREEN, true), (RED, false)] {
            seed_receipt(self, &gate, prefix, passed);
        }
    }
    fn decide(&mut self, expected: policy::Decision) {
        assert_eq!(
            self.decide_after(Duration::from_secs(10)),
            expected,
            "round {} phase {:?}",
            self.d.landing(TASK).unwrap().landing.preparation,
            self.d.landing(TASK).unwrap().landing.phase
        );
    }
    fn clock(&mut self, by: Duration) {
        self.env.now = self.env.now.saturating_add(by);
        self.env.wall = Wall::from_nanos(self.env.wall.as_nanos().checked_add(by.as_nanos()).unwrap());
    }
    fn decide_after(&mut self, by: Duration) -> policy::Decision {
        self.clock(by);
        self.decide_heard(preparing_heard(Some(policy::Preparation::Absent)))
    }
    fn offer(&mut self, heard: LandingHeard) -> Queue<Request> {
        self.entry = self.entry.checked_add(1).unwrap();
        let entry = self.entry;
        self.send(Event::StepLanding { task: TASK, entry, heard })
    }
    fn decide_heard(&mut self, heard: LandingHeard) -> policy::Decision {
        let out = self.offer(heard);
        self.drive(out)
    }
    #[expect(clippy::wildcard_enum_match_arm, reason = "the fixture retains only the landing decision and client call")]
    fn drive(&mut self, mut out: Queue<Request>) -> policy::Decision {
        self.wait_report = None;
        for _ in 0_u32..128 {
            while let Some(request) = out.pop() {
                if let Request::LandingDecision { decision, entry: writing, .. } = request {
                    self.take_wait_report(out);
                    if let Some(entry) = writing {
                        self.effect(entry);
                    }
                    return decision;
                }
                if let Request::LandingRead(note) = request {
                    self.read_log.push(note).unwrap();
                } else if let Request::LandingNews { member, step: LandingNews::Held, .. } = request {
                    self.held_news.push(member).unwrap();
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
                        self.take_wait_report(out);
                        if let Some(entry) = writing {
                            self.effect(entry);
                        }
                        return decision;
                    }
                    Request::LandingRead(note) => self.read_log.push(note).unwrap(),
                    Request::LandingNews { member, step: LandingNews::Held, .. } => {
                        self.held_news.push(member).unwrap();
                    }
                    other => {
                        assert!(
                            !matches!(other, Request::Refused { .. } | Request::ResourceRefused { .. }),
                            "{other:?}"
                        );
                    }
                }
            }
            let Some((call, op)) = call else {
                // Admission from the read line gives the call its own pass.
                self.d.reclaim();
                assert!(self.d.is_ready(), "landing has a read to send");
                continue;
            };
            let Op::Read(read) = op else { panic!("uncommitted write") };
            if let Read::Commits { before, after: GREEN, page } = read
                && before == self.main
            {
                self.prefix_pages.push(page).unwrap();
            }
            if let Read::Compare { before, after } = read {
                self.comparisons.push((before, after)).unwrap();
            }
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
    fn take_wait_report(&mut self, mut out: Queue<Request>) {
        while let Some(request) = out.pop() {
            if let Request::Waits { task: TASK, waits } = request {
                assert!(self.wait_report.is_none(), "one replacement per step");
                self.wait_report = Some(waits.into_boxed());
            }
        }
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
                    if self.refuse_cleanup {
                        assert!(matches!(sent, Write::Close { .. } | Write::DeleteBranch { .. }));
                        self.refuse_write(call);
                        return;
                    }
                    let answer = match &sent {
                        Write::OpenPull { head, .. } => {
                            if let Open::Withhold = self.open {
                                self.open = Open::Sent(call);
                                return;
                            }
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
                        Write::Close { number } => {
                            self.move_main(MainMove::Closing);
                            self.pull = None;
                            self.closed_pull = Some(*number);
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
    fn refuse_write(&mut self, call: Token) {
        let mut out = self.send(Event::Client(skein_forge_client::Event::Answered {
            call,
            free: 0,
            rate: None,
            cost: 1,
            result: Err(Error::Forbidden),
        }));
        while let Some(request) = out.pop() {
            if let Request::Outcome {
                task: TASK,
                outcome: skein_forge_client::Outcome::Failed(Error::Forbidden),
                account,
                ..
            } = request
            {
                self.refusals.push(account).unwrap();
            }
        }
        self.d.reclaim();
    }

    fn release_cleanup(&mut self) {
        let mut heard = preparing_heard(None);
        heard.released = true;
        assert_eq!(self.decide_heard(heard), policy::Decision::None);
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

fn history_id(number: u32, tip: u32) -> api::Commit {
    if number == tip {
        GREEN
    } else if number.checked_add(1) == Some(tip) {
        fixtures::HEAD
    } else {
        [u8::try_from(number.checked_add(32).unwrap()).unwrap(); 32]
    }
}

fn history_page(page: u32, pages: u32) -> Answer {
    let first = page.checked_sub(1).unwrap().checked_mul(CLIENT.rows).unwrap();
    let last = first.checked_add(CLIENT.rows).unwrap().checked_sub(1).unwrap();
    let tip = pages.checked_mul(CLIENT.rows).unwrap().checked_sub(1).unwrap();
    let mut commits = List::with_capacity(CLIENT.rows);
    for number in first..=last {
        let parents: Box<[api::Commit]> = if number == tip {
            Box::new([fixtures::BASE, fixtures::HEAD])
        } else if let Some(prior) = number.checked_sub(1) {
            Box::new([history_id(prior, tip)])
        } else {
            Box::new([fixtures::BASE])
        };
        commits.push(api::CommitMessage { id: history_id(number, tip), parents, message: Box::new([]) }).unwrap();
    }
    Answer::Commits { commits: commits.into_boxed(), more: last < tip }
}

fn kept_lower_prefix(history: PrefixHistory) -> World {
    use policy::Decision as D;
    let mut w = World::with_members(1, false, false, 2);
    w.env.limits.change_policy.stall = STALL;
    w.decide(D::Prepare { member: 0, head: fixtures::HEAD, main: fixtures::BASE });
    w.prefixes = [Some(GREEN), Some(RED), None, None];
    let report = br#"{"combined":[{"member":1,"by":"merge"},{"member":2,"by":"merge"}]}"#;
    w.d.landing.rows.get_mut(&TASK).unwrap().landing.round.failure = report.as_slice().into();
    w.decide(D::Open);
    w.decide(D::None);
    w.moved = true;
    w.decide(D::Leave { member: 1, why: policy::Leave::HeadMoved { head: MOVED } });
    w.decide(D::Close);
    w.decide(D::Delete { member: 1 });
    w.decide(D::None);
    w.history = history;
    w
}

#[test]
fn a_kept_prefixs_refused_second_page_holds_the_round() {
    const READS: u64 = 22;
    let mut w = kept_lower_prefix(PrefixHistory::Refused);
    let poll = w.env.limits.client.poll;
    let since = w.env.wall;
    for _ in 0_u32..2 {
        assert!(matches!(w.decide_after(poll), policy::Decision::Wait { on: policy::Awaited::Read, .. }));
    }
    assert_eq!(w.d.landing_advance(TASK), Some((since, READS)));
    assert_eq!(w.decide_after(poll), policy::Decision::Hold(policy::Hold::Read));
    let ending = alloc::format!(": {REFUSAL}; {READS} reads since 00:02Z, none advanced the round");
    let words = skein_lib::bytes::concat(&[b"read: ", &incident_reading(), ending.as_bytes()]);
    let reason: Box<[u8]> = words.get(..skein_lib::usize_of(crate::history::REASON_BYTES)).unwrap_or(&words).into();
    assert_eq!(w.d.landing_hold_words(TASK), Some(reason.clone()));
    assert_eq!(w.held_news.as_slice(), &[0, 1]);
    assert_eq!(
        w.read_log.as_slice(),
        &[
            read_note(LandingReadNoteKind::Started { why: REFUSAL.as_bytes().into() }),
            read_note(LandingReadNoteKind::Held { elapsed: STALL, words: reason }),
        ]
    );
}

#[test]
fn unchanged_comparisons_stall_across_ancestry_and_read_waits() {
    use policy::{Awaited, Decision};
    const READS: u64 = 41; // Three complete gathers and one ending at pull #2.
    const FIRST_POLL: Duration = Duration::from_secs(4);
    let mut w = World::with_members(1, true, false, 2);
    w.env.limits.change_policy.stall = STALL;
    let since = Wall::from_nanos(127 * Duration::from_secs(60).as_nanos());
    w.env.wall = since;
    let mut row = w.d.landing(TASK).unwrap().clone();
    row.landing.round = policy::Round::new(Box::new([]));
    row.landing.branch = b"main".as_slice().into();
    for member in &mut row.landing.members {
        w.d.changes.get_mut(&member.change).unwrap().base = row.landing.branch.clone();
        member.head = Some(MOVED);
        member.base = Some(fixtures::BASE);
        member.candidate = Some(fixtures::HEAD);
    }
    landing::save(&mut w.d, &w.env, row, &mut Queue::uncapped());
    turn_rule(&mut w.d, b"main", &[1], false);
    for (refused, on, code, change) in [
        (None, Awaited::Ancestry { member: 0 }, LANDING_ANCESTRY, 51),
        (Some(2), Awaited::Read, LANDING_READ, 52),
        (None, Awaited::Ancestry { member: 0 }, LANDING_ANCESTRY, 51),
    ] {
        w.refused_pull = refused;
        assert!(matches!(w.decide_after(FIRST_POLL), Decision::Wait { on: got, .. } if got == on));
        assert_eq!(w.wait_report.as_deref(), Some([forge_wait(code, change, since, STALL)].as_slice()));
        let name = if refused.is_some() { LandingWaitFor::Read } else { LandingWaitFor::Ancestry };
        assert_panel_wait(&w.d, &w.env, TASK, name, Some(&w.wait_report.as_ref().unwrap()[0]));
    }
    assert_eq!(w.comparisons.as_slice(), &[(fixtures::BASE, fixtures::HEAD); 7]);
    assert_eq!(w.d.landing_advance(TASK), Some((since, READS)));
    w.d.wait_landing(&w.env, TASK, Some(wall_after(&w, w.env.limits.client.poll)));
    let before = Duration::from_secs(2);
    assert!(!has_wake(fixtures::advance(&mut w.d, &mut w.env, before)));
    assert!(has_wake(fixtures::advance(&mut w.d, &mut w.env, Duration::from_secs(1))));
    assert_eq!(w.decide_heard(preparing_heard(None)), Decision::Hold(policy::Hold::Read));
    let reason = alloc::format!(
        "read: the comparison of {} and {}, for change 52: answered; {READS} reads since 02:07Z, none advanced the round",
        "04".repeat(fixtures::BASE.len()),
        "03".repeat(fixtures::HEAD.len())
    );
    assert_eq!(w.d.landing_hold_words(TASK).unwrap().as_ref(), reason.as_bytes());
    assert_eq!(w.read_log.len(), 3, "one read run and one hold");
    assert!(matches!(w.read_log.as_slice()[2].kind, LandingReadNoteKind::Held { elapsed: STALL, .. }));
}

#[test]
fn arming_a_prepare_retry_does_not_advance_the_landing() {
    use policy::{Awaited, Decision};
    let mut w = World::with_members(1, true, false, 1);
    w.env.limits.change_policy.stall = STALL;
    w.env.limits.client.poll = STALL.saturating_add(STALL);
    w.decide(Decision::Prepare { member: 0, head: fixtures::HEAD, main: fixtures::BASE });
    let since = w.env.wall;
    let bound = w.env.now.saturating_add(STALL);
    assert!(w.d.landing(TASK).unwrap().landing.last_prepare.as_ref().unwrap().again.is_none());
    let decision = w.decide_after(Duration::from_secs(5));
    let Decision::Wait { on: Awaited::Prefixes, until: Some(again) } = decision else { panic!("{decision:?}") };
    assert_eq!(w.d.landing(TASK).unwrap().landing.last_prepare.as_ref().unwrap().again, Some(again));
    assert_eq!(w.d.landing_advance(TASK).unwrap().0, since);
    assert_eq!(w.d.inspection_landing_bound(TASK), Some(bound));
}

fn forge_wait(step: u32, change: u64, since: Wall, stall: Duration) -> Reported {
    Reported {
        kind: WaitKind::Step,
        object: WaitObject::Procedure {
            step,
            things: List::from_boxed(Box::new([Thing { what: THING_TASK, number: change, owner: None }])),
        },
        since,
        bound: ReportedBound::At { at: advanced_wall(since, stall), then: AtBound::Hold },
    }
}

#[test]
fn an_unanswered_open_holds_at_the_stall_without_withdrawing_its_entry() {
    use crate::history::{Job, JobOutcome, JobStep};
    use policy::{Awaited, Decision};
    let mut w = World::with_members(1, true, false, 1);
    let stall = Duration::from_secs(8);
    w.env.limits.change_policy.stall = stall;
    w.env.limits.client.lifetime = Duration::from_secs(20);
    w.env.limits.client.poll = Duration::from_secs(30);
    w.prepared(true);
    let job = JobStep { task: 71, attempt: 1, job: Job::Prepare, outcome: None, head: None };
    w.send(Event::JobHistory { task: TASK, job });
    w.open = Open::Withhold;
    let gap = Duration::from_secs(2);
    assert_eq!(w.decide_after(gap), Decision::Open);
    let entry = w.entry;
    let advanced = w.env.wall;
    let waiting = Decision::Wait { until: None, on: Awaited::Open };
    assert_eq!(w.decide_after(gap), waiting);
    assert_eq!(w.wait_report.as_deref(), Some([forge_wait(LANDING_OPEN, 51, advanced, stall)].as_slice()));
    assert_eq!(w.decide_after(gap), waiting);
    assert_eq!(w.wait_report, None, "a repeated wait does not publish again");
    assert_eq!(w.d.landing_advance(TASK).unwrap().0, advanced);
    // The prepare's recorded end is real progress, but the open still owes an answer.
    w.clock(gap);
    let out =
        collected(w.send(Event::JobHistory { task: TASK, job: JobStep { outcome: Some(JobOutcome::Done), ..job } }));
    assert_eq!(w.d.landing_advance(TASK), Some((w.env.wall, 0)));
    assert_eq!(wait_report(&out, TASK), Some([forge_wait(LANDING_OPEN, 51, w.env.wall, stall)].as_slice()));
    w.d.wait_landing(&w.env, TASK, None);
    assert!(!has_wake(fixtures::advance(&mut w.d, &mut w.env, gap)));
    assert!(has_wake(fixtures::advance(
        &mut w.d,
        &mut w.env,
        Duration::from_nanos(stall.as_nanos().checked_sub(gap.as_nanos()).unwrap())
    )));
    assert_eq!(w.decide_heard(preparing_heard(None)), Decision::Hold(policy::Hold::Read));
    let hold = w.d.landing_hold_words(TASK).unwrap();
    assert_eq!(hold.as_ref(), b"read: the round's pull request to open: no answer");
    assert!(w.d.entries.contains_key(&entry), "the open remains in the outbox");
    let Open::Sent(call) = w.open else { panic!("the open was sent") };
    w.clock(Duration::from_secs(1));
    fixtures::answer_at(&mut w.d, &w.env, call, Ok(Answer::Created(entry)));
    assert!(!w.d.entries.contains_key(&entry), "the open settled on its answer while held");
    assert_eq!(w.d.landing_hold_words(TASK), Some(hold));
    assert_eq!(w.d.landing_advance(TASK), Some((w.env.wall, 0)), "the recorded answer is an advance");
}

const REFUSAL: &str = "past forge.client.rows (8): at least 9 rows";

fn incident_reading() -> Box<[u8]> {
    alloc::format!(
        "commits after {} up to {}, page 2, for change {}",
        "04".repeat(fixtures::BASE.len()),
        "0b".repeat(GREEN.len()),
        fixtures::CHANGE
    )
    .into_bytes()
    .into_boxed_slice()
}

fn read_note(kind: LandingReadNoteKind) -> LandingReadNote {
    LandingReadNote { task: TASK, round: 2, reading: incident_reading(), kind }
}

fn read_decision(at: Wall) -> policy::Decision {
    policy::Decision::Wait { until: Some(at), on: policy::Awaited::Read }
}

fn wall_after(w: &World, by: Duration) -> Wall {
    Wall::from_nanos(w.env.wall.as_nanos().checked_add(by.as_nanos()).unwrap())
}

#[test]
fn two_short_read_runs_log_each_start_and_end_once() {
    let mut w = kept_lower_prefix(PrefixHistory::Refused);
    let poll = Duration::from_secs(4);
    w.env.limits.client.poll = poll;
    for next in [policy::Decision::Open, policy::Decision::None] {
        w.history = PrefixHistory::Refused;
        w.decide_after(poll);
        w.decide_after(poll);
        w.history = PrefixHistory::Complete(INCIDENT_PAGES);
        assert_eq!(w.decide_after(poll), next);
    }
    let start = read_note(LandingReadNoteKind::Started { why: REFUSAL.as_bytes().into() });
    let end = read_note(LandingReadNoteKind::Read { elapsed: poll.saturating_add(poll) });
    assert_eq!(w.read_log.as_slice(), &[start.clone(), end.clone(), start, end]);
}

#[test]
fn a_read_run_wakes_at_the_stall_before_the_poll() {
    let mut w = kept_lower_prefix(PrefixHistory::Refused);
    let slow_poll = STALL.saturating_add(STALL);
    w.env.limits.client.poll = slow_poll;
    let before = Duration::from_secs(5);
    let decision = w.decide_after(Duration::from_secs(10));
    let deadline = wall_after(&w, before);
    assert_eq!(decision, read_decision(deadline));
    w.d.wait_landing(&w.env, TASK, Some(deadline));
    let out = fixtures::advance(&mut w.d, &mut w.env, before);
    assert!(has_wake(out));
}

fn has_wake(out: Queue<Request>) -> bool {
    let mut out = out;
    while let Some(request) = out.pop() {
        if request == (Request::WakeProcedure { task: TASK }) {
            return true;
        }
    }
    false
}

#[test]
fn reconnecting_restarts_a_landings_read_stall_clock() {
    let mut w = kept_lower_prefix(PrefixHistory::Refused);
    let poll = w.env.limits.client.poll;
    w.decide_after(poll);
    w.clock(poll);
    w.send(Event::Connection { forge: REPO.forge, state: Connection::NotConnected });
    assert_eq!(w.d.inspection_landing_bound(TASK), None);
    w.clock(STALL);
    assert!(matches!(w.decide_after(poll), policy::Decision::Wait { on: policy::Awaited::Read, .. }));
    w.send(Event::Connection {
        forge: REPO.forge,
        state: Connection::Connected { id: 7, login: b"bot".as_slice().into() },
    });
    assert_eq!(w.d.landing_advance(TASK), Some((w.env.wall, 0)));
    assert!(matches!(w.decide_after(poll), policy::Decision::Wait { on: policy::Awaited::Read, .. }));
    assert_eq!(w.decide_after(STALL), policy::Decision::Hold(policy::Hold::Read));
}

#[test]
fn a_read_queued_for_room_holds_without_an_answer() {
    let mut w = kept_lower_prefix(PrefixHistory::Refused);
    w.offer(preparing_heard(None));
    let owner = *w.d.landing.pending.iter().next().unwrap().0;
    // Busy is a terminal from client admission, not the answer to the read.
    landing::answered(&mut w.d, &w.env, owner, Err(Error::Busy), &mut Queue::uncapped());
    let out = fixtures::advance(&mut w.d, &mut w.env, STALL);
    let out = accepted(&mut w.d, &w.env, out);
    let decision = w.drive(out);
    assert_eq!(decision, policy::Decision::Hold(policy::Hold::Read));
    assert_eq!(
        w.d.landing_hold_words(TASK).as_deref(),
        Some(
            b"read: branch temper/c0, for change 51: no answer; 1 reads since 00:02Z, none advanced the round"
                .as_slice()
        )
    );
    assert!(w.d.landing.pending.is_empty());
}

fn accepted(d: &mut Domain, env: &Env<Limits>, mut out: Queue<Request>) -> Queue<Request> {
    accept_landings(d, env, &mut out);
    out
}

#[test]
fn an_outage_retry_restarts_the_read_stall_clock() {
    let mut w = kept_lower_prefix(PrefixHistory::Refused);
    w.env.limits.client.read_attempts = 1;
    let poll = w.env.limits.client.poll;
    w.decide_after(poll);
    w.clock(poll);
    w.offer(preparing_heard(None));
    let (call, _) = fixtures::next_at(&mut w.d, &w.env);
    fixtures::answer_at(&mut w.d, &w.env, call, Err(Error::Unavailable));
    assert_eq!(w.d.inspection_landing_bound(TASK), None);
    let out = fixtures::advance(&mut w.d, &mut w.env, STALL);
    let out = accepted(&mut w.d, &w.env, out);
    assert_eq!(w.drive(out), read_decision(wall_after(&w, poll)));
    assert_eq!(w.d.landing_advance(TASK).unwrap().0, w.env.wall);
    assert_eq!(w.decide_after(poll), read_decision(wall_after(&w, poll)));
    assert_eq!(w.decide_after(STALL), policy::Decision::Hold(policy::Hold::Read));
}

#[test]
fn a_read_hold_releases_into_a_new_round_with_receipts() {
    let mut w = kept_lower_prefix(PrefixHistory::Refused);
    w.seed_receipts();
    let poll = w.env.limits.client.poll;
    w.decide_after(poll);
    assert_eq!(w.decide_after(STALL), policy::Decision::Hold(policy::Hold::Read));
    w.read_log = List::uncapped();
    let mut heard = preparing_heard(None);
    heard.released = true;
    assert_eq!(w.decide_heard(heard), policy::Decision::None);
    assert_eq!(w.d.landing_advance(TASK), Some((w.env.wall, 0)));
    w.history = PrefixHistory::Complete(INCIDENT_PAGES);
    w.decide(policy::Decision::Delete { member: 0 });
    w.decide(policy::Decision::Stale);
    w.decide(policy::Decision::Prepare { member: 0, head: fixtures::HEAD, main: fixtures::BASE });
    assert_eq!(w.d.landing(TASK).unwrap().landing.preparation, 3);
    assert_eq!(w.d.landing(TASK).unwrap().read_hold, None);
    assert_eq!(w.d.landing.receipts.len(), 2);
    assert!(w.read_log.is_empty());
}

#[test]
fn a_kept_prefix_opens_after_its_last_history_page() {
    skein_rows::check(
        [("incident range", INCIDENT_PAGES), ("past the row limit in pages", LONG_PAGES)],
        kept_history_reaches_the_tip,
    );
}

fn kept_history_reaches_the_tip(pages: u32) {
    use policy::Decision as D;
    let mut w = kept_lower_prefix(PrefixHistory::Complete(pages));
    w.decide(D::Open);
    let mut expected = List::with_capacity(pages);
    for page in 1..=pages {
        expected.push(page).unwrap();
    }
    assert_eq!(w.prefix_pages, expected);
    let row = w.d.landing(TASK).unwrap();
    assert_eq!(row.landing.preparation, 2);
    assert_eq!(row.landing.members[0].combined, Some(policy::Combine::Merge));
    assert_eq!(row.landing.members[0].candidate, Some(GREEN));
    w.decide(D::None);
    w.decide(D::Key { member: 0, gate: 0, candidate: GREEN });
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
fn cut_on_a_stale_base(point: MainMove) {
    use policy::{Decision as D, Leave, Posting};
    let mut w = World::with_members(1, false, true, 3);
    w.decide(D::Prepare { member: 0, head: fixtures::HEAD, main: fixtures::BASE });
    // The middle member left on an earlier conflict, before the search blamed the top.
    w.prefixes = [Some(GREEN), None, Some(RED), None];
    w.d.landing.rows.get_mut(&TASK).unwrap().landing.round.failure = br#"{"combined":[{"member":1,"by":"merge"},{"member":3,"by":"merge"}],"conflicts":[{"member":2,"with":"member","files":["file"]}]}"#.as_slice().into();
    w.seed_receipts();
    w.decide(D::Leave { member: 1, why: Leave::EarlierConflict });
    w.decide(D::Open);
    w.decide(D::None);
    w.decide(D::Key { member: 2, gate: 0, candidate: RED });
    w.decide(D::None);
    w.decide(D::Post { member: 2, gate: 0, posting: Posting::Ran { passed: false } });
    w.decide(D::Key { member: 0, gate: 0, candidate: GREEN });
    w.decide(D::None);
    w.decide(D::Leave { member: 2, why: Leave::Blamed { gate: 0, run: 8, red: RED, green: GREEN } });
    w.main_move = Some(point);
    w.decide(D::Close);
    w.decide(D::Delete { member: 2 });
    w.decide(D::None);
    w.move_main(MainMove::Cleaned);
    if point == MainMove::Opened {
        w.decide(D::Open);
        w.move_main(MainMove::Opened);
    }
    w.decide(D::None);
    assert_eq!(w.d.landing(TASK).unwrap().landing.round.ending, Some(policy::RoundEnd::Stale));
    w.decide(D::Delete { member: 0 });
    w.decide(D::Stale);
    w.decide(D::Prepare { member: 0, head: fixtures::HEAD, main: [19; 32] });
    assert_eq!(w.d.landing(TASK).unwrap().landing.members[0].base, Some([19; 32]));
    w.prefixes[0] = Some(FRESH);
    let gate = w.d.landing.gates(5, REPO)[0].clone();
    seed_receipt(&mut w, &gate, FRESH, true);
    w.d.landing.rows.get_mut(&TASK).unwrap().landing.round.failure =
        br#"{"combined":[{"member":1,"by":"merge"}]}"#.as_slice().into();
    w.decide(D::Open);
    w.decide(D::None);
    w.decide(D::Key { member: 0, gate: 0, candidate: FRESH });
    w.decide(D::Post { member: 0, gate: 0, posting: Posting::Received });
    w.decide(D::Proved);
    w.decide(D::Merge { member: 0, candidate: FRESH });
    w.decide(D::Landed { member: 0, prefix: FRESH });
    w.decide(D::Finish);
    assert_eq!(w.main_move, None);
    assert_eq!(w.main, FRESH);
}
#[test]
fn main_moving_before_a_cuts_close_is_answered_prepares_and_lands() {
    cut_on_a_stale_base(MainMove::Closing);
}
#[test]
fn main_moving_after_cut_cleanup_prepares_and_lands() {
    cut_on_a_stale_base(MainMove::Cleaned);
}
#[test]
fn main_moving_after_the_kept_round_opens_prepares_and_lands() {
    cut_on_a_stale_base(MainMove::Opened);
}
fn proved_squash() -> World {
    use policy::Decision as D;
    let mut w = World::new(1, true, false);
    w.prepared(true);
    w.entry = CLEANUP_PULL - 1;
    w.decide(D::Open);
    w.decide(D::None);
    w.decide(D::Proved);
    w
}

#[test]
fn a_stale_squash_turn_without_gates_cleans_up_and_prepares_again() {
    use policy::Decision as D;
    let mut w = proved_squash();
    let since = w.env.wall;
    assert_eq!(w.d.landing(TASK).unwrap().turn_since, Some(since));
    w.main = [19; 32];
    w.decide(D::None);
    assert_eq!(w.d.landing(TASK).unwrap().turn_since, Some(since));
    w.decide(D::Close);
    w.decide(D::Delete { member: 0 });
    w.decide(D::Stale);
    assert_eq!(w.d.landing(TASK).unwrap().turn_since, None);
    w.decide(D::Prepare { member: 0, head: fixtures::HEAD, main: [19; 32] });
    assert_eq!(w.d.landing(TASK).unwrap().landing.preparation, 2);
}

fn stale_cleanup() -> World {
    let mut w = proved_squash();
    w.main = [19; 32];
    w.decide(policy::Decision::None);
    w
}

fn refuse_cleanup(pull_already_closed: Option<u64>, write: policy::Decision) -> World {
    let mut w = stale_cleanup();
    w.closed_pull = pull_already_closed;
    w.refuse_cleanup = true;
    w.decide(write);
    w
}

#[test]
fn a_refused_cleanup_has_an_account_only_on_the_landing() {
    use policy::Decision as D;
    const CASES: &[(Option<u64>, D, &[u8])] =
        &[(None, D::Close, CLOSE_REFUSED), (Some(CLEANUP_PULL), D::Delete { member: 0 }, DELETE_REFUSED)];
    for &(closed, write, words) in CASES {
        let mut w = refuse_cleanup(closed, write);
        w.decide(CLEANUP_HOLD);
        assert_eq!(w.d.landing_hold_words(TASK).as_deref(), Some(words));
        assert_eq!(w.refusals.as_slice(), &[None], "one refused write, no root account");
    }
}

#[test]
fn another_tasks_branch_holds_cleanup_without_a_deletion() {
    let mut w = stale_cleanup();
    w.closed_pull = Some(CLEANUP_PULL);
    let m = &w.d.landing(TASK).unwrap().landing.members[0];
    let resource =
        skein_forge_client::Resource { repository: REPO, what: skein_forge_client::What::Branch(m.branch.clone()) };
    let name = domain::from_client(&resource, &w.env.limits).unwrap();
    w.send(Event::Restore {
        record: Stored::Hold(Hold {
            name,
            task: OTHER_HOLDER,
            writer: None,
            design_writers: Box::new([]),
            beside: None,
            drift: None,
        }),
    });
    w.refuse_cleanup = true;
    w.decide(CLEANUP_HOLD);
    let words = b"cleanup: delete branch: held by task 77; resource org/repo branch temper/1/l61-0";
    assert_eq!(w.d.landing_hold_words(TASK).as_deref(), Some(words.as_slice()));
    assert!(w.refusals.is_empty(), "no deletion sent");
}

fn held_deletion() -> World {
    let mut w = refuse_cleanup(Some(CLEANUP_PULL), policy::Decision::Delete { member: 0 });
    w.decide(CLEANUP_HOLD);
    w
}

#[test]
fn release_finds_an_externally_deleted_branch_and_goes_on() {
    let mut w = held_deletion();
    w.prefixes[0] = None;
    w.release_cleanup();
    w.decide(policy::Decision::Stale);
    w.decide(policy::Decision::Prepare { member: 0, head: fixtures::HEAD, main: [19; 32] });
    assert_eq!(w.refusals.as_slice(), &[None], "no second deletion");
}

#[test]
fn a_second_refusal_holds_after_one_released_deletion() {
    let mut w = held_deletion();
    w.release_cleanup();
    w.decide(policy::Decision::Delete { member: 0 });
    w.decide(CLEANUP_HOLD);
    assert_eq!(w.d.landing_hold_words(TASK).as_deref(), Some(DELETE_REFUSED));
    assert_eq!(w.refusals.as_slice(), &[None, None], "one deletion per try");
}

#[test]
fn a_restarted_cleanup_hold_has_the_same_account() {
    let w = held_deletion();
    let saved = Stored::Landing(w.d.landing(TASK).unwrap().clone());
    let mut cold = World::new(1, true, false);
    cold.send(Event::Restore { record: saved });
    cold.decide(policy::Decision::Wait { until: None, on: policy::Awaited::Release });
    assert_eq!(cold.d.landing_hold_words(TASK).as_deref(), Some(DELETE_REFUSED));
}
