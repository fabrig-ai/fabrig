use std::{
    ffi::OsStr,
    fs,
    net::{SocketAddr, TcpListener},
    os::unix::fs::MetadataExt,
};
use temper_box_shell::testing::{
    self,
    fake_backend_fixtures::{
        Scratch, config_path, prepare_empty, prepared_source, provision_bundle, provisioned_source, start, start_chosen,
    },
    fake_backends::{CHOSEN_PORTS, Chosen, chosen_ports},
};
use temper_config_shell::FILE;

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
fn fresh_preparation_is_private_and_isolates_imported_resources() {
    let scratch = Scratch::new("fresh");
    let source = provisioned_source(&scratch);
    let _prepared = start(&source, None).expect("prepare effective documents");
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
    let document = config_path(&first.directories);
    let old_bytes = fs::read(&document).expect("retained document bytes");
    let old_inode = fs::metadata(&document).expect("retained inode").ino();
    let mut old_file = fs::File::open(&document).expect("hold old retained document open");
    drop(first);
    fs::remove_dir_all(scratch.0.join("bundle-loopback")).expect("old bundle unavailable");
    let replacement = provision_bundle(&scratch.0.join("replacement"));
    let _second = start(&source, Some(&replacement)).expect("replace the retained selection");
    let mut held_bytes = Vec::new();
    old_file.read_to_end(&mut held_bytes).expect("read old inode after publication");
    assert_eq!(held_bytes, old_bytes, "the old inode was never truncated");
    assert_ne!(fs::metadata(&document).expect("new inode").ino(), old_inode);
    assert_eq!(fs::metadata(&document).expect("private replacement").mode() & 0o777, 0o600);
}

fn named_ports() -> ([SocketAddr; 9], Chosen) {
    let addresses = temper_fixtures::ports::addresses::<9>(&[]);
    let value = addresses.map(|address| address.port().to_string()).join(",");
    (addresses, chosen_ports(OsStr::new(&value)).expect("nine chosen ports"))
}

#[test]
fn a_new_sandbox_takes_the_nine_ports_its_test_named_and_a_restart_keeps_them() {
    let scratch = Scratch::new("chosen-ports");
    let source = scratch.dirs();
    let bundle = provision_bundle(&scratch.0.join("bundle"));
    let (addresses, chosen) = named_ports();
    let prepared = start_chosen(&source, Some(&bundle), Some(&chosen)).expect("chosen fresh sandbox");
    for address in [0, 4, 5, 6, 7, 8].map(|at| addresses[at]) {
        assert!(TcpListener::bind(address).is_err(), "new sandbox holds {address}");
    }
    drop(prepared);
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
