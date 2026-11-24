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

#[test]
fn a_stored_run_proof_without_a_worker_reads_with_none() {
    const ADDED_TAIL: usize = 1 + 4;
    let text = super::values::CORE;
    let (before, worker) = text.split_once(" worker:").expect("worker declaration");
    let (_, after) = worker.split_once(" }").expect("end of proof");
    let old_text = format!("{before} }}{after}");
    assert_ne!(old_text, text, "the older proof has no worker");
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
    .expect("worker defaults to none");
    let limits = codec::limits(&jig_protocol_config_fixtures::small().core).expect("codec limits");
    let proof = core::RunProof::claimed(1, 2, 1, None, jig_domain_fleet::HostKind::Worker);
    let bytes = codec::core::encode(&core::CoreRecord::RunProof(proof), &limits).expect("proof encoded");
    let row = wire::Row::decode(&limits.core, &mut Reader::new(&bytes)).expect("wire row");
    let wire::Value::RunProof(record) = row.value else { panic!("run proof") };
    let bytes = record.encode(&limits.core).expect("wire record");
    // The earlier layout ends before the worker option and the later kept-part number.
    assert_eq!(&bytes[bytes.len() - ADDED_TAIL..], &[0; ADDED_TAIL]);
    let old_bytes = &bytes[..bytes.len() - ADDED_TAIL];
    let mut out = Buffer::with_capacity(256);
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    plan.convert(old_bytes, &mut out, &mut work).expect("stored proof converted");
    let read = wire::RunProof::decode(&limits.core, &mut Reader::new(out.as_slice())).expect("current proof");
    assert_eq!(read.worker, None);
    assert_eq!(read, record);
}

#[test]
fn a_stored_run_beginning_without_fresh_reads_with_none() {
    use jig_protocol_store::tasks_v13::{self as tasks, Record as _};
    let text = super::values::TASKS;
    let (before, fresh) = text.split_once(" fresh: option Fresh").expect("fresh field");
    let (_, after) = fresh.split_once(" }").expect("end of BeganLine");
    let old_text = format!("{before} }}{after}");
    assert_ne!(old_text, text, "the older BeganLine has no fresh field");
    let old = skein_codec::schema::parse(old_text.as_bytes()).expect("old schema");
    let new = skein_codec::schema::parse(text.as_bytes()).expect("current schema");
    let plan = skein_codec::layout::Plan::derive(
        &old,
        old.root(b"BeganLine", None).expect("old beginning"),
        &[],
        &new,
        new.root(b"BeganLine", None).expect("current beginning"),
        &[],
    )
    .expect("fresh defaults to none");
    let limits = codec::limits(&jig_protocol_config_fixtures::small().core).expect("codec limits");
    // Independent earlier layout: u64 attempt, Why::Ready's tag, resumed bool; no fresh option tag.
    let mut bytes = 3_u64.to_be_bytes().to_vec();
    bytes.extend([1, 0]);
    let mut out = Buffer::with_capacity(64);
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    plan.convert(&bytes, &mut out, &mut work).expect("stored beginning converted");
    let read = tasks::BeganLine::decode(&limits.tasks, &mut Reader::new(out.as_slice())).expect("current beginning");
    assert_eq!(read.attempt, 3);
    assert_eq!(read.why, tasks::Why::Ready);
    assert!(!read.resumed);
    assert_eq!(read.fresh, None);
}
