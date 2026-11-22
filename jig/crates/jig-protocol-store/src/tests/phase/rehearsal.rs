//! The switch and wires belong to the scratch; transcripts never do (opening.md, 5).
use super::*;

fn opened(pending: bool) -> (Disk, Store, Root, Vec<Item>) {
    let mut items = baseline_with_proof(true);
    items.push(Item::directory(b"transcripts/0000000000000000"));
    items.push(Item::file(b"transcripts/0000000000000000/0000000000000001", b"uncommitted tail"));
    let mut disk = Disk::new(&items);
    let mut store = disk.store_with_chunk(CHUNK);
    store.owner_sets(upgrade_sets(usize::from(pending)));
    assert!(store.wires(&[crate::Wire { name: "rehearsal-wire", version: 2 }]));
    let directory = disk.root();
    let scratch = disk.scratch_root();
    let events = disk.request(
        &mut store,
        Request::Rehearse {
            owner: Token::new(1),
            directory,
            scratch,
            installation: fixtures::INSTALLATION,
            reach: crate::Reach::Whole,
        },
    );
    let events = finish_empty_jobs(&mut disk, &mut store, events);
    assert!(
        events.iter().any(|event| if let Event::Opened { pending: found, .. } = event {
            *found == pending
        } else {
            false
        }),
        "{events:?}"
    );
    assert!(
        events.iter().any(|event| if let Event::Rehearsing { wires: Some(wires), .. } = event {
            wires.is_empty()
        } else {
            false
        }),
        "old wires read before the check commits: {events:?}"
    );
    if pending {
        let switched = disk.request(&mut store, Request::Switch { owner: Token::new(1) });
        let [Event::Switched { .. }] = switched.as_ref() else { panic!("{switched:?}") };
    }
    (disk, store, scratch, items)
}

#[test]
fn rehearsal_switch_and_wires_change_only_the_scratch() {
    for pending in [true, false] {
        let (mut disk, mut store, scratch, before) = opened(pending);
        let active = disk.file(
            2,
            file::Request::Load {
                owner: Token::new(77),
                root: scratch.token,
                path: Box::from(b"active".as_slice()),
                max: 32,
                no_follow: true,
            },
        );
        let file::Event::Loaded { bytes, .. } = active.event else { panic!("scratch active") };
        assert_eq!(bytes.as_ref(), if pending { b"2\n" } else { b"1\n" });
        let number = store.stores.store.last().checked_add(1).unwrap();
        let events = disk.request(&mut store, fixtures::commit(number, Box::new([Write::Wires])));
        let [Event::Committed { number: found, .. }] = events.as_ref() else { panic!("{events:?}") };
        assert_eq!(*found, number);
        let key = crate::keys::encode_own(&crate::keys::Own::Wires, store.limits.store.key).unwrap();
        let bytes = crate::open::get(&mut store, Token::new(77), key).unwrap().unwrap();
        assert_eq!(
            crate::rows::kept_wires(&bytes).unwrap().as_ref(),
            &[crate::KeptWire { name: "rehearsal-wire".into(), version: 2 }]
        );
        assert_eq!(disk.contents(), Disk::new(&before).contents(), "source active, maps and transcripts unchanged");
        let closed = disk.request(&mut store, Request::Close { owner: Token::new(1) });
        let [Event::Closed { removed: true, .. }] = closed.as_ref() else { panic!("{closed:?}") };
        assert_eq!(disk.contents(), Disk::new(&before).contents());
    }
}

#[test]
fn rehearsal_refuses_transcript_writes_and_drops_after_the_switch() {
    for pending in [true, false] {
        for write in [
            Write::DropTranscript { task: 1 },
            fixtures::save(jig_domain_core_vocabulary::Record::Core(jig_domain_core_vocabulary::CoreRecord::Turn(
                fixtures::turn(1, 7, 1, b"one"),
            ))),
        ] {
            let (mut disk, mut store, _, before) = opened(pending);
            let number = store.stores.store.last().checked_add(1).unwrap();
            let requests = disk.requests().len();
            let events = disk.request(&mut store, fixtures::commit(number, Box::new([write])));
            let [Event::Failed { why: crate::CommitFailure::Check(crate::CommitCheck::Switching), .. }] =
                events.as_ref()
            else {
                panic!("{events:?}")
            };
            assert_eq!(disk.requests().len(), requests, "refused before any file request");
            assert_eq!(disk.contents(), Disk::new(&before).contents());
            let closed = disk.request(&mut store, Request::Close { owner: Token::new(1) });
            let [Event::Closed { removed: true, .. }] = closed.as_ref() else { panic!("{closed:?}") };
        }
    }
}
