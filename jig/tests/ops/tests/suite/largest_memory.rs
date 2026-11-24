//! Largest values at the ceilings, or at the largest clamp under the cap.

use super::codec_memory::family;
use jig_ops_client_protocol_wire::v1::Record;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        jig_ops_client_protocol_wire::SCHEMA,
        jig_ops_client_protocol_wire::v1,
        roots [
            ObservabilitySubjectRecord,
            InfrastructureAskRecord,
            InfrastructureAnswerRecord,
            InfrastructureSubjectRecord,
            ObservabilityDocumentRecord,
            InfrastructureDocumentRecord,
            InfrastructureReadAnswer,
            ObservabilityReadAnswer,
        ]
    );
}
