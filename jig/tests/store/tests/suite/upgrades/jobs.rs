//! A pending owner job upgrades a copy, not its source (protocol/store/jobs.md, 1–3).
use jig_protocol_store::{Event, Owner, Request, Write, codec::component::Manifest};
use jig_store_world::{
    root,
    upgrade_fixtures::{self, BASE, Records, SETS},
};

#[test]
fn pending_job_upgrades_once_at_equal_levels() {
    let records = Records::story();
    let mut world = records.committed(901, BASE);
    let before = upgrade_fixtures::source(&mut world);
    world.owner_sets(SETS);
    let last = world.open().expect("pending job opens copy");
    assert!(world.pending, "the pending job requires an upgrade even at equal levels");
    assert_eq!(world.active(), b"1\n");
    assert_eq!(world.ran(), [(Owner::Application, upgrade_fixtures::JOB.name.into(), 3, 2)]);
    records.check_public(&mut world, &"pending job");
    upgrade_fixtures::kept(&before, &upgrade_fixtures::source(&mut world), &"pending job");
    world.switch().expect("root activates copy");
    let contents = world.contents();
    let bytes = contents.get(b"generations/2/manifest".as_slice()).and_then(Option::as_deref).expect("manifest");
    let manifest = Manifest::decode(bytes).expect("manifest codec");
    assert_eq!(manifest.before, manifest.after, "jobs do not advance owner levels");
    assert_eq!((manifest.generation, manifest.from, manifest.store), (2, Some(1), last));
    let next = last + 1;
    assert_eq!(world.commit(Box::new([Write::Save(root::header(next)), Write::Wires])), Some(next));
    world.crash(902);
    assert_eq!(world.open(), Some(next));
    assert!(!world.pending, "the job's done row prevents a second upgrade");
    assert_eq!(world.ran(), []);
    assert_eq!(world.active(), b"2\n");
    records.check(&mut world, &"job done");
    assert!(!world.paths().iter().any(|path| path.starts_with(b"generations/3")));
}

#[test]
fn a_pending_decider_job_repairs_a_live_row_once_without_moving_levels() {
    use jig_domain_core as core;
    use jig_domain_tasks::{self as tasks, Escalation, EscalationDecider, Party, Stored};
    use jig_protocol_store::{OwnerSet, OwnerSets};
    use jig_store_world::World;
    const BEFORE: OwnerSets = OwnerSets {
        sets: &[
            jig_protocol_store::SKEIN_SET,
            jig_protocol_store::SMITH_SET,
            OwnerSet { jobs: &[], ..jig_protocol_accounts::JIG_SET },
        ],
    };
    const NOW: OwnerSets = OwnerSets {
        sets: &[jig_protocol_store::SKEIN_SET, jig_protocol_store::SMITH_SET, jig_protocol_accounts::JIG_SET],
    };
    let mut world = World::unopened(907);
    world.owner_sets(upgrade_fixtures::current(BEFORE));
    assert_eq!(world.open(), Some(0));
    let core::Record::Tasks(Stored::Live(mut parent)) = root::live_task(1) else { panic!("parent") };
    let core::Record::Tasks(Stored::Live(mut held)) = root::live_task(3) else { panic!("held") };
    parent.delegates = Box::new([3]);
    held.requester = Party::Task(1);
    held.root = 1;
    held.depth = 1;
    held.phase =
        tasks::Phase::Held { was: tasks::Was::Active(tasks::Active::Due), why: tasks::Hold::Budget, source: None };
    held.escalation =
        Escalation::Rejected { revision: 1, by: EscalationDecider::Person(1), reason: b"keep held".as_slice().into() };
    assert_eq!(
        world.commit(Box::new([
            Write::Save(root::header(1)),
            Write::Wires,
            Write::Save(core::Record::Tasks(Stored::Live(parent))),
            Write::Save(core::Record::Tasks(Stored::Live(held.clone()))),
        ])),
        Some(1)
    );
    world.crash(908);
    world.owner_sets(upgrade_fixtures::current(NOW));
    let last = world.open().expect("decider job opens copy");
    assert!(world.pending);
    assert_eq!(world.ran().len(), 1);
    assert_eq!((world.ran()[0].0, world.ran()[0].1.as_ref()), (Owner::Jig, "escalation-deciders"));
    held.escalation =
        Escalation::Rejected { revision: 1, by: EscalationDecider::Task(1), reason: b"keep held".as_slice().into() };
    world.switch().expect("activate repaired store");
    assert_eq!(world.get(core::Key::Tasks(tasks::Key::Live(3))), core::Record::Tasks(Stored::Live(held.clone())));
    let contents = world.contents();
    let bytes = contents.get(b"generations/2/manifest".as_slice()).and_then(Option::as_deref).expect("manifest");
    let manifest = Manifest::decode(bytes).expect("manifest codec");
    assert_eq!(manifest.before, manifest.after);
    assert_eq!(world.commit(Box::new([Write::Save(root::header(last + 1)), Write::Wires])), Some(last + 1));
    world.crash(909);
    assert_eq!(world.open(), Some(last + 1));
    assert!(!world.pending);
    assert_eq!(world.ran(), []);
    assert_eq!(world.get(core::Key::Tasks(tasks::Key::Live(3))), core::Record::Tasks(Stored::Live(held)));
}

#[test]
fn private_check_refuses_the_copy_before_switch() {
    const SEED: u64 = 903;
    let records = Records::story();
    let mut trace = records.committed(SEED, BASE);
    trace.owner_sets(SETS);
    trace.trace_writes();
    trace.open_switched().expect("trace the seal before the check");
    let (seal_sync, _, _) = super::check::seal_and_switch(&trace);

    for linked in [false, true] {
        let mut world = records.committed(SEED, BASE);
        let before = upgrade_fixtures::source(&mut world);
        world.owner_sets(SETS);
        world.trace_writes();
        let mut events = world.begin_open();
        loop {
            let [Event::Job { owner, of, job, page }] = events.as_slice() else { panic!("fixture page: {events:?}") };
            let request =
                Request::Job { owner: *owner, answer: Ok(upgrade_fixtures::job(*of, job, page).expect("fixture job")) };
            if page.end {
                world.enqueue(request);
                break;
            }
            events = world.request(request);
        }
        let events = world.settle_until(seal_sync + 1);
        assert!(!events.iter().any(|event| matches!(event, Event::Opened { .. })), "paused before check");
        // Corrupt only the completed copy, after its private files have been made.
        let item = if linked {
            let contents = world.contents();
            let target = contents
                .iter()
                .find(|(path, bytes)| path.starts_with(b"generations/2/secrets/") && bytes.is_some())
                .expect("copy's secret file")
                .0;
            skein_fake_machine::Item::hard_link(b"generations/2/secrets/!public", target)
        } else {
            skein_fake_machine::Item::file(b"generations/2/secrets/!public", b"private check").mode(0o640)
        };
        world.lay(&[item]);
        let events = world.settle();
        assert!(
            matches!(events.as_slice(), [Event::OpenRefused { why: jig_protocol_store::OpenRefusal::Check { why: jig_protocol_store::Check::Private { name } }, .. }] if name.as_ref() == b"!public"),
            "{events:?}"
        );
        assert_eq!(world.active(), b"1\n");
        assert!(!world.paths().iter().any(|path| path == b"generations/2/manifest"));
        upgrade_fixtures::kept(&before, &upgrade_fixtures::source(&mut world), &"privacy refusal");
    }
}
