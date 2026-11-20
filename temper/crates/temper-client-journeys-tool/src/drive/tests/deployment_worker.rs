//! Worker admission through the deployment's signed-in ctl (deployment.md, 5 and 9).
use super::*;

fn worker() -> crate::drive::deployment::Worker {
    crate::drive::deployment::Worker::read(b"engine = \"127.0.0.1:4102\"\nname = \"worker\"\n", &[b'a'; 64])
        .expect("worker")
}
fn listed(bytes: &[u8]) -> Event {
    Event::Exited { process: Process::Ctl, code: 0, stdout: bytes.into(), stderr: Box::new([]) }
}
fn listing() -> Machine {
    let mut machine = at_engine();
    engine_ready(&mut machine);
    step(&mut machine, exited(Process::SignIn, 0));
    step(&mut machine, Event::DeploymentWorker(Ok(worker())));
    machine
}

#[test]
fn worker_is_admitted_from_its_rendered_files_between_sign_in_and_start() {
    let root = skein_scratch::Scratch::new("deployment-worker-admission");
    for path in ["config/worker", "data/engine/secrets"] {
        std::fs::create_dir_all(root.path().join(path)).expect("fixture directory");
    }
    std::fs::write(root.path().join(Document::Worker.path()), b"engine = \"127.0.0.1:4102\"\nname = \"sidecar\"\n")
        .expect("worker document");
    let digest = "42".repeat(32);
    std::fs::write(root.path().join("data/engine/secrets/worker.digest"), format!("{digest}\n"))
        .expect("synthetic digest");
    let mut plan = deployment_plan();
    plan.options.scratch = root.path().into();
    let mut machine = at_engine();
    let actions = engine_ready(&mut machine);
    assert!(matches!(actions.as_slice(), [Action::DeploymentLog { .. }, Action::SignInFile { .. }]));
    assert!(!machine.ready());
    let actions = step(&mut machine, exited(Process::SignIn, 0));
    assert!(matches!(actions.as_slice(), [Action::RemoveLink, Action::ReadDeploymentWorker]));
    let actions = step(&mut machine, Event::DeploymentWorker(io::deployment::worker(&plan)));
    let [Action::Start { process: Process::Ctl, program }] = actions.as_slice() else { panic!("{actions:?}") };
    assert_eq!(program.args, ["ctl", "/scratch/session/ctl", "worker", "list", "--json"]);
    let actions = step(&mut machine, listed(br#"[{"name":"sidecar-extra"}]"#));
    let [Action::Start { process: Process::Ctl, program }] = actions.as_slice() else { panic!("{actions:?}") };
    assert_eq!(
        program.args,
        [
            "ctl",
            "/scratch/session/ctl",
            "worker",
            "add",
            "sidecar",
            "--places",
            "2",
            "--digest",
            &digest,
            "--key",
            "qa-deployment-worker"
        ]
    );
    assert_eq!(program.input, Input::Null);
    assert_eq!(machine.next_deadline(), Some(Time::ZERO.saturating_add(STEP)));
    assert!(!machine.ready());
    assert_start(&step(&mut machine, exited(Process::Ctl, 0)), Process::Worker);
    assert_eq!(machine.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(120))));
    assert!(!machine.ready());
    assert!(matches!(
        process_line(&mut machine, Process::Worker, Output::Stderr, "linked sidecar\n").as_slice(),
        [Action::DeploymentLog { .. }, Action::Ready]
    ));
}

#[test]
fn a_held_worker_starts_without_another_add() {
    let mut machine = listing();
    let actions = step(&mut machine, listed(br#"[{"name":"other"},{"name":"worker"}]"#));
    assert_start(&actions, Process::Worker);
    assert_eq!(actions.len(), 1, "no add or answer escapes the admission step");
    assert!(!machine.ready());
    process_line(&mut machine, Process::Worker, Output::Stderr, "linked worker\n");
    assert!(machine.ready());
}

#[test]
fn a_worker_admission_refusal_or_deadline_fails_the_deployment() {
    for adding in [false, true] {
        for late in [false, true] {
            let mut machine = listing();
            if adding {
                step(&mut machine, listed(b"[]"));
            }
            let actions = if late {
                let at = machine.next_deadline().expect("admission bound");
                machine.step(Event::Deadline, at)
            } else {
                step(&mut machine, exited(Process::Ctl, 1))
            };
            assert!(!actions.iter().any(|action| matches!(action, Action::Start { process: Process::Worker, .. })));
            stopped(&mut machine);
            let actions = step(&mut machine, Event::DeploymentLogs(vec![]));
            assert_eq!(finish(&actions).status, 4);
        }
    }
}
