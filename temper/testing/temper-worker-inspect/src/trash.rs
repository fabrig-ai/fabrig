//! The sweep's retention rule (temper/docs/design/testing/worker.md, 2.3).
use skein_world::inspect::{Entry, Inspection, Release, Table};
use temper_worker_protocol_git::Component;

/// Inspect the world's listing of names immediately beneath `trash/`.
pub fn inspect_trash(component: &Component, trash: &[&[u8]], into: &mut Inspection) {
    let (pending, failures, refused_removal, ended_serial) = component.sweep_state();
    into.table(Table {
        name: "trash",
        capacity: u64::MAX,
        entries: trash
            .iter()
            .map(|name| {
                let serial = std::str::from_utf8(name).ok().and_then(|name| name.parse::<u64>().ok());
                // Unparseable names and entries made since the pass wait for the next pass.
                // Directories::rename checks addition, so serial numbers never wrap.
                let before_pass_ended = match (serial, ended_serial) {
                    (Some(serial), Some(ended_serial)) => serial < ended_serial,
                    _ => false,
                };
                Entry {
                    key: format!("trash/{}", String::from_utf8_lossy(name)),
                    rule: "the sweep",
                    release: if pending || (refused_removal && failures > 0) || !before_pass_ended {
                        Release::Pending
                    } else {
                        Release::Due
                    },
                }
            })
            .collect(),
        refused: Vec::new(),
    });
}
