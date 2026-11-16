//! A rehearsal stays runnable across job commits and KV snapshots (protocol/store/jobs.md, 2–3).
use super::*;
use crate::{Job, JobAnswer, JobWrite, KeyRange, Keys, Owner, PatchSet, PatchSets, Write};
use jig_domain_core_vocabulary as core;
use jig_domain_tasks as tasks;
use skein_io::file;

const BASE: PatchSet = PatchSet {
    owner: Owner::Application,
    name: "fixture",
    families: &[crate::KeptFamily {
        retired: &[],
        current: jig_test_patches::patches::kept::fixture_label::TEXTS[0],
        types: &[],
        name: "fixture-label",
        texts: &[jig_test_patches::patches::kept::fixture_label::TEXTS[0]],
        map: crate::KeptMap::Store,
        keys: &[Keys { from: b"l", to: b"m", root: "Label" }],
        frames: None,
        flights: &[],
    }],
    level: Some(0),
    patches: &[],
    fresh: None,
    jobs: &[],
};
const JOB: Job = Job {
    name: "rewrite-labels",
    keys: &[KeyRange { from: b"l", to: b"m" }],
    converts: None,
    settles: false,
    counts: &[],
};
const ROWS: u64 = 9;

fn source() -> World {
    let mut world = World::new();
    world.store.patch_sets(PatchSets { sets: &[BASE] });
    world.open();
    // Three rows and a header per source commit, within the fixture's four-write edge.
    for number in 1_u8..=3 {
        let mut writes = skein_lib::List::with_capacity(4);
        writes.push(fixtures::header(u64::from(number))).unwrap();
        for row in 0_u8..3 {
            let mut value = skein_lib::Writer::new(7);
            value.put(&1_u16.to_be_bytes()).unwrap();
            value.put(&1_u32.to_be_bytes()).unwrap();
            value.put(b"a").unwrap();
            writes.push(Write::Put { key: Box::new([b'l', b'/', number, b'/', row]), value: value.finish() }).unwrap();
        }
        world.host.down(
            &mut world.store,
            &fixtures::env(),
            fixtures::commit(u64::from(number), writes.into_boxed()),
            &mut world.above,
        );
        world.settle();
        let Some(Event::Committed { number: found, .. }) = world.above.pop() else {
            panic!("source commit is durable");
        };
        assert_eq!(found, u64::from(number));
        assert!(world.above.is_empty());
    }
    // The fixture keeps task history in payload logs.
    world.store.history_routing = true;
    world.host.down(
        &mut world.store,
        &fixtures::env(),
        fixtures::commit(
            4,
            Box::new([
                fixtures::header(4),
                fixtures::save(fixtures::result_index(fixtures::ALICE, 1, fixtures::GOAL)),
                fixtures::save(fixtures::ended_of(fixtures::GOAL, tasks::Party::Person(fixtures::ALICE))),
                fixtures::save(fixtures::question_line(fixtures::GOAL, 7, 1)),
            ]),
        ),
        &mut world.above,
    );
    world.settle();
    let Some(Event::Committed { number: 4, .. }) = world.above.pop() else { panic!("history is durable") };
    world.close();
    world
}

fn answer(page: &crate::JobPage) -> JobAnswer {
    let mut through = None;
    let mut writes = skein_lib::List::with_capacity(2);
    for row in page.rows.iter().take(2) {
        through = Some(row.key.clone());
        writes.push(JobWrite::Put { key: row.key.clone(), value: row.value.clone() }).unwrap();
    }
    JobAnswer { writes: writes.into_boxed(), through, counts: Box::new([]) }
}

fn close_rehearsal(world: &mut World) {
    world.host.down(
        &mut world.store,
        &fixtures::env(),
        crate::Request::Close { owner: Token::new(41) },
        &mut world.above,
    );
    world.settle();
    let Some(Event::Closed { removed: true, .. }) = world.above.pop() else { panic!("scratch emptied") };
}

fn source_was_not_snapshotted(world: &mut World) {
    let source = world.machine.reopen_root(0);
    assert_eq!(
        world.machine.open(source, b"generations/1/store/snapshot", How::Read),
        Err(skein_fake_machine::Refusal::NotFound),
        "snapshots only write the rehearsal copy"
    );
    world.machine.close(source);
}

fn opened_rehearsal() -> World {
    let mut world = source();
    let source = world.machine.reopen_root(0);
    world.fd = world.sim.root(world.pid, Handle::new(source.raw()));
    let scratch = world.machine.lay(&[]);
    let scratch = world.sim.root(world.pid, Handle::new(scratch.raw()));
    let mut limits = fixtures::store_limits();
    limits.store.snapshot_after = 1;
    world.host = Drivers::new(&limits, 2);
    world.store = Store::on_directory(&limits, fixtures::APPLICATION, &[], world.host.seeded());
    world.store.patch_sets(PatchSets { sets: &[PatchSet { jobs: &[JOB], ..BASE }] });
    world
        .host
        .adopt_mode(world.fd, 0, Token::new(41), fixtures::INSTALLATION, super::super::Adoption::Rehearse { scratch })
        .unwrap();
    let mut pages = 0_u64;
    let mut handed = 0_u64;
    let mut consumed = 0_u64;
    let mut ran = false;
    let mut opened = false;
    let mut snapshot_yields = 0_u32;
    let mut job_snapshot_yields = 0_u32;
    let mut answering = None;
    for _ in 0_u32..3000 {
        world.step();
        if let Some((owner, answer)) = answering.take() {
            world.host.down(
                &mut world.store,
                &fixtures::env(),
                crate::Request::Job { owner, answer: Ok(answer) },
                &mut world.above,
            );
        }
        while let Some(event) = world.above.pop() {
            #[expect(
                clippy::wildcard_enum_match_arm,
                reason = "the scenario rejects every unexpected opening terminal"
            )]
            match event {
                Event::Job { owner, of: Owner::Application, job, page } => {
                    assert_eq!(job.as_ref(), JOB.name);
                    assert!(page.rows.len() <= usize::try_from(limits.store.ops).unwrap());
                    handed = handed.checked_add(u64::try_from(page.rows.len()).unwrap()).unwrap();
                    pages = pages.checked_add(1).unwrap();
                    consumed = consumed.checked_add(u64::try_from(page.rows.len().min(2)).unwrap()).unwrap();
                    // Let background snapshots run while the shell prepares its answer.
                    answering = Some((owner, answer(&page)));
                }
                Event::Ran { of: Owner::Application, job, rows, commits, .. } => {
                    assert_eq!(job.as_ref(), JOB.name);
                    assert_eq!(rows, handed);
                    assert_eq!(consumed, ROWS);
                    assert_eq!(commits, pages);
                    assert!(commits > 2, "the job spans pages and durable commits");
                    ran = true;
                }
                Event::Opened { pending: true, .. } => opened = true,
                Event::Rehearsing { .. } | Event::Decoded { .. } | Event::Census { .. } => {}
                other => panic!("unexpected rehearsal event: {other:?}"),
            }
        }
        let hosted = Hosted {
            component: world.store,
            drivers: world.host,
            above: Queue::uncapped(),
            secrets: Queue::uncapped(),
            sources: Queue::uncapped(),
            env: fixtures::env().with(limits),
            not_ready: NotReady::Above,
            closing: false,
        };
        if !opened && hosted.idle() {
            assert!(
                answering.is_some() || hosted.work_pending(fixtures::env().now),
                "rehearsal has no I/O in flight and must run again: {:?}",
                hosted.wait_state()
            );
            if skein_kv::is_ready(&hosted.component.stores.store, &limits.store)
                && !crate::patching::phase::ready(&hosted.component)
            {
                snapshot_yields = snapshot_yields.checked_add(1).unwrap();
                if let Some(crate::patching::phase::Patching::Jobs(_)) = crate::directory::patching(&hosted.component) {
                    job_snapshot_yields = job_snapshot_yields.checked_add(1).unwrap();
                }
            }
        }
        world.store = hosted.component;
        world.host = hosted.drivers;
        if opened {
            break;
        }
    }
    assert!(ran && opened, "the rehearsal finishes its job and opens");
    assert!(snapshot_yields > 0, "a snapshot yields before the open finishes");
    assert!(job_snapshot_yields > 0, "a snapshot advances while paged jobs are running");
    source_was_not_snapshotted(&mut world);
    world
}

#[test]
fn rehearsal_with_paged_jobs_and_snapshots_never_waits_without_work_in_flight() {
    let mut world = opened_rehearsal();
    close_rehearsal(&mut world);
    world.close();
}

#[test]
fn rehearsal_jobs_and_snapshots_finish_the_restart_census() {
    let mut world = opened_rehearsal();
    // Follow the snapshot file from sync to close before restart asks for
    // history (skein/docs/design/data/kv.md, 5 and 6.3).
    let mut snapshot = None;
    let mut closing = false;
    for _ in 0_u32..64 {
        crate::fire(&mut world.store, &fixtures::env(), &mut world.above, &mut world.host.below);
        for request in &world.host.below.store {
            if let file::Request::Sync { file, .. } = request {
                snapshot = Some(*file);
            }
            if let file::Request::Close { file, .. } = request
                && Some(*file) == snapshot
            {
                closing = true;
            }
        }
        world.step();
        if closing {
            break;
        }
    }
    assert!(closing, "the snapshot close awaits delivery at the pending generation switch");
    world.host.down(
        &mut world.store,
        &fixtures::env(),
        crate::Request::Restore {
            owner: Token::new(42),
            range: crate::Restore::Jig(Box::new([core::RestoreRange::Census(core::Census {
                results: Box::new([core::ReadResults { party: fixtures::ALICE, after: 0 }]),
                questions: Box::new([(fixtures::GOAL, 7)]),
            })])),
            after: None,
            rows: 4,
            bytes: 65_536,
        },
        &mut world.above,
    );
    for _ in 0_u32..3000 {
        world.step();
        if world.host.idle() && !crate::directory::ready(&world.store) {
            break;
        }
    }
    let Some(Event::Page { items, next: None, .. }) = world.above.pop() else {
        panic!("restart census must finish without another request");
    };
    let [
        crate::restore::Item::Row(crate::Row::Jig(core::Record::Tasks(tasks::Stored::Ended(ended)))),
        crate::restore::Item::Row(crate::Row::Jig(core::Record::Tasks(tasks::Stored::Activity(question)))),
    ] = items.as_ref()
    else {
        panic!("ended task and question joined from its payload log: {items:?}")
    };
    assert_eq!(ended.number, fixtures::GOAL);
    assert_eq!((question.task, question.position), (fixtures::GOAL, 7));
    assert!(world.above.is_empty());
    close_rehearsal(&mut world);
    world.close();
}
