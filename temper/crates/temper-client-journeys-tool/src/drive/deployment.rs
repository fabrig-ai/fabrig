//! Pure deployment documents and plans (testing/qa/deployment.md, 2–6).
use super::{Input, Plan, Program, options::parse_ports};
use crate::line::{BoxValue, Document};
use sha2::{Digest, Sha256};
use skein_json::{Document as JsonDocument, Kind, document, tokenizer, writer};
use std::{ffi::OsString, os::unix::ffi::OsStrExt, path::PathBuf};

pub const DOCUMENT_BYTES: u32 = 256 * 1024;
pub const PATCH_BYTES: u32 = 64 * 1024;
const JSON: tokenizer::Limits = tokenizer::Limits {
    depth: 128,
    string: DOCUMENT_BYTES,
    number: DOCUMENT_BYTES,
    chunk: 4096,
    length: DOCUMENT_BYTES,
};

/// All paths are absolute; port positions are those of the place, not process order.
#[derive(Clone, Debug)]
pub struct Place {
    ports: super::place::Ports,
    scratch: PathBuf,
    bin: PathBuf,
    bundle: Option<PathBuf>,
}
impl Place {
    pub fn new(ports: [u16; 9], scratch: PathBuf, bin: PathBuf, bundle: Option<PathBuf>) -> Result<Self, String> {
        if !scratch.is_absolute() || !bin.is_absolute() {
            return Err("place scratch and --bin must be absolute".into());
        }
        if bundle.as_ref().is_some_and(|path| !path.is_absolute()) {
            return Err("place --bundle must be absolute".into());
        }
        if ports.contains(&0) || ports.iter().collect::<std::collections::BTreeSet<_>>().len() != 9 {
            return Err("place requires nine distinct nonzero ports".into());
        }
        Ok(Self { ports: super::place::Ports::new(ports), scratch, bin, bundle })
    }
    /// NAME without the `place.` prefix or braces.
    #[must_use]
    pub fn value(&self, name: &str) -> Option<Box<[u8]>> {
        let port = match name {
            "clients" => Some(self.ports.clients),
            "workers" => Some(self.ports.owned),
            "webhooks" => Some(self.ports.webhooks),
            "model" => Some(self.ports.model),
            "forgejo" => Some(self.ports.forgejo),
            _ => None,
        };
        if let Some(port) = port {
            return Some(format!("127.0.0.1:{port}").into_bytes().into());
        }
        let path = match name {
            "config" | "data" | "cache" => self.scratch.join(name),
            "bin" => self.bin.clone(),
            "git" => self.scratch.join("data/bin/git"),
            "bundle" => self.bundle.clone()?,
            _ => return None,
        };
        Some(path.as_os_str().as_bytes().into())
    }
}

pub fn render(document_name: &str, bytes: &[u8], place: &Place) -> Result<Box<[u8]>, String> {
    let mut json = Json::parse(document_name, bytes, DOCUMENT_BYTES)?;
    json.render(document_name, place)?;
    json.encode(document_name)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preparation {
    Fresh,
    Held,
}
pub fn preparation(config_present: bool, data_present: bool) -> Result<Preparation, String> {
    match (config_present, data_present) {
        (false, false) => Ok(Preparation::Fresh),
        (true, true) => Ok(Preparation::Held),
        (true, false) => Err("config/ is present without data/".into()),
        (false, true) => Err("data/ is present without config/".into()),
    }
}

pub type RenderedFile = (PathBuf, Box<[u8]>);

/// The io supplies regular files from config/; no file is read or written here.
/// Fresh preparation's destinations are absolute paths under scratch.
pub fn documents(files: &[(&str, &[u8])], place: &Place) -> Result<Vec<RenderedFile>, String> {
    let names = [
        ("engine.json", Document::Engine.path()),
        ("worker.json", Document::Worker.path()),
        ("forgejo.json", "config/forgejo.json"),
        ("model.json", "config/model.json"),
    ];
    for (name, _) in files {
        if !names.iter().any(|(expected, _)| name == expected) {
            return Err(format!("config/{name}: unexpected deployment document"));
        }
    }
    let mut rendered = vec![];
    for (name, destination) in names {
        let mut found = files.iter().filter(|(given, _)| *given == name);
        let (_, bytes) = found.next().ok_or_else(|| format!("config/{name}: missing deployment document"))?;
        if found.next().is_some() {
            return Err(format!("config/{name}: repeated deployment document"));
        }
        rendered.push((place.scratch.join(destination), render(name, bytes, place)?));
    }
    Ok(rendered)
}

/// Exactly temper-init's pair: hash the hexadecimal digits, not their newline.
#[must_use]
pub fn secret_pair(bytes: &[u8; 32]) -> (Box<[u8]>, Box<[u8]>) {
    let mut secret = crate::recipe::hex(*bytes).into_bytes();
    let digest = Sha256::digest(&secret);
    let mut digest = crate::recipe::hex(digest.into()).into_bytes();
    secret.push(b'\n');
    digest.push(b'\n');
    (secret.into(), digest.into())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Values {
    pub forgejo_api: Box<[u8]>,
    pub forgejo_git: Box<[u8]>,
    pub forgejo_admin: Box<[u8]>,
    pub git: Box<[u8]>,
    pub model_key: Box<[u8]>,
}
impl Values {
    #[must_use]
    pub fn value(&self, name: BoxValue) -> &[u8] {
        match name {
            BoxValue::Api => &self.forgejo_api,
            BoxValue::Git => &self.forgejo_git,
            BoxValue::Admin => &self.forgejo_admin,
            BoxValue::GitProgram => &self.git,
            BoxValue::ModelKey => &self.model_key,
        }
    }
}
mod worker;
pub use worker::Worker;

pub fn values(place: &Place, forgejo: &[u8], model: &[u8]) -> Result<Values, String> {
    let forgejo = Json::parse("forgejo.json", forgejo, DOCUMENT_BYTES)?;
    let model = Json::parse("model.json", model, DOCUMENT_BYTES)?;
    let api = place.value("forgejo").expect("a place has a forgejo listener");
    Ok(Values {
        forgejo_git: [b"http://".as_slice(), &api].concat().into(),
        forgejo_api: api,
        forgejo_admin: forgejo.member_string("forgejo.json", b"admin_token")?,
        git: place.value("git").expect("a place has a git program"),
        model_key: model.member_string("model.json", b"key")?,
    })
}

/// RFC 7396; only patch string values are rendered. Both patch and result are objects.
pub fn merge(document: &[u8], patch: &[u8], place: &Place) -> Result<Box<[u8]>, String> {
    let mut document = Json::parse("document", document, DOCUMENT_BYTES)?;
    let mut patch = Json::parse("patch", patch, PATCH_BYTES)?;
    if !matches!(patch, Json::Object(_)) {
        return Err("patch: expected a JSON object".into());
    }
    patch.render("patch", place)?;
    document.merge(patch);
    if !matches!(document, Json::Object(_)) {
        return Err("merge result: expected a JSON object".into());
    }
    document.encode("merge result")
}

impl Plan {
    pub fn place(&self) -> Result<Place, String> {
        Place::new(
            parse_ports(self.options.ports.as_deref().ok_or("--deployment requires --ports LIST")?)?,
            self.options.scratch.clone(),
            self.options.bin.clone(),
            self.options.bundle.clone(),
        )
    }
    /// Forgejo, model, engine, worker, in start order (deployment.md, 5).
    #[must_use]
    pub fn deployment_programs(&self) -> [Program; 4] {
        let scratch = &self.options.scratch;
        let forgejo = self.program(
            "temper-box",
            vec![
                "fake".into(),
                "forge".into(),
                "--document".into(),
                scratch.join("config/forgejo.json").into_os_string(),
                "--ready".into(),
                "listening ".into(),
                "--state".into(),
                scratch.join("data/forgejo/state").into_os_string(),
            ],
            Input::Null,
        );
        let mut model = vec![
            "fake".into(),
            "llm".into(),
            "--document".into(),
            scratch.join("config/model.json").into_os_string(),
            "--ready".into(),
            "listening ".into(),
        ];
        if !self.options.scripts.is_empty() {
            model.extend(["--scripts".into(), scratch.join("session/scripts.json").into_os_string()]);
        }
        if self.options.strict {
            model.push("--strict".into());
        }
        let model = self.program("temper-box", model, Input::Null);
        let service = |name: &str| {
            let mut args: Vec<OsString> = vec!["serve".into()];
            for (flag, directory) in [("--config", "config"), ("--data", "data"), ("--cache", "cache")] {
                args.extend([flag.into(), scratch.join(directory).join(name).into_os_string()]);
            }
            self.program(&format!("temper-{name}"), args, Input::Null)
        };
        [forgejo, model, service("engine"), service("worker")]
    }
    #[must_use]
    pub fn deployment_sign_in(&self) -> Program {
        self.ctl(
            &[b"sign-in".as_slice().into(), self.options.scratch.join("session/link").as_os_str().as_bytes().into()],
            Input::Null,
        )
    }
}

/// A mutable JSON value keeps object order and number spellings while merging.
#[derive(Debug)]
enum Json {
    Object(Vec<(Box<[u8]>, Self)>),
    Array(Vec<Self>),
    String(Box<[u8]>),
    Number(Box<[u8]>),
    Boolean(bool),
    Null,
}
impl Json {
    fn parse(name: &str, bytes: &[u8], limit: u32) -> Result<Self, String> {
        if bytes.len() > usize::try_from(limit).expect("document byte bound fits usize") {
            return Err(format!("{name}: document exceeds {limit} bytes"));
        }
        let doc = skein_json::parse(
            bytes,
            &tokenizer::Limits { length: limit, ..JSON },
            &document::Limits { tokens: limit, text: limit },
        )
        .map_err(|why| format!("{name}: invalid JSON: {why:?}"))?;
        Ok(Self::read(&doc, &mut 0))
    }
    fn read(doc: &JsonDocument, cursor: &mut u32) -> Self {
        let token = doc.token(*cursor).expect("validated JSON has a value");
        *cursor += 1;
        match token.kind {
            Kind::ObjectStart => {
                let mut members = vec![];
                while doc.token(*cursor).expect("object closes").kind != Kind::ObjectEnd {
                    let key = doc.token(*cursor).expect("object member has a key");
                    let key = doc.text(key).expect("decoded key").into();
                    *cursor += 1;
                    members.push((key, Self::read(doc, cursor)));
                }
                *cursor += 1;
                Self::Object(members)
            }
            Kind::ArrayStart => {
                let mut items = vec![];
                while doc.token(*cursor).expect("array closes").kind != Kind::ArrayEnd {
                    items.push(Self::read(doc, cursor));
                }
                *cursor += 1;
                Self::Array(items)
            }
            Kind::String => Self::String(doc.text(token).expect("decoded string").into()),
            Kind::Number => Self::Number(doc.text(token).expect("validated number").into()),
            Kind::True => Self::Boolean(true),
            Kind::False => Self::Boolean(false),
            Kind::Null => Self::Null,
            Kind::Key | Kind::ObjectEnd | Kind::ArrayEnd | Kind::Long => unreachable!("a parsed JSON value"),
        }
    }
    fn member(&self, name: &str, key: &[u8]) -> Result<&Self, String> {
        if let Self::Object(members) = self
            && let Some((_, value)) = members.iter().find(|(member, _)| member.as_ref() == key)
        {
            return Ok(value);
        }
        Err(format!("{name}: lacks {} member", String::from_utf8_lossy(key)))
    }
    fn member_string(&self, name: &str, key: &[u8]) -> Result<Box<[u8]>, String> {
        if let Self::Object(members) = self
            && let Some((_, Self::String(value))) = members.iter().find(|(member, _)| member.as_ref() == key)
        {
            return Ok(value.clone());
        }
        Err(format!("{name}: lacks {} string member", String::from_utf8_lossy(key)))
    }
    fn render(&mut self, name: &str, place: &Place) -> Result<(), String> {
        match self {
            Self::Object(members) => {
                for (_, value) in members {
                    value.render(name, place)?;
                }
            }
            Self::Array(items) => {
                for value in items {
                    value.render(name, place)?;
                }
            }
            Self::String(text) => *text = render_string(name, text, place)?,
            Self::Number(_) | Self::Boolean(_) | Self::Null => {}
        }
        Ok(())
    }
    fn merge(&mut self, patch: Self) {
        let Self::Object(patch) = patch else {
            *self = patch;
            return;
        };
        if !matches!(self, Self::Object(_)) {
            *self = Self::Object(vec![]);
        }
        let Self::Object(members) = self else { unreachable!("merge target is now an object") };
        for (key, patch) in patch {
            if matches!(patch, Self::Null) {
                members.retain(|(name, _)| name != &key);
            } else if let Some((_, target)) = members.iter_mut().find(|(name, _)| name == &key) {
                target.merge(patch);
            } else {
                let mut target = Self::Null;
                target.merge(patch);
                members.push((key, target));
            }
        }
    }
    fn encode(&self, name: &str) -> Result<Box<[u8]>, String> {
        let limits = writer::Limits { depth: JSON.depth, length: u32::MAX };
        let mut out = writer::Encoder::measure(&limits);
        self.write(&mut out);
        let length = out.measured().map_err(|why| format!("{name}: cannot write JSON: {why:?}"))?;
        let mut out = writer::Encoder::write(length, &limits);
        self.write(&mut out);
        Ok(out.finish())
    }
    fn write(&self, out: &mut writer::Encoder) {
        match self {
            Self::Object(members) => {
                out.object_start();
                for (key, value) in members {
                    out.key(key);
                    value.write(out);
                }
                out.object_end();
            }
            Self::Array(items) => {
                out.array_start();
                for item in items {
                    item.write(out);
                }
                out.array_end();
            }
            Self::String(text) => out.string(text),
            Self::Number(text) => out.number(text),
            Self::Boolean(value) => out.boolean(*value),
            Self::Null => out.null(),
        }
    }
}
fn render_string(name: &str, text: &[u8], place: &Place) -> Result<Box<[u8]>, String> {
    let mut rest = text;
    let mut out = vec![];
    let prefix = b"{place.";
    while let Some(at) = rest.windows(prefix.len()).position(|part| part == prefix) {
        out.extend_from_slice(&rest[..at]);
        rest = &rest[at + prefix.len()..];
        let end = rest
            .iter()
            .position(|byte| *byte == b'}')
            .ok_or_else(|| format!("{name}: unclosed place name {}", String::from_utf8_lossy(rest)))?;
        let key = String::from_utf8_lossy(&rest[..end]);
        if key == "bundle" && place.bundle.is_none() {
            return Err(format!("{name}: place name bundle needs --bundle"));
        }
        let value = place.value(&key).ok_or_else(|| format!("{name}: unknown place name {key}"))?;
        out.extend_from_slice(&value);
        rest = &rest[end + 1..];
    }
    out.extend_from_slice(rest);
    Ok(out.into())
}

#[cfg(test)]
mod tests;
