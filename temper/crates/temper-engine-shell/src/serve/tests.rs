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
fn github_machine_names_wait_and_machine_sockets_are_used_without_log_origins() {
    let roots = super::trust(None).expect("machine trust roots");
    let mut machine = temper_forge_setup::machine::GitHubMachine {
        git: None,
        api: jig_protocol_config::Address::Addr(skein_io::Addr::Ip(([127, 0, 0, 1], 3001).into())),
        tls: false,
        remote: b"http://127.0.0.1:3001".as_slice().into(),
        web: b"http://127.0.0.1:3001".as_slice().into(),
    };
    let fake = super::github_runtime(Some(&machine), &roots).expect("machine socket");
    assert!(fake.log_origins.is_empty());
    assert_eq!(fake.address, ([127, 0, 0, 1], 3001).into());
    assert!(matches!(fake.transport, skein_forge_connection::settings::Transport::Plaintext { .. }));
    machine.api = jig_protocol_config::Address::Host { name: b"machine.github.test".as_slice().into(), port: 8443 };
    machine.tls = true;
    let named = super::github_runtime(Some(&machine), &roots).expect("machine name waits");
    assert_eq!(named.address, temper_engine_service::forge::HELD_ADDRESS);
    assert!(named.log_origins.is_empty());
    let skein_forge_connection::settings::Transport::Tls { server_name, .. } = named.transport else {
        panic!("TLS machine")
    };
    assert_eq!(server_name, skein_tls::Name::new("machine.github.test").unwrap());
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
    let document = config::document(
        include_bytes!("../../../../docs/examples/owned.json"),
        &config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 },
    )
    .expect("owned document");
    let at = super::member(&document, 0, b"engine");
    let mut engine = config::engine(&document, at, &config::Directories::working(), config::Defaults::Passed)
        .expect("engine fixture");
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
fn an_engine_start_refuses_workers_before_opening_their_files() {
    let (directories, _) = temper_shell::directories::resolve(
        &[b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from),
        &[],
        b"/",
    )
    .expect("fixture directories");
    for bytes in [b"{}".as_slice(), include_bytes!("../../../../docs/examples/owned.json").as_slice()] {
        let mut owned = Vec::new();
        let why = super::prepare("temper-engine", &[], b"/", &directories, bytes, &mut owned)
            .err()
            .expect("engine refuses a worker");
        assert_eq!(why, "temper.json names a worker, and temper-engine runs none");
        assert!(owned.is_empty(), "refusal opens no startup descriptors");
    }
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
    let limits = temper_engine_service::limits::profile(temper_engine_service::limits::Profile::Small).clients;
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
