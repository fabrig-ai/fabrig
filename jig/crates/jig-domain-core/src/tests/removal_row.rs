//! Kept deletion rows at restore (temper/docs/design/draft/playground-reset.md, 5).
use super::make_tree;
use crate::{CoreKey, CoreRecord, Key, ProjectRemoval, RemovalStage, RestoreRange, Restored, restore_cursor};

#[test]
fn a_restored_removal_row_is_held() {
    let (mut core, limits) = make_tree();
    let row = ProjectRemoval { project: 7, stage: RemovalStage::Ending };
    assert_eq!(core.restore_core(CoreRecord::Removal(row), &limits), Restored::Live);
    assert!(core.removing(7));
    assert!(!core.removing(8));
    assert_eq!(core.removal(7), Some(&row));
    assert_eq!(core.removal(8), None);
}

#[test]
fn a_second_removal_row_for_a_project_is_rejected() {
    let (mut core, limits) = make_tree();
    let row = ProjectRemoval { project: 7, stage: RemovalStage::Ending };
    assert_eq!(core.restore_core(CoreRecord::Removal(row), &limits), Restored::Live);
    let second = ProjectRemoval { project: 7, stage: RemovalStage::Rows };
    assert_eq!(core.restore_core(CoreRecord::Removal(second), &limits), Restored::Rejected);
    assert_eq!(core.removal(7), Some(&row));
}

#[test]
fn a_removal_row_past_the_project_bound_is_rejected() {
    let (mut core, limits) = make_tree();
    for project in 0..limits.authority.projects {
        let row = ProjectRemoval { project, stage: RemovalStage::Ending };
        assert_eq!(core.restore_core(CoreRecord::Removal(row), &limits), Restored::Live);
    }
    let project = limits.authority.projects;
    let row = ProjectRemoval { project, stage: RemovalStage::Ending };
    assert_eq!(core.restore_core(CoreRecord::Removal(row), &limits), Restored::Rejected);
    assert!(!core.removing(project));
}

#[test]
fn removal_rows_follow_the_other_live_core_rows_in_the_restore_cursor() {
    let ranges = [RestoreRange::Core, RestoreRange::Tasks];
    let removal = Key::Core(CoreKey::Removal(7));
    assert!(RestoreRange::Core.admits(&removal));
    assert_eq!(restore_cursor(&ranges, &removal), Some((1, removal)));
    let proof = Key::Core(CoreKey::RunProof(7));
    assert_eq!(restore_cursor(&ranges, &proof), Some((0, proof)));
    let task = Key::Tasks(jig_domain_tasks::Key::Live(7));
    assert_eq!(restore_cursor(&ranges, &task), Some((2, task)));
}
