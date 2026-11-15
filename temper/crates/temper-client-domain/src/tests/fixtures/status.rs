//! Project status records used by root tests.
use super as f;
use jig_client::Family;
use temper_client_protocol_wire::{self as wire, v1 as w};
pub fn document(landed: u64, seen: u64, brief: Option<u64>) -> w::ProjectStatus {
    w::ProjectStatus::new(
        &w::CEILINGS,
        w::ProjectStatusParts {
            project: f::PROJECT,
            landed,
            last_landed: Some(100),
            brief: match brief {
                Some(landed) => Some(
                    w::StatusBrief::new(
                        &w::CEILINGS,
                        w::StatusBriefParts {
                            sentence: b"A change landed.".as_slice().into(),
                            lede: b"The change is ready.".as_slice().into(),
                            at: 100,
                            landed,
                        },
                    )
                    .unwrap(),
                ),
                None => None,
            },
            previous: Some(50),
            seen,
            queued: 2,
            red: true,
            branch: b"main".as_slice().into(),
        },
    )
    .unwrap()
}
pub fn record(document: &w::ProjectStatus) -> Family {
    Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::project_status(document) }
}
