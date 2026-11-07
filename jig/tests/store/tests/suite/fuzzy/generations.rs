use crate::generations::{a_first_start_cut_is_whole, a_stray_cut_is_gone, committed};
use jig_store_world::World;

#[test]
fn a_first_start_cut_anywhere_is_whole_at_the_next_start() {
    let mut completed = World::unopened(18);
    assert_eq!(completed.open_switched(), Some(0));
    let operations = completed.operation_count();
    for cut in 0..=operations {
        a_first_start_cut_is_whole(World::unopened(u64::from(cut)), cut, u64::from(cut));
    }
}

#[test]
fn strays_cut_anywhere_are_gone_at_the_next_start() {
    let mut completed = committed(19);
    completed.reopen();
    let operations = completed.operation_count();
    for cut in 0..=operations {
        a_stray_cut_is_gone(committed(u64::from(cut)), cut, u64::from(cut));
    }
}
