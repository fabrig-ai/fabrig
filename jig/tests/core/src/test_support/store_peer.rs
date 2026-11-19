//! Opaque store evidence shared by controls and domain stories (jig/docs/design/domain/testing.md, 9).
use jig_domain_core as core;
use jig_test_domain::{self as root, Write};
use skein_lib::Wall;

#[must_use]
pub fn opaque_turn(bytes: &[u8]) -> root::Record {
    root::Record::Core(core::Record::Core(core::CoreRecord::Turn(core::TurnRecord {
        opens_window: false,
        task: (1_u64 << 63) - 2,
        attempt: u64::MAX - 2,
        turn: u32::MAX,
        spent: u64::MAX - 3,
        read: Some(u64::MAX - 4),
        at: Wall::from_nanos(999_999),
        transcript: bytes.into(),
    })))
}

const HEADER: core::Deployment = core::Deployment {
    id: [18; 16],
    tasks: 0,
    people: 0,
    sign_ins: 0,
    messages: 0,
    runs: 0,
    calls: 0,
    connector_rows: 0,
    projects: 0,
    restart_wait_ms: None,
    applications: 0,
    commits: 0,
};
#[must_use]
pub fn header(commits: u64) -> root::Record {
    let mut header = HEADER;
    header.commits = commits;
    root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(header)))
}
#[must_use]
pub fn writes(number: u64, write: Write) -> Box<[Write]> {
    Box::new([Write::Save(header(number)), write])
}
