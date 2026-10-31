//! Optional-workspace admission and host effects belong to the domain world.
//! Contract: smith/docs/design/domain/run.md, sections 3 and 5.2.
use skein_fake_checkout::Checkout;
use skein_fake_llm_domain::api::{Line, Script};
use skein_lib::Duration;
use smith_agent_world::{Job, Settings, World};
use smith_domain::{Transcript, run};

fn scripts() -> Box<[Script]> {
    Box::new([Script {
        cue: b"@hosttools".as_slice().into(),
        turns: [
            Line::Call {
                name: b"host_action".as_slice().into(),
                arguments: br#"{"task":"one durable effect"}"#.as_slice().into(),
            },
            Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
            Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
        ]
        .into_iter()
        .map(|line| smith_agent_world::fixture::calls(Box::new([line]), 1))
        .collect(),
    }])
}

fn no_workspace(world: &World) {
    assert!(world.boundaries().is_empty(), "no discovery, workspace IO or checks");
    assert_eq!(format!("{:?}", world.disk()), format!("{:?}", Checkout::new()));
    assert!(world.checked().is_empty() && world.pushes().is_empty() && world.delivery_submissions().is_empty());
    assert!(matches!(world.answer(), run::Answer::Parked { .. }));
}

#[test]
fn a_host_call_without_a_workspace_parks_and_resumes_without_a_second_effect() {
    let mut settings = Settings { job: Job::HostTools, waiting: Duration::from_secs(1), ..Settings::calm(1001) };
    let mut first = World::with_workspace_scripts(settings, None, None, Checkout::new(), scripts());
    first.run(100_000);
    no_workspace(&first);
    let [submission] = first.host_submissions() else { panic!("one host effect") };
    assert_eq!(submission.name, run::CallName { activation: 1, completion: 1, position: 0 });
    assert_eq!(submission.input.bytes(), br#"{"task":"one durable effect"}"#);
    assert_eq!(first.host_decisions(), 1);
    let turn = &first.turns()[0];
    let saved = Transcript {
        version: turn.version,
        endpoint: turn.endpoint,
        dialect: turn.dialect,
        turns: first.turns().into(),
    };
    settings.resume = true;
    let mut resumed = World::with_workspace_scripts(settings, Some(saved), None, Checkout::new(), scripts());
    resumed.run(100_000);
    no_workspace(&resumed);
    assert!(resumed.host_submissions().is_empty() && resumed.host_terminals().is_empty());
    assert_eq!(resumed.host_decisions(), 0);
}
