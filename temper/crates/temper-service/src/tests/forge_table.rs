//! Forge record conversions and bounded runtime tables (draft 3; contracts, D1).
#![expect(
    clippy::disallowed_macros,
    reason = "ordinary tests compare transport diagnostics and edit template fixtures"
)]
#![expect(clippy::disallowed_methods, reason = "ordinary tests construct and compare configuration fixtures")]
use super::fixtures;
use crate::{
    forge::{
        self,
        table::{self, NotKept, Shell},
    },
    settings,
    startup::{self, application},
};
use jig_protocol_config as config;
use skein_forge_connection::settings as connection;
use skein_forge_setup::host;
use skein_lib::{List, Secret};
use temper_engine_domain_forge::{Forge, record as r};

// The template contains three forges; these ports are its fixture listeners.
const TEMPLATE_FORGES: usize = 3;
// The setup host is pending before restart and running once ready.
const HOST_STATES: usize = 2;
const CLIENTS: u16 = 8367;
const OWNED: u16 = 8368;
const FAKE: u16 = 8369;
const LLM: u16 = 8370;
const WEBHOOKS: u16 = 8371;
const REMOVED: u16 = 2;
const OVER_BOUND: u16 = 77;

fn empty_files() -> settings::Files {
    settings::Files { entries: List::with_capacity(0) }
}
fn parsed(bytes: &[u8], files: &settings::Files) -> settings::Engine {
    let document =
        config::document(bytes, &config::DocumentLimits { bytes: 1 << 20, tokens: 16_384, depth: 32 }).unwrap();
    let directories = config::Directories::working();
    let mut root = config::read::Object::new(&document, 0).unwrap();
    let jig = startup::engine(&document, root.member(&document, b"engine").unwrap(), &directories).unwrap();
    let at = root.member(&document, b"application").unwrap();
    let mut problems = startup::Problems::empty();
    let members = application::parse(&document, at, &jig, &directories, &mut problems).unwrap();
    let resolved = application::resolve(&members, files, &mut problems).unwrap();
    let mut application = config::read::Object::new(&document, at).unwrap();
    let mut setup_problems = temper_forge_setup::problem::Problems::empty();
    let entries = temper_forge_setup::setup::Setup::read(
        &document,
        application.member(&document, b"setup").unwrap(),
        &members.limits.setup.plan(),
        &directories,
        &mut setup_problems,
    )
    .unwrap();
    let mut forges = resolved.forges;
    application::git_programs(&mut forges, &entries, &mut problems);
    let mut setup = List::with_capacity(members.limits.forge.forges);
    let mut records_only = List::with_capacity(members.limits.forge.forges);
    for value in &members.forges {
        if value.records_only {
            records_only.push(value.number).unwrap();
        }
    }
    for entry in entries.entries {
        setup.push(settings::Setup { entry, handover: None }).unwrap();
    }
    let mut engine = settings::Engine {
        jig,
        clients: members.clients,
        webhooks: resolved.webhooks,
        files: settings::Files { entries: files.entries.clone() },
        forges,
        identities: resolved.identities,
        setup: setup.into_boxed(),
        records_only: records_only.into_boxed(),
        demo: members.demo,
        limits: members.limits,
        ..fixtures::engine()
    };
    engine.seed = startup::seed::seed(&members.forges, &members.identities, &engine).unwrap();
    engine
}
fn template() -> settings::Engine {
    use core::num::NonZeroU16;
    let answer = temper_init::files(
        &temper_init::Facts {
            binary: b"/bin/temper",
            git: Some(b"/bin/git"),
            claude: None,
            login: Some(b"person"),
            ports: temper_init::Ports {
                clients: NonZeroU16::new(CLIENTS).unwrap(),
                owned: NonZeroU16::new(OWNED).unwrap(),
                forge: NonZeroU16::new(FAKE).unwrap(),
                llm: NonZeroU16::new(LLM).unwrap(),
                webhooks: NonZeroU16::new(WEBHOOKS).unwrap(),
            },
        },
        &temper_init::Directories { config: b"/config", data: b"/data" },
        &[0; 32],
        &[1; 32],
    )
    .unwrap();
    for file in answer.files {
        if file.name == b"temper.json" {
            let temper_init::Content::Bytes { bytes, .. } = file.content else { panic!("template document") };
            return parsed(&bytes, &empty_files());
        }
    }
    panic!("template document")
}
fn loopback(address: &config::Address) -> Result<core::net::SocketAddr, Box<[u8]>> {
    match address {
        config::Address::Addr(skein_io::Addr::Ip(value)) if value.ip().is_loopback() => Ok(*value),
        config::Address::Host { name, port } if name.as_ref() == b"api.github.com" => {
            Ok(core::net::SocketAddr::from((core::net::Ipv4Addr::LOCALHOST, *port)))
        }
        config::Address::Addr(_) | config::Address::Host { .. } => {
            Err(b"fixture resolves loopback only".as_slice().into())
        }
    }
}
#[expect(clippy::unnecessary_wraps, reason = "the fixture implements the shell resolver signature")]
fn trust(_: Option<&[u8]>) -> Result<skein_tls::Config, Box<[u8]>> {
    for forge in skein_forge_machine_world::tls::settings().forges.into_boxed() {
        if let connection::Transport::Tls { trust, .. } = forge.transport {
            return Ok(trust);
        }
    }
    panic!("TLS fixture trust")
}
fn shell() -> Shell {
    Shell { address: loopback, trust }
}
fn document_endpoints(engine: &settings::Engine) -> connection::Settings {
    let mut runtime = List::with_capacity(engine.limits.forge.forges);
    for source in &engine.forges {
        let transport = match &source.transport {
            settings::Transport::Plaintext => connection::Transport::Plaintext { name: None },
            settings::Transport::Tls { server_name, .. } => connection::Transport::Tls {
                server_name: skein_tls::Name::new(core::str::from_utf8(server_name).unwrap()).unwrap(),
                trust: trust(None).unwrap(),
            },
        };
        runtime
            .push(forge::ForgeRuntime { number: source.number, address: loopback(&source.api).unwrap(), transport })
            .unwrap();
    }
    forge::prepare(engine, runtime).unwrap()
}
fn same_forge(converted: &settings::Forge, document: &settings::Forge) {
    assert_eq!((converted.mode, converted.number, converted.kind), (document.mode, document.number, document.kind));
    assert_eq!(converted.api, document.api);
    match (&converted.transport, &document.transport) {
        (settings::Transport::Plaintext, settings::Transport::Plaintext) => {}
        (
            settings::Transport::Tls { server_name: a, trust: x },
            settings::Transport::Tls { server_name: b, trust: y },
        ) => {
            assert_eq!((a, x), (b, y));
        }
        _ => panic!("same transport"),
    }
    assert_eq!(
        (&converted.git, &converted.git_program, &converted.web),
        (&document.git, &document.git_program, &document.web)
    );
    assert_eq!(
        (converted.app_read_only, converted.user, converted.identity),
        (document.app_read_only, document.user, document.identity)
    );
    assert_eq!((&converted.api_token, &converted.webhook_secret), (&document.api_token, &document.webhook_secret));
}
fn same_identity(converted: &settings::Identity, document: &settings::Identity) {
    assert_eq!((converted.number, converted.forge), (document.number, document.forge));
    assert_eq!(
        (&converted.token, &converted.login, &converted.author),
        (&document.token, &document.login, &document.author)
    );
}
fn same_transport(converted: &connection::Transport, document: &connection::Transport) {
    match (converted, document) {
        (connection::Transport::Plaintext { name: a }, connection::Transport::Plaintext { name: b }) => {
            assert_eq!(a, b);
        }
        (
            connection::Transport::Tls { server_name: a, trust: x },
            connection::Transport::Tls { server_name: b, trust: y },
        ) => {
            assert_eq!(a, b);
            assert_eq!(format!("{x:?}"), format!("{y:?}"));
        }
        _ => panic!("same transport"),
    }
}
fn same_host(converted: &host::Forge, document: &host::Forge) {
    assert_eq!(converted.kind, document.kind);
    assert_eq!(converted.address, document.address);
    match (&converted.transport, &document.transport) {
        (host::Transport::Plaintext { name: a }, host::Transport::Plaintext { name: b }) => assert_eq!(a, b),
        (host::Transport::Tls { server_name: a, trust: x }, host::Transport::Tls { server_name: b, trust: y }) => {
            assert_eq!(a, b);
            assert_eq!(format!("{x:?}"), format!("{y:?}"));
        }
        _ => panic!("same transport"),
    }
}
#[test]
fn the_template_records_convert_as_the_document_does() {
    let engine = template();
    let endpoints = document_endpoints(&engine);
    let mut service = fixtures::started();
    let runtime = service.engine.as_mut().unwrap();
    runtime.setup.context = Some(crate::setup::settings::Context::new(&engine));
    runtime.forge.files.entries = engine.files.entries.clone();
    let (hosts, _) = crate::setup::settings::fixed(&engine, &endpoints).unwrap();
    assert_eq!(engine.seed.len(), TEMPLATE_FORGES);
    assert_eq!(usize::try_from(endpoints.forges.len()).unwrap(), TEMPLATE_FORGES);
    assert_eq!(usize::try_from(hosts.forges.len()).unwrap(), TEMPLATE_FORGES);
    for (((record, document), endpoint), host) in
        engine.seed.iter().zip(engine.forges.iter()).zip(endpoints.forges.iter()).zip(hosts.forges.iter())
    {
        same_forge(&table::settings_forge(record, &engine.files).unwrap(), document);
        let document_identity = engine.identities.iter().find(|value| value.number == record.identity.number).unwrap();
        same_identity(&table::identity(record, &engine.files).unwrap(), document_identity);
        let converted = table::endpoint(record, shell()).unwrap();
        assert_eq!(
            (converted.number, converted.kind, converted.address),
            (endpoint.number, endpoint.kind, endpoint.address)
        );
        assert_eq!((&converted.access, &converted.log_origins), (&endpoint.access, &endpoint.log_origins));
        same_transport(&converted.transport, &endpoint.transport);
        same_host(&table::host_forge(runtime, record, &converted).unwrap(), host);
    }
}
#[test]
fn a_file_form_record_reads_its_files() {
    let engine = parsed(&fixtures::document(), &fixtures::files());
    for ((record, document), document_identity) in
        engine.seed.iter().zip(engine.forges.iter()).zip(engine.identities.iter())
    {
        same_forge(&table::settings_forge(record, &engine.files).unwrap(), document);
        same_identity(&table::identity(record, &engine.files).unwrap(), document_identity);
    }
    let mut with_hook = engine.seed[0].clone();
    with_hook.files.as_mut().unwrap().webhook_secret = Some(b"secrets/forge-1.webhook-secret".as_slice().into());
    assert_eq!(
        table::settings_forge(&with_hook, &engine.files).unwrap().webhook_secret.unwrap().expose(),
        b"fixture-hook-secret"
    );
    let record = &engine.seed[0];
    assert_eq!(table::settings_forge(record, &empty_files()).unwrap_err(), NotKept::File);
    assert_eq!(table::identity(record, &empty_files()).unwrap_err(), NotKept::File);
}
#[test]
fn a_records_only_record_needs_no_file() {
    let mut record = fixtures::engine().seed[0].clone();
    record.files = None;
    assert!(record.files.is_none());
    let converted = table::settings_forge(&record, &empty_files()).unwrap();
    let identity = table::identity(&record, &empty_files()).unwrap();
    assert!(converted.api_token.is_none() && converted.webhook_secret.is_none());
    assert_eq!(converted.user, 0);
    assert!(identity.token.expose().is_empty());
}
#[test]
fn an_unresolved_host_is_held() {
    let mut record = fixtures::engine().seed[0].clone();
    record.api = r::Address::Host { name: b"unresolved.invalid".as_slice().into(), port: FAKE };
    let endpoint = table::endpoint(&record, Shell::refusing()).unwrap();
    assert_eq!(endpoint.address, skein_io::Addr::Ip(forge::HELD_ADDRESS));
    let connection::Transport::Plaintext { name } = endpoint.transport else { panic!("plaintext host") };
    assert_eq!(name.as_deref(), Some(b"unresolved.invalid".as_slice()));
    assert!(endpoint.access.is_none());
}
fn numbered(record: &Forge, number: u16) -> Forge {
    let mut record = record.clone();
    record.number = number;
    record
}
#[test]
fn each_table_takes_one_more_forge_and_drops_it() {
    let configuration = fixtures::engine();
    let mut record = configuration.seed[0].clone();
    record.files = None;
    let identity = &configuration.identities[0];
    let bound = configuration.limits.forge.forges;
    let starts: [fn() -> crate::Service; HOST_STATES] = [fixtures::started, fixtures::ready];
    for start in starts {
        let mut service = start();
        let engine = service.engine.as_mut().unwrap();
        (engine.forge.shell.address)(&configuration.forges[0].api).unwrap();
        for number in 1..=u16::try_from(bound).unwrap() {
            let record = numbered(&record, number);
            let endpoint = table::endpoint(&record, shell()).unwrap();
            let host = table::host_forge(engine, &record, &endpoint).unwrap();
            engine.forge.add_held(number, b"held.invalid".as_slice().into()).unwrap();
            engine.forge_records.rows.add(number).unwrap();
            engine.values.tokens.add(number, identity, configuration.limits.forges.installations).unwrap();
            engine.workers.add_forge(table::settings_forge(&record, &empty_files()).unwrap()).unwrap();
            engine.setup.add_forge(number, Ok(host), Some(Secret::new(b"file-api".as_slice().into()))).unwrap();
            engine.setup.lifecycle.add_forge(number, crate::setup::lifecycle::Reconcile::GitHub).unwrap();
        }
        let extra = numbered(&record, OVER_BOUND);
        assert_eq!(engine.forge.add_held(OVER_BOUND, Box::new([])), Err(NotKept::Bound));
        assert_eq!(engine.forge_records.rows.add(OVER_BOUND), Err(NotKept::Bound));
        assert_eq!(
            engine.values.tokens.add(OVER_BOUND, identity, configuration.limits.forges.installations),
            Err(NotKept::Bound)
        );
        assert_eq!(
            engine.workers.add_forge(table::settings_forge(&extra, &empty_files()).unwrap()),
            Err(NotKept::Bound)
        );
        assert_eq!(engine.setup.add_forge(OVER_BOUND, Err(b"unserved"), None), Err(NotKept::Bound));
        assert_eq!(
            engine.setup.lifecycle.add_forge(OVER_BOUND, crate::setup::lifecycle::Reconcile::GitHub),
            Err(NotKept::Bound)
        );
        let item = temper_protocol_records::Item::GitToken;
        engine.forge_records.rows.push(
            temper_protocol_records::number(REMOVED, item),
            Secret::new(temper_protocol_records::encode(item, b"cached-git-token").unwrap().into_boxed_slice()),
            configuration.limits.records.value_bytes,
        );
        engine.forge.remove_held(REMOVED);
        engine.forge_records.rows.remove(REMOVED);
        engine.values.tokens.remove(REMOVED);
        engine.workers.remove_forge(REMOVED);
        engine.setup.remove_forge(REMOVED);
        assert!(engine.forge.unreachable(REMOVED).is_none());
        assert!(engine.forge_records.rows.forge(REMOVED).is_none());
        assert!(engine.workers.forges.iter().all(|forge| forge.number != REMOVED));
        engine.forge.add_held(OVER_BOUND, Box::new([])).unwrap();
        engine.forge_records.rows.add(OVER_BOUND).unwrap();
        engine.values.tokens.add(OVER_BOUND, identity, configuration.limits.forges.installations).unwrap();
        engine.workers.add_forge(table::settings_forge(&extra, &empty_files()).unwrap()).unwrap();
        engine.setup.add_forge(OVER_BOUND, Err(b"unserved"), None).unwrap();
        engine.setup.remove_forge(OVER_BOUND);
        let endpoint = table::endpoint(&extra, shell()).unwrap();
        let host = table::host_forge(engine, &extra, &endpoint).unwrap();
        engine.setup.add_forge(OVER_BOUND, Ok(host), Some(Secret::new(b"new-file-api".as_slice().into()))).unwrap();
        engine.setup.lifecycle.add_forge(OVER_BOUND, crate::setup::lifecycle::Reconcile::GitHub).unwrap();
        assert_eq!(engine.workers.forges.capacity(), bound);
    }
}

#[path = "forge_added.rs"]
mod added;
