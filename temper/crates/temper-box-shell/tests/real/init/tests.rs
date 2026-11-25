use skein_io::kernel::{Done, Op};
use std::{fs, net::TcpListener, os::unix::fs::MetadataExt};
use temper_box_shell::{
    directories::jig_directories,
    setup,
    testing::{
        init::{Io, write, write_with_key},
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
    use jig_protocol_config::Address::Addr;

    let root = scratch("native-init");
    let dirs = directories(&root);
    let env = environment(&root, false);
    let binary = root.join("bin/temper");
    let first = write(&dirs, &env, &binary, [0; 2]).expect("first init");
    for directory in [
        path(&dirs.config).to_owned(),
        path(&dirs.cache).to_owned(),
        path(&dirs.data).to_owned(),
        path(&dirs.data).join("demo"),
        path(&dirs.data).join("demo/bin"),
    ] {
        assert_eq!(fs::metadata(&directory).expect("directory").mode() & 0o777, 0o700);
    }
    let config = path(&dirs.config).join(temper_config_shell::FILE);
    for (file, mode) in [(path(&dirs.data).join("worker.secret"), 0o600), (config, 0o644)] {
        assert_eq!(fs::metadata(&file).expect("file").mode() & 0o777, mode);
    }
    assert_eq!(fs::read_link(path(&dirs.data).join("demo/bin/git")).expect("git link"), binary);

    let read = temper_config_shell::read(&dirs, temper_config_shell::Shape::Standalone).expect("init file");
    let named = setup::check(&read.document, &jig_directories(&dirs)).expect("setup admits the init file").named;
    let [Addr(clients), Addr(webhooks)] = [named.clients, named.webhooks.expect("webhooks").listener] else {
        panic!("init writes socket addresses");
    };
    let addresses = [clients, webhooks].map(|addr| addr.socket().expect("TCP listener"));
    assert_ne!(addresses[0], addresses[1], "both listeners have their own port");
    assert_eq!(first.listeners.len(), addresses.len());
    for ((fd, port), address) in first.listeners.iter().zip(addresses) {
        assert_eq!(skein_shell::listener_at(*fd).expect("held descriptor is listening"), address);
        assert_eq!(port.get(), address.port(), "descriptor matches written port");
        assert!(TcpListener::bind(address).is_err(), "reservation held");
    }
    let ports = addresses.map(|address| address.port());
    let second_root = root.join("second");
    fs::create_dir(&second_root).expect("second deployment");
    let second = write(&directories(&second_root), &env, &binary, ports).expect("second init beside the first");
    assert_eq!(second.listeners.len(), 2);
    for (_, port) in &second.listeners {
        assert!(!ports.contains(&port.get()), "deployments have disjoint ports");
    }
    for written in [first, second] {
        temper_service_shell::close(written.listeners.into_iter().map(|(fd, _)| fd).collect());
    }
    fs::remove_dir_all(root).expect("cleanup");
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
