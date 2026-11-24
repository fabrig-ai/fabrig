//! The deployment's client (temper/docs/design/protocol/binaries.md).
use crate::support;
use skein_io::kernel::Exit;
use support::ctl::{Call, Step, Story, json};

/// Compiled with the process tier; run after landing (temper/docs/design/protocol/checkpoints.md, 5).
#[test]
fn ctl_from_the_deployments_client_signs_in_reads_me_and_takes_a_checkpoint_while_the_deployment_serves() {
    let scratch = support::Scratch::new("client_checkpoint");
    let engine = support::cluster::Cluster::engine_alone(scratch.path());
    let client = skein_scratch::paths::binary("temper-box").with_file_name("temper");
    assert!(client.is_file(), "build the client's sibling binary: cargo build -p temper --bin temper");
    let seen = support::ctl::run_on_engine(Story::new(
        &engine,
        vec![
            Step::Run(Call::sign_in().by(&client)),
            Step::Run(Call::new(&["me", "--json"]).by(&client)),
            Step::Run(Call::new(&["deployment", "checkpoint", "--timeout", "2m"]).by(&client)),
        ],
    ));
    let [sign_in, me, checkpoint] = seen.calls.as_slice() else { panic!("sign-in, me and checkpoint end") };
    for call in [sign_in, me, checkpoint] {
        assert_eq!(call.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&call.stderr));
    }
    assert!(json(me)["party"]["number"].is_u64(), "me names the signed-in party's number");
    let line = std::str::from_utf8(&checkpoint.stdout).expect("checkpoint line");
    let name = line.strip_prefix("checkpoint ").and_then(|line| line.split_once(" at commit ")).expect("named cut").0;
    assert!(jig_protocol_store::CheckpointName::parse(name.as_bytes()).is_some(), "canonical checkpoint name");
    assert!(engine.join("data/state/checkpoints").join(name).is_dir(), "the printed checkpoint stands: {line}");
    scratch.remove();
}
