//! The box's held listeners and slot boundaries (temper/docs/design/protocol/setup/box.md, 1).
use crate::directories::jig_directories;
use crate::forge::remove_directory;
use std::fs;

fn first_link_named() -> temper_service::startup::Named {
    let directory = std::path::Path::new("/fixture");
    let directories = super::configuration::directories(directory);
    let mut document = super::configuration::check_document(directory);
    document["engine"]["clients"]["link"] = true.into();
    document["application"]["clients"]["listener"] = serde_json::json!({"socket":"127.0.0.1:8367"});
    document["application"]["clients"]["public"] = "https://temper.example.invalid:8443".into();
    temper_service::startup::named(
        &serde_json::to_vec(&document).expect("fixture document"),
        &jig_directories(&directories),
    )
    .expect("checked first-link settings")
}

fn first_link_steps() -> temper_forge_setup::Steps {
    temper_forge_setup::Steps { always: Box::new([]), asked: Box::new([]) }
}

#[test]
fn first_link_draws_a_secret_and_uses_serves_url() {
    let mut named = first_link_named();
    for public in [named.public.take(), None] {
        named.public = public;
        let link = crate::box_command::first_link(&named, &first_link_steps()).expect("draw link").expect("link");
        assert_eq!(link.secret.expose().len(), temper_service::clients::LINK_SECRET_BYTES);
        let address = crate::address(&named.clients).expect("clients address");
        assert!(
            link.url == temper_service::clients::first_link_url(address, named.public.as_ref(), &link.secret),
            "the box uses serve's URL"
        );
        let next = crate::box_command::first_link(&named, &first_link_steps()).expect("draw again").expect("link");
        assert!(link.secret.expose() != next.secret.expose(), "each start draws a fresh secret");
    }
}

#[test]
fn the_front_runs_exactly_when_requested_and_requires_a_first_link() {
    let named = first_link_named();
    for wanted in [false, true] {
        let mut first = crate::box_command::first_link(&named, &first_link_steps()).expect("draw link");
        let front = crate::box_command::configured_front(wanted, &named, first.as_mut()).expect("configure front");
        assert_eq!(front.is_some(), wanted, "only --front enables the pipes, with no terminal condition");
        let absent = crate::box_command::configured_front(wanted, &named, None);
        if wanted {
            assert_eq!(absent.expect_err("no first link"), "--front: no first sign-in link drawn");
        } else {
            assert!(absent.expect("bare box").is_none());
        }
    }
}

#[test]
fn the_front_hands_a_local_link_at_the_clients_listener() {
    let mut named = first_link_named();
    let public = named.public.take();
    for (listener, origin) in [("127.0.0.1:8367", None), ("[::1]:8367", None), ("127.0.0.1:8367", public)] {
        let listener: std::net::SocketAddr = listener.parse().expect("fixture listener");
        named.clients = jig_protocol_config::Address::Addr(skein_io::Addr::Ip(listener));
        named.public = origin;
        let mut first = crate::box_command::first_link(&named, &first_link_steps()).expect("draw link").expect("link");
        let front = crate::box_command::front(&named, &mut first).expect("local front");
        let local = first.local.as_ref().expect("local secret");
        assert!(local.expose() != first.secret.expose(), "local and printed secrets differ");
        assert!(
            front.link.expose() == temper_service::clients::first_link_url(listener, None, local).as_ref(),
            "the front's link uses the listener without a public origin"
        );
        let address = format!("http://{listener}");
        assert_eq!((front.input.raw(), front.link_output.raw(), front.lines.raw()), (0, 3, 4));
        assert!(
            front.link.expose().starts_with(format!("{address}/").as_bytes()),
            "the link's host is the front's address"
        );
        assert!(
            first.url == temper_service::clients::first_link_url(listener, named.public.as_ref(), &first.secret),
            "the printed link keeps its public origin"
        );
    }
}

#[test]
fn first_link_draws_none_when_disabled() {
    let mut named = first_link_named();
    named.link = false;
    assert!(crate::box_command::first_link(&named, &first_link_steps()).expect("disabled link").is_none());
}

#[test]
fn first_link_draws_none_at_port_zero() {
    let mut named = first_link_named();
    named.clients = jig_protocol_config::Address::Addr(skein_io::Addr::Ip(([127, 0, 0, 1], 0).into()));
    assert!(crate::box_command::first_link(&named, &first_link_steps()).expect("port zero").is_none());
}

#[test]
fn first_link_draws_none_with_an_always_link_step() {
    let mut steps = first_link_steps();
    steps.always = Box::new([temper_forge_setup::Step::new(
        b"handoff".as_slice().into(),
        Box::new([]),
        temper_forge_setup::Kind::Link,
    )]);
    assert!(crate::box_command::first_link(&first_link_named(), &steps).expect("link step").is_none());
}

#[test]
fn first_link_draws_none_with_an_asked_link_step() {
    use temper_forge_setup::{Group, Kind, Stage, Step};
    let mut steps = first_link_steps();
    steps.asked = Box::new([Group {
        kind: None,
        remover: None,
        forge: 1,
        label: b"forge".as_slice().into(),
        recorded: false,
        before: 0,
        steps: Box::new([(Stage::Run, Step::new(b"handoff".as_slice().into(), Box::new([]), Kind::Link))]),
    }]);
    assert!(crate::box_command::first_link(&first_link_named(), &steps).expect("asked link step").is_none());
}

#[test]
fn first_link_accepts_4096_bytes_and_refuses_one_more() {
    let mut named = first_link_named();
    let steps = first_link_steps();
    let link = crate::box_command::first_link(&named, &steps).expect("draw link").expect("link");
    let public = named.public.as_mut().expect("public fixture");
    // Extend the checked origin to exercise the URL bound independently of the origin parser's bound.
    let authority_bytes = public.name.len() + 4096 - link.url.len();
    public.name = vec![b'a'; authority_bytes].into_boxed_slice();
    let link = crate::box_command::first_link(&named, &steps).expect("at bound").expect("link");
    assert_eq!(link.url.len(), 4096);
    let public = named.public.as_mut().expect("public fixture");
    public.name = vec![b'a'; authority_bytes + 1].into_boxed_slice();
    assert_eq!(
        crate::box_command::first_link(&named, &steps).expect_err("past URL bound"),
        "serve: sign-in link too long"
    );
}

#[test]
fn a_held_descriptor_is_refused_as_held() {
    use std::os::fd::AsRawFd;
    let file = fs::File::open("/dev/null").expect("open a non-socket");
    let fd = skein_io::kernel::Fd::new(file.as_raw_fd());
    let held = vec![(fd, std::num::NonZeroU16::new(4).expect("nonzero port"))];
    assert_eq!(
        crate::box_command::held_listeners(held),
        Err(format!("held listener: fd {} is not a socket", fd.raw()))
    );
}

#[test]
fn a_refusal_names_whose_descriptor_it_was() {
    use skein_io::kernel::Fd;
    use skein_shell::{ListenerCheck, ListenersRefusal as Why};
    for (why, held, inherited) in [
        (
            Why::Check { fd: Fd::new(4), check: ListenerCheck::Listening, errno: 42 },
            "held listener: fd 4: getsockopt SO_ACCEPTCONN: errno 42",
            "LISTEN_FDS: fd 4: getsockopt SO_ACCEPTCONN: errno 42",
        ),
        (
            Why::Kernel { fd: Fd::new(4), errno: 42 },
            "held listener: fd 4: close-on-exec: errno 42",
            "LISTEN_FDS: fd 4: close-on-exec: errno 42",
        ),
    ] {
        assert_eq!(crate::held_refusal(why), held);
        assert_eq!(crate::listeners_refusal(why), inherited);
    }
}

#[test]
fn forge_remove_refuses_a_path_outside_data() {
    let scratch = skein_scratch::Scratch::new("forge-remove-boundary");
    let data = scratch.path().join("data");
    let cache = scratch.path().join("cache");
    fs::create_dir(&data).expect("data");
    fs::create_dir(&cache).expect("cache");
    fs::write(cache.join("kept"), b"binary").expect("cache binary");
    assert!(remove_directory(&data, &cache, &cache).is_err());
    assert!(remove_directory(&data, &scratch.path().join("absent"), &cache).is_err());
    let link = data.join("forge-7");
    std::os::unix::fs::symlink(&cache, &link).expect("external symlink");
    assert!(remove_directory(&data, &link, &cache).is_err());
    assert!(remove_directory(&data, &data, &cache).is_err());
    assert_eq!(fs::read(cache.join("kept")).expect("kept binary"), b"binary");
}

#[test]
fn forge_remove_of_an_absent_directory_succeeds() {
    let scratch = skein_scratch::Scratch::new("forge-remove-absent");
    let directory = scratch.path().join("forge-7");
    remove_directory(scratch.path(), &directory, &scratch.path().join("cache")).expect("absent succeeds");
    fs::create_dir(&directory).expect("forge directory");
    fs::write(directory.join("state"), b"state").expect("forge state");
    remove_directory(scratch.path(), &directory, &scratch.path().join("cache")).expect("remove forge");
    remove_directory(scratch.path(), &directory, &scratch.path().join("cache")).expect("repeat succeeds");
    assert!(!directory.exists());
}

#[test]
fn forge_commands_compose_the_recorded_kind_with_the_machine() {
    use temper_forge_setup::setup::Mode;
    let scratch = skein_scratch::Scratch::new("forge-kind-entry");
    let directories = super::configuration::directories(scratch.path());
    let bytes = forge_machine();
    let machine = crate::box_command::machine(&bytes, &directories).expect("machine");
    let groups = scratch.path().join("box/groups");
    fs::create_dir_all(&groups).expect("groups");
    for (kind, directory) in [("owned", "forge-17"), ("fake", "playground-17")] {
        fs::write(groups.join("17"), format!("{kind}\n")).expect("group kind");
        let entry = crate::forge::entry(&directories, &machine, 17).expect("composed entry");
        assert_eq!(entry.forge, 17);
        assert_eq!(entry.directory.as_ref(), scratch.path().join(directory).as_os_str().as_encoded_bytes());
        match entry.mode {
            Mode::Owned(owned) => {
                assert_eq!(kind, "owned");
                assert_eq!(owned.listen.port(), 8372);
                assert!(owned.fetch.forgejo);
            }
            Mode::Fake(fake) => {
                assert_eq!(kind, "fake");
                assert_eq!(fake.listen.port(), 8369);
            }
            Mode::Managed { .. } => panic!("a kind slot is owned or fake"),
        }
    }
    fs::write(groups.join("17"), b"unknown\n").expect("invalid group");
    assert!(crate::forge::entry(&directories, &machine, 17).is_err());
}

fn forge_machine() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"application":{"machine":{"owned":{
        "listen":{"socket":"127.0.0.1:8372"},"remote":"http://127.0.0.1:8372",
        "person":{"login":"ada","email":"ada@example.invalid"},"forgejo":"fetched","runner":null
    }, "fake":{"listen":{"socket":"127.0.0.1:8369"},"remote":"http://127.0.0.1:8369","git":"demo/bin/git"}}}}))
    .expect("machine JSON")
}

#[test]
fn a_checked_machine_keeps_resolved_programs_for_both_group_kinds() {
    use temper_forge_setup::setup::Mode;
    let scratch = skein_scratch::Scratch::new("checked-machine-entry");
    let directories = super::configuration::directories(scratch.path());
    let mut document: serde_json::Value =
        serde_json::from_slice(include_bytes!("../../../../docs/examples/owned.json")).expect("owned JSON");
    document["application"]["machine"]["owned"]["forgejo"] = "tools/forgejo".into();
    document["application"]["machine"]["owned"]["runner"]["binary"] = "tools/runner".into();
    document["application"]["machine"]["fake"] = serde_json::json!({
        "listen":{"socket":"127.0.0.1:8369"}, "remote":"http://127.0.0.1:8369", "git":"demo/bin/git"
    });
    let bytes = serde_json::to_vec(&document).expect("document JSON");
    let checked = crate::setup::check(&bytes, &jig_directories(&directories)).expect("checked deployment");
    assert_eq!(
        checked.named.machine,
        crate::box_command::machine(&bytes, &directories).expect("separate machine read")
    );
    let groups = scratch.path().join("box/groups");
    fs::create_dir_all(&groups).expect("groups");
    for kind in ["owned", "fake"] {
        fs::write(groups.join("17"), format!("{kind}\n")).expect("group kind");
        let entry = crate::forge::entry(&directories, &checked.named.machine, 17).expect("checked entry");
        match entry.mode {
            Mode::Owned(owned) => {
                assert_eq!(kind, "owned");
                assert_eq!(owned.forgejo.as_ref(), scratch.path().join("tools/forgejo").as_os_str().as_encoded_bytes());
                assert_eq!(
                    owned.runner.expect("runner").binary.as_ref(),
                    scratch.path().join("tools/runner").as_os_str().as_encoded_bytes()
                );
            }
            Mode::Fake(fake) => {
                assert_eq!(kind, "fake");
                assert_eq!(fake.git.as_ref(), scratch.path().join("demo/bin/git").as_os_str().as_encoded_bytes());
            }
            Mode::Managed { .. } => panic!("a kind slot is owned or fake"),
        }
    }
}

#[test]
fn a_forge_with_no_group_file_is_the_machines_owned() {
    use temper_forge_setup::setup::Mode;
    let scratch = skein_scratch::Scratch::new("forge-no-group-entry");
    let directories = super::configuration::directories(scratch.path());
    let machine = crate::box_command::machine(&forge_machine(), &directories).expect("machine");
    let entry = crate::forge::entry(&directories, &machine, 17).expect("machine's owned entry");
    assert_eq!(entry.forge, 17);
    assert_eq!(entry.directory.as_ref(), scratch.path().join("forge-17").as_os_str().as_encoded_bytes());
    let Mode::Owned(owned) = entry.mode else { panic!("the machine's owned part") };
    assert_eq!(owned.listen.port(), 8372);
    assert!(owned.fetch.forgejo);
    assert!(!scratch.path().join("box").exists());
}

#[test]
fn no_group_file_and_no_owned_part_is_refused_naming_both() {
    let scratch = skein_scratch::Scratch::new("forge-no-owned-entry");
    let directories = super::configuration::directories(scratch.path());
    let document = super::configuration::check_document(scratch.path());
    fs::write(scratch.path().join("temper.json"), serde_json::to_vec(&document).expect("document"))
        .expect("configuration");
    for (verb, arguments) in [
        ("fetch", vec!["fetch".into(), "17".into()]),
        ("prepare", vec!["prepare".into(), "17".into()]),
        ("run", vec!["run".into(), "17".into(), "forgejo".into()]),
    ] {
        let refused = crate::forge::start(&directories, &arguments, None, &[]).expect_err("no owned part");
        assert_eq!(
            refused.line(),
            format!(
                "temper forge {verb}: forge 17: {}: absent, and application.machine names no owned Forgejo\n",
                scratch.path().join("box/groups/17").display()
            )
        );
    }
    assert!(!scratch.path().join("box").exists());
}

#[test]
fn remove_refuses_a_forge_with_no_group_file() {
    let scratch = skein_scratch::Scratch::new("forge-no-group-remove");
    let mut directories = super::configuration::directories(scratch.path());
    directories.cache = scratch.path().join("cache").as_os_str().as_encoded_bytes().into();
    let forge = scratch.path().join("forge-17");
    fs::create_dir(&forge).expect("forge directory");
    fs::write(forge.join("kept"), b"kept").expect("forge file");
    let refused = crate::forge::start(&directories, &["remove".into(), "17".into()], None, &[]).expect_err("no kind");
    assert_eq!(
        refused.line(),
        format!("temper forge remove: forge 17: {}: absent\n", scratch.path().join("box/groups/17").display())
    );
    assert_eq!(fs::read(forge.join("kept")).expect("forge kept"), b"kept");
    assert!(!scratch.path().join("box").exists());
}

#[test]
fn unreadable_group_files_keep_their_refusals() {
    let scratch = skein_scratch::Scratch::new("forge-unreadable-group");
    let directories = super::configuration::directories(scratch.path());
    let groups = scratch.path().join("box/groups");
    fs::create_dir_all(groups.join("17")).expect("group is a directory");
    std::os::unix::fs::symlink(groups.join("absent"), groups.join("18")).expect("dangling group link");
    let bytes = forge_machine();
    let machine = crate::box_command::machine(&bytes, &directories).expect("machine");
    for (number, reason) in [(17, "not a file"), (18, "absent")] {
        assert_eq!(
            crate::forge::entry(&directories, &machine, number),
            Err(format!("{}: {reason}", groups.join(number.to_string()).display()))
        );
    }
}

#[test]
fn sign_in_keeps_serve_before_the_restored_group_with_first_steps() {
    use temper_forge_setup::{Kind, Step, machine::BoxKind};
    let dirs = super::configuration::directories(std::path::Path::new("/fixture"));
    let document = super::configuration::init_document();
    for first in [Vec::new(), vec![Step::new(b"first".as_slice().into(), Box::new([]), Kind::Once)]] {
        let (steps, _) =
            crate::box_command::configuration_steps(&dirs, &document, first.into_boxed_slice()).expect("box steps");
        let owned = steps.asked.iter().find(|group| group.kind == Some(BoxKind::Owned)).expect("owned slot");
        assert_eq!(
            usize::try_from(owned.before).expect("step index"),
            steps.always.len(),
            "serve precedes discovery on restart"
        );
    }
}

#[test]
fn a_box_with_no_sign_in_has_no_source_step_in_its_group() {
    use temper_forge_setup::{Stage, machine::BoxKind};
    let dirs = super::configuration::directories(std::path::Path::new("/fixture"));
    let mut document: serde_json::Value =
        serde_json::from_slice(&super::configuration::init_document()).expect("init JSON");
    document["engine"]["clients"]["provider"] = serde_json::Value::Null;
    document["application"]["machine"]["owned"]["sign_in"] = serde_json::Value::Null;
    let document = serde_json::to_vec(&document).expect("document JSON");
    let (steps, _) = crate::box_command::configuration_steps(&dirs, &document, Box::new([])).expect("box steps");
    let owned = steps.asked.iter().find(|group| group.kind == Some(BoxKind::Owned)).expect("owned slot");
    assert!(!owned.steps.iter().any(|(stage, _)| *stage == Stage::Source), "no source needs the held engine");
    assert_eq!(owned.before, 0, "the group can start before serve");
}

#[test]
fn recording_is_given_only_to_the_serve_step_after_box() {
    let dirs = super::configuration::directories(std::path::Path::new("/fixture"));
    let document = super::configuration::init_document();
    for record in [false, true] {
        let (steps, _) = crate::box_command::configuration_steps_recording(&dirs, &document, Box::new([]), record)
            .expect("box steps");
        let serve = steps.always.iter().find(|step| step.name() == b"serve").expect("serve step");
        let arguments: Vec<_> = serve.arguments().iter().map(std::convert::AsRef::as_ref).collect();
        assert_eq!(arguments.contains(&b"--record".as_slice()), record);
        if record {
            assert!(arguments.ends_with(&[b"--box".as_slice(), b"--record"]));
        }
        for group in &steps.asked {
            for (_, step) in &group.steps {
                assert!(!step.arguments().iter().any(|word| word.as_ref() == b"--record"));
            }
        }
    }
}
