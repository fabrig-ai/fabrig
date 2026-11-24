//! Connected GitHub fixture for the startup teardown story (testing/real-loop.md).
use crate::{fixtures, forge, landing};
use serde_json::{Value, json};
use skein_fake_forge_domain::{self as domain, api};
use skein_fake_forge_github::{
    self as layer,
    installations::{Kind, Reach},
};
use skein_fake_peers::forge::{Forge, Peer};
use skein_lib::{Duration, List, Secret, Wall};
use std::{cell::RefCell, net::SocketAddr, path::Path, rc::Rc};
use temper_github_app::APP;

pub(crate) const INSTALLATION: u64 = 1;
const REPOSITORY: u32 = 1;
const ACME: u64 = 10;
const UMBRA: u64 = 20;
// The layer retains at most this many git calls; a full ring fails the story.
const GIT_RECORDS: u32 = 128;
const WIRE_BYTES: u32 = 32_768;

// A whole bounded tree, git's JSON/base64 framing and the App's documents
// fit 32 KiB. State::new checks this against the domain's file bounds.
pub(crate) fn protocol_limits() -> skein_fake_forge_forgejo::Limits {
    let mut limits = forge::protocol_limits();
    limits.http.body = u64::from(WIRE_BYTES);
    limits.documents.json.string = WIRE_BYTES;
    limits.documents.json.length = WIRE_BYTES;
    limits.documents.writer.length = WIRE_BYTES;
    limits.documents.body_bytes = WIRE_BYTES;
    limits
}

fn setup() -> api::Setup {
    let mut setup = (forge::fixture().setup)();
    setup.hooked = false;
    setup
}
fn layer_config() -> layer::Config {
    let mut accounts = List::with_capacity(2);
    let mut repositories = List::with_capacity(2);
    for (login, id, name, repository) in [("acme", ACME, "app", REPOSITORY), ("umbra", UMBRA, "tool", 2)] {
        accounts
            .push(layer::service::Account { login: login.as_bytes().into(), id, organization: true })
            .expect("two accounts");
        repositories
            .push(layer::service::Row {
                id: repository,
                owner: login.as_bytes().into(),
                owner_id: id,
                organization: true,
                name: name.as_bytes().into(),
                private: true,
            })
            .expect("two repositories");
    }
    let mut permissions = List::with_capacity(16);
    for (name, access) in skein_forge::github::setup::PERMISSIONS {
        permissions
            .push(skein_forge::github::document::Permission { name: (*name).into(), access })
            .expect("App permissions");
    }
    let mut events = List::with_capacity(16);
    for event in skein_forge::github::setup::EVENTS {
        events.push((*event).into()).expect("App events");
    }
    layer::Config {
        deliveries: layer::delivery::Config {
            log_min: Duration::from_millis(10),
            log_max: Duration::from_millis(50),
            ..layer::delivery::Config::standard()
        },
        rate_quota: 5000,
        rate_window: Duration::from_secs(3600),
        token_life: layer::installations::TOKEN_LIFE,
        base: b"http://127.0.0.1".as_slice().into(),
        blob: b"http://127.0.0.1".as_slice().into(),
        third_origin: None,
        blob_redirect: None,
        app: layer::app::App {
            id: APP.id,
            slug: APP.slug.as_bytes().into(),
            owner: ACME,
            permissions: permissions.into_boxed(),
            events: events.into_boxed(),
            key: layer::keys::KEYS[APP.key],
            webhook: Secret::new(b"fake-webhook".as_slice().into()),
        },
        app_facts: layer::service::AppFacts::Complete,
        bot: APP.bot,
        shown: layer::service::Shown::Ruleset,
        ruleset_id: 1,
        pull_requests_only: false,
        required_signatures: false,
        hook: b"https://temper-test.invalid/".as_slice().into(),
        repositories,
        accounts,
        person: b"acme".as_slice().into(),
        tokens: List::with_capacity(0),
        faults: layer::Faults { reset: 0, drop: 0, server_error: 0 },
    }
}
pub(crate) fn write(
    path: &Path,
    address: SocketAddr,
    clients: SocketAddr,
    value: &mut temper_playground::documents::Configuration,
) -> std::io::Result<()> {
    value["world_settings"] = serde_json::from_slice(&std::fs::read(path.join("world-settings.json"))?)?;
    let mut agent: Value = serde_json::from_slice(&std::fs::read(path.join("world-agent.json"))?)?;
    value["application"]["clients"]["listener"] = json!({"socket":clients.to_string()});
    value["application"]["test"]["github"] = json!({"api":{"socket":address.to_string()},
        "transport":"plaintext","remote":format!("http://{address}"),"web":format!("http://{address}")});
    // Use the landing world's procedure and capacities, without a landing rule.
    (landing::fixture().configure)(value, &mut agent, address);
    value["world_settings"]["landing"]["projects"] = json!([]);
    value.temper_over("forge_component.log_poll_ms", 10);
    // Adoption must reach the forge for either account, rather than fail authority first.
    let broaden = |grants: &mut Value| {
        for grant in grants.as_array_mut().expect("grants") {
            grant["pattern"]["segments"] = json!(["forge"]);
            grant["pattern"]["value"] = address.to_string().into();
        }
    };
    for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
        broaden(&mut project["policy"]["ceiling"]["grants"]);
        broaden(&mut project["policy"]["roles"][0]["authority"]["grants"]);
        broaden(&mut project["policy"]["projections"]);
    }
    fixtures::turn_bounds::apply(value, fixtures::turn_bounds::turn_body(&agent));
    temper_agent_fixtures::apply(value, path, &agent);
    fixtures::finish_configuration(path, value)?;
    fixtures::private(&path.join("world-agent.json"), &serde_json::to_vec(&agent)?)
}

pub(crate) struct State {
    pub(crate) handle: Rc<RefCell<Forge>>,
}
impl State {
    pub(crate) fn new() -> (Self, Peer) {
        let mut config = temper_worker_checkout_world::hosted::config();
        // These smoke repositories have two small files each. The GitHub
        // layer pads every API response to its admitted wire capacity, so
        // do not inherit the checkout sweep's 256 files of 64 KiB each.
        config.limits.files = 4;
        config.limits.content_bytes = 1024;
        config.limits.calls = protocol_limits().calls;
        config.limits.title_bytes = 512;
        config.limits.body_bytes = 2048;
        // Mergeability settles on the smoke story's clock, not the checkout world's two seconds.
        config.settle_min = Duration::ZERO;
        config.settle_max = Duration::ZERO;
        assert!(skein_fake_forge_http::git::fits(&protocol_limits().git(), &config.limits), "bounded git trees fit");
        let clock = skein_shell::Clock::new().now();
        config.skew = domain::Skew::Ahead(Duration::from_nanos(
            clock.wall.as_nanos().checked_sub(clock.now.as_nanos()).expect("wall after monotonic"),
        ));
        let mut domain = domain::Domain::new(&config, fixtures::SEED);
        for (id, name) in [(REPOSITORY, b"acme/app".as_slice()), (2, b"umbra/tool".as_slice())] {
            let mut setup = setup();
            setup.id = id;
            setup.name = name.into();
            domain::repository(&mut domain, &config, setup);
        }
        domain::grant(&mut domain, b"acme/app", APP.bot, api::Permission::Admin);
        let env = clock.with(config);
        let handle = Rc::new(RefCell::new(Forge::new(domain, env.limits)));
        let wire = protocol_limits();
        let limits = layer::Limits {
            deliveries: layer::delivery::Limits { entries: 64, payload_bytes: wire.documents.body_bytes },
            http: wire.http,
            github: skein_forge::github::limits::Limits {
                json: wire.documents.json,
                writer: wire.documents.writer,
                string_bytes: wire.documents.string_bytes,
                body_bytes: wire.documents.body_bytes,
                inner: wire.documents.inner,
                target_bytes: wire.documents.target_bytes,
            },
            installations: layer::installations::Limits { installations: 8, tokens: 32, listed: 8 },
            pages: layer::pages::Limits { states: 8, manifest_bytes: 8192 },
            page: 32,
            body: wire.documents.body_bytes,
            response: wire.documents.body_bytes,
            blob_records: 16,
            git_records: GIT_RECORDS,
        };
        let mut hosted = forge::peer_limits();
        // The four call connections share this peer with the component's
        // reserved App/delivery connection, setup's client and hosted git.
        // A call racing a delivery poll must not be refused by the fixture.
        hosted.connections = wire.calls + 3;
        let mut peer = Peer::github(
            ([127, 0, 0, 1], 0).into(),
            ([127, 0, 0, 1], 0).into(),
            hosted,
            layer_config(),
            limits,
            handle.clone(),
            fixtures::SEED,
        )
        .expect("GitHub peer")
        .keeping(skein_fake_peers::forge::Kept::NONE);
        let id = peer
            .github_layer_mut()
            .expect("GitHub layer")
            .installations_mut()
            .install(ACME, b"acme", Kind::Organization, Reach::All, Wall::EPOCH)
            .expect("acme installed");
        assert_eq!(id, INSTALLATION, "layer's first installation");
        (Self { handle }, peer)
    }
}
