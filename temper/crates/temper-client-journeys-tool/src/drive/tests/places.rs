//! One session path over the two places' injected events (testing/qa/kinds.md, 9).
use super::*;

const ORIGIN: &[u8] = b"http://127.0.0.1:4101";
const FIRST: &[u8] = b"http://127.0.0.1:4101/jig/v1/sign-in/link?secret=first";
const FRESH: &[u8] = b"http://127.0.0.1:4101/jig/v1/sign-in/link?secret=fresh";

#[derive(Clone, Copy, Debug)]
enum Kind {
    Box,
    Deployment,
}
const KINDS: [Kind; 2] = [Kind::Box, Kind::Deployment];
impl Kind {
    fn plan(self, ctl: bool) -> Plan {
        let mut plan = plan();
        plan.options.ctl = ctl;
        plan.options.ports = Some("1,2,3,4,5,6,7,8,9".into());
        if matches!(self, Self::Deployment) {
            plan.options.deployment = Some("/recipe/config".into());
        }
        plan
    }
    fn link(self, address: &[u8]) -> Event {
        let line = [address, b"\n"].concat().into();
        match self {
            Self::Box => Event::BoxLine { output: Output::Stdout, line },
            Self::Deployment => Event::ProcessLine { process: Process::Engine, output: Output::Stdout, line },
        }
    }
    fn signing(self, ctl: bool) -> Machine {
        let plan = self.plan(ctl);
        let (mut machine, actions) = Machine::start(plan.clone(), Time::ZERO);
        match self {
            Self::Box => assert!(matches!(actions.as_slice(), [Action::Start { process: Process::Box, .. }])),
            Self::Deployment => {
                assert!(matches!(actions.as_slice(), [Action::PrepareDeployment]));
                let values = crate::drive::deployment::values(
                    &plan.place().expect("place"),
                    br#"{"admin_token":"/scratch/data/forgejo/admin"}"#,
                    br#"{"key":"/scratch/data/model/key"}"#,
                )
                .expect("values");
                step(&mut machine, Event::DeploymentPrepared(Ok(values)));
                for (process, output, text) in [
                    (Process::Forgejo, Output::Stdout, "listening forge\n"),
                    (Process::Model, Output::Stdout, "listening model\n"),
                    (Process::Engine, Output::Stderr, "listening clients 127.0.0.1:4101\n"),
                    (Process::Engine, Output::Stderr, "listening workers 127.0.0.1:4102\n"),
                ] {
                    step(&mut machine, Event::ProcessLine { process, output, line: text.as_bytes().into() });
                }
            }
        }
        let actions = step(&mut machine, self.link(FIRST));
        let client_actions = actions.iter().filter(|a| !matches!(a, Action::DeploymentLog { .. })).collect::<Vec<_>>();
        let [Action::SignInFile { link, program }] = client_actions.as_slice() else { panic!("{actions:?}") };
        assert_eq!(link.strip_suffix(b"\n").unwrap_or(link), FIRST);
        assert_eq!(program.args, ["ctl", "/scratch/session/ctl", "sign-in", "/scratch/session/link"]);
        assert_eq!(program.input, Input::Null);
        assert!(!machine.ready());
        machine
    }
    fn started(self, machine: &mut Machine) -> Vec<Action> {
        match self {
            Self::Box => {
                let mut actions = step(machine, exited(Process::SignIn, 0));
                assert!(matches!(actions.remove(0), Action::RemoveLink));
                actions
            }
            Self::Deployment => {
                super::deployment::admit_worker(machine);
                step(
                    machine,
                    Event::ProcessLine {
                        process: Process::Worker,
                        output: Output::Stderr,
                        line: b"linked worker\n".as_slice().into(),
                    },
                )
                .into_iter()
                .filter(|action| !matches!(action, Action::DeploymentLog { .. }))
                .collect()
            }
        }
    }
    fn stopping(self) -> &'static [Process] {
        match self {
            Self::Box => &[Process::Box],
            Self::Deployment => &[Process::Worker, Process::Engine, Process::Model, Process::Forgejo],
        }
    }
    fn logs(self, refusal: bool) -> Event {
        match self {
            Self::Box => Event::Logs {
                serve: if refusal {
                    vec![b"temper serve: store /scratch/data/box refused".as_slice().into()]
                } else {
                    vec![]
                },
                forges: vec![],
                box_tail: vec![],
            },
            Self::Deployment => Event::DeploymentLogs(vec![
                vec![],
                vec![],
                if refusal {
                    vec![b"temper-engine serve: store /scratch/data/engine refused".as_slice().into()]
                } else {
                    vec![]
                },
                vec![],
            ]),
        }
    }
    fn word(self) -> &'static str {
        match self {
            Self::Box => "box",
            Self::Deployment => "deployment",
        }
    }
}
fn opened(actions: &[Action], address: &[u8]) {
    assert!(matches!(actions, [Action::Open(url)] if url.as_ref() == address), "{actions:?}");
}
fn opening(kind: Kind) -> Machine {
    let mut machine = kind.signing(false);
    opened(&kind.started(&mut machine), FIRST);
    machine
}
fn page_ready(kind: Kind, machine: &mut Machine) {
    assert!(step(machine, Event::FaceDone(Ok(()))).is_empty());
    opened(&step(machine, kind.link(FRESH)), FRESH);
    opened(&step(machine, Event::FaceDone(Ok(()))), &[ORIGIN, b"/"].concat());
    assert!(
        matches!(step(machine, Event::FaceDone(Ok(()))).as_slice(), [Action::Find(find)] if *find == Find::named(Role::Main, b""))
    );
    assert!(!machine.ready());
    assert!(matches!(step(machine, Event::Found(Some(NodeId(1)))).as_slice(), [Action::Ready]));
}
fn stop_expected(actions: &[Action], expected: Process) {
    assert!(matches!(actions, [Action::Stop { process, force: false }] if *process == expected), "{actions:?}");
}

#[test]
fn both_places_use_the_same_session_steps() {
    for kind in KINDS {
        for ctl in [false, true] {
            let mut machine = kind.signing(ctl);
            let actions = kind.started(&mut machine);
            if ctl {
                assert!(matches!(actions.as_slice(), [Action::Ready]), "{kind:?}: {actions:?}");
            } else {
                opened(&actions, FIRST);
                page_ready(kind, &mut machine);
                let actions = line(&mut machine, "tui key enter");
                assert!(
                    matches!(actions.as_slice(), [Action::StartTui { program, .. }] if program.args == [OsStr::from_bytes(ORIGIN)])
                );
                for event in [
                    Event::TuiReady,
                    Event::TuiSaved(super::tui::saves().1),
                    Event::BrowserAddress([ORIGIN, b"/"].concat().into()),
                    Event::FaceDone(Ok(())),
                    Event::Found(Some(NodeId(1))),
                    Event::FaceDone(Ok(())),
                    Event::FaceDone(Ok(())),
                ] {
                    step(&mut machine, event);
                }
                assert!(answer(&step(&mut machine, Event::TuiWritten)).failed.is_none());
            }
            assert!(machine.ready());
            assert!(step(&mut machine, kind.link(b"http://ignored/late")).is_empty());
            let actions = line(&mut machine, "end");
            if ctl {
                assert!(actions.iter().all(|a| !matches!(a, Action::Open(_) | Action::CloseBrowser | Action::StopTui)));
                assert!(actions.iter().any(
                    |a| matches!(a, Action::Stop { process, force: false } if Some(process) == kind.stopping().first())
                ));
            } else {
                assert!(actions.iter().any(|a| matches!(a, Action::StopTui)));
                assert!(!actions.iter().any(|a| matches!(a, Action::Stop { .. } | Action::CloseBrowser)));
                assert!(matches!(step(&mut machine, Event::TuiStopped).as_slice(), [Action::CloseBrowser]));
                stop_expected(
                    &step(&mut machine, Event::FaceDone(Ok(()))),
                    *kind.stopping().first().expect("first stop"),
                );
            }
            let mut actions = vec![];
            for process in kind.stopping() {
                actions = step(&mut machine, exited(*process, 0));
            }
            assert_eq!(finish(&actions), &Exit { status: 0, shown: vec!["ended".into()], error: None });
            assert_eq!(machine.next_deadline(), None);
        }
    }
}

#[test]
fn both_places_ignore_the_spent_links_401() {
    for kind in KINDS {
        let mut machine = opening(kind);
        let deadline = machine.next_deadline();
        assert!(
            step(
                &mut machine,
                Event::PageTrouble {
                    trouble: Trouble::Log,
                    text: b"Failed to load resource: the server responded with a status of 401 (Unauthorized)"
                        .as_slice()
                        .into(),
                    url: [ORIGIN, b"/jig/v1/sign-in/link"].concat().into(),
                }
            )
            .is_empty()
        );
        assert_eq!(machine.next_deadline(), deadline);
        page_ready(kind, &mut machine);
        assert!(machine.ready());
    }
}

#[test]
fn each_places_start_trouble_keeps_its_failure_word_and_status() {
    for kind in KINDS {
        let mut machine = opening(kind);
        let actions = step(
            &mut machine,
            Event::PageTrouble {
                trouble: Trouble::Log,
                text: b"missing 404".as_slice().into(),
                url: [ORIGIN, b"/app.js?secret=SECRET"].concat().into(),
            },
        );
        assert!(actions.iter().any(|a| matches!(a, Action::CloseBrowser)));
        assert!(!actions.iter().any(|a| matches!(a, Action::Stop { .. })));
        stop_expected(&step(&mut machine, Event::FaceDone(Ok(()))), *kind.stopping().first().expect("first stop"));
        for process in kind.stopping() {
            step(&mut machine, exited(*process, 0));
        }
        let actions = step(&mut machine, kind.logs(false));
        let exit = finish(&actions);
        assert_eq!(exit.status, 4);
        assert_eq!(exit.shown, [format!("{} failed: Log: missing 404 (http://127.0.0.1:4101/app.js)", kind.word())]);
    }
}

#[test]
fn each_places_named_store_refusal_is_status_three() {
    for kind in KINDS {
        let mut machine = kind.signing(true);
        kind.started(&mut machine);
        let failed = match kind {
            Kind::Box => Process::Box,
            Kind::Deployment => Process::Engine,
        };
        step(&mut machine, exited(failed, 1));
        step(&mut machine, Event::End);
        for process in kind.stopping().iter().filter(|process| **process != failed) {
            step(&mut machine, exited(*process, 0));
        }
        let actions = step(&mut machine, kind.logs(true));
        let exit = finish(&actions);
        assert_eq!(exit.status, 3);
        assert_eq!(exit.shown.len(), 1);
        assert!(exit.shown.first().expect("refusal").starts_with("refused: temper"));
    }
}

#[test]
fn ports_by_use_keep_the_box_list_and_deployment_positions() {
    let plan = Kind::Box.plan(true);
    let program = plan.box_program();
    assert_eq!(
        program.environment.iter().find(|(name, _)| name == "TEMPER_INTERNAL_FAKE_BOX_PORTS").expect("ports").1,
        "1,2,3,4,5,6,7,8,9"
    );
    let place = plan.place().expect("place");
    for (name, expected) in [("clients", 1), ("workers", 2), ("webhooks", 5), ("model", 8), ("forgejo", 9)] {
        assert_eq!(place.value(name).expect("named port").as_ref(), format!("127.0.0.1:{expected}").as_bytes());
    }
}
