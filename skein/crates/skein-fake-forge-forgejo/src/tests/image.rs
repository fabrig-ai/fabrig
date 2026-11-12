use super::fixtures::{self as f, boxed};
use crate::{Service, delivery::Endpoint, image::Refusal};
use skein_forge::forgejo::{
    document::Document,
    request::{Grant, Request},
};
use skein_lib::Secret;

#[test]
fn setup_image_restores_authentication_counters_and_hooks() {
    let mut source = f::setup_harness();
    let token = f::make_setup_token(&mut source).value.expect("issued token");
    for request in [
        Request::CreateOrg { person: boxed(b"kit"), name: boxed(b"work") },
        Request::CreateTeam { org: boxed(b"work"), spec: f::team_spec() },
        Request::AddMember { team: f::TEAM, login: boxed(b"kit") },
        Request::CreateOrgHook { org: boxed(b"work"), hook: f::hook_spec() },
        Request::AddCollaborator { at: f::at(), login: boxed(b"kit"), permission: Grant::Write },
        Request::CreateRepoHook { at: f::at(), hook: f::hook_spec() },
    ] {
        source.send(&request);
        assert!(source.status() < 300);
    }
    source.service.setup_mut().private(2).expect("privacy");
    source
        .service
        .set_hooks(Some(Endpoint {
            forge: 7,
            host: boxed(b"127.0.0.1:7201"),
            secret: Secret::new(boxed(b"owner hook secret")),
        }))
        .expect("endpoint");
    let image = source.service.image();
    let mut restored = f::Harness::new(f::config());
    restored.service.restore(&image).expect("restore");
    assert!(restored.service.image() == image, "all retained fields");
    restored.ask_as(b"GET", b"/api/v1/user", b"", token.expose());
    assert_eq!(restored.status(), 200);
    let Document::User(user) = restored.decoded(&Request::User).document else {
        panic!("user");
    };
    assert_eq!((user.id, user.login.as_ref()), (f::PERSON, b"kit".as_slice()));
    let expected = source.service.setup_mut().account(b"next", false, b"");
    assert_eq!(restored.service.setup_mut().account(b"next", false, b""), expected);
    for request in [
        Request::Teams { org: boxed(b"work"), page: f::PAGE },
        Request::TeamMembers { team: f::TEAM, page: f::PAGE },
        Request::OrgHooks { org: boxed(b"work"), page: f::PAGE },
        Request::RepoHooks { at: f::at(), page: f::PAGE },
    ] {
        source.send(&request);
        restored.send(&request);
        assert_eq!(restored.status(), 200);
        assert_eq!(restored.decoded(&request).rows, source.decoded(&request).rows);
        assert_eq!(restored.decoded(&request).rows.len(), 1);
    }
    let hook = restored.service.setup().repository_hooks(2).next().expect("repository hook");
    assert_eq!(hook.secret.expose(), f::hook_spec().secret.expose());
    assert_eq!(restored.service.setup().collaborator(2, b"kit"), Some(Grant::Write));
    assert!(restored.service.setup().is_private(2));
    let endpoint = restored.service.config.hooks.as_ref().expect("owner endpoint");
    assert_eq!(
        (endpoint.forge, endpoint.host.as_ref(), endpoint.secret.expose()),
        (7, b"127.0.0.1:7201".as_slice(), b"owner hook secret".as_slice())
    );
}

#[test]
fn setup_image_refuses_malformed_and_over_limit_without_replacing_state() {
    let mut target = f::setup_harness();
    let before = target.service.image();
    let mut source = f::setup_harness();
    source.service.setup_mut().account(b"not in target", false, b"");
    let image = source.service.image();
    for bytes in [b"arbitrary".as_slice(), image.get(..image.len().checked_sub(1).expect("cut")).expect("prefix")] {
        let refusal = target.service.restore(bytes).expect_err("malformed image");
        match refusal {
            Refusal::Malformed(what) => assert!(!what.is_empty()),
            Refusal::Limits(what) => panic!("malformed image diagnosed as limit: {what}"),
        }
        assert!(target.service.image() == before, "refusal is atomic");
    }
    let mut limits = target.env.limits;
    limits.documents.inner = limits.documents.inner.checked_add(1).expect("one extra row");
    let mut large = Service::new(f::config(), &limits, 9).expect("larger source");
    for _ in 0..limits.documents.inner {
        large.setup_mut().account(b"account", false, b"");
    }
    assert_eq!(target.service.restore(&large.image()), Err(Refusal::Limits("accounts")));
    assert!(target.service.image() == before);
}
