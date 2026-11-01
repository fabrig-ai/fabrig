//! Forge configuration and retained reset rows (contracts, section 2).
use super::{FITS, d, w};
use alloc::boxed::Box;
use core::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use d::record as r;
use skein_lib::List;

fn encode_identity(v: &r::Identity) -> w::ForgeIdentity {
    w::ForgeIdentity::new(
        &w::CEILINGS,
        w::ForgeIdentityParts {
            number: v.number,
            login: v.login.clone(),
            name: v.name.clone(),
            email: v.email.clone(),
        },
    )
    .expect(FITS)
}

fn decode_identity(v: w::ForgeIdentity) -> r::Identity {
    let w::ForgeIdentityParts { number, login, name, email } = v.into_parts();
    r::Identity { number, login, name, email }
}

fn encode_files(v: &r::Files) -> w::ForgeFiles {
    w::ForgeFiles::new(
        &w::CEILINGS,
        w::ForgeFilesParts {
            api_token: v.api_token.clone(),
            webhook_secret: v.webhook_secret.clone(),
            user: encode_user(&v.user),
            git_token: v.git_token.clone(),
        },
    )
    .expect(FITS)
}

fn decode_files(v: w::ForgeFiles) -> r::Files {
    let w::ForgeFilesParts { api_token, webhook_secret, user, git_token } = v.into_parts();
    r::Files { api_token, webhook_secret, user: decode_user(user), git_token }
}

fn encode_account(v: &r::Account) -> w::SetupAccount {
    w::SetupAccount::new(&w::CEILINGS, w::SetupAccountParts { login: v.login.clone(), email: v.email.clone() })
        .expect(FITS)
}

fn decode_account(v: w::SetupAccount) -> r::Account {
    let w::SetupAccountParts { login, email } = v.into_parts();
    r::Account { login, email }
}

fn encode_origin(v: &r::Origin) -> w::SetupOrigin {
    w::SetupOrigin::new(&w::CEILINGS, w::SetupOriginParts { host: v.host.clone(), port: v.port }).expect(FITS)
}

fn decode_origin(v: w::SetupOrigin) -> r::Origin {
    let w::SetupOriginParts { host, port } = v.into_parts();
    r::Origin { host, port }
}

fn encode_mirror(v: &r::Mirror) -> w::SetupMirror {
    w::SetupMirror::new(
        &w::CEILINGS,
        w::SetupMirrorParts { forgejo: encode_origin(&v.forgejo), runner: encode_origin(&v.runner) },
    )
    .expect(FITS)
}

fn decode_mirror(v: w::SetupMirror) -> r::Mirror {
    let w::SetupMirrorParts { forgejo, runner } = v.into_parts();
    r::Mirror { forgejo: decode_origin(forgejo), runner: decode_origin(runner) }
}

#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn encode_fetch(v: &r::Fetch) -> w::SetupFetch {
    w::SetupFetch::new(
        &w::CEILINGS,
        w::SetupFetchParts {
            forgejo: v.forgejo,
            runner: v.runner,
            mirror: match &v.mirror {
                Some(v) => Some(encode_mirror(v)),
                None => None,
            },
        },
    )
    .expect(FITS)
}

#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn decode_fetch(v: w::SetupFetch) -> r::Fetch {
    let w::SetupFetchParts { forgejo, runner, mirror } = v.into_parts();
    r::Fetch {
        forgejo,
        runner,
        mirror: match mirror {
            Some(v) => Some(decode_mirror(v)),
            None => None,
        },
    }
}

fn encode_runner(v: &r::Runner) -> w::SetupRunner {
    w::SetupRunner::new(
        &w::CEILINGS,
        w::SetupRunnerParts {
            binary: v.binary.clone(),
            labels: encode_names(&v.labels),
            capacity: v.capacity,
            path: v.path.clone(),
        },
    )
    .expect(FITS)
}

fn decode_runner(v: w::SetupRunner) -> r::Runner {
    let w::SetupRunnerParts { binary, labels, capacity, path } = v.into_parts();
    r::Runner { binary, labels: decode_names(labels), capacity, path }
}

#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn encode_owned(v: &r::OwnedEntry) -> w::SetupOwned {
    w::SetupOwned::new(
        &w::CEILINGS,
        w::SetupOwnedParts {
            public: match &v.public {
                Some(v) => Some(encode_origin(v)),
                None => None,
            },
            fetch: encode_fetch(&v.fetch),
            forgejo: v.forgejo.clone(),
            runner: match &v.runner {
                Some(v) => Some(encode_runner(v)),
                None => None,
            },
            listen: encode_socket(v.listen),
            person: encode_account(&v.person),
        },
    )
    .expect(FITS)
}

#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
fn decode_owned(v: w::SetupOwned) -> r::OwnedEntry {
    let w::SetupOwnedParts { public, fetch, forgejo, runner, listen, person } = v.into_parts();
    r::OwnedEntry {
        public: match public {
            Some(v) => Some(decode_origin(v)),
            None => None,
        },
        fetch: decode_fetch(fetch),
        forgejo,
        runner: match runner {
            Some(v) => Some(decode_runner(v)),
            None => None,
        },
        listen: decode_socket(listen),
        person: decode_account(person),
    }
}

fn encode_repository(v: &r::FakeRepository) -> w::SetupRepository {
    w::SetupRepository::new(&w::CEILINGS, w::SetupRepositoryParts { name: v.name.clone(), landing: v.landing.clone() })
        .expect(FITS)
}

fn decode_repository(v: w::SetupRepository) -> r::FakeRepository {
    let w::SetupRepositoryParts { name, landing } = v.into_parts();
    r::FakeRepository { name, landing }
}

fn encode_fake(v: &r::FakeEntry) -> w::SetupFake {
    w::SetupFake::new(
        &w::CEILINGS,
        w::SetupFakeParts {
            listen: encode_socket(v.listen),
            git: v.git.clone(),
            owner: v.owner.clone(),
            repositories: encode_repositories(&v.repositories),
        },
    )
    .expect(FITS)
}

fn decode_fake(v: w::SetupFake) -> r::FakeEntry {
    let w::SetupFakeParts { listen, git, owner, repositories } = v.into_parts();
    r::FakeEntry { listen: decode_socket(listen), git, owner, repositories: decode_repositories(repositories) }
}

fn encode_setup(v: &r::Setup) -> w::ForgeSetup {
    w::ForgeSetup::new(
        &w::CEILINGS,
        w::ForgeSetupParts {
            directory: v.directory.clone(),
            bot: encode_account(&v.bot),
            orgs: encode_names(&v.orgs),
            mode: encode_mode(&v.mode),
            asked: v.asked,
        },
    )
    .expect(FITS)
}

fn decode_setup(v: w::ForgeSetup) -> r::Setup {
    let w::ForgeSetupParts { directory, bot, orgs, mode, asked } = v.into_parts();
    r::Setup { directory, bot: decode_account(bot), orgs: decode_names(orgs), mode: decode_mode(mode), asked }
}

fn encode_first(v: &r::FirstIntent) -> w::FirstIntent {
    w::FirstIntent::new(
        &w::CEILINGS,
        w::FirstIntentParts {
            org: v.org.clone(),
            repository: v.repository.clone(),
            project: encode_project(&v.project),
            label: v.label.clone(),
        },
    )
    .expect(FITS)
}

fn decode_first(v: w::FirstIntent) -> r::FirstIntent {
    let w::FirstIntentParts { org, repository, project, label } = v.into_parts();
    r::FirstIntent { org, repository, project: decode_project(project), label }
}

#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
pub(super) fn encode_forge(v: &d::Forge) -> w::Forge {
    w::Forge::new(
        &w::CEILINGS,
        w::ForgeParts {
            number: v.number,
            kind: encode_kind(v.kind),
            api: encode_address(&v.api),
            transport: encode_transport(&v.transport),
            git: v.git.clone(),
            git_program: v.git_program.clone(),
            web: v.web.clone(),
            read_only: v.read_only,
            identity: encode_identity(&v.identity),
            files: match &v.files {
                Some(v) => Some(encode_files(v)),
                None => None,
            },
            setup: match &v.setup {
                Some(v) => Some(encode_setup(v)),
                None => None,
            },
            first: match &v.first {
                Some(v) => Some(encode_first(v)),
                None => None,
            },
            playground: v.playground,
            removing: v.removing,
        },
    )
    .expect(FITS)
}

#[expect(clippy::manual_map, reason = "step translations use explicit matches without closures")]
pub(super) fn decode_forge(v: w::Forge) -> d::Forge {
    let w::ForgeParts {
        number,
        kind,
        api,
        transport,
        git,
        git_program,
        web,
        read_only,
        identity,
        files,
        setup,
        first,
        playground,
        removing,
    } = v.into_parts();
    d::Forge {
        number,
        kind: decode_kind(kind),
        api: decode_address(api),
        transport: decode_transport(transport),
        git,
        git_program,
        web,
        read_only,
        identity: decode_identity(identity),
        files: match files {
            Some(v) => Some(decode_files(v)),
            None => None,
        },
        setup: match setup {
            Some(v) => Some(decode_setup(v)),
            None => None,
        },
        first: match first {
            Some(v) => Some(decode_first(v)),
            None => None,
        },
        playground,
        removing,
    }
}

pub(super) fn encode_playground(v: &d::Playground) -> w::Playground {
    w::Playground::new(
        &w::CEILINGS,
        w::PlaygroundParts { state: encode_state(v.state), forge: v.forge, project: v.project, account: v.account },
    )
    .expect(FITS)
}

pub(super) fn decode_playground(v: w::Playground) -> d::Playground {
    let w::PlaygroundParts { state, forge, project, account } = v.into_parts();
    d::Playground { state: decode_state(state), forge, project, account }
}

pub(super) fn encode_kept(v: &d::Kept) -> w::Kept {
    w::Kept::new(&w::CEILINGS, w::KeptParts { key: v.key, act: encode_act(&v.act) }).expect(FITS)
}

pub(super) fn decode_kept(v: w::Kept) -> d::Kept {
    let w::KeptParts { key, act } = v.into_parts();
    d::Kept { key, act: decode_act(act) }
}

fn encode_kind(v: d::Kind) -> w::ForgeKind {
    match v {
        d::Kind::Forgejo => w::ForgeKind::Forgejo,
        d::Kind::GitHub => w::ForgeKind::Github,
    }
}

fn decode_kind(v: w::ForgeKind) -> d::Kind {
    match v {
        w::ForgeKind::Forgejo => d::Kind::Forgejo,
        w::ForgeKind::Github => d::Kind::GitHub,
    }
}

fn encode_state(v: d::PlaygroundState) -> w::PlaygroundState {
    match v {
        d::PlaygroundState::Running => w::PlaygroundState::Running,
        d::PlaygroundState::Resetting => w::PlaygroundState::Resetting,
    }
}

fn decode_state(v: w::PlaygroundState) -> d::PlaygroundState {
    match v {
        w::PlaygroundState::Running => d::PlaygroundState::Running,
        w::PlaygroundState::Resetting => d::PlaygroundState::Resetting,
    }
}

fn encode_names(v: &[Box<[u8]>]) -> List<w::SetupName> {
    let mut rows = List::with_capacity(u32::try_from(v.len()).expect(FITS));
    for value in v {
        rows.push(w::SetupName::new(&w::CEILINGS, w::SetupNameParts { name: value.clone() }).expect(FITS)).expect(FITS);
    }
    rows
}

fn decode_names(v: List<w::SetupName>) -> Box<[Box<[u8]>]> {
    let mut rows = List::with_capacity(v.len());
    for value in v.into_boxed() {
        rows.push(value.into_parts().name).expect(FITS);
    }
    rows.into_boxed()
}

fn encode_repositories(v: &[r::FakeRepository]) -> List<w::SetupRepository> {
    let mut rows = List::with_capacity(u32::try_from(v.len()).expect(FITS));
    for value in v {
        rows.push(encode_repository(value)).expect(FITS);
    }
    rows
}

fn decode_repositories(v: List<w::SetupRepository>) -> Box<[r::FakeRepository]> {
    let mut rows = List::with_capacity(v.len());
    for value in v.into_boxed() {
        rows.push(decode_repository(value)).expect(FITS);
    }
    rows.into_boxed()
}
fn encode_socket(v: SocketAddr) -> w::ForgeSocket {
    match v {
        SocketAddr::V4(v) => w::ForgeSocket::Ip4(
            w::ForgeIp4::new(&w::CEILINGS, w::ForgeIp4Parts { ip: v.ip().octets(), port: v.port() }).expect(FITS),
        ),
        SocketAddr::V6(v) => w::ForgeSocket::Ip6(
            w::ForgeIp6::new(&w::CEILINGS, w::ForgeIp6Parts { ip: v.ip().octets(), port: v.port() }).expect(FITS),
        ),
    }
}
fn decode_socket(v: w::ForgeSocket) -> SocketAddr {
    match v {
        w::ForgeSocket::Ip4(v) => {
            let w::ForgeIp4Parts { ip, port } = v.into_parts();
            SocketAddr::V4(SocketAddrV4::new(ip.into(), port))
        }
        w::ForgeSocket::Ip6(v) => {
            let w::ForgeIp6Parts { ip, port } = v.into_parts();
            SocketAddr::V6(SocketAddrV6::new(ip.into(), port, 0, 0))
        }
    }
}
fn encode_address(v: &r::Address) -> w::ForgeAddress {
    match v {
        r::Address::Socket(v) => w::ForgeAddress::Socket(
            w::ForgeSocketAddress::new(&w::CEILINGS, w::ForgeSocketAddressParts { socket: encode_socket(*v) })
                .expect(FITS),
        ),
        r::Address::Host { name, port } => w::ForgeAddress::Host(
            w::ForgeHost::new(&w::CEILINGS, w::ForgeHostParts { name: name.clone(), port: *port }).expect(FITS),
        ),
    }
}
fn decode_address(v: w::ForgeAddress) -> r::Address {
    match v {
        w::ForgeAddress::Socket(v) => r::Address::Socket(decode_socket(v.into_parts().socket)),
        w::ForgeAddress::Host(v) => {
            let w::ForgeHostParts { name, port } = v.into_parts();
            r::Address::Host { name, port }
        }
    }
}
fn encode_transport(v: &r::Transport) -> w::ForgeTransport {
    match v {
        r::Transport::Plaintext => w::ForgeTransport::Plaintext,
        r::Transport::Tls { server_name, trust } => w::ForgeTransport::Tls(
            w::ForgeTls::new(&w::CEILINGS, w::ForgeTlsParts { server_name: server_name.clone(), trust: trust.clone() })
                .expect(FITS),
        ),
    }
}
fn decode_transport(v: w::ForgeTransport) -> r::Transport {
    match v {
        w::ForgeTransport::Plaintext => r::Transport::Plaintext,
        w::ForgeTransport::Tls(v) => {
            let w::ForgeTlsParts { server_name, trust } = v.into_parts();
            r::Transport::Tls { server_name, trust }
        }
    }
}
fn encode_user(v: &r::FileUser) -> w::ForgeUser {
    match v {
        r::FileUser::Id(id) => {
            w::ForgeUser::Id(w::ForgeUserId::new(&w::CEILINGS, w::ForgeUserIdParts { id: *id }).expect(FITS))
        }
        r::FileUser::File(file) => w::ForgeUser::File(
            w::ForgeUserFile::new(&w::CEILINGS, w::ForgeUserFileParts { file: file.clone() }).expect(FITS),
        ),
    }
}
fn decode_user(v: w::ForgeUser) -> r::FileUser {
    match v {
        w::ForgeUser::Id(v) => r::FileUser::Id(v.into_parts().id),
        w::ForgeUser::File(v) => r::FileUser::File(v.into_parts().file),
    }
}
fn encode_mode(v: &r::SetupMode) -> w::SetupMode {
    match v {
        r::SetupMode::Managed { admin_token } => w::SetupMode::Managed(
            w::SetupManaged::new(&w::CEILINGS, w::SetupManagedParts { admin_token: admin_token.clone() }).expect(FITS),
        ),
        r::SetupMode::Owned(v) => w::SetupMode::Owned(encode_owned(v)),
        r::SetupMode::Fake(v) => w::SetupMode::Fake(encode_fake(v)),
    }
}
fn decode_mode(v: w::SetupMode) -> r::SetupMode {
    match v {
        w::SetupMode::Managed(v) => r::SetupMode::Managed { admin_token: v.into_parts().admin_token },
        w::SetupMode::Owned(v) => r::SetupMode::Owned(decode_owned(v)),
        w::SetupMode::Fake(v) => r::SetupMode::Fake(decode_fake(v)),
    }
}
fn encode_project(v: &d::SetUpProject) -> w::FirstProject {
    match v {
        d::SetUpProject::Existing(project) => w::FirstProject::Existing(
            w::FirstExisting::new(&w::CEILINGS, w::FirstExistingParts { project: *project }).expect(FITS),
        ),
        d::SetUpProject::New(name) => {
            w::FirstProject::New(w::FirstNew::new(&w::CEILINGS, w::FirstNewParts { name: name.clone() }).expect(FITS))
        }
    }
}
fn decode_project(v: w::FirstProject) -> d::SetUpProject {
    match v {
        w::FirstProject::Existing(v) => d::SetUpProject::Existing(v.into_parts().project),
        w::FirstProject::New(v) => d::SetUpProject::New(v.into_parts().name),
    }
}
fn encode_act(v: &d::KeptAct) -> w::KeptAct {
    match *v {
        d::KeptAct::Cancel { project, task } => {
            w::KeptAct::Cancel(w::KeptCancel::new(&w::CEILINGS, w::KeptCancelParts { project, task }).expect(FITS))
        }
        d::KeptAct::RemoveProject { project } => {
            w::KeptAct::RemoveProject(w::KeptProject::new(&w::CEILINGS, w::KeptProjectParts { project }).expect(FITS))
        }
        d::KeptAct::RemoveAccount { account } => {
            w::KeptAct::RemoveAccount(w::KeptAccount::new(&w::CEILINGS, w::KeptAccountParts { account }).expect(FITS))
        }
        d::KeptAct::BoxRemove { forge } => {
            w::KeptAct::BoxRemove(w::KeptForge::new(&w::CEILINGS, w::KeptForgeParts { forge }).expect(FITS))
        }
        d::KeptAct::RemoveForge { forge } => {
            w::KeptAct::RemoveForge(w::KeptForge::new(&w::CEILINGS, w::KeptForgeParts { forge }).expect(FITS))
        }
    }
}
fn decode_act(v: w::KeptAct) -> d::KeptAct {
    match v {
        w::KeptAct::Cancel(v) => {
            let w::KeptCancelParts { project, task } = v.into_parts();
            d::KeptAct::Cancel { project, task }
        }
        w::KeptAct::RemoveProject(v) => d::KeptAct::RemoveProject { project: v.into_parts().project },
        w::KeptAct::RemoveAccount(v) => d::KeptAct::RemoveAccount { account: v.into_parts().account },
        w::KeptAct::BoxRemove(v) => d::KeptAct::BoxRemove { forge: v.into_parts().forge },
        w::KeptAct::RemoveForge(v) => d::KeptAct::RemoveForge { forge: v.into_parts().forge },
    }
}
