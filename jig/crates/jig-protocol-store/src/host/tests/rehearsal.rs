//! A rehearsal stays runnable across job commits and KV snapshots (protocol/store/jobs.md, 2–3).
use super::*;
use crate::{Job, JobAnswer, JobWrite, KeyRange, Keys, Owner, PatchSet, PatchSets, Write};

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

#[test]
fn rehearsal_with_paged_jobs_and_snapshots_never_waits_without_work_in_flight() {
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
    let mut pages = 0;
    let mut handed = 0;
    let mut consumed = 0;
    let mut ran = false;
    let mut opened = false;
    let mut snapshot_yields = 0_u32;
    for _ in 0_u32..3000 {
        world.step();
        while let Some(event) = world.above.pop() {
            #[expect(
                clippy::wildcard_enum_match_arm,
                reason = "the scenario rejects every unexpected opening terminal"
            )]
            match event {
                Event::Job { owner, of: Owner::Application, job, page } => {
                    assert_eq!(job.as_ref(), JOB.name);
                    assert!(page.rows.len() <= usize::try_from(limits.store.ops).unwrap());
                    handed += u64::try_from(page.rows.len()).unwrap();
                    pages += 1;
                    consumed += u64::try_from(page.rows.len().min(2)).unwrap();
                    let answer = answer(&page);
                    world.host.down(
                        &mut world.store,
                        &fixtures::env(),
                        crate::Request::Job { owner, answer: Ok(answer) },
                        &mut world.above,
                    );
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
                hosted.work_pending(fixtures::env().now),
                "rehearsal has no I/O in flight and must run again: {:?}",
                hosted.wait_state()
            );
            if skein_kv::is_ready(&hosted.component.stores.store, &limits.store)
                && !crate::patching::phase::ready(&hosted.component)
            {
                snapshot_yields += 1;
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
    let source = world.machine.reopen_root(0);
    assert_eq!(
        world.machine.open(source, b"generations/1/store/snapshot", How::Read),
        Err(skein_fake_machine::Refusal::NotFound),
        "snapshots only write the rehearsal copy"
    );
    world.machine.close(source);
    close_rehearsal(&mut world);
    world.close();
}
