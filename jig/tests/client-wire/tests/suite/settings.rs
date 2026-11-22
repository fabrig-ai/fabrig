//! Populated settings decode with their content intact.
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{Reader, Writer};

#[test]
fn populated_charter_decodes_its_parts() {
    let parts = skein_lib::List::from_boxed(Box::new([
        w::CharterPart { on: 1, instructions: b"first part".as_slice().into() },
        w::CharterPart { on: 2050, instructions: "later part λ".as_bytes().into() },
    ]));
    let bytes = super::client_values::fixed_value("record_charter_smallest.bin");
    let mut input = Reader::new(bytes);
    let value = w::Charter::decode(&w::CEILINGS, &mut input).unwrap();
    assert!(input.is_empty());
    let mut populated = value;
    populated.parts = parts;
    let value = populated;
    let mut out = Writer::new(
        usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
    );
    value.write(&jig_client_protocol_wire::CEILINGS, &mut out).unwrap();
    let bytes = out.finish();
    let mut input = Reader::new(&bytes);
    let decoded = w::Charter::decode(&w::CEILINGS, &mut input).unwrap();
    assert!(input.is_empty());
    assert_eq!(decoded.parts.len(), 2);
    assert_eq!(decoded.parts.get(0).unwrap().on, 1);
    assert_eq!(decoded.parts.get(0).unwrap().instructions.as_ref(), b"first part");
    assert_eq!(decoded.parts.get(1).unwrap().on, 2050);
    assert_eq!(decoded.parts.get(1).unwrap().instructions.as_ref(), "later part λ".as_bytes());
}

#[test]
fn setting_range_fault_decodes_as_range() {
    let bytes = super::client_values::fixed_value("enum_setting_fault_range_smallest.bin");
    let mut input = Reader::new(bytes);
    let value = w::SettingFault::decode(&w::CEILINGS, &mut input).unwrap();
    assert!(input.is_empty());
    assert_eq!(value, w::SettingFault::Range);
}
