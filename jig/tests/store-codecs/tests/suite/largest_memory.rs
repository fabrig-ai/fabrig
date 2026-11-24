//! Largest values at the ceilings, or at the largest clamp under the cap.

use super::codec_memory::family;
use jig_protocol_store::tasks_v13::Record;

const CAP: u64 = 64 * 1024 * 1024;

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
        jig_protocol_store::STORE_FAMILIES.get(1).expect("store family").current,
        jig_protocol_store::openings_v1,
        roots [
            WorkerOpening,
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
            Number64,
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
