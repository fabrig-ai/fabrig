//! Job-log origin settings are checked before startup.
use super::fixtures::{self as f, FORGE};
use crate::{Unusable, settings::LogOrigin};
use alloc::boxed::Box;
use skein_forge::github::redirect::Origin;
use skein_lib::List;

const LOG_PORT: u16 = 8080;
const LOG_HOST: &[u8] = b"logs.example.test";

fn origin(host: &[u8]) -> Origin {
    Origin { tls: false, host: Box::from(host), port: LOG_PORT }
}

fn configured() -> LogOrigin {
    LogOrigin { origin: origin(LOG_HOST), address: skein_io::Addr::Ip(([127, 0, 0, 1], LOG_PORT).into()) }
}

fn settings() -> crate::Settings {
    let mut settings = f::github_settings();
    let forge = settings.forges.get_mut(0).expect("forge");
    forge.access = None;
    forge.log_origins = List::with_capacity(f::limits().log_origins);
    forge.log_origins.push(configured()).expect("origin bound");
    settings
}

#[test]
fn start_refusals_name_each_origin_value() {
    let mut cases = List::with_capacity(8);
    let mut ip = configured();
    ip.origin.host = Box::from(*b"127.0.0.1");
    cases.push((crate::settings::Kind::GitHub, ip)).expect("row");
    let mut invalid = configured();
    invalid.origin.host = Box::from(*b"bad/name");
    cases.push((crate::settings::Kind::GitHub, invalid)).expect("row");
    let mut inprocess = configured();
    inprocess.address = skein_io::Addr::InProcess(skein_io::Name::new(b"logs").expect("name"));
    cases.push((crate::settings::Kind::GitHub, inprocess)).expect("row");
    let mut remote = configured();
    remote.address = skein_io::Addr::Ip(([192, 0, 2, 1], LOG_PORT).into());
    cases.push((crate::settings::Kind::GitHub, remote)).expect("row");
    let mut https = configured();
    https.origin.tls = true;
    cases.push((crate::settings::Kind::GitHub, https)).expect("row");
    cases.push((crate::settings::Kind::Forgejo, configured())).expect("row");
    for (kind, configured) in cases.into_boxed() {
        let expected = Unusable::Origin { forge: FORGE, origin: configured.origin.clone() };
        let mut settings = settings();
        let forge = settings.forges.get_mut(0).expect("forge");
        forge.kind = kind;
        forge.log_origins.clear();
        forge.log_origins.push(configured).expect("origin");
        assert_eq!(crate::limits::check(&f::limits(), &f::shared(), &settings), Err(expected));
    }
}

#[test]
fn start_names_the_origin_count_past_its_bound() {
    let mut settings = settings();
    let forge = settings.forges.get_mut(0).expect("forge");
    let limit = f::limits().log_origins;
    forge.log_origins = List::with_capacity(limit.checked_add(1).expect("fixture bound"));
    for _ in 0..=limit {
        forge.log_origins.push(configured()).expect("rows");
    }
    assert_eq!(
        crate::limits::check(&f::limits(), &f::shared(), &settings),
        Err(Unusable::LogOrigins { forge: FORGE, origins: limit.checked_add(1).expect("fixture bound"), limit })
    );
}

#[test]
fn tls_forge_accepts_https_and_loopback_http_at_origin_bound() {
    let mut settings = settings();
    let forge = settings.forges.get_mut(0).expect("forge");
    forge.transport = crate::settings::Transport::Tls {
        server_name: skein_tls::Name::new("api.example.test").expect("name"),
        trust: f::trust(),
    };
    let mut https = configured();
    https.origin.tls = true;
    https.address = skein_io::Addr::Ip(([192, 0, 2, 1], 443).into());
    forge.log_origins.push(https).expect("at bound");
    assert_eq!(crate::limits::check(&f::limits(), &f::shared(), &settings), Ok(()));
}

#[test]
fn zero_origin_bound_is_refused() {
    assert_eq!(
        crate::limits::check(&crate::Limits { log_origins: 0, ..f::limits() }, &f::shared(), &settings()),
        Err(Unusable::Zero)
    );
}
