//! Workers and their plaintext listeners (configuration.md, 3).
use crate::{Address, Path, Problems, Segment, What, address, check};
use alloc::boxed::Box;
use skein_json::Document;
use skein_lib::List;
#[derive(Clone, Debug)]
pub struct Hosts {
    pub listener: Box<[Address]>,
}
pub(crate) fn parse(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &crate::Limits,
    _directories: &crate::Directories,
    problems: &mut Problems,
) -> Option<Hosts> {
    let mut object = check::object(document, at, path, problems, "hosts")?;
    if object.member(document, b"workers").is_some() {
        problems.push(path.member(b"workers"), What::Because(b"workers are settings in the store"));
    }
    for (member, configured, requires, offered) in [
        (
            b"fleet.configured_workers".as_slice(),
            limits.core.fleet.configured_workers,
            b"link.names".as_slice(),
            limits.link.names,
        ),
        (
            b"fleet.worker_name_bytes".as_slice(),
            limits.core.fleet.worker_name_bytes,
            b"link.name".as_slice(),
            limits.link.name,
        ),
    ] {
        if offered < configured {
            problems.push(
                Path::root().member(b"limits").member(b"overrides").member(requires),
                What::Capacity { member, configured, requires, required: u64::from(configured) },
            );
        }
    }
    let mut listener = List::with_capacity(document.len());
    if let Some(items) = check::list(&mut object, document, b"listener", document.len(), path, problems) {
        for (index, at) in items.iter().enumerate() {
            let path =
                path.member(b"listener").child(Segment::Index(u32::try_from(index).expect("list bounded by u32")));
            if let Some(address) = address::read(document, *at, &path, problems) {
                check::local(&address, &path, problems);
                listener.push(address).expect("input count bounded by openings");
            }
        }
    }
    object.finish(document, path, problems);
    Some(Hosts { listener: listener.into_boxed() })
}
