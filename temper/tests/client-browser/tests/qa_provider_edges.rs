//! Adversarial process/browser checks of protocol/sign-in.md 9.2 and 9.3.
//! Uses only the public browser tier; every endpoint is scratch loopback.
use skein_fake_person::{Find, Step as Person};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_http::Method;
use temper_client_browser_world::{Tier, WITHIN};

fn request(method: Method, path: &str, status: u16) -> Step {
    Step::Request {
        method,
        path: path.as_bytes().into(),
        host: Box::from(&b"localhost"[..]),
        headers: vec![],
        body: Box::new([]),
        status,
        within: WITHIN,
    }
}

#[test]
fn absent_provider_does_not_publish_endpoints() {
    // 9.2: all four endpoints are absent when provider is null.
    let tier = Tier::new("qa-provider-absent");
    let mut script = Script::new(Box::new([
        request(Method::Get, "/.well-known/openid-configuration", 404),
        request(Method::Get, "/oauth/keys", 404),
        request(Method::Post, "/oauth/token", 404),
        request(Method::Get, "/oauth/userinfo", 404),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

#[test]
fn provider_routes_keep_their_methods_after_a_refusal() {
    // 9.2: invalid methods must not fall through to the SPA or damage the issuer.
    let (tier, origin) = Tier::authorize("qa-provider-methods");
    tier.authorize_callback(&format!("{origin}/callback"));
    let mut script = Script::new(Box::new([
        request(Method::Post, "/.well-known/openid-configuration", 405),
        request(Method::Get, "/.well-known/openid-configuration", 200),
        request(Method::Post, "/oauth/keys", 405),
        request(Method::Get, "/oauth/keys", 200),
        request(Method::Get, "/oauth/token", 405),
        request(Method::Post, "/oauth/userinfo", 405),
        request(Method::Get, "/.well-known/openid-configuration", 200),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

#[test]
fn provider_documents_remain_routable_with_queries() {
    // 9.2: a query does not turn a public document into the bundle's page.
    let (tier, origin) = Tier::authorize("qa-provider-document-query");
    tier.authorize_callback(&format!("{origin}/callback"));
    let mut script = Script::new(Box::new([
        request(Method::Get, "/.well-known/openid-configuration?cache=one", 200),
        request(Method::Get, "/oauth/keys?cache=two", 200),
        request(Method::Post, "/oauth/keys?cache=three", 405),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

#[test]
fn unknown_client_never_leaves_the_authorization_page() {
    // 9.3: a hostile client cannot send the browser to its redirect.
    let (tier, origin) = Tier::authorize("qa-provider-unknown-client");
    tier.authorize_callback(&format!("{origin}/callback"));
    let address = b"/oauth/authorize?response_type=code&client_id=unknown&redirect_uri=http%3A%2F%2F127.0.0.1%3A9%2Fescape&scope=openid&state=qa&code_challenge=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&code_challenge_method=S256";
    let mut script = Script::new(Box::new([
        Step::Person(Person::See { find: Find::exactly_named(skein_ui::Role::Button, b"Sign out"), within: WITHIN }),
        Step::Person(Person::Go { address: address.as_slice().into() }),
        Step::Person(Person::See { find: Find::text("This sign-in can’t continue".as_bytes()), within: WITHIN }),
        Step::Person(Person::See { find: Find::text(b"Nothing was sent anywhere."), within: WITHIN }),
        Step::Address(address.as_slice().into()),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

#[test]
fn unknown_redirect_never_leaves_the_authorization_page() {
    // 9.3: a known client still cannot choose an unregistered callback.
    let (tier, origin) = Tier::authorize("qa-provider-unknown-redirect");
    tier.authorize_callback(&format!("{origin}/callback"));
    let address = b"/oauth/authorize?response_type=code&client_id=forgejo&redirect_uri=http%3A%2F%2F127.0.0.1%3A9%2Fescape&scope=openid&state=qa&code_challenge=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&code_challenge_method=S256";
    let mut script = Script::new(Box::new([
        Step::Person(Person::See { find: Find::exactly_named(skein_ui::Role::Button, b"Sign out"), within: WITHIN }),
        Step::Person(Person::Go { address: address.as_slice().into() }),
        Step::Person(Person::See { find: Find::text("This sign-in can’t continue".as_bytes()), within: WITHIN }),
        Step::Person(Person::See { find: Find::text(b"Nothing was sent anywhere."), within: WITHIN }),
        Step::Address(address.as_slice().into()),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}

#[test]
fn authorization_without_provider_is_a_refusal_page() {
    // 9.2, 9.3: authorize remains a client place with no provider.
    let tier = Tier::new("qa-provider-no-provider-page");
    let address = b"/oauth/authorize?client_id=forgejo&response_type=code";
    let mut script = Script::new(Box::new([
        Step::Person(Person::See { find: Find::exactly_named(skein_ui::Role::Button, b"Sign out"), within: WITHIN }),
        Step::Person(Person::Go { address: address.as_slice().into() }),
        Step::Person(Person::See { find: Find::text("This sign-in can’t continue".as_bytes()), within: WITHIN }),
        Step::Address(address.as_slice().into()),
    ]));
    assert_eq!(Harness::new(tier.setup()).run(&mut [&mut script]), Outcome::Done);
}
