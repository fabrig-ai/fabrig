//! Checkpoint cut, independent recovery and faults on the simulated disk.
use jig_protocol_store::{
    CheckpointName, CheckpointRefusal, Checkpointed, Event, Request, SecretOwner, SecretValue, Secrets, Write,
};
use jig_store_world::{World, referee::Referee, root};
use skein_io::{file, kernel::Error};
use skein_lib::Token;
const OWNER: Token = Token::new(987);
pub(super) fn populated(seed: u64) -> World {
    let mut world = World::new(seed);
    let mut first = root::first().into_vec();
    first.push(Write::Save(root::person(1)));
    assert_eq!(world.commit(first.into_boxed_slice()), Some(1));
    assert_eq!(world.commit(root::turn_commit(2, root::LIVE, 1, b"at the cut")), Some(2));
    world.secret(Secrets::Keep {
        owner: SecretOwner::Service,
        number: 1,
        value: SecretValue::Digest { expiry: u64::MAX, digest: [1; 32] },
    });
    world
}
fn request() -> Request {
    Request::Checkpoint { owner: OWNER, draw: [0; 16] }
}
pub(super) fn checkpoint(events: &[Event]) -> Checkpointed {
    *events
        .iter()
        .find_map(|event| match event {
            Event::Checkpointed { owner, checkpoint } => {
                assert_eq!(*owner, OWNER);
                Some(checkpoint)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("checkpoint terminal: {events:?}"))
}
pub(super) fn recovered(world: &mut World, name: CheckpointName) {
    let contents = world.contents();
    let base = [b"checkpoints/".as_slice(), &name.bytes()].concat();
    let record_path = [base.as_slice(), b"/checkpoint"].concat();
    let record = jig_protocol_store::codec::component::Checkpoint::decode(
        contents.get(&record_path).expect("whole checkpoint record").as_ref().expect("record file"),
    )
    .expect("checkpoint record decodes");
    assert_eq!(record.store, name.store);
    assert_eq!(record.deployment, root::DEPLOYMENT);
    for file in &record.files {
        let relative = match file.part() {
            1 => b"generations/1/store/snapshot".to_vec(),
            2 => format!("generations/1/store/log-{:020}", file.number()).into_bytes(),
            3 => [b"generations/1/store/".as_slice(), &skein_kv::log_path(file.number())].concat(),
            4 => format!("transcripts/{:016x}/{:016x}", file.number() >> 12, file.number()).into_bytes(),
            5 => b"generations/1/secrets/snapshot".to_vec(),
            6 => format!("generations/1/secrets/log-{:020}", file.number()).into_bytes(),
            part => panic!("checkpoint part {part}"),
        };
        let path = [base.as_slice(), b"/", &relative].concat();
        if file.way() == 3 {
            assert!(!contents.contains_key(&path));
            continue;
        }
        let bytes = contents.get(&path).expect("selected file present").as_ref().expect("selected regular file");
        if file.way() == 2 {
            assert_eq!(u64::try_from(bytes.len()).expect("file length"), file.length());
        } else {
            assert!(u64::try_from(bytes.len()).expect("file length") >= file.length());
        }
        if file.part() >= 5 {
            assert_eq!(world.links(&path), 1, "secrets are private copies");
        }
    }
    let mut copy = world.checkpoint_copy(name);
    let restored = copy.checkpoint_reopen();
    let retained = copy.retained();
    for file in &record.files {
        if file.part() == 3 {
            let key = [jig_protocol_store::logs::EXTENTS.as_slice(), &file.number().to_be_bytes()].concat();
            let extent = retained.get(key.as_slice()).expect("extent in copied map");
            assert_eq!(
                file.length(),
                u64::from_be_bytes(extent.as_ref().try_into().expect("extent is eight bytes")),
                "log copied to its extent at C"
            );
        } else if file.part() == 4 {
            let key = jig_protocol_store::keys::encode_own(
                &jig_protocol_store::keys::Own::Transcript { task: file.number() },
                1024,
            )
            .expect("places key");
            let places = jig_protocol_store::codec::component::Transcript::decode(
                retained.get(key.as_ref()).expect("places in copied map"),
            )
            .expect("places decode");
            assert_eq!(file.length(), places.length, "transcript copied to its length at C");
        }
    }
    Referee::checkpoint(&world.model, &restored, name.store)
        .unwrap_or_else(|why| panic!("checkpoint at C {name:?}: {why:?}"));
    copy.mark_if_needed();
    let events = copy.secret(Secrets::Get { owner: SecretOwner::Service, number: 1 });
    assert!(
        matches!(events.as_slice(), [Event::Secret { value: Some(SecretValue::Digest { digest, .. }), .. }] if *digest == [1;32]),
        "secrets map at C: {events:?}"
    );
}
fn acknowledge(world: &mut World, events: &[Event]) {
    for event in events {
        if let Event::Committed { number, .. } = event {
            world.model.acknowledge(*number);
        }
    }
}
fn later(world: &mut World, number: u64) {
    let writes: Box<[Write]> = Box::new([
        Write::Save(root::header(number)),
        Write::Put { key: Box::from(b"after".as_slice()), value: Box::from(b"after the cut".as_slice()) },
    ]);
    world.model.stage(number, &writes);
    world.enqueue(Request::Commit { number, writes });
}
fn selected(world: &mut World) -> Vec<Event> {
    let mut events = Vec::new();
    for _ in 0..1000 {
        if matches!(world.checkpoint_file(), Some(file::Request::MakeDirectory { path, .. }) if path.starts_with(b"checkpoints/"))
        {
            return events;
        }
        for lane in 0..4 {
            events.extend(world.checkpoint_step(Some(lane)));
        }
        if !matches!(world.checkpoint_file(), Some(file::Request::MakeDirectory { path, .. }) if path.starts_with(b"checkpoints/"))
        {
            assert!(
                !events.iter().any(|event| matches!(event, Event::Committed { .. })),
                "later work cannot answer while selection waits"
            );
        }
    }
    panic!("checkpoint did not reach selection");
}
#[test]
fn a_later_commit_waits_for_selection_then_answers_outside_the_checkpoint() {
    let mut world = populated(5);
    let mut proofs = vec![Write::Save(root::header(3))];
    for task in 100..120 {
        proofs.push(Write::Save(root::run_proof(task)));
    }
    assert_eq!(world.commit(proofs.into_boxed_slice()), Some(3));
    world.enqueue(request());
    let writes = root::turn_commit(4, root::LIVE, 2, b"after the cut");
    world.model.stage(4, &writes);
    world.enqueue(Request::Commit { number: 4, writes });
    assert!(world.checkpoint_step(None).is_empty(), "later commit waits before the barrier");
    let mut events = Vec::new();
    for _ in 0..30 {
        events.extend(world.checkpoint_step(Some(3)));
        if world.checkpoint_file().is_none() {
            break;
        }
    }
    world.enqueue(Request::Get { owner: Token::new(991), key: jig_protocol_store::Get::Jig(root::person(1).key()) });
    events.extend(world.checkpoint_step(None));
    assert!(
        events.iter().any(|event| matches!(event, Event::Got { owner, row: Some(_), .. } if *owner == Token::new(991))),
        "gets do not wait on the barrier"
    );
    world.enqueue(Request::Transcript {
        owner: Token::new(992),
        read: jig_protocol_store::TranscriptRead::Page { task: root::LIVE, after: None, backwards: false },
        rows: 2,
        bytes: 4096,
    });
    for _ in 0..10 {
        events.extend(world.checkpoint_step(Some(0)));
    }
    assert!(
        events.iter().any(
            |event| matches!(event, Event::Turns { owner, turns, .. } if *owner == Token::new(992) && turns.len() == 1)
        ),
        "transcripts go on without checkpoint file work"
    );
    events.extend(selected(&mut world));
    for _ in 0..30 {
        events.extend(world.checkpoint_step(Some(0)));
    }
    assert!(
        events.iter().any(|event| matches!(event, Event::Committed { number: 4, .. })),
        "later commit answered while copying is stopped: {events:?}"
    );
    assert!(!events.iter().any(|event| matches!(event, Event::Checkpointed { .. })));
    acknowledge(&mut world, &events);
    events.extend(world.secret(Secrets::Keep {
        owner: SecretOwner::Service,
        number: 1,
        value: SecretValue::Digest { expiry: u64::MAX, digest: [2; 32] },
    }));
    events.extend(world.settle());
    let taken = checkpoint(&events);
    assert_eq!(taken.name.store, 3);
    recovered(&mut world, taken.name);
}
#[test]
fn live_and_ended_histories_are_copied_to_their_extents_and_recover_at_c() {
    let mut world = World::history_world();
    assert_eq!(world.commit(root::history_commit(2)), Some(2));
    assert_eq!(world.commit(Box::new([Write::Save(root::header(3)), Write::Save(root::person(1))])), Some(3));
    world.secret(Secrets::Keep {
        owner: SecretOwner::Service,
        number: 1,
        value: SecretValue::Digest { expiry: u64::MAX, digest: [1; 32] },
    });
    let taken = checkpoint(&world.request(request()));
    let path = [b"checkpoints/".as_slice(), &taken.name.bytes(), b"/checkpoint"].concat();
    let bytes = world.contents().remove(&path).expect("record").expect("file");
    let record = jig_protocol_store::codec::component::Checkpoint::decode(&bytes).expect("record decodes");
    let histories: Vec<_> = record.files.iter().filter(|file| file.part() == 3 && file.number() & 1 == 0).collect();
    assert_eq!(histories.len(), 3);
    assert!(histories.iter().all(|file| file.way() == 2 && file.length() > 0));
    recovered(&mut world, taken.name);
}
#[test]
fn selected_transcript_and_log_drops_commit_then_remove_only_after_release() {
    let mut world = World::mixed_files_checkpoint(19);
    world.secret(Secrets::Keep {
        owner: SecretOwner::Service,
        number: 1,
        value: SecretValue::Digest { expiry: u64::MAX, digest: [1; 32] },
    });
    assert_eq!(world.commit(root::turn_commit(3, root::LIVE + 4, 1, b"log at C")), Some(3));
    world.enqueue(request());
    let mut events = selected(&mut world);
    let writes: Box<[Write]> = Box::new([
        Write::Save(root::header(4)),
        Write::DropTranscript { task: root::LIVE },
        Write::DropTranscript { task: root::LIVE + 4 },
    ]);
    world.model.stage(4, &writes);
    world.enqueue(Request::Commit { number: 4, writes });
    for _ in 0..100 {
        for lane in 0..3 {
            events.extend(world.checkpoint_step(Some(lane)));
        }
    }
    assert!(
        events.iter().any(|event| matches!(event, Event::Committed { number: 4, .. })),
        "drops commit while held: {events:?}"
    );
    acknowledge(&mut world, &events);
    let transcript = b"transcripts/0000000000000000/0000000000000015".to_vec();
    let log = [b"generations/1/store/".as_slice(), &skein_kv::log_path((root::LIVE + 4) * 2 + 1)].concat();
    let paths = world.paths();
    assert!(paths.contains(&transcript));
    assert!(paths.contains(&log));
    events.extend(world.settle());
    let paths = world.paths();
    assert!(!paths.contains(&transcript));
    assert!(!paths.contains(&log));
    recovered(&mut world, checkpoint(&events).name);
}
#[test]
fn refused_links_are_copied_and_counted_and_recover_the_cut() {
    let mut world = populated(6);
    world.crash(6);
    world.reopen();
    world.mark_if_needed();
    world.refuse_links(Some(Error::Permission));
    let taken = checkpoint(&world.request(request()));
    assert_eq!(taken.linked, 0);
    assert!(taken.copied >= 3);
    assert_eq!(taken.held, taken.bytes);
    recovered(&mut world, taken.name);
}
#[test]
fn a_snapshot_finishing_under_the_hold_renames_after_release_and_the_copy_keeps_the_old_one() {
    let mut limits = root::limits();
    limits.store.snapshot_after = 1;
    let mut world = World::unopened_observed(33, limits, None);
    assert_eq!(world.open(), Some(0));
    let mut first = root::first().into_vec();
    first.push(Write::Save(root::person(1)));
    assert_eq!(world.commit(first.into_boxed_slice()), Some(1));
    world.secret(Secrets::Keep {
        owner: SecretOwner::Service,
        number: 1,
        value: SecretValue::Digest { expiry: u64::MAX, digest: [1; 32] },
    });
    let snapshot = b"generations/1/store/snapshot".to_vec();
    let before = world.contents().get(&snapshot).expect("older snapshot").clone();
    let writes = root::turn_commit(2, root::LIVE, 1, b"snapshot at C");
    world.model.stage(2, &writes);
    world.enqueue(Request::Commit { number: 2, writes });
    let mut events = Vec::new();
    for _ in 0..100 {
        events.extend(world.checkpoint_step(None));
        if world.snapshot_chunk_pending() {
            break;
        }
        events.extend(world.checkpoint_step(Some(0)));
    }
    assert!(world.snapshot_chunk_pending());
    acknowledge(&mut world, &events);
    events.clear();
    world.enqueue(request());
    // Leave the pending snapshot chunk while admission and selection finish on the other lane.
    for _ in 0..100 {
        if matches!(world.checkpoint_file(), Some(file::Request::MakeDirectory { path, .. }) if path.starts_with(b"checkpoints/"))
        {
            break;
        }
        events.extend(world.checkpoint_step(Some(3)));
    }
    for _ in 0..100 {
        events.extend(world.checkpoint_step(Some(0)));
    }
    let contents = world.contents();
    assert_eq!(contents.get(&snapshot), Some(&before));
    assert!(contents.contains_key(b"generations/1/store/snapshot.tmp".as_slice()));
    assert!(!events.iter().any(|event| matches!(event, Event::Checkpointed { .. })));
    events.extend(world.settle());
    let taken = checkpoint(&events);
    let copy_snapshot = [b"checkpoints/".as_slice(), &taken.name.bytes(), b"/generations/1/store/snapshot"].concat();
    let contents = world.contents();
    assert_eq!(contents.get(&copy_snapshot), Some(&before));
    assert_ne!(contents.get(&snapshot), Some(&before));
    recovered(&mut world, taken.name);
}
#[test]
fn an_unknown_store_name_refuses_with_no_published_checkpoint() {
    let mut world = populated(7);
    world.lay(&[skein_fake_machine::Item::file(b"generations/1/store/alien", b"unknown")]);
    let events = world.request(request());
    assert!(
        matches!(events.as_slice(), [Event::CheckpointRefused { why: CheckpointRefusal::Unrecognized { path }, .. }] if &**path == "generations/1/store/alien"),
        "{events:?}"
    );
    assert!(!world.paths().iter().any(|path| path.starts_with(b"checkpoints/")));
    assert_eq!(world.commit(root::plain_commit(3)), Some(3));
}
#[test]
fn a_vanished_held_segment_refuses_cleans_and_releases_the_next_commit() {
    let mut world = populated(8);
    world.crash(8);
    world.reopen();
    world.mark_if_needed();
    let next = world.model.acknowledged + 1;
    world.enqueue(request());
    let mut events = Vec::new();
    let vanished = loop {
        if let Some(file::Request::HardLink { from, .. }) = world.checkpoint_file() {
            break [b"generations/1/store/".as_slice(), from].concat();
        }
        for lane in 0..4 {
            events.extend(world.checkpoint_step(Some(lane)));
        }
        assert!(events.is_empty(), "link before terminal: {events:?}");
    };
    world.remove_source(&vanished);
    later(&mut world, next);
    events.extend(world.settle());
    acknowledge(&mut world, &events);
    assert!(events.iter().any(|event| matches!(event, Event::CheckpointRefused { why: CheckpointRefusal::Vanished { path }, .. } if path.as_bytes() == vanished)), "{events:?}");
    assert!(events.iter().any(|event| matches!(event, Event::Committed { number, .. } if *number == next)));
    assert!(!world.paths().iter().any(|path| path.starts_with(b"checkpoints/")));
}
#[test]
fn power_loss_at_each_publication_file_request_leaves_a_temporary_or_a_whole_checkpoint() {
    let mut reference = populated(9);
    let start = reference.operation_count();
    let taken = checkpoint(&reference.request(request()));
    let end = reference.operation_count();
    let name = taken.name;
    for cut in start..=end {
        let mut world = populated(u64::from(cut));
        world.cut_at(cut);
        let events = world.request(request());
        let answered = events.iter().any(|event| matches!(event, Event::Checkpointed { .. }));
        world.crash(u64::from(cut));
        let exists = world.paths().contains(&[b"checkpoints/".as_slice(), &name.bytes()].concat());
        if answered {
            assert!(exists, "answered checkpoint survives power loss, cut {cut}");
        }
        if exists {
            recovered(&mut world, name);
        }
        let restored = world.reopen();
        Referee::check(&world.model, &restored).unwrap_or_else(|why| panic!("live store, cut {cut}: {why:?}"));
        assert!(
            !world.paths().iter().any(|path| path.windows(7).any(|part| part == b".skein-")),
            "next open sweeps temporary, cut {cut}"
        );
    }
}
