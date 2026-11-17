use super::fixtures;
use crate::What;
fn files() -> Box<[crate::File]> {
    crate::files(&fixtures::engine(fixtures::ENGINE_SMALL).unwrap())
}
fn a_workers_listener_on_a_public_address_is_refused() {
    fixtures::refuses(
        br#"{"in_process": "workers"}"#,
        br#"{"socket": "192.0.2.1:7100"}"#,
        fixtures::path(&[b"hosts", b"listener"]).child(crate::Segment::Index(0)),
        What::Plaintext,
    );
}
fn workers_in_the_document_are_refused_as_settings() {
    fixtures::refuses(
        br#""hosts": {"#,
        br#""hosts": {"workers": [],"#,
        fixtures::path(&[b"hosts", b"workers"]),
        What::Because(b"workers are settings in the store"),
    );
}
fn link_limits_hold_every_worker_setting() {
    fixtures::refuses(
        br#""profile": "standard""#,
        br#""profile": "standard", "overrides": {"link.names": 7}"#,
        fixtures::path(&[b"limits", b"overrides", b"link.names"]),
        What::Capacity { member: b"fleet.configured_workers", configured: 64, requires: b"link.names", required: 64 },
    );
    fixtures::refuses(
        br#""profile": "standard""#,
        br#""profile": "standard", "overrides": {"link.name": 31}"#,
        fixtures::path(&[b"limits", b"overrides", b"link.name"]),
        What::Capacity { member: b"fleet.worker_name_bytes", configured: 32, requires: b"link.name", required: 32 },
    );
}
fn an_engine_member_under_hosts_is_refused_as_unknown() {
    fixtures::refuses(
        br#""hosts": {"#,
        br#""hosts": {"engine": {},"#,
        fixtures::path(&[b"hosts", b"engine"]),
        What::Unknown,
    );
}
#[test]
fn files_list_no_worker_digest() {
    for file in files() {
        assert!(!file.path.ends_with(b".digest"));
    }
}
#[test]
fn hosts_configuration_problems_name_the_invalid_fields() {
    table([
        ("a_workers_listener_on_a_public_address_is_refused", a_workers_listener_on_a_public_address_is_refused),
        ("workers_in_the_document_are_refused_as_settings", workers_in_the_document_are_refused_as_settings),
        ("link_limits_hold_every_worker_setting", link_limits_hold_every_worker_setting),
        ("an_engine_member_under_hosts_is_refused_as_unknown", an_engine_member_under_hosts_is_refused_as_unknown),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    skein_rows::check(cases, |case| case());
}
