//! The check of the tree a turn will land (temper/docs/design/domain/forge/temper-dir.md, 4).
use super::{
    GatherRead, Pending, Stage, ask, decide, observe_tip, policy, read_failed, repository, step, wait_for_record,
};
use crate::domain::temper_dir::{covered, fault_words, guards, home};
use crate::{Domain, LandingRow, Limits, Request, TemperDirEvidence};
use alloc::boxed::Box;
use skein_forge::api::{Answer, Commit, Error, Read};
use skein_lib::{Env, List, Queue, Token, len_u64, usize_of};
use temper_protocol_temper_dir::{Reading, Touch, touched};

/// The merge this gather may admit. No result survives the gather or applies
/// to a different prefix, tip or preparation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) struct Target {
    member: u16,
    tree: Commit,
    tip: Commit,
    preparation: u16,
}

impl Target {
    pub(super) fn new(row: &LandingRow, p: &Pending, member: u16, tree: Commit) -> Self {
        let tip = p.facts.get(usize::from(member)).expect("target facts").main;
        Self { member, tree, tip, preparation: row.landing.preparation }
    }

    fn participates(self, row: &LandingRow, index: usize, member: &policy::Member) -> bool {
        let top = usize::from(self.member);
        member.left.is_none()
            && member.landed.is_none()
            && if row.landing.round.branch.is_empty() { index == top } else { index <= top }
    }
}

#[derive(Hash, Debug)]
pub(super) struct Comparison {
    before: Commit,
    after: Commit,
    touch: Touch,
}

impl Comparison {
    pub(super) fn new(before: Commit, after: Commit, answer: &Result<Answer, Error>) -> Self {
        let touch = if let Ok(Answer::Compare { before: from, after: to, changed, files, .. }) = answer
            && *from == before
            && *to == after
        {
            touched(files, len_u64(files.len()) == *changed)
        } else {
            Touch::Unknown
        };
        Self { before, after, touch }
    }
}

#[derive(Hash, Debug)]
pub(super) struct Check {
    target: Target,
    touches: Box<[Touch]>,
    progress: Progress,
}

#[derive(Hash, Debug)]
enum Progress {
    Comparing(u16),
    Candidate(Reading),
    Base { candidate: Reading, base: Reading },
    Tip(Judgement),
    Checked(Judgement),
}

#[derive(Hash, Debug)]
pub(super) enum Judgement {
    Pass { covered: Box<[bool]> },
    Fault(CandidateFault),
}

/// The turn's reason for making these members leave and hold. The durable
/// leave is `Held`; its change's hold retains this account.
#[derive(Hash, Debug)]
pub(super) struct CandidateFault {
    pub(super) covered: Box<[bool]>,
    words: Box<[u8]>,
}

pub(super) fn checked(p: &Pending, target: Target) -> Option<&Judgement> {
    let check = p.temper_dir.as_deref()?;
    if check.target == target
        && let Progress::Checked(judgement) = &check.progress
    {
        Some(judgement)
    } else {
        None
    }
}

pub(super) fn evidence(p: &Pending, member: u16) -> TemperDirEvidence {
    let check = p.temper_dir.as_deref().expect("merge's candidate was checked");
    let Progress::Checked(Judgement::Pass { covered }) = &check.progress else {
        unreachable!("a merge requires a completed pass")
    };
    if *covered.get(usize::from(member)).expect("member coverage") {
        TemperDirEvidence::Passed { tree: check.target.tree }
    } else {
        TemperDirEvidence::Uncovered
    }
}

pub(super) fn start(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    p: Pending,
    row: &LandingRow,
    target: Target,
    out: &mut Queue<Request>,
) {
    if p.temper_dir.is_some() {
        step(d, env, p.task, p.entry, p.heard, out);
        return;
    }
    let mut touches = List::with_capacity(u32::try_from(row.landing.members.len()).expect("bounded members"));
    for _ in &row.landing.members {
        touches.push(Touch::None).expect("one touch per member");
    }
    let check = Box::new(Check { target, touches: touches.into_boxed(), progress: Progress::Comparing(0) });
    compare(d, env, owner, p, row, check, out);
}

fn compare(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    row: &LandingRow,
    mut check: Box<Check>,
    out: &mut Queue<Request>,
) {
    let Progress::Comparing(from) = check.progress else { unreachable!("comparing members") };
    let before = check.target.tip;
    for (index, member) in row.landing.members.iter().enumerate().skip(usize::from(from)) {
        if !check.target.participates(row, index, member) {
            continue;
        }
        let change = d.change(member.change).expect("member change");
        let repo = change.repository;
        if !guards(change, d.repository(repo).expect("adopted member repository")) {
            continue;
        }
        let after = p.facts.get(index).expect("member facts").head;
        let paths = p.paths.get(index).expect("member paths");
        if let Some(diff) = &paths.comparison
            && diff.before == before
            && diff.after == after
        {
            *check.touches.get_mut(index).expect("member touch") = diff.touch;
        } else {
            check.progress = Progress::Comparing(u16::try_from(index).expect("member position"));
            p.stage = Stage::TemperDir;
            p.temper_dir = Some(check);
            ask(d, env, owner, p, repo, Read::Compare { before, after }, out);
            return;
        }
    }
    if could_cover(&check.touches, true, true) {
        let candidate = Reading::new(check.target.tree, env.limits.client.answer_bytes);
        let read = candidate.next().expect("candidate root read");
        let repo = repository(row.landing.members.get(usize::from(check.target.member)).expect("target member"));
        check.progress = Progress::Candidate(candidate);
        p.stage = Stage::TemperDir;
        p.temper_dir = Some(check);
        ask(d, env, owner, p, repo, read, out);
    } else {
        finish(d, owner, p, check, false, false, out);
    }
}

fn could_cover(touches: &[Touch], candidate_method: bool, base_method: bool) -> bool {
    for touch in touches {
        if covered(*touch, candidate_method, base_method) {
            return true;
        }
    }
    false
}

pub(super) fn answered(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    row: &LandingRow,
    answer: &Result<Answer, Error>,
    out: &mut Queue<Request>,
) {
    if p.woken {
        step(d, env, p.task, p.entry, p.heard, out);
        return;
    }
    let mut check = p.temper_dir.take().expect("pending candidate check");
    match &mut check.progress {
        Progress::Comparing(index) => {
            let Read::Compare { before, after } = p.read else { unreachable!("member comparison") };
            // Even with a failed comparison, the candidate and base can establish coverage.
            p.paths.get_mut(usize::from(*index)).expect("member paths").comparison =
                Some(Comparison::new(before, after, answer));
            compare(d, env, owner, p, row, check, out);
        }
        Progress::Candidate(reading) | Progress::Base { base: reading, .. } => {
            if reading.take(answer).is_err() {
                read_failed(d, row, &p, answer.as_ref().err().copied(), out);
            } else {
                read_tree(d, env, owner, p, row, check, out);
            }
        }
        Progress::Tip(_) => {
            let Ok(Answer::Commit(tip)) = answer else {
                read_failed(d, row, &p, answer.as_ref().err().copied(), out);
                return;
            };
            if !row.landing.round.branch.is_empty() {
                let seen = observe_tip(d, &env.limits, p.repository, &row.landing.branch, Some(*tip), out);
                if wait_for_record(d, row, &p, seen, out) {
                    return;
                }
            }
            if *tip != check.target.tip {
                step(d, env, p.task, p.entry, p.heard, out);
                return;
            }
            let Progress::Tip(judgement) = check.progress else { unreachable!("tip follows judgement") };
            check.progress = Progress::Checked(judgement);
            p.temper_dir = Some(check);
            decide(d, owner, p, out);
        }
        Progress::Checked(_) => unreachable!("completed check reads nothing"),
    }
}

fn read_tree(
    d: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    mut p: Pending,
    row: &LandingRow,
    mut check: Box<Check>,
    out: &mut Queue<Request>,
) {
    let member = row.landing.members.get(usize::from(check.target.member)).expect("target member");
    let repo = repository(member);
    let mut base_method = false;
    let reading = match &check.progress {
        Progress::Candidate(reading) => reading,
        Progress::Base { candidate, base } => {
            base_method = base.method_present() != Some(false);
            candidate
        }
        Progress::Comparing(_) | Progress::Tip(_) | Progress::Checked(_) => unreachable!("reading .temper/"),
    };
    let method = reading.method_present() != Some(false);
    if !method
        && !base_method
        && could_cover(&check.touches, false, true)
        && check.target.tree != check.target.tip
        && let Progress::Candidate(_) = &check.progress
    {
        let Progress::Candidate(candidate) = check.progress else { unreachable!("first base read") };
        let base = Reading::new(check.target.tip, env.limits.client.answer_bytes);
        let read = base.next().expect("base root read");
        check.progress = Progress::Base { candidate, base };
        p.temper_dir = Some(check);
        ask(d, env, owner, p, repo, read, out);
        return;
    }
    if !could_cover(&check.touches, method, base_method) {
        finish(d, owner, p, check, method, base_method, out);
        return;
    }
    if method && let Some(read) = reading.next() {
        p.temper_dir = Some(check);
        ask(d, env, owner, p, repo, read, out);
        return;
    }
    // As in the change's own check, a tree known not to hold a method passes.
    let words = if method {
        let verdict = reading.judge(home(d, repo));
        if verdict.faults.is_empty() {
            None
        } else {
            Some(
                fault_words(check.target.tree, &verdict, usize_of(jig_domain_tasks::HOLD_ACCOUNT_BYTES))
                    .expect("task hold fits the check's heading and count"),
            )
        }
    } else {
        None
    };
    let judgement = judgement(&check.touches, method, base_method, words);
    let branch = if row.landing.round.branch.is_empty() {
        d.change(member.change).expect("target change").base.clone()
    } else {
        row.landing.branch.clone()
    };
    check.progress = Progress::Tip(judgement);
    p.temper_dir = Some(check);
    // Don't reuse the tip cached before the file reads.
    p.reads.remove(&(repo, GatherRead::Branch(branch.clone())));
    ask(d, env, owner, p, repo, Read::Branch { branch }, out);
}

fn judgement(touches: &[Touch], method: bool, base_method: bool, words: Option<Box<[u8]>>) -> Judgement {
    let mut coverage = List::with_capacity(u32::try_from(touches.len()).expect("bounded members"));
    for touch in touches {
        coverage.push(covered(*touch, method, base_method)).expect("one coverage per member");
    }
    let covered = coverage.into_boxed();
    match words {
        Some(words) => Judgement::Fault(CandidateFault { covered, words }),
        None => Judgement::Pass { covered },
    }
}

fn finish(
    d: &mut Domain,
    owner: Token,
    mut p: Pending,
    mut check: Box<Check>,
    method: bool,
    base_method: bool,
    out: &mut Queue<Request>,
) {
    check.progress = Progress::Checked(judgement(&check.touches, method, base_method, None));
    p.temper_dir = Some(check);
    decide(d, owner, p, out);
}

pub(super) fn take_fault(p: &mut Pending) -> CandidateFault {
    let check = p.temper_dir.take().expect("faulted check");
    let Progress::Checked(Judgement::Fault(fault)) = check.progress else { unreachable!("only a fault holds members") };
    fault
}

pub(super) fn hold_members(
    d: &mut Domain,
    env: &Env<Limits>,
    row: &LandingRow,
    fault: CandidateFault,
    out: &mut Queue<Request>,
) {
    for (member, covers) in row.landing.members.iter().zip(fault.covered) {
        if covers && member.left == Some(policy::Leave::Held) {
            crate::domain::hold_candidate_member(d, env, member.change, &fault.words, out);
        }
    }
}
