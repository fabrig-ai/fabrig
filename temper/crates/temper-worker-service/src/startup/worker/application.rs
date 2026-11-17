//! The worker's application members (protocol/services.md, 5).
use super::{Limits, Problems, What};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, read};
use skein_json::{Document, document::Kind};
use temper_service_core::startup::at;

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
pub fn member(
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
    let claude = match object.member(document, b"claude") {
        Some(at) => match read::text(document, at, 4096) {
            Ok(value) => {
                if !value.starts_with(b"/") {
                    problems.push(path.member(b"claude"), What::Absolute);
                }
                Some(value)
            }
            Err(what) => {
                problems.push(path.member(b"claude"), What::Jig(what));
                None
            }
        },
        None => None,
    };
    let environment = environment(&mut object, document, &path, problems);
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
    Some(Worker { root, git, claude, environment })
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
            let value = member(document, index, *at, directories, &path, problems);
            if index == 0 {
                own = value;
            }
        }
    }
    if object.member(document, b"limits").is_some() {
        problems.push(path.member(b"limits"), What::Jig(config::What::Because(super::MOVED_LIMITS)));
    }
    let limits = Limits::shared(temper_service_core::limits::profile(temper_service_core::limits::Profile::Standard));
    finish_shape(document, object, &path, problems);
    Some((own?, limits))
}

fn environment(
    object: &mut read::Object,
    document: &Document,
    path: &Path,
    problems: &mut Problems,
) -> Box<[EnvironmentEntry]> {
    let mut entries = skein_lib::List::uncapped();
    if let Some(at) = object.member(document, b"environment") {
        let path = path.member(b"environment");
        match read::list(document, at, document.len()) {
            Ok(ats) => {
                for (index, at) in ats.iter().enumerate() {
                    let path = temper_service_core::startup::at(index, path.clone());
                    if let Some(mut object) = self::object(document, *at, &path, problems) {
                        let name = text(&mut object, document, b"name", &path, problems);
                        let value = text(&mut object, document, b"value", &path, problems);
                        finish(object, document, &path, problems);
                        if let (Some(name), Some(value)) = (name, value) {
                            for (previous, _) in &entries {
                                if previous == &name {
                                    problems.push(path.member(b"name"), What::Jig(config::What::Unique));
                                    break;
                                }
                            }
                            entries.push((name, value)).expect("uncapped environment");
                        }
                    }
                }
            }
            Err(what) => problems.push(path, What::Jig(what)),
        }
    }
    entries.into_boxed()
}
