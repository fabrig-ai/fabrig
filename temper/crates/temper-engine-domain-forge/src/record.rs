//! Durable forge configuration (temper/docs/design/draft/playground-reset.md, 3;
//! temper/docs/plan/playground-reset-r4/notes/contracts.md, 2).
use alloc::boxed::Box;

/// One forge the connector keeps (draft 3; contracts, 2), what the
/// document's `forges[]`, `identities[]` and `setup[]` held for it,
/// less its secrets.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Forge {
    pub number: u16,
    pub kind: crate::Kind,
    pub api: Address,
    pub transport: Transport,
    /// The base of each remote (forges[].git).
    pub git: Box<[u8]>,
    pub git_program: Option<Box<[u8]>>,
    pub web: Option<Box<[u8]>>,
    /// `forges[].app_read_only`.
    pub read_only: bool,
    pub identity: Identity,
    /// The file form's names, none for a records-only forge.
    pub files: Option<Files>,
    /// The forge's setup entry, none for a connected forge.
    pub setup: Option<Setup>,
    /// The first repository's intent, kept until it is made.
    pub first: Option<FirstIntent>,
    /// The playground's fake forge alone.
    pub playground: bool,
    /// Ended, its kept acts not all answered (D3).
    pub removing: bool,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Address {
    Socket(core::net::SocketAddr),
    Host { name: Box<[u8]>, port: u16 },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Transport {
    Plaintext,
    Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
}
/// identities[]: the git identity's number and its login and author,
/// each empty where the document's is null (the bot's defaults).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Identity {
    pub number: u32,
    pub login: Box<[u8]>,
    pub name: Box<[u8]>,
    pub email: Box<[u8]>,
}
/// The file form's names (never the files' bytes): `forges[].api_token`,
/// `webhook_secret`, user, and `identities[].token`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Files {
    pub api_token: Box<[u8]>,
    pub webhook_secret: Option<Box<[u8]>>,
    pub user: FileUser,
    pub git_token: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FileUser {
    Id(u64),
    File(Box<[u8]>),
}
/// One setup[] entry less its `forge` (protocol/setup.md, 8), as
/// `temper_forge_setup::setup::Entry` holds it, paths as resolved.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Setup {
    pub directory: Box<[u8]>,
    pub bot: Account,
    pub orgs: Box<[Box<[u8]>]>,
    pub mode: SetupMode,
    /// `"start": "asked"`.
    pub asked: bool,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Account {
    pub login: Box<[u8]>,
    pub email: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SetupMode {
    Managed { admin_token: Option<Box<[u8]>> },
    Owned(OwnedEntry),
    Fake(FakeEntry),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OwnedEntry {
    pub public: Option<Origin>,
    pub fetch: Fetch,
    pub forgejo: Box<[u8]>,
    pub runner: Option<Runner>,
    pub listen: core::net::SocketAddr,
    pub person: Account,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Fetch {
    pub forgejo: bool,
    pub runner: bool,
    pub mirror: Option<Mirror>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Mirror {
    pub forgejo: Origin,
    pub runner: Origin,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Origin {
    pub host: Box<[u8]>,
    pub port: u16,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Runner {
    pub binary: Box<[u8]>,
    pub labels: Box<[Box<[u8]>]>,
    pub capacity: u32,
    pub path: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FakeEntry {
    pub listen: core::net::SocketAddr,
    pub git: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub repositories: Box<[FakeRepository]>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FakeRepository {
    pub name: Box<[u8]>,
    pub landing: Box<[u8]>,
}
/// `crate::First` less its volatile state, and less `playground`,
/// which the forge's own flag holds.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FirstIntent {
    pub org: Box<[u8]>,
    pub repository: Box<[u8]>,
    pub project: crate::SetUpProject,
    pub label: Box<[u8]>,
}

/// Schema ceiling for API and origin host names, TLS names and account fields.
pub const HOST_BYTES: usize = 255;
/// Schema ceiling for paths, URLs and setup names.
pub const PATH_BYTES: usize = 4096;
/// Schema ceiling for setup organizations.
pub const SETUP_ORGS: usize = 1024;
/// Schema ceiling for runner labels.
pub const RUNNER_LABELS: usize = 8;
/// Schema ceiling for fake repositories.
pub const FAKE_REPOSITORIES: usize = 64;
/// Schema ceiling for first repository, organization and project names.
pub const FIRST_BYTES: usize = 1024;
/// Schema ceiling for the first repository's label.
pub const LABEL_BYTES: usize = 64;

fn add(total: &mut u64, value: &[u8], bound: usize, check: bool) -> Option<()> {
    if check && value.len() > bound {
        return None;
    }
    *total = total.checked_add(u64::try_from(value.len()).ok()?)?;
    Some(())
}
fn optional(total: &mut u64, value: Option<&[u8]>, bound: usize, check: bool) -> Option<()> {
    if let Some(value) = value {
        add(total, value, bound, check)?;
    }
    Some(())
}
fn account(total: &mut u64, value: &Account, check: bool) -> Option<()> {
    add(total, &value.login, HOST_BYTES, check)?;
    add(total, &value.email, HOST_BYTES, check)
}
fn origin(total: &mut u64, value: &Origin, check: bool) -> Option<()> {
    add(total, &value.host, HOST_BYTES, check)
}
fn setup(total: &mut u64, value: &Setup, check: bool) -> Option<()> {
    add(total, &value.directory, PATH_BYTES, check)?;
    account(total, &value.bot, check)?;
    if check && value.orgs.len() > SETUP_ORGS {
        return None;
    }
    for name in &value.orgs {
        add(total, name, PATH_BYTES, check)?;
    }
    match &value.mode {
        SetupMode::Managed { admin_token } => optional(total, admin_token.as_deref(), PATH_BYTES, check)?,
        SetupMode::Owned(value) => {
            if let Some(public) = &value.public {
                origin(total, public, check)?;
            }
            if let Some(mirror) = &value.fetch.mirror {
                origin(total, &mirror.forgejo, check)?;
                origin(total, &mirror.runner, check)?;
            }
            add(total, &value.forgejo, PATH_BYTES, check)?;
            if let Some(runner) = &value.runner {
                add(total, &runner.binary, PATH_BYTES, check)?;
                add(total, &runner.path, PATH_BYTES, check)?;
                if check && runner.labels.len() > RUNNER_LABELS {
                    return None;
                }
                for label in &runner.labels {
                    add(total, label, PATH_BYTES, check)?;
                }
            }
            account(total, &value.person, check)?;
        }
        SetupMode::Fake(value) => {
            add(total, &value.git, PATH_BYTES, check)?;
            add(total, &value.owner, PATH_BYTES, check)?;
            if check && value.repositories.len() > FAKE_REPOSITORIES {
                return None;
            }
            for repository in &value.repositories {
                add(total, &repository.name, PATH_BYTES, check)?;
                add(total, &repository.landing, PATH_BYTES, check)?;
            }
        }
    }
    Some(())
}
impl Forge {
    /// Whether every retained field fits the store schema's ceilings.
    #[must_use]
    pub fn fits(&self) -> bool {
        self.measured(true).is_some()
    }

    pub(crate) fn bytes(&self) -> Option<u64> {
        self.measured(false)
    }

    fn measured(&self, check: bool) -> Option<u64> {
        let mut total = 0;
        match &self.api {
            Address::Host { name, .. } => add(&mut total, name, HOST_BYTES, check)?,
            Address::Socket(_) => {}
        }
        match &self.transport {
            Transport::Tls { server_name, trust } => {
                add(&mut total, server_name, HOST_BYTES, check)?;
                optional(&mut total, trust.as_deref(), PATH_BYTES, check)?;
            }
            Transport::Plaintext => {}
        }
        add(&mut total, &self.git, PATH_BYTES, check)?;
        optional(&mut total, self.git_program.as_deref(), PATH_BYTES, check)?;
        optional(&mut total, self.web.as_deref(), PATH_BYTES, check)?;
        add(&mut total, &self.identity.login, HOST_BYTES, check)?;
        add(&mut total, &self.identity.name, HOST_BYTES, check)?;
        add(&mut total, &self.identity.email, HOST_BYTES, check)?;
        if let Some(files) = &self.files {
            add(&mut total, &files.api_token, PATH_BYTES, check)?;
            optional(&mut total, files.webhook_secret.as_deref(), PATH_BYTES, check)?;
            optional(&mut total, files.git_token.as_deref(), PATH_BYTES, check)?;
            match &files.user {
                FileUser::File(path) => add(&mut total, path, PATH_BYTES, check)?,
                FileUser::Id(_) => {}
            }
        }
        if let Some(value) = &self.setup {
            setup(&mut total, value, check)?;
        }
        if let Some(first) = &self.first {
            add(&mut total, &first.org, FIRST_BYTES, check)?;
            add(&mut total, &first.repository, FIRST_BYTES, check)?;
            add(&mut total, &first.label, LABEL_BYTES, check)?;
            match &first.project {
                crate::SetUpProject::New(name) => add(&mut total, name, FIRST_BYTES, check)?,
                crate::SetUpProject::Existing(_) => {}
            }
        }
        Some(total)
    }
}
impl crate::Configured {
    /// Derive the connector's startup configuration from one durable forge.
    #[must_use]
    pub fn from_record(forge: &Forge) -> Self {
        use crate::{Mode, Owned};
        let mut configured = Self {
            number: forge.number,
            kind: forge.kind,
            mode: Mode::Connected,
            admin_connect: false,
            bot: Box::new([]),
            owned: None,
            asked: forge.playground,
            git: if forge.git.len() <= usize::try_from(crate::PULL_ADDRESS_BYTES).expect("u32 fits usize") {
                forge.git.clone()
            } else {
                Box::new([])
            },
            web: forge.web.clone(),
            read_only: forge.read_only,
        };
        if let Some(setup) = &forge.setup {
            configured.bot.clone_from(&setup.bot.login);
            configured.asked |= setup.asked;
            configured.mode = match &setup.mode {
                SetupMode::Managed { admin_token } => {
                    configured.admin_connect = admin_token.is_none();
                    Mode::Managed
                }
                SetupMode::Owned(value) => {
                    use skein_forge::forgejo::pin;
                    configured.owned = Some(Owned {
                        forgejo: pin::VERSION.as_bytes().into(),
                        runner: value.runner.is_some().then_some(pin::RUNNER_VERSION.as_bytes().into()),
                        password: password_path(&setup.directory),
                    });
                    Mode::Owned
                }
                SetupMode::Fake(_) => Mode::Fake,
            };
        }
        configured
    }
}

fn password_path(directory: &[u8]) -> Box<[u8]> {
    let mut writer =
        skein_lib::Writer::new(directory.len().checked_add(b"/person_password".len()).expect("path length"));
    writer.put(directory).expect("measured path");
    writer.put(b"/person_password").expect("measured path");
    writer.finish()
}
