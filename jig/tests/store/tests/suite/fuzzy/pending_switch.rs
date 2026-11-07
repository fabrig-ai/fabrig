use crate::pending_switch::{a_switch_cut_leaves_one_generation_whole, pending};
use jig_store_world::patches;

#[test]
fn each_cut_around_the_switch_leaves_one_generation_whole() {
    let mut completed = pending();
    let first = completed.operation_count();
    completed.switch();
    let cuts: Vec<_> = patches::cuts(&completed).into_iter().filter(|cut| *cut >= first).collect();
    assert!(!cuts.is_empty());
    for cut in cuts {
        a_switch_cut_leaves_one_generation_whole(cut);
    }
}
