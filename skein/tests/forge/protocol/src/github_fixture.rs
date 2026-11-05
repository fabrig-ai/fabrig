//! GitHub scenario overrides (skein/docs/design/fake-forge.md, 10.6).
use skein_fake_forge_github::{
    app::App,
    fixture, installations, pages,
    service::{Config, Limits, Service},
};
use skein_forge::credential::Credential;
use skein_forge_connection::{Access, Settings, settings::Kind as Dialect};
use skein_forge_kit::fixtures;
use skein_lib::{Duration, Secret, Wall};

pub(crate) fn limits(log_poll: Duration) -> skein_forge_connection::Limits {
    skein_forge_connection::Limits { log_poll, ..crate::fixtures::limits() }
}

pub(crate) const PATH: &[u8] = b"acme/widgets";
pub(crate) const USER: u64 = 9;

pub(crate) fn access(key: &[u8]) -> Access {
    Access { credential: Credential::App { id: 7, key: Secret::new(Box::from(key)) }, user: USER, webhook_secret: None }
}

pub(crate) fn settings() -> Settings {
    let mut settings = fixtures::unconnected();
    settings.forges.get_mut(0).expect("one forge").kind = Dialect::GitHub;
    settings
}

pub(crate) fn service_limits() -> Limits {
    let documents = fixtures::fake_limits().documents;
    Limits {
        http: fixtures::fake_limits().http,
        github: skein_forge::github::limits::Limits {
            json: documents.json,
            writer: documents.writer,
            string_bytes: documents.string_bytes,
            body_bytes: documents.body_bytes,
            inner: documents.inner,
            target_bytes: documents.target_bytes,
        },
        installations: installations::Limits { installations: 1, tokens: 8, listed: 1 },
        pages: pages::Limits { states: 1, manifest_bytes: 2048 },
        page: 2,
        body: 4096,
        response: 16384,
        ..fixture::limits()
    }
}

pub(crate) fn service(seed: u64, key: &'static [u8]) -> Service {
    let mut service = Service::new(
        Config {
            app: App { permissions: skein_fake_forge_github::app::wanted(), key, ..fixture::app() },
            ..fixture::config(fixture::widgets(2))
        },
        &service_limits(),
        seed,
    )
    .expect("GitHub scenario");
    fixture::install_one(&mut service, Wall::EPOCH);
    service
}

pub(crate) fn key() -> &'static [u8] {
    skein_fake_forge_github::fixture::key()
}
