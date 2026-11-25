use super::*;
use skein_lib::List;

const OLDEST: u64 = 10;
const NEWER: u64 = 12;
const NEWEST: u64 = 14;
const CHANGE: u64 = 5;
const OTHER: u64 = 6;
const WHY: policy::Hold = policy::Hold::Claimed { change: CHANGE };
const WORDS: &[u8] = b"change 5 is a member of landing 10 too, which keeps it";
const NEWS: &[u8] = b"landing 12: held: change 5 is a member of landing 10 too, which keeps it";

fn restored(rows: &[u64]) -> Domain {
    let mut tasks = jig_tasks_kit::World::new(71, jig_tasks_kit::LIMITS);
    for task in rows {
        assert_eq!(
            tasks.make(jig_tasks_kit::PERSON_PARTY, vec![jig_tasks_kit::task(*task, &[])]),
            jig_tasks_kit::Reply::Made(vec![*task]),
        );
    }
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    for task in [CHANGE, OTHER] {
        let mut row = fixtures::opening_row();
        row.pull = Some(task);
        row.task = task;
        row.change.task = task;
        row.pending = None;
        row.goal = Some(task.checked_add(100).unwrap());
        let (cold, hot) = row.split();
        for record in [Stored::ChangeHot(Box::new(hot)), Stored::ChangeCold(Box::new(cold))] {
            outputs(&mut d, Event::Restore { record });
        }
    }
    for task in rows.iter().rev().copied() {
        let mut row = merge_landing_row(task != NEWER);
        row.landing.task = task;
        row.landing.phase = policy::Phase::Waiting;
        row.landing.members[0].change = CHANGE;
        row.landing.members[0].pull = CHANGE;
        if task == NEWER {
            let mut other = row.landing.members[0].clone();
            other.change = OTHER;
            other.pull = OTHER;
            row.landing.members = Box::new([row.landing.members[0].clone(), other]);
        }
        outputs(&mut d, Event::Restore { record: Stored::Landing(row) });
    }
    let mut live = jig_tasks_kit::domain(&tasks.env.limits, 71);
    for task in rows {
        jig_domain_tasks::step(
            &mut live,
            &tasks.env,
            jig_domain_tasks::Event::Restore {
                record: jig_domain_tasks::Stored::Live(Box::new(tasks.record(*task).clone())),
            },
            &mut Queue::uncapped(),
        );
    }
    d.start_links(&live, &env(), &mut Queue::uncapped()).unwrap();
    for event in [Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }, Event::CatchUp] {
        for request in &outputs(&mut d, event) {
            assert!(!matches!(request, Request::LandingDecision { .. }), "startup only wakes a due step");
        }
    }
    d
}

fn step(d: &mut Domain, task: u64, released: bool) -> Box<[Request]> {
    let heard = LandingHeard { released, ..preparing_heard(None) };
    outputs(d, Event::StepLanding { task, entry: 91, heard })
}

fn decision(requests: &[Request], task: u64) -> policy::Decision {
    let mut got = None;
    for request in requests {
        if let Request::LandingDecision { task: decided, decision, .. } = request
            && *decided == task
        {
            assert!(got.is_none(), "one decision for {task}");
            got = Some(*decision);
        }
    }
    got.expect("one due landing decision")
}

fn phase(d: &Domain, task: u64) -> &policy::Phase {
    &d.landing(task).unwrap().landing.phase
}

fn cancel(d: &mut Domain, task: u64) {
    outputs(d, Event::SettleEffects { task, root: task, ending: ReleaseEnding::Cancelled });
    assert_eq!(phase(d, task), &policy::Phase::Cancelled);
}

fn gathering(d: &Domain, task: u64) -> Option<u64> {
    for (_, pending) in &d.landing.pending {
        if pending.task == task {
            return Some(pending.entry);
        }
    }
    None
}

fn assert_held(d: &Domain, requests: &[Request], task: u64) {
    assert_eq!(decision(requests, task), policy::Decision::Hold(WHY));
    assert_eq!(phase(d, task), &policy::Phase::Held { was: Box::new(policy::Phase::Waiting), why: WHY });
    assert_eq!(d.landing_hold_words(task).as_deref(), Some(WORDS));
    assert_eq!(gathering(d, task), None, "no gather precedes the hold");
}

#[test]
fn a_restored_claim_holds_before_reads_and_tells_each_members_goal() {
    let mut d = restored(&[OLDEST, NEWER]);
    assert_eq!(d.landing_of(CHANGE), Some(OLDEST));
    assert_eq!(d.landing_of(OTHER), Some(NEWER));
    let requests = step(&mut d, NEWER, false);
    assert_held(&d, &requests, NEWER);
    let mut heard = List::with_capacity(2);
    for request in requests.as_ref() {
        if let Request::LandingNews { task, member, step: news @ LandingNews::Held } = request {
            let (change, words) = d.landing_news_words(*task, *member, *news).unwrap();
            heard.push((d.change(change).unwrap().goal.unwrap(), words)).unwrap();
        }
    }
    assert_eq!(heard.as_slice(), &[(CHANGE + 100, NEWS.into()), (OTHER + 100, NEWS.into())]);
    step(&mut d, OLDEST, false);
    assert_eq!(gathering(&d, OLDEST), Some(91), "oldest gathers for its real step as usual");
}

#[test]
fn release_repeats_the_hold_until_the_owner_ends() {
    let mut d = restored(&[OLDEST, NEWER]);
    step(&mut d, NEWER, false);
    let requests = step(&mut d, NEWER, true);
    assert_held(&d, &requests, NEWER);
    cancel(&mut d, OLDEST);
    assert_eq!(d.landing_of(CHANGE), Some(NEWER));
    let requests = step(&mut d, NEWER, true);
    assert_eq!(decision(&requests, NEWER), policy::Decision::None);
    assert_eq!(phase(&d, NEWER), &policy::Phase::Waiting);
    step(&mut d, NEWER, false);
    assert_eq!(gathering(&d, NEWER), Some(91));
}

#[test]
fn cancelling_the_held_claimant_keeps_the_older_membership() {
    let mut d = restored(&[OLDEST, NEWER]);
    let requests = step(&mut d, NEWER, false);
    assert_held(&d, &requests, NEWER);
    cancel(&mut d, NEWER);
    assert_eq!(d.landing_of(CHANGE), Some(OLDEST));
    assert_eq!(d.landing_of(OTHER), None);
}

#[test]
fn every_newer_claimant_holds_even_when_it_steps_first() {
    let mut d = restored(&[OLDEST, NEWER, NEWEST]);
    for task in [NEWEST, NEWER] {
        let requests = step(&mut d, task, false);
        assert_held(&d, &requests, task);
    }
    assert_eq!(d.landing_of(CHANGE), Some(OLDEST));
}

#[test]
fn the_first_due_step_still_holds_a_claim_found_at_start() {
    let mut d = restored(&[OLDEST, NEWER]);
    cancel(&mut d, OLDEST);
    assert_eq!(d.landing_of(CHANGE), Some(NEWER));
    let requests = step(&mut d, NEWER, false);
    assert_eq!(decision(&requests, NEWER), policy::Decision::Hold(WHY));
    assert_eq!(d.landing_hold_words(NEWER).as_deref(), Some(b"change 5 was claimed by an older landing".as_slice()));
    let requests = step(&mut d, NEWER, true);
    assert_eq!(decision(&requests, NEWER), policy::Decision::None);
    assert_eq!(phase(&d, NEWER), &policy::Phase::Waiting);
}
