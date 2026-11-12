use super::{fixtures as f, wall};
use crate::{
    Service, delivery, fixture,
    image::Refusal,
    inline::Inline,
    installations::{Denied, Kind, Reach, Scope},
};
use skein_forge::github;
use skein_lib::{Duration, Secret};

const INSTALLATION: u64 = 4;

fn get(target: &[u8], authorization: &Secret) -> Box<[u8]> {
    f::parts(&[
        b"GET ",
        target,
        b" HTTP/1.1\r\nHost: github.test\r\nUser-Agent: test\r\nX-GitHub-Api-Version: ",
        github::pin::API_VERSION.as_bytes(),
        b"\r\nAuthorization: ",
        authorization.expose(),
        b"\r\n\r\n",
    ])
}
fn config() -> crate::Config {
    let mut config = f::config();
    config.deliveries.endpoint =
        Some(delivery::Endpoint { host: f::boxed(b"127.0.0.1:7201"), target: f::boxed(b"/hooks") });
    config.deliveries.log_min = Duration::ZERO;
    config.deliveries.log_max = Duration::ZERO;
    config
}
fn setup() -> f::Harness {
    let mut h = f::Harness::with(f::page_limits(), config());
    let answer = f::create(&mut h, b"converted", f::NAME);
    assert_eq!(f::convert(&mut h, &f::code(&answer)).status, 201);
    h.jwt = f::authorization_for(8, crate::keys::KEYS.first().expect("first key"), 1000, 1600);
    assert_eq!(
        h.service.installations_mut().install(fixture::OWNER, b"acme", Kind::Organization, Reach::All, wall(1200)),
        Ok(INSTALLATION)
    );
    h.service.poll(wall(1200));
    h
}
fn inline(h: f::Harness) -> Inline {
    Inline::new(h.service, h.domain, h.env.limits, h.limits)
}

#[test]
fn github_image_keeps_tokens_streams_and_log_but_drops_posts() {
    let mut source = setup();
    let (secret, expires) =
        source.service.installations_mut().mint(INSTALLATION, Scope::Full, wall(1200)).expect("mint");
    let pending = f::create(&mut source, b"pending", b"Next App");
    let code = f::code(&pending);
    let image = source.service.image();
    let mut restored = f::Harness::with(f::page_limits(), config());
    restored.service.restore(&image).expect("restore");
    assert!(restored.service.image() == image, "all retained fields");
    assert!(restored.service.next_post().is_none(), "queued post is lost");
    assert!(source.service.next_post().is_some(), "source had a queued installation delivery");
    let token = f::bearer(secret.expose());
    let app = source.jwt.clone();
    let mut source = inline(source);
    let mut restored = inline(restored);
    for (request, now) in [
        (get(f::REACHABLE, &token), wall(1200)),
        (get(b"/app", &app), wall(1200)),
        (get(b"/app/hook/deliveries?per_page=1", &app), wall(1200)),
    ] {
        let expected = source.serve(&request, now);
        let actual = restored.serve(&request, now);
        assert!(actual.starts_with(b"HTTP/1.1 200"), "route succeeded: {:?}", actual.get(..16));
        assert_eq!(actual, expected);
        if request.starts_with(b"GET /app/hook/deliveries") {
            assert!(skein_lib::bytes::find(&actual, b"installation").is_some(), "delivery log entry");
        }
    }
    let expected = source
        .service
        .installations_mut()
        .mint(INSTALLATION, Scope::Contents, wall(1200))
        .expect("next original token");
    let actual = restored
        .service
        .installations_mut()
        .mint(INSTALLATION, Scope::Contents, wall(1200))
        .expect("next restored token");
    assert!(expected.0.expose() == actual.0.expose(), "token RNG continues");
    assert_eq!(expected.1, actual.1);
    let before_expiry = skein_lib::Wall::from_nanos(expires.as_nanos().checked_sub(1).expect("before expiry"));
    assert!(restored.serve(&get(f::REACHABLE, &token), before_expiry).starts_with(b"HTTP/1.1 200"));
    assert!(restored.serve(&get(f::REACHABLE, &token), expires).starts_with(b"HTTP/1.1 401"));
    assert_eq!(restored.service.installations().token(secret.expose(), expires), Err(Denied::Expired));
    // Codes retain their original expiry too; restoring never gives them a new hour.
    let request = f::parts(&[
        b"POST ",
        &f::conversion_target(&code),
        b" HTTP/1.1\r\nHost: github.test\r\nUser-Agent: test\r\nX-GitHub-Api-Version: ",
        github::pin::API_VERSION.as_bytes(),
        b"\r\nContent-Length: 0\r\n\r\n",
    ]);
    assert!(restored.serve(&request, wall(4800)).starts_with(b"HTTP/1.1 404"));
}

#[test]
fn github_image_refuses_malformed_and_over_limit_without_replacing_state() {
    let source = setup();
    let image = source.service.image();
    let mut target = Service::new(config(), &f::page_limits(), 77).expect("target");
    let before = target.image();
    for bytes in [b"arbitrary".as_slice(), image.get(..image.len().checked_sub(1).expect("cut")).expect("prefix")] {
        let refusal = target.restore(bytes).expect_err("malformed image");
        match refusal {
            Refusal::Malformed(what) => assert!(!what.is_empty()),
            Refusal::Limits(what) => panic!("malformed image diagnosed as limit: {what}"),
        }
        assert!(target.image() == before, "refusal is atomic");
    }
    let mut limits = f::page_limits();
    limits.installations.tokens = 1;
    let mut source = setup();
    for _ in 0_u32..2 {
        source.service.installations_mut().mint(INSTALLATION, Scope::Full, wall(1200)).expect("two tokens");
    }
    let mut target = Service::new(config(), &limits, 77).expect("smaller target");
    let before = target.image();
    assert_eq!(target.restore(&source.service.image()), Err(Refusal::Limits("installation tokens")));
    assert!(target.image() == before);
    let limits = crate::Limits {
        pages: crate::pages::Limits { manifest_bytes: 1, ..limits.pages },
        installations: f::page_limits().installations,
        ..limits
    };
    let mut target = Service::new(config(), &limits, 77).expect("smaller manifest bound");
    let before = target.image();
    assert_eq!(target.restore(&source.service.image()), Err(Refusal::Limits("manifest bytes")));
    assert!(target.image() == before);
}

#[test]
fn github_image_keeps_payloads_and_future_delivery_and_page_draws() {
    use github::delivery::{Ci, Delivery};
    use skein_forge::text::Body;
    let mut source = setup();
    let pending = f::create(&mut source, b"next", b"Second App");
    let code = f::code(&pending);
    let mut ids = skein_lib::List::with_capacity(32);
    for payload in super::deliveries::values().into_boxed() {
        let (id, _) = source.service.release(payload, Some(INSTALLATION), wall(1200)).expect("delivery");
        source.service.delivered(id, 204);
        ids.push(id).expect("ids");
    }
    for payload in [
        Delivery::Issue { repository: 2, number: 3, body: Body::Kept(f::boxed(b"issue")) },
        Delivery::Comment { repository: 2, number: 3, body: Body::Kept(f::boxed(b"comment")) },
        Delivery::Review { repository: 2, number: 3, body: Body::Kept(f::boxed(b"review")) },
        Delivery::Remark { repository: 2, number: 3 },
        Delivery::Ref { repository: 2, branch: f::boxed(b"topic"), created: false },
        Delivery::Ci { repository: 2, commit: crate::reads::commit(1), event: Ci::WorkflowRun },
    ] {
        ids.push(source.service.release(payload, Some(INSTALLATION), wall(1200)).expect("delivery").0).expect("ids");
    }
    let mut restored = f::Harness::with(f::page_limits(), config());
    restored.service = Service::new(config(), &restored.limits, 77).expect("different launch seed");
    let image = source.service.image();
    restored.service.restore(&image).expect("restore");
    assert!(restored.service.image() == image, "launch seed does not change retained keys or streams");
    let jwt = source.jwt.clone();
    let mut source = inline(source);
    let mut restored = inline(restored);
    for id in &ids {
        let target = f::parts(&[b"/app/hook/deliveries/", skein_lib::Decimal::of(*id).as_bytes()]);
        let request = get(&target, &jwt);
        let expected = source.serve(&request, wall(1200));
        let actual = restored.serve(&request, wall(1200));
        assert!(actual.starts_with(b"HTTP/1.1 200"));
        assert!(actual == expected, "whole logged delivery, including payload and signature");
    }
    let (expected_id, expected) =
        source.service.redeliver(*ids.last().expect("delivery"), wall(1200)).expect("redelivery");
    let (actual_id, actual) =
        restored.service.redeliver(*ids.last().expect("delivery"), wall(1200)).expect("redelivery");
    let expected = expected.expect("endpoint");
    let actual = actual.expect("endpoint");
    assert_eq!(actual_id, expected_id);
    assert!(actual.head == expected.head && actual.body == expected.body, "delivery stream and counter continue");
    assert_eq!(source.service.rng.next_u64(), restored.service.rng.next_u64(), "service fault stream continues");
    let target = f::conversion_target(&code);
    let request = f::parts(&[
        b"POST ",
        &target,
        b" HTTP/1.1\r\nHost: github.test\r\nUser-Agent: test\r\nX-GitHub-Api-Version: ",
        github::pin::API_VERSION.as_bytes(),
        b"\r\nContent-Length: 0\r\n\r\n",
    ]);
    let expected = source.serve(&request, wall(1200));
    let actual = restored.serve(&request, wall(1200));
    assert!(actual.starts_with(b"HTTP/1.1 201"));
    assert!(actual == expected, "page stream, code and key rotation continue");
}
