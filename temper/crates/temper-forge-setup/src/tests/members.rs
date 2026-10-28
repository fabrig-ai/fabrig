// Fixture parsing without a deployment keeps written paths.
use super::fixtures;
use crate::{
    check::{self, Members, User},
    problem,
};
use alloc::boxed::Box;
use core::net::{Ipv4Addr, SocketAddr};
fn refused(bytes: &[u8], members: &Members, expected: &[u8]) {
    let problems =
        check::checked(&fixtures::document(bytes), 0, &jig_protocol_config::Directories::working(), members, None, &[])
            .unwrap_err();
    assert_eq!(&*problem::lines(&problems), expected);
}
#[test]
fn a_forge_whose_api_token_names_another_file_is_refused_naming_its_path() {
    let mut members = fixtures::members();
    members.forges[0].api_token = b"elsewhere".as_slice().into();
    refused(fixtures::TWO, &members, b"application.forges[0].api_token: invalid form\n");
}
#[test]
fn a_set_up_forge_whose_user_is_a_number_is_refused() {
    let mut members = fixtures::members();
    members.forges[0].user = User::Id(42);
    refused(fixtures::TWO, &members, b"application.forges[0].user: invalid form\n");
}
#[test]
fn a_local_forge_at_odds_with_its_setup_names_its_path() {
    for (index, api, plaintext, git, expected) in [
        (1, 4000, true, "http://127.0.0.1:3000", "application.forges[1].api: invalid form\n"),
        (2, 4000, true, "http://127.0.0.1:8369", "application.forges[2].api: invalid form\n"),
        (1, 3000, false, "http://127.0.0.1:3000", "application.forges[1].transport: invalid form\n"),
        (2, 8369, false, "http://127.0.0.1:8369", "application.forges[2].transport: invalid form\n"),
        (1, 3000, true, "http://127.0.0.1:9", "application.forges[1].git: invalid form\n"),
        (2, 8369, true, "http://127.0.0.1:9", "application.forges[2].git: invalid form\n"),
    ] {
        let mut members = fixtures::members();
        members.forges[index].api = Some(SocketAddr::from((Ipv4Addr::LOCALHOST, api)));
        members.forges[index].plaintext = plaintext;
        members.forges[index].git = git.as_bytes().into();
        refused(&fixtures::three(), &members, expected.as_bytes());
    }
}
#[test]
fn a_listen_taken_by_another_listener_is_refused() {
    for (listen, expected) in [
        (3000, b"application.setup[1].mode.owned.listen: invalid form\n".as_slice()),
        (8369, b"application.setup[2].mode.fake.listen: invalid form\n".as_slice()),
    ] {
        let mut members = fixtures::members();
        members.listeners = Box::new([SocketAddr::from((Ipv4Addr::LOCALHOST, listen))]);
        refused(&fixtures::three(), &members, expected);
    }
}
#[test]
fn local_listens_are_unique_across_owned_and_fake_entries() {
    use crate::setup::Mode;
    // Each pair shares the fake's socket and written forge members so only
    // the second entry's listen conflicts, whichever mode comes first.
    for (owned, expected) in [
        (&[0_usize][..], b"application.setup[1].mode.fake.listen: invalid form\n".as_slice()),
        (&[1][..], b"application.setup[1].mode.owned.listen: invalid form\n".as_slice()),
        (&[][..], b"application.setup[1].mode.fake.listen: invalid form\n".as_slice()),
        (&[0, 1][..], b"application.setup[1].mode.owned.listen: invalid form\n".as_slice()),
    ] {
        let mut setup = fixtures::fake_setup();
        let mut first = setup.entry(3).unwrap().clone();
        first.forge = 2;
        first.directory = b"forge-2".as_slice().into();
        setup.entries = Box::new([first, setup.entry(3).unwrap().clone()]);
        for index in owned {
            setup.entries[*index].mode = Mode::Owned(crate::setup::Owned {
                listen: SocketAddr::from((Ipv4Addr::LOCALHOST, 8369)),
                ..fixtures::owned()
            });
        }
        let mut members = fixtures::members();
        members.forges[1].api = Some(SocketAddr::from((Ipv4Addr::LOCALHOST, 8369)));
        members.forges[1].git = b"http://127.0.0.1:8369".as_slice().into();
        let mut problems = problem::Problems::empty();
        check::members(&setup, &members, false, &mut problems);
        assert_eq!(problem::lines(&problems).as_ref(), expected);
    }
}
#[test]
fn all_three_modes_pass_the_whole_setup_check() {
    check::checked(
        &fixtures::document(&fixtures::three()),
        0,
        &jig_protocol_config::Directories::working(),
        &fixtures::members(),
        None,
        &[],
    )
    .expect("all three modes checked");
}
#[test]
fn an_entry_at_odds_with_itself_or_its_forge_is_refused_naming_its_path() {
    for (from, to, expected) in [
        (
            br#""forge":2"#.as_slice(),
            br#""forge":1"#.as_slice(),
            b"application.setup[1].forge: invalid form\n".as_slice(),
        ),
        (br#""directory":"forge-2""#, br#""directory":"forge-1""#, b"application.setup[1].directory: invalid form\n"),
        (br#""login":"ada""#, br#""login":"temper""#, b"application.setup[1].mode.owned.person.login: invalid form\n"),
        (
            br#""login":"ada""#,
            br#""login":"temper-admin""#,
            b"application.setup[1].mode.owned.person.login: invalid form\n",
        ),
        (br#""owner":"demo""#, br#""owner":"temper""#, b"application.setup[2].mode.fake.owner: invalid form\n"),
        (br#""owner":"demo""#, br#""owner":"temper-admin""#, b"application.setup[2].mode.fake.owner: invalid form\n"),
    ] {
        let mut problems = problem::Problems::empty();
        let setup = crate::Setup::read(
            &fixtures::document(&fixtures::changed_in(&fixtures::three(), from, to)),
            0,
            &crate::limits::small(),
            &jig_protocol_config::Directories::working(),
            &mut problems,
        )
        .unwrap();
        check::members(&setup, &fixtures::members(), false, &mut problems);
        assert!(skein_lib::bytes::find(&problem::lines(&problems), expected).is_some(), "case {from:?}");
    }
    let mut setup = fixtures::setup();
    setup.entries[0].asked = true;
    let mut ps = problem::Problems::empty();
    check::members(&setup, &fixtures::members(), false, &mut ps);
    assert_eq!(&*problem::lines(&ps), b"application.setup[0].start: invalid form\n");
    for (token, login, expected) in [
        (b"elsewhere".as_slice(), b"temper".as_slice(), b"application.setup[0].forge: invalid form\n".as_slice()),
        (
            b"forge-1/git_token".as_slice(),
            b"other".as_slice(),
            b"application.setup[0].bot.login: invalid form\n".as_slice(),
        ),
    ] {
        let mut members = fixtures::members();
        members.forges[0].identity_token = token.into();
        members.forges[0].identity_login = login.into();
        refused(fixtures::TWO, &members, expected);
    }
}

#[test]
fn a_webhook_secret_off_the_forge_directory_is_refused() {
    let mut members = fixtures::hooked_members();
    members.forges[0].webhook_secret = Some(b"elsewhere".as_slice().into());
    refused_webhooks(&members, b"application.forges[0].webhook_secret: invalid form\n");
}

fn refused_webhooks(members: &Members, expected: &[u8]) {
    let problems = check::checked(
        &fixtures::document(fixtures::TWO),
        0,
        &jig_protocol_config::Directories::working(),
        members,
        Some(fixtures::listener()),
        &[],
    )
    .unwrap_err();
    assert_eq!(&*problem::lines(&problems), expected);
}
#[test]
fn a_set_up_forge_without_a_webhook_secret_is_refused() {
    let mut members = fixtures::hooked_members();
    members.forges[0].webhook_secret = None;
    refused_webhooks(&members, b"application.forges[0].webhook_secret: invalid form\n");
}

#[test]
fn records_only_local_members_skip_files_but_keep_local_checks() {
    let mut members = fixtures::members();
    for index in [1, 2] {
        let f = &mut members.forges[index];
        f.api_token = Box::new([]);
        f.user = User::Id(0);
        f.identity_token = Box::new([]);
        f.webhook_secret = None;
    }
    let checked = check::checked(
        &fixtures::document(&fixtures::three()),
        0,
        &jig_protocol_config::Directories::working(),
        &members,
        None,
        &[],
    )
    .unwrap();
    assert_eq!(checked.records_only.as_ref(), &[2, 3]);
    members.forges[1].plaintext = false;
    refused(&fixtures::three(), &members, b"application.forges[1].transport: invalid form\n");
}
