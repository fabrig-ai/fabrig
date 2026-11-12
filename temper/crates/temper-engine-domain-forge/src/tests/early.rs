//! The top's early producer facts and checkout starts (domain/forge/change/questions.md, 1).
use super::*;
use crate::settings;
use skein_forge::api;
use temper_engine_domain_forge_change as change;

const CHILD: u64 = 71;

fn producing() -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = change::EffectResult::None;
    row.delegate = Some((CHILD, change::Delegate::Produce));
    row.delegate_status = change::Status::Unknown;
    row.change.last_head = None;
    row.change.state = change::State::Producing { requested: true };
    row.change.early = change::Early::Started(Box::new([
        change::StartedFrom { task: 41, head: fixtures::HEAD },
        change::StartedFrom { task: 42, head: fixtures::BASE },
    ]));
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row)) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d
}

fn seed(saved: bool) -> items::Seed {
    items::Seed {
        project: 5,
        modify: false,
        task: CHILD,
        parent: Some(fixtures::CHANGE),
        repository: REPO,
        run_branch: b"temper/r71".as_slice().into(),
        saved_branch: if saved { Some(b"temper/s71".as_slice().into()) } else { None },
    }
}

fn head(d: &mut Domain, branch: &[u8], commit: api::Commit) {
    outputs(
        d,
        Event::Restore {
            record: Stored::BranchHead(BranchHead {
                name: Name {
                    forge: REPO.forge,
                    repository: REPO.repository,
                    what: What::Branch(Box::new([branch.into()])),
                },
                commit,
                owned: true,
            }),
        },
    );
}

fn producing_under_rule(base: &[u8], last: GateLast, up_to_date: bool) -> Domain {
    let mut d = producing();
    let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
    row.base = base.into();
    row.change.early = change::Early::None;
    let repository = adopted();
    d.give_settings_defaults(Box::new([settings::Stock {
        kind: 1,
        scope: 0,
        number: 1,
        value: settings::SizedValue {
            size: 0,
            value: settings::Value::Rule(settings::LandingRule {
                sets: Box::new([]),
                pattern: GatePattern {
                    segments: Box::new([
                        b"forge".as_slice().into(),
                        repository.host,
                        repository.owner,
                        repository.name,
                        b"branch".as_slice().into(),
                    ]),
                    last,
                },
                enforce: true,
                ci: false,
                up_to_date,
                gates: Box::new([]),
                approvals: Box::new([]),
            }),
        },
    }]));
    head(&mut d, base, fixtures::BASE);
    d
}

#[test]
fn the_first_early_producer_merges_dependency_heads_in_order() {
    let d = producing();
    let item = d.workspace_item(seed(false)).unwrap();
    assert_eq!(
        item.start,
        items::Start::Merged { branch: b"main".as_slice().into(), heads: Box::new([fixtures::HEAD, fixtures::BASE]) }
    );
    assert_eq!(item.push.as_deref(), Some(b"temper/51".as_slice()));
    assert_eq!(item.holder, Some(fixtures::CHANGE));
    assert_eq!(item.landing.as_deref(), Some(b"main".as_slice()));
}

#[test]
fn an_early_changes_next_producer_merges_the_landing_tip_into_branch_or_saved_work() {
    for spent in [false, true] {
        for saved in [false, true] {
            let mut d = producing();
            head(&mut d, b"main", fixtures::BASE);
            // A single component containing '/' is sufficient for branch_matches.
            head(&mut d, b"temper/51", fixtures::HEAD);
            if spent {
                let row = d.changes.get_mut(&fixtures::CHANGE).unwrap();
                let change::Early::Started(heads) = &row.change.early else { panic!("early fixture") };
                row.change.early = change::Early::Spent(heads.clone());
            }
            let item = d.workspace_item(seed(saved)).unwrap();
            assert_eq!(
                item.start,
                items::Start::Merge {
                    branch: if saved { b"temper/s71".as_slice().into() } else { b"temper/51".as_slice().into() },
                    base: fixtures::BASE,
                }
            );
            assert_eq!(item.landing, Some(b"main".as_slice().into()));
        }
    }
}

#[test]
fn a_goal_branch_changes_next_producer_merges_the_landing_tip_into_branch_or_saved_work() {
    for saved in [false, true] {
        let mut d = producing_under_rule(b"temper/7/g7", GateLast::Open(b"temper".as_slice().into()), false);
        head(&mut d, b"temper/51", fixtures::HEAD);
        let item = d.workspace_item(seed(saved)).unwrap();
        assert_eq!(
            item.start,
            items::Start::Merge {
                branch: if saved { b"temper/s71".as_slice().into() } else { b"temper/51".as_slice().into() },
                base: fixtures::BASE,
            }
        );
        assert_eq!(item.landing, Some(b"temper/7/g7".as_slice().into()));
    }
}

#[test]
fn a_main_changes_next_producer_starts_from_its_branch() {
    for (saved, expected) in [
        (false, items::Start::Branch(b"temper/51".as_slice().into())),
        (true, items::Start::Saved(b"temper/s71".as_slice().into())),
    ] {
        let mut d = producing_under_rule(b"main", GateLast::Exact(b"main".as_slice().into()), true);
        head(&mut d, b"temper/51", fixtures::HEAD);
        assert_eq!(d.workspace_item(seed(saved)).unwrap().start, expected);
    }
}

#[test]
fn a_goal_branch_change_with_no_branch_or_saved_work_uses_the_landing_branch() {
    let d = producing_under_rule(b"temper/7/g7", GateLast::Open(b"temper".as_slice().into()), false);
    assert_eq!(d.workspace_item(seed(false)).unwrap().start, items::Start::Base(b"temper/7/g7".as_slice().into()));
}

#[test]
fn a_spent_start_with_no_branch_or_saved_work_uses_the_landing_branch() {
    let mut d = producing();
    d.changes.get_mut(&fixtures::CHANGE).unwrap().change.early = change::Early::Spent(Box::new([]));
    assert_eq!(d.workspace_item(seed(false)).unwrap().start, items::Start::Base(b"main".as_slice().into()));
}

#[test]
fn a_start_conflict_is_kept_only_for_the_live_producing_delegate() {
    let mut d = producing();
    for (child, tag) in [(CHILD + 1, tag(1, 2)), (CHILD, tag(1, 3))] {
        outputs(&mut d, Event::StartConflicted { child, tag, head: fixtures::HEAD });
        assert_eq!(d.start_conflict(fixtures::CHANGE), None);
    }
    outputs(&mut d, Event::StartConflicted { child: CHILD, tag: tag(1, 2), head: fixtures::HEAD });
    assert_eq!(d.start_conflict(fixtures::CHANGE), Some(fixtures::HEAD));
    assert!(d.procedure_wakes.contains_key(&fixtures::CHANGE));
    outputs(
        &mut d,
        Event::DelegateResult {
            task: fixtures::CHANGE,
            child: CHILD,
            status: change::Status::Failed,
            no_change: false,
            failed: false,
            cancelled: true,
            words: Box::new([]),
        },
    );
    assert_eq!(d.start_conflict(fixtures::CHANGE), None);
}

#[test]
fn the_first_branch_read_wakes_the_changes_dependents_through_the_root() {
    let mut d = producing();
    let name = Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([b"temper".as_slice().into(), b"51".as_slice().into()])),
    };
    assert_eq!(d.branch_head(&name), None);
    d.procedure_wakes.remove(&fixtures::CHANGE);
    fixtures::step_change(&mut d);
    fixtures::answer_next(&mut d, Ok(api::Answer::Commit(fixtures::HEAD)));
    assert_eq!(d.branch_head(&name), Some(fixtures::HEAD));
    assert_eq!(d.procedure_wakes.get(&fixtures::CHANGE), Some(&false), "the dependency itself needs no new step");
}
