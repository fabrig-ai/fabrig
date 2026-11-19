//! Main endpoint proofs and their registered default (protocol/store/layouts.md, 6).
use jig_domain_core as core;
use jig_protocol_store::{
    codec,
    core_v6::{self as wire, Record},
};
use skein_lib::{Buffer, Reader};

#[test]
fn run_proofs_round_trip_with_and_without_their_main_endpoint() {
    let limits = codec::limits(&jig_protocol_config_fixtures::small().core).expect("codec limits");
    for endpoint in [Some(3), None] {
        let mut proof = core::RunProof::claimed(1, 2, 1, None, jig_domain_fleet::HostKind::Worker);
        proof.endpoint = endpoint;
        let row = core::CoreRecord::RunProof(proof);
        let bytes = codec::core::encode(&row, &limits).expect("proof encoded");
        assert_eq!(codec::core::decode(&bytes, &limits), Ok(row));
    }
}

#[test]
fn a_stored_run_proof_without_an_endpoint_reads_with_none() {
    const ENDPOINT: usize = 3 * 8;
    let text = super::values::CORE;
    let (before, endpoint) = text.split_once(" endpoint:").expect("endpoint declaration");
    let (_, after) = endpoint.split_once(" accounts:").expect("next field");
    let old_text = format!("{before} accounts:{after}");
    assert_ne!(old_text, text, "the older layout has no endpoint field");
    let old = skein_codec::schema::parse(old_text.as_bytes()).expect("old schema");
    let new = skein_codec::schema::parse(text.as_bytes()).expect("current schema");
    let plan = skein_codec::layout::Plan::derive(
        &old,
        old.root(b"RunProof", None).expect("old proof"),
        &[],
        &new,
        new.root(b"RunProof", None).expect("current proof"),
        &[],
    )
    .expect("endpoint defaults to none");
    let limits = codec::limits(&jig_protocol_config_fixtures::small().core).expect("codec limits");
    let proof = core::RunProof::claimed(1, 2, 1, None, jig_domain_fleet::HostKind::Worker);
    let bytes = codec::core::encode(&core::CoreRecord::RunProof(proof), &limits).expect("proof encoded");
    let row = wire::Row::decode(&limits.core, &mut Reader::new(&bytes)).expect("wire row");
    let wire::Value::RunProof(record) = row.value else { panic!("run proof") };
    let bytes = record.encode(&limits.core).expect("wire record");
    // Three u64 identities precede the absent option's tag. All later fields stay.
    assert_eq!(bytes[ENDPOINT], 0);
    let old_bytes = [&bytes[..ENDPOINT], &bytes[ENDPOINT + 1..]].concat();
    let mut out = Buffer::with_capacity(u32::try_from(bytes.len()).expect("record size"));
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    plan.convert(&old_bytes, &mut out, &mut work).expect("stored proof converted");
    let read = wire::RunProof::decode(&limits.core, &mut Reader::new(out.as_slice())).expect("current proof");
    assert_eq!(read, record);
    assert_eq!(read.endpoint, None);
}
