use jig_store_world::{World, patches, referee::Referee, root, world::Change};
use skein_fake_machine::Item;

fn whole(world: &mut World) {
    assert_eq!(world.active(), b"1\n");
    let paths = world.paths();
    let generations: Vec<_> =
        paths.iter().filter(|p| p.starts_with(b"generations/") && skein_lib::bytes::count(p, b"/", 2) == 1).collect();
    assert_eq!(generations, [b"generations/1".as_slice()]);
    assert!(!paths.iter().any(|p| p.starts_with(b"active.skein-")));
}
pub(crate) fn a_first_start_cut_is_whole(mut world: World, cut: u32, seed: u64) {
    world.cut_at(cut);
    world.open_switched();
    assert_eq!(world.operation_count(), cut, "first-start cut reached");
    world.crash(seed);
    let restored = world.reopen();
    assert_eq!(restored.last, 0, "empty recovered generation, cut {cut}");
    whole(&mut world);
    world.crash(seed);
    assert_eq!(world.reopen().last, 0, "second restart after cut {cut}");
    whole(&mut world);
    assert_eq!(world.commit(root::first()), Some(1));
    assert_eq!(world.commit(root::plain_commit(2)), Some(2));
    world.crash(seed);
    let restored = world.reopen();
    Referee::check(&world.model, &restored).unwrap_or_else(|e| panic!("first-start cut {cut}: {e:?}"));
}

#[test]
fn a_first_start_cut_at_its_first_directory_and_around_its_switch_is_whole_at_the_next_start() {
    let mut completed = World::unopened(18);
    completed.trace_writes();
    assert_eq!(completed.open_switched(), Some(0));
    let first_directory = patches::first_change(&completed, Change::Make);
    let rename = patches::last_change(&completed, Change::Rename);
    let switch_sync = completed
        .changes()
        .iter()
        .find(|(operation, change)| *operation > rename && *change == Change::Sync)
        .expect("switch sync reached")
        .0;
    for (why, cut) in [("first directory", first_directory), ("active rename", rename), ("active sync", switch_sync)] {
        eprintln!("{why}: {cut}");
        let mut world = World::unopened(u64::from(cut));
        world.trace_writes();
        a_first_start_cut_is_whole(world, cut, u64::from(cut));
    }
}
fn strays(world: &mut World) {
    world.lay(&[
        Item::directory(b"generations/2"),
        Item::directory(b"generations/2/store"),
        Item::file(b"generations/2/store/row", b"stray"),
        Item::directory(b"generations/2/secrets").mode(0o700),
        Item::file(b"generations/2/secrets/key", b"private stray"),
        Item::directory(b"generations/dropped"),
        Item::directory(b"generations/dropped/store"),
        Item::file(b"generations/dropped/store/row", b"stray"),
        Item::file(b"active.skein-0123456789abcdef", b"temporary"),
    ]);
}
pub(crate) fn committed(seed: u64) -> World {
    let mut world = World::new(seed);
    assert_eq!(world.commit(root::first()), Some(1));
    strays(&mut world);
    world.crash(seed);
    world
}
pub(crate) fn a_stray_cut_is_gone(mut world: World, cut: u32, seed: u64) {
    world.cut_at(cut);
    world.open_switched();
    assert_eq!(world.operation_count(), cut, "stray-removal cut reached");
    world.crash(seed);
    let restored = world.reopen();
    whole(&mut world);
    Referee::check(&world.model, &restored).unwrap_or_else(|e| panic!("stray-removal cut {cut}: {e:?}"));
    world.crash(seed);
    let restored = world.reopen();
    whole(&mut world);
    Referee::check(&world.model, &restored).unwrap_or_else(|e| panic!("second stray-removal restart {cut}: {e:?}"));
}

#[test]
fn strays_cut_at_their_first_and_last_removal_are_gone_at_the_next_start() {
    let mut completed = committed(19);
    completed.trace_writes();
    completed.reopen();
    let first_removal = patches::first_change(&completed, Change::Make);
    let last_removal = patches::last_change(&completed, Change::Remove);
    for (why, cut) in [("first stray removal", first_removal), ("last stray removal", last_removal)] {
        eprintln!("{why}: {cut}");
        let mut world = committed(u64::from(cut));
        world.trace_writes();
        a_stray_cut_is_gone(world, cut, u64::from(cut));
    }
}
