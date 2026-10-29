//! The shipped service and agent, killed and restarted over one durable store.
mod support;
use jig_client_protocol_world::native::Script;

/// The fake provider checks the credential presented by the real run.
fn peer() -> (skein_fake_peers::llm::Peer, std::net::SocketAddr, skein_io::kernel::Fd) {
    let mut script = jig_channel_world::fixtures::one_chat(b"Hello");
    script.result = br#"{"form":"report","text":"Process answer","fields":{}}"#.as_slice().into();
    support::ports::provider(
        Box::new([skein_fake_llm_domain::api::Script {
            cue: Box::default(),
            turns: jig_channel_world::fixtures::provider_turns(&script),
        }]),
        skein_llm::Credential::bearer(b"token".as_slice().into()),
    )
}

#[test]
fn a_killed_jig_test_finds_its_work_after_restart() {
    let directory = support::scratch();
    support::configuration(directory.path());
    assert_eq!(support::run(directory.path(), Script::answered_chat(b"Hello", support::ANSWER), Some(peer())), (1, 1));
    assert_eq!(support::run(directory.path(), Script::found_chat(support::ANSWER), None), (1, 0));
}

#[test]
fn an_agent_trace_captures_content_without_the_grant_or_authorization_header() {
    let directory = support::scratch();
    support::configuration(directory.path());
    support::trace::configure(directory.path());
    assert_eq!(support::run(directory.path(), Script::answered_chat(b"Hello", support::ANSWER), Some(peer())), (1, 1));
    support::trace::check(directory.path(), Some(b"token"));
}

#[path = "support/browser.rs"]
mod browser;
#[test]
fn jig_test_serves_its_bundle_s_page_under_the_reference_policy() {
    let directory = support::scratch();
    browser::configuration(directory.path(), &browser::fixture_bundle(directory.path()));
    let mut binary = browser::start(directory.path());
    let link = browser::printed_link_line(&mut binary);
    let (status, headers, body) = browser::get(&link, "/tasks/7");
    assert_eq!(status, 200);
    assert_eq!(browser::field(&headers, "content-security-policy"), Some(browser::POLICY));
    assert_eq!(body, browser::PAGE);
}

#[test]
fn a_binary_told_to_stop_with_a_sign_in_under_way_ends_at_once() {
    accounts::sigterm();
}

// Reuse the account sign-in world's browser source; it remains a private test helper.
use jig_protocol_accounts_world::fixtures;
#[expect(dead_code, reason = "only the accounts world stops a browser at its redirect, to paste it")]
#[path = "../../../tests/protocol-accounts/src/visitor.rs"]
mod account_sign_in_visitor;

#[path = "support/accounts.rs"]
mod accounts;
