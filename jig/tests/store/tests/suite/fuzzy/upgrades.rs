use crate::upgrades::{check, frames, patching, rehearsal};
use jig_store_world::patches;

#[test]
fn drop_copy_and_switch_recover_after_every_mutation() {
    for link_error in [None, Some(skein_io::kernel::Error::Other(18))] {
        let mut trace = patching::second_generation();
        trace.refuse_links(link_error);
        trace.trace_writes();
        let expected_last = trace.open_switched().expect("traced second upgrade with task logs");
        patching::drop_copy_or_switch_cuts(trace.changes(), expected_last, link_error);
    }
}

#[test]
fn cuts_across_check_and_switch_recover_whole_maps() {
    let records = patches::Records::story();
    let mut complete = records.committed(check::SEED, patches::BASE);
    complete.owner_sets(patches::SETS);
    complete.trace_writes();
    complete.open_switched().expect("complete job upgrade to enumerate its cuts");
    let (first, after_switch, _) = check::seal_and_switch(&complete);
    let mut source_opened = false;
    let mut switched = false;
    // Include every check read: the check itself has no writes at which to cut.
    for source_active in check::check_cuts(&records, (first..=after_switch).collect()) {
        source_opened |= source_active;
        switched |= !source_active;
    }
    assert!(source_opened && switched, "cuts reached each whole generation");
}

#[test]
fn every_wires_commit_cut_keeps_back_open_or_refuses_as_committed() {
    let (first, sync) = rehearsal::wires_commit_cuts();
    let mut lost = false;
    let mut durable = false;
    let cuts: Vec<_> = (first..=sync + 1).collect();
    for (cut, outcome) in cuts.iter().zip(rehearsal::wires_cuts(&cuts)) {
        eprintln!("wires cut {cut}: {outcome:?}");
        lost |= outcome == rehearsal::WiresOutcome::Lost;
        durable |= outcome == rehearsal::WiresOutcome::Durable;
    }
    assert!(lost && durable, "sweep reached both a lost and a durable wires commit");
}

#[test]
fn every_file_cut_in_a_patched_read_preserves_all_files() {
    for read in frames::kinds() {
        frames::a_read_cut_preserves_files(read, |operations| (0..=operations).collect());
    }
}
