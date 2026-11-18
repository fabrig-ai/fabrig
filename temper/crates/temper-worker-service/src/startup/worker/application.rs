//! The worker's application members (protocol/services.md, 5).
use super::{Limits, Problems, What};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, read};
use skein_json::Document;
use skein_lib::{List, bytes};
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
pub fn fixed(
    directories: &config::Directories,
    git: Option<&[u8]>,
    environment: Box<[EnvironmentEntry]>,
    problems: &mut Problems,
) -> Option<Worker> {
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
        Some(git) => Some(bytes::copy_of(git)),
        None => None,
    };
    Some(Worker { root, git, claude: None, environment })
}
/// The agents' variables, with only HOME and PATH inherited (configuration/figures.md, 6).
#[must_use]
pub fn agents_environment(given: Option<&[EnvironmentEntry]>, own: &[EnvironmentEntry]) -> Box<[EnvironmentEntry]> {
    let given = given.unwrap_or(&[]);
    let bound = given.len().checked_add(own.len()).expect("environment inputs fit together");
    let mut entries = List::with_capacity(u32::try_from(bound).expect("bounded environment inputs"));
    let mut home = false;
    let mut path = false;
    for (name, value) in given {
        home |= name.as_ref() == b"HOME";
        path |= name.as_ref() == b"PATH";
        entries.push((bytes::copy_of(name), bytes::copy_of(value))).expect("input environment bound");
    }
    for wanted in [b"HOME", b"PATH"] {
        if (wanted == b"HOME" && home) || (wanted == b"PATH" && path) {
            continue;
        }
        for (name, value) in own {
            if name.as_ref() == wanted {
                entries.push((bytes::copy_of(name), bytes::copy_of(value))).expect("input environment bound");
                break;
            }
        }
    }
    entries.into_boxed()
}
/// Read the same entries for a worker alone and the box (configuration/README.md, 3).
pub fn environment(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Box<[EnvironmentEntry]> {
    let mut entries = List::with_capacity(document.len());
    match read::list(document, at, document.len()) {
        Ok(ats) => {
            for (index, at) in ats.iter().enumerate() {
                let path = temper_service_core::startup::at(index, path.clone());
                if let Some(mut object) = object(document, *at, &path, problems) {
                    let name = text(&mut object, document, b"name", &path, problems);
                    let value = text(&mut object, document, b"value", &path, problems);
                    let mut found = config::Problems::empty();
                    object.finish(document, &path, &mut found);
                    problems.append(found, &Path::root());
                    if let (Some(name), Some(value)) = (name, value) {
                        for (previous, _) in &entries {
                            if previous == &name {
                                problems.push(path.member(b"name"), What::Jig(config::What::Unique));
                                break;
                            }
                        }
                        entries.push((name, value)).expect("document bounds environment");
                    }
                }
            }
        }
        Err(what) => problems.push(path.clone(), What::Jig(what)),
    }
    entries.into_boxed()
}
fn text(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    let at = required(object, document, name, path, problems)?;
    match read::text(document, at, 1_048_576) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), What::Jig(what));
            None
        }
    }
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
    let environment = match object.member(document, b"environment") {
        Some(at) => environment(document, at, &path.member(b"environment"), problems),
        None => Box::default(),
    };
    let own = fixed(directories, None, environment, problems);
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
