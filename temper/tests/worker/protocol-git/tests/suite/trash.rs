use std::collections::BTreeMap;

use skein_io::kernel::{self, Error, Op};
use skein_lib::{Time, Token};
use skein_world::{
    inspect::{Moment, Release, check_tables},
    streams::Streams,
};
use temper_worker_inspect::{inspect_trash, inspection};
use temper_worker_protocol_git::{Component, Request};
use temper_worker_protocol_git_world::{
    fixtures,
    io::{Below, Input, Node, Script},
    referee::Verdict,
    world::World,
};

fn remove(below: &mut Below, name: &[u8], directory: bool) -> Result<kernel::Done, Error> {
    below.out.kernel.push(kernel::Submit {
        op: Token::new(1),
        kind: Op::Remove { dir: kernel::Fd::new(3), name: name.into(), directory },
    });
    below.drain();
    let Some(Input::Kernel(complete)) = below.next(&mut 0) else { panic!("removal completion") };
    complete.result
}

#[test]
fn a_stuck_path_refuses_every_removal_but_not_its_neighbours() {
    for node in [Node::File(Box::from(&b"kept"[..])), Node::Directory] {
        let mut below = Below::new(Script::default());
        below.tree.insert(b"stuck".to_vec(), node);
        below.remove_failure = Some((b"stuck".to_vec(), Error::Permission));
        for _ in 0..2 {
            for directory in [false, true] {
                assert_eq!(remove(&mut below, b"stuck", directory), Err(Error::Permission));
            }
            below.reset(Script::default());
        }
        assert!(below.tree.contains_key(b"stuck".as_slice()));
        below.tree.insert(b"file".to_vec(), Node::File(Box::new([])));
        below.tree.insert(b"directory".to_vec(), Node::Directory);
        assert_eq!(remove(&mut below, b"file", false), Ok(kernel::Done::Nothing));
        assert_eq!(remove(&mut below, b"directory", true), Ok(kernel::Done::Nothing));
        assert!(!below.tree.contains_key(b"file".as_slice()));
        assert!(!below.tree.contains_key(b"directory".as_slice()));
    }
}

#[test]
fn trash_is_due_after_a_successful_pass_and_pending_after_a_refused_removal() {
    let mut world = World::new(fixtures::SEED, BTreeMap::new());
    let make = fixtures::operations().remove(0).1;
    assert_eq!(world.run(&make).expect("inspection").verdict, Verdict::Passed);
    assert_eq!(world.component.sweep_state(), (false, 0, false, Some(1004)));
    let mut inspected = inspection();
    inspect_trash(&world.component, &[b"17"], &mut inspected);
    let table = &inspected.tables()[0];
    assert_eq!(table.name, "trash");
    assert_eq!(table.capacity, u64::MAX);
    assert_eq!(table.entries[0].release, Release::Due);
    let moment = Moment { seed: fixtures::SEED, at: Time::ZERO, iteration: 1 };
    let finding = check_tables(&[&inspected], moment).expect_err("leftover entry is due");
    assert!(finding.contains("trash/17"), "{finding}");
    assert!(finding.contains("the sweep"), "{finding}");

    world.component.down(
        Time::ZERO,
        Request::Io { owner: fixtures::OWNER, op: make.op, deadline: fixtures::DEADLINE, identity: None },
        &mut world.below.out,
    );
    for _ in 0..4 {
        world.below.drain();
        let Some(Input::Kernel(complete)) = world.below.next(&mut 0) else { panic!("Make completion") };
        world.component.complete(Time::ZERO, complete, &mut world.below.out);
    }
    assert_eq!(world.component.sweep_state(), (true, 0, false, Some(1004)));
    let mut running = inspection();
    inspect_trash(&world.component, &[b"17"], &mut running);
    assert_eq!(running.tables()[0].entries[0].release, Release::Pending);
    check_tables(&[&running], moment).expect("the running pass defers the entry");
    world.shutdown();

    let mut world = World::with_trash_faults(Streams::new(fixtures::SEED));
    assert_eq!(world.run(&fixtures::trash_make()).expect("inspection").verdict, Verdict::Passed);
    assert_eq!(world.component.sweep_state(), (true, 2, true, Some(1004)));
    let mut inspected = inspection();
    let names = world.below.trash();
    let names: Vec<_> = names.iter().map(Vec::as_slice).collect();
    inspect_trash(&world.component, &names, &mut inspected);
    assert_ne!(inspected.tables()[0].entries.len(), 0);
    assert!(inspected.tables()[0].entries.iter().all(|entry| entry.release == Release::Pending));
    check_tables(&[&inspected], moment).expect("refused removal defers the entries");
    world.shutdown();
}

#[test]
fn trash_created_after_the_last_pass_waits_for_the_next_pass() {
    let component =
        Component::new(&fixtures::limits(), &fixtures::checkout(), fixtures::settings()).expect("valid limits");
    let mut before_start = inspection();
    inspect_trash(&component, &[b"0"], &mut before_start);
    let moment = Moment { seed: fixtures::SEED, at: Time::ZERO, iteration: 1 };
    assert_eq!(before_start.tables()[0].entries[0].release, Release::Pending);
    check_tables(&[&before_start], moment).expect("no pass has ended");

    let mut world = World::new(fixtures::SEED, BTreeMap::new());
    let ended_serial = world.component.sweep_state().3.expect("startup pass ended");
    let mut store = fixtures::operations().into_iter().find(|(name, _)| *name == "store").expect("Store").1;
    let temper_worker_domain_checkout::git::Op::Store { aside, .. } = &mut store.op else { panic!("Store fixture") };
    *aside = Some(Token::new(8));
    store.files.insert(b"stores/8".to_vec(), Node::Directory);
    store.files.insert(b"stores/8/old".to_vec(), Node::File(b"old store".as_slice().into()));
    store.files.insert(b"stores/9".to_vec(), Node::Directory);
    assert_eq!(world.run(&store).expect("fresh trash is pending").verdict, Verdict::Passed);
    assert_eq!(world.component.sweep_state(), (false, 0, false, Some(ended_serial)));
    assert!(world.below.tree.contains_key(format!("trash/{ended_serial}/old").as_bytes()));

    let old = (ended_serial - 1).to_string();
    let fresh = ended_serial.to_string();
    let after = (ended_serial + 1).to_string();
    let mut inspected = inspection();
    inspect_trash(&world.component, &[old.as_bytes(), fresh.as_bytes(), after.as_bytes(), b"unknown"], &mut inspected);
    let entries = &inspected.tables()[0].entries;
    assert_eq!(entries[0].release, Release::Due, "pre-pass leftovers are due");
    assert!(entries[1..].iter().all(|entry| entry.release == Release::Pending));
    world.check_trash().expect("the world's fresh entry passes");
    world.shutdown();
}

#[test]
fn a_stuck_workspace_is_counted_and_later_operations_end_as_before() {
    let mut stuck = World::with_trash_faults(Streams::new(fixtures::SEED));
    let mut ordinary = World::new(fixtures::SEED, BTreeMap::new());
    let scenarios = [fixtures::trash_make(), fixtures::point(), fixtures::fetch(), fixtures::operations().remove(0).1];
    for scenario in scenarios {
        let actual = stuck.run(&scenario).expect("stuck trash inspection");
        let expected = ordinary.run(&scenario).expect("ordinary trash inspection");
        assert_eq!(actual.verdict, Verdict::Passed);
        assert_eq!(actual.terminals, expected.terminals);
        assert_eq!(actual.spawns, expected.spawns);
        let (path, error) = stuck.below.remove_failure.as_ref().expect("file marked after Make's rename");
        assert_eq!(*error, Error::Permission);
        assert!(path.starts_with(b"trash/"));
        assert!(matches!(stuck.below.tree.get(path), Some(Node::File(bytes)) if bytes.as_ref() == b"payload"));
    }
    let (pending, failures, refused, ended_serial) = stuck.component.sweep_state();
    assert_eq!((pending, failures, refused), (true, 4, true));
    assert!(ended_serial >= Some(1008), "the sweep can move nested directories into new trash names");
    assert_eq!(stuck.ledger.entries().len(), 1);
    assert_eq!(stuck.ledger.entries()[0].until, None);
    ordinary.shutdown();
    stuck.shutdown();
}
