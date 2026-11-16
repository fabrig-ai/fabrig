use super::{exited, finish, plan, step};
use crate::drive::{Action, Event, Input, Machine, Output, Process, io};
use skein_lib::Time;
use skein_scratch::Scratch;
use std::{fs, io::Write};

const FIRST: &[u8] = b"http://127.0.0.1:9/link?secret=first\n";
const FRESH: &[u8] = b"http://127.0.0.1:9/link?secret=fresh\n";
const LATER: &[u8] = b"http://127.0.0.1:9/link?secret=later\n";
const DATA: &[u8] = b"box: effective data /scratch/data\n";
const FORGE: &[u8] = b"box: fake forgejo http://127.0.0.1:8002\n";

fn box_output(machine: &mut Machine, file: &mut fs::File, output: Output, line: &[u8]) -> Vec<Action> {
    let actions = step(machine, Event::BoxLine { output, line: line.into() });
    for action in &actions {
        if let Action::Log { line } = action {
            file.write_all(line).expect("append session/box.log");
        }
    }
    actions
}

fn log_and_failure_tail(ctl: bool) {
    for failure in [b"serve: failed; giving up\n".as_slice(), b"serve: failed; restarting in 1 s\n".as_slice()] {
        let scratch = Scratch::new("drive-box-log");
        fs::create_dir(scratch.path().join("session")).expect("session");
        let path = scratch.path().join("session/box.log");
        let mut file = fs::OpenOptions::new().create_new(true).append(true).open(&path).expect("box log");
        let mut plan = plan();
        plan.options.scratch = scratch.path().into();
        plan.options.ctl = ctl;
        let (mut machine, _) = Machine::start(plan, Time::ZERO);

        let actions = box_output(&mut machine, &mut file, Output::Stderr, DATA);
        assert!(matches!(actions.as_slice(), [Action::Log { line }] if line.as_ref() == DATA));
        let actions = box_output(&mut machine, &mut file, Output::Stdout, FIRST);
        let [Action::SignInFile { program, .. }] = actions.as_slice() else {
            panic!("stdout starts sign-in without logging: {actions:?}")
        };
        assert_eq!(program.input, Input::Null);
        let actions = box_output(&mut machine, &mut file, Output::Stderr, FORGE);
        assert!(matches!(actions.as_slice(), [Action::Log { line }] if line.as_ref() == FORGE));
        if !ctl {
            let actions = step(&mut machine, exited(Process::SignIn, 0));
            assert!(
                matches!(actions.as_slice(), [Action::RemoveLink, Action::Open(url)] if url.as_ref() == FIRST.strip_suffix(b"\n").expect("newline"))
            );
            assert!(step(&mut machine, Event::FaceDone(Ok(()))).is_empty());
        }
        let actions = box_output(&mut machine, &mut file, Output::Stdout, FRESH);
        if ctl {
            assert!(actions.is_empty(), "ctl drops fresh links: {actions:?}");
        } else {
            assert!(
                matches!(actions.as_slice(), [Action::Open(url)] if url.as_ref() == FRESH.strip_suffix(b"\n").expect("newline"))
            );
        }
        assert!(box_output(&mut machine, &mut file, Output::Stdout, LATER).is_empty());
        let actions = box_output(&mut machine, &mut file, Output::Stderr, failure);
        if ctl {
            assert!(matches!(
                actions.as_slice(),
                [Action::Log { .. }, Action::StopCtl, Action::Stop { process: Process::Box, force: false }]
            ));
        } else {
            assert!(matches!(
                actions.as_slice(),
                [Action::Log { .. }, Action::CancelFace, Action::StopCtl, Action::CloseBrowser]
            ));
            assert!(matches!(
                step(&mut machine, Event::FaceDone(Ok(()))).as_slice(),
                [Action::Stop { process: Process::Box, force: false }]
            ));
        }
        assert!(matches!(step(&mut machine, exited(Process::Box, 1)).as_slice(), [Action::Classify]));

        let bytes = fs::read(&path).expect("box log bytes");
        assert_eq!(bytes, [DATA, FORGE, failure].concat());
        assert!(!bytes.windows(b"secret=".len()).any(|part| part == b"secret="));
        for link in [FIRST, FRESH, LATER] {
            assert!(!bytes.windows(link.len()).any(|part| part == link));
        }
        let (serve, forges, box_tail) = io::logs(scratch.path()).expect("failure logs read from scratch");
        let actions = step(&mut machine, Event::Logs { serve, forges, box_tail });
        let exit = finish(&actions);
        assert_eq!(exit.status, 4);
        assert_eq!(exit.error, None);
        let failure = String::from_utf8_lossy(failure);
        assert_eq!(exit.shown.first(), Some(&format!("box failed: {} ", failure.trim_end())));
        let expected: Vec<String> = [DATA, FORGE, failure.as_bytes()]
            .iter()
            .map(|line| format!("| {}", String::from_utf8_lossy(line).trim_end()))
            .collect();
        assert_eq!(exit.shown.get(1..), Some(expected.as_slice()));
        assert!(!exit.shown.join("\n").contains("secret="));
        for link in [FIRST, FRESH, LATER] {
            assert!(!exit.shown.join("\n").contains(String::from_utf8_lossy(link).trim_end()));
        }
    }
}

#[test]
fn full_session_logs_only_stderr_and_failure_tail_has_no_link() {
    log_and_failure_tail(false);
}

#[test]
fn ctl_session_logs_only_stderr_and_failure_tail_has_no_link() {
    log_and_failure_tail(true);
}
