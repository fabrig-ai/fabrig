//! Codec entry points for synthetic benchmark observations.

use smith_events::{Capture, Limits, Record};
use super::events::{bytes, LIMITS};

#[must_use]
pub fn observe(records: &[Record]) -> smith_bench::agent::smith::events::Observer {
    let bytes = bytes(records);
    let ceiling = u32::try_from(bytes.split_inclusive(|byte| *byte == b'\n').map(<[u8]>::len).max().expect("records"))
        .expect("bounded fixture lines");
    observe_bytes(&bytes, &LIMITS, ceiling)
}

#[must_use]
pub fn observe_bytes(bytes: &[u8], limits: &Limits, ceiling: u32) -> smith_bench::agent::smith::events::Observer {
    let mut observer = smith_bench::agent::smith::events::Observer::with_ceiling(limits, ceiling).expect("receiving bounds");
    observer.feed(bytes).expect("fixture codec records");
    observer.finish().expect("complete fixture framing");
    observer
}

#[must_use]
pub fn captured(records: &[Record], capture: &Capture, limits: &Limits) -> smith_bench::agent::smith::events::Observer {
    let mut bytes = Vec::new();
    let mut ceiling = 1;
    for record in records {
        let line = smith_events::write(record, capture, limits).expect("bounded fixture").expect("retained metadata");
        ceiling = ceiling.max(u32::try_from(line.len()).expect("bounded fixture line"));
        bytes.extend_from_slice(&line);
    }
    observe_bytes(&bytes, limits, ceiling)
}
