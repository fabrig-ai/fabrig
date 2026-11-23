//! Composed worlds check the durable edge, not its acknowledgement.
use jig_conformance::store_check::StoreChecks;
use temper_engine_domain::{Record, Write};
use temper_engine_domain_world::store::Fault;
use temper_system_world::composed::{World, report_script};

#[test]
fn a_composed_chat_checks_every_commit() {
    let mut world = World::chat(910, report_script(false));
    world.store_checks(StoreChecks::every());
    world.run();
    let checks = world.store_check_counts();
    assert!(checks.commits() > 0);
    assert_eq!(checks.checked(), checks.commits());
}

#[test]
fn a_composed_held_commit_returns_its_refused_record() {
    let mut world = World::chat(910, report_script(false));
    world.store_checks(StoreChecks::every());
    let (cold, _) = temper_engine_forge_world::changes::change_row().split();
    let mut header = world.store.header();
    header.commits += 1;
    let commit = header.commits;
    world.store.submit(
        commit,
        Box::new([
            Write::Save(Record::Forge {
                row: Box::new(temper_engine_domain_forge::Stored::ChangeCold(Box::new(cold))),
            }),
            Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header)))),
        ]),
        0,
    );
    world.store.fault(Fault::Hold { commits: 1 });
    let error = world.try_tick().expect_err("the held commit is already durable");
    assert_eq!(error, format!("store check: commit {commit}: start refused at Link: ForgeRow(ChangeCold(51))"));
    assert_eq!(world.store_check_counts().checked(), 1);
    assert_eq!(world.store.held.front(), Some(&commit));
}
