use super::{
    BRANCH_BODY, FORGE, Harness, Limits, OTHER_FORGE, Shared, ok, response_with_fields, settings_ok, two_forges,
    user_ok,
};
use skein_lib::Duration;
use std::boxed::Box;

#[must_use]
pub fn last_page(body: &[u8]) -> Box<[u8]> {
    response_with_fields(200, body, b"X-HasMore: false\r\n")
}
pub const BRANCH: &[u8] = b"main";
pub const BRANCH_HEAD: [u8; 32] = [0; 32];
pub const REPOSITORY_BODY: &[u8] = br#"{"id":1,"owner":{"id":2,"login":"fixture"},"name":"specimen","object_format_name":"sha1","default_branch":"main","default_merge_style":"merge","allow_merge_commits":true,"allow_squash_merge":false,"allow_rebase":false,"html_url":"http://127.0.0.1:3000/fixture/specimen"}"#;
#[must_use]
pub fn ready_two(limits: Limits, shared: Shared) -> Harness {
    let mut h = Harness::with(limits, shared, two_forges());
    for forge in [FORGE, OTHER_FORGE] {
        h.connected(forge);
        h.answer(forge, &user_ok());
        h.answer(forge, &settings_ok());
        h.run(limits.idle);
    }
    h
}
pub fn branch_answers(h: &mut Harness, forge: u16) {
    h.answer(forge, &ok(REPOSITORY_BODY));
    h.answer(forge, &ok(BRANCH_BODY));
    h.run(Duration::ZERO);
}

// Eviction shares one connection and one call slot between two sequential requests.
pub const EVICTION_CONNECTIONS: u32 = 1; // A single connection forces reassignment.
pub const EVICTION_CALLS: u32 = EVICTION_CONNECTIONS;
pub const EVICTION_NO_CONNECTIONS: u32 = 0; // The evicted forge has no connection.
pub const EVICTION_ANSWERS: u32 = 2; // Each of the two sequential requests answers once.
