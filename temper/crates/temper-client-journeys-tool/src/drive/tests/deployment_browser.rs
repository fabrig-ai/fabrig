//! Full deployment sessions over injected events (deployment.md, 5–7).
use super::*;

const ORIGIN: &[u8] = b"http://127.0.0.1:4101";
const REFUSED: &[u8] = b"Failed to load resource: the server responded with a status of 401 (Unauthorized)";
fn trouble(text: &[u8], url: &[u8]) -> Event {
    Event::PageTrouble { trouble: Trouble::Log, text: text.into(), url: url.into() }
}
fn opened(actions: &[Action], expected: &[u8]) {
    assert!(matches!(actions, [Action::Open(url)] if url.as_ref() == expected), "{actions:?}");
}
fn opening() -> Machine {
    let mut plan = deployment_plan();
    plan.options.ctl = false;
    let (mut machine, _) = Machine::start(plan, Time::ZERO);
    step(&mut machine, Event::DeploymentPrepared(Ok(values())));
    process_line(&mut machine, Process::Forgejo, Output::Stdout, "listening forge\n");
    process_line(&mut machine, Process::Model, Output::Stdout, "listening model\n");
    engine_ready(&mut machine);
    assert!(process_line(&mut machine, Process::Engine, Output::Stdout, "http://ignored/before-sign-in\n").is_empty());
    admit_worker(&mut machine);
    let actions = process_line(&mut machine, Process::Worker, Output::Stderr, "linked worker\n");
    let [Action::DeploymentLog { .. }, Action::Open(url)] = actions.as_slice() else { panic!("{actions:?}") };
    assert_eq!(url.as_ref(), b"http://127.0.0.1:4101/link?secret=not-logged");
    machine
}
fn fresh(machine: &mut Machine) -> Vec<Action> {
    process_line(machine, Process::Engine, Output::Stdout, "http://127.0.0.1:4101/link?secret=fresh\n")
}
fn ready_page() -> Machine {
    let mut machine = opening();
    step(&mut machine, Event::FaceDone(Ok(())));
    fresh(&mut machine);
    step(&mut machine, Event::FaceDone(Ok(())));
    step(&mut machine, Event::FaceDone(Ok(())));
    assert!(matches!(step(&mut machine, Event::Found(Some(NodeId(1)))).as_slice(), [Action::Ready]));
    machine
}
fn running_tui(machine: &mut Machine) {
    let actions = line(machine, "tui key enter");
    let [Action::StartTui { program, .. }] = actions.as_slice() else { panic!("{actions:?}") };
    assert_eq!(program.program, Path::new("/build/bin/temper"));
    assert_eq!(program.args, [OsStr::from_bytes(ORIGIN)]);
    step(machine, Event::TuiReady);
    // The drive reads only the browse record here; rendering a TUI world is unrelated to deployment control.
    let mut saved = Vec::new();
    temper_client_shell_tui::journey::Line::Browse(b"http://127.0.0.1:4101/terminal?code=WDJB-MJHT".as_slice().into())
        .write(&mut saved);
    step(machine, Event::TuiSaved(saved.into()));
    step(machine, Event::BrowserAddress([ORIGIN, b"/"].concat().into()));
    step(machine, Event::FaceDone(Ok(())));
    step(machine, Event::Found(Some(NodeId(1))));
    step(machine, Event::FaceDone(Ok(())));
    step(machine, Event::FaceDone(Ok(())));
    assert!(answer(&step(machine, Event::TuiWritten)).failed.is_none());
}
fn finish_stops(machine: &mut Machine) -> Vec<Action> {
    for process in [Process::Worker, Process::Engine, Process::Model, Process::Forgejo] {
        // The next process cannot be stopped until this one exits.
        let actions = step(machine, exited(process, 0));
        if process == Process::Forgejo {
            return actions;
        }
    }
    unreachable!("forge stopped")
}

#[test]
fn browser_start_takes_only_the_fresh_link_in_either_pipe_order() {
    for early in [true, false] {
        let mut machine = opening();
        let refused = || trouble(REFUSED, b"http://127.0.0.1:4101/jig/v1/sign-in/link");
        assert!(step(&mut machine, refused()).is_empty());
        if early {
            assert!(fresh(&mut machine).is_empty());
            assert!(
                process_line(&mut machine, Process::Engine, Output::Stdout, "http://ignored/second-fresh\n").is_empty()
            );
        } else {
            assert!(step(&mut machine, Event::FaceDone(Ok(()))).is_empty());
        }
        let actions = if early { step(&mut machine, Event::FaceDone(Ok(()))) } else { fresh(&mut machine) };
        opened(&actions, b"http://127.0.0.1:4101/link?secret=fresh");
        assert!(step(&mut machine, refused()).is_empty());
        assert!(fresh(&mut machine).is_empty());
        opened(&step(&mut machine, Event::FaceDone(Ok(()))), b"http://127.0.0.1:4101/");
        assert!(step(&mut machine, refused()).is_empty());
        assert!(matches!(step(&mut machine, Event::FaceDone(Ok(()))).as_slice(), [Action::Find(_)]));
        assert!(step(&mut machine, refused()).is_empty());
        assert!(!machine.ready());
        assert!(step(&mut machine, Event::Found(None)).is_empty());
        assert!(matches!(step(&mut machine, Event::Found(Some(NodeId(1)))).as_slice(), [Action::Ready]));
        assert!(machine.ready());
        assert_eq!(machine.next_deadline(), None);
        assert!(fresh(&mut machine).is_empty());
    }
}

#[test]
fn browser_start_failures_close_before_stops_and_classify_with_deployment_tails() {
    for fault in ["trouble", "fresh deadline", "main deadline"] {
        let mut machine = opening();
        let (actions, words) = if fault == "trouble" {
            let actions =
                step(&mut machine, trouble(b"missing 404", b"http://127.0.0.1:4101/app.js?secret=SECRET#part"));
            (actions, "Log: missing 404 (http://127.0.0.1:4101/app.js)")
        } else {
            step(&mut machine, Event::FaceDone(Ok(())));
            let (seconds, words) = if fault == "fresh deadline" {
                (30, "FreshLink: deadline passed")
            } else {
                fresh(&mut machine);
                step(&mut machine, Event::FaceDone(Ok(())));
                step(&mut machine, Event::FaceDone(Ok(())));
                step(&mut machine, Event::Found(None));
                (60, "Main: deadline passed")
            };
            let at = Time::ZERO.saturating_add(Duration::from_secs(seconds));
            (machine.step(Event::Deadline, at), words)
        };
        assert!(actions.iter().any(|a| matches!(a, Action::CloseBrowser)), "{actions:?}");
        assert!(!actions.iter().any(|a| matches!(a, Action::Stop { .. } | Action::Classify)));
        stop(&step(&mut machine, Event::FaceDone(Ok(()))), Process::Worker, false);
        assert!(matches!(finish_stops(&mut machine).as_slice(), [Action::DeploymentLogs]));
        let actions = step(
            &mut machine,
            Event::DeploymentLogs(vec![vec![b"forge tail".as_slice().into()], vec![], vec![], vec![]]),
        );
        let exit = finish(&actions);
        assert_eq!(exit.status, 4);
        assert_eq!(exit.shown, [format!("deployment failed: {words}"), "| forge tail".into()]);
        assert!(!format!("{actions:?}").contains("SECRET"));
    }
}

#[test]
fn full_deployment_lines_and_end_use_the_engine_origin_and_close_clients_first() {
    let mut machine = ready_page();
    opened(&line(&mut machine, "web reload \"/projects/1\""), b"http://127.0.0.1:4101/projects/1");
    step(&mut machine, Event::FaceDone(Ok(())));
    assert!(answer(&step(&mut machine, Event::Found(Some(NodeId(1))))).failed.is_none());
    running_tui(&mut machine);
    let actions = step(&mut machine, Event::End);
    assert!(actions.iter().any(|a| matches!(a, Action::StopTui)));
    assert!(!actions.iter().any(|a| matches!(a, Action::CloseBrowser | Action::Stop { .. })));
    assert!(matches!(step(&mut machine, Event::TuiStopped).as_slice(), [Action::CloseBrowser]));
    stop(&step(&mut machine, Event::FaceDone(Ok(()))), Process::Worker, false);
    assert_eq!(finish(&finish_stops(&mut machine)).status, 0);
}

fn patch_restarted(machine: &mut Machine) -> Vec<Action> {
    step(machine, exited(Process::Worker, 0));
    assert!(matches!(step(machine, exited(Process::Engine, 0)).as_slice(), [Action::WriteDeploymentPatch(_)]));
    assert_start(&step(machine, Event::DeploymentWritten(Ok(()))), Process::Engine);
    assert_start(&engine_ready(machine), Process::Worker);
    process_line(machine, Process::Worker, Output::Stderr, "linked worker\n")
}
#[test]
fn full_patch_validates_then_stops_tui_and_reloads_before_answering() {
    for main in [true, false] {
        let mut machine = ready_page();
        running_tui(&mut machine);
        line(&mut machine, "deployment patch engine landing.json");
        assert!(
            answer(&step(&mut machine, Event::DeploymentPatch(Err("missing landing.json".into())))).failed.is_some()
        );
        assert!(machine.tui_running);
        line(&mut machine, "deployment patch engine landing.json");
        assert!(step(&mut machine, trouble(b"engine down", ORIGIN)).is_empty());
        let actions = step(
            &mut machine,
            Event::DeploymentPatch(Ok(("/scratch/config/engine/temper.json".into(), b"{}".as_slice().into()))),
        );
        assert!(matches!(actions.as_slice(), [Action::StopTui]), "{actions:?}");
        stop(&step(&mut machine, Event::TuiStopped), Process::Worker, false);
        let actions = patch_restarted(&mut machine);
        assert!(
            matches!(actions.as_slice(), [Action::DeploymentLog { .. }, Action::Open(url)] if url.as_ref() == b"http://127.0.0.1:4101/"),
            "{actions:?}"
        );
        assert!(fresh(&mut machine).is_empty());
        assert!(step(&mut machine, trouble(b"engine down", ORIGIN)).is_empty());
        assert!(matches!(step(&mut machine, Event::FaceDone(Ok(()))).as_slice(), [Action::Find(_)]));
        if main {
            let actions = step(&mut machine, Event::Found(Some(NodeId(1))));
            assert_eq!(answer(&actions).print(), "| patched engine\nok\n");
        } else {
            step(&mut machine, Event::Found(None));
            let actions = machine.step(Event::Deadline, Time::ZERO.saturating_add(STEP));
            assert!(matches!(actions.as_slice(), [Action::CancelFace, Action::Describe { .. }]));
            let actions = step(&mut machine, Event::Description(b"loading".as_slice().into()));
            let answer = answer(&actions);
            assert!(
                answer
                    .failed
                    .as_deref()
                    .expect("failed page")
                    .contains("not found within 60 s; shown instead: loading")
            );
            assert_eq!(answer.shown, Vec::<String>::new());
        }
        assert!(machine.ready());
        assert_eq!(machine.next_deadline(), None);
        assert!(!machine.tui_running);
        assert!(!line(&mut machine, "ctl me").is_empty(), "deployment still takes lines");
    }
}

#[test]
fn restart_keeps_documents_and_reloads_only_a_full_session() {
    let mut boxed = super::super::ready();
    let actions = line(&mut boxed, "deployment restart");
    assert!(matches!(actions.as_slice(), [Action::Answer(answer)]
        if answer.failed.as_deref() == Some("line 1: a box takes no deployment line")));
    for ctl in [true, false] {
        let mut machine = if ctl { deployment_ready() } else { ready_page() };
        if !ctl {
            running_tui(&mut machine);
        }
        let source = "deployment restart";
        let actions = line(&mut machine, source);
        if ctl {
            stop(&actions, Process::Worker, false);
        } else {
            assert!(matches!(actions.as_slice(), [Action::StopTui]), "{actions:?}");
            stop(&step(&mut machine, Event::TuiStopped), Process::Worker, false);
        }
        stop(&step(&mut machine, exited(Process::Worker, 0)), Process::Engine, false);
        // No patch write: the next effect is starting the engine with the same documents.
        assert_start(&step(&mut machine, exited(Process::Engine, 0)), Process::Engine);
        assert_start(&engine_ready(&mut machine), Process::Worker);
        let mut actions = process_line(&mut machine, Process::Worker, Output::Stderr, "linked worker\n");
        if !ctl {
            assert!(
                matches!(actions.as_slice(), [Action::DeploymentLog { .. }, Action::Open(url)]
                if url.as_ref() == b"http://127.0.0.1:4101/"),
                "{actions:?}"
            );
            step(&mut machine, Event::FaceDone(Ok(())));
            actions = step(&mut machine, Event::Found(Some(NodeId(1))));
        }
        assert!(answer(&actions).failed.is_none());
        assert!(
            actions.iter().any(|action| matches!(action, Action::Keep(kept) if kept.as_ref() == source.as_bytes()))
        );
        assert!(machine.ready());
        assert!(!machine.tui_running);
    }
}
