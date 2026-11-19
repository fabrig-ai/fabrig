//! The file's vocabulary and reader (temper/docs/design/protocol/configuration/file.md, 2–7).
mod document;
mod machine;
mod merge;
mod pages;
mod parse;
mod render;
pub mod rows;
mod source;
mod template;

pub use machine::{facts, read};
pub use pages::{help, reference};
pub use parse::{parse, patch};
pub use render::render;
pub use source::{Keys, Refusal};
use std::{net::SocketAddr, path::PathBuf};
pub use temper_service_core::startup::Shape;
pub use template::substitute;

pub const FILE: &str = "temper.toml";
pub const FORMER: &str = "temper.json";
const FILE_BYTES: usize = 1 << 20;

/// Explicit values alone: defaults remain absent, including those inside `[openid]`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct File {
    pub listen: Option<SocketAddr>,
    pub public: Option<String>,
    pub webhooks: Option<SocketAddr>,
    pub owner: Option<String>,
    pub workers: Option<SocketAddr>,
    pub engine: Option<SocketAddr>,
    pub name: Option<String>,
    pub sign_in: Option<SignIn>,
    pub openid: Option<OpenId>,
    pub push: Option<Push>,
    pub programs: Programs,
    pub environment: Vec<(String, String)>,
    pub internal: Internal,
    pub test: Test,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignIn {
    pub client: String,
    pub secret: PathBuf,
    pub members: Vec<Member>,
    pub forges: Vec<ForgeSignIn>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub address: String,
    pub login: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForgeSignIn {
    pub forge: u16,
    pub client: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenId {
    pub key: Option<PathBuf>,
    pub addresses: Option<Vec<u16>>,
    pub clients: Vec<OpenIdClient>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenIdClient {
    pub id: String,
    pub secret: Option<PathBuf>,
    pub redirect: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Push {
    pub key: PathBuf,
    pub subject: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Programs {
    pub git: Option<PathBuf>,
    pub claude: Option<PathBuf>,
    pub forgejo: Option<PathBuf>,
    pub runner: Option<PathBuf>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Internal {
    pub bundle: Option<PathBuf>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Test {
    pub github: Option<TestGitHub>,
    pub google: Option<TestGoogle>,
    pub push_services: Option<Vec<String>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestGitHub {
    pub api: String,
    pub transport: Transport,
    pub remote: String,
    pub web: String,
    pub git: Option<PathBuf>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestGoogle {
    pub issuer: String,
    pub authorization: String,
    pub token: String,
    pub keys: String,
    pub transport: Transport,
}
/// TOML's plain transport becomes the decoder's tagged TLS object in the start's document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transport {
    Plaintext,
    Tls { server_name: String, trust: Option<PathBuf> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    pub login: String,
    pub hostname: String,
    pub provider_key: bool,
}
#[derive(Debug)]
pub struct Read {
    pub file: File,
    pub document: Box<[u8]>,
    pub keys: Keys,
}

/// A deterministic standalone start for shared test harnesses. The file goes
/// through the same renderer and reader as init's output (file.md, 4 and 7).
#[must_use]
pub fn init_document(provider_key: bool) -> Box<[u8]> {
    let file = File {
        listen: Some(([127, 0, 0, 1], 8367).into()),
        webhooks: Some(([127, 0, 0, 1], 8371).into()),
        ..File::default()
    };
    let facts = Facts { login: "ada".into(), hostname: "fixture".into(), provider_key };
    parse(&render(&file, Shape::Standalone), Shape::Standalone, &facts)
        .expect("the fixture's init file is valid")
        .document
}

pub(crate) const fn binary(shape: Shape) -> &'static str {
    match shape {
        Shape::Standalone => "temper-box",
        Shape::EngineAlone => "temper-engine",
        Shape::WorkerAlone => "temper-worker",
    }
}

#[cfg(test)]
mod fuzzy;
#[cfg(test)]
mod tests;
