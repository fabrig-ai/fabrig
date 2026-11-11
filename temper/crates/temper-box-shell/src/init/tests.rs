use super::*;
use crate::directories::jig_directories;
use std::{fs, net::TcpListener, os::unix::fs::PermissionsExt};

fn scratch(row: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("temper-init-{}-{row}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("prior scratch");
    }
    fs::create_dir(&root).expect("scratch");
    root
}
fn directories(root: &Path) -> Directories {
    let paths = [root.join("config"), root.join("data"), root.join("cache")];
    let args = [
        b"--config".as_slice(),
        paths[0].as_os_str().as_bytes(),
        b"--data",
        paths[1].as_os_str().as_bytes(),
        b"--cache",
        paths[2].as_os_str().as_bytes(),
    ];
    crate::directories::resolve(&args.iter().map(|v| Box::from(*v)).collect::<Vec<_>>(), &[], b"/")
        .expect("explicit directories")
        .0
}
fn environment(root: &Path, claude: bool) -> Vec<EnvironmentEntry> {
    fs::create_dir(root.join("bin")).expect("bin");
    for name in if claude { vec!["git", "claude"] } else { vec!["git"] } {
        let path = root.join("bin").join(name);
        fs::write(&path, b"#!/bin/sh\nexit 0\n").expect("program");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("executable");
    }
    vec![(Box::from(b"PATH".as_slice()), root.join("bin").as_os_str().as_bytes().into())]
}
#[test]
fn path_lookup_uses_only_absolute_executable_candidates_in_order() {
    for (row, entries, executable, expected) in [
        ("absolute", "/one:/two", vec![true, true], Some("/one/git")),
        ("relative skipped", "relative::/two", vec![true], Some("/two/git")),
        ("none", "relative:", vec![], None),
        ("non executable", "/one:/two", vec![false, true], Some("/two/git")),
        ("no hit", "/one", vec![false], None),
    ] {
        let env = vec![(Box::from(b"PATH".as_slice()), Box::from(entries.as_bytes()))];
        let candidates = candidates(&env, b"git");
        assert_eq!(candidates.len(), executable.len(), "{row}");
        let facts: Vec<_> = candidates.into_iter().zip(executable).collect();
        assert_eq!(lookup(&env, b"git", &facts).as_deref(), expected.map(Path::new), "{row}");
    }
}
#[test]
fn taken_default_owned_and_webhooks_ports_get_distinct_kernel_ports() {
    for at in [1, 4] {
        let holder = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("hold preferred port");
        let preferred_port = holder.local_addr().expect("held address").port();
        let mut io = Io::open().expect("startup io");
        let mut preferred = [0; 5];
        preferred[at] = preferred_port;
        let held = io.ports(&preferred).expect("reserved ports");
        let chosen = held.iter().map(|(_, port)| *port).collect::<Vec<_>>();
        assert_ne!(chosen[at].get(), preferred_port);
        for (index, port) in chosen.iter().enumerate() {
            assert!(!chosen[..index].contains(port), "five distinct ports");
        }
        for (fd, _) in held {
            io.close(fd);
        }
        drop(holder);
    }
}

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
        let ports = check_deployment(&dirs, &binary, &first, &row, claude, git);
        for (at, available) in [!held, !held && !same, !held && !same, !held, !held && !same].into_iter().enumerate() {
            assert_eq!(ports[at] == preferred[at], available, "{row}: preferred port {at}");
        }
        let second_root = root.join("second");
        fs::create_dir(&second_root).expect("second deployment");
        let second_dirs = directories(&second_root);
        // The first deployment holds all five preferred ports while the second initializes.
        let second = write(&second_dirs, &env, &binary, ports).expect("second init beside the first");
        let second_ports = check_deployment(&second_dirs, &binary, &second, &row, claude, git);
        for port in second_ports {
            assert!(!ports.contains(&port), "{row}: deployments have disjoint ports");
        }
        drop(holder);
        for written in [first, second] {
            crate::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
        }
        fs::remove_dir_all(root).expect("cleanup");
    }
}

fn preferred_ports(held: bool, same: bool) -> ([u16; 5], Option<[TcpListener; 5]>) {
    let listeners = [0; 5].map(|_| TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("preferred listener"));
    let mut preferred = listeners.each_ref().map(|listener| listener.local_addr().expect("port").port());
    if same {
        preferred = [preferred[0], preferred[0], preferred[0], preferred[3], preferred[0]];
    }
    let holder = if held {
        Some(listeners)
    } else {
        drop(listeners);
        None
    };
    (preferred, holder)
}

fn check_deployment(
    dirs: &Directories,
    binary: &Path,
    written: &Written,
    row: &str,
    claude: bool,
    git: bool,
) -> [u16; 5] {
    check_files(dirs, binary, written, row);
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(dirs.document()).expect("document")).expect("JSON");
    let agent: serde_json::Value =
        serde_json::from_slice(&fs::read(path(&dirs.config).join("agent.json")).expect("agent")).expect("JSON");
    assert!(document["application"]["clients"]["bundle"].is_null());
    assert!(document["application"]["clients"].as_object().expect("clients object").contains_key("public"));
    assert!(document["application"]["clients"]["public"].is_null());
    assert_eq!(
        agent["programs"]["claude_code"]["path"],
        if claude { serde_json::json!(binary.parent().unwrap().join("claude")) } else { serde_json::json!("fetched") }
    );
    assert_eq!(written.missing_git, !git);
    assert_eq!(document["application"]["workers"][0]["git"].is_null(), !git);
    let ports = check_listeners(&document, &agent, written, row);
    check_agent(dirs, agent);
    ports
}

fn check_listeners(document: &serde_json::Value, agent: &serde_json::Value, written: &Written, row: &str) -> [u16; 5] {
    let application = &document["application"];
    let addresses = [
        &application["clients"]["listener"]["socket"],
        &application["machine"]["owned"]["listen"]["socket"],
        &application["machine"]["fake"]["listen"]["socket"],
        &agent["endpoints"][2]["address"],
        &application["webhooks"]["listener"]["socket"],
    ]
    .map(|value| value.as_str().expect("address").parse::<std::net::SocketAddr>().expect("socket"));
    let ports = addresses.map(|address| address.port());
    for (index, address) in addresses.iter().enumerate() {
        assert_eq!(address.ip(), std::net::Ipv4Addr::LOCALHOST, "{row}: loopback listener");
        assert!(!ports[..index].contains(&address.port()), "{row}: five distinct reservations");
        assert!(TcpListener::bind(address).is_err(), "{row}: reservation held");
    }
    for (name, address) in [("owned", addresses[1]), ("fake", addresses[2])] {
        assert_eq!(application["machine"][name]["remote"], format!("http://{address}"), "{row}");
        assert_eq!(application["machine"][name]["listen"], serde_json::json!({"socket": address.to_string()}), "{row}");
    }
    let llm = serde_json::json!({"socket": addresses[3].to_string()});
    assert_eq!(application["demo"]["llm"]["listen"], llm, "{row}");
    assert_eq!(document["engine"]["llm"]["endpoints"][2]["address"], llm, "{row}");
    assert_eq!(written.listeners.len(), ports.len(), "{row}: five held descriptors");
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
    document["endpoints"][0]["address"] = "127.0.0.1:443".into();
    document["endpoints"][0]["trust_der"] = serde_json::json!(config.join("root.der"));
    document["endpoints"][1]["address"] = "127.0.0.1:443".into();
    document["endpoints"][1]["trust_der"] = serde_json::json!(config.join("root.der"));
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
        ("worker.digest", true, Some(0o600)),
        ("demo/llm.key", true, Some(0o600)),
        ("demo/bin/git", true, None),
        ("agent.json", false, Some(0o644)),
        ("temper.json", false, Some(0o644)),
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
    let key = fs::read(path(&dirs.data).join("demo/llm.key")).expect("demo key");
    assert_eq!(key.len(), 65, "{row}: 64 digits and newline");
    assert!(key[..64].iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)), "{row}");
    assert_eq!(key[64], b'\n', "{row}");
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
fn init_refusals_leave_every_file_and_directory_untouched() {
    for (row, data, dangling) in [
        ("temper.json", false, false),
        ("worker.secret", true, false),
        ("worker.digest", true, false),
        ("demo/llm.key", true, false),
        ("demo/bin/git", true, true),
    ] {
        let root = scratch(&row.replace('/', "-"));
        let dirs = directories(&root);
        let env = environment(&root, false);
        let destination = path(if data { &dirs.data } else { &dirs.config }).join(row);
        fs::create_dir_all(destination.parent().expect("parent")).expect("parent");
        if dangling {
            std::os::unix::fs::symlink("/nonexistent/temper-init-demo", &destination).expect("dangling link");
        } else {
            fs::write(&destination, b"kept").expect("existing file");
        }
        let before = snapshot(&root);
        if !row.starts_with("demo/") {
            let why = write_bare(&dirs, &env, &root.join("bin/temper"), [0; 2]).expect_err("bare refused");
            assert_eq!(why, format!("{}: already exists", destination.display()));
            assert_eq!(snapshot(&root), before, "{row}: bare refusal changes nothing");
        }
        let why = write(&dirs, &env, &root.join("bin/temper"), [0, 0, 0, 0, 0]).expect_err("refused");
        assert_eq!(why, format!("{}: already exists", destination.display()));
        assert_eq!(snapshot(&root), before, "{row}: every file and directory untouched");
        fs::remove_dir_all(root).expect("cleanup");
    }
}

fn snapshot(root: &Path) -> Vec<(PathBuf, u32, Vec<u8>)> {
    let mut pending = vec![root.to_path_buf()];
    let mut entries = Vec::new();
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path).expect("entry");
        let content = if metadata.is_dir() {
            pending.extend(fs::read_dir(&path).expect("directory").map(|entry| entry.expect("child").path()));
            Vec::new()
        } else if metadata.is_symlink() {
            fs::read_link(&path).expect("link").as_os_str().as_bytes().to_vec()
        } else {
            fs::read(&path).expect("file")
        };
        entries.push((path, metadata.mode(), content));
    }
    entries.sort();
    entries
}

#[test]
fn generated_configuration_passes_the_same_checks_as_serve() {
    let root = scratch("acceptance");
    let dirs = directories(&root);
    let env = environment(&root, true);
    let written = write(&dirs, &env, &root.join("bin/temper"), [0, 0, 0, 0, 0]).expect("init");
    let bytes = fs::read(dirs.document()).expect("document");
    let deployment =
        crate::setup::check(&bytes, &jig_directories(&dirs)).expect("init document passes setup and named");
    let machine = crate::box_command::machine(&bytes, &dirs).expect("machine");
    assert!(machine.owned.is_some());
    assert!(machine.fake.is_some());
    let demo = deployment.demo.expect("init declares the demo");
    assert_eq!(demo.provider(), 3);
    crate::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn the_person_diagnostic_names_the_login_or_the_uid() {
    use temper_init::Person::{FellBack, Found};
    for (person, login, expected) in [
        (Found, Some(b"ada".as_slice()), None),
        (
            FellBack,
            Some(b"Admin".as_slice()),
            Some("init: Admin cannot be a Forgejo login: forge 3's person is person"),
        ),
        (FellBack, None, Some("init: uid 42 has no login in /etc/passwd: forge 3's person is person")),
    ] {
        assert_eq!(person_line(person, login, 42).as_deref(), expected);
    }
}

#[test]
fn explicit_program_fact_precedes_path_discovery() {
    let root = scratch("program-fact");
    let dirs = directories(&root);
    let env = environment(&root, true);
    let binary = root.join("temper");
    let adapter = root.join("fake-claude");
    let written = write_with_program(&dirs, &env, &binary, [0; 5], Some(&adapter)).expect("injected init fact");
    let agent: serde_json::Value =
        serde_json::from_slice(&fs::read(path(&dirs.config).join("agent.json")).expect("agent")).expect("JSON");
    assert_eq!(agent["programs"]["claude_code"]["path"], adapter.to_str().expect("path"));
    crate::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    fs::remove_dir_all(root).expect("fixture cleanup");
}

#[test]
fn a_bare_configuration_passes_the_same_checks_as_serve() {
    for claude in [false, true] {
        let root = scratch(&format!("bare-acceptance-{claude}"));
        let dirs = directories(&root);
        let env = environment(&root, claude);
        let written = write_bare(&dirs, &env, &root.join("bin/temper"), [0; 2]).expect("bare init");
        let bytes = fs::read(dirs.document()).expect("document");
        let deployment =
            crate::setup::check(&bytes, &jig_directories(&dirs)).expect("bare document passes setup and named");
        let machine = crate::box_command::machine(&bytes, &dirs).expect("machine");
        assert!(machine.owned.is_none());
        assert!(machine.fake.is_none());
        assert!(deployment.demo.is_none());
        assert!(!path(&dirs.data).join("demo").exists());
        assert_eq!(written.person, temper_init::Person::Found);
        assert!(written.login.is_none());
        assert_eq!(
            written.paths,
            [
                path(&dirs.data).join("worker.secret"),
                path(&dirs.data).join("worker.digest"),
                path(&dirs.config).join("agent.json"),
                dirs.document()
            ]
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
        let agent: serde_json::Value =
            serde_json::from_slice(&fs::read(path(&dirs.config).join("agent.json")).expect("agent")).expect("JSON");
        assert_eq!(
            agent["programs"]["claude_code"]["path"],
            if claude { serde_json::json!(root.join("bin/claude")) } else { serde_json::json!("fetched") }
        );
        check_agent(&dirs, agent);
        crate::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
        fs::remove_dir_all(root).expect("cleanup");
    }
}

#[test]
fn a_path_without_openssl_writes_a_box_without_sign_in_and_keeps_running() {
    let root = scratch("no-openssl");
    let dirs = directories(&root);
    let env = environment(&root, false);
    let written = write(&dirs, &env, &root.join("temper"), [0; 5]).expect("init succeeds without openssl");
    assert_eq!(written.no_sign_in.as_deref(), Some("openssl is not on PATH"));
    let value: serde_json::Value = serde_json::from_slice(&fs::read(dirs.document()).unwrap()).unwrap();
    assert!(value["engine"]["clients"]["provider"].is_null());
    assert!(value["application"]["machine"]["owned"]["sign_in"].is_null());
    assert!(!path(&dirs.data).join("provider.key").exists());
    crate::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    fs::remove_dir_all(root).unwrap();
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
    let written = write(&dirs, &env, &root.join("temper"), [0; 5]).unwrap();
    let after = fs::metadata(&key).unwrap();
    assert_eq!(before.ino(), after.ino());
    assert_eq!(before.len(), after.len());
    assert!(written.no_sign_in.is_none());
    let value: serde_json::Value = serde_json::from_slice(&fs::read(dirs.document()).unwrap()).unwrap();
    assert_eq!(value["engine"]["clients"]["provider"]["key"], "provider.key");
    assert_eq!(value["application"]["machine"]["owned"]["sign_in"], "forgejo");
    crate::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
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
    let written = write(&dirs, &env, &root.join("temper"), [0; 5]).unwrap();
    let elapsed = clock.now().now.saturating_since(started);
    let Done::Usage(after) = io.perform(Op::Usage).unwrap() else { panic!("usage") };
    eprintln!(
        "init provider key: wall {elapsed:?}, child user {} ns, child system {} ns",
        after.children.user.as_nanos().saturating_sub(before.children.user.as_nanos()),
        after.children.system.as_nanos().saturating_sub(before.children.system.as_nanos())
    );
    assert!(written.no_sign_in.is_none());
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
    let value: serde_json::Value = serde_json::from_slice(&fs::read(dirs.document()).unwrap()).unwrap();
    assert_eq!(value["engine"]["clients"]["provider"]["key"], "provider.key");
    crate::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    fs::remove_dir_all(root).unwrap();
}
