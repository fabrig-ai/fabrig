use crate::route;
use crate::route::Invalid;
use jig_domain_host as host;
use skein_lib::Writer;
use smith_host_domain as agent;

#[test]
fn delivery_evidence_disagreeing_with_its_outcome_is_invalid() {
    let record = smith_channel::DeliveryReply::new(
        &smith_channel::CEILINGS,
        smith_channel::DeliveryReplyParts { value: smith_channel::Delivery::Nothing },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(record.measure()).unwrap());
    record.encode(&mut writer).unwrap();
    let bytes = writer.finish();
    // DeliveryReply is unversioned and unchanged by smith's RunResult 3 -> 4.
    assert_eq!(bytes.as_ref(), &[1], "delivery evidence golden");
    for (row, outcome, evidence, expected) in [
        ("agreeing", host::DeliveryOutcome::Nothing, bytes.as_ref(), Ok(agent::Delivery::Nothing)),
        ("disagreeing", host::DeliveryOutcome::Delivered, bytes.as_ref(), Err(Invalid::Evidence)),
        ("malformed", host::DeliveryOutcome::Nothing, &[255, 0, 1], Err(Invalid::Evidence)),
    ] {
        assert_eq!(route::evidence(outcome, evidence), expected, "{row}");
    }
    let mut writer = Writer::new(bytes.len() + 1);
    writer.put(&bytes).unwrap();
    writer.put(&[0]).unwrap();
    let trailing = writer.finish();
    assert_eq!(route::evidence(host::DeliveryOutcome::Nothing, &trailing), Err(Invalid::Evidence), "trailing");
}
