use super::*;
use crate::line::Document;
use sha2::{Digest, Sha256};

fn deployment_plan() -> Plan {
    let mut plan = plan();
    plan.options.deployment = Some("/recipe/config".into());
    plan.options.bundle = None;
    plan.options.ports = Some("4101,4102,4103,4104,4105,4106,4107,4108,4109".into());
    plan.options.files = Some("/recipe/files".into());
    plan
}
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
fn deployment_ready() -> Machine {
    let mut machine = at_engine();
    assert_start(&engine_ready(&mut machine), Process::Worker);
    process_line(&mut machine, Process::Worker, Output::Stderr, "linked worker\n");
    assert!(matches!(step(&mut machine, exited(Process::SignIn, 0)).as_slice(), [Action::RemoveLink, Action::Ready]));
    machine
}
fn stop(actions: &[Action], expected: Process, force: bool) {
    assert!(actions.iter().any(|action| matches!(action, Action::StopDeployment { process, force: forced } if *process == expected && *forced == force)), "{actions:?}");
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
        finish(&step(&mut refused, Event::DeploymentPrepared(Err("engine.json: missing".into())))),
        &Exit::own("engine.json: missing".into())
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
    assert_start(
        &process_line(&mut machine, Process::Engine, Output::Stderr, "listening workers yes\n"),
        Process::Worker,
    );
    assert_eq!(machine.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(120))));
    assert!(
        process_line(&mut machine, Process::Engine, Output::Stdout, "http://127.0.0.1:4101/link?secret=later\n")
            .is_empty()
    );
    let actions = process_line(&mut machine, Process::Worker, Output::Stderr, "linked worker\n");
    let [Action::DeploymentLog { .. }, Action::SignInFile { link, program }] = actions.as_slice() else {
        panic!("{actions:?}")
    };
    assert!(link.ends_with(b"secret=first\n"));
    assert_eq!(program.args, ["ctl", "/scratch/session/ctl", "sign-in", "/scratch/session/link"]);
    assert_eq!(program.input, Input::Null);
    assert!(!format!("{program:?}").contains("secret="));
    assert!(matches!(step(&mut machine, exited(Process::SignIn, 0)).as_slice(), [Action::RemoveLink, Action::Ready]));
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
        assert!(answer(&actions).print().ends_with(": a deployment takes ctl, forge and deployment lines\n"));
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
    let source = "deployment patch engine landing.json";
    assert!(matches!(
        line(&mut machine, source).as_slice(),
        [Action::ReadDeploymentPatch { document: Document::Engine, .. }]
    ));
    let refused = step(&mut machine, Event::DeploymentPatch(Err("landing.json: missing".into())));
    assert!(matches!(refused.as_slice(), [Action::Keep(_), Action::Answer(_)]));
    assert!(answer(&refused).failed.as_ref().expect("refusal").contains("landing.json"));
    line(&mut machine, source);
    assert_eq!(machine.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(300))));
    let file = (PathBuf::from("/scratch/config/engine/temper.json"), b"{}".as_slice().into());
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
            if stage == Process::SignIn {
                process_line(&mut machine, Process::Worker, Output::Stderr, "linked worker\n");
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
    line(&mut machine, "deployment patch engine landing.json");
    step(
        &mut machine,
        Event::DeploymentPatch(Ok(("/scratch/config/engine/temper.json".into(), b"{}".as_slice().into()))),
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
        line(&mut machine, "deployment patch engine landing.json");
        step(
            &mut machine,
            Event::DeploymentPatch(Ok(("/scratch/config/engine/temper.json".into(), b"{}".as_slice().into()))),
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

#[test]
fn preparation_and_patch_files_are_private_held_and_atomic() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let root = skein_scratch::Scratch::new("drive-deployment-files");
    let mut plan = deployment_plan();
    plan.options.scratch = root.path().into();
    plan.options.deployment = Some(root.path().join("recipe"));
    plan.options.scripts.clear();
    let source = plan.options.deployment.as_ref().expect("source");
    fs::create_dir(source).expect("fixture source");
    for (name, bytes) in [
        ("engine.json", r#"{"keep":7,"application":{"landing":{"projects":[1]}}}"#),
        ("worker.json", "{}"),
        ("agent.json", "{}"),
        ("forgejo.json", r#"{"admin_token":"{place.data}/forgejo/admin"}"#),
        ("model.json", r#"{"key":"{place.data}/model/key"}"#),
    ] {
        fs::write(source.join(name), bytes).expect("fixture document");
    }
    let values = io::deployment::prepare_with(&plan, Some([42; 32])).expect("fresh");
    assert!(values.forgejo_admin.ends_with(b"/data/forgejo/admin"));
    assert_eq!(fs::read_link(root.path().join("data/bin/git")).expect("git link"), Path::new("/build/bin/temper-box"));
    for relative in [
        "config",
        "config/engine",
        "config/worker",
        "data",
        "data/bin",
        "data/engine",
        "data/engine/secrets",
        "data/worker",
        "data/worker/secrets",
        "data/forgejo",
        "data/model",
        "cache",
        "cache/engine",
        "cache/worker",
    ] {
        assert_eq!(fs::metadata(root.path().join(relative)).expect("directory").permissions().mode() & 0o777, 0o700);
    }
    for relative in [
        "config/engine/temper.json",
        "config/worker/temper.json",
        "config/worker/agent.json",
        "config/forgejo.json",
        "config/model.json",
        "data/worker/secrets/worker.secret",
        "data/engine/secrets/worker.digest",
    ] {
        assert_eq!(fs::metadata(root.path().join(relative)).expect("file").permissions().mode() & 0o777, 0o600);
    }
    let secret_path = root.path().join("data/worker/secrets/worker.secret");
    let digest_path = root.path().join("data/engine/secrets/worker.digest");
    let pair = crate::drive::deployment::secret_pair(&[42; 32]);
    let secret_hash = Sha256::digest(fs::read(&secret_path).expect("synthetic secret"));
    let digest_hash = Sha256::digest(fs::read(&digest_path).expect("synthetic digest"));
    assert_eq!(secret_hash, Sha256::digest(&pair.0));
    assert_eq!(digest_hash, Sha256::digest(&pair.1));
    fs::remove_dir_all(source).expect("held reads no recipe");
    plan.options.bin = "/new/bin".into();
    io::deployment::prepare_with(&plan, Some([13; 32])).expect("held");
    assert_eq!(fs::read_link(root.path().join("data/bin/git")).expect("remade git"), Path::new("/new/bin/temper-box"));
    assert_eq!(Sha256::digest(fs::read(&secret_path).expect("held secret")), secret_hash);
    assert_eq!(Sha256::digest(fs::read(&digest_path).expect("held digest")), digest_hash);
    let engine = root.path().join("config/engine/temper.json");
    let before = fs::read(&engine).expect("document");
    let patch_source = root.path().join("files");
    fs::create_dir(&patch_source).expect("files");
    plan.options.files = Some(patch_source.clone());
    for (name, content) in
        [("not-object", "[]"), ("invalid", "{"), ("landing.json", r#"{"application":{"landing":{"projects":[]}}}"#)]
    {
        fs::write(patch_source.join(name), content).expect("patch fixture");
    }
    for name in ["absent", "not-object", "invalid"] {
        assert!(io::deployment::patch(&plan, Document::Engine, name.as_bytes()).is_err());
        assert_eq!(fs::read(&engine).expect("unchanged"), before);
    }
    let (destination, bytes) = io::deployment::patch(&plan, Document::Engine, b"landing.json").expect("valid patch");
    assert_eq!(fs::read(&engine).expect("not written yet"), before);
    io::deployment::replace(&destination, &bytes).expect("atomic replacement");
    assert_eq!(fs::read(&engine).expect("patched"), br#"{"keep":7,"application":{"landing":{"projects":[]}}}"#);
    assert_eq!(fs::metadata(&engine).expect("private replacement").permissions().mode() & 0o777, 0o600);
    plan.options.files = None;
    assert!(
        io::deployment::patch(&plan, Document::Engine, b"landing.json")
            .expect_err("files missing")
            .contains("no files directory")
    );
    let link = root.path().join("session/link");
    io::deployment::directory(&root.path().join("session")).expect("session");
    io::deployment::write(&link, b"synthetic link\n").expect("link file");
    assert_eq!(fs::metadata(&link).expect("private link").permissions().mode() & 0o777, 0o600);
    io::deployment::remove_link(root.path()).expect("link removed");
    assert!(!link.exists());
    fs::remove_dir_all(root.path().join("data")).expect("remove data");
    assert!(io::deployment::prepare_with(&plan, Some([0; 32])).expect_err("config alone").contains("config/"));
    fs::rename(root.path().join("config"), root.path().join("data")).expect("data alone");
    assert!(io::deployment::prepare_with(&plan, Some([0; 32])).expect_err("data alone").contains("data/"));
}

#[test]
fn script_files_use_the_shared_joiner_and_name_a_bad_source() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let root = skein_scratch::Scratch::new("drive-script-files");
    let mut plan = deployment_plan();
    plan.options.scratch = root.path().into();
    for relative in [
        "config",
        "config/engine",
        "config/worker",
        "data",
        "data/engine",
        "data/worker",
        "data/forgejo",
        "data/model",
        "session",
    ] {
        fs::create_dir(root.path().join(relative)).expect("held fixture");
    }
    fs::write(root.path().join("config/forgejo.json"), br#"{"admin_token":"/unused/admin"}"#).expect("forge document");
    fs::write(root.path().join("config/model.json"), br#"{"listen":{"socket":"127.0.0.1:0"},"key":"/unused/key","tls":null,"models":[{"name":"qa","window":4096,"output":1024}],"cued":[],"asked":[],"otherwise":["fallback"]}"#).expect("model document");
    let first = root.path().join("first.json");
    let second = root.path().join("second.json");
    fs::write(&first, br#"{"asked":[{"cue":"one","turns":["a"]}],"otherwise":["old"]}"#).expect("first scripts");
    fs::write(&second, br#"{"asked":[{"cue":"two","turns":["b"]}],"otherwise":["new"]}"#).expect("second scripts");
    plan.options.scripts = vec![first, second.clone()];
    io::deployment::prepare_with(&plan, None).expect("scripts prepared");
    let output = root.path().join("session/scripts.json");
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&output).expect("joined file")).expect("JSON");
    assert_eq!(value["asked"], serde_json::json!([{"cue":"one","turns":["a"]},{"cue":"two","turns":["b"]}]));
    assert_eq!(value["otherwise"], serde_json::json!(["new"]));
    assert_eq!(fs::metadata(&output).expect("private scripts").permissions().mode() & 0o777, 0o600);
    fs::remove_file(&output).expect("next session");
    fs::write(&second, br#"{"unknown":[]}"#).expect("bad source");
    let refusal = io::deployment::prepare_with(&plan, None).expect_err("invalid scripts refused before processes");
    assert!(refusal.contains(&format!("{}: unknown: unknown member", second.display())), "{refusal}");
    assert!(!output.exists());
}
