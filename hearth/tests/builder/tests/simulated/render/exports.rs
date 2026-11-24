//! Exports as printed by render over the protocol world's cut tool pipes.
use super::{Form, fixtures, protocol, render};

fn render_x1(files: &[fixtures::File], form: Form) -> protocol::Run {
    render(files, &[b"x1-prod"], form)
}

#[test]
fn hearth_render_prints_the_exports_a_target_publishes() {
    let run = render_x1(&fixtures::thin_with(fixtures::EXPORTS), Form::Text);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["  vpn."]),
        "  vpn.address    \"10.0.0.2\"\n  vpn.endpoint   \"198.51.100.7\"\n"
    );
    assert_eq!(run.code, Some(0));
}

#[test]
fn an_optional_export_with_no_setting_prints_no_line() {
    let run = render_x1(&fixtures::optional_export(), Form::Text);
    assert_eq!(fixtures::lines_starting(&run.stdout, &["  vpn."]), "  vpn.address   \"10.0.0.2\"\n");
}

#[test]
fn json_prints_the_exports_part_nested_under_its_task() {
    let run = render_x1(&fixtures::thin_with(fixtures::EXPORTS), Form::JsonLines);
    assert!(
        skein_lib::bytes::find(
            &run.stdout,
            b"\"exports\":{\"vpn\":{\"address\":\"10.0.0.2\",\"endpoint\":\"198.51.100.7\"}}"
        )
        .is_some()
    );
}

#[test]
fn an_export_nested_29_lists_deep_prints_as_json() {
    let run = render_x1(&fixtures::deep(fixtures::DEEP_EXPORT), Form::JsonLines);
    assert!(
        skein_lib::bytes::find(&run.stdout, fixtures::DEEP_NEST_JSON).is_some(),
        "{}",
        String::from_utf8_lossy(&run.stdout)
    );
}

fn export_error(value: &str) -> String {
    let run = render_x1(&fixtures::export_value(value), Form::Text);
    fixtures::lines_starting(&run.stdout, &["tasks/vpn/"])
}

#[test]
fn an_export_of_the_wrong_type_prints_at_its_value() {
    assert_eq!(export_error("{{ 7 }}"), "tasks/vpn/task.json:2:53: x1-prod: vpn: not a string\n");
}

#[test]
fn a_value_with_text_around_its_tag_prints_once_without_a_target() {
    assert_eq!(
        export_error("v {{ settings.address }}"),
        "tasks/vpn/task.json:2:53: vpn: an export's value must be one {{ }} tag\n"
    );
}

#[test]
fn an_export_naming_an_undeclared_setting_prints_at_its_name() {
    assert_eq!(
        export_error("{{ settings.adress }}"),
        "tasks/vpn/task.json:2:53: vpn: undeclared setting \"adress\"; vpn declares address, endpoint\n"
    );
}

#[test]
fn an_export_reading_tasks_is_refused() {
    assert_eq!(
        export_error("{{ tasks.core.domain }}"),
        "tasks/vpn/task.json:2:53: vpn: an export reads settings and target alone, not tasks\n"
    );
}

#[test]
fn a_reader_sees_the_exports_of_its_own_environment_alone() {
    let run = render(&fixtures::thin_with(fixtures::VPN), &[b"carbon-lab", b"carbon-prod"], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["  /etc/vpn/peers"]),
        concat!(
            "  /etc/vpn/peers           file 0644 root:root ce8ea3ad3272\n",
            "  /etc/vpn/peers           file 0644 root:root 46c9fc904003\n",
        )
    );
    assert_eq!(run.code, Some(0));
}

#[test]
fn an_exporters_fault_prints_for_it_and_for_its_reader() {
    let files = fixtures::thin_with(&[fixtures::VPN, &[fixtures::VPN_BAD]].concat());
    let run = render(&files, &[b"carbon-lab", b"carbon-prod", b"west-lab", b"west-prod"], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["hosts/west/west-prod.json:"]),
        concat!(
            "hosts/west/west-prod.json:1:97: carbon-prod: vpn: exports.vpn of west-prod: not a string\n",
            "hosts/west/west-prod.json:1:97: west-prod: vpn: not a string\n",
        )
    );
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["target       "]),
        concat!(
            "target       carbon-lab   base carbon   environment lab\n",
            "target       west-lab   base west   environment lab\n",
        )
    );
    assert_eq!(run.code, Some(2));
}
