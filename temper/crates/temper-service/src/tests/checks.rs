// Fixture parsing without a deployment keeps written paths.
#![expect(clippy::disallowed_methods, reason = "ordinary tests edit UTF-8 configuration fixtures")]
#![expect(clippy::disallowed_types, clippy::disallowed_macros, reason = "ordinary step tests build diagnostic tables")]
use super::fixtures;
use crate::startup::{Problems, What, checks, lines};
use jig_protocol_config::{self as config, Path};
use temper_protocol_worker_wire as payload;

fn started(input: &[u8], files: &crate::settings::Files, agent: &crate::settings::Agent) -> Problems {
    match fixtures::start(
        input,
        &jig_protocol_config::Directories::working(),
        files,
        agent,
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::forge_runtime(),
    ) {
        Ok(service) => {
            drop(service);
            Problems::empty()
        }
        Err(problems) => problems,
    }
}
#[test]
fn standalone_shape_is_required_of_jigs_parts() {
    let mut engine = fixtures::engine().jig;
    let mut clients = fixtures::engine().clients;
    engine.hosts.workers = Box::new([engine.hosts.workers.into_iter().next().unwrap()]);
    let mut workers = Box::new([fixtures::worker_settings().jig, fixtures::worker_settings().jig]);
    for (change, path, what) in standalone_shapes() {
        let count = apply_shape(change, &mut engine, &mut clients, &mut *workers);
        let mut found = Problems::empty();
        checks::standalone(&engine, &clients, &workers[..count], &mut found);
        assert_eq!(found.problems.len(), 1, "row {path:?}");
        assert_eq!(found.problems.get(0).unwrap().what, what, "row {path:?}");
        assert_eq!(found.problems.get(0).unwrap().path, path, "row {change:?}");
    }
}
#[test]
fn the_smallest_resumed_start_fits_the_engines_room_or_names_it() {
    // Header 8 + fixed fields 57 + calls * (14 + name 64 + tool 64 + cut words):
    // 32 calls with 69-byte words; 128 with 70-byte words (jig/docs/design/protocol/channel/README.md, 10).
    const SMALL: u32 = 6_817;
    const STANDARD: u32 = 27_201;
    let mut engine = fixtures::engine().jig;
    for (profile, smallest, words) in [
        (
            config::Profile::Small,
            SMALL,
            "engine.limits.io.output: 6816 bytes, short of the smallest resumed start, 6817 bytes\n",
        ),
        (
            config::Profile::Standard,
            STANDARD,
            "engine.limits.io.output: 27200 bytes, short of the smallest resumed start, 27201 bytes\n",
        ),
    ] {
        let limits = config::profile(profile);
        for (row, output, expected) in
            [("stock", limits.io.output, ""), ("met", smallest, ""), ("one short", smallest - 1, words)]
        {
            engine.limits = limits;
            engine.limits.io.output = output;
            let mut problems = Problems::empty();
            checks::resumed_start(&engine, &mut problems);
            assert_eq!(lines(&problems).as_ref(), expected.as_bytes(), "{profile:?} {row}");
        }
    }
}
#[test]
fn the_longest_remote_of_an_empty_machine_measures_github_com() {
    let machine = temper_forge_setup::machine::Machine { owned: None, fake: None, github: None };
    assert_eq!(checks::machine_remote(&machine, 10), Some(44));
}
#[test]
fn a_remote_past_the_checkouts_names_refuses_the_start() {
    let document = fixtures::with(
        &fixtures::document(),
        b"application.limits.overrides",
        br#"{"checkout.name_bytes":300,"checkout.repositories":4,"forge.name_bytes":256,"forge.resources_per_task":4}"#,
    );
    let found = crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err();
    assert_eq!(
        &*lines(&found),
        b"application.limits.overrides.checkout.name_bytes: must agree with the longest remote\n".as_slice()
    );
}
#[test]
fn a_checkout_with_one_repository_can_start() {
    let document = fixtures::with(
        &fixtures::document(),
        b"application.limits.overrides",
        br#"{"checkout.name_bytes":600,"checkout.repositories":1,"git_component.stdout_bytes":131072}"#,
    );
    let found = started(&document, &fixtures::files(), &fixtures::startup_agent());
    assert!(found.is_empty(), "{:?}", lines(&found));
}
#[test]
fn default_engine_and_worker_grants_fit_the_channel_and_link() {
    let own = crate::limits::profile(crate::limits::Profile::Small);
    for limits in [
        crate::translate::limits(&own).unwrap(),
        crate::worker_settings::payload_limits(&crate::limits::worker(&own)).unwrap(),
    ] {
        assert_eq!(payload::largest(&limits).unwrap().grant, 7057);
        assert_eq!(jig_protocol_worker_wire::CEILINGS.application_value_secret, 8192);
        let worker = fixtures::worker_settings();
        crate::worker_settings::link_limits(&worker.jig, &limits, own.forge.forges).unwrap();
    }
    assert!(started(&fixtures::document(), &fixtures::files(), &fixtures::startup_agent()).is_empty());
    let worker =
        crate::startup::worker_part(&fixtures::worker_alone_document(), &config::Directories::working()).unwrap();
    assert_eq!(worker.limits.peer.installations, crate::limits::FORGE_INSTALLATIONS);
    assert_eq!(worker.limits.peer.token_bytes, crate::limits::FORGE_TOKEN_BYTES);
}

#[test]
fn a_worker_alone_takes_installation_and_token_bounds_from_its_configuration() {
    let document = fixtures::with(
        &fixtures::worker_alone_document(),
        b"application.limits.overrides",
        br#"{"forges.installations":3,"forge_component.token_bytes":64}"#,
    );
    let part = crate::startup::worker_part(&document, &config::Directories::working()).unwrap();
    let limits = crate::worker_settings::payload_limits(&part.limits).unwrap();
    assert_eq!(limits.entries_entries, 3);
    assert_eq!(limits.entry_token, 64);
    let mut engine = crate::limits::profile(crate::limits::Profile::Small);
    crate::limits::apply(&mut engine, b"forges.installations", 3).unwrap();
    crate::limits::apply(&mut engine, b"forge_component.token_bytes", 64).unwrap();
    let engine_limits = crate::translate::limits(&engine).unwrap();
    assert_eq!(engine_limits.entries_entries, limits.entries_entries);
    assert_eq!(engine_limits.entry_token, limits.entry_token);
}

#[test]
fn an_endpoint_or_model_missing_from_the_agent_is_named() {
    let document = fixtures::document();
    for (row, endpoints, models, expected) in [
        (
            "endpoint",
            false,
            true,
            b"engine.llm.endpoints[0]: not in the agent's configuration under the same name\n".as_slice(),
        ),
        ("model", true, false, b"engine.charters[0].model: not in the agent's configuration under the same name"),
    ] {
        let mut agent = fixtures::startup_agent();
        if !endpoints {
            agent.endpoints = smith_protocol_channel::Endpoints::new(skein_lib::List::with_capacity(0));
        }
        if !models {
            agent.bounds.endpoints = Box::new([]);
        }
        let result = fixtures::start(
            &document,
            &config::Directories::working(),
            &fixtures::files(),
            &agent,
            &fixtures::seeds(),
            None,
            fixtures::worker_runtime(),
            fixtures::forge_runtime(),
        );
        if models {
            assert_eq!(&*lines(&result.err().expect("endpoint is deployment configuration")), expected, "row {row}");
        } else {
            let mut service = result.expect("a charter model shortfall warns");
            assert_eq!(service.notes().pop(), Some(crate::Note::Engine(crate::EngineNote::StartupWarning { line: expected.into() })));
            assert_eq!(service.notes().pop(), None, "one selected warning");
        }
    }
}
#[test]
fn git_component_limits_against_the_checkout_name_both() {
    let mut own = crate::limits::profile(crate::limits::Profile::Small);
    own.checkout.name_bytes = 600;
    let git = own.git_component;
    for (git, expected) in [
        (
            crate::limits::GitComponent { operations: 2, ..git },
            b"application.limits.overrides.git_component.operations: must agree with checkout.workspaces\n".as_slice(),
        ),
        (
            crate::limits::GitComponent { stderr_bytes: 1, ..git },
            b"application.limits.overrides.git_component.stderr_bytes: must agree with checkout.push_diagnostic\n",
        ),
    ] {
        let mut found = Problems::empty();
        checks::limits(&own, &git, &mut found);
        assert_eq!(&*lines(&found), expected, "row {git:?}");
    }
}
#[test]
fn checkout_capacity_names_stdout_and_both_link_rooms() {
    let document = fixtures::with(
        &fixtures::document(),
        b"application.limits.overrides",
        br#"{"checkout.name_bytes":600,"checkout.repositories":2,"checkout.conflicts":1024,"checkout.path_bytes":4096,"git_component.stdout_bytes":262144}"#,
    );
    let mut document: serde_json::Value = serde_json::from_slice(&document).unwrap();
    document["engine"]["limits"]["overrides"]["io.output"] = 1_048_576_u32.into();
    document["workers"][0]["limits"]["overrides"]["io.output"] = 1_048_576_u32.into();
    let mut agent = fixtures::startup_agent();
    agent.host_limits.conflicts = 1024;
    agent.host_limits.path_bytes = 4096;
    let found = started(&serde_json::to_vec(&document).unwrap(), &fixtures::files(), &agent);
    assert_eq!(found.problems.len(), 1);
    assert_eq!(
        String::from_utf8_lossy(&lines(&found)),
        "checkout.conflicts 1024, checkout.path_bytes 4096, checkout.repositories 2: raise these with them\ngit_component.stdout_bytes: 262144 bytes, short of checkout.conflicts * (checkout.path_bytes + 1), 4195328 bytes\nengine.limits.io.output: 1048576 bytes, short of the largest answer a run's saved work makes, 8401142 bytes\nworkers[0].limits.io.output: 1048576 bytes, short of the largest answer a run's saved work makes, 8401142 bytes\n"
    );
}

#[test]
fn checkout_capacity_names_stdout_alone_when_the_links_hold() {
    let document = fixtures::with(
        &fixtures::document(),
        b"application.limits.overrides",
        br#"{"checkout.name_bytes":600,"checkout.repositories":2,"checkout.conflicts":1024,"git_component.stdout_bytes":1024}"#,
    );
    let mut agent = fixtures::startup_agent();
    agent.host_limits.conflicts = 1024;
    let found = started(&document, &fixtures::files(), &agent);
    assert_eq!(found.problems.len(), 1);
    assert_eq!(
        String::from_utf8_lossy(&lines(&found)),
        "checkout.conflicts 1024, checkout.path_bytes 255, checkout.repositories 2: raise these with them\ngit_component.stdout_bytes: 1024 bytes, short of checkout.conflicts * (checkout.path_bytes + 1), 262144 bytes\n"
    );
}

#[test]
fn checkout_capacity_keeps_the_agent_saved_and_rooms_in_one_refusal() {
    let document = fixtures::with(
        &fixtures::document(),
        b"application.limits.overrides",
        br#"{"checkout.name_bytes":600,"checkout.repositories":4,"checkout.conflicts":1024,"checkout.path_bytes":4096,"git_component.stdout_bytes":262144}"#,
    );
    let mut agent = fixtures::startup_agent();
    agent.host_limits.conflicts = 512;
    let found = started(&document, &fixtures::files(), &agent);
    assert_eq!(found.problems.len(), 1);
    assert_eq!(
        String::from_utf8_lossy(&lines(&found)),
        "checkout.conflicts 1024, checkout.path_bytes 4096, checkout.repositories 4: raise these with them\ngit_component.stdout_bytes: 262144 bytes, short of checkout.conflicts * (checkout.path_bytes + 1), 4195328 bytes\nworkers[0].agent.conflicts: 512 paths, short of checkout.conflicts, 1024 paths\nworkers[0].agent.path_bytes: 255 bytes, short of checkout.path_bytes, 4096 bytes\njig's Work.saved: 16777216 bytes, short of Saved's encoded bound, 16793660 bytes\nengine.limits.io.output: 1048576 bytes, short of the largest answer a run's saved work makes, 16798050 bytes\nworkers[0].limits.io.output: 1048576 bytes, short of the largest answer a run's saved work makes, 16798050 bytes\n"
    );
}

#[test]
fn a_channel_limit_past_a_ceiling_names_the_limit() {
    let mut own = crate::limits::profile(crate::limits::Profile::Small);
    own.checkout.name_bytes = 600;
    let normal = payload::Bounds {
        repositories: 1,
        name_bytes: 32,
        remote_bytes: 32,
        conflicts: 0,
        path_bytes: 0,
        token_bytes: 4096,
        entries: 8,
        entry_token_bytes: 512,
        text_bytes: 256,
    };
    let mut found = Problems::empty();
    let too_many = payload::Bounds { repositories: u32::MAX, ..normal };
    checks::channel(Some(&too_many), Some(&normal), None, &own, &mut found);
    assert_eq!(found.problems.len(), 1);
    assert_eq!(
        found.problems.get(0).unwrap().path,
        Path::root().member(b"application").member(b"limits").member(b"overrides").member(b"checkout.repositories")
    );
    let What::Past { bytes, .. } = found.problems.get(0).unwrap().what else {
        panic!("repository ceiling named");
    };
    assert_eq!(bytes, 4_294_967_295);
}
#[test]
fn one_ios_entities_hold_both_services_and_their_children() {
    let document = fixtures::with(&fixtures::document(), b"application.limits.overrides", br#"{"checkout.name_bytes":600,"checkout.repositories":4,"git_component.stdout_bytes":131072,"git_component.operations":61}"#);
    let found = crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err();
    assert_eq!(&*lines(&found), b"engine.limits.io.sockets: 260 bytes, past its bound of 256\n".as_slice());
}

#[derive(Clone, Copy, Debug)]
enum Shape {
    TwoWorkers,
    WorkerSocket,
    HostSocket,
    ClientProcess,
    ClientRemote,
    ClientRemoteApproval,
    Connectors,
    Program,
    Configuration,
}
fn apply_shape(
    shape: Shape,
    engine: &mut config::Engine,
    clients: &mut crate::startup::application::Clients,
    workers: &mut [config::Worker],
) -> usize {
    use core::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    use skein_io::{Addr, Name};
    let socket = config::Address::Addr(Addr::Ip(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7100))));
    let local = config::Address::Addr(Addr::InProcess(Name::new(b"workers").unwrap()));
    engine.hosts.listener = Box::new([local.clone()]);
    clients.listener = socket.clone();
    engine.connectors = Box::new([1]);
    for worker in workers.iter_mut() {
        worker.engine = local.clone();
        worker.agent.program = b"/opt/temper".as_slice().into();
        worker.agent.configuration = b"/opt/agent.json".as_slice().into();
    }
    match shape {
        Shape::TwoWorkers => return 2,
        Shape::WorkerSocket => workers[0].engine = socket,
        Shape::HostSocket => engine.hosts.listener = Box::new([socket]),
        Shape::ClientProcess => clients.listener = local,
        Shape::ClientRemote => {
            clients.listener =
                config::Address::Addr(Addr::Ip(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 1), 7100))));
        }
        Shape::ClientRemoteApproval => {
            clients.listener =
                config::Address::Addr(Addr::Ip(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 1), 7100))));
            engine.clients.approval = true;
            engine.clients.link = false;
        }
        Shape::Connectors => engine.connectors = Box::new([1, 2]),
        Shape::Program => workers[0].agent.program = b"temper".as_slice().into(),
        Shape::Configuration => workers[0].agent.configuration = b"agent.json".as_slice().into(),
    }
    1
}

fn standalone_shapes() -> [(Shape, Path, What); 9] {
    [
        (Shape::TwoWorkers, Path::root().member(b"workers"), What::Standalone),
        (
            Shape::WorkerSocket,
            Path::root().member(b"workers").child(config::Segment::Index(0)).member(b"engine"),
            What::Standalone,
        ),
        (Shape::HostSocket, Path::root().member(b"workers"), What::Half(crate::startup::EngineWhat::EngineAlone)),
        (
            Shape::ClientProcess,
            Path::root().member(b"application").member(b"clients").member(b"listener"),
            What::Half(crate::startup::EngineWhat::Loopback),
        ),
        (
            Shape::ClientRemote,
            Path::root().member(b"application").member(b"clients").member(b"listener"),
            What::Half(crate::startup::EngineWhat::Loopback),
        ),
        (
            Shape::ClientRemoteApproval,
            Path::root().member(b"application").member(b"clients").member(b"listener"),
            What::Half(crate::startup::EngineWhat::Loopback),
        ),
        (
            Shape::Connectors,
            Path::root().member(b"engine").member(b"connectors"),
            What::Half(crate::startup::EngineWhat::Connectors),
        ),
        (
            Shape::Program,
            Path::root().member(b"workers").child(config::Segment::Index(0)).member(b"agent").member(b"program"),
            What::Absolute,
        ),
        (
            Shape::Configuration,
            Path::root().member(b"workers").child(config::Segment::Index(0)).member(b"agent").member(b"configuration"),
            What::Absolute,
        ),
    ]
}

#[test]
fn a_v6_clients_listener_off_loopback_is_refused_at_the_start() {
    for document in [fixtures::document(), fixtures::engine_alone_document()] {
        let document = fixtures::with(&document, b"application.clients.listener", br#"{"socket":"[::]:7100"}"#);
        let found = crate::named(&document, &config::Directories::working()).expect_err("off-loopback IPv6");
        assert_eq!(&*lines(&found), b"application.clients.listener: must be a socket address on loopback\n");
    }
}
#[test]
fn a_webhook_listener_off_loopback_is_refused() {
    let document = fixtures::with(
        &fixtures::webhook_document(),
        b"application.webhooks.listener",
        br#"{"socket":"192.0.2.1:7200"}"#,
    );
    assert_eq!(
        &*lines(&crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err()),
        b"application.webhooks.listener: must be a socket address on loopback\n"
    );
}
#[test]
fn a_webhook_listener_equal_to_clients_is_refused() {
    let document = fixtures::with(
        &fixtures::webhook_document(),
        b"application.webhooks.listener",
        br#"{"socket":"127.0.0.1:7100"}"#,
    );
    assert_eq!(
        &*lines(&crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err()),
        b"application.webhooks.listener: must be unique\n"
    );
}
#[test]
fn webhook_listeners_on_port_zero_do_not_clash() {
    let document =
        fixtures::with(&fixtures::webhook_document(), b"application.webhooks.listener", br#"{"socket":"127.0.0.1:0"}"#);
    let document = fixtures::with(&document, b"application.clients.listener", br#"{"socket":"127.0.0.1:0"}"#);
    let named = crate::named(&document, &jig_protocol_config::Directories::working()).unwrap();
    assert_eq!(named.webhooks.unwrap().listener, named.clients);
}
#[test]
fn hook_connections_past_the_process_sockets_are_refused() {
    let document =
        fixtures::with(&fixtures::webhook_document(), b"application.limits.overrides", br#"{"checkout.name_bytes":600,"checkout.repositories":4,"git_component.stdout_bytes":131072,"forge_component.hooks":228}"#);
    assert_eq!(
        &*lines(&crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err()),
        b"engine.limits.io.sockets: 257 bytes, past its bound of 256\n"
    );
}
#[test]
fn the_shipped_profile_admits_a_webhook_listener() {
    let found = started(&fixtures::webhook_document(), &fixtures::files(), &fixtures::startup_agent());
    assert_eq!(&*lines(&found), b"");
}
#[test]
fn a_bundle_limit_of_zero_refuses_the_start() {
    for (overrides, expected) in [
        (
            br#"{"clients.bundle_files":0}"#.as_slice(),
            "application.limits.overrides.clients.bundle_files: must agree with a nonempty bundle\n",
        ),
        (
            br#"{"clients.bundle_bytes":0}"#.as_slice(),
            "application.limits.overrides.clients.bundle_bytes: must agree with a nonempty bundle\n",
        ),
    ] {
        let document = fixtures::with(&fixtures::document(), b"application.limits.overrides", overrides);
        let found = crate::named(&document, &jig_protocol_config::Directories::working()).unwrap_err();
        assert_eq!(&*lines(&found), expected.as_bytes(), "row {overrides:?}");
    }
}

#[test]
fn fixed_redirect_ports_reject_listeners_on_the_reserved_number() {
    use crate::startup::Listener;
    for (port, listener, expected) in [
        (7100, Listener::Clients, "redirect_port: fixed port conflicts with application.clients.listener\n"),
        (7101, Listener::Webhooks, "redirect_port: fixed port conflicts with application.webhooks.listener\n"),
        (7100, Listener::Owned(2), "redirect_port: fixed port conflicts with application.machine.owned.listen\n"),
        (7100, Listener::Demo, "redirect_port: fixed port conflicts with application.demo.llm.listen\n"),
        (7100, Listener::Fake(2), "redirect_port: fixed port conflicts with application.machine.fake.listen\n"),
    ] {
        let mut problems = Problems::empty();
        checks::redirect_ports(&fixed_redirects(), Some(port), listener, &mut problems);
        assert_eq!(problems.problems.len(), 1, "{listener:?} {port:?}");
        assert_eq!(problems.problems.get(0).unwrap().what, What::Half(crate::startup::EngineWhat::Redirect(listener)));
        assert_eq!(lines(&problems).as_ref(), expected.as_bytes());
    }
}
#[test]
fn fixed_redirect_ports_accept_listeners_without_a_reserved_number() {
    use crate::startup::Listener;
    for (port, listener) in [(Some(7102), Listener::Clients), (Some(0), Listener::Clients), (None, Listener::Webhooks)]
    {
        let mut problems = Problems::empty();
        checks::redirect_ports(&fixed_redirects(), port, listener, &mut problems);
        assert_eq!(problems.problems.len(), 0, "{listener:?} {port:?}");
    }
}
#[test]
fn provider_redirect_ports_are_checked_before_files_are_read() {
    const PROVIDERS: &[u8] = br#"[{"name":"Fake","number":1,"signed_in_plan":false,"endpoint":"fake","sources":{"sign_in":{"issuer":"https://issuer.example","client":"client","scopes":[],"authorization":"https://issuer.example/auth","token":"https://issuer.example/token","keys":"https://issuer.example/keys","transport":"tls","server_name":"issuer.example","must_grant":[],"redirect_path":"callback","redirect_port":7100}}}]"#;
    let input = fixtures::with(&fixtures::document(), b"engine.llm.providers", PROVIDERS);
    let input = fixtures::with(&input, b"application.limits.overrides", br#"{"checkout.name_bytes":600,"checkout.repositories":4,"git_component.stdout_bytes":131072,"git_component.operations":2,"checkout.workspaces":2,"forge_component.hooks":1}"#);
    let input = core::str::from_utf8(&input).unwrap();
    for (listener, expected) in [
        (r#"{"socket":"127.0.0.1:7100"}"#, true),
        (r#"{"socket":"[::1]:7100"}"#, true),
        (r#"{"socket":"127.0.0.1:7102"}"#, false),
        (r#"{"socket":"127.0.0.1:0"}"#, false),
    ] {
        let document = fixtures::with(input.as_bytes(), b"application.clients.listener", listener.as_bytes());
        let result = crate::named(&document, &jig_protocol_config::Directories::working());
        if expected {
            assert_eq!(&*lines(&result.unwrap_err()), b"engine.llm.providers[0].sources.sign_in.redirect_port: fixed port conflicts with application.clients.listener\n");
        } else {
            let named = result.unwrap();
            assert_eq!(named.redirects.len(), 1);
            assert_eq!(&*named.redirects[0].redirect_path, b"callback");
        }
    }
    let hooks = input.replace("\"webhooks\":null", r#""webhooks":{"listener":{"socket":"[::1]:7100"}}"#);
    let hooks = fixtures::with(hooks.as_bytes(), b"application.clients.listener", br#"{"socket":"127.0.0.1:7102"}"#);
    assert_eq!(&*lines(&crate::named(&hooks, &jig_protocol_config::Directories::working()).unwrap_err()), b"engine.llm.providers[0].sources.sign_in.redirect_port: fixed port conflicts with application.webhooks.listener\n");
    let ephemeral = input.replace(",\"redirect_port\":7100", "");
    assert!(
        crate::named(ephemeral.as_bytes(), &jig_protocol_config::Directories::working()).unwrap().redirects.is_empty()
    );
    let input = input.replace("\"redirect_port\":7100", "\"redirect_port\":0");
    assert_eq!(
        &*lines(&crate::named(input.as_bytes(), &jig_protocol_config::Directories::working()).unwrap_err()),
        b"engine.llm.providers[0].sources.sign_in.redirect_port: invalid form\n"
    );
}

#[test]
fn every_offered_adapter_must_be_declared_by_the_agent() {
    let document = fixtures::with(
        &fixtures::document(),
        b"engine.llm.providers",
        br#"[{"name":"Fake","number":1,"manage_usage":"https://claude.ai/settings/usage","endpoint":"fake","sources":{"programs":["claude_code"]}}]"#,
    );
    let document = jig_protocol_config::document(
        &document,
        &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    let engine_at = config::read::Object::new(&document, 0).unwrap().member(&document, b"engine").unwrap();
    let mut engine =
        config::engine(&document, engine_at, &jig_protocol_config::Directories::working(), config::Defaults::Passed)
            .unwrap();
    let defaults =
        crate::stock::with_models(Box::new([fixtures::small_model()]), engine.limits.core.tasks.result_bytes);
    config::pass(&mut engine, defaults).unwrap();
    for declared in [false, true] {
        let mut agent = fixtures::startup_agent();
        if declared {
            agent.programs = Box::new([crate::settings::Program {
                adapter: crate::settings::Adapter::ClaudeCode,
                path: b"/bin/claude".as_slice().into(),
                directory: b"/programs".as_slice().into(),
            }]);
        }
        let mut problems = Problems::empty();
        checks::agent(&engine, &agent.declaration(), &mut problems);
        assert_eq!(
            &*lines(&problems),
            if declared {
                b"".as_slice()
            } else {
                b"engine.llm.providers[0].sources.programs[0]: program adapter not declared in the agent's programs\n"
                    .as_slice()
            }
        );
        let mut no_offers = fixtures::engine().jig;
        no_offers.llm.providers = Box::new([]);
        let mut problems = Problems::empty();
        checks::agent(&no_offers, &agent.declaration(), &mut problems);
        assert!(problems.is_empty(), "no offers need no declaration");
    }
}
#[test]
fn agent_model_bounds_are_checked_under_the_configured_name() {
    for window in [true, false] {
        let engine = fixtures::engine().jig;
        let mut agent = fixtures::startup_agent();
        if window {
            agent.bounds.endpoints[0].window = 1;
        } else {
            agent.bounds.endpoints[0].output = 1;
        }
        let mut problems = Problems::empty();
        checks::agent(&engine, &agent.declaration(), &mut problems);
        assert!(problems.is_empty(), "charter models do not refuse the start");
        let core = config::core(&engine, [7; 16], 7);
        let warnings = checks::admission(&core, &engine, &agent.declaration(), &mut problems);
        assert!(problems.is_empty());
        assert_eq!(warnings.problems.len(), 1);
        assert_eq!(
            &*lines(&warnings),
            if window {
                b"engine.charters[0].model: needs a context window of 8192 tokens; the agent offers 1 tokens\n"
                    .as_slice()
            } else {
                b"engine.charters[0].model: needs 1024 output tokens; the agent offers 1 tokens\n".as_slice()
            }
        );
    }
}

#[test]
fn credential_capacity_is_checked_on_both_sides_and_against_smith() {
    let mut engine = fixtures::engine().jig;
    let mut worker = fixtures::worker_settings().jig.limits;
    let agent = fixtures::worker_agent();
    let bound = agent.limits.bodies.credential_value_bytes;
    for (sent, received, accepted) in
        [(bound, bound, true), (bound - 1, bound, true), (bound, bound - 1, false), (bound + 1, bound + 1, false)]
    {
        engine.limits.token = sent;
        worker.token = received;
        let mut problems = Problems::empty();
        checks::grants(&engine, &worker, &agent, &mut problems);
        assert_eq!(problems.is_empty(), accepted, "{sent} -> {received}");
        if !accepted {
            assert!(core::str::from_utf8(&lines(&problems)).unwrap().contains("accounts.token"));
        }
    }
}

#[test]
fn configured_keys_use_the_token_bound_instead_of_the_moved_file_ceiling() {
    let input = fixtures::with(
        fixtures::ENGINE,
        b"llm.providers",
        br#"[{"name":"Fake","number":1,"endpoint":"fake","sources":{"keys":true}}]"#,
    );
    let document =
        config::document(&input, &config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 32 }).unwrap();
    let mut engine =
        config::engine(&document, 0, &jig_protocol_config::Directories::working(), config::Defaults::Passed).unwrap();
    let defaults =
        crate::stock::with_models(Box::new([fixtures::small_model()]), engine.limits.core.tasks.result_bytes);
    config::pass(&mut engine, defaults).unwrap();
    engine.limits.token = 1;
    let key = engine.limits.token;
    assert_eq!(crate::values::credential_bytes(&engine), key);
    let mut worker = fixtures::worker_settings().jig.limits;
    worker.token = 1;
    let mut agent = fixtures::worker_agent();
    for bytes in [key - 1, key, key + 1] {
        agent.limits.bodies.credential_value_bytes = bytes;
        let mut problems = Problems::empty();
        checks::grants(&engine, &worker, &agent, &mut problems);
        assert_eq!(problems.is_empty(), bytes >= key);
    }
    agent.limits.bodies.program_value_name = u32::try_from(smith_host_domain::SourceName::BYTES).unwrap() - 1;
    let mut problems = Problems::empty();
    checks::grants(&engine, &worker, &agent, &mut problems);
    assert!(core::str::from_utf8(&lines(&problems)).unwrap().contains("program_value_name"));
}

fn fixed_redirects() -> [crate::startup::RedirectPort; 2] {
    [7100, 7101].map(|port| crate::startup::RedirectPort {
        port,
        path: Path::root().member(b"redirect_port"),
        redirect_path: b"callback".as_slice().into(),
    })
}

#[test]
fn charter_admission_shortfalls_warn_but_do_not_hide_deployment_refusals() {
    let mut agent = fixtures::startup_agent();
    agent.bounds.turns = 1;
    let engine = fixtures::guard_engine().jig;
    let core = config::core(&engine, [7; 16], 7);
    let mut found = Problems::empty();
    let warnings = checks::admission(&core, &engine, &agent.declaration(), &mut found);
    assert!(found.is_empty(), "charter bounds belong to stored settings: {found:?}");
    assert_eq!(
        core::str::from_utf8(&lines(&warnings)).unwrap(),
        "engine.charters[0].turns: needs 64 turns; the agent's profile.policy.max_turns allows 1\n"
    );
    agent.bounds.message_bytes = 0;
    checks::admission(&core, &engine, &agent.declaration(), &mut found);
    assert!(core::str::from_utf8(&lines(&found)).unwrap().contains("messages (tasks.message_bytes)"));
}

#[test]
fn admission_names_global_and_time_requirements_with_their_units() {
    use skein_lib::Duration;
    for (time, waiting, spend, inbox, unacknowledged, expected) in [
        (
            119_999,
            60_000,
            100,
            8,
            8,
            "engine.charters[0].time_ms: needs 3600000 ms of run time; the agent's profile.policy.max_time allows 119999 ms\n",
        ),
        (
            28_800_000,
            9_999,
            256,
            8,
            8,
            "engine.charters[0].waiting_ms: needs 10000 ms of waiting; the agent's profile.policy.max_waiting allows 9999 ms\n",
        ),
        (
            28_800_000,
            60_000,
            9,
            8,
            8,
            "engine.rules.maximum_run_spend: needs a spend allowance of 256; the agent's profile.policy.max_spend allows 9\n",
        ),
        (
            28_800_000,
            60_000,
            256,
            1,
            1,
            "engine.limits: needs an inbox of 2 (tasks.run_inbox); the agent's profile.policy.inbox allows 1\n",
        ),
        (
            28_800_000,
            60_000,
            256,
            2,
            0,
            "engine.limits: needs an unacknowledged-turn allowance of 1 (hub.turns); the agent's profile.policy.unacknowledged allows 0\n",
        ),
    ] {
        let mut agent = fixtures::startup_agent();
        agent.bounds.time = Duration::from_millis(time);
        agent.bounds.waiting = Duration::from_millis(waiting);
        agent.bounds.spend = spend;
        agent.bounds.inbox = inbox;
        agent.bounds.unacknowledged = unacknowledged;
        let engine = fixtures::guard_engine().jig;
        let core = config::core(&engine, [7; 16], 7);
        let mut found = Problems::empty();
        let warnings = checks::admission(&core, &engine, &agent.declaration(), &mut found);
        let (reported, empty) =
            if expected.starts_with("engine.limits") { (&found, &warnings) } else { (&warnings, &found) };
        assert!(empty.is_empty(), "{empty:?}");
        assert_eq!(&*lines(reported), expected.as_bytes());
    }
}

#[test]
fn missing_and_small_charter_models_warn_at_their_configuration_path() {
    for (window, output, expected) in [
        (None, 8192, "engine.charters[0].model: not in the agent's configuration under the same name\n"),
        (Some(1), 8192, "engine.charters[0].model: needs a context window of 8192 tokens; the agent offers 1 tokens\n"),
        (Some(1_048_576), 1, "engine.charters[0].model: needs 1024 output tokens; the agent offers 1 tokens\n"),
    ] {
        let mut agent = fixtures::startup_agent();
        agent.bounds.endpoints = match window {
            None => Box::new([]),
            Some(window) => Box::new([jig_domain_link::EndpointBound { endpoint: 1, window, output }]),
        };
        let engine = fixtures::guard_engine().jig;
        let core = config::core(&engine, [7; 16], 7);
        let mut found = Problems::empty();
        let warnings = checks::admission(&core, &engine, &agent.declaration(), &mut found);
        assert!(found.is_empty(), "{found:?}");
        assert!(core::str::from_utf8(&lines(&warnings)).unwrap().starts_with(expected));
    }
}

#[test]
fn each_document_shape_is_read_from_its_members() {
    use crate::startup::{Shape, shape};
    for (document, expected) in [
        (fixtures::document(), Shape::Standalone),
        (fixtures::engine_alone_document(), Shape::EngineAlone),
        (fixtures::worker_alone_document(), Shape::WorkerAlone),
    ] {
        assert_eq!(shape(&document, &config::Directories::working()), Ok(expected));
    }
}

type ShapeRow = (&'static [u8], &'static [u8], Vec<(Path, What)>);

fn shape_problems(input: &[u8]) -> Vec<(Path, What)> {
    crate::startup::shape(input, &config::Directories::working())
        .unwrap_err()
        .problems
        .iter()
        .map(|problem| (problem.path.clone(), problem.what))
        .collect()
}

#[test]
fn an_engine_alone_names_each_shape_departure() {
    use config::Segment::Index;
    let root = Path::root();
    let hosts = root.member(b"engine").member(b"hosts");
    let listener = hosts.member(b"listener");
    let base = fixtures::engine_alone_document();
    let rows: &[ShapeRow] = &[
        (b"workers", b"[{}]", vec![(root.member(b"workers"), What::Half(crate::startup::EngineWhat::EngineAlone))]),
        (
            b"application.workers",
            br#"[{"root":"worker-root","git":null}]"#,
            vec![(root.member(b"application").member(b"workers"), What::Half(crate::startup::EngineWhat::EngineAlone))],
        ),
        (
            b"engine.hosts.listener",
            br#"[{"socket":"127.0.0.1:0"},{"in_process":"workers"}]"#,
            vec![
                (listener.clone(), What::Half(crate::startup::EngineWhat::EngineAlone)),
                (listener.child(Index(1)), What::Half(crate::startup::EngineWhat::EngineAlone)),
            ],
        ),
        (
            b"engine.hosts.listener",
            br#"[{"socket":"127.0.0.1:0"},{"socket":"[::1]:0"}]"#,
            vec![(listener.clone(), What::Half(crate::startup::EngineWhat::EngineAlone))],
        ),
        (
            b"engine.hosts.listener",
            br#"[{"socket":"192.0.2.1:7000"}]"#,
            vec![(listener.child(Index(0)), What::Jig(config::What::Plaintext))],
        ),
        (
            b"engine.hosts.listener",
            br#"[{"socket":"127.0.0.1:0","transport":"tls"}]"#,
            vec![(listener.child(Index(0)).member(b"transport"), What::Jig(config::What::Unknown))],
        ),
        (
            b"engine.hosts.workers",
            b"[]",
            vec![(hosts.member(b"workers"), What::Half(crate::startup::EngineWhat::EngineAlone))],
        ),
        (
            b"engine.hosts.workers",
            br#"[{"name":"worker-a","digest":"same"},{"name":"worker-b","digest":"same"}]"#,
            vec![(
                hosts.member(b"workers").child(Index(1)).member(b"digest"),
                What::Half(crate::startup::EngineWhat::EngineAlone),
            )],
        ),
    ];
    for (member, value, expected) in rows {
        let input = fixtures::with(&base, member, value);
        assert_eq!(&shape_problems(&input), expected, "{}", core::str::from_utf8(member).unwrap());
    }
}

#[test]
fn the_socket_listener_in_the_old_hostsocket_case_now_names_both_local_worker_lists() {
    let input = fixtures::with(&fixtures::document(), b"engine.hosts.listener", br#"[{"socket":"127.0.0.1:7100"}]"#);
    assert_eq!(
        shape_problems(&input),
        vec![
            (Path::root().member(b"workers"), What::Half(crate::startup::EngineWhat::EngineAlone)),
            (
                Path::root().member(b"application").member(b"workers"),
                What::Half(crate::startup::EngineWhat::EngineAlone)
            ),
        ]
    );
}

#[test]
fn each_turn_bound_accepts_equality_and_names_one_byte_short() {
    const TURN: u32 = 65_536;
    let mut engine = config::profile(config::Profile::Small);
    let mut worker = fixtures::worker_settings().jig;
    let mut own = crate::limits::profile(crate::limits::Profile::Small);
    let mut agent = fixtures::startup_agent();
    agent.limits.bodies.turn_body = TURN;
    let loaded = TURN + u32::try_from(size_of::<temper_engine_domain::Record>()).unwrap();
    let window = u64::from(TURN + 4) * 4;
    let rows = [
        ("engine.limits.overrides.hub.turn_bytes", u64::from(TURN), "the agent's largest turn"),
        ("workers[0].limits.overrides.hub.turn_bytes", u64::from(TURN), "the agent's largest turn"),
        ("engine.limits.overrides.core.resume_bytes", u64::from(TURN + 4), "the agent's largest turn with its length"),
        ("application.limits.overrides.root.loads.transcript_bytes", u64::from(TURN), "the agent's largest turn"),
        (
            "application.limits.overrides.root.loads.bytes",
            u64::from(loaded),
            "the agent's largest turn with a loaded row",
        ),
        ("engine.limits.overrides.hub.transcript_bytes", window, "the engine's resume window"),
        ("workers[0].limits.overrides.hub.transcript_bytes", window, "the engine's resume window"),
        (
            "engine.limits.overrides.hub.outcome_bytes",
            agent.host_limits.outcome_bytes,
            "the agent's outcome (agent.json profile.declared.tool_payload)",
        ),
        (
            "workers[0].limits.overrides.hub.outcome_bytes",
            agent.host_limits.outcome_bytes,
            "the agent's outcome (agent.json profile.declared.tool_payload)",
        ),
        (
            "application.limits.overrides.forge.client.op_bytes",
            u64::from(engine.core.tasks.result_bytes),
            "tasks.result_bytes",
        ),
    ];
    for (index, (path, needs, of)) in rows.into_iter().enumerate() {
        engine.core.resume_bytes = TURN + 4;
        engine.hub.turn_bytes = u64::from(TURN);
        worker.limits.hub.turn_bytes = u64::from(TURN);
        engine.hub.transcript_bytes = window;
        worker.limits.hub.transcript_bytes = window;
        own.root.loads.transcript_bytes = TURN;
        own.root.loads.bytes = loaded;
        own.root.loads.reply_bytes = loaded;
        engine.hub.outcome_bytes = agent.host_limits.outcome_bytes;
        worker.limits.hub.outcome_bytes = agent.host_limits.outcome_bytes;
        own.forge.client.op_bytes = engine.core.tasks.result_bytes;
        let mut found = Problems::empty();
        checks::turns(Some(&engine), core::slice::from_ref(&worker), &own, Some(&agent.declaration()), &mut found);
        assert_eq!(&*lines(&found), b"", "{path} at equality");
        match index {
            0 => engine.hub.turn_bytes -= 1,
            1 => worker.limits.hub.turn_bytes -= 1,
            2 => engine.core.resume_bytes -= 1,
            3 => own.root.loads.transcript_bytes -= 1,
            4 => own.root.loads.bytes -= 1,
            5 => engine.hub.transcript_bytes -= 1,
            6 => worker.limits.hub.transcript_bytes -= 1,
            7 => engine.hub.outcome_bytes -= 1,
            8 => worker.limits.hub.outcome_bytes -= 1,
            9 => own.forge.client.op_bytes -= 1,
            _ => unreachable!("ten turn and payload relations"),
        }
        checks::turns(Some(&engine), core::slice::from_ref(&worker), &own, Some(&agent.declaration()), &mut found);
        assert_eq!(&*lines(&found), format!("{path}: {} bytes, short of {of}, {needs} bytes\n", needs - 1).as_bytes(),);
    }
}

#[test]
fn the_cluster_shapes_accept_both_loopback_families_and_the_whole_v4_range() {
    for socket in [br#"{"socket":"127.42.3.4:7100"}"#.as_slice(), br#"{"socket":"[::1]:7100"}"#] {
        let listener = [b"[".as_slice(), socket, b"]"].concat();
        let engine = fixtures::with(&fixtures::engine_alone_document(), b"engine.hosts.listener", &listener);
        assert_eq!(
            crate::startup::shape(&engine, &config::Directories::working()),
            Ok(crate::startup::Shape::EngineAlone)
        );
    }
}
#[test]
fn turn_checks_use_only_what_each_process_knows() {
    let engine = config::profile(config::Profile::Small);
    let own = crate::limits::profile(crate::limits::Profile::Small);
    // The small hub's 262144 bytes hold its 65536-byte resume window exactly.
    let mut found = Problems::empty();
    checks::turns(Some(&engine), &[], &own, None, &mut found);
    assert_eq!(&*lines(&found), b"");
    let mut short_engine = engine;
    short_engine.hub.transcript_bytes -= 1;
    checks::turns(Some(&short_engine), &[], &own, None, &mut found);
    assert_eq!(&*lines(&found), b"engine.limits.overrides.hub.transcript_bytes: 262143 bytes, short of the engine's resume window, 262144 bytes\n");
}
#[test]
fn installations_past_the_grant_bound_refuse_the_start_naming_the_limit() {
    // Nine entries fit; ten entries are the first count past jig's ceiling.
    let document = fixtures::with(&fixtures::document(), b"application.limits.overrides", br#"{"checkout.name_bytes":600,"checkout.repositories":4,"git_component.stdout_bytes":131072,"forge.name_bytes":256,"forge.resources_per_task":4,"forges.installations":10,"forge_component.installations":10}"#);
    let found = started(&document, &fixtures::files(), &fixtures::startup_agent());
    assert_eq!(&*lines(&found), b"engine.limits.application_value_secret: 8625 bytes, past its bound of 8192\nworkers.limits.application_value_secret: 8625 bytes, past its bound of 8192\n".as_slice());
}
