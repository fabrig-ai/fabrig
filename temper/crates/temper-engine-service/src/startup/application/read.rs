//! The application's strict fields use jig's readers and paths.
use super::{Problems, What};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, read};
use skein_json::Document;
use skein_lib::List;

pub(super) fn object(
    document: &Document,
    at: u32,
    shape: &'static str,
    path: &Path,
    problems: &mut Problems,
) -> Option<super::super::members::Object> {
    match super::super::members::object(document, at, shape) {
        Ok(object) => Some(object),
        Err(what) => {
            problems.push(path.clone(), What::Jig(what));
            None
        }
    }
}
pub(super) fn required(
    object: &mut super::super::members::Object,
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
pub(super) fn finish(object: super::super::members::Object, document: &Document, path: &Path, problems: &mut Problems) {
    let mut found = config::Problems::empty();
    object.finish(document, path, &mut found);
    problems.append(found, &Path::root());
}
pub(super) fn text(
    object: &mut super::super::members::Object,
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
pub(super) fn integer(
    object: &mut super::super::members::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<u64> {
    let at = required(object, document, name, path, problems)?;
    match read::integer(document, at, u64::MAX) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), What::Jig(what));
            None
        }
    }
}
pub(super) fn number(
    object: &mut super::super::members::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<u32> {
    let value = integer(object, document, name, path, problems)?;
    match u32::try_from(value) {
        Ok(value) => Some(value),
        Err(_) => {
            problems.push(path.member(name), What::Jig(config::What::Bound));
            None
        }
    }
}
pub(super) fn short(
    object: &mut super::super::members::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<u16> {
    let value = integer(object, document, name, path, problems)?;
    match u16::try_from(value) {
        Ok(value) => Some(value),
        Err(_) => {
            problems.push(path.member(name), What::Jig(config::What::Bound));
            None
        }
    }
}
pub(super) fn boolean(
    object: &mut super::super::members::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<bool> {
    let at = required(object, document, name, path, problems)?;
    match read::boolean(document, at) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), What::Jig(what));
            None
        }
    }
}
pub(super) fn list(
    object: &mut super::super::members::Object,
    document: &Document,
    name: &[u8],
    max: u32,
    path: &Path,
    problems: &mut Problems,
) -> Option<List<u32>> {
    let at = required(object, document, name, path, problems)?;
    match read::list(document, at, max) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), What::Jig(what));
            None
        }
    }
}
pub(super) fn index(path: &Path, index: usize) -> Path {
    path.child(config::Segment::Index(u32::try_from(index).expect("bounded document count")))
}
pub(super) fn form(valid: bool, path: Path, problems: &mut Problems) {
    if !valid {
        problems.push(path, What::Jig(config::What::Form));
    }
}

/// These optional values are explicit in init's document; old files need their fix.
pub(super) fn nullable(
    object: &mut super::super::members::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<u32> {
    let at = object.member(document, name);
    if at.is_none() {
        problems.push(path.member(name), What::Half(crate::startup::EngineWhat::MissingNull));
    }
    at
}

// Override keys are names in the limits table, not fixed configuration members.
pub(super) fn overrides(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<read::Object> {
    match read::Object::new(document, at) {
        Ok(object) => Some(object),
        Err(what) => {
            problems.push(path.clone(), What::Jig(what));
            None
        }
    }
}
