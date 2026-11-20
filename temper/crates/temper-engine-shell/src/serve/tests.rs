#[test]
fn fixed_start_seeds_use_independent_repeatable_draws_and_keep_box_secrets() {
    let start = skein_shell::FixedStart::read(b"1767690000,7").expect("canonical start");
    let make = || {
        super::seeds_from(
            &start,
            Some(skein_lib::Secret::new(Box::new([11; 32]))),
            Some(skein_lib::Secret::new(Box::new([12; 32]))),
        )
    };
    let seeds = make();
    let again = make();
    assert_eq!(seeds.seed, 7);
    assert_eq!(again.seed, seeds.seed);
    let mut crypto = [0; 32];
    let mut deployment = [0; 32];
    let mut installation = [0; 32];
    start.draw(1, &mut crypto);
    start.draw(2, &mut deployment);
    start.draw(3, &mut installation);
    assert_eq!(seeds.crypto.expose(), crypto);
    assert_eq!(seeds.deployment, deployment[..16]);
    assert_eq!(seeds.installation, installation[..16]);
    assert_eq!(again.crypto.expose(), seeds.crypto.expose());
    assert_eq!(again.deployment, seeds.deployment);
    assert_eq!(again.installation, seeds.installation);
    assert_ne!(seeds.deployment, seeds.installation);
    assert_ne!(&seeds.crypto.expose()[..16], seeds.deployment);
    assert_ne!(&seeds.crypto.expose()[..16], seeds.installation);
    assert_eq!(seeds.link_secret.expect("box link").expose(), &[11; 32]);
    assert_eq!(seeds.local_secret.expect("box local").expose(), &[12; 32]);
}

#[test]
fn github_defaults_are_unresolved_in_resolution_order() {
    let roots = super::trust(None).expect("machine roots");
    let runtime = super::github_runtime(None, &roots).expect("unresolved default");
    assert_eq!(runtime.address, temper_engine_service::forge::HELD_ADDRESS);
    let skein_forge_connection::settings::Transport::Tls { server_name, .. } = runtime.transport else {
        panic!("TLS default")
    };
    assert_eq!(server_name, skein_tls::Name::new("api.github.com").unwrap());
    assert_eq!(runtime.log_origins.len(), 20);
    for (number, entry) in runtime.log_origins.iter().enumerate() {
        assert_eq!(entry.origin.host.as_ref(), format!("productionresultssa{number}.blob.core.windows.net").as_bytes());
        assert!(entry.origin.tls);
        assert_eq!(entry.origin.port, 443);
        assert_eq!(entry.address, skein_io::Addr::Ip(temper_engine_service::forge::HELD_ADDRESS));
    }
}

#[test]
fn github_test_names_wait_and_test_sockets_are_used_without_log_origins() {
    let document = jig_protocol_config::document(
        br#"{"github":{"api":{"socket":"127.0.0.1:3001"},"transport":"plaintext","remote":"http://127.0.0.1:3001","web":"http://127.0.0.1:3001","git":"bin/git"}}"#,
        &jig_protocol_config::DocumentLimits { bytes: 4096, tokens: 128, depth: 8 },
    ).expect("test member");
    let mut problems = jig_protocol_config::Problems::empty();
    let mut test_github = temper_forge_setup::machine::read_test(
        &document,
        Some(0),
        &jig_protocol_config::Directories::working(),
        &mut problems,
    )
    .expect("test GitHub");
    assert!(problems.is_empty());
    let super::Runtimes { github: fake, .. } = super::runtimes(Some(&test_github)).expect("test socket");
    assert!(fake.log_origins.is_empty());
    assert_eq!(fake.address, ([127, 0, 0, 1], 3001).into());
    assert!(matches!(fake.transport, skein_forge_connection::settings::Transport::Plaintext { .. }));
    test_github.api = jig_protocol_config::Address::Host { name: b"github.example.test".as_slice().into(), port: 8443 };
    test_github.tls = true;
    let super::Runtimes { github: named, .. } = super::runtimes(Some(&test_github)).expect("test name waits");
    assert_eq!(named.address, temper_engine_service::forge::HELD_ADDRESS);
    assert!(named.log_origins.is_empty());
    let skein_forge_connection::settings::Transport::Tls { server_name, .. } = named.transport else {
        panic!("TLS test peer")
    };
    assert_eq!(server_name, skein_tls::Name::new("github.example.test").unwrap());
}

#[test]
fn accounts_runtime_holds_unresolved_hosts() {
    use jig_protocol_config as config;
    use skein_oauth_accounts::Transport;

    fn missing(_: &str, _: u16) -> Option<std::net::SocketAddr> {
        None
    }
    fn found(host: &str, port: u16) -> Option<std::net::SocketAddr> {
        let last = match host {
            "token.example.test" => 1,
            "issuer.example.test" => 2,
            "keys.example.test" => 3,
            "revoke.example.test" => 4,
            _ => return None,
        };
        Some(([192, 0, 2, last], port).into())
    }
    let file = temper_config_shell::File {
        workers: Some("127.0.0.1:7200".parse().expect("workers listener")),
        ..temper_config_shell::File::default()
    };
    let start = temper_config_shell::parse(
        &temper_config_shell::render(&file, temper_config_shell::Shape::EngineAlone),
        temper_config_shell::Shape::EngineAlone,
        &temper_config_shell::Facts { login: "sam".into(), hostname: "fixture".into(), provider_key: false },
    )
    .expect("engine file")
    .document;
    let document = config::document(&start, &config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 })
        .expect("owned document");
    let at = super::member(&document, 0, b"engine");
    let mut engine =
        temper_engine_service::startup::engine(&document, at, &config::Directories::working()).expect("engine fixture");
    let provider =
        engine.llm.providers.iter_mut().find(|provider| provider.sign_in.is_some()).expect("sign-in provider");
    let number = provider.number;
    let registration = provider.sign_in.as_mut().expect("registration");
    registration.token = b"https://token.example.test:8443/token".as_slice().into();
    registration.issuer = b"https://issuer.example.test/issuer".as_slice().into();
    registration.keys = b"http://keys.example.test/keys".as_slice().into();
    registration.revocation = Some(b"https://revoke.example.test:9443/revoke".as_slice().into());
    let held_address = ([127, 0, 0, 1], 0).into();
    for tls in [false, true] {
        engine
            .llm
            .providers
            .iter_mut()
            .find(|provider| provider.number == number)
            .expect("provider")
            .sign_in
            .as_mut()
            .expect("registration")
            .endpoint_transport = if tls {
            config::Transport::Tls { server_name: b"sign-in.example.test".as_slice().into(), trust: None }
        } else {
            config::Transport::Plaintext
        };
        let held = super::accounts_runtime::resolve(&engine, missing).expect("unresolved hosts are held");
        let resolved = super::accounts_runtime::resolve(&engine, found).expect("resolved hosts");
        let (held_number, held_endpoint, held_origins) = held.providers.get(0).expect("held provider");
        let (resolved_number, resolved_endpoint, resolved_origins) =
            resolved.providers.get(0).expect("resolved provider");
        assert_eq!((*held_number, *resolved_number), (number, number));
        assert_eq!(held_endpoint.address, held_address);
        assert_eq!(resolved_endpoint.address, found("token.example.test", 8443).expect("token address"));
        assert_eq!(held_origins.len(), 3);
        assert_eq!(resolved_origins.len(), held_origins.len());
        for ((held, resolved), (uri, host, port)) in held_origins.iter().zip(resolved_origins.iter()).zip([
            (b"https://issuer.example.test".as_slice(), "issuer.example.test", 443),
            (b"http://keys.example.test".as_slice(), "keys.example.test", 80),
            (b"https://revoke.example.test:9443".as_slice(), "revoke.example.test", 9443),
        ]) {
            assert_eq!(held.uri.as_ref(), uri);
            assert_eq!(resolved.uri, held.uri);
            assert_eq!(held.endpoint.address, held_address);
            assert_eq!(resolved.endpoint.address, found(host, port).expect("origin address"));
        }
        for endpoint in [held_endpoint, resolved_endpoint]
            .into_iter()
            .chain(held_origins.iter().chain(resolved_origins.iter()).map(|origin| &origin.endpoint))
        {
            assert_eq!(matches!(endpoint.transport, Transport::Tls { .. }), tls);
        }
    }
    let registration = engine
        .llm
        .providers
        .iter_mut()
        .find(|provider| provider.number == number)
        .expect("provider")
        .sign_in
        .as_mut()
        .expect("registration");
    registration.token = b"ftp://token.example.test/token".as_slice().into();
    assert_eq!(
        super::accounts_runtime::resolve(&engine, missing).expect_err("non-HTTP scheme refused"),
        "sign-in endpoint requires HTTP or HTTPS"
    );
}

#[test]
fn an_engine_start_refuses_a_workers_file_before_opening_its_files() {
    use temper_config_shell::{FILE, File, Shape};
    let scratch = skein_scratch::Scratch::new("engine-worker-file");
    let directories = crate::test_configuration::directories(scratch.path());
    let file = File { engine: Some("127.0.0.1:7200".parse().expect("engine address")), ..File::default() };
    std::fs::write(scratch.path().join(FILE), temper_config_shell::render(&file, Shape::WorkerAlone))
        .expect("worker file");
    let why = super::prepare_in("temper-engine", &[], b"/", &directories).err().expect("engine refuses a worker");
    assert_eq!(
        why,
        format!(
            "temper-engine serve: {FILE}:3: engine: a key of temper-worker's file, not temper-engine's\ntemper-engine serve: {FILE}: workers: required in temper-engine's file\n"
        )
    );
    assert_eq!(std::fs::read_dir(scratch.path()).expect("scratch").count(), 1, "only the supplied file remains");
}

#[test]
fn before_root_records_only_when_requested_and_refuses_a_headless_bundle() {
    use std::os::unix::ffi::OsStrExt;
    let scratch = skein_scratch::Scratch::new("record-bundle");
    std::fs::create_dir(scratch.path().join("0123456789abcdef")).expect("digest directory");
    let page = scratch.path().join("index.html");
    std::fs::write(&page, b"<html><head></head><body></body></html>").expect("page");
    let mut required: Box<[jig_protocol_config::File]> = Box::new([]);
    let mut asked: Box<[jig_protocol_config::File]> = Box::new([]);
    let configured = scratch.path().as_os_str().as_bytes();
    let limits = temper_limits_fixtures::small().clients;
    let mut tags = Vec::new();
    for record in [false, true] {
        let inputs = super::before_root(
            &mut required,
            &mut asked,
            None,
            Some(configured),
            limits,
            skein_shell::effective_user(),
            record,
        )
        .expect("page prepared");
        let bundle = inputs.bundle.expect("configured bundle");
        let text = core::str::from_utf8(&bundle.page).expect("page UTF-8");
        assert_eq!(text.contains("<meta name=\"temper-record\" content=\"on\">"), record);
        tags.push(bundle.etag);
    }
    assert_ne!(tags[0], tags[1]);
    std::fs::write(&page, b"<html><body></body></html>").expect("headless page");
    let Err(why) = super::before_root(
        &mut required,
        &mut asked,
        None,
        Some(configured),
        limits,
        skein_shell::effective_user(),
        true,
    ) else {
        panic!("missing head must be refused");
    };
    assert_eq!(
        why,
        format!(
            "application.clients.bundle: {}: the page has no </head> for its sign-in metadata",
            scratch.path().display()
        )
    );
}
