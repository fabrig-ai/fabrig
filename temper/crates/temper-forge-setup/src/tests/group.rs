use super::fixtures;
use crate::{
    Kind, Stage, Step,
    group::{self, GroupFile, Machine},
    problem::Problems,
    setup::{Mode, Setup},
};
use alloc::boxed::Box;
use core::net::{Ipv4Addr, SocketAddr};
use skein_lib::{List, Writer, bytes};

const FLAGS: &[&[u8]] = &[b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache"];
// The template's entries, with its machine's ports and person substituted, without forge.
const OWNED: &[u8] = br#"{
 "directory":"forge-3", "bot":{"login":"temper","email":"temper@localhost"}, "start":"asked",
 "mode":{"owned":{"forgejo":"fetched", "runner":{"binary":"fetched","labels":["temper-host:host"],
 "capacity":1,"path":"/usr/local/bin:/usr/bin:/bin"}, "listen":{"socket":"127.0.0.1:3000"},
 "person":{"login":"ada","email":"ada@localhost"}}}}
"#;
const FAKE: &[u8] = br#"{
 "directory":"demo/forge", "bot":{"login":"temper","email":"temper@localhost"},
 "mode":{"fake":{"listen":{"socket":"127.0.0.1:8369"},"git":"demo/bin/git","owner":"playground",
 "repositories":[{"name":"widget","landing":"main"}]}}}
"#;
const FETCH: &[&[u8]] =
    &[b"forge", b"fetch", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"2"];
const PREPARE: &[&[u8]] =
    &[b"forge", b"prepare", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"2"];
const FORGEJO: &[&[u8]] =
    &[b"forge", b"run", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"2", b"forgejo"];
const RUNNER: &[&[u8]] =
    &[b"forge", b"run", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"2", b"runner"];
const FAKE_PREPARE: &[&[u8]] =
    &[b"forge", b"prepare", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"3"];
const FAKE_FORGE: &[&[u8]] =
    &[b"fake", b"forge", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"3"];
const FAKE_LLM: &[&[u8]] = &[b"fake", b"llm", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache"];

fn machine() -> Machine {
    let mut flags = List::with_capacity(u32::try_from(FLAGS.len()).unwrap());
    for flag in FLAGS {
        flags.push(bytes::copy_of(flag)).unwrap();
    }
    Machine {
        flags: flags.into_boxed(),
        directories: jig_protocol_config::Directories::given(b"/config", b"/data", b"/cache").unwrap(),
        limits: crate::limits::small(),
        llm: Some(SocketAddr::from((Ipv4Addr::LOCALHOST, 8370))),
        forgejo_listens: true,
    }
}
fn owned() -> GroupFile {
    let mut entry = fixtures::setup().entry(2).unwrap().clone();
    let Mode::Owned(owned) = &mut entry.mode else { panic!("owned fixture") };
    owned.fetch.forgejo = true;
    owned.fetch.runner = true;
    GroupFile { entry, playground: false }
}
fn fake(playground: bool) -> GroupFile {
    GroupFile { entry: fixtures::fake_setup().entry(3).unwrap().clone(), playground }
}
type ExpectedCommand = (Stage, &'static [u8], Kind, &'static [&'static [u8]]);
fn commands(steps: &[(Stage, Step)], expected: &[ExpectedCommand]) {
    assert_eq!(steps.len(), expected.len());
    for ((stage, step), (want_stage, name, kind, arguments)) in steps.iter().zip(expected) {
        assert_eq!(stage, want_stage);
        assert_eq!(step.name(), *name);
        assert_eq!(step.kind(), *kind);
        assert_eq!(step.arguments().len(), arguments.len());
        for (argument, want) in step.arguments().iter().zip(*arguments) {
            assert_eq!(argument.as_ref(), *want);
        }
    }
}

#[test]
fn an_owned_group_fetches_prepares_then_runs_forgejo_and_its_runner() {
    let built = group::build(&owned(), &machine()).unwrap();
    assert_eq!(built.group.forge, 2);
    assert_eq!(built.group.label.as_ref(), b"Forge 2 (Forgejo)");
    assert!(built.group.recorded);
    assert_eq!(built.group.before, 0);
    commands(
        &built.group.steps,
        &[
            (Stage::Fetch, b"fetch 2", Kind::Once, FETCH),
            (Stage::Prepare, b"prepare 2", Kind::Once, PREPARE),
            (Stage::Run, b"forgejo 2", Kind::Long, FORGEJO),
            (Stage::Run, b"runner 2", Kind::Long, RUNNER),
        ],
    );
    let listen = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000));
    let forgejo = &built.group.steps[2].1;
    assert_eq!(forgejo.ready(), Some(listen));
    assert_eq!(forgejo.listeners(), &[listen]);
    assert!(!forgejo.started_notice().is_empty());
    for index in [0, 1, 3] {
        let step = &built.group.steps[index].1;
        assert_eq!(step.ready(), None);
        assert!(step.listeners().is_empty());
        assert!(step.started_notice().is_empty());
    }
}
#[test]
fn an_owned_group_without_fetched_binaries_or_runner_has_neither_step() {
    let mut file = owned();
    let Mode::Owned(owned) = &mut file.entry.mode else { panic!("owned fixture") };
    owned.fetch.forgejo = false;
    owned.fetch.runner = false;
    owned.runner = None;
    let built = group::build(&file, &machine()).unwrap();
    commands(
        &built.group.steps,
        &[(Stage::Prepare, b"prepare 2", Kind::Once, PREPARE), (Stage::Run, b"forgejo 2", Kind::Long, FORGEJO)],
    );
}
#[test]
fn forgejo_listens_false_leaves_forgejo_its_own_bind() {
    let mut machine = machine();
    machine.forgejo_listens = false;
    let built = group::build(&owned(), &machine).unwrap();
    let forgejo = &built.group.steps[2].1;
    assert!(forgejo.listeners().is_empty());
    assert_eq!(forgejo.ready(), Some(SocketAddr::from((Ipv4Addr::LOCALHOST, 3000))));
}
#[test]
fn the_owned_started_notice_names_the_api_the_person_and_its_headers() {
    let mut file = owned();
    file.entry.directory = bytes::copy_of(b"/data/forge-2");
    let built = group::build(&file, &machine()).unwrap();
    assert_eq!(built.group.steps[2].1.started_notice(), b"Forge 2 (Forgejo): started\nForge 2 (Forgejo): api http://127.0.0.1:3000/api/v1\nForge 2 (Forgejo): person ada, read header /data/forge-2/person_read.header, write header /data/forge-2/person_write.header\n");
    let Mode::Owned(owned) = &mut file.entry.mode else { panic!("owned fixture") };
    owned.listen = SocketAddr::V6(core::net::SocketAddrV6::new(core::net::Ipv6Addr::LOCALHOST, 3000, 0, 7));
    let built = group::build(&file, &machine()).unwrap();
    assert_eq!(built.group.steps[2].1.started_notice(), b"Forge 2 (Forgejo): started\nForge 2 (Forgejo): api http://[::1%7]:3000/api/v1\nForge 2 (Forgejo): person ada, read header /data/forge-2/person_read.header, write header /data/forge-2/person_write.header\n");
}
#[test]
fn the_playground_group_runs_the_fake_llm_then_prepares_and_runs_its_forge() {
    let built = group::build(&fake(true), &machine()).unwrap();
    assert_eq!(built.group.label.as_ref(), b"Playground");
    assert!(!built.group.recorded);
    assert_eq!(built.group.before, 0);
    commands(
        &built.group.steps,
        &[
            (Stage::Run, b"fake llm", Kind::Long, FAKE_LLM),
            (Stage::Prepare, b"prepare 3", Kind::Once, FAKE_PREPARE),
            (Stage::Run, b"fake forge 3", Kind::Long, FAKE_FORGE),
        ],
    );
    assert_eq!(built.group.steps[0].1.listeners(), &[SocketAddr::from((Ipv4Addr::LOCALHOST, 8370))]);
    assert!(built.group.steps[1].1.listeners().is_empty());
    assert_eq!(built.group.steps[2].1.listeners(), &[SocketAddr::from((Ipv4Addr::LOCALHOST, 8369))]);
}
#[test]
fn a_playground_group_without_an_llm_port_is_refused() {
    let mut machine = machine();
    machine.llm = None;
    assert_eq!(
        group::build(&fake(true), &machine).unwrap_err().as_ref(),
        b"forge 3 is the playground: the machine has no fake LLM port"
    );
}
#[test]
fn a_fake_group_outside_the_playground_prepares_and_runs_its_forge_recorded() {
    let mut machine = machine();
    machine.llm = None;
    let built = group::build(&fake(false), &machine).unwrap();
    assert_eq!(built.group.label.as_ref(), b"Forge 3 (fake)");
    assert!(built.group.recorded);
    assert_eq!(built.group.before, 0);
    commands(
        &built.group.steps,
        &[
            (Stage::Prepare, b"prepare 3", Kind::Once, FAKE_PREPARE),
            (Stage::Run, b"fake forge 3", Kind::Long, FAKE_FORGE),
        ],
    );
    assert_eq!(built.group.steps[1].1.listeners(), &[SocketAddr::from((Ipv4Addr::LOCALHOST, 8369))]);
}
#[test]
fn a_managed_entry_builds_no_group() {
    let file = GroupFile { entry: fixtures::setup().entry(1).unwrap().clone(), playground: false };
    assert_eq!(
        group::build(&file, &machine()).unwrap_err().as_ref(),
        b"forge 1 is managed: the box runs no group for it"
    );
}
#[test]
fn every_group_has_its_removal_step() {
    // Both fake variants and the owned kind run a group; managed entries are refused.
    for mut file in [owned(), fake(true), fake(false)] {
        file.entry.forge = 65535;
        let built = group::build(&file, &machine()).unwrap();
        let remove = built.remove;
        assert_eq!(remove.name(), b"remove 65535");
        assert_eq!(remove.kind(), Kind::Once);
        let expected: &[&[u8]] =
            &[b"forge", b"remove", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"65535"];
        assert_eq!(remove.arguments().len(), expected.len());
        for (argument, want) in remove.arguments().iter().zip(expected) {
            assert_eq!(argument.as_ref(), *want);
        }
        assert_eq!(remove.ready(), None);
        assert!(remove.listeners().is_empty());
    }
}

#[test]
fn compose_then_read_gives_the_entry_numbered_and_resolved() {
    let machine = machine();
    for (source, playground, directory, executable) in [
        (OWNED, false, b"/data/forge-3".as_slice(), b"/cache/forgejo/".as_slice()),
        (FAKE, true, b"/data/demo/forge".as_slice(), b"/data/demo/bin/git".as_slice()),
    ] {
        let composed = group::compose(source, 42, playground).unwrap();
        let file = group::read(&composed, 42, &machine).unwrap();
        // The setup list uses the same JSON entry, independently of group::read.
        let setup_json = crate::check::joined(&[b"[{\"forge\":42,", source.get(1..).unwrap(), b"]"]);
        let mut problems = Problems::empty();
        let setup =
            Setup::read(&fixtures::document(&setup_json), 0, &machine.limits, &machine.directories, &mut problems)
                .unwrap();
        assert!(problems.is_empty());
        assert_eq!(file.entry, *setup.entry(42).unwrap());
        assert_eq!(file.entry.forge, 42);
        assert_eq!(file.playground, playground);
        assert_eq!(file.entry.directory.as_ref(), directory);
        match &file.entry.mode {
            Mode::Owned(owned) => {
                assert!(owned.forgejo.starts_with(executable));
                assert!(owned.runner.as_ref().unwrap().binary.starts_with(b"/cache/forgejo-runner/"));
            }
            Mode::Fake(fake) => assert_eq!(fake.git.as_ref(), executable),
            Mode::Managed { .. } => panic!("local template entries"),
        }
    }
    for (source, expected) in [
        (b"{}".as_slice(), b"{\"setup\":{\"forge\":7},\"playground\":false}".as_slice()),
        (b" \n{ \t}\n".as_slice(), b"{\"setup\":{\"forge\":7 \t}\n,\"playground\":false}".as_slice()),
        (b" {\"x\": 1 } \n".as_slice(), b"{\"setup\":{\"forge\":7,\"x\": 1 } \n,\"playground\":false}".as_slice()),
    ] {
        assert_eq!(group::compose(source, 7, false).unwrap().as_ref(), expected);
    }
    assert_eq!(group::path(65535).as_ref(), b"box/groups/65535");
}

#[test]
fn read_refuses_another_forge_an_unknown_member_and_an_oversized_file() {
    let machine = machine();
    let valid = group::compose(FAKE, 3, true).unwrap();
    let unknown = crate::check::joined(&[b"{\"extra\":null,", valid.get(1..).unwrap()]);
    let wrong_playground = fixtures::changed_in(&valid, b"\"playground\":true", b"\"playground\":0");
    let nested_unknown = fixtures::changed_in(&valid, b"\"owner\":", b"\"extra\":0,\"owner\":");
    let bad_path = fixtures::changed_in(&valid, b"demo/forge", b"../forge");
    let duplicate = crate::check::joined(&[b"{\"playground\":true,", valid.get(1..).unwrap()]);
    let rows: &[(&[u8], u16, &[u8])] = &[
        (&valid, 4, b"box/groups/4: setup.forge: invalid form"),
        (&unknown, 3, b"box/groups/3: extra: unknown member"),
        (&wrong_playground, 3, b"box/groups/3: playground: wrong value kind"),
        (&nested_unknown, 3, b"box/groups/3: setup.mode.fake.extra: unknown member"),
        (&bad_path, 3, b"box/groups/3: setup.directory: relative path must be non-empty with no .. component"),
        (&duplicate, 3, b"box/groups/3: playground: duplicate"),
        (b"not JSON", 3, b"box/groups/3: invalid form"),
        (b"[]", 3, b"box/groups/3: wrong value kind"),
        (b"{}", 3, b"box/groups/3: setup: missing"),
    ];
    for (input, forge, expected) in rows {
        assert_eq!(group::read(input, *forge, &machine).unwrap_err().as_ref(), *expected);
    }
    let bound = usize::try_from(group::FILE_BYTES).unwrap();
    let at_bound = group::read(&padded(&valid, bound), 3, &machine).unwrap();
    assert_eq!(at_bound.entry.forge, 3);
    let oversized = padded(&valid, bound.checked_add(1).unwrap());
    assert_eq!(group::read(&oversized, 3, &machine).unwrap_err().as_ref(), b"box/groups/3: past its bound");
    let mut bad_limits = machine.clone();
    bad_limits.limits.orgs = 0;
    assert_eq!(
        group::read(&valid, 3, &bad_limits).unwrap_err().as_ref(),
        b"box/groups/3: application.limits.overrides.setup.orgs: outside its type"
    );
}

#[test]
fn compose_refuses_a_setup_that_is_not_an_object_or_names_its_forge() {
    // Empty S's output is 40 bytes; padding S adds the same number of bytes to it.
    const EMPTY_OUTPUT: usize = 40;
    let rows: &[(&[u8], &[u8])] = &[
        (b"[]", b"box/groups/7: setup: wrong value kind"),
        (b"null", b"box/groups/7: setup: wrong value kind"),
        (b"{\"forge\":1}", b"box/groups/7: setup.forge: unknown member"),
        (br#"{"f\u006frge":1}"#, b"box/groups/7: setup.forge: unknown member"),
        (b"{} {}", b"box/groups/7: invalid form"),
        (b"not JSON", b"box/groups/7: invalid form"),
    ];
    for (input, expected) in rows {
        assert_eq!(group::compose(input, 7, false).unwrap_err().as_ref(), *expected);
    }
    let bound = usize::try_from(group::FILE_BYTES).unwrap();
    let padding = bound.checked_sub(EMPTY_OUTPUT).unwrap();
    let source_size = padding.checked_add(2).unwrap();
    assert_eq!(group::compose(&padded(b"{}", source_size), 7, false).unwrap().len(), bound);
    let oversized = padded(b"{}", source_size.checked_add(1).unwrap());
    assert_eq!(group::compose(&oversized, 7, false).unwrap_err().as_ref(), b"box/groups/7: past its bound");
}

#[test]
fn names_and_labels_fit_their_bounds() {
    for mut file in [owned(), fake(true), fake(false)] {
        file.entry.forge = 65535;
        let built = group::build(&file, &machine()).unwrap();
        assert!(built.group.label.len() <= usize::try_from(group::LABEL_BYTES).unwrap());
        assert!(built.group.steps.len() <= usize::try_from(group::STEPS).unwrap());
        for (_, step) in &built.group.steps {
            assert!(step.name().len() <= usize::try_from(group::NAME_BYTES).unwrap());
        }
        assert!(built.remove.name().len() <= usize::try_from(group::NAME_BYTES).unwrap());
    }
}

fn padded(source: &[u8], size: usize) -> Box<[u8]> {
    let mut writer = Writer::new(size);
    writer.put(source).unwrap();
    for _ in source.len()..size {
        writer.put(b" ").unwrap();
    }
    writer.finish()
}
