//! The permanent baseline refuses at F0, before switching generations.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary Rust baseline disk test"
)]
use super::unseeded_upgrade::{SETS, component, disk::Disk};
use jig_protocol_store::{self as store, Event, Request};

#[test]
fn baseline_store_refuses_retired_milestones_without_switching_active() {
    fn load(directory: &std::path::Path, prefix: &str, items: &mut Vec<skein_fake_machine::Item>) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().into_string().unwrap();
            let relative = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
            if entry.file_type().unwrap().is_dir() {
                items.push(skein_fake_machine::Item::directory(relative.as_bytes()).mode(0o700));
                load(&entry.path(), &relative, items);
            } else {
                items.push(
                    skein_fake_machine::Item::file(relative.as_bytes(), &std::fs::read(entry.path()).unwrap())
                        .mode(if relative == "active" || relative.ends_with("/manifest") { 0o644 } else { 0o600 }),
                );
            }
        }
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/one-process/golden/baseline/state");
    let active = std::fs::read(path.join("active")).unwrap();
    let mut items = Vec::new();
    load(&path, "", &mut items);
    let mut disk = Disk::with_items(&items);
    let mut current = component(&disk, SETS);
    let mut pending = alloc::collections::VecDeque::from(disk.open(&mut current));
    let mut refusal = None;
    while let Some(event) = pending.pop_front() {
        match event {
            Event::Job { owner, of, job, page } => {
                let answer = match of {
                    store::Owner::Jig => store::jobs::page(&job, &page),
                    store::Owner::Application => {
                        let bounds =
                            crate::bounds(&super::fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
                        let present = crate::jobs::read_present(&mut current, &bounds).unwrap();
                        crate::jobs::page(&job, &page, &present, &bounds)
                    }
                    other => panic!("unexpected owner {other:?}"),
                };
                pending.extend(disk.request(&mut current, Request::Job { owner, answer }));
            }
            Event::OpenRefused { why, .. } => refusal = Some(why),
            Event::Patched { .. } | Event::Ran { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    let Some(store::OpenRefusal::Layout { family, value_type: 10, why: store::Unplanned::Retired, .. }) = &refusal
    else {
        panic!("{refusal:?}; {:?}", current.start_open_failure());
    };
    assert_eq!(family.as_ref(), "jig-store-tasks");
    assert_eq!(disk.active(), active);
}
