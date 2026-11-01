//! Whole-project deletion at its families and cold durable cuts (draft 5, 8).
use jig_conformance::{
    Cut,
    scenarios::{Scenario, run},
};
use jig_conformance_world::{
    Testing, replay,
    scenarios::removal::{Family, check_family},
};
const SEED: u64 = 1880;
fn family(family: Family) {
    let world = replay::fresh(Scenario::ProjectRemoved, Cut::None, SEED, false).expect("project deletion conforms");
    check_family(&world, family).expect("family erased");
    if matches!(family, Family::Core | Family::Tasks) {
        for (key, row) in &world.store.rows {
            if matches!(
                row,
                jig_test_domain::Record::Connector { record: jig_test_connector::Record::Made { .. }, .. }
                    | jig_test_domain::Record::Core(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Update(
                        _
                    )))
            ) {
                eprintln!("permitted survivor at seed {SEED}: {key:?}");
            }
        }
    }
}
#[test]
fn a_deleted_project_leaves_no_core_row() {
    family(Family::Core);
}
#[test]
fn a_deleted_project_leaves_no_tasks_row() {
    family(Family::Tasks);
}
#[test]
fn a_deleted_project_leaves_no_people_row() {
    family(Family::People);
}
#[test]
fn a_deleted_project_leaves_no_notes_row() {
    family(Family::Notes);
}
#[test]
fn a_deleted_project_leaves_no_accounts_row() {
    family(Family::Accounts);
}
#[test]
fn a_deleted_project_leaves_no_settings_row() {
    family(Family::Settings);
}
#[test]
fn a_deleted_project_cut_at_each_chosen_point_ends_the_same() {
    let cuts = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let reached = std::rc::Rc::clone(&cuts);
    let baseline = replay::every_cut(Scenario::ProjectRemoved, SEED, move |cut, world| {
        assert!(world.outcome().crashes.contains(&cut));
        reached.borrow_mut().push(cut);
    })
    .expect("every durable setup, removal decision and erasure cut conforms");
    assert_eq!(*cuts.borrow(), (1..=baseline.commits).collect::<Vec<_>>());
    eprintln!("project deletion seed {SEED}: {} durable cuts", baseline.commits);
}
#[test]
fn a_deleted_projects_scenario_replays_from_its_seed() {
    let cut = Cut::Random { seed: SEED };
    let first = run::<Testing>(Scenario::ProjectRemoved, cut, SEED).expect("first replay").expect("bound");
    let second = run::<Testing>(Scenario::ProjectRemoved, cut, SEED).expect("second replay").expect("bound");
    assert_eq!(first, second);
}
