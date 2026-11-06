//! Public clients retain the bindings of oauth-issuer.md, 5 and 9.
use super::*;
const TYPE: Option<&[u8]> = Some(b"application/x-www-form-urlencoded");
fn mixed() -> Issuer {
    let mut settings = settings();
    settings
        .clients
        .push(Client { id: boxed(b"public"), secret: None, redirect_uri: boxed(REDIRECT) })
        .expect("public slot");
    Issuer::new(settings, limits()).expect("mixed issuer")
}
fn query() -> Box<[u8]> {
    replace(QUERY, "client_id=client", "client_id=public")
}
fn body() -> Box<[u8]> {
    replace(&replace(BODY, "client_id=client", "client_id=public"), "&client_secret=secret", "")
}
#[test]
fn public_client_accepts_each_single_method_and_ignores_its_password() {
    let issuer = mixed();
    let authorization = issuer.authorize(&query()).expect("public authorization");
    for password in [b"".as_slice(), b"any-password"] {
        let header = token::basic(&[b"public:".as_slice(), password].concat());
        let request = issuer.token_request(Some(&header), TYPE, &body()).expect("public Basic");
        issuer.redeem(&request, &authorization).expect("public redemption");
    }
    for body in [body(), replace(&body(), "client_id=public", "client_id=public&client_secret=anything")] {
        let request = issuer.token_request(None, TYPE, &body).expect("public body");
        issuer.redeem(&request, &authorization).expect("public redemption");
    }
}
#[test]
fn public_client_still_refuses_two_methods_and_unknown_ids() {
    let issuer = mixed();
    let header = token::basic(b"public:");
    for secret in ["anything", ""] {
        let body = replace(&body(), "client_id=public", &format!("client_id=public&client_secret={secret}"));
        assert_eq!(issuer.token_request(Some(&header), TYPE, &body).unwrap_err(), TokenError::InvalidRequest);
    }
    assert_eq!(
        issuer.token_request(None, TYPE, &replace(&body(), "public", "unknown")).unwrap_err(),
        TokenError::InvalidClient { header: false }
    );
    assert_eq!(
        issuer
            .token_request(Some(&token::basic(b"unknown:")), TYPE, &replace(&body(), "&client_id=public", ""))
            .unwrap_err(),
        TokenError::InvalidClient { header: true }
    );
}
#[test]
fn public_authorization_refuses_another_redirect() {
    assert_eq!(mixed().authorize(&replace(&query(), "%2Fcallback", "%2Fother")), Err(Refusal::UnknownRedirect));
}
#[test]
fn public_authorization_requires_a_challenge() {
    assert_eq!(
        mixed().authorize(&replace(&query(), "&code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM", "")),
        Err(back(b"invalid_request", None))
    );
}
#[test]
fn public_token_requires_a_verifier() {
    assert_eq!(
        mixed()
            .token_request(
                None,
                TYPE,
                &replace(&body(), "&code_verifier=dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk", "")
            )
            .unwrap_err(),
        TokenError::InvalidRequest
    );
}
#[test]
fn public_redemption_refuses_another_redirect() {
    let issuer = mixed();
    let request =
        issuer.token_request(None, TYPE, &replace(&body(), "%2Fcallback", "%2Fother")).expect("parsed request");
    assert_eq!(
        issuer.redeem(&request, &issuer.authorize(&query()).expect("authorization")),
        Err(TokenError::InvalidGrant)
    );
}
#[test]
fn public_redemption_refuses_another_verifier() {
    let issuer = mixed();
    let request = issuer.token_request(None, TYPE, &replace(&body(), "dBjft", "eBjft")).expect("parsed request");
    assert_eq!(
        issuer.redeem(&request, &issuer.authorize(&query()).expect("authorization")),
        Err(TokenError::InvalidGrant)
    );
}
#[test]
fn public_registration_does_not_weaken_confidential_clients_or_code_binding() {
    let issuer = mixed();
    for password in ["wrong", ""] {
        assert_eq!(
            issuer
                .token_request(None, TYPE, &replace(BODY, "client_secret=secret", &format!("client_secret={password}")))
                .unwrap_err(),
            TokenError::InvalidClient { header: false }
        );
        assert_eq!(
            issuer
                .token_request(
                    Some(&token::basic(format!("client:{password}").as_bytes())),
                    TYPE,
                    &replace(BODY, "&client_secret=secret", "")
                )
                .unwrap_err(),
            TokenError::InvalidClient { header: true }
        );
    }
    assert_eq!(
        issuer.token_request(None, TYPE, &replace(BODY, "&client_secret=secret", "")).unwrap_err(),
        TokenError::InvalidClient { header: false }
    );
    let confidential = issuer.token_request(None, TYPE, BODY).expect("confidential request");
    let public = issuer.token_request(None, TYPE, &body()).expect("public request");
    assert_eq!(
        issuer.redeem(&confidential, &issuer.authorize(&query()).expect("public authorization")),
        Err(TokenError::InvalidGrant)
    );
    assert_eq!(
        issuer.redeem(&public, &issuer.authorize(QUERY).expect("confidential authorization")),
        Err(TokenError::InvalidGrant)
    );
}
#[test]
fn discovery_adds_none_only_when_a_client_is_public() {
    let expected = replace(&issuer().discovery(), "\"client_secret_post\"]", "\"client_secret_post\",\"none\"]");
    assert_eq!(mixed().discovery(), expected);
}
