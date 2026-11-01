//! The binary's group composition and listeners (protocol/setup.md, 7.1, 7.3).
use super::*;
use serde_json::{Value, json};
use std::{
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
};
use temper_forge_setup::{Group, Kind, Stage, Step, Steps};

fn scratch(row: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("temper-box-groups-{}-{row}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("prior scratch");
    }
    fs::create_dir(&root).expect("scratch");
    root
}
fn directories(root: &Path) -> crate::Directories {
    crate::directories::resolve(
        &[
            Box::from(b"--config".as_slice()),
            root.join("config").as_os_str().as_bytes().into(),
            Box::from(b"--data".as_slice()),
            root.join("data").as_os_str().as_bytes().into(),
            Box::from(b"--cache".as_slice()),
            root.join("cache").as_os_str().as_bytes().into(),
        ],
        &[],
        b"/",
    )
    .expect("fixture directories")
    .0
}
fn document(directories: &crate::Directories) -> Value {
    use core::num::NonZeroU16;
    use temper_init::{Content, Facts, Ports};
    let answer = temper_init::files(
        &Facts {
            binary: b"/bin/temper",
            git: Some(b"/usr/bin/git"),
            claude: None,
            login: Some(b"person"),
            ports: Ports {
                clients: NonZeroU16::new(7100).expect("clients"),
                webhooks: NonZeroU16::new(7200).expect("webhooks"),
                forge: NonZeroU16::new(8369).expect("fake forge"),
                owned: NonZeroU16::new(3000).expect("owned forge"),
                llm: NonZeroU16::new(8370).expect("LLM"),
            },
        },
        &temper_init::Directories { config: &directories.config, data: &directories.data },
        &[1; 32],
        &[2; 32],
    )
    .expect("init template");
    let file = answer.files.iter().find(|file| file.name == b"temper.json").expect("document");
    let Content::Bytes { bytes, .. } = &file.content else { panic!("document bytes") };
    serde_json::from_slice(bytes).expect("template JSON")
}
fn read(directories: &crate::Directories, document: &Value) -> crate::setup::Deployment {
    crate::setup::check(&serde_json::to_vec(document).expect("JSON"), &directories.jig()).expect("checked document")
}
fn template(entry: &Value) -> Value {
    let mut template = entry.clone();
    let object = template.as_object_mut().expect("entry object");
    object.remove("forge");
    object.remove("start");
    template
}
fn write_group(directories: &crate::Directories, source: &Value, forge: u16, playground: bool) -> PathBuf {
    let mut entry = source.clone();
    entry["forge"] = forge.into();
    let path = crate::path(&directories.data).join(format!("box/groups/{forge}"));
    fs::create_dir_all(path.parent().expect("groups directory")).expect("groups directory");
    fs::write(&path, serde_json::to_vec(&json!({"setup": entry, "playground": playground})).expect("JSON"))
        .expect("group file");
    path
}

#[test]
fn a_group_file_is_an_asked_group_before_serve_and_the_documents_entry_wins() {
    let root = scratch("files");
    let dirs = directories(&root);
    let document = document(&dirs);
    let deployment = read(&dirs, &document);
    assert!(
        groups::files(&dirs, &deployment.checked, deployment.demo.as_ref(), &dirs.flags(), true)
            .expect("absent groups")
            .is_empty()
    );
    let source = &document["application"]["setup"][1];
    write_group(&dirs, source, 9, false);
    write_group(&dirs, source, 7, false);
    write_group(&dirs, &document["application"]["setup"][0], 12, true);
    let overridden = write_group(&dirs, source, 3, false);
    fs::remove_file(&overridden).expect("remove group file");
    fs::create_dir(&overridden).expect("document number is never read, even as a directory");
    for name in ["07", "65536", "+8", "notes"] {
        fs::write(overridden.parent().expect("groups").join(name), b"not JSON").expect("skipped entry");
    }
    let first = Step::new(b"peers".as_slice().into(), Box::new([]), Kind::Long);
    let (steps, vacant) = steps(&dirs, &deployment, Box::new([first])).expect("composed groups");
    assert!(vacant.is_empty());
    assert_eq!(steps.always[0].name(), b"peers");
    assert_eq!(steps.asked.iter().map(|group| group.forge).collect::<Vec<_>>(), [1, 3, 7, 9, 12]);
    let before = u32::try_from(steps.always.len() - 1).expect("serve position");
    assert_eq!(steps.always.last().expect("serve").name(), b"serve");
    for forge in [7, 9, 12] {
        let group = steps.asked.iter().find(|group| group.forge == forge).expect("file group");
        assert_eq!(group.before, before);
        assert_eq!(group.recorded, forge != 12);
    }
    let document_group = steps.asked.iter().find(|group| group.forge == 3).expect("document group");
    assert_eq!(document_group.steps[0].1.arguments().last().expect("forge argument").as_ref(), b"3");
    let playground = steps.asked.iter().find(|group| group.forge == 12).expect("playground");
    assert_eq!(playground.steps[0].1.name(), b"fake llm");
    assert_eq!(playground.steps[0].1.listeners(), &[deployment.demo.as_ref().expect("demo").llm().listen()]);
    assert!(!crate::path(&dirs.data).join("box/started").exists());
    // The same placed, recorded group follows the box's existing startup policy.
    for recorded in [false, true] {
        let spawned = startup_names(steps.clone(), &deployment.limits, if recorded { &[7, 12] } else { &[] });
        assert_eq!(spawned.iter().any(|name| name == b"prepare 7"), recorded);
        assert!(!spawned.iter().any(|name| name == b"fake llm"), "even a playground record cannot start it");
        assert_eq!(spawned.last().expect("serve starts").as_slice(), b"serve");
    }
    fs::remove_dir_all(root).expect("cleanup");
}

fn startup_names(steps: Steps, limits: &temper_box::limits::Limits, recorded: &[u16]) -> Vec<Vec<u8>> {
    let placed = temper_box::layout::flatten(steps);
    let mut machine = temper_box::machine::Machine::new(limits, &placed).expect("policy");
    let mut orders = skein_lib::Queue::with_capacity(64);
    let now = skein_lib::Time::ZERO;
    machine.start(now, recorded, &mut orders);
    let mut spawned = Vec::new();
    while let Some(order) = orders.pop() {
        use temper_box::machine::{Input, Order};
        match order {
            Order::Spawn { step } => {
                let command = &placed[step].step;
                spawned.push(command.name().to_vec());
                machine.input(now, Input::Started { step }, &mut orders);
                if command.kind() == Kind::Once {
                    machine.input(now, Input::Exited { step, exit: Exit::Code(0) }, &mut orders);
                } else if command.ready().is_some() {
                    machine.input(now, Input::Ready { step }, &mut orders);
                }
            }
            Order::Close { step } => machine.input(now, Input::Closed { step }, &mut orders),
            Order::Hold { .. } | Order::Wait { .. } | Order::Terminal { .. } | Order::Say { .. } => {}
            Order::Record { .. }
            | Order::Read { .. }
            | Order::Fill { .. }
            | Order::Remove { .. }
            | Order::Withdraw { .. }
            | Order::Signal { .. } => panic!("unexpected startup order {order:?}"),
        }
    }
    spawned
}

#[test]
fn a_refused_group_file_refuses_the_start_naming_its_path() {
    let root = scratch("refusal");
    let dirs = directories(&root);
    let document = document(&dirs);
    let deployment = read(&dirs, &document);
    let path = write_group(&dirs, &document["application"]["setup"][1], 7, false);
    let mut padded = fs::read(&path).expect("valid fixture group");
    fs::write(&path, b"{\"setup\":{},\"playground\":false}").expect("malformed setup");
    let bytes = serde_json::to_vec(&document).expect("document JSON");
    let why = run_configuration(&dirs, &bytes, Vec::new(), Box::new([]), Echo::None).expect_err("start refused");
    assert!(why.starts_with(&format!("{}: ", path.display())), "{why}");
    assert!(why.contains("setup.forge"), "{why}");
    padded.resize(usize::try_from(temper_forge_setup::file::BYTES).expect("bound"), b' ');
    fs::write(&path, &padded).expect("file at byte bound");
    assert!(steps(&dirs, &deployment, Box::new([])).is_ok(), "the whole-file bound is inclusive");
    padded.push(b' ');
    fs::write(&path, padded).expect("oversized file");
    let why = steps(&dirs, &deployment, Box::new([])).expect_err("bounded read");
    assert!(why.contains(&path.display().to_string()) && why.contains("TooLarge"), "{why}");
    fs::remove_file(path).expect("remove refused file");
    let path = write_group(&dirs, &document["application"]["setup"][0], 8, true);
    let why = groups::files(&dirs, &deployment.checked, None, &dirs.flags(), true).expect_err("playground needs LLM");
    assert_eq!(why, format!("{}: playground requires application.demo", path.display()));
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn no_machine_member_gives_no_kinds_group() {
    let root = scratch("no-kinds");
    let dirs = directories(&root);
    let document = document(&dirs);
    let deployment = read(&dirs, &document);
    let (_, vacant) = steps(&dirs, &deployment, Box::new([])).expect("ordinary steps");
    assert!(vacant.is_empty());
    assert!(groups::kinds(None, None, &dirs.flags(), true).is_empty());
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn the_machine_members_give_the_owned_and_playground_kinds_numbered_65535() {
    let root = scratch("kinds");
    let dirs = directories(&root);
    let mut document = document(&dirs);
    document["application"]["owned"] = template(&document["application"]["setup"][1]);
    document["application"]["demo"]["fake"] = template(&document["application"]["setup"][0]);
    write_group(&dirs, &document["application"]["setup"][1], 7, false);
    let deployment = read(&dirs, &document);
    let (steps, vacant) = steps(&dirs, &deployment, Box::new([])).expect("kinds and file groups");
    assert_eq!(vacant.len(), 2);
    for group in &vacant {
        assert_eq!(group.forge, u16::MAX);
        assert_eq!(usize::try_from(group.before).expect("position"), steps.always.len() - 1);
    }
    let owned = vacant.iter().find(|group| group.recorded).expect("owned kind");
    let file = steps.asked.iter().find(|group| group.forge == 7).expect("file group");
    assert_eq!(&owned.numbered(7), file);
    let playground = vacant.iter().find(|group| !group.recorded).expect("playground kind");
    assert_eq!(playground.label.as_ref(), b"Playground");
    assert_eq!(playground.steps[0].1.name(), b"fake llm");
    assert_eq!(playground.steps[0].1.listeners(), &[deployment.demo.as_ref().expect("demo").llm().listen()]);
    fs::remove_dir_all(root).expect("cleanup");
}

fn listening_group(forge: u16, address: std::net::SocketAddr) -> Group {
    Group {
        forge,
        before: 0,
        recorded: true,
        label: format!("Forge {forge}").into_bytes().into_boxed_slice(),
        steps: Box::new([(
            Stage::Run,
            Step::new(b"forge".as_slice().into(), Box::new([]), Kind::Long).listening(Box::new([address])),
        )]),
    }
}
#[test]
fn groups_may_share_a_listener_and_an_always_step_may_not() {
    let root = scratch("listeners");
    let dirs = directories(&root);
    let (fd, _) = skein_shell::open_listener(([127, 0, 0, 1], 0).into(), 1).expect("owned listener");
    let address = skein_shell::listener_at(fd).expect("listener address");
    let steps =
        Steps { always: Box::new([]), asked: Box::new([listening_group(3, address), listening_group(7, address)]) };
    let vacant = [listening_group(u16::MAX, address)];
    let mut owned = vec![fd];
    let listeners = open_listeners(&dirs, &steps, &vacant, Vec::new(), &[(fd, address)], &mut owned)
        .expect("shared inherited listener");
    assert_eq!(listeners, [(fd, address)]);
    assert_eq!(owned, [fd]);
    let always = vacant[0].steps[0].1.clone();
    let conflict = Steps { always: Box::new([always.clone()]), asked: steps.asked };
    let why = open_listeners(&dirs, &conflict, &vacant, Vec::new(), &[(fd, address)], &mut owned)
        .expect_err("always conflict");
    assert!(why.contains("two steps name the same listener"), "{why}");
    crate::close(owned);
    let mut opened = Vec::new();
    let steps = Steps { always: Box::new([]), asked: Box::new([listening_group(7, address)]) };
    let listeners =
        open_listeners(&dirs, &steps, &vacant, Vec::new(), &[], &mut opened).expect("one new listener for both groups");
    assert_eq!(listeners.len(), 1);
    assert_eq!(opened.len(), 1);
    assert!(TcpListener::bind(address).is_err(), "listener is held from startup");
    crate::close(opened);
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn a_vacant_kinds_listener_is_held_from_start_and_accepts_an_inherited_socket() {
    let root = scratch("kind-listener");
    let dirs = directories(&root);
    let (fd, _) = skein_shell::open_listener(([127, 0, 0, 1], 0).into(), 1).expect("listener");
    let address = skein_shell::listener_at(fd).expect("address");
    let vacant = [listening_group(u16::MAX, address)];
    let steps = Steps { always: Box::new([]), asked: Box::new([]) };
    let mut owned = vec![fd];
    assert_eq!(
        open_listeners(&dirs, &steps, &vacant, Vec::new(), &[(fd, address)], &mut owned)
            .expect("kind accepts inherited socket"),
        [(fd, address)]
    );
    assert_eq!(owned, [fd]);
    crate::close(owned);
    let mut owned = Vec::new();
    let listeners = open_listeners(&dirs, &steps, &vacant, Vec::new(), &[], &mut owned).expect("kind opens listener");
    assert_eq!(listeners.len(), 1);
    assert_eq!(listeners[0].1, address);
    assert!(TcpListener::bind(address).is_err());
    crate::close(owned);
    fs::remove_dir_all(root).expect("cleanup");
}

fn notice(group: &Group) -> &[u8] {
    group
        .steps
        .iter()
        .map(|(_, step)| step)
        .find(|step| step.ready().is_some())
        .expect("owned Forgejo")
        .started_notice()
}
#[test]
fn each_owned_group_takes_its_own_entrys_notice() {
    let root = scratch("notices");
    let dirs = directories(&root);
    let mut document = document(&dirs);
    document["application"]["owned"] = template(&document["application"]["setup"][1]);
    document["application"]["owned"]["mode"]["owned"]["person"]["login"] = "kind-person".into();
    let mut source = document["application"]["setup"][1].clone();
    source["mode"]["owned"]["person"]["login"] = "file-person".into();
    source["directory"] = "file-forge".into();
    write_group(&dirs, &source, 7, false);
    let deployment = read(&dirs, &document);
    let (steps, vacant) = steps(&dirs, &deployment, Box::new([])).expect("notices");
    for (forge, login, directory, group) in [
        (
            3,
            "person",
            deployment.checked.setup.entry(3).expect("document entry").directory.as_ref(),
            steps.asked.iter().find(|group| group.forge == 3).expect("document group"),
        ),
        (
            7,
            "file-person",
            crate::path(&dirs.data).join("file-forge").as_os_str().as_bytes(),
            steps.asked.iter().find(|group| group.forge == 7).expect("file group"),
        ),
        (
            u16::MAX,
            "kind-person",
            deployment.owned.as_ref().expect("owned kind").directory.as_ref(),
            vacant.iter().find(|group| group.recorded).expect("kind group"),
        ),
    ] {
        let expected = format!(
            "Forge {forge} (Forgejo): started\nForge {forge} (Forgejo): api http://127.0.0.1:3000/api/v1\nForge {forge} (Forgejo): person {login}, read header {}/person_read.header, write header {}/person_write.header\n",
            String::from_utf8_lossy(directory),
            String::from_utf8_lossy(directory)
        );
        assert_eq!(notice(group), expected.as_bytes());
    }
    fs::remove_dir_all(root).expect("cleanup");
}
