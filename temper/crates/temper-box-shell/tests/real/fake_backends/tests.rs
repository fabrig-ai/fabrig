use std::{
    ffi::OsStr,
    fs,
    net::{SocketAddr, TcpListener},
    os::unix::fs::MetadataExt,
    path::Path,
};
use temper_box_shell::{
    directories::{Directories, make_private},
    testing::{
        self,
        fake_backend_fixtures::{
            Scratch, config_path, parsed, prepare_empty, prepared_source, provision_bundle, provisioned_source,
            read_json, start, start_chosen, start_options, store_file,
        },
        fake_backends::{
            ADAPTER, CHOSEN_PORTS, Chosen, Options, Prepared, bytes, chosen_ports, configured_claude, reserve_from,
            validate, write,
        },
    },
};
use temper_config_shell::FILE;
use temper_fake::backends::Peers;

fn edit_retained(prepared: &Prepared) -> serde_json::Value {
    let mut edited = read_json(&config_path(&prepared.directories));
    edited["engine"]["owners"][0]["subject"] = "retained-owner@localhost".into();
    let mut file = parsed(&config_path(&prepared.directories)).file;
    file.owner = Some("retained-owner@localhost".into());
    store_file(&config_path(&prepared.directories), &file);
    edited
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
    let model = testing::path(&fresh.directories.config).join("llm.json");
    let original = fs::read(&model).expect("generated model document");
    let joined = testing::path(&fresh.directories.config).join("llm-scripts.json");
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
        if step.name() != b"fake model" {
            assert!(
                !step.arguments().iter().any(|word| [b"--scripts".as_slice(), b"--strict"].contains(&word.as_ref()))
            );
            continue;
        }
        let mut expected = Vec::new();
        if let Some(path) = scripts {
            expected.extend([Box::from(b"--scripts".as_slice()), bytes(path)]);
        }
        if strict {
            expected.push(Box::from(b"--strict".as_slice()));
        }
        assert_eq!(step.arguments().get(6..).expect("base peer arguments"), expected);
    }
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
fn explicit_bundle_leaves_existing_empty_source_configuration_untouched() {
    let scratch = Scratch::new("empty-explicit");
    let source = scratch.dirs();
    make_private(testing::path(&source.config), skein_shell::effective_user()).expect("empty source configuration");
    let before = fs::symlink_metadata(testing::path(&source.config)).expect("source directory metadata");
    let _prepared = prepare_empty(&scratch, &source);
    let after = fs::symlink_metadata(testing::path(&source.config)).expect("unchanged source directory");
    assert!(fs::read_dir(testing::path(&source.config)).expect("source remains empty").next().is_none());
    assert_eq!(after.ino(), before.ino());
    assert_eq!(after.mode(), before.mode());
    assert_eq!(after.mtime(), before.mtime());
    assert_eq!(after.mtime_nsec(), before.mtime_nsec());
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
    let tree = read_json(&config_path(&prepared.directories));
    assert!(tree["workers"][0].get("agent").is_none());
    assert_eq!(
        tree["application"]["clients"]["bundle"],
        scratch.0.join("bundle-loopback").to_str().expect("provisioned bundle path")
    );
    validate(
        &prepared.directories,
        &fs::read(config_path(&prepared.directories)).unwrap(),
        &scratch.0.join("fake-claude"),
    )
    .expect("ordinary deployment and agent checks");
    let root = testing::path(&source.data).join("box/fake-backends");
    for name in ["", "config", "data"] {
        assert_eq!(fs::symlink_metadata(root.join(name)).expect("private directory").mode() & 0o777, 0o700, "{name}");
    }
    for name in [
        "prepared",
        "lock",
        &format!("config/{FILE}"),
        "config/github.json",
        "config/oauth.json",
        "config/llm.json",
        "data/worker.secret",
    ] {
        assert_eq!(fs::symlink_metadata(root.join(name)).expect("private file").mode() & 0o777, 0o600, "{name}");
    }
    assert!(start(&source, None).err().expect("live sandbox lock").contains("another fake-backend box"));
}

#[test]
fn reuse_atomically_replaces_bundle_and_preserves_every_other_edit() {
    use std::io::Read;
    let scratch = Scratch::new("reuse-override");
    let (source, first) = prepared_source(&scratch);
    let mut edited = edit_retained(&first);
    let document = config_path(&first.directories);
    let old_bytes = fs::read(&document).expect("retained document bytes");
    let old_inode = fs::metadata(&document).expect("retained inode").ino();
    let mut old_file = fs::File::open(&document).expect("hold old retained document open");
    let root = testing::path(&source.data).join("box/fake-backends");
    let resources = ["prepared", "config/github.json", "config/oauth.json", "config/llm.json", "data/worker.secret"]
        .map(|name| (name, fs::read(root.join(name)).expect("retained fixture resource")));
    let data = root.join("data/retained-fixture");
    write(&data, b"keep this data", false).expect("retained data");
    drop(first);
    fs::remove_dir_all(scratch.0.join("bundle-loopback")).expect("old bundle unavailable");
    let replacement = provision_bundle(&scratch.0.join("replacement"));
    let second = start(&source, Some(&replacement)).expect("replace the retained selection");
    edited["application"]["clients"]["bundle"] = replacement.to_str().expect("replacement path").into();
    assert_eq!(read_json(&document), edited);
    assert_eq!(second.bytes, parsed(&document).document);
    let mut held_bytes = Vec::new();
    old_file.read_to_end(&mut held_bytes).expect("read old inode after publication");
    assert_eq!(held_bytes, old_bytes, "the old inode was never truncated");
    assert_ne!(fs::metadata(&document).expect("new inode").ino(), old_inode);
    assert_eq!(fs::metadata(&document).expect("private replacement").mode() & 0o777, 0o600);
    assert!(!document.with_file_name(format!("{FILE}.bundle-staging")).exists());
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
    let document = config_path(&first.directories);
    let retained = [b"\n ".as_slice(), fs::read(&document).unwrap().as_slice(), b" \n"].concat();
    write(&document, &retained, true).expect("retain custom formatting");
    let inode = fs::metadata(&document).expect("retained inode").ino();
    drop(first);
    let same = scratch.0.join("bundle-loopback");
    let second = start(&source, Some(&same)).expect("same bundle is already selected");
    assert_eq!(fs::read(&document).unwrap(), retained);
    assert_eq!(second.bytes, parsed(&document).document);
    assert_eq!(fs::metadata(&document).expect("unchanged inode").ino(), inode);
}

#[test]
fn invalid_replacement_bundle_preserves_retained_document() {
    let scratch = Scratch::new("reuse-invalid-bundle");
    let (source, first) = prepared_source(&scratch);
    let _edited = edit_retained(&first);
    let document = config_path(&first.directories);
    drop(first);
    refused_replacement_preserves_document(&source, &document, &scratch.0.join("absent"), "index.html");
}

#[test]
fn invalid_retained_configuration_refuses_a_valid_replacement_without_mutation() {
    let scratch = Scratch::new("reuse-invalid-config");
    let (source, first) = prepared_source(&scratch);
    let document = config_path(&first.directories);
    let invalid = [fs::read(&document).unwrap(), b"\n[application]\nworkers = []\n".to_vec()].concat();
    write(&document, &invalid, true).unwrap();
    drop(first);
    let replacement = provision_bundle(&scratch.0.join("valid-replacement"));
    refused_replacement_preserves_document(&source, &document, &replacement, "application");
}

#[test]
fn fake_adapter_is_retained_outside_fixed_worker_configuration() {
    let scratch = Scratch::new("source-program-fact");
    let source = provisioned_source(&scratch);
    let first = start(&source, None).expect("prepare fake box");
    let effective = read_json(&config_path(&first.directories));
    assert!(effective["application"].get("workers").is_none());
    let record = testing::path(&first.directories.config).join(ADAPTER);
    assert_eq!(fs::read(&record).expect("pinned adapter"), bytes(&scratch.0.join("fake-claude")).as_ref());
    assert_eq!(fs::metadata(&record).expect("adapter record").mode() & 0o777, 0o600);
    fs::remove_file(&record).expect("remove pinned adapter");
    assert!(configured_claude(&first.directories).expect_err("never fall back to PATH").contains(ADAPTER));
}

#[test]
fn own_app_key_is_the_seeded_apps_private_pem() {
    let scratch = Scratch::new("own-app-key");
    let (source, prepared) = prepared_source(&scratch);
    let path = testing::path(&prepared.directories.data).join("github-app.pem");
    let key = fs::read(&path).expect("own-App key left for the person");
    let forge = temper_fake::forge::Document::decode(
        &fs::read(testing::path(&prepared.directories.config).join("github.json")).expect("forge document"),
    )
    .expect("generated forge");
    assert_eq!(key, forge.github_app_key().expect("seeded App").as_ref());
    assert!(key.starts_with(b"-----BEGIN RSA PRIVATE KEY-----"));
    assert_eq!(fs::metadata(&path).expect("key metadata").mode() & 0o777, 0o600);
    drop(prepared);
    let _again = start(&source, None).expect("reuse keeps the key");
    assert_eq!(fs::read(&path).expect("retained key"), key);
}

fn named_ports() -> ([SocketAddr; 9], Chosen) {
    let addresses = temper_fixtures::ports::addresses::<9>(&[]);
    let value = addresses.map(|address| address.port().to_string()).join(",");
    (addresses, chosen_ports(OsStr::new(&value)).expect("nine chosen ports"))
}

fn prepared_ports(prepared: &Prepared) -> [u16; 6] {
    let document = read_json(&config_path(&prepared.directories));
    let listeners = ["/application/clients/listener/socket", "/application/webhooks/listener/socket"].map(|pointer| {
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
    [listeners[0], listeners[1], github.port(), oauth.port(), llm.port(), forgejo.port()]
}

#[test]
fn a_new_sandbox_takes_the_nine_ports_its_test_named_and_a_restart_keeps_them() {
    let scratch = Scratch::new("chosen-ports");
    let source = scratch.dirs();
    let bundle = provision_bundle(&scratch.0.join("bundle"));
    let (addresses, chosen) = named_ports();
    let prepared = start_chosen(&source, Some(&bundle), Some(&chosen)).expect("chosen fresh sandbox");
    assert_eq!(prepared_ports(&prepared), [0, 4, 5, 6, 7, 8].map(|at| addresses[at].port()));
    for address in [0, 4, 5, 6, 7, 8].map(|at| addresses[at]) {
        assert!(TcpListener::bind(address).is_err(), "new sandbox holds {address}");
    }
    drop(prepared);
    let (_, others) = named_ports();
    let retained = start_chosen(&source, Some(&bundle), Some(&others)).expect("retained sandbox ignores new choices");
    assert_eq!(prepared_ports(&retained), [0, 4, 5, 6, 7, 8].map(|at| addresses[at].port()));
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

fn refused_replacement_preserves_document(source: &Directories, document: &Path, replacement: &Path, member: &str) {
    let before = fs::read(document).expect("retained bytes before refusal");
    let inode = fs::metadata(document).expect("retained inode").ino();
    let error = start(source, Some(replacement)).err().expect("replacement refused");
    assert!(error.contains(member), "{error}");
    assert_eq!(fs::read(document).expect("retained bytes after refusal"), before);
    assert_eq!(fs::metadata(document).expect("unchanged inode").ino(), inode);
    assert!(!document.with_file_name(format!("{FILE}.bundle-staging")).exists());
}
