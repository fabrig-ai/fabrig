//! The provider clients joined to Forgejo (protocol/sign-in.md, 9.7).
use super::super::read;
use crate::startup::{Problems, What};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path};
use skein_json::Document;
use skein_lib::List;
#[derive(Clone, Debug)]
pub struct ForgeSignIn {
    pub forge: u16,
    pub client: Box<[u8]>,
}
pub(super) fn parse(
    document: &Document,
    object: &mut config::read::Object,
    path: &Path,
    problems: &mut Problems,
) -> Box<[ForgeSignIn]> {
    let Some(at) = object.member(document, b"forges") else { return Box::new([]) };
    let path = path.member(b"forges");
    let entries = match config::read::list(document, at, document.len()) {
        Ok(entries) => entries,
        Err(what) => {
            problems.push(path, What::Jig(what));
            return Box::new([]);
        }
    };
    let mut rows = List::<ForgeSignIn>::with_capacity(document.len());
    for (index, at) in entries.iter().enumerate() {
        let path = read::index(&path, index);
        let Some(mut object) = read::object(document, *at, &path, problems) else { continue };
        let forge = read::short(&mut object, document, b"forge", &path, problems);
        let client = read::text(&mut object, document, b"client", &path, problems);
        read::finish(object, document, &path, problems);
        if let (Some(forge), Some(client)) = (forge, client) {
            for previous in &rows {
                if previous.forge == forge {
                    problems.push(path.member(b"forge"), What::Jig(config::What::Unique));
                }
                if previous.client == client {
                    problems.push(path.member(b"client"), What::Jig(config::What::Unique));
                }
            }
            rows.push(ForgeSignIn { forge, client }).expect("document bounds entries");
        }
    }
    rows.into_boxed()
}
/// The machine's owned template supplies the public browser origin for sign-in.
pub(crate) fn origin(machine: &temper_engine_domain_forge::Machine) -> Option<Box<[u8]>> {
    let record = machine.owned.as_ref()?;
    let text = record.setup.as_ref()?;
    let document = config::document(text, &config::DocumentLimits { bytes: 4096, tokens: 4096, depth: 32 }).ok()?;
    let mut entry = config::read::Object::new(&document, 0).ok()?;
    let mut mode = config::read::Object::new(&document, entry.member(&document, b"mode")?).ok()?;
    let mut owned = config::read::Object::new(&document, mode.member(&document, b"owned")?).ok()?;
    let public = match owned.member(&document, b"public") {
        Some(at) => match config::read::text(&document, at, 4096) {
            Ok(value) => {
                let parsed = skein_forge::github::redirect::parse(&value, 4096, 1).ok()?;
                Some(temper_forge_setup::setup::Origin { host: parsed.origin.host, port: parsed.origin.port })
            }
            Err(_) => None,
        },
        None => None,
    };
    Some(super::super::forge_origin(&record.git, public.as_ref()))
}
pub(crate) fn check(
    rows: &[ForgeSignIn],
    machine: &temper_engine_domain_forge::Machine,
    jig: &config::Engine,
    problems: &mut Problems,
) {
    let path = super::path().member(b"forges");
    for (index, row) in rows.iter().enumerate() {
        let path = read::index(&path, index);
        let mut visible = !row.client.is_empty();
        for byte in &row.client {
            if !byte.is_ascii_graphic() {
                visible = false;
            }
        }
        if !visible {
            problems.push(path.member(b"client"), What::Jig(config::What::Form));
        }
        let origin = origin(machine);
        if origin.is_none() {
            problems.push(path.member(b"forge"), What::Jig(config::What::Refers));
        }
        let Some(provider) = &jig.clients.provider else {
            problems.push(path.member(b"client"), What::Against { other: b"engine.clients.provider" });
            continue;
        };
        let mut found = false;
        for client in &provider.clients {
            if client.id == row.client {
                found = true;
                if let Some(origin) = &origin {
                    let suffix = client.redirect_uri.strip_prefix(origin.as_ref());
                    if suffix != Some(b"/user/oauth2/temper/callback".as_slice()) {
                        problems.push(path.member(b"client"), What::Jig(config::What::Form));
                    }
                }
            }
        }
        if !found {
            problems.push(path.member(b"client"), What::Jig(config::What::Refers));
        }
    }
}
