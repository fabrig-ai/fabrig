//! Job logs follow only a configured origin, without carrying the forge's access.
use skein_forge::{
    api::{self, Answer, Error, Op, Read},
    github::redirect::Origin,
};
use skein_forge_connection::seam::Access;
use skein_forge_connection::{Event, Fact, settings::LogOrigin};
use skein_forge_machine_world::hand::{self as f, FORGE, Harness, REPO};
use skein_lib::{Duration, List};
use std::boxed::Box;

const LOG_ENDPOINT: u16 = 2;
const LOG_PORT: u16 = 8080;
const LOG_HOST: &[u8] = b"logs.example.test";
// The fixture's log retention, independent of the component's bound.
const TAIL_BYTES: u32 = 4;
const METADATA: &[u8] =
    br#"{"id":11,"run_id":22,"run_attempt":2,"head_sha":"0000000000000000000000000000000000000000","steps":[]}"#;
fn attempt() -> api::JobAttempt {
    api::JobAttempt { head: [0; 32], run: 22, job: 11, attempt: 2 }
}
fn job() -> Op {
    Op::Read(Read::Job { part: api::LogPart::End, attempt: attempt(), max_bytes: TAIL_BYTES })
}
fn origin(host: &[u8]) -> Origin {
    Origin { tls: false, host: Box::from(host), port: LOG_PORT }
}
fn configured() -> LogOrigin {
    LogOrigin { origin: origin(LOG_HOST), address: skein_io::Addr::Ip(([127, 0, 0, 1], LOG_PORT).into()) }
}
fn settings() -> skein_forge_connection::Settings {
    let mut settings = f::github_settings();
    let forge = settings.forges.get_mut(0).expect("forge");
    forge.access = None;
    forge.log_origins = List::with_capacity(f::limits().log_origins);
    forge.log_origins.push(configured()).expect("origin bound");
    settings
}
fn harness(settings: skein_forge_connection::Settings) -> Harness {
    let mut h = Harness::new(settings);
    let api = *h.endpoints.get(0).expect("API endpoint");
    h.endpoints = List::with_capacity(2);
    h.endpoints.push(api).expect("API endpoint");
    h.endpoints.push((configured().address, LOG_ENDPOINT)).expect("log endpoint");
    h.component.test_calls_ready(f::app_access(skein_fake_forge_github::keys::KEYS.first().expect("test key"), 7, 9));
    f::github_account(&mut h, 1, 22, b"acme", b"artificial-full-one");
    f::github_repository(&mut h, REPO, 22);
    h
}
fn redirect(host: &[u8]) -> Box<[u8]> {
    let mut fields = skein_lib::Writer::new(
        b"Location: http://"
            .len()
            .checked_add(host.len())
            .expect("fixture host")
            .checked_add(b":8080/job.txt?signature=artificial\r\n".len())
            .expect("fixture fields"),
    );
    fields.put(b"Location: http://").expect("fixture head");
    fields.put(host).expect("fixture head");
    fields.put(b":8080/job.txt?signature=artificial\r\n").expect("fixture head");
    f::response_with_fields(302, b"", &fields.finish())
}
fn log_read(h: &mut Harness, call: u64) {
    h.call(call, REPO, job());
    h.run(Duration::ZERO);
    if h.connections_for(FORGE) == 1 && h.request_lines(FORGE).is_empty() {
        h.connected(FORGE);
    }
    h.answer(FORGE, &f::ok(METADATA));
    h.run(Duration::ZERO);
}
fn follow(h: &mut Harness) {
    log_read(h, 1);
    h.answer(FORGE, &redirect(b"LOGS.EXAMPLE.TEST"));
    h.run(Duration::ZERO);
    h.connected(LOG_ENDPOINT);
}
fn followed() -> Harness {
    let mut h = harness(settings());
    follow(&mut h);
    h
}
#[expect(clippy::needless_pass_by_value, reason = "the scripted scenario consumes its expected answer")]
fn terminal(h: &mut Harness, call: u64, cost: u32, result: Result<Answer, Error>) {
    h.run(Duration::ZERO);
    let answer = h.answered(call).expect("one terminal");
    assert_eq!((answer.cost, &answer.result), (cost, &result));
}
fn origins(h: &Harness) -> List<Origin> {
    let mut said = List::with_capacity(32);
    for event in h.facts.as_ref().expect("facts") {
        if let Event::Fact { fact: Fact::Origin { origin }, .. } = event {
            said.push(origin.clone()).expect("fixture facts");
        }
    }
    said
}
#[test]
fn configured_origin_matches_case_and_keeps_only_tail() {
    let mut h = followed();
    assert!(f::contains(&h.request_bytes(LOG_ENDPOINT), b"GET /job.txt?signature=artificial HTTP/1.1\r\n"));
    assert!(f::contains(&h.request_bytes(LOG_ENDPOINT), b"Host: logs.example.test:8080\r\n"));
    h.answer(LOG_ENDPOINT, &f::ok(b"0123456789"));
    terminal(
        &mut h,
        1,
        3,
        Ok(Answer::Job { attempt: attempt(), step: None, log: Box::from(*b"6789"), truncated: true }),
    );
    assert!(origins(&h).is_empty());
}
#[test]
fn followed_bytes_carry_no_authorization_range_or_forge_secret() {
    let mut h = harness(settings());
    // All values are public fixture material: no deployed credential is read.
    let key = skein_fake_forge_github::keys::KEYS.first().expect("public test key");
    let jwt = h
        .component
        .test_github_bearer(
            7,
            &skein_lib::Secret::new(Box::from(*key)),
            skein_lib::Wall::EPOCH,
            Duration::from_secs(10),
        )
        .expect("fixture JWT")
        .clone();
    follow(&mut h);
    // Calibration: the API's own requests carry the installation token.
    let api = h.request_bytes(FORGE);
    assert!(f::contains(&api, b"Authorization: "));
    assert!(f::contains(&api, b"artificial-full-one"));
    let bytes = h.request_bytes(LOG_ENDPOINT);
    for secret in [b"Authorization:".as_slice(), b"Range:", b"artificial-full-one", key, jwt.expose()] {
        assert!(!f::contains(&bytes, secret));
    }
    assert!(f::contains(&bytes, b"User-Agent: "));
    assert!(f::contains(&bytes, b"Accept: "));
}
#[test]
fn api_and_origin_connections_return_to_separate_pools() {
    let mut h = followed();
    h.answer(LOG_ENDPOINT, &f::ok(b"log"));
    h.run(Duration::ZERO);
    let opened = h.connections_opened();
    log_read(&mut h, 2);
    h.answer(FORGE, &redirect(LOG_HOST));
    h.run(Duration::ZERO);
    h.answer(LOG_ENDPOINT, &f::ok(b"log"));
    terminal(
        &mut h,
        2,
        3,
        Ok(Answer::Job { attempt: attempt(), step: None, log: Box::from(*b"log"), truncated: false }),
    );
    assert_eq!(h.connections_opened(), opened);
    assert_eq!(h.connections_for(FORGE), 1);
    assert_eq!(h.connections_for(LOG_ENDPOINT), 1);
}
#[test]
fn said_origins_forget_the_oldest_first() {
    let mut h = harness(settings());
    for (call, host) in
        [(1, b"one.test".as_slice()), (2, b"two.test"), (3, b"three.test"), (4, b"two.test"), (5, b"one.test")]
    {
        log_read(&mut h, call);
        h.answer(FORGE, &redirect(host));
        terminal(&mut h, call, 2, Err(Error::Refused));
    }
    assert_eq!(
        origins(&h).as_slice(),
        &[origin(b"one.test"), origin(b"two.test"), origin(b"three.test"), origin(b"one.test")]
    );
}
#[test]
fn original_deadline_covers_follow_before_and_after_its_write() {
    for (connected, cost) in [(false, 2), (true, 3)] {
        let mut h = harness(settings());
        log_read(&mut h, 1);
        h.run(Duration::from_secs(1));
        h.answer(FORGE, &redirect(LOG_HOST));
        h.run(Duration::ZERO);
        if connected {
            h.connected(LOG_ENDPOINT);
        }
        let deadline = h.env.limits.deadline;
        h.run(Duration::from_nanos(deadline.as_nanos().saturating_sub(Duration::from_secs(1).as_nanos())));
        terminal(&mut h, 1, cost, Err(Error::Timeout));
    }
}
#[test]
fn an_empty_origin_list_reads_no_job_log() {
    let mut settings = settings();
    settings.forges.get_mut(0).expect("forge").log_origins.clear();
    let mut h = harness(settings);
    h.call(1, REPO, job());
    terminal(&mut h, 1, 0, Err(Error::Refused));
    assert_eq!(h.connections_opened(), 0);
}

#[test]
fn unconfigured_origin_fails_before_its_api_body_arrives() {
    let mut h = harness(settings());
    log_read(&mut h, 1);
    h.answer(FORGE, b"HTTP/1.1 302 Found\r\nLocation: http://other.test:8080/job\r\nContent-Length: 1000\r\n\r\n");
    terminal(&mut h, 1, 2, Err(Error::Refused));
    assert_eq!(origins(&h).as_slice(), &[origin(b"other.test")]);
    let deadline = h.env.limits.deadline;
    h.run(deadline);
    assert_eq!(h.connections_for(FORGE), 0);
}

#[test]
fn said_origin_keeps_the_dialects_complete_bounded_host() {
    // The landed redirect parser bounds host by Location, not DNS's 253 bytes.
    const HOST_BYTES: usize = 254;
    let host = [b'a'; HOST_BYTES];
    let mut h = harness(settings());
    log_read(&mut h, 1);
    h.answer(FORGE, &redirect(&host));
    terminal(&mut h, 1, 2, Err(Error::Refused));
    assert_eq!(origins(&h).as_slice(), &[origin(&host)]);
}
