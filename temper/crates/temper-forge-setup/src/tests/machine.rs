use super::fixtures;
use crate::{
    FORGE_ARGUMENT, Kind, Stage, kinds, limits,
    machine::{self, BoxKind, Machine},
    problem::{self, Problems},
    setup::Mode,
};
use alloc::boxed::Box;
use jig_protocol_config::Directories;
use skein_lib::{Writer, bytes};

const TEMPLATE_MACHINE: &[u8] = br#"{
    "owned": {"listen":{"socket":"127.0.0.1:8372"},"remote":"http://127.0.0.1:8372",
        "person":{"login":"ada","email":"ada@example.invalid"},"forgejo":"fetched",
        "runner":{"binary":"fetched","labels":["temper-host:host"],"capacity":1,"path":"/usr/local/bin:/usr/bin:/bin"}},
    "fake": {"listen":{"socket":"127.0.0.1:8369"},"remote":"http://127.0.0.1:8369","git":"demo/bin/git"},
    "github": null
}"#;
fn directories() -> Directories {
    Directories::given(b"/configuration", b"/data", b"/cache").unwrap()
}
fn read(input: &[u8], directories: &Directories) -> Machine {
    let mut problems = Problems::empty();
    let result = machine::read(&fixtures::document(input), 0, &limits::small(), directories, &mut problems);
    assert!(problems.is_empty(), "{:?}", problem::lines(&problems));
    result.unwrap()
}
fn replace(input: &[u8], old: &[u8], new: &[u8]) -> Box<[u8]> {
    let at = bytes::find(input, old).unwrap();
    let after = at.checked_add(old.len()).unwrap();
    let size = input.len().checked_sub(old.len()).unwrap().checked_add(new.len()).unwrap();
    let mut writer = Writer::new(size);
    writer.put(&input[..at]).unwrap();
    writer.put(new).unwrap();
    writer.put(&input[after..]).unwrap();
    writer.finish()
}
#[test]
fn the_template_machine_reads_back() {
    let machine = read(TEMPLATE_MACHINE, &directories());
    let owned = machine.owned.unwrap();
    assert_eq!(owned.listen, ([127, 0, 0, 1], 8372).into());
    assert_eq!(owned.remote.as_ref(), b"http://127.0.0.1:8372");
    assert_eq!(owned.person.login.as_ref(), b"ada");
    assert!(owned.public.is_none());
    assert!(owned.fetch.forgejo && owned.fetch.runner);
    assert!(owned.fetch.mirror.is_none());
    assert_eq!(
        owned.forgejo.as_ref(),
        crate::check::joined(&[b"/cache/forgejo/", skein_forge::forgejo::pin::VERSION.as_bytes(), b"/forgejo"])
            .as_ref()
    );
    let runner = owned.runner.unwrap();
    assert_eq!(runner.capacity, 1);
    assert_eq!(runner.labels.as_ref(), &[Box::from(b"temper-host:host".as_slice())]);
    assert_eq!(runner.path.as_ref(), b"/usr/local/bin:/usr/bin:/bin");
    assert!(runner.binary.starts_with(b"/cache/forgejo-runner/"));
    let fake = machine.fake.unwrap();
    assert_eq!(fake.listen, ([127, 0, 0, 1], 8369).into());
    assert_eq!(fake.remote.as_ref(), b"http://127.0.0.1:8369");
    assert_eq!(fake.git.as_ref(), b"/data/demo/bin/git");
    assert!(machine.github.is_none());
}

#[test]
fn an_unknown_member_is_refused_by_path() {
    for (old, new, path) in [
        (
            b"\"github\": null".as_slice(),
            b"\"github\": null, \"extra\": true".as_slice(),
            b"application.machine.extra".as_slice(),
        ),
        (b"\"forgejo\":\"fetched\"", b"\"forgejo\":\"fetched\",\"extra\":true", b"application.machine.owned.extra"),
        (b"\"capacity\":1", b"\"capacity\":1,\"extra\":true", b"application.machine.owned.runner.extra"),
        (b"\"git\":\"demo/bin/git\"", b"\"git\":\"demo/bin/git\",\"extra\":true", b"application.machine.fake.extra"),
    ] {
        let mut problems = Problems::empty();
        let input = replace(TEMPLATE_MACHINE, old, new);
        assert!(
            machine::read(&fixtures::document(&input), 0, &limits::small(), &directories(), &mut problems).is_none()
        );
        let expected = crate::check::joined(&[path, b": unknown member\n"]);
        assert_eq!(problem::lines(&problems), expected);
    }
}

#[test]
fn owned_entry_uses_its_number_and_machine() {
    for dirs in [Directories::working(), directories()] {
        let machine = read(TEMPLATE_MACHINE, &dirs);
        let entry = machine::entry(&machine, 3, BoxKind::Owned, &dirs).unwrap();
        assert_eq!(entry.forge, 3);
        assert_eq!(entry.directory, dirs.join(jig_protocol_config::Directory::Data, b"forge-3").unwrap());
        assert!(entry.asked);
        assert!(entry.orgs.is_empty());
    }
}

#[test]
fn fake_entry_uses_its_number_and_playground_repository() {
    for dirs in [Directories::working(), directories()] {
        let machine = read(TEMPLATE_MACHINE, &dirs);
        let entry = machine::entry(&machine, 1, BoxKind::Fake, &dirs).unwrap();
        assert_eq!(entry.forge, 1);
        assert_eq!(entry.directory, dirs.join(jig_protocol_config::Directory::Data, b"playground-1").unwrap());
        let Mode::Fake(fake) = entry.mode else { panic!("fake entry") };
        assert_eq!(fake.owner.as_ref(), b"playground");
        assert_eq!(fake.repositories[0].name.as_ref(), b"widget");
        assert_eq!(fake.repositories[0].landing.as_ref(), b"main");
    }
}

#[test]
fn group_files_round_trip_and_refuse_other_words() {
    for (kind, word, file) in
        [(BoxKind::Owned, b"owned".as_slice(), b"owned\n".as_slice()), (BoxKind::Fake, b"fake", b"fake\n")]
    {
        assert_eq!(kind.word(), word);
        assert_eq!(BoxKind::parse(word), Some(kind));
        assert_eq!(machine::group_file(kind).as_ref(), file);
        assert_eq!(machine::parse_group_file(file), Some(kind));
    }
    for invalid in [
        b"".as_slice(),
        b"owned",
        b"fake",
        b"owned\r\n",
        b"fake\n\n",
        b"Owned\n",
        b"managed\n",
        b" owned\n",
        b"owned\nextra",
    ] {
        assert_eq!(machine::parse_group_file(invalid), None);
    }
    for (forge, owned, fake, group) in [
        (0, b"forge-0".as_slice(), b"playground-0".as_slice(), b"box/groups/0".as_slice()),
        (u16::MAX, b"forge-65535", b"playground-65535", b"box/groups/65535"),
    ] {
        assert_eq!(machine::directory(forge, BoxKind::Owned).as_ref(), owned);
        assert_eq!(machine::directory(forge, BoxKind::Fake).as_ref(), fake);
        assert_eq!(machine::group_path(forge).as_ref(), group);
    }
    assert_eq!(BoxKind::parse(b"managed"), None);
}

#[test]
fn the_owned_slot_names_its_forge_by_the_argument() {
    let machine = read(TEMPLATE_MACHINE, &directories());
    let flags = Box::new([Box::from(b"--data".as_slice()), Box::from(b"/data".as_slice())]);
    let listener = ([127, 0, 0, 1], 8367).into();
    for listens in [false, true] {
        let steps = kinds(&machine, flags.as_ref(), Box::new([listener]), listens);
        assert_eq!(steps.always.len(), 1);
        assert_eq!(steps.always[0].name(), b"serve");
        assert_eq!(steps.always[0].listeners(), &[listener]);
        assert_eq!(steps.asked.len(), 1);
        let group = &steps.asked[0];
        assert_eq!(group.kind, Some(BoxKind::Owned));
        assert_eq!((group.forge, group.before), (0, 0));
        assert_eq!(group.label.as_ref(), b"Forgejo");
        assert!(group.recorded);
        assert_eq!(group.steps.len(), 4);
        for ((stage, step), (expected_stage, name, action, run)) in group.steps.iter().zip([
            (Stage::Fetch, b"fetch".as_slice(), b"fetch".as_slice(), None),
            (Stage::Prepare, b"prepare", b"prepare", None),
            (Stage::Run, b"forgejo", b"run", Some(b"forgejo".as_slice())),
            (Stage::Run, b"runner", b"run", Some(b"runner".as_slice())),
        ]) {
            assert_eq!(*stage, expected_stage);
            assert_eq!(step.name(), name);
            assert_eq!(step.kind(), if run.is_some() { Kind::Long } else { Kind::Once });
            assert_eq!(step.arguments()[0].as_ref(), b"forge");
            assert_eq!(step.arguments()[1].as_ref(), action);
            assert_eq!(step.arguments()[2..4], flags[..]);
            assert_eq!(step.arguments()[4].as_ref(), FORGE_ARGUMENT);
            assert_eq!(step.arguments().len(), if run.is_some() { 6 } else { 5 });
            if let Some(run) = run {
                assert_eq!(step.arguments()[5].as_ref(), run);
            }
        }
        let forgejo = &group.steps[2].1;
        let owned_listener = machine.owned.as_ref().unwrap().listen;
        assert_eq!(forgejo.ready(), Some(owned_listener));
        let held = [owned_listener];
        assert_eq!(forgejo.listeners(), if listens { &held[..] } else { &[] });
        let remover = group.remover.as_ref().unwrap();
        assert_eq!(remover.name(), b"remove");
        assert_eq!(remover.kind(), Kind::Once);
        let expected: &[&[u8]] = &[b"forge", b"remove", b"--data", b"/data", b"$forge"];
        assert_eq!(remover.arguments().len(), expected.len());
        for (arg, expected) in remover.arguments().iter().zip(expected) {
            assert_eq!(arg.as_ref(), *expected);
        }
    }
}

#[test]
fn absent_kinds_have_no_entry_or_slot_and_optional_processes_are_omitted() {
    let empty = read(b"{}", &directories());
    for kind in [BoxKind::Owned, BoxKind::Fake] {
        assert!(machine::entry(&empty, 5, kind, &directories()).is_none());
    }
    let steps = kinds(&empty, &[], Box::new([]), false);
    assert_eq!(steps.always.len(), 1);
    assert!(steps.asked.is_empty());
    let mut machine = read(TEMPLATE_MACHINE, &directories());
    let owned = machine.owned.as_mut().unwrap();
    owned.runner = None;
    owned.fetch.forgejo = false;
    owned.fetch.runner = false;
    owned.listen = ([127, 0, 0, 1], 0).into();
    let steps = kinds(&machine, &[], Box::new([]), true);
    assert_eq!(steps.asked[0].steps.len(), 2);
    assert_eq!(steps.asked[0].steps[0].1.name(), b"prepare");
    assert_eq!(steps.asked[0].steps[1].1.name(), b"forgejo");
    assert!(steps.asked[0].steps[1].1.listeners().is_empty());
    let entry = machine::entry(&machine, 42, BoxKind::Owned, &directories()).unwrap();
    assert_eq!(entry.directory.as_ref(), b"/data/forge-42");
    let Mode::Owned(entry_owned) = entry.mode else { panic!("owned") };
    assert!(entry_owned.runner.is_none());
}

#[test]
fn machine_refusals_keep_the_member_path() {
    for (old, new, path) in [
        (b"\"capacity\":1".as_slice(), b"\"capacity\":0".as_slice(), b"owned.runner.capacity".as_slice()),
        (b"\"capacity\":1", b"\"capacity\":65", b"owned.runner.capacity"),
        (b"\"forgejo\":\"fetched\"", b"\"forgejo\":\"../forgejo\"", b"owned.forgejo"),
        (b"\"git\":\"demo/bin/git\"", b"\"git\":\"../git\"", b"fake.git"),
        (b"\"git\":\"demo/bin/git\"", b"\"git\":false", b"fake.git"),
        (b"\"git\":\"demo/bin/git\"", b"\"git\":\"demo/bin/git\",\"git\":\"other\"", b"fake.git"),
        (b"\"git\":\"demo/bin/git\"", b"\"other\":\"git\"", b"fake.git"),
        (b"127.0.0.1:8372", b"192.0.2.1:8372", b"owned.listen"),
        (b"\"ada@example.invalid\"", b"\"bad\"", b"owned.person.email"),
        (b"http://127.0.0.1:8369", b"http://example.invalid", b"fake.remote"),
        (b"http://127.0.0.1:8372", b"http://example.invalid", b"owned.remote"),
    ] {
        let input = replace(TEMPLATE_MACHINE, old, new);
        let mut problems = Problems::empty();
        assert!(
            machine::read(&fixtures::document(&input), 0, &limits::small(), &directories(), &mut problems).is_none()
        );
        assert!(
            bytes::find(&problem::lines(&problems), &crate::check::joined(&[b"application.machine.", path, b":"]))
                .is_some()
        );
    }
}

#[test]
fn github_accepts_a_fake_or_tls_and_refuses_unknown_transport_members() {
    for (transport, tls) in
        [(br#""plaintext""#.as_slice(), false), (br#"{"tls":{"server_name":"localhost","trust":null}}"#, true)]
    {
        let input = crate::check::joined(&[
            br#"{"github":{"api":{"socket":"127.0.0.1:9999"},"transport":"#,
            transport,
            br#", "remote":"http://127.0.0.1:9999","web":"http://127.0.0.1:9999"}}"#,
        ]);
        let github = read(&input, &directories()).github.unwrap();
        assert_eq!(github.tls, tls);
        assert_eq!(github.remote.as_ref(), b"http://127.0.0.1:9999");
        assert_eq!(github.web.as_ref(), b"http://127.0.0.1:9999");
        let input = replace(&input, b"\"transport\":", b"\"unexpected\":1,\"transport\":");
        let mut problems = Problems::empty();
        assert!(
            machine::read(&fixtures::document(&input), 0, &limits::small(), &directories(), &mut problems).is_none()
        );
        assert_eq!(problem::lines(&problems).as_ref(), b"application.machine.github.unexpected: unknown member\n");
    }
}

#[test]
fn owned_public_mirrors_and_explicit_binaries_keep_setup_semantics() {
    let input = replace(
        TEMPLATE_MACHINE,
        b"\"forgejo\":\"fetched\"",
        br#""forgejo":"/opt/forgejo","public":"https://git.example.invalid:8443","mirror":{"forgejo":"https://downloads.example.invalid","runner":"https://runner.example.invalid"}"#,
    );
    let input = replace(&input, b"\"binary\":\"fetched\"", br#""binary":"bin/runner""#);
    let machine = read(&input, &directories());
    let owned = machine.owned.unwrap();
    assert!(!owned.fetch.forgejo && !owned.fetch.runner);
    assert_eq!(owned.forgejo.as_ref(), b"/opt/forgejo");
    assert_eq!(owned.runner.unwrap().binary.as_ref(), b"/cache/bin/runner");
    assert_eq!(owned.public.unwrap().port, 8443);
    assert_eq!(owned.fetch.mirror.unwrap().forgejo.host.as_ref(), b"downloads.example.invalid");
    let input = replace(
        &input,
        br#""public":"https://git.example.invalid:8443""#,
        br#""public":"https://git.example.invalid/path""#,
    );
    let mut problems = Problems::empty();
    assert!(machine::read(&fixtures::document(&input), 0, &limits::small(), &directories(), &mut problems).is_none());
    assert_eq!(problem::lines(&problems).as_ref(), b"application.machine.owned.public: invalid form\n");
}

#[test]
fn github_refuses_non_network_endpoints_and_insecure_remote_places() {
    for (api, transport, remote, path) in [
        (
            br#"{"in_process":"fake"}"#.as_slice(),
            br#"{"tls":{"server_name":"localhost","trust":null}}"#.as_slice(),
            b"https://example.invalid".as_slice(),
            b"api".as_slice(),
        ),
        (br#"{"socket":"192.0.2.1:9999"}"#, br#""plaintext""#, b"https://example.invalid", b"transport"),
        (br#"{"host":"example.invalid","port":443}"#, br#""plaintext""#, b"https://example.invalid", b"transport"),
        (
            br#"{"host":"example.invalid","port":443}"#,
            br#"{"tls":{"server_name":"example.invalid","trust":null}}"#,
            b"http://example.invalid",
            b"remote",
        ),
    ] {
        let input = crate::check::joined(&[
            br#"{"github":{"api":"#,
            api,
            br#","transport":"#,
            transport,
            br#","remote":""#,
            remote,
            br#"","web":"https://example.invalid"}}"#,
        ]);
        let mut problems = Problems::empty();
        assert!(
            machine::read(&fixtures::document(&input), 0, &limits::small(), &directories(), &mut problems).is_none()
        );
        assert!(
            bytes::find(
                &problem::lines(&problems),
                &crate::check::joined(&[b"application.machine.github.", path, b":"])
            )
            .is_some()
        );
    }
}

#[test]
fn kind_slots_fetch_before_preparation_only_when_a_binary_is_fetched() {
    for (forgejo, runner) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut machine = read(TEMPLATE_MACHINE, &directories());
        let owned = machine.owned.as_mut().unwrap();
        owned.fetch.forgejo = forgejo;
        owned.fetch.runner = runner;
        let steps = kinds(&machine, &[], Box::new([]), false);
        let expected: &[Stage] = if forgejo || runner {
            &[Stage::Fetch, Stage::Prepare, Stage::Run, Stage::Run]
        } else {
            &[Stage::Prepare, Stage::Run, Stage::Run]
        };
        assert_eq!(steps.asked[0].steps.len(), expected.len());
        for ((stage, _), expected) in steps.asked[0].steps.iter().zip(expected) {
            assert_eq!(stage, expected);
        }
    }
}
