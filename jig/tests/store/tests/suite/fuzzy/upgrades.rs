use crate::upgrades::{check, frames, patching};
use jig_store_world::patches;

#[test]
fn drop_copy_and_switch_recover_after_every_mutation() {
    let mut trace = patching::second_generation();
    trace.trace_writes();
    let expected_last = trace.open_switched().expect("traced second upgrade");
    for &(operation, change) in trace.changes() {
        patching::a_drop_copy_or_switch_cut_recovers(operation, change, expected_last);
    }
}

#[test]
fn cuts_across_check_and_switch_recover_whole_maps() {
    let records = patches::Records::story();
    let mut complete = records.committed(check::SEED, patches::BASE);
    complete.patch_sets(patches::SETS);
    complete.trace_writes();
    complete.open_switched().expect("complete patch to enumerate its cuts");
    let (first, after_switch, _) = check::seal_and_switch(&complete);
    let mut source_opened = false;
    let mut switched = false;
    // Include every check read: the check itself has no writes at which to cut.
    for cut in first..=after_switch {
        let source_active = check::a_check_cut_recovers(&records, cut);
        source_opened |= source_active;
        switched |= !source_active;
    }
    assert!(source_opened && switched, "cuts reached each whole generation");
}

#[test]
fn every_file_cut_in_a_patched_read_preserves_all_files() {
    for read in frames::kinds() {
        frames::a_read_cut_preserves_files(read, |operations| (0..=operations).collect());
    }
}
