use super::fixtures::secret;
use crate::{
    call::{Auth, Call},
    reconcile::Need,
};
use alloc::boxed::Box;
use skein_forge::{forgejo, github};

#[test]
fn a_request_converts_into_the_call_of_its_dialect() {
    let forgejo = forgejo::request::Request::User;
    let github = github::request::Request::App;
    assert_eq!(Call::from(forgejo.clone()), Call::Forgejo(forgejo));
    assert_eq!(Call::from(github.clone()), Call::GitHub(github));
}

fn credentials() -> [Auth; 3] {
    [
        Auth::Token(secret(b"hunter2-secret")),
        Auth::App { id: 7, key: secret([[0x30, 0x82, 0xAB, 0xCD]; 8].as_flattened()) },
        Auth::Bearer(secret(b"installation-secret")),
    ]
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the test checks the entire credential Debug rendering for redaction")]
fn auth_debug_shows_a_length_and_never_a_byte() {
    let expected =
        ["Token(Secret { len: 14 })", "App { id: 7, key: Secret { len: 32 } }", "Bearer(Secret { len: 19 })"];
    for (auth, expected) in credentials().into_iter().zip(expected) {
        assert_eq!(format!("{auth:?}"), expected);
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the test checks the entire send Debug rendering for redaction")]
fn a_need_send_debug_holds_no_credential_byte() {
    let expected = [
        "Send { request: GitHub(App), auth: Token(Secret { len: 14 }) }",
        "Send { request: GitHub(App), auth: App { id: 7, key: Secret { len: 32 } } }",
        "Send { request: GitHub(App), auth: Bearer(Secret { len: 19 }) }",
    ];
    for (auth, expected) in credentials().into_iter().zip(expected) {
        let need = Need::Send { request: github::request::Request::App.into(), auth };
        assert_eq!(format!("{need:?}"), expected);
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the test renders plan sends and replies to check secret redaction")]
fn plan_send_and_reply_debug_hold_no_secret_bytes() {
    let send = super::fixtures::send(forgejo::request::Request::CreateUser {
        login: b"kit".as_slice().into(),
        email: Box::new([]),
        full_name: Box::new([]),
        password: secret(b"bot-password"),
        must_change_password: false,
    });
    let reply = super::fixtures::token_event();
    for rendered in [format!("{send:?}"), format!("{reply:?}")] {
        for value in [super::fixtures::ADMIN, b"bot-password", b"new-token"] {
            assert!(skein_lib::bytes::find(rendered.as_bytes(), value).is_none());
            assert!(!rendered.contains(&format!("{value:?}")));
        }
    }
}
