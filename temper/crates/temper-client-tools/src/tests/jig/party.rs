//! Service acts are keyed; reissue consults me before sending.
use crate::tests::fixtures as f;
use jig_client::contract as c;
#[test]
fn party_make_sends_the_named_project_role_and_name_keyed() {
    for project in ["1", "shop"] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&[
            "party",
            "make",
            "--project",
            project,
            "--role",
            "2",
            "--out",
            "service",
            "scout",
            "--key",
            "scout",
        ]);
        assert_eq!(
            f::act_ask(&root, call),
            &c::Ask::MakeService {
                project: 1,
                name: c::Text::new(b"scout".as_slice().into(), crate::limits::TEXT).unwrap(),
                role: c::HeldRole::Member,
            }
        );
        assert_eq!(root.sent_name(call), Some(f::named("scout")));
        root.done(call, c::Outcome::Made { what: c::Made::Service { party: 9 } });
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 0);
        assert_eq!(printed.stdout.as_ref(), b"made: party 9\n");
        assert_eq!(printed.file, crate::jig::FileEnd::Service { party: 9 });
        assert!(printed.stderr.is_empty());
    }
}

pub(super) const PARTY_REISSUE_KEYED: &[&str] = &["party", "reissue", "9", "--out", "service", "--key", "rotate"];

#[test]
fn party_reissue_sends_the_party_with_an_optional_key() {
    for (argv, explicit) in [(f::PARTY_REISSUE_CALL, None), (PARTY_REISSUE_KEYED, Some(f::named("rotate")))] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(argv);
        assert_eq!(f::act_ask(&root, call), &c::Ask::ReissueService { party: f::SERVICE_PARTY });
        let key = root.sent_name(call).expect("keyed reissue");
        if let Some(explicit) = explicit {
            assert_eq!(key, explicit);
        }
    }
}

#[test]
fn party_reissue_does_not_send_for_its_own_party() {
    let mut root = f::signed_in(f::SERVICE_PARTY);
    let call = root.submit(f::PARTY_REISSUE_CALL);
    let printed = root.answer(call).expect("local refusal");
    assert!(!root.sends.contains_key(&call));
    assert_eq!(printed.status, 1);
    assert_eq!(
        printed.stderr.as_ref(),
        b"party 9 is this directory's own party; an owner's directory reissues its credential\n"
    );
}

#[test]
fn party_reissue_explains_the_engines_refusals() {
    for (refusal, message) in [
        (
            c::Refusal::Role { held: None },
            "temper ctl: refused: party 9's credential is reissued by an owner of the deployment, or of every project it holds a role in\n",
        ),
        (
            c::Refusal::NotOffered,
            "temper ctl: refused: party 9 is not a service; only a service's credential is reissued\n",
        ),
        (c::Refusal::Unknown, "party reissue: No visible object with that name\n"),
    ] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(f::PARTY_REISSUE_CALL);
        root.answered(call, c::Answer::Refused { refusal, at: None });
        let printed = root.answer(call).expect("engine refusal");
        assert_eq!((printed.status, printed.stderr.as_ref()), (2, message.as_bytes()));
    }
}
