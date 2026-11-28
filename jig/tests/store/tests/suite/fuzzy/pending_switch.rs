use crate::pending_switch::{pending, switch_cuts};
use jig_store_world::upgrade_fixtures;

#[test]
fn each_cut_around_the_switch_leaves_one_generation_whole() {
    let mut completed = pending();
    let first = completed.operation_count();
    completed.switch();
    let cuts: Vec<_> = upgrade_fixtures::cuts(&completed).into_iter().filter(|cut| *cut >= first).collect();
    assert_ne!(cuts.len(), 0);
    switch_cuts(cuts);
}
