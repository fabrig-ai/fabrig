//! The six seed17 alert cut positions share one baseline and the full story assertions.
use crate::stories::assert_alert;
use jig_conformance::Cut;

#[test]
fn the_alert_storm_survives_its_six_commit_cuts() {
    let world = jig_ops_world::conformance::alert(17, Cut::None);
    assert_alert(&world);
    let acceptance = world.peers.accepted[0];
    let entry = world
        .peers
        .entries
        .iter()
        .filter_map(|(commit, kind)| (*kind == 1).then_some(*commit))
        .min()
        .expect("restart entry");
    let lead_commit = world.peers.lead_commits.first().expect("same-decision incident lead receipt").0;
    for (name, commit) in [
        ("before the lead receipt", lead_commit - 1),
        ("at the lead receipt", lead_commit),
        ("after the lead receipt", lead_commit + 1),
        ("at the person's acceptance", acceptance),
        ("at the restart's entry", entry),
        ("after the restart's entry", entry + 1),
    ] {
        println!("{name}: cut after commit {commit}");
        let recovered = jig_ops_world::conformance::alert(17, Cut::AfterCommit(commit));
        assert_eq!(recovered.outcome().crashes, vec![commit]);
        assert_alert(&recovered);
    }
}
