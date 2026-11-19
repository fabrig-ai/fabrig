//! Restore, undo, refusals and independently replayed power cuts (restoring.md, 8).
use jig_protocol_store::{
    CheckpointName, CheckpointRefusal, Event, Request, RestoreRefusal, SecretOwner, SecretValue, Secrets, Write,
    host::Adoption,
};
use jig_store_world::{World, referee::Referee, root, world::Change};
use skein_lib::Token;
const OWNER: Token = Token::new(987);
pub(super) fn scene(seed: u64) -> (World, CheckpointName, u64) {
    let mut world = World::mixed_files_checkpoint(seed);
    let mut limits = root::limits();
    limits.checkpoints.keep = 1;
    limits.store.snapshot_after = 1;
    limits.store.segment = 1;
    world.restart_with_limits(seed, limits);
    world.checkpoint_reopen();
    world.mark_if_needed();
    let number = world.model.acknowledged + 1;
    let mut before = root::history_commit(number).into_vec();
    before.push(Write::Save(root::live_task(root::LIVE)));
    before.push(Write::DropTranscript { task: root::LIVE + 2 });
    world.commit(before.into_boxed_slice()).expect("history and drop before C");
    world.secret(Secrets::Keep {
        owner: SecretOwner::Service,
        number: 1,
        value: SecretValue::Digest { expiry: u64::MAX, digest: [1; 32] },
    });
    let events = world.request(Request::Checkpoint { owner: OWNER, draw: [1; 16] });
    let [Event::Checkpointed { checkpoint, .. }] = events.as_slice() else { panic!("source: {events:?}") };
    let name = checkpoint.name;
    let number = world.model.acknowledged + 1;
    world
        .commit(Box::new([Write::Save(root::header(number)), Write::Save(root::live_task(root::LIVE + 3))]))
        .expect("new task after C");
    let number = world.model.acknowledged + 1;
    world.commit(root::turn_commit(number, root::LIVE + 3, 1, b"new task's turn")).expect("turn after C");
    let number = world.model.acknowledged + 1;
    world
        .commit(Box::new([
            Write::Save(root::header(number)),
            Write::DropTranscript { task: root::LIVE + 1 },
            Write::DropTranscript { task: root::LIVE },
            Write::Erase(root::live_task(root::LIVE).key()),
            Write::Erase(jig_domain_core::Key::Tasks(jig_domain_tasks::Key::LiveCold(root::LIVE))),
            Write::Erase(root::run_proof(root::LIVE).key()),
            Write::Put { key: Box::from(b"later".as_slice()), value: Box::from(b"later".as_slice()) },
        ]))
        .expect("drop after C");
    let number = world.model.acknowledged + 1;
    world.commit(root::history_commit(number)).expect("snapshot after C");
    world.secret(Secrets::Keep {
        owner: SecretOwner::Service,
        number: 1,
        value: SecretValue::Digest { expiry: u64::MAX, digest: [2; 32] },
    });
    let later = world.model.acknowledged;
    world.crash(seed);
    (world, name, later)
}
fn restored(events: &[Event]) -> jig_protocol_store::Restored {
    let [Event::Restored { owner, restored }] = events else { panic!("restore terminal: {events:?}") };
    assert_eq!(*owner, OWNER);
    *restored
}
type Tree = std::collections::BTreeMap<Vec<u8>, Option<Vec<u8>>>;
pub(super) struct Pair {
    old: Tree,
    restored: Tree,
}
fn subtree(contents: &Tree, prefix: &[u8]) -> Tree {
    contents
        .iter()
        .filter_map(|(path, bytes)| {
            let suffix = path.strip_prefix(prefix)?;
            if !suffix.is_empty() && !suffix.starts_with(b"/") {
                return None;
            }
            Some((suffix.to_vec(), bytes.clone()))
        })
        .collect()
}
pub(super) fn oracle(world: &mut World, name: CheckpointName) -> Pair {
    let contents = world.contents();
    Pair {
        old: subtree(&contents, b"transcripts"),
        restored: subtree(&contents, &[b"checkpoints/".as_slice(), &name.bytes(), b"/transcripts"].concat()),
    }
}
pub(super) fn pair(world: &mut World, source: u64, later: u64, expected: &Pair) {
    let found = world.checkpoint_reopen();
    assert!(found.last == source || found.last == later, "whole old or restored prefix: {}", found.last);
    Referee::checkpoint(&world.model, &found, found.last)
        .unwrap_or_else(|why| panic!("matching maps/logs/transcripts: {why:?}"));
    let tree = subtree(&world.contents(), b"transcripts");
    assert_eq!(
        tree,
        if found.last == source { &expected.restored } else { &expected.old }.clone(),
        "the transcript tree belongs to the recovered maps"
    );
    world.model.acknowledged = found.last;
    world.mark_if_needed();
    let secret = if found.last == source { [1; 32] } else { [2; 32] };
    let events = world.secret(Secrets::Load { owner: SecretOwner::Service, after: None });
    let [Event::SecretsLoaded { rows, next: None, .. }] = events.as_slice() else {
        panic!("secret map page: {events:?}")
    };
    assert!(
        matches!(rows.as_ref(),[jig_protocol_store::SecretRow { number:1,value:SecretValue::Digest { digest,.. },.. }] if *digest==secret),
        "the secret map belongs to the same prefix"
    );
}

#[test]
fn a_restore_and_its_safety_undo_recover_each_maps_logs_and_transcripts() {
    let (mut world, name, later) = scene(41);
    let answer = restored(&world.restore_checkpoint(name, [2; 16]));
    assert_eq!((answer.generation, answer.safety.store), (2, later));
    assert!(
        world.paths().contains(&[b"checkpoints/".as_slice(), &name.bytes()].concat()),
        "safety retention skips its locked source even with keep=1"
    );
    world.crash(42);
    assert!(matches!(
        world.back().as_slice(),
        [Event::BackRefused { why: jig_protocol_store::BackRefusal::NoPrevious, .. }]
    ));
    world.crash(43);
    let found = world.checkpoint_reopen();
    Referee::checkpoint(&world.model, &found, name.store).expect("state at C");
    // Serving the secret needs the start mark; preserve the pre-restore referee for the undo.
    let prefixes = world.model.prefixes.clone();
    let acknowledged = world.model.acknowledged;
    world.model.acknowledged = found.last;
    world.mark_if_needed();
    assert!(
        matches!(world.secret(Secrets::Get { owner:SecretOwner::Service,number:1 }).as_slice(),[Event::Secret { value:Some(SecretValue::Digest { digest,.. }),.. }] if *digest == [1;32])
    );
    world.model.prefixes = prefixes;
    world.model.acknowledged = acknowledged;
    world.crash(44);
    let undo = restored(&world.restore_checkpoint(answer.safety, [3; 16]));
    assert_eq!(undo.generation, 3);
    world.crash(45);
    let found = world.checkpoint_reopen();
    Referee::checkpoint(&world.model, &found, later).expect("safety brings later state back");
    world.model.acknowledged = found.last;
    world.mark_if_needed();
    assert!(
        matches!(world.secret(Secrets::Get { owner:SecretOwner::Service,number:1 }).as_slice(),[Event::Secret { value:Some(SecretValue::Digest { digest,.. }),.. }] if *digest == [2;32])
    );
}
#[test]
fn every_open_and_back_resolve_each_durable_intent_row_before_their_checks() {
    use jig_protocol_store::component_v3::{CEILINGS, Record, RestoreIntent};
    use skein_fake_machine::Item;
    let (mut reference, name, _) = scene(71);
    let old = subtree(&reference.contents(), b"transcripts");
    let answer = restored(&reference.restore_checkpoint(name, [2; 16]));
    let contents = reference.contents();
    let new = subtree(&contents, b"transcripts");
    let intent = RestoreIntent {
        generation: answer.generation,
        active: skein_lib::sha256::digest(b"1\n"),
        source: Box::from(name.bytes()),
        safety: Box::from(answer.safety.bytes()),
        made: 1,
    };
    let bytes = intent.encode(&CEILINGS).expect("intent bytes");
    for (s, t, a, active) in [
        (true, true, false, b"1\n".as_slice()),
        (true, false, true, b"1\n"),
        (false, true, true, b"1\n"),
        (false, true, true, b"2\n"),
    ] {
        for operation in 0..3 {
            let mut items = Vec::new();
            for (path, data) in &contents {
                if path.starts_with(b"transcripts") || path.as_slice() == b"active" {
                    continue;
                }
                items.push(match data {
                    Some(bytes) => Item::file(path, bytes).mode(0o600),
                    None => Item::directory(path).mode(0o700),
                });
            }
            items.push(Item::file(b"active", active));
            items.push(Item::file(b"restore", &bytes));
            for (stands, prefix, tree) in [
                (s, b"transcripts.staged".as_slice(), &new),
                (t, b"transcripts", if s && !a { &old } else { &new }),
                (a, b"transcripts.aside", &old),
            ] {
                if !stands {
                    continue;
                }
                for (suffix, data) in tree {
                    let path = [prefix, suffix].concat();
                    items.push(match data {
                        Some(bytes) => Item::file(&path, bytes).mode(0o600),
                        None => Item::directory(&path).mode(0o700),
                    });
                }
            }
            let mut world = World::unopened(71);
            world.lay(&items);
            if operation == 1 {
                let events = world.stopped_adoption(Adoption::Checkpoint { draw: [4; 16] }, None);
                assert!(
                    matches!(events.as_slice(),[Event::Checkpointed { checkpoint,.. }] if checkpoint.name.store==name.store),
                    "checkpoint open: {events:?}"
                );
                world.crash(72);
            } else if operation == 2 {
                let events = world.stopped_adoption(Adoption::Back, None);
                assert!(
                    matches!(
                        events.as_slice(),
                        [Event::BackRefused { why: jig_protocol_store::BackRefusal::NoPrevious, .. }]
                    ),
                    "back: {events:?}"
                );
                world.crash(72);
            }
            let found = world.checkpoint_reopen();
            Referee::checkpoint(&reference.model, &found, name.store).expect("resolved before recovery");
            assert_eq!(subtree(&world.contents(), b"transcripts"), new);
        }
    }
}
#[test]
fn source_and_deployment_refusals_leave_the_live_pair_unchanged() {
    for case in 0..7 {
        let (mut world, mut name, later) = scene(50 + case);
        let path = [b"checkpoints/".as_slice(), &name.bytes()].concat();
        let mut held = None;
        match case {
            0 => held = Some(world.hold_path(b".")),
            1 => held = Some(world.hold_path(&path)),
            2 => name.suffix = 999,
            3 => {
                let file = [path.as_slice(), b"/checkpoint"].concat();
                let bytes = world.contents().remove(&file).flatten().expect("source record");
                let mut record = jig_protocol_store::codec::component::Checkpoint::decode(&bytes).expect("record");
                record.deployment = [99; 16];
                world.rewrite(&file, &record.encode().expect("record"));
            }
            4 => world.rewrite(&[path.as_slice(), b"/checkpoint"].concat(), b"undecodable"),
            5 | 6 => {
                let bytes =
                    world.contents().remove(&[path.as_slice(), b"/checkpoint"].concat()).flatten().expect("record");
                let record = jig_protocol_store::codec::component::Checkpoint::decode(&bytes).expect("record");
                let file = record
                    .files
                    .iter()
                    .find(|file| (file.part == 1 || file.part == 2) && file.length > 1)
                    .expect("nonempty immutable file");
                let leaf =
                    if file.part == 1 { b"snapshot".to_vec() } else { format!("log-{:020}", file.number).into_bytes() };
                let file = [path.as_slice(), b"/generations/1/store/", &leaf].concat();
                if case == 5 {
                    world.rewrite(&file, b"x");
                } else {
                    world.remove_source(&file);
                }
            }
            _ => unreachable!(),
        }
        let before = world.contents();
        let events = world.restore_checkpoint(name, [2; 16]);
        let [Event::RestoreRefused { why, .. }] = events.as_slice() else { panic!("case {case}: {events:?}") };
        assert!(
            matches!(
                (case, why),
                (0, RestoreRefusal::Busy)
                    | (1, RestoreRefusal::Source { why: CheckpointRefusal::InUse })
                    | (2, RestoreRefusal::Source { why: CheckpointRefusal::Missing })
                    | (3, RestoreRefusal::Deployment)
                    | (4, RestoreRefusal::Unreadable { .. })
                    | (5, RestoreRefusal::Staging { error: None, .. })
                    | (6, RestoreRefusal::Staging { error: Some(_), .. })
            ),
            "case {case}: {why:?}"
        );
        let after = world.contents();
        for (path, bytes) in
            before.iter().filter(|(path, _)| path.as_slice() == b"active" || path.starts_with(b"transcripts/"))
        {
            assert_eq!(after.get(path), Some(bytes));
        }
        assert!(
            !after.keys().any(|path| path.starts_with(b"generations/2") || path.as_slice() == b"restore"),
            "case {case}, {why:?}, staged: {:?}",
            after.keys().filter(|path| path.starts_with(b"generations/2")).collect::<Vec<_>>()
        );
        if case != 3 && case < 5 {
            assert_eq!(before, after, "refused before recovery");
        }
        if let Some(held) = held {
            world.release_path(held);
        }
        if case >= 5 {
            world.crash(case);
            let found = world.checkpoint_reopen();
            Referee::checkpoint(&world.model, &found, later).expect("failed staging left live state unchanged");
        }
    }
}
#[test]
fn each_restore_write_sync_link_and_rename_recovers_one_matching_pair() {
    let (mut reference, name, _) = scene(61);
    restored(&reference.restore_checkpoint(name, [2; 16]));
    let cuts = reference.restore_cuts();
    assert!(cuts.len() > 20, "safety, generation, tree and intent are durable separately");
    for cut in cuts {
        let (mut world, name, later) = scene(61);
        let expected = oracle(&mut world, name);
        world.cut_at(cut);
        world.restore_checkpoint(name, [2; 16]);
        world.crash(u64::from(cut));
        pair(&mut world, name.store, later, &expected);
    }
}
#[test]
fn failed_restore_renames_links_copies_and_intent_reopen_as_one_matching_pair() {
    let (mut reference, name, _) = scene(62);
    restored(&reference.restore_checkpoint(name, [2; 16]));
    let mut faults = Vec::new();
    for (at, kind) in reference.changes() {
        if matches!(kind, Change::Rename | Change::Link | Change::Write) {
            faults.push(*at);
        }
    }
    assert!(faults.len() > 5);
    for at in faults {
        let (mut world, name, later) = scene(62);
        let expected = oracle(&mut world, name);
        let events = world.stopped_adoption(Adoption::Restore { name, draw: [2; 16] }, Some(at));
        assert!(matches!(events.as_slice(), [Event::RestoreRefused { .. }]), "fault {at}: {events:?}");
        world.crash(u64::from(at));
        pair(&mut world, name.store, later, &expected);
    }
}
