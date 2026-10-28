// Fixture parsing without a deployment keeps written paths.
use super::fixtures;
use crate::{
    limits,
    problem::{self, Problems},
    setup::{Mode, Setup},
};
use core::net::{Ipv4Addr, SocketAddr};
#[test]
fn an_owned_entry_with_a_runner_reads_into_its_members() {
    let document = fixtures::document(fixtures::TWO);
    let mut problems = Problems::empty();
    let setup =
        Setup::read(&document, 0, &limits::small(), &jig_protocol_config::Directories::working(), &mut problems)
            .expect("design example");
    assert!(problems.is_empty());
    let entry = setup.entry(2).unwrap();
    assert_eq!(&*entry.directory, b"forge-2");
    let Mode::Owned(owned) = &entry.mode else { panic!("owned") };
    assert_eq!(owned.listen, SocketAddr::from((Ipv4Addr::LOCALHOST, 3000)));
    assert_eq!(owned.runner.as_ref().unwrap().capacity, 1);
    assert!(entry.orgs.is_empty());
}
#[test]
fn a_fake_entry_reads_every_member_and_joins_data_paths() {
    let directories = jig_protocol_config::Directories::given(b"/configuration", b"/data", b"/cache").unwrap();
    let mut problems = Problems::empty();
    let setup =
        Setup::read(&fixtures::document(&fixtures::three()), 0, &limits::small(), &directories, &mut problems).unwrap();
    let entry = setup.entry(3).unwrap();
    assert_eq!(
        entry,
        &crate::setup::Entry {
            forge: 3,
            directory: b"/data/demo/forge".as_slice().into(),
            bot: crate::setup::Account {
                login: b"temper".as_slice().into(),
                email: b"temper@example.invalid".as_slice().into()
            },
            orgs: Box::new([]),
            asked: false,
            mode: Mode::Fake(crate::setup::Fake {
                listen: SocketAddr::from((Ipv4Addr::LOCALHOST, 8369)),
                git: b"/data/demo/bin/git".as_slice().into(),
                owner: b"demo".as_slice().into(),
                repositories: Box::new([crate::setup::Repository {
                    name: b"widget".as_slice().into(),
                    landing: b"main".as_slice().into()
                }]),
            }),
        }
    );
    assert!(problems.is_empty());
}
#[test]
fn a_managed_entry_reads_its_admin_token_file() {
    let setup = fixtures::setup();
    assert_eq!(
        setup.entry(1).unwrap().mode,
        Mode::Managed { admin_token: Some(b"secrets/forgejo-admin".as_slice().into()) }
    );
}
#[test]
fn an_owned_entry_reads_a_null_runner_as_none_and_refuses_a_missing_one() {
    let from = br#""runner":{"binary":"/opt/forgejo/forgejo-runner","labels":["temper-host:host"],"capacity":1,"path":"/usr/local/bin:/usr/bin:/bin"},"#;
    let null = fixtures::changed(from, br#""runner":null,"#);
    let setup = Setup::read(
        &fixtures::document(&null),
        0,
        &limits::small(),
        &jig_protocol_config::Directories::working(),
        &mut Problems::empty(),
    )
    .unwrap();
    let Mode::Owned(owned) = &setup.entry(2).unwrap().mode else { panic!("owned") };
    assert_eq!(owned.runner, None);
    refused(&fixtures::changed(from, b""), b"application.setup[1].mode.owned.runner: missing\n");
}
fn refused(bytes: &[u8], expected: &[u8]) {
    let mut problems = Problems::empty();
    assert_eq!(
        Setup::read(
            &fixtures::document(bytes),
            0,
            &limits::small(),
            &jig_protocol_config::Directories::working(),
            &mut problems
        ),
        None
    );
    assert_eq!(&*problem::lines(&problems), expected);
}
#[test]
fn an_entry_past_the_orgs_limit_is_refused() {
    let bytes = fixtures::changed(br#""orgs":["acme"]"#, br#""orgs":["acme","extra"]"#);
    let limits = skein_forge_setup::limits::Limits { orgs: 1, ..limits::small() }; // Below the shipped eight.
    let mut problems = Problems::empty();
    assert_eq!(
        Setup::read(
            &fixtures::document(&bytes),
            0,
            &limits,
            &jig_protocol_config::Directories::working(),
            &mut problems
        ),
        None
    );
    assert_eq!(&*problem::lines(&problems), b"application.setup[0].orgs: past its bound\n");
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table names each malformed member and its refusal path")]
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary Rust test fixtures (programming-model.md, 10.2)"
)]
fn a_member_outside_its_bounds_is_refused_naming_its_path() {
    // protocol/setup.md, 8: eight repositories, names 100 bytes, branches 256.
    const NAME_BYTES: usize = 100;
    const BRANCH_BYTES: usize = 256;
    const REPOSITORIES: usize = 8;
    let long_name = format!("\"name\":\"{}\"", "x".repeat(NAME_BYTES + 1));
    let long_branch = format!("\"landing\":\"{}\"", "x".repeat(BRANCH_BYTES + 1));
    let too_many =
        format!("\"repositories\":[{}]", [r#"{"name":"widget","landing":"main"}"#; REPOSITORIES + 1].join(","));
    let document = fixtures::three();
    for (from, to, expected) in [
        (
            br#""forge":2"#.as_slice(),
            br#""forge":2,"start":"now""#.as_slice(),
            b"application.setup[1].start: invalid form\n".as_slice(),
        ),
        (br#""managed":{"admin_token":"secrets/forgejo-admin"}"#, b"", b"application.setup[0].mode: invalid form\n"),
        (
            br#""mode":{"managed":{"admin_token":"secrets/forgejo-admin"}}"#,
            br#""mode":{},"extra":0"#,
            b"application.setup[0].mode: invalid form\napplication.setup[0].extra: unknown member\n",
        ),
        (
            br#""orgs":["acme"],"mode":{"managed":{"admin_token":"secrets/forgejo-admin"}}"#,
            br#""orgs":false,"mode":{}"#,
            b"application.setup[0].mode: invalid form\napplication.setup[0].orgs: wrong value kind\n",
        ),
        (
            br#""managed":{"admin_token":"secrets/forgejo-admin"}"#,
            br#""managed":{},"owned":{}"#,
            b"application.setup[0].mode: invalid form\n",
        ),
        (
            br#""managed":{"admin_token":"secrets/forgejo-admin"}"#,
            br#""managed":{},"fake":{}"#,
            b"application.setup[0].mode: invalid form\n",
        ),
        (
            br#""managed":{"admin_token":"secrets/forgejo-admin"}"#,
            br#""managed":{},"owned":{},"fake":{}"#,
            b"application.setup[0].mode: invalid form\n",
        ),
        (br#""forge":1"#, br#""forge":1,"start":"asked""#, b"application.setup[0].start: invalid form\n"),
        (br#""forge":2"#, br#""forge":2,"start":true"#, b"application.setup[1].start: wrong value kind\n"),
        (
            br#""orgs":["acme"]"#.as_slice(),
            br#""orgs":[]"#.as_slice(),
            b"application.setup[0].orgs: invalid form\n".as_slice(),
        ),
        (br#""capacity":1"#, br#""capacity":0"#, b"application.setup[1].mode.owned.runner.capacity: invalid form\n"),
        (
            br#""labels":["temper-host:host"]"#,
            br#""labels":[]"#,
            b"application.setup[1].mode.owned.runner.labels: invalid form\n",
        ),
        (
            br#""forgejo":"/opt/forgejo/forgejo""#,
            br#""forgejo":"""#,
            b"application.setup[1].mode.owned.forgejo: invalid form\n",
        ),
        (
            br#""binary":"/opt/forgejo/forgejo-runner""#,
            br#""binary":"""#,
            b"application.setup[1].mode.owned.runner.binary: invalid form\n",
        ),
        (
            br#""socket":"127.0.0.1:8369""#,
            br#""socket":"192.0.2.1:8369""#,
            b"application.setup[2].mode.fake.listen: invalid form\n",
        ),
        (
            br#""socket":"127.0.0.1:8369""#,
            br#""host":"localhost","port":8369"#,
            b"application.setup[2].mode.fake.listen: invalid form\n",
        ),
        (br#""git":"demo/bin/git""#, br#""git":"""#, b"application.setup[2].mode.fake.git: invalid form\n"),
        (br#""owner":"demo""#, br#""owner":"bad/name""#, b"application.setup[2].mode.fake.owner: invalid form\n"),
        (
            br#""repositories":[{"name":"widget","landing":"main"}]"#,
            br#""repositories":[]"#,
            b"application.setup[2].mode.fake.repositories: invalid form\n",
        ),
        (
            br#""name":"widget""#,
            br#""name":"bad/name""#,
            b"application.setup[2].mode.fake.repositories[0].name: invalid form\n",
        ),
        (
            br#""landing":"main""#,
            br#""landing":"bad..branch""#,
            b"application.setup[2].mode.fake.repositories[0].landing: invalid form\n",
        ),
        (
            br#""repositories":[{"name":"widget","landing":"main"}]"#,
            br#""repositories":[{"name":"widget","landing":"main"},{"name":"widget","landing":"other"}]"#,
            b"application.setup[2].mode.fake.repositories[1].name: invalid form\n",
        ),
        (
            br#""repositories":[{"name":"widget","landing":"main"}]"#,
            br#""repositories":[{"name":"widget","landing":"main"},{"name":"Widget","landing":"other"}]"#,
            b"application.setup[2].mode.fake.repositories[1].name: invalid form\n",
        ),
        (
            br#""owner":"demo""#,
            br#""owner":"demo","extra":0"#,
            b"application.setup[2].mode.fake.extra: unknown member\n",
        ),
        (
            br#""name":"widget""#,
            br#""name":"widget","extra":0"#,
            b"application.setup[2].mode.fake.repositories[0].extra: unknown member\n",
        ),
        (br#""mode":{"fake":"#, br#""mode":{"owned":{},"fake":"#, b"application.setup[2].mode: invalid form\n"),
        (br#""forge":3"#, br#""forge":3,"start":"asked""#, b"application.setup[2].start: invalid form\n"),
        (br#""forge":3"#, br#""forge":3,"orgs":[]"#, b"application.setup[2].orgs: invalid form\n"),
        (br#""owner":"demo","#, b"", b"application.setup[2].mode.fake.owner: missing\n"),
        (br#""owner":"demo""#, br#""owner":true"#, b"application.setup[2].mode.fake.owner: wrong value kind\n"),
        (br#""name":"widget","#, b"", b"application.setup[2].mode.fake.repositories[0].name: missing\n"),
        (
            br#""landing":"main""#,
            br#""landing":null"#,
            b"application.setup[2].mode.fake.repositories[0].landing: wrong value kind\n",
        ),
        (
            br#""repositories":[{"name":"widget","landing":"main"}]"#,
            br#""repositories":{}"#,
            b"application.setup[2].mode.fake.repositories: wrong value kind\n",
        ),
        (
            br#""repositories":[{"name":"widget","landing":"main"}]"#,
            br#""repositories":[false]"#,
            b"application.setup[2].mode.fake.repositories[0]: wrong value kind\n",
        ),
        (
            br#""name":"widget""#,
            long_name.as_bytes(),
            b"application.setup[2].mode.fake.repositories[0].name: past its bound\n",
        ),
        (
            br#""landing":"main""#,
            long_branch.as_bytes(),
            b"application.setup[2].mode.fake.repositories[0].landing: past its bound\n",
        ),
        (
            br#""repositories":[{"name":"widget","landing":"main"}]"#,
            too_many.as_bytes(),
            b"application.setup[2].mode.fake.repositories: past its bound\n",
        ),
    ] {
        refused(&fixtures::changed_in(&document, from, to), expected);
    }
}

#[test]
fn setup_paths_join_their_kind_keep_absolute_values_and_refuse_escaping_relatives() {
    let directories = jig_protocol_config::Directories::given(b"/configuration", b"/data", b"/cache").unwrap();
    for (from, to, kind, expected) in [
        (
            br#""directory":"forge-2""#.as_slice(),
            br#""directory":"forge-2""#.as_slice(),
            "directory",
            b"/data/forge-2".as_slice(),
        ),
        (br#""directory":"forge-2""#, br#""directory":"/kept/forge""#, "directory", b"/kept/forge"),
        (br#""forgejo":"/opt/forgejo/forgejo""#, br#""forgejo":"forgejo""#, "forgejo", b"/cache/forgejo"),
        (br#""forgejo":"/opt/forgejo/forgejo""#, br#""forgejo":"/kept/forgejo""#, "forgejo", b"/kept/forgejo"),
        (br#""binary":"/opt/forgejo/forgejo-runner""#, br#""binary":"runner""#, "runner", b"/cache/runner"),
        (br#""binary":"/opt/forgejo/forgejo-runner""#, br#""binary":"/kept/runner""#, "runner", b"/kept/runner"),
        (br#""admin_token":"secrets/forgejo-admin""#, br#""admin_token":"admin""#, "admin", b"/data/admin"),
        (br#""admin_token":"secrets/forgejo-admin""#, br#""admin_token":"/kept/admin""#, "admin", b"/kept/admin"),
    ] {
        let mut problems = Problems::empty();
        let setup = Setup::read(
            &fixtures::document(&fixtures::changed(from, to)),
            0,
            &limits::small(),
            &directories,
            &mut problems,
        )
        .unwrap();
        assert!(problems.is_empty());
        let entry = setup.entry(2).unwrap();
        let Mode::Owned(owned) = &entry.mode else { panic!("owned") };
        let value = match kind {
            "directory" => &entry.directory,
            "forgejo" => &owned.forgejo,
            "runner" => &owned.runner.as_ref().unwrap().binary,
            "admin" => {
                let Mode::Managed { admin_token } = &setup.entry(1).unwrap().mode else { panic!("managed") };
                admin_token.as_ref().unwrap()
            }
            _ => unreachable!(),
        };
        assert_eq!(value.as_ref(), expected, "{kind}");
    }
    for (from, to, path) in [
        (
            br#""directory":"forge-2""#.as_slice(),
            br#""directory":"../forge""#.as_slice(),
            "application.setup[1].directory",
        ),
        (
            br#""forgejo":"/opt/forgejo/forgejo""#,
            br#""forgejo":"../forgejo""#,
            "application.setup[1].mode.owned.forgejo",
        ),
        (
            br#""binary":"/opt/forgejo/forgejo-runner""#,
            br#""binary":"../runner""#,
            "application.setup[1].mode.owned.runner.binary",
        ),
    ] {
        let mut problems = Problems::empty();
        assert!(
            Setup::read(
                &fixtures::document(&fixtures::changed(from, to)),
                0,
                &limits::small(),
                &directories,
                &mut problems
            )
            .is_none()
        );
        assert!(problem::lines(&problems).starts_with(path.as_bytes()));
    }
}

#[test]
fn an_owned_entry_reads_start_absent_or_asked() {
    for (text, asked) in [(br#""forge":2"#.as_slice(), false), (br#""forge":2,"start":"asked""#, true)] {
        let bytes = fixtures::changed(br#""forge":2"#, text);
        let setup = Setup::read(
            &fixtures::document(&bytes),
            0,
            &limits::small(),
            &jig_protocol_config::Directories::working(),
            &mut Problems::empty(),
        )
        .unwrap();
        assert_eq!(setup.entry(2).unwrap().asked, asked);
    }
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary Rust test fixtures (programming-model.md, 10.2)"
)]
fn a_fake_entry_accepts_the_repository_and_name_bounds() {
    // protocol/setup.md, 8: eight repositories; names 100 and branches 256 bytes.
    const REPOSITORIES: usize = 8;
    const NAME_BYTES: usize = 100;
    const BRANCH_BYTES: usize = 256;
    let repositories = (0..REPOSITORIES)
        .map(|index| {
            format!(r#"{{"name":"{}{index}","landing":"{}"}}"#, "x".repeat(NAME_BYTES - 1), "x".repeat(BRANCH_BYTES))
        })
        .collect::<Vec<_>>()
        .join(",");
    let bytes = fixtures::changed_in(
        &fixtures::three(),
        br#""repositories":[{"name":"widget","landing":"main"}]"#,
        format!("\"repositories\":[{repositories}]").as_bytes(),
    );
    let mut problems = Problems::empty();
    let setup = Setup::read(
        &fixtures::document(&bytes),
        0,
        &limits::small(),
        &jig_protocol_config::Directories::working(),
        &mut problems,
    )
    .unwrap();
    let Mode::Fake(fake) = &setup.entry(3).unwrap().mode else { panic!("fake") };
    assert_eq!(fake.repositories.len(), REPOSITORIES);
    assert!(problems.is_empty());
}
#[test]
fn a_fake_git_path_keeps_absolute_values_and_refuses_escape() {
    let directories = jig_protocol_config::Directories::given(b"/configuration", b"/data", b"/cache").unwrap();
    for (path, expected) in [
        (br#""git":"/kept/git""#.as_slice(), Ok(b"/kept/git".as_slice())),
        (
            br#""git":"../git""#.as_slice(),
            Err(b"application.setup[2].mode.fake.git: relative path must be non-empty with no .. component\n"
                .as_slice()),
        ),
    ] {
        let bytes = fixtures::changed_in(&fixtures::three(), br#""git":"demo/bin/git""#, path);
        let mut problems = Problems::empty();
        let found = match Setup::read(&fixtures::document(&bytes), 0, &limits::small(), &directories, &mut problems) {
            Some(setup) => match setup.entry(3).unwrap().mode.clone() {
                Mode::Fake(fake) => Ok(fake.git),
                Mode::Managed { .. } | Mode::Owned(_) => panic!("fake"),
            },
            None => Err(problem::lines(&problems)),
        };
        let found = match &found {
            Ok(path) => Ok(path.as_ref()),
            Err(why) => Err(why.as_ref()),
        };
        assert_eq!(found, expected);
    }
}

#[test]
fn an_owned_entry_refuses_person_orgs_as_an_unknown_member() {
    refused(
        &fixtures::changed(br#""login":"ada""#, br#""login":"ada","orgs":["work"]"#),
        b"application.setup[1].mode.owned.person.orgs: unknown member\n",
    );
}

#[test]
fn an_owned_entry_refuses_entry_orgs_as_an_unknown_member() {
    refused(
        &fixtures::changed(br#""forge":2"#, br#""forge":2,"orgs":["work"]"#),
        b"application.setup[1].orgs: unknown member\n",
    );
}
