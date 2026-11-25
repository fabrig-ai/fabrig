use super::*;
use crate::line::Document;
use crate::testing::deployment_plan;

fn values() -> crate::drive::deployment::Values {
    crate::drive::deployment::values(
        &deployment_plan().place().expect("place"),
        br#"{"admin_token":"/scratch/data/forgejo/admin"}"#,
        br#"{"key":"/scratch/data/model/key"}"#,
    )
    .expect("values")
}
fn process_line(machine: &mut Machine, process: Process, output: Output, text: &str) -> Vec<Action> {
    step(machine, Event::ProcessLine { process, output, line: text.as_bytes().into() })
}
fn starting() -> Machine {
    let (mut machine, actions) = Machine::start(deployment_plan(), Time::ZERO);
    assert!(matches!(actions.as_slice(), [Action::PrepareDeployment]));
    let actions = step(&mut machine, Event::DeploymentPrepared(Ok(values())));
    assert_start(&actions, Process::Forgejo);
    machine
}
fn assert_start(actions: &[Action], expected: Process) {
    let program = actions
        .iter()
        .find_map(|action| {
            if let Action::Start { process, program } = action {
                assert_eq!(*process, expected);
                Some(program)
            } else {
                None
            }
        })
        .expect("process started");
    assert_eq!(program.input, Input::Null);
    assert_eq!(program.directory, Path::new("/scratch"));
    assert!(!format!("{program:?}").contains("cargo"));
    assert!(!format!("{program:?}").contains("scripts/bundle.sh"));
    for (name, value) in &program.environment {
        if name != "PATH" {
            assert!(Path::new(value).starts_with("/scratch/session"));
        }
    }
}
fn engine_ready(machine: &mut Machine) -> Vec<Action> {
    process_line(machine, Process::Engine, Output::Stderr, "listening workers 127.0.0.1:4102\n");
    process_line(machine, Process::Engine, Output::Stdout, "http://127.0.0.1:4101/link?secret=not-logged\n");
    process_line(machine, Process::Engine, Output::Stderr, "listening clients 127.0.0.1:4101\n")
}
fn at_engine() -> Machine {
    let mut machine = starting();
    assert_start(&process_line(&mut machine, Process::Forgejo, Output::Stdout, "listening forge\n"), Process::Model);
    assert_start(&process_line(&mut machine, Process::Model, Output::Stdout, "listening model\n"), Process::Engine);
    machine
}
const EMPTY_WORKER_LIST: &[u8] = br#"{"rows":[],"digest":"1633902c6cbba5e7"}"#;

fn listed(bytes: &[u8]) -> Event {
    Event::Exited { process: Process::Ctl, code: 0, stdout: bytes.into(), stderr: Box::new([]) }
}

pub(super) fn admit_worker(machine: &mut Machine) {
    assert!(matches!(
        step(machine, exited(Process::SignIn, 0)).as_slice(),
        [Action::RemoveLink, Action::ReadDeploymentWorker]
    ));
    let worker =
        crate::drive::deployment::Worker::read(b"engine = \"127.0.0.1:4102\"\nname = \"worker\"\n", &[b'a'; 64])
            .expect("worker");
    assert_start(&step(machine, Event::DeploymentWorker(Ok(worker))), Process::Ctl);
    assert_start(&step(machine, listed(EMPTY_WORKER_LIST)), Process::Ctl);
    assert_start(&step(machine, exited(Process::Ctl, 0)), Process::Worker);
}
fn deployment_ready() -> Machine {
    let mut machine = at_engine();
    engine_ready(&mut machine);
    admit_worker(&mut machine);
    let actions = process_line(&mut machine, Process::Worker, Output::Stderr, "linked worker\n");
    assert!(actions.iter().any(|action| matches!(action, Action::Ready)));
    machine
}
fn stop(actions: &[Action], expected: Process, force: bool) {
    assert!(actions.iter().any(|action| matches!(action, Action::Stop { process, force: forced } if *process == expected && *forced == force)), "{actions:?}");
}
fn stopped(machine: &mut Machine) -> Vec<Action> {
    step(machine, Event::End);
    let mut actions = vec![];
    for process in [Process::Worker, Process::Engine, Process::Model, Process::Forgejo] {
        actions = step(machine, exited(process, 0));
    }
    actions
}
#[test]
fn start_waits_for_each_stream_and_uses_private_sign_in_file() {
    let (mut refused, _) = Machine::start(deployment_plan(), Time::ZERO);
    assert_eq!(
        finish(&step(&mut refused, Event::DeploymentPrepared(Err("engine.toml: missing".into())))),
        &Exit::own("engine.toml: missing".into())
    );
    let mut machine = starting();
    assert_eq!(machine.next_deadline(), Some(Time::ZERO.saturating_add(STEP)));
    let wrong = process_line(&mut machine, Process::Forgejo, Output::Stderr, "listening forge\n");
    assert!(matches!(wrong.as_slice(), [Action::DeploymentLog { process: Process::Forgejo, .. }]));
    assert_start(&process_line(&mut machine, Process::Forgejo, Output::Stdout, "listening forge\n"), Process::Model);
    assert_eq!(machine.next_deadline(), Some(Time::ZERO.saturating_add(STEP)));
    assert_start(&process_line(&mut machine, Process::Model, Output::Stdout, "listening model\n"), Process::Engine);
    assert!(process_line(&mut machine, Process::Engine, Output::Stdout, "not a link\n").is_empty());
    assert_eq!(machine.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(120))));
    process_line(&mut machine, Process::Engine, Output::Stderr, "listening clients yes\n");
    let actions =
        process_line(&mut machine, Process::Engine, Output::Stdout, "http://127.0.0.1:4101/link?secret=first\n");
    assert!(actions.is_empty(), "engine must wait for both listeners without logging stdout");
    assert!(!machine.ready());
    let actions = process_line(&mut machine, Process::Engine, Output::Stderr, "listening workers yes\n");
    assert_eq!(machine.next_deadline(), Some(Time::ZERO.saturating_add(STEP)));
    assert!(
        process_line(&mut machine, Process::Engine, Output::Stdout, "http://127.0.0.1:4101/link?secret=later\n")
            .is_empty()
    );
    let [Action::DeploymentLog { .. }, Action::SignInFile { link, program }] = actions.as_slice() else {
        panic!("{actions:?}")
    };
    assert!(link.ends_with(b"secret=first\n"));
    assert_eq!(program.args, ["ctl", "/scratch/session/ctl", "sign-in", "/scratch/session/link"]);
    assert_eq!(program.input, Input::Null);
    assert!(!format!("{program:?}").contains("secret="));
    admit_worker(&mut machine);
    assert_eq!(machine.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(120))));
    let actions = process_line(&mut machine, Process::Worker, Output::Stderr, "linked worker\n");
    assert!(matches!(actions.as_slice(), [Action::DeploymentLog { .. }, Action::Ready]));
    assert!(
        process_line(&mut machine, Process::Engine, Output::Stdout, "http://127.0.0.1:4101/link?secret=late\n")
            .is_empty()
    );
    assert!(machine.ready());
}
#[test]
fn values_and_forge_hands_come_from_documents_and_face_lines_are_unkept() {
    let mut machine = deployment_ready();
    for source in ["web look", "web look 800", r#"web see "text""#, "tui look", r#"tui see "text""#] {
        let actions = line(&mut machine, source);
        assert!(matches!(actions.as_slice(), [Action::Answer(_)]), "{actions:?}");
        assert!(answer(&actions).print().ends_with(": this session takes ctl, forge and deployment lines alone\n"));
    }
    let source = "ctl forge connect forgejo-admin {forgejo.admin} --api {forgejo.api} --git {forgejo.git} --git-program {git} --key {model.key}";
    let actions = line(&mut machine, source);
    let [Action::Start { process: Process::Ctl, program }] = actions.as_slice() else { panic!("{actions:?}") };
    for value in [
        "/scratch/data/forgejo/admin",
        "127.0.0.1:4109",
        "http://127.0.0.1:4109",
        "/scratch/data/bin/git",
        "/scratch/data/model/key",
    ] {
        assert!(program.args.contains(&value.into()), "{program:?}");
    }
    assert!(
        matches!(step(&mut machine, exited(Process::Ctl, 0)).as_slice(), [Action::Keep(bytes), Action::Answer(_)] if bytes.as_ref() == source.as_bytes())
    );
    for source in
        [r#"forge commit forgejo work/repo "message" "a""#, r#"forge push forgejo work/repo branch "message" "a""#]
    {
        assert!(
            matches!(line(&mut machine, source).as_slice(), [Action::ReadForgeFiles { directory, .. }] if directory == Path::new("/recipe/files"))
        );
        let actions = step(&mut machine, Event::ForgeFiles(Ok(vec![b"content".as_slice().into()])));
        let [Action::ForgeRequest { api, token_file, .. }] = actions.as_slice() else { panic!("{actions:?}") };
        assert_eq!(api.as_ref(), b"http://127.0.0.1:4109/api/v1");
        assert_eq!(token_file, Path::new("/scratch/data/forgejo/admin"));
        step(&mut machine, Event::ForgeFailure("injected refusal".into()));
    }
}
#[test]
fn patch_validates_before_stop_then_restarts_engine_and_worker_without_sign_in() {
    let mut machine = deployment_ready();
    let source = "deployment patch engine public.toml";
    assert!(matches!(
        line(&mut machine, source).as_slice(),
        [Action::ReadDeploymentPatch { document: Document::Engine, .. }]
    ));
    let refused = step(&mut machine, Event::DeploymentPatch(Err("public.toml: missing".into())));
    assert!(matches!(refused.as_slice(), [Action::Keep(_), Action::Answer(_)]));
    assert!(answer(&refused).failed.as_ref().expect("refusal").contains("public.toml"));
    line(&mut machine, source);
    assert_eq!(machine.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(300))));
    let file =
        (PathBuf::from("/scratch").join(Document::Engine.path()), b"workers = \"127.0.0.1:4102\"\n".as_slice().into());
    stop(&step(&mut machine, Event::DeploymentPatch(Ok(file))), Process::Worker, false);
    stop(&step(&mut machine, exited(Process::Worker, 0)), Process::Engine, false);
    assert!(matches!(step(&mut machine, exited(Process::Engine, 0)).as_slice(), [Action::WriteDeploymentPatch(_)]));
    assert_start(&step(&mut machine, Event::DeploymentWritten(Ok(()))), Process::Engine);
    assert_start(&engine_ready(&mut machine), Process::Worker);
    let actions = process_line(&mut machine, Process::Worker, Output::Stderr, "linked worker\n");
    assert_eq!(answer(&actions).print(), "| patched engine\nok\n");
    assert!(actions.iter().any(|action| matches!(action, Action::Keep(bytes) if bytes.as_ref() == source.as_bytes())));
    assert!(!actions.iter().any(|action| matches!(action, Action::SignInFile { .. })));
    assert!(machine.ready());
    assert_eq!(finish(&stopped(&mut machine)).status, 0);
}
#[test]
fn clean_stop_is_reverse_and_each_bound_kills_the_group_on_expiry() {
    let mut machine = deployment_ready();
    stop(&step(&mut machine, Event::End), Process::Worker, false);
    for (process, next, seconds) in [
        (Process::Worker, Some(Process::Engine), 60),
        (Process::Engine, Some(Process::Model), 60),
        (Process::Model, Some(Process::Forgejo), 30),
        (Process::Forgejo, None, 30),
    ] {
        assert_eq!(machine.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(seconds))));
        let actions = step(&mut machine, exited(process, 0));
        if let Some(next) = next {
            stop(&actions, next, false);
        } else {
            assert_eq!(finish(&actions), &Exit { status: 0, shown: vec!["ended".into()], error: None });
        }
    }
    for target in [Process::Worker, Process::Engine, Process::Model, Process::Forgejo] {
        let mut machine = deployment_ready();
        step(&mut machine, Event::End);
        for process in [Process::Worker, Process::Engine, Process::Model, Process::Forgejo] {
            if process == target {
                break;
            }
            step(&mut machine, exited(process, 0));
        }
        let at = machine.next_deadline().expect("stop bound");
        let actions = machine.step(Event::Deadline, at);
        stop(&actions, target, true);
        // Every still live process is reaped; a killed process's late exit cannot finish a later stop.
        for process in [Process::Worker, Process::Engine, Process::Model, Process::Forgejo] {
            step(&mut machine, exited(process, 0));
        }
        let actions = step(&mut machine, Event::DeploymentLogs(vec![vec![], vec![], vec![], vec![]]));
        assert_eq!(finish(&actions).status, 4);
    }
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one fault matrix holds the start and restart failure outcomes and deadlines"
)]
fn failed_starts_and_patch_restart_failures_end_as_deployment_failures() {
    for stage in [Process::Forgejo, Process::Model, Process::Engine, Process::Worker, Process::SignIn] {
        for deadline in [false, true] {
            let mut machine = starting();
            if stage != Process::Forgejo {
                process_line(&mut machine, Process::Forgejo, Output::Stdout, "listening forge\n");
            }
            if !matches!(stage, Process::Forgejo | Process::Model) {
                process_line(&mut machine, Process::Model, Output::Stdout, "listening model\n");
            }
            if matches!(stage, Process::Worker | Process::SignIn) {
                engine_ready(&mut machine);
            }
            if stage == Process::Worker {
                admit_worker(&mut machine);
            }
            let actions = if deadline {
                let at = machine.next_deadline().expect("start deadline");
                machine.step(Event::Deadline, at)
            } else {
                step(&mut machine, exited(stage, 1))
            };
            assert!(actions.iter().any(|action| matches!(action, Action::RemoveLink)));
            for process in [Process::Worker, Process::Engine, Process::Model, Process::Forgejo] {
                step(&mut machine, exited(process, 0));
            }
            let logs = (0..4).map(|_| (0..25).map(|n| format!("log {n}").into_bytes().into()).collect()).collect();
            let actions = step(&mut machine, Event::DeploymentLogs(logs));
            let exit = finish(&actions);
            assert_eq!(exit.status, 4);
            assert!(exit.shown.first().expect("why").starts_with("deployment failed: "));
            assert_eq!(exit.shown.len(), 81);
            assert_eq!(exit.shown.get(1).expect("tail"), "| log 5");
        }
    }
    let mut machine = at_engine();
    stop(&step(&mut machine, exited(Process::Forgejo, 1)), Process::Engine, false);
    let mut machine = starting();
    let at = machine.next_deadline().expect("forge deadline");
    let actions = machine.step(
        Event::ProcessLine {
            process: Process::Forgejo,
            output: Output::Stdout,
            line: b"listening late\n".as_slice().into(),
        },
        at,
    );
    assert!(!actions.iter().any(|action| matches!(action, Action::Start { .. })));
    let mut machine = deployment_ready();
    line(&mut machine, "deployment patch engine public.toml");
    step(
        &mut machine,
        Event::DeploymentPatch(Ok((
            PathBuf::from("/scratch").join(Document::Engine.path()),
            b"workers = \"127.0.0.1:4102\"\n".as_slice().into(),
        ))),
    );
    step(&mut machine, exited(Process::Worker, 0));
    step(&mut machine, exited(Process::Engine, 0));
    let late = Time::ZERO.saturating_add(Duration::from_secs(250));
    machine.step(Event::DeploymentWritten(Ok(())), late);
    machine.step(
        Event::ProcessLine {
            process: Process::Engine,
            output: Output::Stderr,
            line: b"listening clients yes\n".as_slice().into(),
        },
        late,
    );
    machine.step(
        Event::ProcessLine {
            process: Process::Engine,
            output: Output::Stderr,
            line: b"listening workers yes\n".as_slice().into(),
        },
        late,
    );
    machine.step(
        Event::ProcessLine {
            process: Process::Engine,
            output: Output::Stdout,
            line: b"http://127.0.0.1:4101/link\n".as_slice().into(),
        },
        late,
    );
    assert_eq!(machine.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(300))));
    let expired = machine.step(Event::Deadline, Time::ZERO.saturating_add(Duration::from_secs(300)));
    assert!(answer(&expired).failed.is_some());
    for process in [Process::Engine, Process::Worker] {
        let mut machine = deployment_ready();
        line(&mut machine, "deployment patch engine public.toml");
        step(
            &mut machine,
            Event::DeploymentPatch(Ok((
                PathBuf::from("/scratch").join(Document::Engine.path()),
                b"workers = \"127.0.0.1:4102\"\n".as_slice().into(),
            ))),
        );
        step(&mut machine, exited(Process::Worker, 0));
        step(&mut machine, exited(Process::Engine, 0));
        step(&mut machine, Event::DeploymentWritten(Ok(())));
        if process == Process::Worker {
            engine_ready(&mut machine);
        }
        let actions = step(&mut machine, exited(process, 1));
        assert!(answer(&actions).failed.is_some());
        assert_eq!(answer(&line(&mut machine, "ctl me")).print(), "failed: the deployment exited\n");
        let refused = line(&mut machine, r#"web see "never run""#);
        assert!(matches!(refused.as_slice(), [Action::Answer(_)]));
        assert_eq!(answer(&refused).print(), "failed: the deployment exited\n");
        assert_eq!(answer(&line(&mut machine, "invalid syntax")).print(), "failed: the deployment exited\n");
        stopped(&mut machine);
        assert_eq!(finish(&step(&mut machine, Event::DeploymentLogs(vec![]))).status, 4);
    }
}
#[test]
fn named_store_and_forge_refusals_are_status_three() {
    for (index, text, status) in [
        (2, "temper-engine serve: The store in /scratch/data/engine holds an unreadable record", 3),
        (0, "temper-box fake forge: state /scratch/data/forgejo/state refused: unknown version", 3),
        (2, "temper-engine serve: The store in /scratch-other/data holds a record", 4),
        (0, "temper-box fake forge: state /outside/state refused: unknown version", 4),
    ] {
        let mut machine = deployment_ready();
        step(&mut machine, exited(Process::Engine, 1));
        stopped(&mut machine);
        let mut logs = vec![vec![], vec![], vec![], vec![]];
        logs[index].push(text.as_bytes().into());
        let actions = step(&mut machine, Event::DeploymentLogs(logs));
        assert_eq!(finish(&actions).status, status);
        if status == 3 {
            assert_eq!(finish(&actions).shown, [format!("refused: {text}")]);
        }
    }
}

#[path = "deployment_worker.rs"]
mod admission;
#[path = "deployment_browser.rs"]
mod browser;
