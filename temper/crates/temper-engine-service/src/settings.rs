//! Typed engine settings on the core's shared settings (protocol/services.md, 4).
use alloc::boxed::Box;
use skein_lib::{List, Secret};

/// The scripted provider and its key file (temper/docs/design/protocol/setup/playground.md).
#[derive(Debug)]
pub struct Demo {
    pub provider: u32,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub cues: Box<[Box<[u8]>]>,
    pub landing: Box<[u8]>,
    pub key_file: Box<[u8]>,
}

/// A runtime forge row made from its kept record (temper/docs/design/protocol/forge/engine.md, 1).
#[derive(Clone, Debug)]
pub struct Forge {
    /// Resolved configured job-log origins; none uses the kind's runtime defaults
    /// (temper/docs/design/protocol/configuration/engine.md).
    pub log_origins: Option<List<skein_forge_connection::settings::LogOrigin>>,
    pub mode: temper_engine_domain_forge::Mode,
    pub number: u16,
    pub kind: Kind,
    /// Where the forge component connects, never an in-process name.
    pub api: jig_protocol_config::Address,
    pub transport: Transport,
    /// The base of each remote and the host of the forge's resource paths.
    pub git: Box<[u8]>,
    /// The forge's resolved git program, in place of the worker's git.
    pub git_program: Option<Box<[u8]>>,
    pub web: Option<Box<[u8]>>,
    /// The pasted GitHub App, its hook and installations belong to another owner.
    pub app_read_only: bool,
    /// The forge user the API token belongs to: the forge writer.
    pub user: u64,
    /// The number of this forge's git identity.
    pub identity: u32,
}

/// The documents a forge speaks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Forgejo,
    GitHub,
}

/// A forge component's connections.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Transport {
    /// Only to a socket address on loopback.
    Plaintext,
    /// The trust file's bytes, or none for the machine's roots.
    Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
}

/// A runtime git identity made from its kept forge (temper/docs/design/protocol/forge/engine.md, 1).
#[derive(Clone, Debug)]
pub struct Identity {
    pub number: u32,
    pub forge: u16,
    pub token: Secret,
    pub login: Box<[u8]>,
    pub author: Author,
}

/// The author and committer of the commits made with an identity.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Author {
    pub name: Box<[u8]>,
    pub email: Box<[u8]>,
}

pub use temper_service_core::settings::{Adapter, Files, PROGRAM_PATH_BYTES, Program};

/// The seeds the shell draws at start, as jig's reference takes them.
#[derive(Debug)]
pub struct Seeds {
    pub deployment: [u8; 16],
    pub seed: u64,
    pub crypto: Secret,
    /// The first link's secret supplied by the box (temper/docs/design/protocol/setup/box.md, 1).
    pub link_secret: Option<Secret>,
    /// The local link's first secret supplied by the box (temper/docs/design/protocol/setup/box.md, 1).
    pub local_secret: Option<Secret>,
    /// The installation's identifier drawn for a first start; a later start
    /// sends the one its store keeps (jig's configuration.md, 7).
    pub installation: [u8; 16],
}

/// One setup fixture entry and its handover (protocol/setup/playground.md).
#[cfg(test)]
#[derive(Clone, Debug)]
pub struct Setup {
    pub entry: temper_forge_setup::setup::Entry,
    /// The private handover supplied to `TakeOver`.
    pub handover: Option<Secret>,
}

/// The engine's settings: jig's part, typed by jig-protocol-config, and
/// temper's members (protocol/configuration/README.md, section 2 and protocol/configuration/engine.md).
#[derive(Debug)]
pub struct Engine {
    /// Checked receiving agent reply allowance; none when no local agent is configured.
    pub host_reply_bytes: Option<u32>,
    /// The local agent's charter allowance, smith's `run_bytes`; none when no local agent is configured.
    pub agent_charter_bytes: Option<u64>,
    /// Charter brief-room costs of the configured agent (temper/docs/design/domain/agent/tools.md).
    pub host_part: Option<temper_engine_domain::room::HostPart>,
    pub machine: temper_forge_setup::machine::Machine,
    pub directories: jig_protocol_config::Directories,
    pub jig: jig_protocol_config::Engine,
    pub clients: crate::startup::application::Clients,
    pub sign_in: Option<SignIn>,
    pub push: Option<Push>,
    pub webhooks: Option<skein_forge_connection::hooks::Webhooks>,
    pub files: Files,
    pub demo: Option<Demo>,
    pub welcome: bool,
    pub program_environment: Box<[u8]>,
    pub ways_stock: Box<[temper_engine_domain_ways::StockWay]>,
    pub kinds_stock: Box<[temper_engine_domain_forge::settings::Stock]>,
    pub landings: skein_lib::Map<u32, temper_engine_domain::ProjectLanding>,
    pub limits: super::limits::Limits,
}

pub use temper_service_core::agent::Declaration as Agent;

/// Checked push key and destinations (domain/forge/attention.md, 7.3).
#[derive(Debug)]
pub struct Push {
    pub key: skein_push::Vapid,
    pub subject: Box<[u8]>,
    pub services: Box<[Box<[u8]>]>,
}

/// Checked Google registration; the client secret is private startup input.
#[derive(Debug)]
pub struct SignIn {
    pub provider: u16,
    pub client: Box<[u8]>,
    pub secret: Secret,
    pub issuer: Box<[u8]>,
    pub authorization: Box<[u8]>,
    pub token: Box<[u8]>,
    pub keys: Box<[u8]>,
    pub transport: Option<Transport>,
    pub members: Box<[crate::startup::application::sign_in::Member]>,
    pub forges: Box<[crate::startup::application::sign_in::ForgeSignIn]>,
}
