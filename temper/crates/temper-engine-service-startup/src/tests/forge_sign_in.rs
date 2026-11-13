//! Forge sign-in and its page follow kept records (temper/docs/design/protocol/openid-provider.md, 7, 8 and 9).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary protocol fixtures construct textual requests and inspect page bytes"
)]
use super::fixtures;

#[test]
fn machine_owned_sign_in_is_refused_at_its_path() {
    for (owned_sign_in, sign_in, expected) in [
        ("unknown", "null", "application.machine.owned.sign_in: unknown reference\n"),
        (
            "forgejo",
            r#"{"google":{"provider":3,"client":"client","secret":"google.secret","endpoints":null},"members":[{"address":"ada@localhost","login":"ada"}]}"#,
            "application.machine.owned.sign_in: a box's sign-in takes application.sign_in null\n",
        ),
    ] {
        let input = super::stock::generated();
        let input = fixtures::with(
            &input,
            b"engine.providers",
            br#"[{"number":1,"kind":"link"},{"number":2,"kind":"deployment"},{"number":3,"kind":"application"}]"#,
        );
        let input = fixtures::with(&input, b"engine.owners", br#"[{"provider":3,"subject":"ada@localhost"}]"#);
        let input = fixtures::with(&input, b"engine.clients.provider", br#"{"key":"provider.key","addresses":[1],"clients":[{"id":"forgejo","secret":null,"redirect_uri":"https://forge.example/user/oauth2/temper/callback"}]}"#);
        let owned = format!(
            r#"{{"listen":{{"socket":"127.0.0.1:8368"}},"remote":"http://127.0.0.1:8368","person":{{"login":"ada","email":"ada@localhost"}},"forgejo":"fetched","runner":null,"sign_in":"{owned_sign_in}"}}"#
        );
        let input = fixtures::with(&input, b"application.machine.owned", owned.as_bytes());
        let input = fixtures::with(&input, b"application.sign_in", sign_in.as_bytes());
        let problems =
            crate::startup::named(&fixtures::engine_part(&input), &jig_protocol_config::Directories::working())
                .unwrap_err();
        assert_eq!(core::str::from_utf8(&crate::startup::lines(&problems)).unwrap(), expected);
    }
}

#[test]
fn sign_in_members_login_refuses_the_bot_without_any_forge_entries() {
    for login in ["temper", "TEMPER"] {
        let input = super::stock::generated();
        let input = fixtures::with(
            &input,
            b"engine.providers",
            br#"[{"number":1,"kind":"link"},{"number":2,"kind":"deployment"},{"number":3,"kind":"application"}]"#,
        );
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
