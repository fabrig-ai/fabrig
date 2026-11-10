//! Copy recovery through the service (temper/docs/design/domain/forge/adoption.md, 5).
use temper_one_process_world::copy;

#[test]
fn a_copy_beside_a_repository_joins_its_project_keeping_the_sources_workflow() {
    let outcome = copy::beside_repository();
    assert_eq!(outcome.copies, 1, "the source was copied once");
}

#[test]
fn a_restart_mid_copy_finishes_and_adopts_once() {
    let outcome = copy::restart();
    assert!(outcome.copying, "the copying mark was kept");
    assert!(outcome.restarted, "the engine restarted under the slow source");
    assert!(outcome.reran, "the restarted engine reran the kept plan");
    assert_eq!(outcome.adopted, 1, "one saved adoption act");
}

#[test]
fn a_copy_cut_short_is_deleted_and_copied_again() {
    let outcome = copy::cut_short();
    assert!(outcome.copying, "a cut transfer keeps the copying mark");
    assert_eq!(outcome.deletions, 1, "the partial copy was deleted");
    assert_eq!(outcome.copies, 2, "Cut and Whole each ran once");
    assert_eq!(outcome.adopted, 1, "only the whole copy was adopted");
}
