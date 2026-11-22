use skein_io::kernel::{Done, Op};
use std::{
    fs,
    net::TcpListener,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
};
use temper_box_shell::{
    directories::{Directories, jig_directories},
    setup,
    testing::{
        init::{Io, Written, write, write_bare, write_with_key},
        init_fixtures::{directories, environment, scratch},
        path,
    },
};

const PROVIDER_KEY: &[u8] = include_bytes!("../../../src/init/fixtures/provider.pem");

#[test]
fn a_port_another_init_listens_on_between_bind_and_listen_gets_a_kernel_port() {
    let mut io = Io::open().expect("startup io");
    let bind = |io: &mut Io, port: u16| {
        let fd = io.socket().expect("socket");
        let Ok(Done::Bound(address)) = io.perform(Op::Bind { fd, addr: ([127, 0, 0, 1], port).into() }) else {
            panic!("bound at {port}")
        };
        (fd, address)
    };
    // Both bind the port under skein's SO_REUSEADDR; the other listens first.
    let (other, address) = bind(&mut io, 0);
    let (late, _) = bind(&mut io, address.port());
    io.perform(Op::Listen { fd: other, backlog: 1 }).expect("the other reservation listens");
    let (fd, port) = io.listened(late, address).expect("a kernel-chosen reservation");
    assert_ne!(port.get(), address.port(), "the lost port is not reported");
    assert!(TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port.get())).is_err(), "the chosen port is held");
    io.close(fd);
    io.close(other);
}

#[test]
fn a_kernel_chosen_port_another_takes_before_listen_gets_another() {
    let mut io = Io::open().expect("startup io");
    let late = io.socket().expect("kernel-chosen socket");
    let Ok(Done::Bound(address)) = io.perform(Op::Bind { fd: late, addr: ([127, 0, 0, 1], 0).into() }) else {
        panic!("kernel-chosen bind")
    };
    let other = io.socket().expect("competing socket");
    io.perform(Op::Bind { fd: other, addr: address }).expect("both sockets bind under SO_REUSEADDR");
    io.perform(Op::Listen { fd: other, backlog: 1 }).expect("the other reservation listens first");
    let (fd, port) = io.listened(late, address).expect("another kernel-chosen reservation");
    assert_ne!(port.get(), address.port(), "the lost port is not reported");
    assert!(TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port.get())).is_err(), "the chosen port is held");
    io.close(fd);
    io.close(other);
}

#[test]
fn init_finds_ports_and_programs_and_writes_exact_modes_in_order() {
    let mut rows = Vec::new();
    for held in [false, true] {
        for claude in [false, true] {
            rows.push((held, claude, true, false));
        }
    }
    rows.extend([(false, false, false, false), (false, false, true, true)]);
    for (held, claude, git, same) in rows {
        let row = format!("facts-{held}-{claude}-{git}-{same}");
        let root = scratch(&row);
        let dirs = directories(&root);
        let env = environment(&root, claude);
        if !git {
            fs::remove_file(root.join("bin/git")).expect("no git on PATH");
        }
        let binary = root.join("bin/temper");
        let (preferred, holder) = preferred_ports(held, same);
        let first = write(&dirs, &env, &binary, preferred).expect("first init");
        let ports = check_deployment(&dirs, &binary, &first, &row, git);
        for (at, available) in [!held, !held && !same].into_iter().enumerate() {
            assert_eq!(ports[at] == preferred[at], available, "{row}: preferred port {at}");
        }
        let second_root = root.join("second");
        fs::create_dir(&second_root).expect("second deployment");
        let second_dirs = directories(&second_root);
        // The first deployment holds both preferred ports while the second initializes.
        let second = write(&second_dirs, &env, &binary, ports).expect("second init beside the first");
        let second_ports = check_deployment(&second_dirs, &binary, &second, &row, git);
        for port in second_ports {
            assert!(!ports.contains(&port), "{row}: deployments have disjoint ports");
        }
        drop(holder);
        for written in [first, second] {
            temper_service_shell::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
        }
        fs::remove_dir_all(root).expect("cleanup");
    }
}

fn preferred_ports(held: bool, same: bool) -> ([u16; 2], Option<[TcpListener; 2]>) {
    let listeners = [0; 2].map(|_| TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("preferred listener"));
    let mut preferred = listeners.each_ref().map(|listener| listener.local_addr().expect("port").port());
    if same {
        preferred = [preferred[0]; 2];
    }
    let holder = if held {
        Some(listeners)
    } else {
        drop(listeners);
        None
    };
    (preferred, holder)
}

fn check_deployment(dirs: &Directories, binary: &Path, written: &Written, row: &str, git: bool) -> [u16; 2] {
    check_files(dirs, binary, written, row);
    let document: serde_json::Value = serde_json::from_slice(
        &temper_config_shell::read(dirs, temper_config_shell::Shape::Standalone).expect("init file").document,
    )
    .expect("JSON");
    let agent = temper_agent_fixtures::configuration(&document);
    assert!(document["application"]["clients"]["bundle"].is_null());
    assert!(document["application"]["clients"].as_object().expect("clients object").contains_key("public"));
    assert!(document["application"]["clients"]["public"].is_null());
    assert_eq!(agent["programs"]["claude_code"]["path"], serde_json::json!("fetched"));
    assert_eq!(written.missing_git, !git);
    assert!(document["application"].get("workers").is_none());
    let ports = check_listeners(&document, &agent, written, row);
    check_agent(dirs, agent);
    ports
}

fn check_listeners(document: &serde_json::Value, _agent: &serde_json::Value, written: &Written, row: &str) -> [u16; 2] {
    let application = &document["application"];
    let addresses = [&application["clients"]["listener"]["socket"], &application["webhooks"]["listener"]["socket"]]
        .map(|value| value.as_str().expect("address").parse::<std::net::SocketAddr>().expect("socket"));
    let ports = addresses.map(|address| address.port());
    for (index, address) in addresses.iter().enumerate() {
        assert_eq!(address.ip(), std::net::Ipv4Addr::LOCALHOST, "{row}: loopback listener");
        assert!(!ports[..index].contains(&address.port()), "{row}: two distinct reservations");
        assert!(TcpListener::bind(address).is_err(), "{row}: reservation held");
    }
    assert!(application.get("machine").is_none());
    assert!(application.get("demo").is_none(), "{row}");
    assert!(document["engine"].get("llm").is_none(), "{row}: providers are settings");
    assert_eq!(written.listeners.len(), ports.len(), "{row}: two held descriptors");
    for (fd, port) in &written.listeners {
        let address = skein_shell::listener_at(*fd).expect("held descriptor is listening");
        assert_eq!(address.port(), port.get(), "{row}: descriptor matches written port");
        assert!(ports.contains(&port.get()), "{row}: descriptor's port is in the document");
    }
    ports
}

fn check_agent(dirs: &Directories, mut document: serde_json::Value) {
    let config = path(&dirs.config);
    // The generated route is asserted above; metadata preflight needs no public DNS
    // or machine trust bundle (as in tests/support/owned.rs's local_agent).
    for endpoint in document["endpoints"].as_array_mut().expect("endpoints") {
        if endpoint["transport"] == "tls" {
            endpoint["address"] = "127.0.0.1:443".into();
            endpoint["trust_der"] = serde_json::json!(config.join("root.der"));
        }
    }
    fs::write(config.join("root.der"), skein_tls_kit::pki::ROOT).expect("fixture trust");
    let checked = config.join("agent-check.json");
    fs::write(&checked, serde_json::to_vec(&document).expect("agent JSON")).expect("metadata fixture");
    let agent = temper_worker_shell::agent::read(&checked).expect("written agent configuration parses");
    assert_eq!(
        u64::from(agent.tool_payload),
        document["profile"]["declared"]["tool_payload"].as_u64().expect("declared tool payload")
    );
}

fn check_files(dirs: &Directories, binary: &Path, written: &Written, row: &str) {
    let names = [
        ("worker.secret", true, Some(0o600)),
        ("demo/bin/git", true, None),
        (temper_config_shell::FILE, false, Some(0o644)),
    ];
    let expected: Vec<_> =
        names.iter().map(|(name, data, _)| path(if *data { &dirs.data } else { &dirs.config }).join(name)).collect();
    assert_eq!(written.paths, expected, "{row}: file order");
    for (file, (_, _, mode)) in written.paths.iter().zip(names) {
        match mode {
            Some(mode) => {
                assert_eq!(fs::metadata(file).expect("file").mode() & 0o777, mode, "{row}: {}", file.display());
            }
            None => assert_eq!(fs::read_link(file).expect("git link"), binary, "{row}"),
        }
    }
    assert!(!path(&dirs.data).join("demo/llm.key").exists(), "{row}: key belongs to the playground start");
    for directory in ["demo", "demo/bin"] {
        assert_eq!(
            fs::metadata(path(&dirs.data).join(directory)).expect("demo directory").mode() & 0o777,
            0o700,
            "{row}"
        );
    }
    for directory in [&dirs.config, &dirs.data, &dirs.cache] {
        assert_eq!(fs::metadata(path(directory)).expect("directory").mode() & 0o777, 0o700);
    }
    assert!(!path(&dirs.config).join("charters").exists());
}

#[test]
fn init_renders_only_listen_unless_its_next_port_is_taken() {
    use temper_config_shell::{FILE, FORMER, File, Shape, render};
    for (boxed, taken) in [(false, false), (true, false), (true, true)] {
        let root = scratch(&format!("one-line-{boxed}-{taken}"));
        let dirs = directories(&root);
        let (client, next) = (0..16)
            .find_map(|_| {
                let client = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("client candidate");
                let port = client.local_addr().expect("client address").port().checked_add(1)?;
                TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).ok().map(|next| (client, next))
            })
            .expect("a pair of consecutive free ports");
        let port = client.local_addr().expect("client address").port();
        drop(client);
        let next = taken.then_some(next);
        // Drop the other reservation before init when testing the default.
        let written = if boxed {
            write(&dirs, &[], &root.join("temper"), [port, 0])
        } else {
            write_bare(&dirs, &[], &root.join("temper"), [port, 0])
        }
        .expect("init");
        assert_eq!(written.listeners[0].1.get(), port);
        let expected = File {
            listen: Some(([127, 0, 0, 1], port).into()),
            webhooks: taken.then(|| ([127, 0, 0, 1], written.listeners[1].1.get()).into()),
            ..File::default()
        };
        let path = path(&dirs.config).join(FILE);
        assert_eq!(fs::read(&path).expect("file"), render(&expected, Shape::Standalone).as_ref());
        assert_eq!(fs::metadata(&path).expect("file mode").mode() & 0o777, 0o644);
        assert_eq!(written.paths.last(), Some(&path), "configuration is written last");
        assert!(!path.with_file_name(FORMER).exists());
        if taken {
            assert_ne!(written.listeners[1].1.get(), port.checked_add(1).expect("next port"));
        } else {
            assert_eq!(written.listeners[1].1.get(), port.checked_add(1).expect("next port"));
        }
        temper_service_shell::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
        drop(next);
        fs::remove_dir_all(root).expect("cleanup");
    }
}

#[test]
fn a_bare_configuration_passes_the_same_checks_as_serve() {
    for claude in [false, true] {
        let root = scratch(&format!("bare-acceptance-{claude}"));
        let dirs = directories(&root);
        let env = environment(&root, claude);
        let written = write_bare(&dirs, &env, &root.join("bin/temper"), [0; 2]).expect("bare init");
        let bytes =
            temper_config_shell::read(&dirs, temper_config_shell::Shape::Standalone).expect("init file").document;
        let deployment = setup::check(&bytes, &jig_directories(&dirs)).expect("bare document passes setup and named");
        let document: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(document["application"].get("machine").is_none());
        assert!(deployment.demo.is_some(), "every box supplies its scripted provider");
        assert!(!path(&dirs.data).join("demo").exists());
        assert_eq!(written.person, temper_init::Person::Found);
        assert!(written.login.is_none());
        assert_eq!(
            written.paths,
            [path(&dirs.data).join("worker.secret"), path(&dirs.config).join(temper_config_shell::FILE)]
        );
        let document: serde_json::Value = serde_json::from_slice(&bytes).expect("JSON");
        assert!(document["application"].get("welcome").is_none());
        let ports = ["clients", "webhooks"].map(|name| {
            document["application"][name]["listener"]["socket"]
                .as_str()
                .expect("socket")
                .parse::<std::net::SocketAddr>()
                .expect("address")
                .port()
        });
        assert_ne!(ports[0], ports[1]);
        assert_eq!(written.listeners.iter().map(|(_, port)| port.get()).collect::<Vec<_>>(), ports);
        for (_, port) in &written.listeners {
            assert!(TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port.get())).is_err(), "bare listener held");
        }
        let agent = temper_agent_fixtures::configuration(&document);
        assert_eq!(agent["programs"]["claude_code"]["path"], serde_json::json!("fetched"));
        check_agent(&dirs, agent);
        temper_service_shell::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
        fs::remove_dir_all(root).expect("cleanup");
    }
}

#[test]
fn init_keeps_an_existing_provider_key_even_without_openssl() {
    let root = scratch("kept-key");
    let dirs = directories(&root);
    dirs.open(skein_shell::effective_user()).unwrap();
    let key = path(&dirs.data).join("provider.key");
    // Only a marker is needed: init must not read or replace an existing file.
    fs::write(&key, b"kept").unwrap();
    fs::set_permissions(&key, fs::Permissions::from_mode(0o600)).unwrap();
    let before = fs::metadata(&key).unwrap();
    let env = environment(&root, false);
    let written = write(&dirs, &env, &root.join("temper"), [0; 2]).unwrap();
    let after = fs::metadata(&key).unwrap();
    assert_eq!(before.ino(), after.ino());
    assert_eq!(before.len(), after.len());
    assert!(written.no_sign_in.is_none());
    let value: serde_json::Value = serde_json::from_slice(
        &temper_config_shell::read(&dirs, temper_config_shell::Shape::Standalone).expect("init file").document,
    )
    .unwrap();
    assert_eq!(value["engine"]["clients"]["provider"]["key"], "provider.key");
    assert_eq!(value["engine"]["clients"]["provider"]["clients"], serde_json::json!([]));
    temper_service_shell::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn init_makes_a_private_3072_bit_provider_key_before_writing_its_document() {
    use std::process::{Command, Stdio};
    let root = scratch("provider-key");
    let dirs = directories(&root);
    let env = vec![(b"PATH".as_slice().into(), b"/usr/bin:/bin".as_slice().into())];
    let mut io = Io::open().unwrap();
    let Done::Usage(before) = io.perform(Op::Usage).unwrap() else { panic!("usage") };
    let clock = skein_shell::Clock::new();
    let started = clock.now().now;
    let written = write_with_key(&dirs, &env, &root.join("temper"), [0; 2], PROVIDER_KEY).unwrap();
    let elapsed = clock.now().now.saturating_since(started);
    let Done::Usage(after) = io.perform(Op::Usage).unwrap() else { panic!("usage") };
    eprintln!(
        "init provider key: wall {elapsed:?}, child user {} ns, child system {} ns",
        after.children.user.as_nanos().saturating_sub(before.children.user.as_nanos()),
        after.children.system.as_nanos().saturating_sub(before.children.system.as_nanos())
    );
    assert!(written.no_sign_in.is_none());
    assert_eq!(
        written.paths,
        [
            path(&dirs.data).join("worker.secret"),
            path(&dirs.data).join("provider.key"),
            path(&dirs.data).join("demo/bin/git"),
            path(&dirs.config).join(temper_config_shell::FILE),
        ],
        "the provider key is between the secret and launcher, before the configuration"
    );
    let key = path(&dirs.data).join("provider.key");
    assert_eq!(fs::metadata(&key).unwrap().mode() & 0o777, 0o600);
    let public = root.join("public.pem");
    assert!(
        Command::new("openssl")
            .args(["pkey", "-in"])
            .arg(&key)
            .arg("-pubout")
            .arg("-out")
            .arg(&public)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    );
    let details = Command::new("openssl")
        .args(["pkey", "-pubin", "-in"])
        .arg(&public)
        .args(["-text", "-noout"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(details.status.success());
    assert!(details.stdout.starts_with(b"Public-Key: (3072 bit)"));
    let value: serde_json::Value = serde_json::from_slice(
        &temper_config_shell::read(&dirs, temper_config_shell::Shape::Standalone).expect("init file").document,
    )
    .unwrap();
    assert_eq!(value["engine"]["clients"]["provider"]["key"], "provider.key");
    temper_service_shell::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    fs::remove_dir_all(root).unwrap();
}
