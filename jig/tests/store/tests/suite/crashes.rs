use jig_protocol_store::{TranscriptRead, Write};
use jig_store_world::{
    World,
    referee::Referee,
    root::{self, LIVE},
};

fn committed(seed: u64) -> World {
    let mut world = World::new(seed);
    assert_eq!(world.commit(root::first()), Some(1));
    world
}

fn checked_cut(cut: u32, writes: Box<[Write]>, label: &str) -> (jig_store_world::Restored, Option<u64>) {
    let mut world = committed(u64::from(cut));
    world.cut_at(cut);
    let acknowledged = world.commit(writes);
    world.crash(u64::from(cut));
    let restored = world.reopen();
    Referee::check(&world.model, &restored)
        .unwrap_or_else(|error| panic!("{label}, cut {cut}: {error:?}, {restored:?}"));
    (restored, acknowledged)
}

fn operations(writes: Box<[Write]>) -> u32 {
    let mut world = committed(18);
    assert_eq!(world.commit(writes), Some(2));
    world.operation_count()
}

#[test]
fn a_crash_at_each_file_operation_of_a_turn_keeps_a_whole_prefix() {
    let count = operations(root::turn_commit(2, LIVE, 1, b"one turn"));
    assert!(count <= 12, "turn commit's {count} file operations fit the prescribed cuts");
    for cut in 0..=12 {
        let label = "whole turn prefix";
        let (restored, acknowledged) = checked_cut(cut, root::turn_commit(2, LIVE, 1, b"one turn"), label);
        assert!(restored.last == 1 || restored.last == 2, "{label}, cut {cut}");
        if acknowledged.is_some() {
            assert_eq!(restored.last, 2, "acknowledged turn, cut {cut}");
        }
        let expected = if restored.last == 2 { vec![root::turn(LIVE, 1, 1, b"one turn")] } else { Vec::new() };
        assert_eq!(restored.turns(LIVE), expected, "{label}, cut {cut}");
    }
}

#[test]
fn a_crash_at_each_operation_of_a_plain_commit_keeps_a_whole_prefix() {
    let count = operations(root::plain_commit(2));
    for cut in 0..=count {
        let (restored, acknowledged) = checked_cut(cut, root::plain_commit(2), "whole plain prefix");
        assert!(restored.last == 1 || restored.last == 2, "whole plain prefix, cut {cut}");
        if acknowledged.is_some() {
            assert_eq!(restored.last, 2, "acknowledged plain commit, cut {cut}");
        }
        let expected = (restored.last == 2).then(|| root::live_task(LIVE + 1));
        assert_eq!(
            restored.rows.get(&jig_domain_core::Key::Tasks(jig_domain_tasks::Key::Live(LIVE + 1))),
            expected.as_ref(),
            "whole task row, cut {cut}"
        );
    }
}

fn account_claim() -> Box<[Write]> {
    use jig_domain_core::{CoreRecord, Record};
    let Record::Tasks(jig_domain_tasks::Stored::Live(mut task)) = root::live_task(LIVE + 1) else {
        unreachable!("live task fixture")
    };
    task.kept_account = Some(7);
    let Record::Core(CoreRecord::RunProof(mut proof)) = root::run_proof(LIVE + 1) else {
        unreachable!("claim fixture")
    };
    proof.accounts = Box::new([jig_domain_authority::AccountBinding { provider: 2, account: 7, source: 9 }]);
    proof.placement = Some(skein_lib::Token::new(3));
    Box::new([
        Write::Save(root::header(2)),
        Write::Save(Record::Tasks(jig_domain_tasks::Stored::Live(task))),
        Write::Save(Record::Core(CoreRecord::RunProof(proof))),
    ])
}

#[test]
fn a_crash_keeps_the_task_pin_and_claim_bindings_together() {
    use jig_domain_core::{CoreKey, Key};
    let count = operations(account_claim());
    let writes = account_claim();
    let [_, Write::Save(task), Write::Save(proof)] = writes.as_ref() else { unreachable!("claim writes") };
    for cut in 0..=count {
        let (restored, acknowledged) = checked_cut(cut, account_claim(), "account claim");
        assert_eq!(
            restored.rows.get(&Key::Tasks(jig_domain_tasks::Key::Live(LIVE + 1))),
            (restored.last == 2).then_some(task),
            "task pin, cut {cut}"
        );
        assert_eq!(
            restored.rows.get(&Key::Core(CoreKey::RunProof(LIVE + 1))),
            (restored.last == 2).then_some(proof),
            "claim bindings, cut {cut}"
        );
        assert!(acknowledged.is_none() || restored.last == 2, "acknowledged claim, cut {cut}");
    }
}

#[test]
fn a_frame_past_its_commit_is_cut_at_open() {
    let mut complete = committed(18);
    assert_eq!(complete.commit(root::turn_commit(2, LIVE, 1, b"uncommitted frame")), Some(2));
    let synced = complete.transcript_sync().expect("frame sync before map commit");
    let mut world = committed(18);
    let writes = root::turn_commit(2, LIVE, 1, b"uncommitted frame");
    world.cut_at(synced);
    assert_eq!(world.commit(writes), None);
    world.crash(18);
    let restored = world.reopen();
    assert_eq!(restored.last, 1);
    assert_eq!(restored.turns(LIVE), []);
    assert_eq!(world.commit(root::turn_commit(2, LIVE, 1, b"replacement frame")), Some(2));
    world.crash(19);
    let restored = world.reopen();
    assert_eq!(restored.turns(LIVE), [root::turn(LIVE, 1, 1, b"replacement frame")]);
}

#[test]
fn the_header_matches_the_store_after_each_crash() {
    let count = operations(root::plain_commit(2));
    for cut in 0..=count {
        let (restored, _) = checked_cut(cut, root::plain_commit(2), "header number matches whole prefix");
        assert_eq!(
            restored.rows.get(&jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment)),
            Some(&root::header(restored.last)),
            "header number, cut {cut}"
        );
    }
}

#[test]
fn a_dropped_transcript_stays_dropped_after_a_crash() {
    let mut world = committed(18);
    assert_eq!(world.commit(root::turn_commit(2, LIVE, 1, b"removed turn")), Some(2));
    assert_eq!(world.commit(Box::new([Write::Save(root::header(3)), Write::DropTranscript { task: LIVE }])), Some(3));
    world.crash(18);
    let restored = world.reopen();
    Referee::check(&world.model, &restored).expect("dropped transcript prefix");
    assert_eq!(restored.turns(LIVE), []);
    assert_eq!(world.transcript(TranscriptRead::Page { task: LIVE, backwards: false, after: None }).len(), 0);
}

fn enqueue(world: &mut World, number: u64, person: bool) {
    let writes = root::linked_commit(number, person);
    world.model.stage(number, &writes);
    world.enqueue(jig_protocol_store::Request::Commit { number, writes });
}

fn acknowledge(world: &mut World, events: &[jig_protocol_store::Event]) {
    for event in events {
        let jig_protocol_store::Event::Committed { number, .. } = event else { panic!("commit terminal: {event:?}") };
        world.model.acknowledge(*number);
    }
}

fn linked_world(snapshot: bool) -> World {
    let mut limits = root::limits();
    limits.commits.in_flight = 3;
    limits.store.queued = 3;
    limits.store.queued_bytes = u64::from(limits.store.commit) * 3;
    if snapshot {
        limits.store.snapshot_after = 1;
    }
    let mut world = World::unopened_observed(18_401, limits, None);
    assert_eq!(world.open(), Some(0));
    let mut writes = root::first().into_vec();
    writes.push(Write::Save(root::person(1)));
    assert_eq!(world.commit(writes.into_boxed_slice()), Some(1));
    world.trace_writes();
    world
}

fn linked_referee(world: &mut World, label: &str, cut: u32) {
    let restored = world.reopen();
    Referee::check(&world.model, &restored)
        .unwrap_or_else(|error| panic!("{label}, cut {cut}: {error:?}, {restored:?}"));
    assert!(restored.rows.contains_key(&root::person(1).key()), "{label}, cut {cut}");
    let jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(task)) =
        restored.rows.get(&root::live_task(LIVE).key()).expect("task row")
    else {
        panic!("live task")
    };
    let jig_domain_tasks::Party::Person(person) = task.requester else { panic!("person requester") };
    assert!(restored.rows.contains_key(&root::person(person).key()), "{label}: requester is present");
    if restored.last >= 3 {
        assert_eq!(person, root::LINKED_PERSON);
    }
}

#[test]
fn snapshots_racing_linked_commits_and_several_commits_in_flight_keep_whole_prefixes() {
    for snapshot in [false, true] {
        // The first write has one frame; the two queued commits share the
        // next write and sync. A small fixture ceiling exposes every snapshot.
        let pause = snapshot.then(|| {
            let mut probe = linked_world(true);
            enqueue(&mut probe, 2, true);
            let events = probe.settle();
            acknowledge(&mut probe, &events);
            probe.written().iter().find(|(_, bytes)| bytes.starts_with(b"SKVS")).expect("snapshot header written").0 + 1
        });
        let mut reference = linked_world(snapshot);
        let start = reference.operation_count();
        enqueue(&mut reference, 2, true);
        if let Some(pause) = pause {
            let events = reference.settle_until(pause);
            acknowledge(&mut reference, &events);
            assert!(reference.snapshot_chunk_pending(), "commits race the snapshot's map chunk");
        }
        enqueue(&mut reference, 3, false);
        enqueue(&mut reference, 4, true);
        let events = reference.settle();
        acknowledge(&mut reference, &events);
        assert_eq!(reference.model.acknowledged, 4);
        let count = reference.operation_count();
        if !snapshot {
            assert!(
                reference
                    .written()
                    .iter()
                    .any(|(_, bytes)| bytes.windows(4).filter(|bytes| *bytes == b"SKVC").count() == 2),
                "two queued commits share one write"
            );
        }
        eprintln!("snapshot {snapshot}: operations {start}..={count}, pause {pause:?}");
        for cut in start..=count {
            let mut world = linked_world(snapshot);
            enqueue(&mut world, 2, true);
            world.cut_at(cut);
            if let Some(pause) = pause {
                let events = world.settle_until(pause);
                acknowledge(&mut world, &events);
                if !world.reached_cut() {
                    assert!(world.snapshot_chunk_pending());
                    enqueue(&mut world, 3, false);
                    enqueue(&mut world, 4, true);
                }
            } else {
                enqueue(&mut world, 3, false);
                enqueue(&mut world, 4, true);
            }
            let events = world.settle();
            acknowledge(&mut world, &events);
            world.crash(u64::from(cut));
            linked_referee(&mut world, if snapshot { "snapshot race" } else { "group commit" }, cut);
        }
    }
}
