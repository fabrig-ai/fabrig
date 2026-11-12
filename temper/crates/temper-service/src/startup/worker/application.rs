//! The worker's application members (protocol/services.md, 5).
use super::{Limits, Problems, What};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, read};
use skein_json::{Document, document::Kind};
use temper_service_core::startup::at;

pub(super) struct Worker {
    pub root: Box<[u8]>,
    pub git: Option<Box<[u8]>>,
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
fn finish(object: read::Object, document: &Document, path: &Path, problems: &mut Problems) {
    let mut found = config::Problems::empty();
    object.finish(document, path, &mut found);
    problems.append(found, &Path::root());
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
fn worker(
    document: &Document,
    index: usize,
    entry: u32,
    directories: &config::Directories,
    path: &Path,
    problems: &mut Problems,
) -> Option<Worker> {
    let path = at(index, path.member(b"workers"));
    let mut object = object(document, entry, &path, problems)?;
    let root = text(&mut object, document, b"root", &path, problems);
    let git_at = required(&mut object, document, b"git", &path, problems);
    let mut git = None;
    if let Some(at) = git_at {
        let null = match document.token(at) {
            Some(token) => token.kind == Kind::Null,
            None => false,
        };
        if !null {
            match read::text(document, at, 1_048_576) {
                Ok(value) => git = Some(value),
                Err(what) => problems.push(path.member(b"git"), What::Jig(what)),
            }
        }
    }
    finish(object, document, &path, problems);
    let root = root?;
    if let Some(git) = &git
        && !git.starts_with(b"/")
    {
        problems.push(path.member(b"git"), What::Absolute);
    }
    let root = match directories.join(config::Directory::Data, &root) {
        Ok(root) => root,
        Err(what) => {
            problems.push(path.member(b"root"), What::Jig(what));
            return None;
        }
    };
    Some(Worker { root, git })
}
pub(super) fn read(
    document: &Document,
    entry: u32,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<(Worker, Limits)> {
    let path = Path::root().member(b"application");
    let mut object = object(document, entry, &path, problems)?;
    let ats = match required(&mut object, document, b"workers", &path, problems) {
        Some(at) => match read::list(document, at, document.len()) {
            Ok(ats) => Some(ats),
            Err(what) => {
                problems.push(path.member(b"workers"), What::Jig(what));
                None
            }
        },
        None => None,
    };
    let mut own = None;
    if let Some(ats) = ats {
        if ats.len() != 1 {
            problems.push(path.member(b"workers"), What::WorkerAlone);
        }
        for (index, at) in ats.iter().enumerate() {
            let value = worker(document, index, *at, directories, &path, problems);
            if index == 0 {
                own = value;
            }
        }
    }
    let limits = match required(&mut object, document, b"limits", &path, problems) {
        Some(at) => temper_service_core::startup::configured_limits(document, at, &path.member(b"limits"), problems),
        None => None,
    };
    let limits = match limits {
        Some(limits) => Some(Limits::shared(limits)),
        None => None,
    };
    if let Some(limits) = &limits {
        super::git_limits(limits, &limits.git_component, problems);
        super::checks::channel(&limits.bounds(), problems);
    }
    finish_shape(document, object, &path, problems);
    Some((own?, limits?))
}
