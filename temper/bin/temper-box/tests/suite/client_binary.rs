//! The deployment's client (temper/docs/design/protocol/binaries.md).
use crate::support;
use skein_io::kernel::Exit;
use support::ctl::{Call, Step, Story, json};

#[test]
fn ctl_from_the_deployments_client_signs_in_to_an_engine_and_reads_me() {
    let scratch = support::Scratch::new("client_binary_ctl");
    let engine = support::cluster::Cluster::engine_alone(scratch.path());
    let client = skein_scratch::paths::binary("temper-box").with_file_name("temper-client");
    assert!(client.is_file(), "build the client's sibling binary: cargo build -p temper-client --bin temper-client");
    let seen = support::ctl::run_on_engine(Story::new(
        &engine,
        vec![Step::Run(Call::sign_in().by(&client)), Step::Run(Call::new(&["me", "--json"]).by(&client))],
    ));
    let [sign_in, me] = seen.calls.as_slice() else { panic!("sign-in and me end") };
    for call in [sign_in, me] {
        assert_eq!(call.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&call.stderr));
    }
    assert!(json(me)["party"]["number"].is_u64(), "me names the signed-in party's number");
    scratch.remove();
}
