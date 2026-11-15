//! Runtime origin admission keeps Add's checks and stable indices (connection.md, 3.1).
use super::{StepFixture, fixtures};
use crate::{
    Request,
    seam::Access,
    settings::{Kind, LogOrigin},
};
use skein_forge::{api, github::redirect::Origin};
use skein_lib::{Lifecycle, List};

const LOG_PORT: u16 = 8080;
const FIRST: &[u8] = b"logs.example.test";
const SECOND: &[u8] = b"other.example.test";
const THIRD: &[u8] = b"third.example.test";

fn configured(host: &[u8]) -> LogOrigin {
    LogOrigin {
        origin: Origin { tls: false, host: Box::from(host), port: LOG_PORT },
        address: skein_io::Addr::Ip(([127, 0, 0, 1], LOG_PORT).into()),
    }
}
fn idle(kind: Kind, origins: &[LogOrigin], capacity: u32) -> StepFixture {
    let mut settings = fixtures::github_settings();
    let forge = settings.forges.get_mut(0).expect("forge");
    forge.kind = kind;
    forge.access = None;
    forge.log_origins = List::with_capacity(capacity);
    for origin in origins {
        forge.log_origins.push(origin.clone()).expect("fixture room");
    }
    StepFixture::new(settings)
}
fn held(h: &StepFixture) -> &List<LogOrigin> {
    &h.component.forges.get(0).expect("forge").base.forge.log_origins
}
fn append(h: &mut StepFixture, origin: LogOrigin) {
    h.send(Request::LogOrigin { forge: fixtures::FORGE, origin });
}

#[test]
fn small_origin_lists_grow_in_order_to_the_bound() {
    for capacity in [0, 1] {
        let mut h = idle(Kind::GitHub, &[], capacity);
        append(&mut h, configured(FIRST));
        append(&mut h, configured(SECOND));
        assert_eq!(held(&h).as_slice(), &[configured(FIRST), configured(SECOND)]);
        assert_eq!(held(&h).capacity(), h.env.limits.log_origins);
        assert!(h.up.is_empty());
        assert!(h.io.is_empty());
    }
}

#[test]
fn refused_runtime_origins_leave_the_list_and_events_unchanged() {
    let mut remote = configured(SECOND);
    remote.address = skein_io::Addr::Ip(([192, 0, 2, 1], LOG_PORT).into());
    let mut inprocess = configured(SECOND);
    inprocess.address = skein_io::Addr::InProcess(skein_io::Name::new(b"logs").expect("name"));
    let mut https = configured(SECOND);
    https.origin.tls = true;
    let mut duplicate = configured(b"LOGS.EXAMPLE.TEST");
    duplicate.address = skein_io::Addr::Ip(([127, 0, 0, 2], LOG_PORT).into());
    for (kind, forge, initial, origin) in [
        (Kind::GitHub, fixtures::FORGE, Box::<[LogOrigin]>::from([configured(FIRST)]), remote),
        (Kind::GitHub, fixtures::FORGE, Box::<[LogOrigin]>::from([configured(FIRST)]), inprocess),
        (Kind::GitHub, fixtures::FORGE, Box::<[LogOrigin]>::from([configured(FIRST)]), https),
        (Kind::GitHub, fixtures::FORGE, Box::<[LogOrigin]>::from([configured(FIRST)]), configured(b"bad/name")),
        (Kind::GitHub, fixtures::FORGE, Box::<[LogOrigin]>::from([configured(FIRST)]), configured(b"127.0.0.1")),
        (Kind::GitHub, fixtures::FORGE, Box::<[LogOrigin]>::from([configured(FIRST)]), duplicate),
        (
            Kind::GitHub,
            fixtures::FORGE,
            Box::<[LogOrigin]>::from([configured(FIRST), configured(SECOND)]),
            configured(THIRD),
        ),
        (Kind::Forgejo, fixtures::FORGE, Box::<[LogOrigin]>::from([]), configured(FIRST)),
        (Kind::GitHub, fixtures::OTHER_FORGE, Box::<[LogOrigin]>::from([configured(FIRST)]), configured(SECOND)),
    ] {
        let mut h = idle(kind, &initial, fixtures::limits().log_origins);
        h.send(Request::LogOrigin { forge, origin });
        assert_eq!(held(&h).as_slice(), initial.as_ref());
        assert!(h.up.is_empty());
        assert!(h.io.is_empty());
    }
}

#[test]
fn origin_request_head_holds_at_its_bound_only() {
    // Isolate the origin's head from the API head, which carries a credential.
    let head = fixtures::limits().target_bytes
        + u32::try_from(b"logs.example.test:8080".len()).expect("host length")
        + u32::try_from(fixtures::USER_AGENT.len()).expect("agent length")
        + 2 * (u32::try_from(api::VALIDATOR_BYTES).expect("validator length") + 22)
        + 192;
    for (cap, expected) in
        [(head, Box::<[LogOrigin]>::from([configured(FIRST)])), (head - 1, Box::<[LogOrigin]>::from([]))]
    {
        let mut h = idle(Kind::GitHub, &[], 0);
        h.component.common.shared.io.output = cap;
        append(&mut h, configured(FIRST));
        assert_eq!(held(&h).as_slice(), expected.as_ref());
        assert!(h.up.is_empty());
        assert!(h.io.is_empty());
    }
}

#[test]
fn ending_ignores_origins_without_touching_the_list() {
    for request in [Request::Close, Request::Abort] {
        let mut h = idle(Kind::GitHub, &[configured(FIRST)], 1);
        h.component.test_open_call(&h.env, &mut h.io);
        h.send(request);
        let pending = h.io.len();
        append(&mut h, configured(SECOND));
        assert_eq!(held(&h).as_slice(), &[configured(FIRST)]);
        assert!(h.up.is_empty());
        assert_eq!(h.io.len(), pending);
    }
    for lifecycle in [Lifecycle::Closing, Lifecycle::Aborting, Lifecycle::Closed] {
        let mut h = idle(Kind::GitHub, &[configured(FIRST)], 1);
        // Hold each ending phase before its last io completion.
        h.component.common.lifecycle = lifecycle;
        append(&mut h, configured(SECOND));
        assert_eq!(held(&h).as_slice(), &[configured(FIRST)]);
        assert!(h.up.is_empty());
        assert!(h.io.is_empty());
    }
}
