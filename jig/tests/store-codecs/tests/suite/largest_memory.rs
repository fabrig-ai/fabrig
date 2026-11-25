//! Largest values at the ceilings, or at the largest clamp under the cap.

use super::codec_memory::family;
use jig_protocol_store::tasks_v13::Record;

const CAP: u64 = 64 * 1024 * 1024;

#[test]
fn largest_worker_opening_stays_within_its_codec_bounds() {
    use jig_protocol_store::openings_v2 as wire;
    assert_eq!(wire::WorkerOpening::encoded_bound(&wire::CEILINGS), Some(525_530));
    family!(
        CAP,
        jig_protocol_store::STORE_FAMILIES.get(1).expect("store family").current,
        jig_protocol_store::openings_v2,
        roots [
            WorkerOpening,
        ]
    );
}

#[expect(clippy::too_many_lines, reason = "one dispatch per schema root")]
#[test]
fn largest_roots_stay_within_decoded_bounds() {
    family!(
        CAP,
        jig_protocol_store::STORE_FAMILIES.get(3).expect("store family").current,
        jig_protocol_store::core_v6,
        roots [
            Row,
        ]
    );
    family!(
        CAP,
        jig_protocol_store::STORE_FAMILIES.get(7).expect("store family").current,
        jig_protocol_store::accounts_v2,
        roots [
            Installation,
            Row,
        ]
    );
    family!(
        CAP,
        jig_protocol_store::STORE_FAMILIES.first().expect("store family").current,
        jig_protocol_store::component_v3,
        roots [
            Wires,
            Format,
            Manifest,
            Checkpoint,
            RestoreIntent,
            Transcript,
            Attempt,
            HistoryLog,
            TurnLog,
            LogAttempt,
            Floor,
        ]
    );
    family!(
        CAP,
        jig_protocol_store::STORE_FAMILIES.get(6).expect("store family").current,
        jig_protocol_store::notes_v1,
        roots [
            Row,
        ]
    );
    family!(
        CAP,
        jig_protocol_store::STORE_FAMILIES.get(2).expect("store family").current,
        jig_protocol_store::registry_v1,
        roots [
            Layout,
            Text,
            Job,
            Order,
        ]
    );
    family!(
        CAP,
        jig_protocol_store::STORE_FAMILIES.get(9).expect("store family").current,
        jig_protocol_store::secrets_v1,
        roots [
            Digest,
            Source,
            Carried,
            NextCarried,
        ]
    );
    family!(
        CAP,
        jig_protocol_store::STORE_FAMILIES.get(5).expect("store family").current,
        jig_protocol_store::people_v17,
        roots [
            Segment,
            ResourceScope,
            Executor,
            PermissionRole,
            Parameter,
            Dependency,
            FieldRule,
            Verdict,
            Row,
        ]
    );
    // QuestionAnswerer now reaches Number64 as an ordinary record, so it
    // is not a schema root. Keep checking its standalone decoder's bound.
    people_number64();
    family!(
        CAP,
        jig_protocol_store::STORE_FAMILIES.get(8).expect("store family").current,
        jig_protocol_store::settings_v3,
        roots [
            FieldRule,
            RevisionItem,
            ApplicationState,
            Default,
            Changed,
            Revision,
            Extent,
            CharterVersion,
            Proposal,
        ]
    );
    family!(
        CAP,
        jig_protocol_store::STORE_FAMILIES.get(4).expect("store family").current,
        jig_protocol_store::tasks_v13,
        roots [
            Parameter,
            PartSegment,
            FieldRule,
            Grant,
            ResourceScope,
            Segment,
            Creation,
            ResultField,
            Row,
            TaskProblem,
            EndRecord,
            PhaseRecord,
            ChangeRecord,
        ]
    );
}

fn people_number64() {
    use jig_protocol_store::people_v17 as codec;
    let schema = &skein_codegen::values(jig_protocol_store::STORE_FAMILIES[5].current).schema;
    let limits = codec::CEILINGS;
    let variants = skein_codegen::largest_variants(schema, "Number64", limits.values().as_slice())
        .expect("largest embedded Number64");
    let [bytes] = variants.as_slice() else { panic!("one Number64 encoding") };
    let length = u64::try_from(bytes.len()).expect("length fits");
    assert_eq!(codec::Number64::encoded_bound(&limits), Some(length));
    assert!(length <= CAP);
    let bound = codec::Number64::decoded_bound(&limits).expect("decoded bound");
    let mut reader = skein_lib::Reader::new(bytes);
    let meter = skein_heap::Meter::new();
    meter.start();
    let value = codec::Number64::decode(&limits, &mut reader);
    let measured = meter.end();
    let value = value.expect("embedded Number64 decodes");
    assert!(reader.is_empty());
    assert!(measured.peak() <= bound);
    assert_eq!(u64::from(value.measure(&limits).expect("measured")), length);
    assert_eq!(value.encode(&limits).expect("round trip").as_ref(), bytes.as_slice());
    assert_eq!(meter.held(), 0);
}
