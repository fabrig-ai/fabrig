//! The testing application's row-part connector (domain/engine/views.md, 2).
//! Owner 7: task number, a deliberately small kind, busy, and requested version.

use alloc::boxed::Box;
use jig_domain_core::PartInputs;
use jig_domain_core::documents::parts::PartKind;

pub use jig_test_connector::parts::make;

#[must_use]
pub fn kinds() -> Box<[PartKind]> {
    let all = PartInputs { line: true, notes: true, delegates: true, delegate_notes: true, records: true };
    let line = PartInputs { line: true, notes: false, delegates: false, delegate_notes: false, records: false };
    Box::new([
        PartKind { owner: 7, kind: 1, connector: 1, bytes: 8, inputs: all },
        PartKind { owner: 7, kind: 2, connector: 1, bytes: 4, inputs: line },
        PartKind { owner: 7, kind: 3, connector: 2, bytes: 8, inputs: line },
        PartKind { owner: 7, kind: 4, connector: 2, bytes: 8, inputs: line },
    ])
}
