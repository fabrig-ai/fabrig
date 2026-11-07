//! Plan stories run through the host component (skein/docs/design/forge-setup.md, 6.3 and 7).
use super::{Heard, HostWorld};
use crate::fixtures;
use skein_fake_forge_forgejo::{self as fake, Service};
use skein_forge::forgejo::request::{HookSpec, Request as ForgeRequest};
use skein_forge_setup::{
    given::Given,
    host::{File, Kind, Plan, Request},
    item::Item,
    reconcile::{Failure, Outcome},
};
use skein_lib::Secret;
use std::net::SocketAddr;
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Ended {
    Done,
    Stop(Failure),
}
impl HostWorld {
    #[must_use]
    pub fn owned(seed: u64) -> Self {
        Self::configured(seed, true, fixtures::config())
    }
    pub(crate) fn configured(seed: u64, owned: bool, config: fake::Config) -> Self {
        let mut settings = fixtures::host_settings();
        settings.forges.get_mut(0).expect("one forge").kind = Kind::Forgejo(fixtures::given(owned));
        Self::with_settings(seed, &[], config, settings)
    }
    #[must_use]
    pub fn managed_paged(seed: u64, cap: u32) -> Self {
        let mut config = fixtures::config_with_cap(cap);
        fixtures::picker_repositories(&mut config);
        let mut world = Self::with_config(seed, &[], config);
        fixtures::picker_owners(world.store_mut());
        world
    }
    fn given_mut(&mut self) -> &mut Given {
        match &mut self.forge.kind {
            Kind::Forgejo(given) => given,
            Kind::GitHub(_) => panic!("Forgejo fixture"),
        }
    }
    fn play(&mut self, plan: Plan) -> Ended {
        self.secrets.extend(self.service.setup().issued().iter().map(|token| token.secret.clone()));
        let since = self.heard.len();
        self.run_plan(1, plan);
        self.finish();
        match self.heard.get(since).expect("plan ended") {
            Heard::Done { .. } => Ended::Done,
            Heard::Stop { failure, .. } => Ended::Stop(failure.clone()),
            Heard::Closed { .. } => panic!("plan terminal event"),
        }
    }
    pub fn reconcile(&mut self) -> Ended {
        self.play(Plan::Reconcile { forge: self.forge.number(), adopted: self.adopted.clone() })
    }
    pub fn adopt(&mut self, repository: u64) -> Ended {
        self.play(Plan::Adopt {
            forge: self.forge.number(),
            repository: skein_forge_setup::host::Repository::Id(
                core::num::NonZeroU64::new(repository).expect("repository"),
            ),
        })
    }
    pub fn check(&mut self) -> Ended {
        self.play(Plan::Check { forge: self.forge.number() })
    }
    pub fn reachable(&mut self, installation: Option<u64>, page: u32) -> Ended {
        self.play(Plan::Reachable {
            forge: self.forge.number(),
            installation,
            page: core::num::NonZeroU32::new(page).expect("page"),
        })
    }
    pub fn take_over(&mut self, token: &[u8]) -> Ended {
        let admin_token = fixtures::secret(token);
        self.secrets.push(admin_token.clone());
        self.play(Plan::TakeOver { forge: self.forge.number(), admin_token })
    }
    pub fn create(&mut self, org: &[u8], repository: &[u8], files: Box<[File]>) -> Ended {
        self.play(Plan::Create { forge: self.forge.number(), org: org.into(), repository: repository.into(), files })
    }
    #[must_use]
    pub fn outcome(&self) -> Option<&Outcome> {
        match self.heard.last() {
            Some(Heard::Done { outcome, .. }) => Some(outcome),
            Some(Heard::Stop { .. } | Heard::Closed { .. }) | None => None,
        }
    }
    #[must_use]
    pub fn requests(&self) -> &[ForgeRequest] {
        &self.requests
    }
    #[must_use]
    pub fn secrets_absent(&self) -> bool {
        let trace = format!("{:?} {:?} {:?}", self.needs, self.heard, self.said);
        self.secrets
            .iter()
            .chain(self.service.setup().issued().iter().map(|token| &token.secret))
            .chain(
                self.adopted
                    .iter()
                    .filter_map(|id| u32::try_from(*id).ok())
                    .flat_map(|id| self.store().repository_hooks(id))
                    .map(|hook| &hook.secret),
            )
            .filter(|secret| !secret.is_empty())
            .all(|secret| !trace.as_bytes().windows(secret.len()).any(|bytes| bytes == secret.expose()))
    }
    #[must_use]
    pub fn adopted_ids(&self) -> &[u64] {
        &self.adopted
    }
    pub fn adopted(&mut self, ids: &[u64]) {
        self.adopted = ids.into();
    }
    fn replace_given(&mut self, change: impl FnOnce(&mut Given)) {
        self.action(Request::Remove { forge: self.forge.number() });
        self.finish();
        change(self.given_mut());
        self.action(Request::Add(self.forge.clone()));
    }
    pub fn listening(&mut self, address: SocketAddr) {
        self.replace_given(|given| given.listener = Some(address));
    }
    pub fn person(&mut self, login: Option<&[u8]>) {
        self.replace_given(|given| given.person = login.map(Into::into));
    }
    pub fn org_owner(&mut self, owner: Option<&[u8]>) {
        self.replace_given(|given| {
            given.orgs = Box::new([]);
            given.org_owner = owner.map(Into::into);
        });
    }
    pub fn forge_number(&mut self, forge: u16) {
        self.replace_given(|given| given.forge = forge);
    }
    pub fn service_word(&mut self, word: &[u8]) {
        self.replace_given(|given| {
            given.word = word.into();
            given.bot_login = word.into();
            given.api_name = [word, b"-api"].concat().into();
            given.git_name = [word, b"-git"].concat().into();
        });
        self.files.retain(|(item, _)| *item == Item::AdminToken);
    }
    pub fn admin_scopes(&mut self, scopes: &[&[u8]]) {
        self.store_mut().revoke(fixtures::ADMIN);
        let admin = self.store().find(b"kit-admin").expect("admin").id;
        self.store_mut().token(admin, b"limited", scopes, fixtures::secret(fixtures::ADMIN));
    }
    pub fn refuse(&mut self, route: fake::Route) {
        self.service.refuse(Some(route));
    }
    pub fn permit(&mut self) {
        self.service.refuse(None);
    }
    pub fn creation_domain(&mut self) {
        let config = &mut self.domain_env.limits;
        config.latency_min = skein_lib::Duration::ZERO;
        config.latency_max = skein_lib::Duration::ZERO;
        config.limits.repositories = 4;
        config.limits.name_bytes = 512;
        config.limits.content_bytes = 4096;
        self.domain = Some(skein_fake_forge_domain::Domain::new(config, 1));
    }
    pub fn domain_mut(&mut self) -> (&mut skein_fake_forge_domain::Domain, &skein_fake_forge_domain::Config) {
        (self.domain.as_mut().expect("creation domain"), &self.domain_env.limits)
    }
    #[must_use]
    pub fn repository_hooks(&self, repository: u32) -> Vec<&fake::setup::RepositoryHook> {
        self.store().repository_hooks(repository).collect()
    }
    pub fn with_hook(&mut self, repository: u32, url: &[u8], secret: Secret) -> u64 {
        self.secrets.push(secret.clone());
        self.store_mut()
            .seed_repository_hook(repository, HookSpec { url: url.into(), secret, events: Box::new([]), active: false })
            .expect("scenario hook")
    }
    pub fn delete_repository_hook(&mut self, repository: u32, id: u64) {
        assert!(self.store_mut().delete_repository_hook(repository, id));
    }
}
pub(crate) fn seeded_config(
    seed: u64,
    owned: bool,
    config: skein_fake_forge_forgejo::Config,
) -> (Service, skein_fake_forge_forgejo::Limits) {
    let limits = skein_forge_kit::fixtures::fake_limits();
    let mut service = Service::new(config, &limits, seed).expect("world service");
    let admin = service.setup_mut().account(b"kit-admin", true, b"");
    service.setup_mut().token(
        admin,
        b"setup-admin",
        &[b"write:admin", b"read:user", b"write:organization", b"write:repository"],
        fixtures::secret(fixtures::ADMIN),
    );
    if owned {
        service.setup_mut().account(b"ada", true, b"Ada");
    }
    (service, limits)
}
