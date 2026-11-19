//! The one key table (temper/docs/design/protocol/configuration/file.md, 3–5).
use crate::Shape;
use Shape::{EngineAlone as E, Standalone as S, WorkerAlone as W};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Set {
    Public,
    Internal,
    Test,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Table,
    Members,
    Records,
    Socket,
    Origin,
    Address,
    Name,
    Visible(usize),
    DataPath,
    AbsolutePath,
    ConfigPath,
    Login,
    Forge,
    Addresses,
    Redirect,
    Subject,
    Environment,
    Endpoint,
    Api,
    Base,
    Transport,
    Services,
}
impl Form {
    #[must_use]
    pub fn words(self) -> String {
        match self {
            Self::Table => "a table".into(),
            Self::Members => "an array of 1 to 64 member tables".into(),
            Self::Records => "an array of tables".into(),
            Self::Socket => "a socket address on loopback, such as \"127.0.0.1:7100\"".into(),
            Self::Origin => "an origin within 267 bytes; the start checks its form".into(),
            Self::Address => "an address of 3 to 254 visible ASCII bytes with one @".into(),
            Self::Name => "1 to 32 bytes".into(),
            Self::Visible(bound) => format!("1 to {bound} visible ASCII bytes"),
            Self::DataPath => "a nonempty data path, no NUL or relative .. component, at most 4096 bytes".into(),
            Self::ConfigPath => {
                "a nonempty configuration path, no NUL or relative .. component, at most 4096 bytes".into()
            }
            Self::AbsolutePath => "an absolute path, no NUL, at most 1048576 bytes".into(),
            Self::Login => {
                "a login of 1 to 40 ASCII letters, digits, -, _ or ., starting and ending alphanumeric".into()
            }
            Self::Forge => "a forge number of 1 to 65535".into(),
            Self::Addresses => "a list containing only 1 (printed link) or 3 (Google)".into(),
            Self::Redirect => "a redirect URI within 2048 bytes; the start checks its form".into(),
            Self::Subject => "a mailto: or https: subject, at most 256 visible ASCII bytes".into(),
            Self::Environment => "a table of strings; names nonempty without = or NUL, values without NUL".into(),
            Self::Endpoint => "an HTTP(S) URL within 2048 bytes; HTTP only to loopback".into(),
            Self::Api => "a socket address or HOST:PORT".into(),
            Self::Base => "an HTTP(S) URL base with no final slash, at most 4096 bytes".into(),
            Self::Transport => "\"plaintext\" on loopback, or { server_name = \"...\", trust = \"...\" }".into(),
            Self::Services => "1 to 8 origins within 1024 bytes; HTTP only to loopback".into(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Required {
    No,
    Parent,
    Engine,
    Worker,
}
#[derive(Debug)]
pub struct Row {
    pub key: &'static str,
    pub set: Set,
    pub shapes: &'static [Shape],
    pub form: Form,
    pub default: &'static str,
    pub line: &'static str,
    /// Array entries use `[]`; Keys substitutes their indices.
    pub members: &'static [&'static str],
    pub required: Required,
}

macro_rules! row {
    ($key:literal, $set:ident, $shapes:expr, $form:expr, $default:literal, $line:literal, $members:expr, $required:ident) => {
        Row {
            key: $key,
            set: Set::$set,
            shapes: $shapes,
            form: $form,
            default: $default,
            line: $line,
            members: $members,
            required: Required::$required,
        }
    };
}
use Form::{
    AbsolutePath, Address, Addresses, Api, Base, ConfigPath, DataPath, Endpoint, Environment, Forge, Login, Members,
    Name, Origin, Records, Redirect, Services, Socket, Subject, Table, Transport, Visible,
};

pub static ROWS: &[Row] = &[
    row!(
        "listen",
        Public,
        &[S, E],
        Socket,
        "127.0.0.1:7100",
        "where the client connects",
        &["application.clients.listener"],
        No
    ),
    row!(
        "public",
        Public,
        &[S, E],
        Origin,
        "the listener's origin",
        "where people reach it",
        &["application.clients.public"],
        No
    ),
    row!(
        "webhooks",
        Public,
        &[S, E],
        Socket,
        "listen's next port (0 stays 0)",
        "the forge's webhooks' listener",
        &["application.webhooks"],
        No
    ),
    row!(
        "owner",
        Public,
        &[S, E],
        Address,
        "the user's login at localhost",
        "the first owner, read at the first start alone",
        &["engine.owners"],
        No
    ),
    row!(
        "workers",
        Public,
        &[E],
        Socket,
        "required",
        "where workers connect, on loopback",
        &["engine.hosts.listener"],
        Engine
    ),
    row!(
        "engine",
        Public,
        &[W],
        Socket,
        "required",
        "where its engine listens for workers",
        &["workers[0].engine"],
        Worker
    ),
    row!(
        "name",
        Public,
        &[W],
        Name,
        "the host's name",
        "the name its engine knows it by, if not the host's",
        &["workers[0].name"],
        No
    ),
    row!(
        "sign_in",
        Public,
        &[S, E],
        Table,
        "no Google sign-in",
        "Google sign-in and its members",
        &["application.sign_in"],
        No
    ),
    row!(
        "sign_in.google",
        Public,
        &[S, E],
        Table,
        "no Google sign-in",
        "Google's sign-in",
        &["application.sign_in.google"],
        No
    ),
    row!(
        "sign_in.google.client",
        Public,
        &[S, E],
        Visible(256),
        "required with Google",
        "Google's client",
        &["application.sign_in.google.client"],
        Parent
    ),
    row!(
        "sign_in.google.secret",
        Public,
        &[S, E],
        DataPath,
        "required with Google",
        "in the data directory, mode 0600",
        &["application.sign_in.google.secret"],
        Parent
    ),
    row!(
        "sign_in.members",
        Public,
        &[S, E],
        Members,
        "required with Google",
        "one for each member",
        &["application.sign_in.members"],
        No
    ),
    row!(
        "sign_in.members[].address",
        Public,
        &[S, E],
        Address,
        "required",
        "the member's address",
        &["application.sign_in.members[].address"],
        Parent
    ),
    row!(
        "sign_in.members[].login",
        Public,
        &[S, E],
        Login,
        "required",
        "the member's login",
        &["application.sign_in.members[].login"],
        Parent
    ),
    row!(
        "sign_in.forges",
        Public,
        &[S, E],
        Records,
        "none",
        "each Forgejo whose sign-in goes through temper",
        &["application.sign_in.forges"],
        No
    ),
    row!(
        "sign_in.forges[].forge",
        Public,
        &[S, E],
        Forge,
        "required",
        "the forge's number",
        &["application.sign_in.forges[].forge"],
        Parent
    ),
    row!(
        "sign_in.forges[].client",
        Public,
        &[S, E],
        Visible(255),
        "required",
        "the provider's client",
        &["application.sign_in.forges[].client"],
        Parent
    ),
    row!(
        "openid",
        Public,
        &[S, E],
        Table,
        "the box's provider, where provider.key exists; none on an engine",
        "temper as an OpenID provider for Forgejo's sign-in",
        &["engine.clients.provider"],
        No
    ),
    row!(
        "openid.key",
        Public,
        &[S, E],
        DataPath,
        "provider.key",
        "the provider's signing key in the data directory",
        &["engine.clients.provider.key"],
        No
    ),
    row!(
        "openid.addresses",
        Public,
        &[S, E],
        Addresses,
        "[3] with Google, else [1]",
        "the providers whose subjects are given as email",
        &["engine.clients.provider.addresses"],
        No
    ),
    row!(
        "openid.clients",
        Public,
        &[S, E],
        Records,
        "none",
        "each client of the provider",
        &["engine.clients.provider.clients"],
        No
    ),
    row!(
        "openid.clients[].id",
        Public,
        &[S, E],
        Visible(255),
        "required",
        "the client's id",
        &["engine.clients.provider.clients[].id"],
        Parent
    ),
    row!(
        "openid.clients[].secret",
        Public,
        &[S, E],
        DataPath,
        "public client",
        "the client's secret's file in the data directory",
        &["engine.clients.provider.clients[].secret"],
        No
    ),
    row!(
        "openid.clients[].redirect",
        Public,
        &[S, E],
        Redirect,
        "required",
        "the client's redirect URI",
        &["engine.clients.provider.clients[].redirect_uri"],
        Parent
    ),
    row!("push", Public, &[S, E], Table, "no push", "the deployment's push", &["application.push"], No),
    row!(
        "push.key",
        Public,
        &[S, E],
        DataPath,
        "required",
        "the push key's file in the data directory",
        &["application.push.key"],
        Parent
    ),
    row!(
        "push.subject",
        Public,
        &[S, E],
        Subject,
        "required",
        "where a push service may reach the operator",
        &["application.push.subject"],
        Parent
    ),
    row!("programs", Public, &[S, W], Table, "found or fetched", "programs by their paths", &[], No),
    row!("programs.git", Public, &[S, W], AbsolutePath, "found on PATH", "the git executable", &[], No),
    row!(
        "programs.claude",
        Public,
        &[S, W],
        AbsolutePath,
        "found on PATH or fetched",
        "the Claude executable",
        &[],
        No
    ),
    row!("programs.forgejo", Public, &[S], AbsolutePath, "fetched at set-up", "the Forgejo executable", &[], No),
    row!("programs.runner", Public, &[S], AbsolutePath, "fetched at set-up", "the Forgejo runner executable", &[], No),
    row!(
        "environment",
        Public,
        &[S, W],
        Environment,
        "PATH and HOME appended where absent",
        "one line for each variable",
        &["application.environment"],
        No
    ),
    row!("internal", Internal, &[S, E], Table, "the binary's own", "internal build inputs", &[], No),
    row!(
        "internal.bundle",
        Internal,
        &[S, E],
        AbsolutePath,
        "the binary's own bundle",
        "a client bundle of another build",
        &["application.clients.bundle"],
        No
    ),
    row!("test", Test, &[S, E], Table, "the real services", "test services", &[], No),
    row!("test.github", Test, &[S, E], Table, "github.com's", "a fake GitHub", &["application.test.github"], No),
    row!(
        "test.github.api",
        Test,
        &[S, E],
        Api,
        "required",
        "the fake GitHub's API",
        &["application.test.github.api"],
        Parent
    ),
    row!(
        "test.github.transport",
        Test,
        &[S, E],
        Transport,
        "required",
        "the fake GitHub's transport",
        &["application.test.github.transport"],
        Parent
    ),
    row!(
        "test.github.transport.server_name",
        Test,
        &[S, E],
        Visible(4096),
        "required with TLS",
        "the TLS server name",
        &["application.test.github.transport.tls.server_name"],
        Parent
    ),
    row!(
        "test.github.transport.trust",
        Test,
        &[S, E],
        ConfigPath,
        "the machine's roots",
        "the TLS trust file in the configuration directory",
        &["application.test.github.transport.tls.trust"],
        No
    ),
    row!(
        "test.github.remote",
        Test,
        &[S, E],
        Base,
        "required",
        "the fake's remote URL base",
        &["application.test.github.remote"],
        Parent
    ),
    row!(
        "test.github.web",
        Test,
        &[S, E],
        Base,
        "required",
        "the fake's web URL base",
        &["application.test.github.web"],
        Parent
    ),
    row!(
        "test.github.git",
        Test,
        &[S, E],
        DataPath,
        "the worker's git",
        "the fake's git in the data directory",
        &["application.test.github.git"],
        No
    ),
    row!(
        "test.google",
        Test,
        &[S, E],
        Table,
        "Google's",
        "a fake issuer for Google sign-in",
        &["application.sign_in.google.endpoints"],
        No
    ),
    row!(
        "test.google.issuer",
        Test,
        &[S, E],
        Endpoint,
        "required",
        "the fake issuer's URL",
        &["application.sign_in.google.endpoints.issuer"],
        Parent
    ),
    row!(
        "test.google.authorization",
        Test,
        &[S, E],
        Endpoint,
        "required",
        "the fake's authorization URL",
        &["application.sign_in.google.endpoints.authorization"],
        Parent
    ),
    row!(
        "test.google.token",
        Test,
        &[S, E],
        Endpoint,
        "required",
        "the fake's token URL",
        &["application.sign_in.google.endpoints.token"],
        Parent
    ),
    row!(
        "test.google.keys",
        Test,
        &[S, E],
        Endpoint,
        "required",
        "the fake's keys URL",
        &["application.sign_in.google.endpoints.keys"],
        Parent
    ),
    row!(
        "test.google.transport",
        Test,
        &[S, E],
        Transport,
        "required",
        "the fake issuer's transport",
        &["application.sign_in.google.endpoints.transport"],
        Parent
    ),
    row!(
        "test.google.transport.server_name",
        Test,
        &[S, E],
        Visible(4096),
        "required with TLS",
        "the TLS server name",
        &["application.sign_in.google.endpoints.transport.tls.server_name"],
        Parent
    ),
    row!(
        "test.google.transport.trust",
        Test,
        &[S, E],
        ConfigPath,
        "the machine's roots",
        "the TLS trust file in the configuration directory",
        &["application.sign_in.google.endpoints.transport.tls.trust"],
        No
    ),
    row!(
        "test.push",
        Test,
        &[S, E],
        Table,
        "the real push services",
        "fake push services",
        &["application.push.services"],
        No
    ),
    row!(
        "test.push.services",
        Test,
        &[S, E],
        Services,
        "required",
        "the fake push services' origins",
        &["application.push.services"],
        Parent
    ),
];

pub static MOVED: &[(&str, &str)] = &[
    ("slots", "a worker's places are its setting's; temper ctl worker places NAME N"),
    ("secret", "a worker's secret is secrets/worker.secret in the data directory"),
    ("limits", "limits are settings; ask temper, or use temper ctl settings apply"),
    ("memory_bytes", "limits are settings; ask temper, or use temper ctl settings apply"),
    ("agent", "temper writes the agent's configuration from the engine's figures"),
    ("welcome", "the welcome is a setting; change it with temper ctl settings"),
    ("machine", "a forge's place is chosen when it's set up, and kept with the forge"),
    ("demo", "LLM providers are settings; see them with temper ctl settings show provider"),
    ("llm", "LLM providers are settings; see them with temper ctl settings show provider"),
    ("owners", "the first owner is owner; later owners are the store's"),
    ("clients", "temper.toml is flat; {binary} help config lists its keys"),
    ("hosts", "temper.toml is flat; {binary} help config lists its keys"),
    ("application", "temper.toml is flat; {binary} help config lists its keys"),
];

pub(crate) fn find(key: &str) -> Option<&'static Row> {
    ROWS.iter().find(|row| row.key == key)
}
