//! The turn's `.temper/` check across the connector and a fake forge.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary Rust world and assertions"
)]
use super::*;
use crate::tests::temper_revision::World;
use skein_fake_forge_domain::{api as raw, commit_bytes, commit_count};
use skein_forge::api::{self, Answer, Commit, Error, Op, Read, Write};
use temper_engine_domain_forge_change as change;

const TASK: u64 = 61;
const FIRST: u64 = fixtures::CHANGE;
const SECOND: u64 = 52;
const MAIN: &[u8] = b"main";
const STACK: &[u8] = b"temper/1/l61";
const LOWER: &[u8] = b"temper/1/l61-1";
const METHOD: &[u8] = b".temper/ways/x.toml";
const GOOD: &[u8] = b"format = 1\n";
const BAD: &[u8] = b"format = 99\n";

struct Turn {
    d: Domain,
    forge: World,
    reads: Vec<Read>,
    base: Commit,
    head: Commit,
    candidate: Commit,
}

impl Turn {
    fn new(single: bool, covered: bool, fault: bool) -> Self {
        let mut forge = World::new(33, Vec::new());
        let base = forge.tip();
        fork(&mut forge, MAIN, base);
        fork(&mut forge, b"temper/51", base);
        let path = if covered { METHOD } else { b"src/first" };
        let head = forge.push(b"temper/51", path, GOOD);
        let first = open(&mut forge, b"temper/51");
        let mut d = landing_fixture();
        install_change(&mut d, FIRST, b"temper/51", first, head);
        let mut row = merge_landing_row(single);
        row.landing.members[0].head = Some(head);
        row.landing.members[0].base = Some(base);
        row.landing.members[0].pull = first;
        let candidate = if single {
            row.landing.members[0].candidate = Some(head);
            head
        } else {
            fork(&mut forge, LOWER, head);
            let prefix = forge.push(LOWER, path, GOOD);
            row.landing.members[0].branch = LOWER.into();
            row.landing.members[0].candidate = Some(prefix);
            fork(&mut forge, b"temper/52", base);
            let upper_head = forge.push(b"temper/52", b"src/other", b"other\n");
            let second = open(&mut forge, b"temper/52");
            install_change(&mut d, SECOND, b"temper/52", second, upper_head);
            fork(&mut forge, STACK, prefix);
            let mut tree = forge.push(STACK, b"src/other", b"other\n");
            if fault {
                tree = forge.push(STACK, METHOD, BAD);
                for n in 0_u16..17 {
                    tree = forge.push(STACK, format!(".temper/ways/y{n:02}.toml").as_bytes(), BAD);
                }
            }
            let round = open(&mut forge, STACK);
            let upper = policy::Member {
                unit: 1,
                change: SECOND,
                head: Some(upper_head),
                pull: second,
                branch: STACK.into(),
                candidate: Some(tree),
                ..row.landing.members[0].clone()
            };
            row.landing.members = Box::new([row.landing.members[0].clone(), upper]);
            row.landing.round.pull = Some(round);
            tree
        };
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::CatchUp);
        fixtures::repository_heard(&mut d, &env(), REPO);
        outputs(&mut d, Event::Landing { row });
        Self { d, forge, reads: Vec::new(), base, head, candidate }
    }

    fn step(&mut self, entry: u64, fault: ReadFault) -> Vec<Request> {
        let requests = outputs(
            &mut self.d,
            Event::StepLanding { task: TASK, entry, heard: preparing_heard(Some(policy::Preparation::Idle)) },
        );
        self.drain(requests, fault)
    }

    fn drain(&mut self, initial: Box<[Request]>, read_fault: ReadFault) -> Vec<Request> {
        let mut all = Vec::new();
        let mut queue = alloc::collections::VecDeque::from(initial.into_vec());
        let mut checked = false;
        for _ in 0_u16..140 {
            if queue.is_empty() {
                queue.extend(ready(&mut self.d).into_vec());
                if queue.is_empty() {
                    return all;
                }
            }
            let request = queue.pop_front().unwrap();
            if let Request::Call { call, op, repository } = request {
                assert_eq!(repository, REPO);
                let answer = match op {
                    Op::Read(read) => {
                        checked |= matches!(&read, Read::File { head, path, .. } if *head == self.candidate && path.as_ref() == METHOD);
                        self.reads.push(read.clone());
                        match read_fault {
                            ReadFault::File if matches!(&read, Read::File { path, .. } if path.as_ref() == METHOD) => {
                                Err(Error::Forbidden)
                            }
                            ReadFault::MoveTip
                                if checked
                                    && matches!(&read, Read::Branch { branch } if branch.as_ref() == MAIN)
                                    && self.forge.branch_tip(MAIN) == self.base =>
                            {
                                self.forge.push(MAIN, b"src/moved", b"moved\n");
                                self.read(&read)
                            }
                            ReadFault::None | ReadFault::File | ReadFault::MoveTip => self.read(&read),
                        }
                    }
                    Op::Write(Write::Merge { number, head, how: api::MergeHow::FastForwardOnly }) => {
                        let answer = self.forge.call(raw::Op::Write(raw::Write::Merge {
                            number,
                            head: commit_count(&head).unwrap(),
                            how: raw::MergeHow::FastForwardOnly,
                        }));
                        assert!(matches!(answer, Ok(raw::Answer::Merged { .. })), "{answer:?}");
                        Ok(Answer::Done)
                    }
                    Op::Write(write) => panic!("unexpected write: {write:?}"),
                };
                queue.extend(fixtures::answer(&mut self.d, call, answer).into_vec());
            } else {
                all.push(request);
            }
        }
        panic!("landing's reads did not settle: {:?}", self.reads);
    }

    fn read(&mut self, read: &Read) -> Result<Answer, Error> {
        if let Read::Pull { number, .. } | Read::Reviews { number, .. } = read {
            let raw::Answer::Pull(pull) = self.forge.call(raw::Op::Read(raw::Read::Pull { number: *number })).unwrap()
            else {
                panic!("not a pull")
            };
            if matches!(read, Read::Reviews { .. }) {
                assert!(pull.reviews.is_empty(), "review mapping is exercised at the merge judge");
                return Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false });
            }
            Ok(Answer::Pull(api::Pull {
                reviewers: pull.reviewers,
                number: pull.number,
                state: match pull.state {
                    raw::State::Open => api::State::Open,
                    raw::State::Closed => api::State::Closed,
                },
                head: pull.head,
                base: pull.base,
                commit: commit_bytes(pull.commit),
                base_commit: pull.base_commit.map(commit_bytes),
                merged: pull.merged.map(commit_bytes),
                mergeable: pull.mergeable,
                ci: api::Ci::None,
                queued: None,
                dequeued: None,
                size: None,
            }))
        } else if let Read::Commits { before, after, .. } = read {
            let raw::Answer::Commits { commits } = self
                .forge
                .call(raw::Op::Read(raw::Read::Commits {
                    before: commit_count(before).unwrap(),
                    after: commit_count(after).unwrap(),
                }))
                .unwrap()
            else {
                panic!("not a commit range")
            };
            Ok(Answer::Commits {
                commits: commits
                    .into_vec()
                    .into_iter()
                    .map(|c| api::CommitMessage {
                        id: commit_bytes(c.id),
                        parents: c.parents.into_vec().into_iter().map(commit_bytes).collect(),
                        message: c.message,
                    })
                    .collect(),
                more: false,
            })
        } else {
            self.forge.answer_for(REPO, read)
        }
    }

    fn merge(&mut self, entry: u64) {
        let requests = outputs(&mut self.d, Event::Committed { entry });
        self.drain(requests, ReadFault::None);
        assert_eq!(self.forge.branch_tip(MAIN), self.candidate);
    }
}

#[derive(Clone, Copy)]
enum ReadFault {
    None,
    File,
    MoveTip,
}

fn fork(forge: &mut World, branch: &[u8], commit: Commit) {
    forge
        .call(raw::Op::Write(raw::Write::CreateBranch {
            branch: branch.into(),
            commit: commit_count(&commit).unwrap(),
        }))
        .unwrap();
}

fn open(forge: &mut World, branch: &[u8]) -> u64 {
    let raw::Answer::Created(number) = forge
        .call(raw::Op::Write(raw::Write::OpenPull {
            title: b"a change".as_slice().into(),
            body: Box::new([]),
            head: branch.into(),
            base: MAIN.into(),
            opens_empty: false,
        }))
        .unwrap()
    else {
        panic!("pull did not open")
    };
    number
}

fn install_change(d: &mut Domain, task: u64, branch: &[u8], pull: u64, head: Commit) {
    let mut row = fixtures::opening_row();
    row.task = task;
    row.change.task = task;
    row.branch = branch.into();
    row.pull = Some(pull);
    row.change.way = change::Way::Turn;
    row.change.state = change::State::Ready { head, ready_since: env().wall };
    row.change.last_head = Some(head);
    d.changes.insert(task, row).unwrap();
}

fn evidence(requests: &[Request], id: u64) -> &[LandingEvidence] {
    for request in requests {
        if let Request::LandingDecision {
            decision: policy::Decision::Merge { .. },
            entry: Some(entry),
            evidence: Some(evidence),
            ..
        } = request
            && *entry == id
        {
            return evidence;
        }
    }
    panic!("no merge evidence for {id}: {requests:?}");
}

#[test]
fn the_turn_checks_the_stack_tree_not_the_members_own_head() {
    let mut turn = Turn::new(false, true, false);
    let requests = turn.step(91, ReadFault::None);
    let evidence = evidence(&requests, 91);
    assert!(turn.reads.contains(&Read::Compare { before: turn.base, after: turn.head }));
    assert_ne!(evidence[0].facts.head, Some(turn.candidate));
    assert_eq!(evidence[0].facts.temper_dir, TemperDirEvidence::Passed { tree: turn.candidate });
    assert_eq!(evidence[1].facts.temper_dir, TemperDirEvidence::Uncovered);
    assert!(
        turn.reads
            .iter()
            .any(|r| matches!(r, Read::File { head, path, .. } if *head == turn.candidate && path.as_ref() == METHOD))
    );
    assert!(!turn.reads.iter().any(|r| matches!(r, Read::File { head, .. } if *head == turn.head)));
    turn.merge(91);
}

#[test]
fn the_turns_owner_sign_off_judges_each_covered_prefix() {
    let mut turn = Turn::new(false, true, false);
    turn.d
        .landing
        .settings
        .insert(
            5,
            crate::settings::ProjectLanding { temper_dir_sign_off: true, ..crate::settings::ProjectLanding::default() },
        )
        .unwrap();
    let requests = turn.step(91, ReadFault::None);
    let members = evidence(&requests, 91);
    let prefix = members[0].facts.head.unwrap();
    let owner = Reviewer {
        person: 8,
        role: jig_domain_people::Role::Owner.number(),
        head: turn.head,
        verdict: api::Verdict::Approve,
        at: env().wall,
    };
    let judge = |evidence: &ChangeEvidence, reviewers: &[Reviewer]| {
        turn.d.judge_landing(5, 5, 0, TASK, evidence.head.unwrap(), evidence, reviewers)
    };
    assert_eq!(judge(&members[1].facts, &[]), Some(JudgeVerdict::Met));
    assert_eq!(judge(&members[0].facts, &[]), Some(JudgeVerdict::Wait));
    assert_eq!(
        judge(&members[0].facts, &[Reviewer { role: jig_domain_people::Role::Member.number(), ..owner }]),
        Some(JudgeVerdict::Wait)
    );
    assert_eq!(judge(&members[0].facts, &[owner]), Some(JudgeVerdict::Met));
    for stale in [
        ChangeEvidence { temper_dir: TemperDirEvidence::Passed { tree: prefix }, ..members[0].facts.clone() },
        ChangeEvidence { base_tip: turn.head, ..members[0].facts.clone() },
    ] {
        assert_eq!(judge(&stale, &[owner]), Some(JudgeVerdict::Wait));
    }
}

#[test]
fn a_green_prefix_gets_its_own_judgement() {
    let mut turn = Turn::new(false, true, true);
    let round = open(&mut turn.forge, LOWER);
    let mut row = turn.d.landing(TASK).unwrap().clone();
    row.landing.round.search = Some(policy::Search { tests: policy::SearchTests::Whole, green: 1, red: 2 });
    row.landing.round.pull = Some(round);
    restore_landing(&mut turn.d, row);
    let green = turn.forge.branch_tip(LOWER);
    let requests = turn.step(91, ReadFault::None);
    let members = evidence(&requests, 91);
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].facts.temper_dir, TemperDirEvidence::Passed { tree: green });
    assert!(
        turn.reads
            .iter()
            .any(|r| matches!(r, Read::File { head, path, .. } if *head == green && path.as_ref() == METHOD))
    );
    assert!(!turn.reads.iter().any(|r| matches!(r, Read::File { head, .. } if *head == turn.candidate)));
    turn.candidate = green;
    turn.merge(91);
}

#[test]
fn a_candidate_fault_holds_only_covered_members_without_repair() {
    let mut turn = Turn::new(false, true, true);
    let requests = turn.step(91, ReadFault::None);
    assert!(requests.iter().any(|r| matches!(
        r,
        Request::LandingDecision { decision: policy::Decision::Leave { member: 0, why: policy::Leave::Held }, .. }
    )));
    let row = turn.d.landing(TASK).unwrap();
    assert_eq!(row.landing.round.ending, Some(policy::RoundEnd::Leave));
    assert!(row.landing.members[1].left.is_none());
    assert!(turn.d.landing_of(FIRST).is_none());
    assert_eq!(turn.d.landing_of(SECOND), Some(TASK));
    let first = turn.d.change(FIRST).unwrap();
    let change::State::Held { hold, .. } = &first.change.state else { panic!("covered change didn't hold") };
    let heading = format!(
        "temper's check of .temper/ at {}:\n.temper/ways/x.toml:1: ",
        core::str::from_utf8(&commit_text(&turn.candidate)).unwrap()
    );
    assert!(hold.words.starts_with(heading.as_bytes()), "{:?}", core::str::from_utf8(&hold.words));
    let lines: Vec<_> = core::str::from_utf8(&hold.words).unwrap().lines().collect();
    let more: usize =
        lines.last().unwrap().strip_prefix("and ").unwrap().strip_suffix(" more").unwrap().parse().unwrap();
    assert_eq!(lines.len().saturating_sub(2).saturating_add(more), 18, "the count includes every omitted fault");
    assert_eq!(first.change.repairs, 0);
    assert!(!turn.d.entries.contains_key(&91));
    assert_eq!(turn.forge.branch_tip(MAIN), turn.base);
}

#[test]
fn an_uncovered_turn_does_not_read_the_candidate() {
    let mut turn = Turn::new(false, false, true);
    let requests = turn.step(91, ReadFault::None);
    assert!(evidence(&requests, 91).iter().all(|e| e.facts.temper_dir == TemperDirEvidence::Uncovered));
    assert!(
        !turn.reads.iter().any(|r| matches!(r, Read::File { head, .. } if *head == turn.candidate)),
        "{:?}",
        turn.reads
    );
    turn.merge(91);
}

#[test]
fn a_refused_candidate_file_reads_again_on_the_next_step() {
    let mut turn = Turn::new(false, true, false);
    let requests = turn.step(91, ReadFault::File);
    assert!(
        requests.iter().any(|r| matches!(
            r,
            Request::LandingDecision {
                decision: policy::Decision::Wait { until: Some(_), on: policy::Awaited::Read },
                ..
            }
        )),
        "{requests:?}"
    );
    assert!(!turn.d.entries.contains_key(&91));
    let requests = turn.step(92, ReadFault::None);
    assert_eq!(evidence(&requests, 92)[0].facts.temper_dir, TemperDirEvidence::Passed { tree: turn.candidate });
}

#[test]
fn the_single_landing_checks_the_tree_its_merge_would_land() {
    let mut turn = Turn::new(true, true, false);
    let requests = turn.step(91, ReadFault::None);
    assert_eq!(evidence(&requests, 91)[0].facts.temper_dir, TemperDirEvidence::Passed { tree: turn.candidate });
    turn.merge(91);
}

#[test]
fn a_tip_moving_after_the_file_read_makes_the_round_stale() {
    let mut turn = Turn::new(false, true, false);
    turn.step(91, ReadFault::MoveTip);
    assert_ne!(turn.forge.branch_tip(MAIN), turn.base);
    assert!(!turn.d.entries.contains_key(&91));
    let row = turn.d.landing(TASK).unwrap();
    assert_eq!(row.landing.round.ending, Some(policy::RoundEnd::Stale));
}
