//! The worker's application members (protocol/services.md, 5).
use super::{Limits, Problems, What};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, read};
use skein_json::Document;
pub type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);

#[derive(Debug)]
pub struct Worker {
    pub root: Box<[u8]>,
    pub git: Option<Box<[u8]>>,
    pub claude: Option<Box<[u8]>>,
    pub environment: Box<[EnvironmentEntry]>,
}
pub(super) fn object(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<read::Object> {
    match read::Object::new(document, at) {
        Ok(object) => Some(object),
        Err(what) => {
            problems.push(path.clone(), What::Jig(what));
            None
        }
    }
}
pub(super) fn required(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<u32> {
    let at = object.member(document, name);
    if at.is_none() {
        problems.push(path.member(name), What::Jig(config::What::Missing));
    }
    at
}
pub(super) fn finish_shape(document: &Document, object: read::Object, path: &Path, problems: &mut Problems) {
    let mut found = config::Problems::empty();
    object.finish(document, path, &mut found);
    for found in &found.problems {
        problems.push(
            found.path.clone(),
            if found.what == config::What::Unknown { What::WorkerAlone } else { What::Jig(found.what) },
        );
    }
    problems.past = problems.past.saturating_add(found.past);
}
/// The shell supplies git; workspaces belong to the data directory (moved.md, 2).
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub fn fixed(directories: &config::Directories, git: Option<&[u8]>, problems: &mut Problems) -> Option<Worker> {
    let path = Path::root().member(b"application").member(b"workers").child(config::Segment::Index(0));
    if let Some(git) = git
        && !git.starts_with(b"/")
    {
        problems.push(path.member(b"git"), What::Absolute);
    }
    let root = match directories.join(config::Directory::Data, b"workspaces") {
        Ok(root) => root,
        Err(what) => {
            problems.push(path.member(b"root"), What::Jig(what));
            return None;
        }
    };
    let git = match git {
        Some(git) => Some(skein_lib::bytes::copy_of(git)),
        None => None,
    };
    Some(Worker { root, git, claude: None, environment: Box::default() })
}
pub(super) fn read(
    document: &Document,
    entry: u32,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<(Worker, Limits)> {
    read_over(document, entry, directories, temper_service_core::derived::STOCK, problems)
}
pub(super) fn read_over(
    document: &Document,
    entry: u32,
    directories: &config::Directories,
    sources: temper_service_core::derived::Sources,
    problems: &mut Problems,
) -> Option<(Worker, Limits)> {
    let path = Path::root().member(b"application");
    let mut object = object(document, entry, &path, problems)?;
    if object.member(document, b"workers").is_some() {
        problems.push(path.member(b"workers"), What::Jig(config::What::Fixed));
    }
    let own = fixed(directories, None, problems);
    if object.member(document, b"limits").is_some() {
        problems.push(path.member(b"limits"), What::Jig(config::What::Because(super::MOVED_LIMITS)));
    }
    let mut limits = temper_service_core::limits::stock();
    limits.checkout.message_bytes = sources.payload;
    if let Some(slots) = temper_service_core::derived::slots(document) {
        temper_service_core::derived::places(document, entry, slots, &mut limits);
    }
    let limits = Limits::shared(limits);
    finish_shape(document, object, &path, problems);
    Some((own?, limits))
}
