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
    assert_eq!(default.address, temper_service::forge::HELD_ADDRESS);
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
fn each_box_adoption_refusal_renders_one_line_naming_what_it_lacks() {
    use temper_service::system::BoxRefusal;
    for (why, expected) in [
        (BoxRefusal::Sockets { limit: 1 }, "the box's pipes need two free socket slots; the service socket limit is 1"),
        (BoxRefusal::Adopted, "the box's pipes were already adopted"),
        (BoxRefusal::NoEngine, "the box's pipes need an engine"),
        (BoxRefusal::ZeroLine, "the box's pipes need a nonzero application.limits.overrides.box.line_bytes"),
    ] {
        assert_eq!(
            super::prefixed("temper serve", &super::box_refusal(why)),
            format!("temper serve: --box: {expected}\n")
        );
    }
}

#[test]
fn a_boxed_serve_reads_none_or_a_whole_secret() {
    use std::io::Cursor;
    use temper_service::clients::LINK_SECRET_BYTES;

    let mut empty = Cursor::new([]);
    let (printed, local) = super::given_secret(&mut empty).expect("empty input");
    assert!(printed.is_none() && local.is_none());
    let bytes = [23; LINK_SECRET_BYTES];
    let mut whole = Cursor::new(bytes);
    let (printed, local) = super::given_secret(&mut whole).expect("whole secret");
    assert!(printed.expect("given secret").expose() == bytes, "the secret is taken whole");
    assert!(local.is_none());
    let mut both = Cursor::new([vec![23; LINK_SECRET_BYTES], vec![37; LINK_SECRET_BYTES]].concat());
    let (printed, local) = super::given_secret(&mut both).expect("both given secrets");
    assert!(printed.expect("printed secret").expose() == bytes, "the printed secret comes first");
    assert!(local.expect("local secret").expose() == [37; LINK_SECRET_BYTES], "the local secret comes second");
    for length in [LINK_SECRET_BYTES - 1, LINK_SECRET_BYTES + 1, 63, 65] {
        let mut input = Cursor::new(vec![23; length]);
        assert_eq!(
            super::given_secret(&mut input).expect_err("partial or oversized secret"),
            "--box: stdin: expected the first link's secret"
        );
    }
    let mut oversized = Cursor::new(vec![23; LINK_SECRET_BYTES * 3]);
    assert!(super::given_secret(&mut oversized).is_err(), "oversized input is refused");
    assert_eq!(oversized.position(), 65);
}

#[test]
fn a_start_without_a_worker_half_refuses_workers_before_opening_their_files() {
    let (directories, _) = temper_shell::directories::resolve(
        &[b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from),
        &[],
        b"/",
    )
    .expect("fixture directories");
    for bytes in [b"{}".as_slice(), include_bytes!("../../../../docs/examples/owned.json").as_slice()] {
        let mut owned = Vec::new();
        let why = super::prepare("temper-engine", None, &directories, bytes, &mut owned).err().expect("no worker half");
        assert_eq!(why, "temper.json names a worker, and temper-engine runs none");
        assert!(owned.is_empty(), "refusal opens no startup descriptors");
    }
}
