//! Shared fixture values and durable rows (temper/docs/design/testing/checks.md, 1).
use super::*;

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "the complete parsed configuration is compared without adding equality to product types"
)]
fn engine_copies_match_the_parsed_fixture_and_are_independent() {
    let expected = parse_engine(ENGINE);
    let mut first = engine();
    assert_eq!(format!("{:?}", first.jig), format!("{expected:?}"), "cached engine matches the parsed fixture");
    first.jig.projects[0].name = "changed".into();
    first.jig.projects[0].policy.period_spend = 0;
    first.jig.limits.core.tasks.tasks = 0;
    let second = engine();
    assert_eq!(format!("{:?}", second.jig), format!("{expected:?}"), "each fixture owns its configuration");
}

#[test]
fn json_edits_keep_unedited_bytes_and_use_original_spans_in_any_order() {
    let input = br#" { "text": "a\"b", "items": [{"n":7}, {"n":8}], "keep": [true, null] } "#;
    let json = Json::new(input);
    assert_eq!(json.value(b"items[1].n"), b"8");
    assert_eq!(json.value(b"text"), br#""a\"b""#);
    assert_eq!(
        json.with(&[(b"items[1].n", b"99"), (b"text", br#""longer""#)]).as_ref(),
        br#" { "text": "longer", "items": [{"n":7}, {"n":99}], "keep": [true, null] } "#
    );
}

#[test]
fn configured_service_keeps_the_supplied_records_store() {
    let mut store = records_store();
    commit(
        &mut store,
        1,
        Box::new([temper_engine_domain::Write::Save(temper_engine_domain::Record::Forge {
            row: Box::new(landed_row()),
        })]),
    );
    assert!(store.host.above.pop().is_some());
    let mut config = configured_engine();
    fill_files(&mut config);
    let mut service = configured_service_with_store(config, &seeds(), None, forge_runtime(), Some(store)).unwrap();
    let rows = load(&mut service.engine.as_mut().expect("engine").store, temper_engine_domain::Range::Forge);
    let mut kept = false;
    for row in rows {
        if let temper_engine_domain::Record::Forge { row } = row
            && *row == landed_row()
        {
            kept = true;
        }
    }
    assert!(kept, "the supplied durable rows survive configuration");
}
