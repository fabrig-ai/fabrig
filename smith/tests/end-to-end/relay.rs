//! The shipped binary's relay dispatch (smith/docs/design/protocol/programs.md, section 5.3).
use smith_real_world::Scratch;
#[test]
fn the_agent_binary_refuses_a_relay_without_its_secret() {
    let scratch = Scratch::new();
    let stderr = crate::support::refusal_program(
        &skein_scratch::paths::binary("smith-agent"),
        &scratch,
        vec!["--mcp-relay".into(), "127.0.0.1:9".into()],
    );
    assert!(stderr.starts_with(b"smith-agent: usage: smith-agent --mcp-relay"));
    assert_eq!(skein_lib::bytes::count(&stderr, b"\n", 2), 1);
}
