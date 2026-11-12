use super::*;
use crate::directories::jig_directories;
use std::ffi::OsString;

const PAGE: &[u8] = b"<!doctype html><html><head></head><body>effective fixture</body></html>";
struct Scratch(PathBuf);
impl Scratch {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("temper-fake-backends-{}-{name}", std::process::id()));
        fs::create_dir(&path).expect("new private scratch");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("private scratch");
        Self(path)
    }
    fn dirs(&self) -> Directories {
        Directories {
            config: bytes(&self.0.join("source-config")),
            data: bytes(&self.0.join("source-data")),
            cache: bytes(&self.0.join("cache")),
            origins: [skein_shell::Origin::Flag; 3],
        }
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove private fixture");
    }
}
fn read_json(path: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).expect("fixture bytes")).expect("fixture JSON")
}
fn store_json(path: &Path, value: &serde_json::Value) {
    write(path, &serde_json::to_vec(value).expect("fixture JSON"), true).expect("write fixture JSON");
}
fn initialized_source(scratch: &Scratch) -> Directories {
    let source = scratch.dirs();
    let binary = std::env::current_exe().expect("fixture binary");
    // Optional executables are explicitly absent, independent of the test runner's PATH.
    let initialized = crate::init::write(&source, &[], &binary, [0; 5]).expect("source init without PATH");
    crate::close(initialized.listeners.into_iter().map(|(fd, _)| fd).collect());
    for path in initialized.paths {
        if fs::symlink_metadata(&path).expect("fixture metadata").is_file() {
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("private fixture file");
        }
    }
    source
}
// A unit test binary has no fake-claude sibling: each scratch provisions an adapter beside its source directories.
fn start(source: &Directories, bundle: Option<&Path>) -> Result<Prepared, String> {
    start_chosen(source, bundle, None)
}
fn start_chosen(source: &Directories, bundle: Option<&Path>, chosen: Option<&Chosen>) -> Result<Prepared, String> {
    start_options(source, &Options { bundle: bundle.map(Path::to_path_buf), ..Options::default() }, chosen)
}
fn start_options(source: &Directories, options: &Options, chosen: Option<&Chosen>) -> Result<Prepared, String> {
    let adapter = crate::path(&source.config).parent().expect("scratch root").join("fake-claude");
    if !adapter.exists() {
        write(&adapter, b"#!/bin/sh\nexit 1\n", false).expect("fixture adapter");
        fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).expect("executable fixture adapter");
    }
    // Use the source fixture's explicit absence of optional executables.
    prepare(source, options, &adapter, chosen, &[])
}
fn provision_bundle(path: &Path) -> PathBuf {
    make_private(&path.join("0000000000000001"), skein_shell::effective_user()).expect("private fixture bundle");
    write(&path.join("index.html"), PAGE, false).expect("fixture page");
    write(&path.join("0000000000000001/main.js"), b"// fixture\n", false).expect("fixture script");
    path.to_path_buf()
}
fn provisioned_source(scratch: &Scratch) -> Directories {
    let source = initialized_source(scratch);
    let bundle = provision_bundle(&scratch.0.join("bundle-loopback"));
    let mut temper = read_json(&source.document());
    temper["application"]["clients"]["bundle"] = bundle.to_str().expect("fixture bundle path").into();
    store_json(&source.document(), &temper);
    source
}

fn stock_box_source(scratch: &Scratch, sign_in: bool) -> Directories {
    let source = scratch.dirs();
    make_private(crate::path(&source.config), skein_shell::effective_user()).expect("private source config");
    let binary = std::env::current_exe().expect("fixture binary");
    let port = NonZeroU16::new(7100).expect("nonzero fixture port");
    let answer = temper_init::files(
        &temper_init::Facts {
            binary: binary.as_os_str().as_bytes(),
            git: None,
            claude: None,
            ports: temper_init::Ports {
                clients: port,
                owned: NonZeroU16::new(8370).expect("owned port"),
                forge: NonZeroU16::new(8371).expect("forge port"),
                llm: NonZeroU16::new(8372).expect("model port"),
                webhooks: NonZeroU16::new(8373).expect("webhooks port"),
            },
            login: Some(b"ada"),
            sign_in,
        },
        &temper_init::Directories { config: &source.config, data: &source.data },
        &[0; 32],
        &[0; 32],
    )
    .expect("stock box files");
    for file in answer.files {
        if file.name == b"temper.json" || file.name == b"agent.json" {
            let temper_init::Content::Bytes { bytes, .. } = file.content else { panic!("public document") };
            write(&crate::path(&source.config).join(crate::path(file.name)), &bytes, false)
                .expect("unchanged public init document");
        }
    }
    source
}

#[test]
fn untouched_box_init_imports_with_and_without_sign_in() {
    for sign_in in [false, true] {
        let scratch = Scratch::new(if sign_in { "stock-sign-in" } else { "stock-no-sign-in" });
        let source = stock_box_source(&scratch, sign_in);
        let original = fs::read(source.document()).expect("source public bytes");
        let imported = Source::read(&source, &[]).expect("untouched box init imports");
        let (_, effective) = directories(&source).expect("sandbox directories");
        let (temper, agent) = imported.rebase(&effective).expect("rebase public paths").expect("source documents");
        let rebased: serde_json::Value = serde_json::from_slice(&temper).expect("rebased document");
        let original_tree: serde_json::Value = serde_json::from_slice(&original).expect("source document");
        for pointer in ["/engine/clients/provider", "/application/machine/owned/sign_in"] {
            assert_eq!(rebased.pointer(pointer), original_tree.pointer(pointer), "{pointer}");
        }
        let checked = crate::setup::check(&temper, &jig_directories(&effective)).expect("ordinary effective admission");
        assert_eq!(checked.named.deployment, bytes(&crate::path(&effective.data).join("deployment")));
        assert_rebased_box_paths(&Json::decode(&temper).expect("effective JSON"), &agent, &effective, sign_in);
        assert_eq!(fs::read(source.document()).expect("source unchanged"), original);
        assert!(!crate::path(&source.data).exists(), "source resources and key are never needed");
        assert!(!crate::path(&effective.data).exists(), "import and rebase only inspect public documents");
    }
}

fn assert_rebased_box_paths(temper: &Json, agent: &[u8], effective: &Directories, sign_in: bool) {
    for (pointer, name) in [
        ("engine/hosts/workers/0/digest", "worker.digest"),
        ("workers/0/secret", "worker.secret"),
        ("application/machine/fake/git", "demo/bin/git"),
        ("application/demo/llm/key", "demo/llm.key"),
        ("application/workers/0/root", "workspaces"),
    ] {
        assert_eq!(
            resolved(crate::path(&effective.data), temper.string(pointer).expect("resource path")).expect("resolved"),
            crate::path(&effective.data).join(name),
            "{pointer}"
        );
    }
    let agent_path = bytes(&crate::path(&effective.config).join("agent.json"));
    assert_eq!(temper.string("workers/0/agent/configuration").expect("agent config"), agent_path.as_ref());
    assert_eq!(temper.string("workers/0/agent/arguments/2").expect("agent argument"), agent_path.as_ref());
    let agent = Json::decode(agent).expect("rebased agent");
    assert_eq!(
        agent.string("programs/claude_code/directory").expect("program directory"),
        bytes(&crate::path(&effective.data).join("programs")).as_ref()
    );
    if sign_in {
        assert_eq!(
            resolved(crate::path(&effective.data), temper.string("engine/clients/provider/key").expect("key path"))
                .expect("effective provider key"),
            crate::path(&effective.data).join("provider.key")
        );
    }
}

#[test]
fn modified_box_sign_in_is_refused() {
    let scratch = Scratch::new("modified-sign-in");
    let source = stock_box_source(&scratch, true);
    let stock = read_json(&source.document());
    for (pointer, value) in [
        ("/engine/clients/provider", serde_json::Value::Null),
        ("/application/machine/owned/sign_in", serde_json::Value::Null),
        ("/application/machine/owned/sign_in", serde_json::json!("other")),
        ("/engine/clients/provider/key", serde_json::json!("/source/provider.key")),
        ("/engine/clients/provider/key", serde_json::json!("other.key")),
        ("/engine/clients/provider/clients/0/id", serde_json::json!("other")),
        ("/engine/clients/provider/clients/0/secret", serde_json::json!("custom")),
        ("/engine/clients/provider/clients/0/redirect_uri", serde_json::json!("http://127.0.0.1:9999/callback")),
        ("/engine/clients/provider/addresses", serde_json::json!([])),
    ] {
        let mut changed = stock.clone();
        *changed.pointer_mut(pointer).expect("stock member") = value;
        store_json(&source.document(), &changed);
        let error = Source::read(&source, &[]).err().unwrap_or_else(|| panic!("{pointer}: modified sign-in accepted"));
        let refused = if pointer == "/engine/clients/provider" {
            "application/machine/owned/sign_in"
        } else {
            pointer.trim_start_matches('/')
        };
        assert!(error.contains(refused), "{pointer}: {error}");
    }
    assert!(!crate::path(&source.data).exists(), "refusal inspects no source resource");
}

#[test]
fn program_login_line_names_the_model_steps_placeholder_and_quotes_effective_directories() {
    let dirs = Directories {
        config: b"/effective owner's/config".as_slice().into(),
        data: b"/effective data".as_slice().into(),
        cache: b"/shared cache".as_slice().into(),
        origins: [skein_shell::Origin::Flag; 3],
    };
    assert_eq!(
        program_login_line(&dirs, Path::new("/bin owner's/temper")),
        "'/bin owner'\\''s/temper' program-login --config '/effective owner'\\''s/config' --data '/effective data' --cache '/shared cache' NAME (the name the Model step shows)"
    );
}

#[test]
fn fake_backends_is_an_additive_box_only_switch() {
    let environment = [(Box::<[u8]>::from(b"HOME".as_slice()), Box::<[u8]>::from(b"/fixture/home".as_slice()))];
    for (words, expected) in [
        (vec!["--fake-backends"], true),
        (vec!["--no-tui", "--fake-backends", "--bundle", "/bundle"], true),
        (vec!["--fake-backends", "--no-tui", "--bundle", "/bundle"], true),
        (vec!["--fake-backends", "--bundle", "/bundle", "--no-tui"], true),
        (vec!["--no-tui", "--fake-backends", "--no-tui"], false),
        (vec!["--fake-backends", "--config", "/c", "--data", "/d", "--cache", "/k"], true),
        (vec!["--fake-backends", "--fake-backends"], false),
        (vec!["--fake-backends", "extra"], false),
        (vec!["serve", "--fake-backends"], false),
        (vec!["init", "--fake-backends"], false),
    ] {
        let args = words.iter().map(|s| Box::<[u8]>::from(s.as_bytes())).collect::<Vec<_>>();
        let result = crate::configure_with(&args, &environment, b"/fixture");
        assert_eq!(matches!(result, Ok(crate::Command::BoxFakeBackends { .. })), expected, "{words:?}: {result:?}");
    }
    let args = [];
    assert!(matches!(crate::configure_with(&args, &environment, b"/fixture"), Ok(crate::Command::Box { .. })));
}

fn configure(words: &[&[u8]]) -> Result<crate::Command, String> {
    let environment = [(Box::<[u8]>::from(b"HOME".as_slice()), Box::<[u8]>::from(b"/fixture/home".as_slice()))];
    let args = words.iter().map(|word| Box::<[u8]>::from(*word)).collect::<Vec<_>>();
    crate::configure_with(&args, &environment, b"/fixture")
}

#[test]
fn script_flags_resolve_in_order_and_require_fake_backends() {
    let command = configure(&[
        b"--scripts",
        b"first.json",
        b"--config",
        b"/c",
        b"--strict",
        b"--fake-backends",
        b"--scripts",
        b"/second.json",
    ])
    .expect("repeatable scripts with interleaved directory settings");
    let crate::Command::BoxFakeBackends { scripts, strict, .. } = command else { panic!("fake box") };
    assert_eq!(scripts, [PathBuf::from("/fixture/first.json"), PathBuf::from("/second.json")]);
    assert!(strict);
    assert!(matches!(
        configure(&[b"--fake-backends", b"--strict"]),
        Ok(crate::Command::BoxFakeBackends { strict: true, .. })
    ));
    for words in [
        vec![b"--scripts".as_slice(), b"first.json"],
        vec![b"--strict"],
        vec![b"--fake-backends", b"--scripts"],
        vec![b"--fake-backends", b"--scripts", b""],
        vec![b"--fake-backends", b"--scripts", b"bad\0path"],
        vec![b"--fake-backends", b"--strict", b"--strict"],
    ] {
        assert!(configure(&words).is_err(), "{words:?}");
    }
}

#[test]
fn given_scripts_join_at_each_start_without_replacing_the_model() {
    let scratch = Scratch::new("scripts-each-start");
    let source = provisioned_source(&scratch);
    let a = scratch.0.join("a.json");
    let b = scratch.0.join("b.json");
    fs::write(
        &a,
        br#"{"cued":[{"cue":"A","turns":["a"]}],"asked":[{"cue":"A","turns":["asked a"]}],"otherwise":["X"]}"#,
    )
    .expect("first scripts");
    fs::write(&b, br#"{"cued":[{"cue":"B","turns":["b"]}],"asked":[{"cue":"B","turns":["asked b"]}]}"#)
        .expect("second scripts");
    let options = Options { scripts: vec![a, b.clone()], strict: true, ..Options::default() };
    let fresh = start_options(&source, &options, None).expect("fresh scripts");
    let model = crate::path(&fresh.directories.config).join("llm.json");
    let original = fs::read(&model).expect("generated model document");
    let joined = crate::path(&fresh.directories.config).join("llm-scripts.json");
    assert_eq!(fs::metadata(&joined).expect("join metadata").mode() & 0o777, 0o600);
    assert_eq!(
        read_json(&joined),
        serde_json::json!({
            "cued":[{"cue":"A","turns":["a"]},{"cue":"B","turns":["b"]}],
            "asked":[{"cue":"A","turns":["asked a"]},{"cue":"B","turns":["asked b"]}],
            "otherwise":["X"]
        })
    );
    assert_model_script_arguments(&fresh, Some(&joined), true);
    drop(fresh);
    // Last present fallback replaces; an absent fallback above did not erase X.
    fs::write(&b, br#"{"otherwise":[]}"#).expect("replace second scripts");
    let retained = start_options(&source, &options, None).expect("retained scripts");
    assert_eq!(read_json(&joined)["otherwise"], serde_json::json!([]));
    assert_model_script_arguments(&retained, Some(&joined), true);
    assert_eq!(fs::read(&model).expect("retained model"), original);
    drop(retained);
    let plain = start(&source, None).expect("later start without scripts");
    assert!(!joined.exists());
    assert_model_script_arguments(&plain, None, false);
    assert_eq!(fs::read(&model).expect("retained model"), original);
    drop(plain);
    let strict = start_options(&source, &Options { strict: true, ..Options::default() }, None).expect("strict alone");
    assert_model_script_arguments(&strict, None, true);
    assert!(!joined.exists());
    assert_eq!(fs::read(&model).expect("retained model"), original);
}

fn assert_model_script_arguments(prepared: &Prepared, scripts: Option<&Path>, strict: bool) {
    for step in &prepared.steps {
        let mut expected = Vec::new();
        if step.name() == b"fake model" {
            if let Some(path) = scripts {
                expected.extend([Box::from(b"--scripts".as_slice()), bytes(path)]);
            }
            if strict {
                expected.push(Box::from(b"--strict".as_slice()));
            }
        }
        assert_eq!(step.arguments().get(6..).expect("base peer arguments"), expected);
    }
}

#[test]
fn script_join_refuses_the_file_before_start() {
    let scratch = Scratch::new("scripts-refused");
    let source = provisioned_source(&scratch);
    let prepared = start(&source, None).expect("sandbox");
    let model = crate::path(&prepared.directories.config).join("llm.json");
    let original = fs::read(&model).expect("model document");
    let joined = crate::path(&prepared.directories.config).join("llm-scripts.json");
    drop(prepared);
    let first = scratch.0.join("first.json");
    let second = scratch.0.join("second.json");
    let options = Options { scripts: vec![first.clone(), second.clone()], ..Options::default() };
    // Each individual group fits; their joined list crosses the document reader's bound.
    let full = (0..temper_fake::ENTRIES)
        .map(|i| serde_json::json!({"cue":format!("cue {i}"),"turns":["ok"]}))
        .collect::<Vec<_>>();
    for (a, b, detail) in [
        (
            serde_json::json!({"cued":[{"cue":"same","turns":["a"]}]}),
            Some(serde_json::json!({"cued":[{"cue":"same","turns":["b"]}]})),
            "unique",
        ),
        (
            serde_json::json!({"asked":[{"cue":"same","turns":["a"]}]}),
            Some(serde_json::json!({"asked":[{"cue":"same","turns":["b"]}]})),
            "unique",
        ),
        (serde_json::json!({}), Some(serde_json::json!({"mystery":[]})), "mystery"),
        (serde_json::json!({"cued":full}), Some(serde_json::json!({"cued":[{"cue":"extra","turns":["b"]}]})), "bound"),
        (serde_json::json!({}), None, "second.json"),
    ] {
        fs::write(&first, serde_json::to_vec(&a).expect("first JSON")).expect("first scripts");
        if let Some(b) = b {
            fs::write(&second, serde_json::to_vec(&b).expect("second JSON")).expect("second scripts");
        } else {
            fs::remove_file(&second).expect("unreadable second file");
        }
        let error = start_options(&source, &options, None).err().expect("refuse before steps start");
        assert!(error.contains(second.to_str().expect("path")), "{error}");
        assert!(error.contains(detail), "{error}");
        assert!(!joined.exists(), "no unvalidated join published");
        assert_eq!(fs::read(&model).expect("retained model"), original);
    }
}

#[test]
fn bundle_paths_resolve_against_the_invocation_directory() {
    for (given, expected) in
        [(b"assets".as_slice(), "/fixture/assets"), (b"/bundles/loopback".as_slice(), "/bundles/loopback")]
    {
        let command = configure(&[b"--fake-backends", b"--bundle", given]).expect("bundle arguments");
        let crate::Command::BoxFakeBackends { bundle, .. } = command else { panic!("fake box") };
        assert_eq!(bundle, Some(PathBuf::from(expected)));
    }
}

#[test]
fn bundle_and_fake_flags_allow_interleaved_directory_settings() {
    let cases: &[&[&[u8]]] = &[
        &[b"--bundle", b"assets", b"--config", b"/c", b"--fake-backends", b"--data", b"/d", b"--cache", b"/k"],
        &[b"--data", b"/d", b"--fake-backends", b"--cache", b"/k", b"--bundle", b"assets", b"--config", b"/c"],
    ];
    for words in cases {
        let command = configure(words).expect("interleaved bundle and directory settings");
        let crate::Command::BoxFakeBackends { directories, bundle, .. } = command else { panic!("fake box") };
        assert_eq!(bundle, Some(PathBuf::from("/fixture/assets")));
        assert_eq!(directories.config.as_ref(), b"/c");
        assert_eq!(directories.data.as_ref(), b"/d");
        assert_eq!(directories.cache.as_ref(), b"/k");
    }
}

#[test]
fn malformed_bundle_grammar_is_refused() {
    let cases: &[&[&[u8]]] = &[
        &[b"--bundle", b"assets"],
        &[b"--fake-backends", b"--bundle"],
        &[b"--bundle", b"--fake-backends"],
        &[b"--fake-backends", b"--bundle", b"assets", b"extra"],
        &[b"--fake-backends", b"--bundle", b"assets", b"--bundle", b"other"],
        &[b"--fake-backends", b"--bundle", b"assets", b"--fake-backends"],
        &[b"serve", b"--fake-backends", b"--bundle", b"assets"],
        &[b"init", b"--bundle", b"assets"],
    ];
    for words in cases {
        assert!(configure(words).is_err(), "{words:?}");
    }
}

#[test]
fn bundle_paths_check_both_sides_of_the_absolute_bound() {
    // protocol/configuration/directories.md: the resolved absolute path is bounded at 4096 bytes.
    const PATH_BYTES: usize = 4096;
    const WORKING: &[u8] = b"/fixture/";
    let mut absolute = vec![b'a'; PATH_BYTES];
    absolute[0] = b'/';
    let relative = vec![b'a'; PATH_BYTES - WORKING.len()];
    for (given, accepted) in [
        (absolute.clone(), true),
        ([absolute, b"a".to_vec()].concat(), false),
        (relative.clone(), true),
        ([relative, b"a".to_vec()].concat(), false),
        (Vec::new(), false),
        (b"assets\0other".to_vec(), false),
    ] {
        assert_eq!(configure(&[b"--fake-backends", b"--bundle", &given]).is_ok(), accepted);
    }
}

fn prepare_empty(scratch: &Scratch, source: &Directories) -> Prepared {
    // Init holds five ordinary listeners; the four welcome peers each add one.
    const INIT_LISTENERS: usize = 9;
    let supplied = provision_bundle(&scratch.0.join("explicit-bundle"));
    let prepared = start(source, Some(&supplied)).expect("init directly into effective directories");
    let tree = read_json(&prepared.directories.document());
    assert_eq!(tree["application"]["clients"]["bundle"], supplied.to_str().expect("fixture bundle path"));
    assert!(crate::path(&prepared.directories.data).join("worker.secret").is_file());
    assert!(!source.document().exists());
    assert_eq!(prepared.listeners.0.len(), INIT_LISTENERS);
    for (_, port) in &prepared.listeners.0 {
        assert!(TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port.get())).is_err(), "listener stays held");
    }
    validate(&prepared.directories, &prepared.bytes, &scratch.0.join("fake-claude"))
        .expect("ordinary effective config and agent checks");
    prepared
}

#[test]
fn a_reservation_that_loses_its_port_takes_another() {
    let mut calls = 0;
    let listener = reserve_from(&[], || {
        calls += 1;
        if calls == 1 {
            Err(std::io::ErrorKind::AddrInUse.into())
        } else {
            assert_eq!(calls, 2, "one replacement bind");
            TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        }
    })
    .expect("another reservation after losing a port");
    assert_eq!(calls, 2);
    let address = listener.local_addr().expect("held address");
    assert_ne!(address.port(), 0);
    assert!(TcpListener::bind(address).is_err(), "the reported port is held");
}

#[test]
fn a_reservation_refuses_at_once_on_any_other_error() {
    let mut calls = 0;
    let error = reserve_from(&[], || {
        calls += 1;
        assert_eq!(calls, 1, "a non-address error must not retry");
        Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "injected permission denial"))
    })
    .expect_err("permission denied refuses the reservation");
    assert_eq!(calls, 1);
    assert_eq!(error, "fake peer reservation: injected permission denial");
}

#[test]
fn a_reservation_refuses_after_32_lost_ports() {
    let mut calls = 0;
    let error = reserve_from(&[], || {
        calls += 1;
        assert!(calls <= 32, "lost ports must exhaust the retry bound");
        Err(std::io::ErrorKind::AddrInUse.into())
    })
    .expect_err("all ports lost");
    assert_eq!(calls, 32);
    assert_eq!(error, "fake peer reservation: could not choose a port distinct from the deployment and callback");
}

#[test]
fn peer_reservations_transfer_once_and_close_with_preparation() {
    let scratch = Scratch::new("peer-ownership");
    let source = scratch.dirs();
    let prepared = prepare_empty(&scratch, &source);
    let peers = prepared.peers;
    for address in [peers.github, peers.oauth, peers.llm, peers.forgejo] {
        assert_eq!(prepared.listeners.0.iter().filter(|(_, port)| port.get() == address.port()).count(), 1);
        assert!(TcpListener::bind(address).is_err(), "peer reservation remains owned");
    }
    drop(prepared);
    // Hold the released addresses so reuse would fail if it tried to bind any peer.
    let held: Vec<TcpListener> = [peers.github, peers.oauth, peers.llm, peers.forgejo]
        .into_iter()
        .map(|address| TcpListener::bind(address).expect("owned peer fd closes on preparation drop"))
        .collect();
    let reused = start(&source, None).expect("retained configuration");
    assert_eq!(reused.peers, peers);
    assert!(reused.listeners.0.is_empty(), "reuse leaves binding to the ordinary box");
    drop(held);
}

#[test]
fn retained_peer_edits_are_checked_without_replacement() {
    let scratch = Scratch::new("peer-validation");
    let source = provisioned_source(&scratch);
    let first = start(&source, None).expect("fresh sandbox");
    let config = crate::path(&first.directories.config).to_path_buf();
    let peers = first.peers;
    let (ordinary, _) =
        crate::box_command::configuration_steps(&first.directories, &first.bytes, Box::new([])).expect("steps");
    let ordinary_socket =
        read_json(&first.directories.document())["application"]["clients"]["listener"]["socket"].clone();
    let mut first = Some(first);
    for kind in ["edited", "zero", "collision", "ordinary", "outside", "oauth"] {
        let file = if kind == "oauth" { "oauth.json" } else { "llm.json" };
        let path = config.join(file);
        let original = fs::read(&path).expect("original public peer document");
        let mut edited: serde_json::Value = serde_json::from_slice(&original).expect("fixture JSON");
        let mut reservation = None;
        match kind {
            "edited" => {
                let mut taken = vec![1455, peers.github.port(), peers.oauth.port(), peers.forgejo.port()];
                for step in ordinary
                    .always
                    .iter()
                    .chain(ordinary.asked.iter().flat_map(|group| group.steps.iter().map(|(_, step)| step)))
                {
                    taken.extend(step.listeners().iter().map(SocketAddr::port));
                    if let Some(ready) = step.ready() {
                        taken.push(ready.port());
                    }
                }
                let socket = reserve(&taken).expect("edited peer port");
                edited["listen"]["socket"] = addr(&socket).expect("socket").to_string().into();
                reservation = Some(socket);
            }
            "zero" => edited["listen"]["socket"] = "127.0.0.1:0".into(),
            "collision" => edited["listen"]["socket"] = peers.github.to_string().into(),
            "ordinary" => edited["listen"]["socket"] = ordinary_socket.clone(),
            "outside" => edited["key"] = "/outside/llm.key".into(),
            "oauth" => edited["redirect_port"] = 0.into(),
            _ => unreachable!("fixture kind"),
        }
        store_json(&path, &edited);
        drop(first.take());
        let result = start(&source, None);
        if kind == "edited" {
            assert_eq!(
                result.expect("safe effective edit retained").peers.llm.to_string(),
                edited["listen"]["socket"].as_str().expect("edited socket")
            );
        } else {
            let error = result.err().expect("unsafe retained peer refused");
            assert!(error.contains(file), "{kind}: {error}");
        }
        assert_eq!(read_json(&path), edited, "retained edits never silently replaced");
        drop(reservation);
        write(&path, &original, true).expect("restore public peer document for the next case");
    }
}

#[test]
fn explicit_bundle_initializes_without_creating_absent_source_configuration() {
    let scratch = Scratch::new("absent-explicit");
    let source = scratch.dirs();
    let _prepared = prepare_empty(&scratch, &source);
    assert!(!crate::path(&source.config).exists());
}

#[test]
fn explicit_bundle_leaves_existing_empty_source_configuration_untouched() {
    let scratch = Scratch::new("empty-explicit");
    let source = scratch.dirs();
    make_private(crate::path(&source.config), skein_shell::effective_user()).expect("empty source configuration");
    let before = fs::symlink_metadata(crate::path(&source.config)).expect("source directory metadata");
    let _prepared = prepare_empty(&scratch, &source);
    let after = fs::symlink_metadata(crate::path(&source.config)).expect("unchanged source directory");
    assert!(fs::read_dir(crate::path(&source.config)).expect("source remains empty").next().is_none());
    assert_eq!(after.ino(), before.ino());
    assert_eq!(after.mode(), before.mode());
    assert_eq!(after.mtime(), before.mtime());
    assert_eq!(after.mtime_nsec(), before.mtime_nsec());
}

#[test]
fn explicit_bundle_overrides_an_unavailable_source_bundle_without_rewriting_source() {
    let scratch = Scratch::new("source-override");
    let source = provisioned_source(&scratch);
    let original = fs::read(source.document()).expect("source public bytes");
    fs::remove_dir_all(scratch.0.join("bundle-loopback")).expect("source bundle unavailable");
    let supplied = provision_bundle(&scratch.0.join("override"));
    let first = start(&source, Some(&supplied)).expect("explicit selection overrides source bundle");
    assert_eq!(
        read_json(&first.directories.document())["application"]["clients"]["bundle"],
        supplied.to_str().expect("supplied path")
    );
    let retained = first.bytes.clone();
    drop(first);
    let second = start(&source, None).expect("restart without repeating the selection");
    assert_eq!(second.bytes, retained);
    assert_eq!(fs::read(source.document()).expect("unchanged source bytes"), original);
}

#[test]
fn explicit_bundle_imports_init_configuration_without_a_source_bundle() {
    let scratch = Scratch::new("init-explicit");
    let source = initialized_source(&scratch);
    let original = fs::read(source.document()).expect("source document");
    let supplied = provision_bundle(&scratch.0.join("supplied"));
    let prepared = start(&source, Some(&supplied)).expect("init-shaped source plus explicit bundle");
    assert_eq!(
        read_json(&prepared.directories.document())["application"]["clients"]["bundle"],
        supplied.to_str().expect("supplied path")
    );
    assert_eq!(fs::read(source.document()).expect("source unchanged"), original);
}

#[test]
fn preparation_does_not_discover_optional_programs_outside_its_environment() {
    let scratch = Scratch::new("optional-programs");
    let source = scratch.dirs();
    let bundle = provision_bundle(&scratch.0.join("bundle"));
    let prepared = start(&source, Some(&bundle)).expect("prepare without optional executables");
    let document = read_json(&prepared.directories.document());
    assert!(document["application"]["workers"][0]["git"].is_null());
    assert!(document["engine"]["clients"]["provider"].is_null());
    assert!(document["application"]["machine"]["owned"]["sign_in"].is_null());
    assert!(!crate::path(&prepared.directories.data).join("provider.key").exists());
}

#[test]
fn fresh_preparation_is_private_and_isolates_imported_resources() {
    let scratch = Scratch::new("fresh");
    let source = provisioned_source(&scratch);
    let prepared = start(&source, None).expect("prepare effective documents");
    assert_ne!(prepared.directories.config, source.config);
    assert_ne!(prepared.directories.data, source.data);
    assert_eq!(prepared.directories.cache, source.cache);
    assert_eq!(prepared.directories.config, bytes(&scratch.0.join("source-data/box/fake-backends/config")));
    let tree = read_json(&prepared.directories.document());
    assert_eq!(
        tree["workers"][0]["agent"]["configuration"],
        crate::path(&prepared.directories.config).join("agent.json").to_str().expect("path")
    );
    assert_eq!(
        tree["application"]["clients"]["bundle"],
        scratch.0.join("bundle-loopback").to_str().expect("provisioned bundle path")
    );
    validate(&prepared.directories, &prepared.bytes, &scratch.0.join("fake-claude"))
        .expect("ordinary deployment and agent checks");
    let root = crate::path(&source.data).join("box/fake-backends");
    for name in ["", "config", "data"] {
        assert_eq!(fs::symlink_metadata(root.join(name)).expect("private directory").mode() & 0o777, 0o700, "{name}");
    }
    for name in [
        "prepared",
        "lock",
        "config/temper.json",
        "config/agent.json",
        "config/github.json",
        "config/oauth.json",
        "config/llm.json",
        "data/worker.secret",
        "data/worker.digest",
    ] {
        assert_eq!(fs::symlink_metadata(root.join(name)).expect("private file").mode() & 0o777, 0o600, "{name}");
    }
    assert!(start(&source, None).err().expect("live sandbox lock").contains("another fake-backend box"));
}

#[test]
fn empty_configuration_refuses_before_creating_data_or_cache() {
    let scratch = Scratch::new("empty");
    let source = scratch.dirs();
    // Missing provisioning refuses for both absent C and present empty C.
    assert_eq!(
        missing_bundle_refuses(&source),
        "application.clients.bundle: no provisioned bundle; run scripts/bundle.sh DIR && temper-box --fake-backends --bundle BUNDLE (directory flags as usual)"
    );
    make_private(crate::path(&source.config), skein_shell::effective_user()).expect("empty configuration");
    missing_bundle_refuses(&source);
}

fn missing_bundle_refuses(source: &Directories) -> String {
    let error = start(source, None).err().expect("no bundle");
    assert!(error.contains("scripts/bundle.sh DIR"), "{error}");
    assert!(error.contains("application.clients.bundle"), "{error}");
    assert!(error.contains("temper-box --fake-backends --bundle BUNDLE"), "{error}");
    assert!(!crate::path(&source.data).exists(), "refuse before data mutation");
    assert!(!crate::path(&source.cache).exists(), "refuse before cache mutation");
    error
}

#[test]
fn initialized_source_without_bundle_refuses_before_sandbox_mutation() {
    let scratch = Scratch::new("unprovisioned");
    let source = initialized_source(&scratch);
    // Configuration import must not need any resource under source D.
    fs::remove_dir_all(crate::path(&source.data)).expect("remove source fixture data");
    fs::remove_dir_all(crate::path(&source.cache)).expect("remove source fixture cache");
    missing_bundle_refuses(&source);
}

#[test]
fn partial_sandbox_refuses_with_explicit_reset_instructions() {
    let scratch = Scratch::new("partial");
    let source = provisioned_source(&scratch);
    let (root, _) = directories(&source).expect("sandbox paths");
    make_private(&root, skein_shell::effective_user()).expect("partial fixture");
    let error = start(&source, None).err().expect("partial refused");
    assert!(
        error.contains("partial preparation") && error.contains("remove this sandbox directory explicitly"),
        "{error}"
    );
}

#[test]
fn reuse_keeps_effective_edits_peer_addresses_and_data() {
    let scratch = Scratch::new("reuse");
    let source = provisioned_source(&scratch);
    let first = start(&source, None).expect("fresh sandbox");
    let peer = fs::read(crate::path(&first.directories.config).join("oauth.json")).expect("peer document");
    let data = crate::path(&first.directories.data).join("retained-fixture");
    write(&data, b"keep this data", false).expect("fixture data");
    let mut edited = read_json(&first.directories.document());
    edited["engine"]["deployment"]["seed"] = 42.into();
    let retained_bundle = provision_bundle(&scratch.0.join("retained-bundle"));
    edited["application"]["clients"]["bundle"] = retained_bundle.to_str().expect("retained bundle path").into();
    store_json(&first.directories.document(), &edited);
    let secret = fs::read(crate::path(&first.directories.data).join("worker.secret")).expect("generated test secret");
    drop(first);
    // Source documents stay unchanged, but the retained bundle no longer needs the source directory.
    fs::remove_dir_all(scratch.0.join("bundle-loopback")).expect("remove original provisioned bundle");
    let second = start(&source, None).expect("reuse the retained configuration's bundle");
    assert_eq!(read_json(&second.directories.document()), edited);
    assert_eq!(fs::read(crate::path(&second.directories.config).join("oauth.json")).expect("peer document"), peer);
    assert_eq!(fs::read(crate::path(&second.directories.data).join("worker.secret")).expect("test secret"), secret);
    assert_eq!(fs::read(data).expect("retained data"), b"keep this data");
    assert!(second.listeners.0.is_empty(), "reuse reopens its own listeners");
}

fn edit_retained(prepared: &Prepared) -> serde_json::Value {
    let mut edited = read_json(&prepared.directories.document());
    edited["engine"]["deployment"]["seed"] = 42.into();
    store_json(&prepared.directories.document(), &edited);
    edited
}

fn prepared_source(scratch: &Scratch) -> (Directories, Prepared) {
    let source = provisioned_source(scratch);
    let prepared = start(&source, None).expect("fresh sandbox");
    (source, prepared)
}

#[test]
fn reservations_avoid_every_port_the_peer_check_refuses() {
    let scratch = Scratch::new("ordinary-ports");
    let (_source, first) = prepared_source(&scratch);
    let document = read_json(&first.directories.document());
    let machine = &document["application"]["machine"];
    let ports = ordinary_ports(&first.directories, &first.bytes).expect("ports");
    for listen in [&machine["owned"]["listen"], &machine["fake"]["listen"]] {
        let socket = listen["socket"].as_str().expect("socket");
        let port = socket.parse::<SocketAddr>().expect("address").port();
        assert!(ports.contains(&port), "{socket}");
    }
}

#[test]
fn reuse_atomically_replaces_bundle_and_preserves_every_other_edit() {
    use std::io::Read;
    let scratch = Scratch::new("reuse-override");
    let (source, first) = prepared_source(&scratch);
    let mut edited = edit_retained(&first);
    let document = first.directories.document();
    let old_bytes = fs::read(&document).expect("retained document bytes");
    let old_inode = fs::metadata(&document).expect("retained inode").ino();
    let mut old_file = fs::File::open(&document).expect("hold old retained document open");
    let root = crate::path(&source.data).join("box/fake-backends");
    let resources = [
        "prepared",
        "config/agent.json",
        "config/github.json",
        "config/oauth.json",
        "config/llm.json",
        "data/worker.secret",
    ]
    .map(|name| (name, fs::read(root.join(name)).expect("retained fixture resource")));
    let data = root.join("data/retained-fixture");
    write(&data, b"keep this data", false).expect("retained data");
    drop(first);
    fs::remove_dir_all(scratch.0.join("bundle-loopback")).expect("old bundle unavailable");
    let replacement = provision_bundle(&scratch.0.join("replacement"));
    let second = start(&source, Some(&replacement)).expect("replace the retained selection");
    edited["application"]["clients"]["bundle"] = replacement.to_str().expect("replacement path").into();
    assert_eq!(read_json(&document), edited);
    assert_eq!(second.bytes.as_ref(), fs::read(&document).expect("published configuration"));
    let mut held_bytes = Vec::new();
    old_file.read_to_end(&mut held_bytes).expect("read old inode after publication");
    assert_eq!(held_bytes, old_bytes, "the old inode was never truncated");
    assert_ne!(fs::metadata(&document).expect("new inode").ino(), old_inode);
    assert_eq!(fs::metadata(&document).expect("private replacement").mode() & 0o777, 0o600);
    assert!(!document.with_file_name("temper.json.bundle-staging").exists());
    for (name, content) in resources {
        assert_eq!(fs::read(root.join(name)).expect("unchanged retained resource"), content, "{name}");
    }
    assert_eq!(fs::read(data).expect("retained data"), b"keep this data");
    let retained = second.bytes.clone();
    drop(second);
    let third = start(&source, None).expect("replacement remains selected without a flag");
    assert_eq!(third.bytes, retained);
}

#[test]
fn repeating_the_same_bundle_preserves_document_bytes_and_inode() {
    let scratch = Scratch::new("reuse-same-bundle");
    let (source, first) = prepared_source(&scratch);
    let document = first.directories.document();
    let retained = [b"\n ".as_slice(), first.bytes.as_ref(), b" \n"].concat();
    write(&document, &retained, true).expect("retain custom formatting");
    let inode = fs::metadata(&document).expect("retained inode").ino();
    drop(first);
    let same = scratch.0.join("bundle-loopback");
    let second = start(&source, Some(&same)).expect("same bundle is already selected");
    assert_eq!(second.bytes.as_ref(), retained);
    assert_eq!(fs::metadata(&document).expect("unchanged inode").ino(), inode);
}

fn refused_replacement_preserves_document(source: &Directories, document: &Path, replacement: &Path, member: &str) {
    let before = fs::read(document).expect("retained bytes before refusal");
    let inode = fs::metadata(document).expect("retained inode").ino();
    let error = start(source, Some(replacement)).err().expect("replacement refused");
    assert!(error.contains(member), "{error}");
    assert_eq!(fs::read(document).expect("retained bytes after refusal"), before);
    assert_eq!(fs::metadata(document).expect("unchanged inode").ino(), inode);
    assert!(!document.with_file_name("temper.json.bundle-staging").exists());
}

#[test]
fn invalid_replacement_bundle_preserves_retained_document() {
    let scratch = Scratch::new("reuse-invalid-bundle");
    let (source, first) = prepared_source(&scratch);
    let _edited = edit_retained(&first);
    let document = first.directories.document();
    drop(first);
    refused_replacement_preserves_document(&source, &document, &scratch.0.join("absent"), "index.html");
}

#[test]
fn invalid_retained_configuration_refuses_a_valid_replacement_without_mutation() {
    let scratch = Scratch::new("reuse-invalid-config");
    let (source, first) = prepared_source(&scratch);
    let document = first.directories.document();
    let mut edited = read_json(&document);
    edited["workers"][0]["agent"]["arguments"] = serde_json::json!(["internal", "agent", "/outside/agent.json"]);
    store_json(&document, &edited);
    drop(first);
    let replacement = provision_bundle(&scratch.0.join("valid-replacement"));
    refused_replacement_preserves_document(&source, &document, &replacement, "workers/0/agent/arguments");
}

#[test]
fn staging_file_collision_leaves_the_retained_document_untouched() {
    let scratch = Scratch::new("reuse-staging-collision");
    let (source, first) = prepared_source(&scratch);
    let document = first.directories.document();
    let before = fs::read(&document).expect("retained bytes");
    let staging = document.with_file_name("temper.json.bundle-staging");
    write(&staging, b"existing staging file", false).expect("staging collision fixture");
    drop(first);
    let replacement = provision_bundle(&scratch.0.join("replacement"));
    let error = start(&source, Some(&replacement)).err().expect("staging collision refused");
    assert!(error.contains("temper.json.bundle-staging"), "{error}");
    assert_eq!(fs::read(&document).expect("retained document unchanged"), before);
    assert_eq!(fs::read(staging).expect("staging file unchanged"), b"existing staging file");
}

#[test]
fn reuse_refuses_a_missing_effective_bundle_and_preserves_edits() {
    let scratch = Scratch::new("retained-missing-bundle");
    let source = provisioned_source(&scratch);
    let first = start(&source, None).expect("fresh sandbox");
    let document = first.directories.document();
    let mut edited = read_json(&document);
    let absent = scratch.0.join("absent-retained-bundle");
    edited["application"]["clients"]["bundle"] = absent.to_str().expect("absent bundle path").into();
    store_json(&document, &edited);
    drop(first);
    let error = start(&source, None).err().expect("missing retained bundle refused");
    assert!(error.contains("absent-retained-bundle"), "{error}");
    assert_eq!(read_json(&document), edited);

    edited["application"]["clients"]["bundle"] = serde_json::Value::Null;
    store_json(&document, &edited);
    let error = start(&source, None).err().expect("unnamed retained bundle refused");
    assert_eq!(error, "application.clients.bundle: effective document must name its bundle");
    assert_eq!(read_json(&document), edited);
}

#[test]
fn current_init_import_ignores_charter_files_and_rebases_paths() {
    let scratch = Scratch::new("import");
    let source = provisioned_source(&scratch);
    let mut temper = read_json(&source.document());
    // Equivalent absolute resource spelling is admitted without following source D's resources.
    temper["workers"][0]["secret"] = crate::path(&source.data).join("worker.secret").to_str().expect("path").into();
    store_json(&source.document(), &temper);
    fs::create_dir_all(crate::path(&source.config).join("charters")).expect("stray public directory");
    write(&crate::path(&source.config).join("charters/chat.txt"), b"A custom public charter.\n", false)
        .expect("public charter");
    let before = ["temper.json", "agent.json", "charters/chat.txt"].map(|name| {
        let path = crate::path(&source.config).join(name);
        (name, fs::read(&path).expect("source public fixture"), fs::metadata(path).expect("mode").mode())
    });
    let source_secret = fs::read(crate::path(&source.data).join("worker.secret")).expect("generated fixture secret");
    let prepared = start(&source, None).expect("current init-shaped import");
    assert_ne!(
        fs::read(crate::path(&prepared.directories.data).join("worker.secret")).expect("fresh test secret"),
        source_secret
    );
    assert!(!crate::path(&prepared.directories.config).join("charters").exists());
    for (name, contents, mode) in &before {
        let path = crate::path(&source.config).join(name);
        assert_eq!(fs::read(&path).expect("source unchanged"), *contents, "{name}");
        assert_eq!(fs::metadata(path).expect("source mode").mode(), *mode, "{name}");
    }
    assert_eq!(
        fs::read(crate::path(&source.data).join("worker.secret")).expect("unchanged source test secret"),
        source_secret
    );
    assert_eq!(fs::read(scratch.0.join("bundle-loopback/index.html")).expect("source page unchanged"), PAGE);
    assert_eq!(
        fs::read(scratch.0.join("bundle-loopback/0000000000000001/main.js")).expect("source script unchanged"),
        b"// fixture\n"
    );
    drop(prepared);
    write(&crate::path(&source.config).join("charters/chat.txt"), b"Changed source charter.\n", true)
        .expect("changed public source");
    let reused = start(&source, None).expect("stray charter edits do not change the imported defaults");
    assert!(!crate::path(&reused.directories.config).join("charters").exists());
}

#[test]
fn unsupported_imports_refuse_with_the_public_member_named() {
    let scratch = Scratch::new("unsupported");
    let source = provisioned_source(&scratch);
    let mut temper = read_json(&source.document());
    temper["workers"][0]["agent"]["arguments"] =
        serde_json::json!(["internal", "agent", "/elsewhere/agent.json", "extra"]);
    store_json(&source.document(), &temper);
    let error = start(&source, None).err().expect("custom argv refused");
    assert!(error.contains("workers/0/agent/arguments"), "{error}");
    assert!(!crate::path(&source.data).join("box/fake-backends").exists());
}

#[test]
fn effective_commands_reopen_the_retained_directories() {
    let scratch = Scratch::new("commands");
    let source = provisioned_source(&scratch);
    let prepared = start(&source, None).expect("prepare effective document");
    let (steps, _) = crate::box_command::configuration_steps(
        &prepared.directories,
        &prepared.bytes,
        backends::steps(&prepared.directories.config, &prepared.directories.data, &prepared.peers, None, false),
    )
    .expect("steps");
    let prefix = backends::steps(&prepared.directories.config, &prepared.directories.data, &prepared.peers, None, false);
    assert_eq!(&steps.always[..prefix.len()], prefix.as_ref());
    for step in steps.always[prefix.len()..]
        .iter()
        .chain(steps.asked.iter().flat_map(|group| group.steps.iter().map(|(_, step)| step)))
    {
        // All setup/serve commands receive the effective directory flags, including the retained cache.
        let args =
            step.arguments().iter().map(|arg| OsString::from(std::ffi::OsStr::from_bytes(arg))).collect::<Vec<_>>();
        for (flag, value) in [
            ("--config", &prepared.directories.config),
            ("--data", &prepared.directories.data),
            ("--cache", &prepared.directories.cache),
        ] {
            assert!(
                args.windows(2).any(|pair| pair[0] == flag && pair[1].as_os_str().as_bytes() == value.as_ref()),
                "{flag}: {args:?}"
            );
        }
    }
}

#[test]
fn missing_provisioned_directory_refuses_before_data_or_cache_mutation() {
    let scratch = Scratch::new("missing-bundle");
    let source = provisioned_source(&scratch);
    fs::remove_dir_all(scratch.0.join("bundle-loopback")).expect("remove provisioned fixture");
    fs::remove_dir_all(crate::path(&source.data)).expect("remove source fixture data");
    fs::remove_dir_all(crate::path(&source.cache)).expect("remove source fixture cache");
    missing_bundle_refuses(&source);
}

#[test]
fn oversized_explicit_bundle_refuses_before_creating_data_or_cache() {
    let scratch = Scratch::new("explicit-bundle-bytes");
    let source = scratch.dirs();
    let supplied = provision_bundle(&scratch.0.join("supplied"));
    let limits = temper_service::limits::profile(temper_service::limits::Profile::Small).clients;
    let page = OpenOptions::new().write(true).open(supplied.join("index.html")).expect("fixture page");
    page.set_len(u64::from(limits.bundle_bytes) + 1).expect("one byte past the profile's bundle bound");
    let error = start(&source, Some(&supplied)).err().expect("oversized explicit bundle refused");
    assert!(error.contains("clients.bundle_bytes"), "{error}");
    assert!(!crate::path(&source.config).exists());
    assert!(!crate::path(&source.data).exists());
    assert!(!crate::path(&source.cache).exists());
}

#[test]
fn oversized_source_bundle_refuses_before_sandbox_creation() {
    let scratch = Scratch::new("source-bundle-bytes");
    let source = provisioned_source(&scratch);
    let original = fs::read(source.document()).expect("source public bytes");
    let limits = temper_service::limits::profile(temper_service::limits::Profile::Small).clients;
    let page = OpenOptions::new().write(true).open(scratch.0.join("bundle-loopback/index.html")).expect("fixture page");
    page.set_len(u64::from(limits.bundle_bytes) + 1).expect("one byte past the admitted source bundle bound");
    let error = start(&source, None).err().expect("oversized source bundle refused");
    assert!(error.contains("clients.bundle_bytes"), "{error}");
    assert!(!crate::path(&source.data).join("box/fake-backends").exists());
    assert_eq!(fs::read(source.document()).expect("unchanged source public bytes"), original);
}

#[test]
fn unsupported_provisioned_file_refuses_before_sandbox_creation() {
    let scratch = Scratch::new("bundle-type");
    let source = provisioned_source(&scratch);
    write(&scratch.0.join("bundle-loopback/0000000000000001/unsupported.exe"), b"unsupported", false)
        .expect("invalid bundle fixture");
    let error = start(&source, None).err().expect("unsupported bundle file refused");
    assert!(error.contains("application.clients.bundle") && error.contains("unsupported.exe"), "{error}");
    assert!(!crate::path(&source.data).join("box/fake-backends").exists());
}

#[test]
fn linked_provisioned_page_refuses_before_reading_its_target() {
    let scratch = Scratch::new("bundle-link");
    let source = provisioned_source(&scratch);
    let page = scratch.0.join("bundle-loopback/index.html");
    fs::remove_file(&page).expect("remove fixture page");
    std::os::unix::fs::symlink(scratch.0.join("unread-target"), &page).expect("linked fixture page");
    let error = start(&source, None).err().expect("linked bundle page refused");
    assert!(error.contains("index.html") && error.contains("links"), "{error}");
    assert!(!crate::path(&source.data).join("box/fake-backends").exists());
}

#[test]
fn source_public_asset_links_are_refused_before_reading_them() {
    let scratch = Scratch::new("source-links");
    let source = provisioned_source(&scratch);
    let document = crate::path(&source.config).join("agent.json");
    fs::remove_file(&document).expect("remove public fixture");
    // The target is absent, so a refusal about links proves it was never opened or copied.
    std::os::unix::fs::symlink(scratch.0.join("unread-target"), &document).expect("linked public fixture");
    let error = start(&source, None).err().expect("source public link refused");
    assert!(error.contains("links"), "{error}");
    assert!(!crate::path(&source.data).join("box/fake-backends").exists());
}

#[test]
fn import_preserves_disabled_git_and_adds_the_fake_program() {
    let scratch = Scratch::new("disabled-programs");
    let source = provisioned_source(&scratch);
    let prepared = start(&source, None).expect("import optional choices independently of today's PATH");
    let temper = read_json(&prepared.directories.document());
    let agent = read_json(&crate::path(&prepared.directories.config).join("agent.json"));
    assert!(temper["application"]["workers"][0]["git"].is_null());
    assert_eq!(agent["programs"]["claude_code"]["path"], scratch.0.join("fake-claude").to_str().expect("path"));
}

#[test]
fn fake_github_machine_passes_ordinary_admission_without_document_forges() {
    let scratch = Scratch::new("four-forge-admission-r2");
    let (_source, first) = prepared_source(&scratch);
    crate::setup::check(&first.bytes, &jig_directories(&first.directories)).expect("ordinary admission");
    let machine = crate::box_command::machine(&first.bytes, &first.directories).expect("machine");
    assert_eq!(crate::address(&machine.github.expect("fake GitHub").api).expect("socket"), first.peers.github);
    validate(&first.directories, &first.bytes, &scratch.0.join("fake-claude")).expect("sandbox validation");
    let operations =
        crate::operations_in(&[], b"/", &first.directories, &first.bytes).expect("ordinary computed startup sizing");
    assert!(operations > 0);
    eprintln!("four-forge ordinary sizing: {operations} operations");
}

#[test]
fn missing_required_retained_documents_refuse_without_writes() {
    let scratch = Scratch::new("required");
    let (source, first) = prepared_source(&scratch);
    let config = crate::path(&first.directories.config).to_path_buf();
    let marker = fs::read(config.parent().expect("sandbox").join("prepared")).expect("marker");
    drop(first);
    for name in REQUIRED {
        let original = fs::read(config.join(name)).expect("original public fixture document");
        fs::remove_file(config.join(name)).expect("remove required fixture document");
        let before = fs::read_dir(&config).expect("config listing").count();
        let error = start(&source, None).err().expect("missing required document refused");
        assert!(error.contains(name) && error.contains("reset"), "{error}");
        assert_eq!(fs::read_dir(&config).expect("unchanged listing").count(), before);
        assert!(fs::read(config.parent().expect("sandbox").join("prepared")).expect("marker") == marker);
        assert!(!config.join(name).exists());
        write(&config.join(name), &original, false).expect("restore required public document for the next case");
    }
}

#[test]
fn foreign_retained_adapter_refuses_without_resetting_or_rewriting_it() {
    let scratch = Scratch::new("foreign-adapter");
    let (source, first) = prepared_source(&scratch);
    let path = crate::path(&first.directories.config).join("agent.json");
    let mut agent = read_json(&path);
    agent["programs"]["claude_code"]["path"] = "/foreign/claude".into();
    store_json(&path, &agent);
    // Moving temper makes both retained executable paths foreign; the adapter reset must win.
    let mut deployment = read_json(&first.directories.document());
    deployment["workers"][0]["agent"]["program"] = "/foreign/temper".into();
    store_json(&first.directories.document(), &deployment);
    let before = fs::read(&path).expect("fixture bytes");
    drop(first);
    let error = start(&source, None).err().expect("foreign adapter refused");
    assert!(error.contains("agent.json/programs/claude_code/path") && error.contains("reset"), "{error}");
    assert!(fs::read(path).expect("unchanged") == before);
}

#[test]
fn exact_sibling_and_provisioning_refusals_use_explicit_paths() {
    let expected = sibling(Path::new("/fixture/temper")).expect("absolute sibling");
    assert_eq!(expected, Path::new("/fixture/fake-claude"));
    adapter_identity(b"/fixture/fake-claude", &expected).expect("correct identity");
    assert!(adapter_identity(b"/foreign/claude", &expected).expect_err("foreign").contains("reset"));
    let scratch = Scratch::new("program-provisioning");
    let path = scratch.0.join("fake-claude");
    for exists in [false, true] {
        if exists {
            write(&path, b"not executable", false).expect("nonexecutable fixture");
        }
        let error = provisioned_program(&path).expect_err("provision outside startup");
        assert!(
            error.contains(path.to_str().expect("path"))
                && error.contains("cargo build -p smith-fake-claude --bin fake-claude"),
            "{error}"
        );
        assert!(!error.contains("reset"), "{error}");
    }
}

#[test]
fn source_adapter_is_a_bounded_fact_without_path_discovery() {
    let scratch = Scratch::new("source-program-fact");
    let source = provisioned_source(&scratch);
    let path = crate::path(&source.config).join("agent.json");
    let mut agent = read_json(&path);
    agent["programs"] = serde_json::json!({"claude_code":{"path":"/absent/source/claude","directory":crate::path(&source.data).join("programs")}});
    store_json(&path, &agent);
    let first = start(&source, None).expect("source adapter does not need to exist or match PATH");
    let effective = read_json(&crate::path(&first.directories.config).join("agent.json"));
    assert_ne!(effective["programs"]["claude_code"]["path"], agent["programs"]["claude_code"]["path"]);
}

#[test]
fn fake_backends_echo_takes_the_flag() {
    use temper_box_machine::host::Echo;
    let args =
        ["--fake-backends", "--config", "/c", "--data", "/d", "--cache", "/k"].map(|word| Box::from(word.as_bytes()));
    for (flags, expected) in
        [(vec![], Echo::None), (vec!["--echo", "steps"], Echo::Steps), (vec!["--echo", "none"], Echo::None)]
    {
        let mut arguments = args.to_vec();
        arguments.extend(flags.into_iter().map(|word| Box::from(word.as_bytes())));
        let crate::Command::BoxFakeBackends { echo, .. } =
            crate::configure_with(&arguments, &[], b"/").expect("valid fake-backend echo")
        else {
            panic!("fake box")
        };
        assert_eq!(echo, expected);
    }
    let mut arguments = args.to_vec();
    arguments.extend(["--echo", "bad"].map(|word| Box::from(word.as_bytes())));
    assert!(
        crate::configure_with(&arguments, &[], b"/")
            .expect_err("invalid echo")
            .contains("--echo: expected none or steps")
    );
}

#[test]
fn own_app_key_is_the_seeded_apps_private_pem() {
    let scratch = Scratch::new("own-app-key");
    let (source, prepared) = prepared_source(&scratch);
    let path = crate::path(&prepared.directories.data).join("github-app.pem");
    let key = fs::read(&path).expect("own-App key left for the person");
    let forge = temper_fake::forge::Document::decode(
        &fs::read(crate::path(&prepared.directories.config).join("github.json")).expect("forge document"),
    )
    .expect("generated forge");
    assert_eq!(key, forge.github_app_key().expect("seeded App").as_ref());
    assert!(key.starts_with(b"-----BEGIN RSA PRIVATE KEY-----"));
    assert_eq!(fs::metadata(&path).expect("key metadata").mode() & 0o777, 0o600);
    drop(prepared);
    let _again = start(&source, None).expect("reuse keeps the key");
    assert_eq!(fs::read(&path).expect("retained key"), key);
}

#[test]
fn changed_source_bytes_or_directory_require_explicit_reset_without_writes() {
    let scratch = Scratch::new("source-identity");
    let source = provisioned_source(&scratch);
    let prepared = start(&source, None).expect("initial preparation");
    let retained = prepared.directories.document();
    let before = fs::read(&retained).expect("effective public document");
    drop(prepared);
    for name in ["temper.json", "agent.json"] {
        let path = crate::path(&source.config).join(name);
        let original = fs::read(&path).expect("source public document");
        let changed = [original.as_slice(), b"\n"].concat();
        write(&path, &changed, true).expect("changed public bytes");
        let refusal = start(&source, None).err().expect("changed source refused");
        assert!(refusal.contains("changed public source documents"), "{refusal}");
        assert_eq!(fs::read(&retained).expect("retained document"), before);
        write(&path, &original, true).expect("restore source fixture");
    }
    let relocated = scratch.0.join("other-source");
    make_private(&relocated, skein_shell::effective_user()).expect("second public source");
    for name in ["temper.json", "agent.json"] {
        let content = fs::read(crate::path(&source.config).join(name)).expect("source public document");
        write(&relocated.join(name), &content, false).expect("same public bytes");
    }
    let other = Directories {
        config: bytes(&relocated),
        data: source.data.clone(),
        cache: source.cache.clone(),
        origins: source.origins,
    };
    let refusal = start(&other, None).err().expect("different source refused");
    assert!(refusal.contains("remove this sandbox directory explicitly to reset it"), "{refusal}");
    assert_eq!(fs::read(&retained).expect("retained document"), before);
}

fn named_ports() -> ([SocketAddr; 9], Chosen) {
    let addresses = temper_fixtures::ports::addresses::<9>(&[]);
    let value = addresses.map(|address| address.port().to_string()).join(",");
    (addresses, chosen_ports(OsStr::new(&value)).expect("nine chosen ports"))
}

fn prepared_ports(prepared: &Prepared) -> [u16; 9] {
    let document = read_json(&prepared.directories.document());
    let listeners = [
        "/application/clients/listener/socket",
        "/application/machine/owned/listen/socket",
        "/application/machine/fake/listen/socket",
        "/application/demo/llm/listen/socket",
        "/application/webhooks/listener/socket",
    ]
    .map(|pointer| {
        document
            .pointer(pointer)
            .expect("listener")
            .as_str()
            .expect("socket")
            .parse::<SocketAddr>()
            .expect("address")
            .port()
    });
    let Peers { github, oauth, llm, forgejo } = prepared.peers;
    [
        listeners[0],
        listeners[1],
        listeners[2],
        listeners[3],
        listeners[4],
        github.port(),
        oauth.port(),
        llm.port(),
        forgejo.port(),
    ]
}

#[test]
fn a_new_sandbox_takes_the_nine_ports_its_test_named_and_a_restart_keeps_them() {
    let scratch = Scratch::new("chosen-ports");
    let source = scratch.dirs();
    let bundle = provision_bundle(&scratch.0.join("bundle"));
    let (addresses, chosen) = named_ports();
    let prepared = start_chosen(&source, Some(&bundle), Some(&chosen)).expect("chosen fresh sandbox");
    assert_eq!(prepared_ports(&prepared), addresses.map(|address| address.port()));
    for address in addresses {
        assert!(TcpListener::bind(address).is_err(), "new sandbox holds {address}");
    }
    drop(prepared);
    let (_, others) = named_ports();
    let retained = start_chosen(&source, Some(&bundle), Some(&others)).expect("retained sandbox ignores new choices");
    assert_eq!(prepared_ports(&retained), addresses.map(|address| address.port()));
    drop(retained);
    let _again = addresses.map(|address| TcpListener::bind(address).expect("all nine listen again"));
}

#[test]
fn a_named_port_held_elsewhere_refuses_a_new_sandbox_naming_it() {
    for (name, slot) in [("chosen-listener-held", 0), ("chosen-peer-held", 5)] {
        let scratch = Scratch::new(name);
        let source = scratch.dirs();
        let bundle = provision_bundle(&scratch.0.join("bundle"));
        let (addresses, chosen) = named_ports();
        let held = TcpListener::bind(addresses[slot]).expect("hold chosen port");
        let why = start_chosen(&source, Some(&bundle), Some(&chosen)).err().expect("taken port refuses preparation");
        assert!(why.contains(CHOSEN_PORTS), "{why}");
        assert!(why.contains(&held.local_addr().expect("held address").port().to_string()), "{why}");
    }
}

#[test]
fn chosen_ports_refuses_all_but_nine_nonzero_ports() {
    use std::os::unix::ffi::OsStringExt;
    for value in [
        OsString::from("1,2,3,4,5,6,7,8"),
        OsString::from("1,2,3,4,5,6,7,8,9,10"),
        OsString::from("1,2,3,4,0,6,7,8,9"),
        OsString::from("1,2,3,4,word,6,7,8,9"),
        OsString::from_vec(b"1,2,3,4,\xff,6,7,8,9".to_vec()),
    ] {
        let why = chosen_ports(&value).err().expect("invalid choice refused");
        assert!(why.contains(CHOSEN_PORTS), "{why}");
    }
}
