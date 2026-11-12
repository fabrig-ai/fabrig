//! The QA forge's document, fixed seed and setup acts (protocol/setup/playground.md).
pub mod github;
pub mod observations;
use crate::{Refusal, read};
use alloc::boxed::Box;
use core::net::SocketAddr;
use jig_protocol_config::Path;
use skein_fake_forge_domain::{self as domain, api};
use skein_fake_forge_forgejo as forgejo;
use skein_fake_peers::forge::Forge;
use skein_forge_setup::given::login;
use skein_lib::{Duration, Env, List, Queue, ReplyTo, Secret, Time, Token, Wall, bytes};
use temper_forge_setup::name::{branch, relative, repository};

/// Setup increments its account number before insertion, so it cannot issue MAX.
pub const OWNER: u64 = u64::MAX;
/// The one named check, with no silence or rerun.
pub const CONTEXT: &[u8] = b"check";
/// A first commit has an empty message in skein's repository constructor.
pub const MESSAGE: &[u8] = b"";
/// The fixed tree describes the check; the fake never runs its workflow.
pub const FILES: [(&[u8], &[u8]); 3] = [
    (b"README.md", b"# Widget\n\nA small repository for temper's QA loop.\n"),
    (temper_forge_setup::PLAYGROUND_PATH, temper_forge_setup::PLAYGROUND_MAIN),
    (b".forgejo/workflows/check.yml", b"name: check\non: [push, pull_request]\njobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo 'The fake judges the configured CI cue; this workflow is not run.'\n"),
];
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Ci {
    File { path: Box<[u8]>, green: Box<[u8]> },
    Declared { path: Box<[u8]> },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Repository {
    pub id: Option<u32>,
    pub tree: Option<Box<[api::File]>>,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub landing: Box<[u8]>,
    pub ci: Option<Ci>,
}
/// A source repository and its public catalogue address (protocol/setup/box.md, 5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Source {
    pub address: Box<[u8]>,
    pub repository: Box<[u8]>,
    pub landing: Box<[u8]>,
    pub tree: Box<[api::File]>,
    pub branches: Box<[Box<[u8]>]>,
    pub private: bool,
}
/// The checked standalone forge document; it reads no deployment or bot identity.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Document {
    listen: SocketAddr,
    admin_token: Box<[u8]>,
    owner: Box<[u8]>,
    repositories: Box<[Repository]>,
    sources: Box<[Source]>,
    github: Option<github::Github>,
    ci_ms: u32,
    made_checks: api::Checks,
}
impl Document {
    /// The fake forge of a checked setup entry, its token at `admin_token` (protocol/setup/playground.md).
    #[must_use]
    pub fn fake(fake: &temper_forge_setup::setup::Fake, admin_token: &[u8]) -> Self {
        let mut repositories =
            List::with_capacity(u32::try_from(fake.repositories.len()).expect("checked repositories"));
        for repository in &fake.repositories {
            repositories
                .push(Repository {
                    id: None,
                    tree: None,
                    owner: fake.owner.clone(),
                    name: repository.name.clone(),
                    landing: repository.landing.clone(),
                    ci: None,
                })
                .expect("checked repositories");
        }
        Self {
            listen: fake.listen,
            admin_token: bytes::copy_of(admin_token),
            owner: fake.owner.clone(),
            repositories: repositories.into_boxed(),
            sources: Box::new([]),
            github: None,
            ci_ms: 0,
            made_checks: limits().made_checks,
        }
    }
    /// The seeded App's public test private key, for the fake box's own-App path.
    #[must_use]
    pub fn github_app_key(&self) -> Option<Box<[u8]>> {
        let github = self.github.as_ref()?;
        Some(github.pem())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, Refusal> {
        let d = read::document(bytes)?;
        let p = Path::root();
        let mut o = read::object(&d, 0, &p)?;
        let listen_at = read::required(&mut o, &d, b"listen", &p)?;
        if let Some(at) = o.member(&d, b"git_program") {
            let path = read::text(&d, at, crate::limits::PATH_BYTES, &p.member(b"git_program"))?;
            read::absolute(&path, &p.member(b"git_program"))?;
        }
        let admin_token = read::field(&mut o, &d, b"admin_token", crate::limits::PATH_BYTES, &p)?;
        let owner = read::field(&mut o, &d, b"owner", 40, &p)?;
        let repositories_at = read::required(&mut o, &d, b"repositories", &p)?;
        let sources = match o.member(&d, b"sources") {
            Some(at) => sources(&d, at, &p.member(b"sources"))?,
            None => Box::new([]),
        };
        let github = match o.member(&d, b"github") {
            Some(at) => Some(github::decode(&d, at, &p.member(b"github"))?),
            None => None,
        };
        let made_ci_at = o.member(&d, b"made_ci");
        let ci_ms = read::pace(&mut o, &d, b"ci_ms", &p)?;
        let made_checks = match made_ci_at {
            Some(at) => checks_paced(ci(&d, at, &p.member(b"made_ci"), false)?.as_ref(), ci_ms),
            None => limits().made_checks,
        };
        read::finish(o, &d, &p)?;
        let listen = read::listen(&d, listen_at, &p.member(b"listen"))?;
        read::absolute(&admin_token, &p.member(b"admin_token"))?;
        if !login(&owner) || owner.as_ref() == b"temper-admin" {
            return Err(read::why(&p.member(b"owner"), b"expected a Forgejo login other than temper-admin"));
        }
        let p = p.member(b"repositories");
        let items = read::list(&d, repositories_at, 1, &p)?;
        let mut repositories = List::<Repository>::with_capacity(items.len());
        for (i, at) in items.iter().enumerate() {
            let p = read::index(&p, i);
            let mut o = read::object(&d, *at, &p)?;
            let id = match o.member(&d, b"id") {
                Some(at) => {
                    let id = read::integer(&d, at, u64::from(u32::MAX), &p.member(b"id"))?;
                    if id == 0 {
                        return Err(read::why(&p.member(b"id"), b"must be positive"));
                    }
                    Some(u32::try_from(id).expect("bounded id"))
                }
                None => None,
            };
            let tree = match o.member(&d, b"tree") {
                Some(at) => Some(tree(&d, at, &p.member(b"tree"))?),
                None => None,
            };
            let name = read::field(&mut o, &d, b"name", 100, &p)?;
            let landing = read::field(&mut o, &d, b"landing", limits().limits.name_bytes, &p)?;
            let repository_owner = if github.is_some() {
                match o.member(&d, b"owner") {
                    Some(at) => read::text(&d, at, 39, &p.member(b"owner"))?,
                    None => owner.clone(),
                }
            } else {
                owner.clone()
            };
            let ci_at = read::required(&mut o, &d, b"ci", &p)?;
            read::finish(o, &d, &p)?;
            if !repository(&name) {
                return Err(read::why(&p.member(b"name"), b"expected a Forgejo repository name"));
            }
            if !branch(&landing) {
                return Err(read::why(&p.member(b"landing"), b"expected a branch name"));
            }
            for prev in &repositories {
                if prev.id.is_some() != id.is_some() {
                    return Err(read::why(&p.member(b"id"), b"every repository must give an id, or none"));
                }
                if id.is_some() && prev.id == id {
                    return Err(read::why(&p.member(b"id"), b"must be unique"));
                }
                if prev.name == name && prev.owner == repository_owner {
                    return Err(read::why(&p.member(b"name"), b"must be unique"));
                }
            }
            let ci = ci(&d, ci_at, &p.member(b"ci"), tree.is_none())?;
            repositories
                .push(Repository { id, tree, owner: repository_owner, name, landing, ci })
                .expect("sixteen repositories");
        }
        if let Some(github) = &github {
            github.check(repositories.as_slice())?;
        }
        check_sources(&sources, repositories.as_slice())?;
        Ok(Self { listen, admin_token, owner, repositories: repositories.into_boxed(), sources, github, ci_ms, made_checks })
    }
    #[must_use]
    pub fn listen(&self) -> SocketAddr {
        self.listen
    }
    #[must_use]
    pub fn admin_token(&self) -> &[u8] {
        &self.admin_token
    }
    #[must_use]
    pub fn owner(&self) -> &[u8] {
        &self.owner
    }
    #[must_use]
    pub fn repositories(&self) -> &[Repository] {
        &self.repositories
    }
    #[must_use]
    pub fn sources(&self) -> &[Source] {
        &self.sources
    }
    /// The same seed order gives the same commit at every start: skein stores
    /// tree, message and parents, with no author or time; its id is a count.
    /// Source branch setup retains no write permission. Seed observations are drained,
    /// so a file holds the run alone.
    #[must_use]
    pub fn seed(&self, seed: u64) -> (Forge, forgejo::Config) {
        let mut configuration = limits();
        configuration.made_checks = self.made_checks.clone();
        let mut domain = domain::Domain::new(&configuration, seed);
        let mut repositories = List::with_capacity(crate::ENTRIES);
        for (i, repository) in self.repositories.iter().enumerate() {
            let id = repository
                .id
                .unwrap_or(u32::try_from(i).expect("sixteen repositories").checked_add(1).expect("numbered from one"));
            let full = read::joined(&[&repository.owner, b"/", &repository.name]);
            let tree = match &repository.tree {
                Some(tree) => tree.clone(),
                None => {
                    let mut tree = List::with_capacity(3);
                    for (path, content) in FILES {
                        tree.push(api::File { path: bytes::copy_of(path), content: bytes::copy_of(content) })
                            .expect("three seed files");
                    }
                    tree.into_boxed()
                }
            };
            domain::repository(
                &mut domain,
                &limits(),
                api::Setup {
                    workflows: Box::new([]),
                    id,
                    name: full.clone(),
                    default: repository.landing.clone(),
                    tree,
                    labels: Box::new([]),
                    checks: checks_paced(repository.ci.as_ref(), self.ci_ms),
                    protection: None,
                    hooked: true,
                },
            );
            if let Some(github) = &self.github {
                domain::grant(&mut domain, &full, github.bot(), api::Permission::Admin);
            }
            repositories.push(forgejo::Repository { id, full, protected: None }).expect("sixteen repositories");
        }
        let sources = seed_sources(&mut domain, &mut repositories, &self.sources);
        drain_seed_observations(&mut domain);
        let mut users = List::with_capacity(1);
        users.push((OWNER, self.owner.clone())).expect("one domain owner");
        let config = forgejo::Config {
            base: bytes::copy_of(b"http://127.0.0.1"),
            tokens: List::with_capacity(0),
            users,
            repositories,
            max_response_items: 32,
            default_paging_num: 30,
            sources,
            hooks: None,
            faults: forgejo::Faults { reset: 0, drop: 0, server_error: 0 },
        };
        (Forge::new(domain, configuration), config)
    }
}
fn ci(d: &skein_json::Document, at: u32, p: &Path, seed_tree: bool) -> Result<Option<Ci>, Refusal> {
    if read::is_null(d, at) { return Ok(None); }
    let mut o = read::object(d, at, p)?;
    if let Some(at) = o.member(d, b"declared") {
        let path = read::text(d, at, limits().limits.name_bytes, &p.member(b"declared"))?;
        read::finish(o, d, p)?;
        if path.is_empty() || !relative(&path) {
            return Err(read::why(&p.member(b"declared"), b"expected a relative file path with no empty, . or .. component"));
        }
        return Ok(Some(Ci::Declared { path }));
    }
    let path = read::field(&mut o, d, b"path", limits().limits.name_bytes, p)?;
    let green = read::field(&mut o, d, b"green", limits().limits.content_bytes, p)?;
    read::finish(o, d, p)?;
    if !relative(&path) {
        return Err(read::why(&p.member(b"path"), b"expected a relative file path with no . or .. component"));
    }
    for (seed, _) in FILES {
        if seed_tree && path.as_ref() == seed {
            return Err(read::why(&p.member(b"path"), b"must not name a seed file"));
        }
    }
    read::nonempty(&green, &p.member(b"green"))?;
    Ok(Some(Ci::File { path, green }))
}

fn sources(d: &skein_json::Document, at: u32, p: &Path) -> Result<Box<[Source]>, Refusal> {
    let items = read::list(d, at, 0, p)?;
    let mut sources = List::<Source>::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        let p = read::index(p, i);
        let mut o = read::object(d, *at, &p)?;
        let address = read::field(&mut o, d, b"address", 512, &p)?;
        source_address(&address, &p.member(b"address"))?;
        let repository_at = read::required(&mut o, d, b"repository", &p)?;
        let private = match o.member(d, b"private") {
            Some(at) => match jig_protocol_config::read::boolean(d, at) {
                Ok(private) => private,
                Err(why) => return Err(read::kind(&p.member(b"private"), why)),
            },
            None => false,
        };
        read::finish(o, d, &p)?;
        let rp = p.member(b"repository");
        let mut o = read::object(d, repository_at, &rp)?;
        let owner = read::field(&mut o, d, b"owner", 40, &rp)?;
        let name = read::field(&mut o, d, b"name", 100, &rp)?;
        if !login(&owner) {
            return Err(read::why(&rp.member(b"owner"), b"expected a Forgejo login"));
        }
        if !repository(&name) {
            return Err(read::why(&rp.member(b"name"), b"expected a Forgejo repository name"));
        }
        let repository = read::joined(&[&owner, b"/", &name]);
        let landing = read::field(&mut o, d, b"landing", limits().limits.name_bytes, &rp)?;
        if !branch(&landing) {
            return Err(read::why(&rp.member(b"landing"), b"expected a branch name"));
        }
        let at = read::required(&mut o, d, b"tree", &rp)?;
        let tree = tree(d, at, &rp.member(b"tree"))?;
        let mut branches = List::with_capacity(crate::ENTRIES);
        if let Some(at) = o.member(d, b"branches") {
            let bp = rp.member(b"branches");
            for (i, at) in read::list(d, at, 0, &bp)?.iter().enumerate() {
                let bp = read::index(&bp, i);
                let name = read::text(d, *at, limits().limits.name_bytes, &bp)?;
                if !branch(&name) {
                    return Err(read::why(&bp, b"expected a branch name"));
                }
                if name == landing || branches.as_slice().contains(&name) {
                    return Err(read::why(&bp, b"must be unique and other than landing"));
                }
                branches.push(name).expect("bounded branches");
            }
        }
        read::finish(o, d, &rp)?;
        for previous in &sources {
            if previous.address == address || previous.repository == repository {
                return Err(read::why(&p, b"source address and repository must be unique"));
            }
        }
        sources
            .push(Source { address, repository, landing, tree, branches: branches.into_boxed(), private })
            .expect("bounded sources");
    }
    Ok(sources.into_boxed())
}
fn source_address(address: &[u8], p: &Path) -> Result<(), Refusal> {
    let why = read::why(p, b"expected an https source address with a DNS host");
    let tail = address.strip_prefix(b"https://").ok_or(why.clone())?;
    let slash = bytes::find(tail, b"/").ok_or(why.clone())?;
    let authority = tail.get(..slash).expect("found separator");
    let host = match bytes::find(authority, b":") {
        None => authority,
        Some(colon) => {
            let port = authority.get(colon.saturating_add(1)..).expect("found colon");
            let port = skein_forge::forgejo::id::unsigned(port).ok_or(why.clone())?;
            if port == 0 || port > u64::from(u16::MAX) {
                return Err(why);
            }
            authority.get(..colon).expect("found colon")
        }
    };
    if !host.ends_with(b".invalid") || host.len() <= b".invalid".len() {
        return Err(read::why(&p.member(host), b"expected a host ending in .invalid"));
    }
    let mut label_start = true;
    let mut hyphen = false;
    for byte in host {
        if *byte == b'.' {
            if label_start || hyphen {
                return Err(why);
            }
            label_start = true;
        } else {
            if (!byte.is_ascii_alphanumeric() && *byte != b'-') || (label_start && *byte == b'-') {
                return Err(why);
            }
            label_start = false;
        }
        hyphen = *byte == b'-';
    }
    if label_start || hyphen {
        return Err(why);
    }
    Ok(())
}
fn check_sources(sources: &[Source], repositories: &[Repository]) -> Result<(), Refusal> {
    let p = Path::root().member(b"sources");
    let count = sources.len().checked_add(repositories.len()).expect("bounded lists");
    if count > usize::try_from(crate::ENTRIES).expect("u32 fits") {
        return Err(read::why(&p, b"sources and repositories together are past their bound"));
    }
    let count = u32::try_from(sources.len()).expect("bounded sources");
    for repository in repositories {
        if repository.id.unwrap_or(0).checked_add(count).is_none() {
            return Err(read::why(&p, b"no repository ids left for sources"));
        }
        let full = read::joined(&[&repository.owner, b"/", &repository.name]);
        for source in sources {
            if source.repository == full {
                return Err(read::why(&p, b"source repository must be other than repositories"));
            }
        }
    }
    Ok(())
}
fn seed_sources(
    domain: &mut domain::Domain,
    repositories: &mut List<forgejo::Repository>,
    sources: &[Source],
) -> Box<[forgejo::Source]> {
    let mut catalogue = List::with_capacity(crate::ENTRIES);
    let mut id = 0_u32;
    for repository in &*repositories {
        id = id.max(repository.id);
    }
    for source in sources {
        id = id.checked_add(1).expect("checked source ids");
        let first = domain::repository(
            domain,
            &limits(),
            api::Setup {
                workflows: Box::new([]),
                id,
                name: source.repository.clone(),
                default: source.landing.clone(),
                tree: source.tree.clone(),
                labels: Box::new([]),
                checks: checks(None),
                protection: None,
                hooked: false,
            },
        );
        seed_branches(domain, source, first);
        domain::grant(domain, &source.repository, 1, api::Permission::Read);
        drain_seed_observations(domain);
        repositories
            .push(forgejo::Repository { id, full: source.repository.clone(), protected: None })
            .expect("checked joint repository bound");
        catalogue
            .push(forgejo::Source {
                address: source.address.clone(),
                repository: source.repository.clone(),
                private: source.private,
                fill: api::Fill::Whole,
            })
            .expect("bounded sources");
    }
    catalogue.into_boxed()
}
fn seed_branches(domain: &mut domain::Domain, source: &Source, first: u64) {
    if source.branches.is_empty() {
        return;
    }
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    let mut out = Queue::with_capacity(limits().limits.calls);
    domain::grant(domain, &source.repository, OWNER, api::Permission::Admin);
    for branch in &source.branches {
        domain::step(
            domain,
            &env,
            domain::Event::Call {
                reply_to: ReplyTo::new(Token::new(1)),
                user: OWNER,
                repository: source.repository.clone(),
                op: api::Op::Write(api::Write::CreateBranch { branch: branch.clone(), commit: first }),
            },
            &mut out,
        );
        domain::fire(domain, &env, &mut out);
        match out.pop().expect("zero-latency branch answer") {
            domain::Request::Reply { result, .. } => {
                result.expect("checked source branch");
            }
            domain::Request::Hook { .. } => unreachable!("source hooks are disabled"),
        }
        assert!(out.is_empty(), "one source branch answer");
        domain.reclaim();
    }
    domain::grant(domain, &source.repository, OWNER, api::Permission::None);
}
fn drain_seed_observations(domain: &mut domain::Domain) {
    for _ in 0..limits().limits.observations {
        if domain.pop_observation().is_none() {
            break;
        }
    }
}
fn tree(d: &skein_json::Document, at: u32, p: &Path) -> Result<Box<[api::File]>, Refusal> {
    let items = match jig_protocol_config::read::list(d, at, limits().limits.files) {
        Ok(items) => items,
        Err(why) => return Err(read::kind(p, why)),
    };
    let mut files = List::<api::File>::with_capacity(items.len());
    for (i, at) in items.iter().enumerate() {
        let p = read::index(p, i);
        let mut o = read::object(d, *at, &p)?;
        let path = read::field(&mut o, d, b"path", limits().limits.name_bytes, &p)?;
        let content = read::field(&mut o, d, b"content", limits().limits.content_bytes, &p)?;
        read::finish(o, d, &p)?;
        if !relative(&path) {
            return Err(read::why(&p.member(b"path"), b"expected a relative file path with no . or .. component"));
        }
        for prev in &files {
            if prev.path == path {
                return Err(read::why(&p.member(b"path"), b"must be unique"));
            }
        }
        files.push(api::File { path, content }).expect("bounded files");
    }
    Ok(files.into_boxed())
}
/// Makes the setup's one site admin and one token; `all` is Forgejo's every scope.
pub fn setup(store: &mut forgejo::setup::Store, admin_token: &[u8; 40]) {
    let admin = store.account(b"temper-admin", true, b"temper admin");
    store.token(admin, b"temper-admin", &[b"all"], Secret::new(bytes::copy_of(admin_token)));
}
#[must_use]
pub fn checks(ci: Option<&Ci>) -> api::Checks {
    checks_paced(ci, 0)
}
/// The same CI cue with a fixed verdict latency (protocol/setup/playground.md).
#[must_use]
pub fn checks_paced(ci: Option<&Ci>, ci_ms: u32) -> api::Checks {
    let latency = Duration::from_millis(u64::from(ci_ms));
    api::Checks {
        cues: Box::new([]),
        contexts: Box::new([bytes::copy_of(CONTEXT)]),
        latency_min: latency,
        latency_max: latency,
        silent: 0,
        passes: 1000,
        reruns: 0,
        cue: match ci {
            Some(Ci::File { path, green }) => Some(api::Cue::File { path: path.clone(), green: green.clone() }),
            Some(Ci::Declared { path }) => Some(api::Cue::Declared { path: path.clone() }),
            None => None,
        },
    }
}
/// Bounded store for sixteen repositories, with small mutable QA histories.
#[must_use]
pub fn limits() -> domain::Config { domain::Config {
    setup_faults: 0,
    lost: 0,
    limits: domain::Limits {
        workflows: 0,
        workflow_runs: 0,
        repositories: crate::ENTRIES,
        users: 32,
        bypassing_users: 8,
        labels: 8,
        items: 32,
        comments: 8,
        dependencies: 8,
        reviews: 8,
        branches: 32,
        commits: 128,
        files: 16,
        statuses: 32,
        contexts: 8,
        pages: 8,
        name_bytes: 256,
        title_bytes: 256,
        body_bytes: 4096,
        content_bytes: 8192,
        page_size: 32,
        calls: 8,
        hooks: 32,
        observations: 128,
    },
    latency_min: Duration::ZERO,
    latency_max: Duration::ZERO,
    settle_min: Duration::ZERO,
    settle_max: Duration::ZERO,
    late: 0,
    late_min: Duration::ZERO,
    late_max: Duration::ZERO,
    unavailable: 0,
    timeouts: 0,
    landing: 0,
    land_min: Duration::ZERO,
    land_max: Duration::ZERO,
    rate_limit: 0,
    rate_window: Duration::ZERO,
    ci: 0,
    made_checks: domain::Config::test().made_checks,
    hook_min: Duration::ZERO,
    hook_max: Duration::ZERO,
    hooks_late: 0,
    hooks_lost: 0,
    resolution: Duration::from_secs(1),
    skew: domain::Skew::None,
    status_updates: false,
    edit_updates: false,
} }

/// The git route's wire bounds, shared by the peer and its process client.
#[must_use]
pub fn git_limits() -> skein_fake_forge_forgejo::Limits {
    crate::wire::forge_limits()
}

/// Constructs the checked document's peer without io (protocol/setup/playground.md).
/// The peer's shared Forge is lent back to the host for observation drains.
/// This follows skein's testing/fakes/forge.md, 5, `Rc<RefCell<Forge>>` boundary;
/// the departure from programming-model.md's ownership rule is confined here.
#[expect(
    clippy::disallowed_types,
    reason = "skein-fake-peers requires shared Forge ownership for host observation drains"
)]
pub fn peer(
    document: &Document,
    token: &[u8; 40],
    seed: u64,
) -> Result<(skein_fake_peers::forge::Peer, alloc::rc::Rc<core::cell::RefCell<Forge>>), skein_fake_peers::Error> {
    let (forge, config) = document.seed(seed);
    let shared = alloc::rc::Rc::new(core::cell::RefCell::new(forge));
    if let Some(github) = &document.github {
        return Ok((github.peer(document, shared.clone(), seed)?, shared));
    }
    let mut peer = skein_fake_peers::forge::Peer::new(
        document.listen(),
        crate::limits::peer(),
        config,
        git_limits(),
        shared.clone(),
        seed,
    )?
    .without_records();
    setup(peer.setup_mut(), token);
    Ok((peer, shared))
}
