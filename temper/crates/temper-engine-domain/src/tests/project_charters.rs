//! Task creation through the child boundary for a project with no charter set.
use super::step_project_fixture::{World, old_store, start};
use crate::{Domain, Limits};
use jig_domain_tasks as tasks;
use skein_lib::Env;
use temper_engine_step_kit::{project_charters::*, start::limits};

#[test]
fn a_project_without_method_or_home_keeps_the_zero_task_stamp() {
    let mut world = World::new(&[]);
    let (row, sets) = world.revision_commit();
    assert!(!row.method);
    assert_eq!(sets, None);
    assert_zero_stamp(&mut world.root, &world.env);

    let limits = limits();
    let mut store = old_store(&limits, false);
    store.keep(revision(&[(IMPLEMENTER, WORDS), (WAYS, OFFERS), (WAY, BUILD)]));
    let (mut root, env) = start(&limits, &mut store, true);
    assert!(root.project_home(PROJECT).is_none());
    assert!(root.project_temper_revision(PROJECT).is_some(), "last good revision outlives its home status");
    assert_zero_stamp(&mut root, &env);
}

fn assert_zero_stamp(root: &mut Domain, env: &Env<Limits>) {
    let task = super::projection_settled::make_task(
        root,
        env,
        None,
        tasks::Executor::Agent { charter: 1 },
        tasks::Party::Deployment { project: PROJECT },
    );
    assert_eq!(root.core().project_charters(PROJECT), 0);
    assert_eq!(root.tasks().task(task).unwrap().charters, 0);
}
