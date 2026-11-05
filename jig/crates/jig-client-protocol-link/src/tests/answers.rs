use super::*;

#[test]
fn original_nonzero_recipient_survives_wire_decoding_and_contract_translation() {
    use skein_lib::Writer;
    let limits = Limits::new(jig_client_domain::Limits { owners: &[], ..*fixtures::limits().client() }, 4)
        .expect("existing small client fixture bounds");
    for to in [7, 91, u32::MAX] {
        let original = w::DocumentOutcome::Proposed(
            w::OutcomeProposed::new(limits.wire(), w::OutcomeProposedParts { proposal: 31, to })
                .expect("recipient record"),
        );
        let mut out = Writer::new(skein_lib::length::usize_of(original.measure()));
        original.encode(&mut out).expect("measured room");
        let bytes = out.finish();
        let decoded = w::DocumentOutcome::decode(limits.wire(), &mut Reader::new(&bytes)).expect("real record decoder");
        assert_eq!(
            translate::answers::document_outcome(&decoded, &limits).expect("contract translation"),
            c::DocumentOutcome::Proposed { proposal: 31, to }
        );
    }
}
