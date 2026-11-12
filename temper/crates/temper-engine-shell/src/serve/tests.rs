#[test]
fn github_default_log_origins_include_the_second_results_host() {
    fn resolve(configured: &jig_protocol_config::Address) -> Result<std::net::SocketAddr, String> {
        let jig_protocol_config::Address::Host { port, .. } = configured else {
            return Err("expected default host".into());
        };
        Ok(([192, 0, 2, 17], *port).into())
    }
    let roots = super::trust(None).expect("machine roots");
    let runtime = super::github_runtime("temper", None, &roots, resolve).expect("default resolves");
    assert_eq!(runtime.log_origins.len(), 20);
    assert!(
        runtime
            .log_origins
            .iter()
            .any(|entry| entry.origin.host.as_ref() == b"productionresultssa17.blob.core.windows.net")
    );
    assert!(runtime.log_origins.iter().all(|entry| entry.origin.tls && entry.origin.port == 443));
}

#[test]
fn github_log_origins_skip_unresolved_defaults_but_refuse_explicit_hosts() {
    fn missing(_: &jig_protocol_config::Address) -> Result<std::net::SocketAddr, String> {
        Err("no address".into())
    }
    assert!(super::github_logs::resolve(None, missing).expect("offline startup").is_empty());
    let explicit =
        [skein_forge::github::redirect::Origin { tls: true, host: b"logs.example.test".as_slice().into(), port: 443 }];
    let why = super::github_logs::resolve(Some(&explicit), missing).expect_err("explicit host must resolve");
    assert!(why.contains("logs.example.test"));
    assert!(super::github_logs::resolve(Some(&[]), missing).expect("disabled").is_empty());
}

#[test]
fn github_runtime_holds_unresolved_hosts_and_uses_the_machine_socket() {
    fn reject_names(configured: &jig_protocol_config::Address) -> Result<std::net::SocketAddr, String> {
        match configured {
            jig_protocol_config::Address::Host { name, port }
                if name.as_ref() == b"productionresultssa1.blob.core.windows.net" =>
            {
                Ok(([192, 0, 2, 1], *port).into())
            }
            jig_protocol_config::Address::Host { .. } => Err("no resolver in this test".into()),
            jig_protocol_config::Address::Addr(address) => Ok(address.socket().expect("socket fixture")),
        }
    }
    let roots = super::trust(None).expect("machine trust roots");
    let default = super::github_runtime("temper", None, &roots, reject_names).expect("held default");
    assert_eq!(default.address, temper_engine_service::forge::HELD_ADDRESS);
    assert_eq!(default.log_origins.len(), 1);
    let machine = temper_forge_setup::machine::GitHubMachine {
        git: None,
        api: jig_protocol_config::Address::Addr(skein_io::Addr::Ip(([127, 0, 0, 1], 3001).into())),
        tls: false,
        remote: b"http://127.0.0.1:3001".as_slice().into(),
        web: b"http://127.0.0.1:3001".as_slice().into(),
    };
    let fake = super::github_runtime("temper", Some(&machine), &roots, reject_names).expect("machine socket");
    assert_eq!(fake.address, ([127, 0, 0, 1], 3001).into());
    assert!(matches!(fake.transport, skein_forge_connection::settings::Transport::Plaintext { .. }));
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
