//! Forge sign-in and its page follow kept records (temper/docs/design/protocol/openid-provider.md, 7, 8 and 9).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary protocol fixtures construct textual requests and inspect page bytes"
)]
use super::fixtures;

#[test]
fn application_machine_is_refused_with_its_moved_destination() {
    use skein_lib::Writer;
    let input = super::stock::generated();
    let app = fixtures::value(&input, b"application");
    let prefix = b"{\"machine\":{},";
    let tail = app.strip_prefix(b"{").unwrap();
    let mut write = Writer::new(prefix.len().checked_add(tail.len()).unwrap());
    write.put(prefix).unwrap();
    write.put(tail).unwrap();
    let input = fixtures::with(&input, b"application", &write.finish());
    let Err(problems) =
        crate::startup::named(&fixtures::engine_part(&input), &jig_protocol_config::Directories::working())
    else {
        panic!("application.machine was admitted")
    };
    assert_eq!(
        crate::startup::lines(&problems).as_ref(),
        b"application.machine: moved: a forge's place is chosen when it's set up, and kept with the forge\n"
    );
}

#[test]
fn sign_in_members_login_refuses_the_bot_without_any_forge_entries() {
    for login in ["temper", "TEMPER"] {
        let input = super::stock::generated();
        let input = fixtures::with(&input, b"engine.owners", br#"[{"provider":3,"subject":"ada@localhost"}]"#);
        let sign_in = format!(
            r#"{{"google":{{"provider":3,"client":"client","secret":"google.secret","endpoints":null}},"members":[{{"address":"ada@localhost","login":"{login}"}}]}}"#
        );
        let input = fixtures::with(&input, b"application.sign_in", sign_in.as_bytes());
        let problems =
            crate::startup::named(&fixtures::engine_part(&input), &jig_protocol_config::Directories::working())
                .unwrap_err();
        assert!(
            core::str::from_utf8(&crate::startup::lines(&problems))
                .unwrap()
                .contains("application.sign_in.members[0].login:")
        );
    }
}
