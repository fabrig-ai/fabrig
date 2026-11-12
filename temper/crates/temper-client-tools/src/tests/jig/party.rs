//! A service creation is keyed and names the project resolved through me.
use crate::tests::fixtures as f;
use jig_client::contract as c;
#[test]
fn party_make_sends_the_named_project_role_and_name_keyed() {
    for project in ["1", "shop"] {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&["party", "make", "--project", project, "--role", "2", "--out", "service", "scout", "--key", "scout"]);
        assert_eq!(f::act_ask(&root, call), &c::Ask::MakeService {
            project: 1,
            name: c::Text::new(b"scout".as_slice().into(), crate::limits::TEXT).unwrap(),
            role: c::HeldRole::Member,
        });
        assert_eq!(root.sent_name(call), Some(f::named("scout")));
        root.done(call, c::Outcome::Made { what: c::Made::Service { party: 9 } });
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 0);
        assert_eq!(printed.stdout.as_ref(), b"made: party 9\n");
        assert_eq!(printed.file, crate::jig::FileEnd::Service { party: 9 });
        assert!(printed.stderr.is_empty());
    }
}
