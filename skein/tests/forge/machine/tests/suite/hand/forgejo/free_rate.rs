//! Call envelopes gather response facts before the dialect's plan hears them.
use skein_forge::api::{Answer, Op, Read};
use skein_forge_connection::Answered;
use skein_forge_machine_world::hand::{self as f, FORGE, Harness, REPO};
use skein_lib::Duration;
use std::boxed::Box;

const CALL: u64 = 1;

const ONE_REQUEST: u32 = 1;

const FIRST_FIELDS: &[u8] =
    b"X-RateLimit-Limit: 5000\r\nX-RateLimit-Remaining: 4999\r\nX-RateLimit-Reset: 1800000000\r\n";

fn branch() -> Op {
    Op::Read(Read::Branch { branch: Box::from(f::BRANCH) })
}

fn begin(h: &mut Harness) {
    h.call(CALL, REPO, branch());
    h.run(Duration::ZERO);
    h.connected(FORGE);
}

fn answer(h: &mut Harness, status: u16, body: &[u8], fields: &[u8]) {
    h.answer(FORGE, &f::response_with_fields(status, body, fields));
    h.run(Duration::ZERO);
}

fn terminal(h: &Harness) -> &Answered {
    h.answered(CALL).expect("call terminal")
}

#[test]
fn forgejo_ignores_github_rate_headers() {
    let mut h = f::known();
    begin(&mut h);
    answer(&mut h, 200, f::BRANCH_BODY, FIRST_FIELDS);
    let got = terminal(&h);
    assert_eq!(
        (got.cost, got.free, got.rate, &got.result),
        (ONE_REQUEST, 0, None, &Ok(Answer::Commit(f::BRANCH_HEAD)))
    );
}
