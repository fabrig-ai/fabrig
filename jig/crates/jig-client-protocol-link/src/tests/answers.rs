use super::*;

#[test]
fn original_nonzero_recipient_survives_wire_decoding_and_contract_translation() {
    use skein_lib::{Duration, RequestLimits, Writer};
    let limits = Limits::new(
        jig_client_domain::Limits {
            owners: &[],
            watches: 4,
            handles: 8,
            turn_bytes: 32,
            ask_bytes: 16384,
            requests: RequestLimits {
                requests: 4,
                bytes: 65536,
                out: 16,
                first: Duration::from_nanos(10),
                most: Duration::from_nanos(80),
                margin: Duration::from_secs(1),
            },
            fields: 4,
            reads: 4,
            parts: 8,
            delegates: 2,
            document_bytes: 65536,
            records: 2,
            kept: 4,
            projects: 2,
            text: 32,
            turn_text: 32,
            rows: 2,
            segments: 4,
            backoff: jig_client_domain::Backoff { first: Duration::from_nanos(10), most: Duration::from_nanos(80) },
            authority: 8,
            accounts: jig_client_domain::AccountCounts { accounts: 8, sources: 8, sign_ins: 8 },
            heartbeat: Duration::from_nanos(100),
        },
        4,
    )
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
