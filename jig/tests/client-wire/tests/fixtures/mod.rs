//! Complete scripted values and one shared byte-round-trip assertion.
use jig_client_domain::contract::{ProposalKey, Proposer, Revision};
use jig_client_test_domain::{connector_values::ConnectorLimits, family};

pub const PROJECT: u32 = 1;
pub const ITEM: u64 = 7;
pub const TASK: u64 = 9;
pub const REVISION: Revision = Revision { value: 3 };
pub const PARTY: u64 = 7;
pub const PROPOSAL_NUMBER: u64 = 1;
pub const PROPOSAL: ProposalKey = ProposalKey { proposer: Proposer::Party(PARTY), number: PROPOSAL_NUMBER };
pub const SUBJECT_KINDS: [family::Kind; 10] = [
    family::Kind::Item(ITEM),
    family::Kind::Panel(TASK),
    family::Kind::Result(TASK),
    family::Kind::Effect(PROPOSAL),
    family::Kind::Effect(ProposalKey { proposer: Proposer::Task(TASK), number: PROPOSAL_NUMBER }),
    family::Kind::OwnerEffect(PROPOSAL_NUMBER),
    family::Kind::Outbox(PROPOSAL_NUMBER),
    family::Kind::Hold(TASK),
    family::Kind::Project(PROJECT),
    family::Kind::Deployment,
];

pub fn assert_document(value: family::Document) {
    let bytes = family::wire::encode_document(&value);
    let bounds = jig_core_world::world::limits().connector;
    let limits = ConnectorLimits { rows: 2, paths: 2, segments: 2, segment_bytes: 32, bytes: 65536 };
    assert_eq!(family::wire::decode_document(&bytes, &bounds, &limits), Ok(value));
}
pub fn assert_answer(value: family::Answer) {
    let bytes = family::wire::encode_answer(&value).expect("application answer");
    let bounds = jig_core_world::world::limits().connector;
    assert_eq!(family::wire::decode_answer(&bytes, &bounds), Ok(value));
}
pub fn assert_subject(subject: family::Subject) {
    use jig_client_test_protocol_wire::codec;
    let (_, value) = family::wire::subject(subject);
    let bounds = jig_core_world::world::limits().connector;
    let bytes = codec::family::subject(value.clone());
    let decoded = codec::family::subject_value(&bytes, &bounds).expect("family subject");
    assert_eq!(family::wire::subject_value(decoded), subject, "subject: {subject:?}");
    let bytes = codec::family::read(value);
    let decoded = codec::family::read_value(&bytes, &bounds).expect("family read");
    assert_eq!(family::wire::subject_value(decoded), subject, "read: {subject:?}");
}
pub fn assert_change(supplied: u64) {
    use jig_client_test_protocol_wire::codec;
    let ask = family::Ask::ChangeObject { project: PROJECT, object: ITEM, revision: REVISION, value: supplied };
    let (project, value) = family::wire::request(&ask);
    let bytes = codec::family::request_record(project, value);
    let bounds = jig_core_world::world::limits().connector;
    let decoded = codec::family::request_value(&bytes, &bounds).expect("family request");
    let decoded_value = decoded.value.into_parts();
    assert_eq!(decoded.project, PROJECT);
    assert_eq!(
        (decoded_value.object, decoded_value.revision, decoded_value.value),
        (ITEM, REVISION.value, Some(supplied))
    );
    assert_eq!(decoded_value.release, None);
}
