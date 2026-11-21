//! Record defaults over values (redeploy.md, 5).
use serde_json::Value;
use temper_redeploy_tool::record::Record;

#[test]
fn records_before_checkpoints_read_the_new_fields_as_null() {
    let mut value =
        serde_json::to_value(Record::new("abcdef012345", temper_redeploy_tool::redeploy::Mode::Engine)).unwrap();
    value.as_object_mut().unwrap().remove("checkpoint").unwrap();
    value.as_object_mut().unwrap().remove("verdict").unwrap();
    let record: Record = serde_json::from_value(value).unwrap();
    assert_eq!(record.checkpoint, None);
    assert_eq!(record.verdict, None);
    let value = serde_json::to_value(record).unwrap();
    assert_eq!(value["checkpoint"], Value::Null);
    assert_eq!(value["verdict"], Value::Null);
}
