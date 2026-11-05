//! Installation reconciliation fixtures for step tests.
use super::fixtures::{self, Harness};
use crate::installations::Read;
use alloc::boxed::Box;
use skein_forge::github::document;
use skein_lib::{Duration, Wall};
const SECOND: Duration = Duration::from_secs(1);
pub(super) fn row(id: u64, suspended: bool) -> document::Installation {
    document::Installation {
        id,
        account: 22,
        login: Box::from(b"acme".as_slice()),
        kind: document::Account::Organization,
        created: Wall::EPOCH,
        suspended,
    }
}

// Feed the committed reconciliation transition directly to cross a written job
// with loss; socket serialization itself is checked by the wire stories.
pub(super) fn commit(h: &mut Harness, rows: Box<[document::Installation]>) {
    let gh = h.component.test_github_mut();
    let mut reading = crate::installations::Reading::new(gh.rows.capacity(), gh.generation, SECOND);
    reading.page(rows, &gh.rows, &gh.accepted, fixtures::shared().name_bytes).expect("fixture read");
    gh.read = Read::Reading(reading);
    gh.commit();
}
