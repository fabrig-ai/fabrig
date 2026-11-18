//! The box's scripted provider and embedded scripts (temper/docs/design/protocol/configuration/providers.md, 4).
use crate::{Refusal, llm, read};
use alloc::boxed::Box;
use core::net::SocketAddr;
use jig_protocol_config::{self as config, Directory, Path};
use skein_fake_llm_domain::api;
use skein_lib::{List, bytes};

pub(crate) const SCRIPTED_WINDOW: u32 = 1_048_576;
pub(crate) const SCRIPTED_OUTPUT: u32 = 8192;

/// The key file's read bound: the largest account key and its newline.
/// jig's client-v11 Keyed.value ceiling is 4096 bytes; account keys are raw bytes.
pub const KEY_FILE_BYTES: u32 = 4097;

/// Why the demo key cannot be added as an account's key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyRefused {
    Empty,
    TooLong,
}
impl KeyRefused {
    #[must_use]
    pub fn reason(self) -> &'static str {
        match self {
            Self::Empty => "holds no key",
            Self::TooLong => "key past 4096 bytes",
        }
    }
}
/// A demo key file's key: its bytes less a final newline (protocol/setup/playground.md).
pub fn key(file: &[u8]) -> Result<&[u8], KeyRefused> {
    let key = match file.strip_suffix(b"\n") {
        Some(key) => key,
        None => file,
    };
    if key.is_empty() {
        return Err(KeyRefused::Empty);
    }
    if key.len() > usize::try_from(KEY_FILE_BYTES - 1).expect("key bound") {
        return Err(KeyRefused::TooLong);
    }
    Ok(key)
}

/// The binary's scripts at the checked scripted provider's endpoint (protocol/setup/playground.md).
#[derive(Debug)]
pub struct Demo {
    llm: llm::Document,
    provider: u32,
}
impl Demo {
    #[must_use]
    pub fn llm(&self) -> &llm::Document {
        &self.llm
    }
    #[must_use]
    pub fn provider(&self) -> u32 {
        self.provider
    }
}
/// What the demo is checked against, all from the checked document.
/// A borrowed view used only during this read; no service state retains it.
#[derive(Debug)]
pub struct Facts<'a> {
    pub llm: &'a config::Llm,
    pub listeners: &'a [SocketAddr],
    pub directories: &'a config::Directories,
}
/// The box's scripted provider, when its checked catalogue holds it.
pub fn read(facts: &Facts<'_>) -> Result<Option<Demo>, Refusal> {
    let p = Path::root().member(b"demo");
    let mut found = None;
    for provider in &facts.llm.providers {
        if provider.number == 3 && provider.name.as_ref() == b"demo" {
            found = Some(provider);
        }
    }
    let Some(provider) = found else { return Ok(None) };
    let mut listen = None;
    for endpoint in &facts.llm.endpoints {
        if Some(&endpoint.name) == provider.endpoint.as_ref() {
            listen = match &endpoint.address {
                config::EndpointAddress::Socket(socket) => Some(*socket),
                config::EndpointAddress::Host { .. } => None,
            };
        }
    }
    let listen = listen.ok_or(read::why(&p, b"must name a loopback socket address"))?;
    if !listen.ip().is_loopback() {
        return Err(read::why(&p, b"must name a loopback socket address"));
    }
    unique_listen(listen, facts, &p)?;
    let models = models(3, listen, facts.llm, &p)?;
    let key = match facts.directories.join(Directory::Data, b"demo/llm.key") {
        Ok(path) => path,
        Err(what) => return Err(read::kind(&p, what)),
    };
    let llm = llm::Document::checked(listen, key, None, models, scripts()?, &p)?;
    Ok(Some(Demo { llm, provider: 3 }))
}

/// The playground's scripts, embedded and checked as protocol/setup/playground.md requires.
#[expect(clippy::disallowed_macros, reason = "the playground's scripts are the binary's constant")]
pub const SCRIPTS: &[u8] = include_bytes!("demo/scripts.json");

pub fn scripts() -> Result<api::Scripts, Refusal> {
    decode_scripts(SCRIPTS)
}

pub(crate) fn decode_scripts(bytes: &[u8]) -> Result<api::Scripts, Refusal> {
    let document = read::document(bytes)?;
    let document = &document;
    let lp = Path::root().member(b"demo");
    let mut o = read::object(document, 0, &lp)?;
    let asked_at = read::required(&mut o, document, b"asked", &lp)?;
    let cued_at = o.member(document, b"cued");
    let otherwise_at = read::required(&mut o, document, b"otherwise", &lp)?;
    read::finish(o, document, &lp)?;
    let asked = llm::scripts(document, asked_at, &lp.member(b"asked"))?;
    let cp = lp.member(b"cued");
    let cued = match cued_at {
        Some(at) => llm::scripts(document, at, &cp)?,
        None => Box::new([]),
    };
    let count = asked.len().checked_add(cued.len()).expect("two bounded script lists");
    if count > usize::try_from(crate::ENTRIES).expect("sixteen scripts") {
        return Err(read::why(&cp, b"past its bound"));
    }
    for (index, script) in cued.iter().enumerate() {
        for previous in &asked {
            if previous.cue == script.cue {
                return Err(read::why(&read::index(&cp, index).member(b"cue"), b"must be unique"));
            }
        }
    }
    let scripts =
        api::Scripts { cued, asked, otherwise: llm::turns(document, otherwise_at, &lp.member(b"otherwise"))? };
    llm::check_scripts(&scripts, &lp.member(b"otherwise"))?;
    Ok(scripts)
}
fn unique_listen(listen: SocketAddr, facts: &Facts<'_>, p: &Path) -> Result<(), Refusal> {
    if facts.listeners.contains(&listen) {
        return Err(read::why(p, b"must be unique"));
    }
    Ok(())
}
fn models(number: u32, listen: SocketAddr, llm: &config::Llm, p: &Path) -> Result<Box<[llm::Model]>, Refusal> {
    let mut found = None;
    for provider in &llm.providers {
        if provider.number == number {
            found = Some(provider);
        }
    }
    let Some(provider) = found else {
        return Err(read::why(p, b"names no LLM provider"));
    };
    if !provider.keys || provider.sign_in.is_some() || !provider.programs.is_empty() || provider.signed_in_plan {
        return Err(read::why(p, b"must offer a key alone"));
    }
    let mut models = List::with_capacity(u32::try_from(llm.models.len()).expect("catalogue model count"));
    for entry in &llm.models {
        let model = &entry.model;
        if model.provider != number {
            continue;
        }
        let mut found = None;
        for endpoint in &llm.endpoints {
            if endpoint.number == model.endpoint {
                found = Some(endpoint);
            }
        }
        let endpoint = found.expect("checked model's endpoint");
        let socket = match &endpoint.address {
            config::EndpointAddress::Socket(socket) => Some(*socket),
            config::EndpointAddress::Host { .. } => None,
        };
        if socket != Some(listen) {
            return Err(read::why(p, b"names an endpoint not at llm.listen"));
        }
        match &endpoint.transport {
            config::Transport::Tls { .. } => return Err(read::why(p, b"names an endpoint with TLS")),
            config::Transport::Plaintext => {}
        }
        // Dialect 3 is OpenAI's public Responses route (protocol/configuration/README.md, section 2).
        if endpoint.dialect != 3 {
            return Err(read::why(p, b"names an endpoint not in OpenAI's dialect"));
        }
        let (window, output) = if entry.number == 8 && model.provider == 3 && model.name.as_ref() == b"demo" {
            (SCRIPTED_WINDOW, SCRIPTED_OUTPUT)
        } else {
            (model.window, model.output)
        };
        models.push(llm::Model { name: bytes::copy_of(&model.name), window, output }).expect("catalogue model count");
    }
    if models.is_empty() {
        return Err(read::why(p, b"names a provider with no models"));
    }
    llm::check_models(models.as_slice(), p, false)?;
    Ok(models.into_boxed())
}
