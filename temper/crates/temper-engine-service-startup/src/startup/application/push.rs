//! Deployment push configuration (temper/docs/design/protocol/configuration/engine.md, push).
use super::{Problems, What, read, settings};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path};
use skein_json::{Document, document::Kind};
use skein_lib::{List, bytes};

#[derive(Debug)]
pub struct Push {
    pub key: Box<[u8]>,
    pub subject: Box<[u8]>,
    pub services: Box<[Box<[u8]>]>,
}
fn path() -> Path {
    Path::root().member(b"application").member(b"push")
}
fn services(document: &Document, at: u32, problems: &mut Problems) -> Option<Box<[Box<[u8]>]>> {
    if document.token(at)?.kind == Kind::Null {
        return Some(Box::new([
            bytes::copy_of(b"https://fcm.googleapis.com"),
            bytes::copy_of(b"https://updates.push.services.mozilla.com"),
            bytes::copy_of(b"https://web.push.apple.com"),
        ]));
    }
    let path = path().member(b"services");
    let ats = match config::read::list(document, at, 8) {
        Ok(ats) => ats,
        Err(what) => {
            problems.push(path, What::Jig(what));
            return None;
        }
    };
    read::form(!ats.is_empty(), path.clone(), problems);
    let mut out = List::with_capacity(8);
    for (index, at) in ats.iter().enumerate() {
        let path = read::index(&path, index);
        match config::read::text(document, *at, 1024) {
            Ok(value) => {
                let valid = match super::sign_in::url(&value) {
                    Some((origin, _, _, _)) => origin == value.as_ref(),
                    None => false,
                };
                read::form(valid, path, problems);
                out.push(value).expect("eight services");
            }
            Err(what) => problems.push(path, What::Jig(what)),
        }
    }
    Some(out.into_boxed())
}
pub(super) fn parse(
    document: &Document,
    at: Option<u32>,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<Push> {
    let at = at?;
    if document.token(at)?.kind == Kind::Null {
        return None;
    }
    let path = path();
    let mut object = read::object(document, at, "push", &path, problems)?;
    let key = read::text(&mut object, document, b"key", &path, problems);
    let subject_at = read::required(&mut object, document, b"subject", &path, problems);
    let services_at = read::required(&mut object, document, b"services", &path, problems);
    read::finish(object, document, &path, problems);
    let subject = match config::read::text(
        document,
        subject_at?,
        u32::try_from(skein_push::SUBJECT_BYTES).expect("subject bound"),
    ) {
        Ok(value) => value,
        Err(what) => {
            problems.push(path.member(b"subject"), What::Jig(what));
            return None;
        }
    };
    let mut valid = subject.starts_with(b"mailto:") || subject.starts_with(b"https:");
    for byte in &subject {
        valid &= (b'!'..=b'~').contains(byte) && *byte != b'"' && *byte != b'\\';
    }
    read::form(valid, path.member(b"subject"), problems);
    let services = services(document, services_at?, problems)?;
    let key = super::joined(directories, config::Directory::Data, &key?, &path.member(b"key"), problems)?;
    Some(Push { key, subject, services })
}
pub(super) fn file(push: &Push) -> config::File {
    config::File {
        at: path().member(b"key"),
        path: push.key.clone(),
        max: u32::try_from(skein_push::VAPID_PEM_BYTES).expect("PEM bound"),
        private: true,
    }
}
pub(super) fn resolve(push: &Push, files: &settings::Files, problems: &mut Problems) -> Option<settings::Push> {
    let pem = super::contents(files, &push.key, file(push).max, path().member(b"key"), problems)?;
    let Some(key) = skein_push::Vapid::from_pkcs8_pem(&pem) else {
        problems.push_file(path().member(b"key"), What::Jig(config::What::Form), Some(push.key.clone()));
        return None;
    };
    Some(settings::Push { key, subject: push.subject.clone(), services: push.services.clone() })
}
