use super::fixtures;
use crate::{
    Setup,
    group::{self, GroupFile, GroupKind},
    limits,
    problem::Problems,
    setup::Mode,
};
use jig_protocol_config as config;

fn directories() -> config::Directories {
    config::Directories::given(b"/config", b"/data", b"/cache").unwrap()
}

fn setup(bytes: &[u8]) -> Setup {
    let mut problems = Problems::empty();
    let setup = Setup::read(&fixtures::document(bytes), 0, &limits::small(), &directories(), &mut problems).unwrap();
    assert!(problems.is_empty());
    setup
}

fn owned() -> GroupFile {
    GroupFile { forge: 2, kind: GroupKind::Owned, entry: setup(fixtures::TWO).entries[1].clone() }
}

#[test]
fn an_owned_group_file_round_trips_with_its_resolved_paths() {
    for fetched in [false, true] {
        let bytes = if fetched {
            fixtures::changed_in(
                &fixtures::changed(br#""forgejo":"/opt/forgejo/forgejo""#, br#""forgejo":"fetched""#),
                br#""binary":"/opt/forgejo/forgejo-runner""#,
                br#""binary":"fetched""#,
            )
        } else {
            fixtures::changed(br#""forgejo":"/opt/forgejo/forgejo""#, br#""forgejo":"bin/forgejo""#)
        };
        let mut file = GroupFile { forge: 2, kind: GroupKind::Owned, entry: setup(&bytes).entries[1].clone() };
        file.entry.asked = true;
        let Mode::Owned(value) = &mut file.entry.mode else { panic!("owned fixture") };
        assert!(value.forgejo.starts_with(b"/cache/"));
        value.public = Some(crate::setup::Origin { host: b"forge.example.invalid".as_slice().into(), port: 8443 });
        value.fetch.mirror = Some(crate::setup::Mirror {
            forgejo: crate::setup::Origin { host: b"mirror.example.invalid".as_slice().into(), port: 443 },
            runner: crate::setup::Origin { host: b"mirror.example.invalid".as_slice().into(), port: 8443 },
        });
        assert_eq!(file.entry.directory.as_ref(), b"/data/forge-2");
        if let Some(runner) = &mut value.runner {
            runner.labels = Box::new([Box::from(b"host:\"quoted\"\\label".as_slice())]);
            runner.path = b"/bin:/path with \"quotes\"".as_slice().into();
        }
        assert_eq!(group::decode(&group::encode(&file), &limits::small(), &directories()).unwrap(), file);
        value_without_runner_round_trips(file);
    }
}

fn value_without_runner_round_trips(mut file: GroupFile) {
    let Mode::Owned(value) = &mut file.entry.mode else { panic!("owned fixture") };
    value.runner = None;
    value.fetch.runner = false;
    assert_eq!(group::decode(&group::encode(&file), &limits::small(), &directories()).unwrap(), file);
}

#[test]
fn a_fake_group_file_round_trips_with_and_without_the_llm() {
    for llm in [false, true] {
        let entry = setup(&fixtures::three()).entries[2].clone();
        let file = GroupFile { forge: 3, kind: GroupKind::Fake { llm }, entry };
        assert_eq!(file.entry.directory.as_ref(), b"/data/demo/forge");
        let Mode::Fake(fake) = &file.entry.mode else { panic!("fake fixture") };
        assert_eq!(fake.git.as_ref(), b"/data/demo/bin/git");
        assert_eq!(group::decode(&group::encode(&file), &limits::small(), &directories()).unwrap(), file);
    }
}

#[test]
fn a_group_file_refuses_a_disagreeing_number_a_wrong_mode_and_trailing_bytes() {
    let encoded = group::encode(&owned());
    for (from, to) in [
        (br#"{"forge":2"#.as_slice(), br#"{"forge":3"#.as_slice()),
        (br#""kind":"owned""#, br#""kind":"fake""#),
        (br#""kind":"owned""#, br#""kind":"owned","llm":false""#),
        (br#""kind":"owned""#, br#""kind":"other""#),
        (br#""kind":"owned""#, br#""kind":"owned","extra":true""#),
        (br#""kind":"owned""#, br#""kind":"owned","kind":"owned""#),
        (br#""mode":{"owned""#, br#""unknown":true,"mode":{"owned""#),
    ] {
        let bytes = fixtures::changed_in(&encoded, from, to);
        assert!(!group::decode(&bytes, &limits::small(), &directories()).unwrap_err().is_empty());
    }
    for trailing in [b"x".as_slice(), b"{}", b"\x00"] {
        let bytes = crate::check::joined(&[&encoded, trailing]);
        assert!(!group::decode(&bytes, &limits::small(), &directories()).unwrap_err().is_empty());
    }
    let mut file = owned();
    file.entry = setup(&fixtures::three()).entries[2].clone();
    file.forge = file.entry.forge;
    assert!(!group::decode(&group::encode(&file), &limits::small(), &directories()).unwrap_err().is_empty());
    let managed = br#"{"forge":1,"kind":"owned","setup":{"forge":1,"bot":{"login":"temper","email":"temper@example.invalid"},"mode":"managed"}}"#;
    assert!(!group::decode(managed, &limits::small(), &directories()).unwrap_err().is_empty());
    let bytes = group::encode(&GroupFile { kind: GroupKind::Fake { llm: false }, ..file });
    for (from, to) in [
        (br#""llm":false"#.as_slice(), br#""llm":null"#.as_slice()),
        (br#""directory":"/data/demo/forge""#, br#""directory":"../forge""#),
        (br#""git":"/data/demo/bin/git""#, br#""git":"../git""#),
        (br#""forge":3,"directory""#, br#""forge":2,"directory""#),
        (br#""mode":{"fake""#, br#""mode":{"other""#),
    ] {
        let bytes = fixtures::changed_in(&bytes, from, to);
        assert!(!group::decode(&bytes, &limits::small(), &directories()).unwrap_err().is_empty());
    }
    let without_llm = fixtures::changed_in(&bytes, br#""llm":false,"#, b"");
    assert_eq!(
        group::decode(&without_llm, &limits::small(), &directories()).unwrap().kind,
        GroupKind::Fake { llm: false }
    );
}

#[test]
fn a_group_file_past_its_bound_is_refused() {
    let encoded = group::encode(&owned());
    let mut writer = skein_lib::Writer::new(usize::try_from(group::GROUP_BYTES).unwrap());
    writer.put(&encoded).unwrap();
    for _ in encoded.len()..usize::try_from(group::GROUP_BYTES).unwrap() {
        writer.put(b" ").unwrap();
    }
    let bytes = writer.finish();
    assert_eq!(group::decode(&bytes, &limits::small(), &directories()).unwrap(), owned());
    let bytes = crate::check::joined(&[&bytes, b" "]);
    assert!(!group::decode(&bytes, &limits::small(), &directories()).unwrap_err().is_empty());
}

#[test]
fn an_owned_group_file_builds_the_steps_its_document_entry_builds() {
    let flags = Box::new([Box::from(b"--data".as_slice()), Box::from(b"/data".as_slice())]);
    for fetched in [false, true] {
        for runner in [false, true] {
            for listens in [false, true] {
                let mut file = owned();
                file.entry.asked = true;
                let Mode::Owned(o) = &mut file.entry.mode else { panic!("owned fixture") };
                o.fetch.forgejo = fetched;
                if !runner {
                    o.runner = None;
                }
                let built = crate::step::group(&file, flags.as_ref(), Box::new([]), listens);
                let setup = Setup { entries: Box::new([file.entry]) };
                let document = crate::steps(&setup, flags.as_ref(), Box::new([]), listens);
                assert_eq!(document.asked.as_ref(), core::slice::from_ref(&built));
                assert!(built.recorded);
                assert_eq!(built.before, 0);
                assert_eq!(built.label.as_ref(), b"Forge 2 (Forgejo)");
                assert_eq!(built.steps.len(), 2 + usize::from(fetched) + usize::from(runner));
                assert_eq!(built.steps[0].0, if fetched { crate::Stage::Fetch } else { crate::Stage::Prepare });
            }
        }
    }
}
