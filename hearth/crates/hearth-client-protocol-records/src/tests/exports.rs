use super::fixtures;
use crate::{
    form::{self, Form},
    limits,
    records::Record,
};
use hearth_domain_render_vocabulary as render;

#[test]
fn exports_print_as_task_dot_name_and_a_json_value() {
    assert!(skein_lib::bytes::find(&fixtures::printed_exports(Form::Text), b"  vpn.port   51820\n").is_some());
}
#[test]
fn json_nests_exports_under_their_task_and_name() {
    assert!(
        skein_lib::bytes::find(&fixtures::printed_exports(Form::JsonLines), b"\"exports\":{\"vpn\":{\"port\":51820}}")
            .is_some()
    );
}
#[test]
fn an_export_nested_32_lists_deep_prints_as_json() {
    let spec = fixtures::export_spec(fixtures::lists_around(fixtures::EXPORT_PRINT_DEPTH, b"x"));
    let printed = form::print(Form::JsonLines, &Record::Spec(spec), None, &limits::SHIPPED).expect("deep exports fit");
    let expected: &[&[u8]] =
        &[b"\"nest\":", &[b'['; fixtures::EXPORT_PRINT_BRACKETS], b"\"x\"", &[b']'; fixtures::EXPORT_PRINT_BRACKETS]];
    assert!(fixtures::has_run(&printed, expected));
    let spec = fixtures::export_spec(fixtures::lists_around(fixtures::EXPORT_PRINT_DEPTH, b"x"));
    let printed = form::print(Form::Text, &Record::Spec(spec), None, &limits::SHIPPED).expect("deep exports fit");
    let text: &[&[u8]] = &[
        b"  vpn.nest   ",
        &[b'['; fixtures::EXPORT_PRINT_BRACKETS],
        b"\"x\"",
        &[b']'; fixtures::EXPORT_PRINT_BRACKETS],
        b"\n",
    ];
    assert!(fixtures::has_run(&printed, text));
}

#[test]
fn an_export_change_32_deep_prints_in_both_forms() {
    use render::{
        diff::PartChange,
        parts::exports::{Change, ExportChange},
    };
    let record = fixtures::change_record(PartChange::Exports(Change {
        entries: Box::new([ExportChange {
            task: skein_lib::bytes::copy_of(b"core"),
            name: skein_lib::bytes::copy_of(b"nest"),
            before: Some(fixtures::lists_around(fixtures::EXPORT_PRINT_DEPTH, b"x")),
            after: None,
        }]),
    }));
    let brackets = fixtures::EXPORT_PRINT_BRACKETS;
    let text = fixtures::joined(&[
        b"west-prod                                           nothing to build\n  exports  - core.nest   ",
        &[b'['; fixtures::EXPORT_PRINT_BRACKETS],
        b"\"x\"",
        &[b']'; fixtures::EXPORT_PRINT_BRACKETS],
        b"\n",
    ]);
    let json = fixtures::joined(&[
        br#"{"record":"change","target":"west-prod","kind":"changed","changed":{"parts":[{"kind":"exports","exports":[{"before":"#,
        &[b'['; fixtures::EXPORT_PRINT_BRACKETS],
        b"\"x\"",
        &[b']'; fixtures::EXPORT_PRINT_BRACKETS],
        b",\"name\":\"nest\",\"task\":\"core\"}]}],\"rebuild\":{\"kind\":\"nothing\"}}}\n",
    ]);
    for (format, expected) in [(Form::Text, text), (Form::JsonLines, json)] {
        assert_eq!(
            form::print(format, &record, Some(record.kind()), &limits::SHIPPED).as_deref(),
            Ok(expected.as_ref()),
            "{format:?}, {brackets} lists"
        );
    }
}

use render::errors::{ExportFault, Root};
const FAULTS: &[(ExportFault, &[u8])] = &[
    (ExportFault::Form, b"an export's value must be one {{ }} tag\n"),
    (ExportFault::Root { root: Root::Tasks }, b"an export reads settings and target alone, not tasks\n"),
    (ExportFault::Root { root: Root::Exports }, b"an export reads settings and target alone, not exports\n"),
    (ExportFault::Root { root: Root::Debian }, b"an export reads settings and target alone, not debian\n"),
];
#[test]
fn an_exports_static_fault_names_its_form_or_root() {
    for (row, (fault, expected)) in FAULTS.iter().enumerate() {
        assert_eq!(
            fixtures::print_error(Form::Text, fixtures::export_error(*fault)).as_deref(),
            Ok(*expected),
            "row {row}"
        );
    }
}
const JSON_FAULTS: &[(ExportFault, &[u8])] = &[
    (ExportFault::Form, b"\"schema\":{\"kind\":\"export\",\"fault\":\"form\"}"),
    (
        ExportFault::Root { root: Root::Tasks },
        b"\"schema\":{\"kind\":\"export\",\"fault\":\"root\",\"root\":\"tasks\"}",
    ),
];
#[test]
fn json_names_an_exports_static_fault() {
    for (row, (fault, expected)) in JSON_FAULTS.iter().enumerate() {
        let printed = fixtures::print_error(Form::JsonLines, fixtures::export_error(*fault)).expect("error fits");
        assert!(skein_lib::bytes::find(&printed, expected).is_some(), "row {row}");
    }
}
