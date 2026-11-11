//! Own-scope devices reader (temper/docs/design/client/forge/records.md, devices).
use super::{Limits, wire_limits};
use alloc::boxed::Box;
use jig_client::{Family, Watchable, contract as c};
use skein_lib::{List, Wall};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub number: u32,
    pub label: Box<[u8]>,
    pub at: Wall,
    pub endpoint: Box<[u8]>,
}
#[must_use]
pub fn watch(limits: Limits) -> Option<Watchable> {
    let record = w::Watching::new(&wire_limits(&limits)?, w::WatchingParts { watch: w::Watch::Devices }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Own,
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&record) },
    })
}
#[must_use]
pub fn rows(record: &Family, limits: Limits) -> Option<Box<[Row]>> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let document = wire::decode_devices(&record.bytes, &wire_limits(&limits)?)?;
    let mut rows = List::uncapped();
    for row in document.into_parts().rows.into_boxed() {
        let row = row.into_parts();
        rows.push(Row { number: row.number, label: row.label, at: Wall::from_nanos(row.at), endpoint: row.endpoint })
            .expect("uncapped devices");
    }
    Some(rows.into_boxed())
}

/// Only the push service's host is shown; never the subscription path.
#[must_use]
pub fn host(endpoint: &[u8]) -> &[u8] {
    let rest = match endpoint.strip_prefix(b"https://") {
        Some(rest) => rest,
        None => match endpoint.strip_prefix(b"http://") { Some(rest) => rest, None => return b"" },
    };
    for (index, byte) in rest.iter().enumerate() {
        if *byte == b'/' || *byte == b':' { return rest.get(..index).expect("host prefix"); }
    }
    rest
}
