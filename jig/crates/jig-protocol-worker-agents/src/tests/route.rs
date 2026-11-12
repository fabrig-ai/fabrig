use crate::route;
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
        ("disagreeing", host::DeliveryOutcome::Delivered, bytes.as_ref(), Err(host::Undecoded::Evidence)),
        ("malformed", host::DeliveryOutcome::Nothing, &[255, 0, 1], Err(host::Undecoded::Evidence)),
    ] {
        assert_eq!(route::evidence(outcome, evidence), expected, "{row}");
    }
    let mut writer = Writer::new(bytes.len() + 1);
    writer.put(&bytes).unwrap();
    writer.put(&[0]).unwrap();
    let trailing = writer.finish();
    assert_eq!(route::evidence(host::DeliveryOutcome::Nothing, &trailing), Err(host::Undecoded::Evidence), "trailing");
}

#[test]
fn a_change_at_its_body_bound_encodes_within_the_hubs_outcome() {
    use smith_domain_run::outcome::{self, Change, ChangeSpec, Declared, Field, FieldRule, OutcomeSpec};
    // Reserve 512 bytes for the title and framing, then fill each aggregate outcome.
    // These three rows measured 0.004782 s CPU, including the test harness.
    for (outcome_bytes, body_max) in [(4096, 3584), (10_662, 10_150), (131_072, 130_560)] {
        let title_max = jig_protocol_charter::CONTRACT_TEXT_MAX;
        let mut change = Change {
            fields: Box::new([
                Field { name: b"title".as_slice().into(), value: Box::new([]) },
                // The binary codec length-prefixes text; no byte requires escaping.
                Field {
                    name: b"body".as_slice().into(),
                    value: skein_lib::bytes::zeroed(usize::try_from(body_max).unwrap()),
                },
            ]),
        };
        let title_bytes = outcome_bytes - change.owned_bytes().unwrap();
        assert!(title_bytes <= u64::from(title_max));
        change.fields[0].value = skein_lib::bytes::zeroed(usize::try_from(title_bytes).unwrap());
        assert_eq!(change.owned_bytes(), Some(outcome_bytes));
        let spec = OutcomeSpec {
            change: Some(ChangeSpec {
                checks_must_pass: jig_protocol_charter::CONTRACT_CHECKS_MUST_PASS,
                fields: Box::new([
                    FieldRule { name: b"title".as_slice().into(), max: title_max },
                    FieldRule { name: b"body".as_slice().into(), max: body_max },
                ]),
            }),
            verdicts: Box::new([]),
            report: None,
            failure: None,
        };
        let declared = Declared::Change(change.clone());
        assert_eq!(outcome::judge(&spec, &declared), Ok(()));
        let accepted = agent::Declared::new(Declared::Change(change), outcome_bytes).unwrap();
        let encoded = route::encode_result(
            agent::RunResult::Accepted { outcome: accepted },
            &smith_host_protocol_fixtures::small(),
        )
        .unwrap();
        assert!(encoded.len() <= usize::try_from(outcome_bytes).unwrap(), "encoded bytes: {}", encoded.len());
        let decoded = smith_protocol_channel::decode_declared(&encoded, &jig_protocol_charter::limits()).unwrap();
        assert_eq!(decoded, declared);
        assert_eq!(outcome::owned_bytes(&decoded), Some(outcome_bytes));
        assert_eq!(outcome::judge(&spec, &decoded), Ok(()));
    }
}
