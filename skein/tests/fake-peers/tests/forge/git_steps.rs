use super::fixtures as f;
use skein_fake_forge_http::git;
use skein_fake_peers::{Error, forge::Peer};
use std::rc::Rc;

#[test]
fn peer_git_tree_room_holds_at_bound_and_refuses_one_less() {
    let shared = f::shared();
    let bound = git::tree_bound(&shared.borrow().config.limits).expect("tree bound");
    for (label, writer, request, accepted) in [
        ("answer bound", bound.answer, u64::from(bound.request), true),
        ("answer short", bound.answer.saturating_sub(1), u64::from(bound.request), false),
        ("request short", bound.answer, u64::from(bound.request.saturating_sub(1)), false),
    ] {
        let mut limits = f::protocol_limits();
        limits.documents.writer.length = writer;
        limits.http.body = request;
        let result = Peer::new(f::address(), f::limits(), f::config(), limits, Rc::clone(&shared), f::SEED);
        assert_eq!(result.err(), if accepted { None } else { Some(Error::Limits) }, "{label}");
    }
}
