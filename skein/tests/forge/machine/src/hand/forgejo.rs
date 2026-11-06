use super::{BRANCH_BODY, Harness, ok, response_with_fields};
use skein_lib::Duration;
use std::boxed::Box;

#[must_use]
pub fn last_page(body: &[u8]) -> Box<[u8]> {
    response_with_fields(200, body, b"X-HasMore: false\r\n")
}
pub const REPOSITORY_BODY: &[u8] = br#"{"id":1,"owner":{"id":2,"login":"fixture"},"name":"specimen","object_format_name":"sha1","default_branch":"main","default_merge_style":"merge","allow_merge_commits":true,"allow_squash_merge":false,"allow_rebase":false,"html_url":"http://127.0.0.1:3000/fixture/specimen"}"#;
pub fn branch_answers(h: &mut Harness, forge: u16) {
    h.answer(forge, &ok(REPOSITORY_BODY));
    h.answer(forge, &ok(BRANCH_BODY));
    h.run(Duration::ZERO);
}
