use hearth::Form;
use hearth_builder_world::{fixtures, protocol};

fn vpn_diff(change: fixtures::File, form: Form) -> protocol::Run {
    let before = fixtures::thin_with(fixtures::VPN);
    let later = fixtures::edited(&before, &[change], &[]);
    let scenario = fixtures::Scenario { form, ..fixtures::diffing(&before, &later) };
    protocol::run(&scenario, fixtures::SIMULATION_SEED)
}

#[test]
fn a_changed_export_rebuilds_its_readers_and_builds_nothing_where_it_is_made() {
    let run = vpn_diff(fixtures::VPN_MOVED, Form::Text);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["carbon-prod", "west-prod", "  exports "]),
        concat!(
            "carbon-prod                                         rebuild from config\n",
            "west-prod                                           nothing to build\n",
            "  exports  ~ vpn.endpoint   \"203.0.113.157\" → \"203.0.113.158\"\n",
        )
    );
    assert_eq!(fixtures::lines_starting(&run.stdout, &["carbon-lab", "west-lab"]), "");
    assert_eq!(run.code, Some(1));
}

#[test]
fn a_setting_no_export_reads_changes_its_target_alone() {
    let run = vpn_diff(fixtures::VPN_MTU, Form::Text);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["carbon-prod", "west-prod", "carbon-lab", "west-lab"]),
        "west-prod                                           rebuild from config\n"
    );
}
#[test]
fn json_gives_an_exports_change_with_one_side_absent() {
    let run = vpn_diff(fixtures::VPN_NO_ENDPOINT, Form::JsonLines);
    assert!(
        skein_lib::bytes::find(
            &run.stdout,
            br#"{"kind":"exports","exports":[{"before":"203.0.113.157","name":"endpoint","task":"vpn"}]}"#,
        )
        .is_some()
    );
}
