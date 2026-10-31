use jig_conformance::Cut;
use jig_conformance_world::deletion::{self, Family};

#[test]
fn core_rows_of_a_deleted_project_go() {
    let _world = deletion::run(Family::Core, 71, Cut::None);
}
#[test]
fn tasks_rows_of_a_deleted_project_go() {
    let _world = deletion::run(Family::Tasks, 72, Cut::None);
}
#[test]
fn people_rows_of_a_deleted_project_go() {
    let _world = deletion::run(Family::People, 73, Cut::None);
}
#[test]
fn notes_of_a_deleted_project_go() {
    let _world = deletion::run(Family::Notes, 74, Cut::None);
}
#[test]
fn accounts_of_a_deleted_project_lose_its_use() {
    let _world = deletion::run(Family::Accounts, 75, Cut::None);
}
#[test]
fn the_applications_rows_of_a_deleted_project_go() {
    let _world = deletion::run(Family::Application, 76, Cut::None);
}
