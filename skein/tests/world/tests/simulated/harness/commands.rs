//! Command pipes, exits and deadlines in the hosted harness (skein/docs/design/ui/web/testing.md, 2).
use super::{LISTENING, Play, Process, Run, run, server};
use skein_fake_person_browser::{
    Harness, Outcome, Script, Setup, Step,
    harness::{Command, Element, Names, Opens, Output, Printed, Scripted, Sent, Shown, Stdin},
    person::{self, Find},
};
use skein_io::kernel::{Done, Exit, Spawn};
use skein_lib::Duration;
use skein_world::{HostedProgram, Inherited};
use skein_world_tests::hosted::{Act, Script as Child};

const SERVER: &[u8] = b"/scripted-server";
const FIXTURE: &str = "harness_fixture";
const SECOND: Duration = Duration::from_secs(1);
const BOUND: Duration = Duration::from_millis(50);

fn command_harness(sent: Sent) -> Harness {
    let (prefix, names) = if sent == Sent::Command {
        (b"http://".as_slice(), Names::Url)
    } else {
        (LISTENING.as_bytes(), Names::Address)
    };
    Harness::new(Setup {
        server: "/scripted-server".into(),
        args: if sent == Sent::Command { vec!["held".into()] } else { vec![] },
        printed: Printed { sent, prefix, on: Output::Stderr, names },
        restart: vec![],
        downloads: None,
        chromium: "/not-a-hosted-program".into(),
        expected: vec![],
        origins: vec![],
    })
}

fn server_fixture<'a, 'p, 'l>(spawn: &Spawn, inherited: &Inherited) -> Process<'a, 'p, 'l> {
    let script = match spawn.args.as_ref() {
        [] => server(inherited, 200),
        [mode] if mode.as_ref() == b"held" => Child::child(
            inherited,
            &[
                Act::ListenAndPrint { output: pipe(inherited, 2), prefix: "http://", suffix: "/held" },
                Act::AnswerRequests(200),
            ],
        ),
        other => panic!("unexpected server arguments: {other:?}"),
    };
    Process::Child(Box::new(script))
}

fn pipe(inherited: &Inherited, child: u32) -> usize {
    inherited.pipes.iter().position(|(at, _)| *at == child).expect("inherited command pipe")
}

fn command_fixture<'a, 'p, 'l>(spawn: &Spawn, inherited: &Inherited) -> Process<'a, 'p, 'l> {
    let mut script = match spawn.args[0].as_ref() {
        b"closed" => Child::child(inherited, &[Act::ReadToEnd(pipe(inherited, 0))]),
        b"no-link" => Child::child(inherited, &[Act::Write(pipe(inherited, 2), b"fixture exits without a link\n")]),
        b"code" => {
            let code = std::str::from_utf8(&spawn.args[1]).expect("test code text").parse().expect("test code");
            Child::child(inherited, &[Act::ExitCode(code)])
        }
        // No page releases these commands; they wait until harness teardown.
        b"wait" => Child::child(inherited, &[Act::ReadSignal]),
        b"link" => Child::child(inherited, &[Act::PrintArgument { output: pipe(inherited, 1), argument: 1 }]),
        other => panic!("unexpected command mode: {other:?}"),
    };
    script.args.clone_from(&spawn.args);
    Process::Child(Box::new(script))
}

fn run_story(harness: &mut Harness, people: &mut [&mut dyn Play], browser: Scripted) -> Run {
    run(
        harness,
        people,
        &mut [],
        browser,
        [
            HostedProgram { program: SERVER.into(), make: server_fixture, instances: 1, operations: 8 },
            HostedProgram { program: FIXTURE.as_bytes().into(), make: command_fixture, instances: 2, operations: 8 },
        ],
    )
}

fn run_one(steps: impl Into<Box<[Step]>>, sent: Sent) -> Run {
    let mut person = Script::new(steps.into());
    run_story(&mut command_harness(sent), &mut [&mut person], Scripted::new())
}

fn command(args: &[&str], opens: bool) -> Command {
    Command {
        program: FIXTURE.into(),
        args: args.iter().map(std::ffi::OsString::from).collect(),
        env: vec![],
        stdin: Stdin::Closed,
        opens: opens.then(|| Opens {
            prefix: b"http://".as_slice().into(),
            on: Output::Stdout,
            keep: b"link".as_slice().into(),
        }),
        within: SECOND,
    }
}

fn exited(code: i32) -> Step {
    Step::Exited { code, within: SECOND }
}

fn assert_waiting_command_ended(run: &Run) {
    let [killed] = run.killed.as_slice() else { panic!("only the waiting command was killed: {:?}", run.killed) };
    assert_eq!(killed.exit, Exit::Signal(9));
    assert_eq!(run.children.len(), 1, "only the server exited normally");
}

#[test]
fn closed_command_stdin_receives_only_eof() {
    let run = run_one([Step::Run(command(&["closed"], false)), exited(0)], Sent::Command);
    assert_eq!(run.outcome, Outcome::Done);
    let [_, child] = run.children.as_slice() else { panic!("server and command both settled") };
    assert_eq!(child.results.first(), Some(&Ok(Done::Count(0))), "closed stdin EOF");
    assert!(child.received.is_empty(), "closed stdin has no held line");
}

#[test]
fn a_command_without_the_link_fails_naming_the_command() {
    let run = run_one([Step::Run(command(&["no-link"], true)), exited(0)], Sent::Page);
    let Outcome::Failed { step: 0, report, .. } = run.outcome else {
        panic!("missing command link fails: {:?}", run.outcome)
    };
    assert!(report.contains("harness_fixture: exited without matching link"), "{report}");
    assert!(report.contains("fixture exits without a link"), "command stderr retained: {report}");
}

#[test]
fn exited_accepts_the_commands_expected_code() {
    let run = run_one([Step::Run(command(&["code", "7"], false)), exited(7)], Sent::Page);
    assert_eq!(run.outcome, Outcome::Done);
}

#[test]
fn exited_rejects_a_different_code_naming_the_command() {
    let run = run_one([Step::Run(command(&["code", "7"], false)), exited(0)], Sent::Page);
    let Outcome::Failed { step: 1, report, .. } = run.outcome else {
        panic!("wrong exit code fails: {:?}", run.outcome)
    };
    assert!(report.contains("harness_fixture: exited with code 7, expected 0"), "{report}");
}

#[test]
fn a_live_command_at_story_end_fails_and_settles() {
    let run = run_one([Step::Run(command(&["wait"], false))], Sent::Page);
    assert_waiting_command_ended(&run);
    let Outcome::Failed { report, .. } = run.outcome else { panic!("live command fails story end: {:?}", run.outcome) };
    assert!(report.contains("harness_fixture still running at story end"), "{report}");
    assert!(!report.contains("deadline passed"), "teardown settles: {report}");
}

#[test]
fn a_person_runs_two_commands_in_turn() {
    let run = run_one(
        [Step::Run(command(&["code", "7"], false)), exited(7), Step::Run(command(&["code", "8"], false)), exited(8)],
        Sent::Page,
    );
    assert_eq!(run.outcome, Outcome::Done);
}

#[test]
fn a_second_run_while_the_command_runs_fails() {
    let run = run_one([Step::Run(command(&["wait"], false)), Step::Run(command(&["code", "0"], false))], Sent::Page);
    // Counting both normal and killed children rules out a second spawn.
    assert_waiting_command_ended(&run);
    let Outcome::Failed { step: 1, report, .. } = run.outcome else {
        panic!("overlapping commands fail: {:?}", run.outcome)
    };
    assert!(report.contains("harness_fixture: another Run before it settled"), "{report}");
}

#[test]
fn a_link_keep_and_a_selector_keep_append_to_one_sequence() {
    let mut harness = command_harness(Sent::Page);
    let mut person = Script::new(Box::new([
        Step::Run(command(&["link", "http://{address}/opened"], true)),
        Step::Keep { selector: b"input".as_slice().into(), attribute: None, name: b"field".as_slice().into() },
        exited(0),
    ]));
    let browser = Scripted::new().page(
        b"/opened",
        Shown {
            nodes: vec![],
            elements: vec![Element {
                selector: b"input".as_slice().into(),
                attributes: vec![(b"value".as_slice().into(), b"fixture value".as_slice().into())],
            }],
        },
    );
    assert_eq!(run_story(&mut harness, &mut [&mut person], browser).outcome, Outcome::Done);
    let [link, field] = harness.kept() else { panic!("exactly one link keep and one selector keep") };
    assert_eq!((link.person, link.name.as_ref()), (0, b"link".as_slice()));
    assert_eq!((field.person, field.name.as_ref()), (0, b"field".as_slice()));
    assert_eq!(field.value.as_ref(), b"fixture value");
}

#[test]
fn two_peoples_commands_have_independent_status_and_keeps() {
    let mut harness = command_harness(Sent::Page);
    let mut first = Script::new(Box::new([
        Step::Run(command(&["link", "http://{address}/first"], true)),
        Step::Person(person::Step::See {
            find: Find::exactly_named(skein_ui::Role::Heading { level: 1 }, b"First"),
            within: SECOND,
        }),
        exited(0),
    ]));
    let mut second = Script::new(Box::new([Step::Run(command(&["code", "7"], false)), exited(7)]));
    let browser = Scripted::new().page(
        b"/first",
        Shown { nodes: vec![(b"heading".as_slice().into(), b"First".as_slice().into())], elements: vec![] },
    );
    assert_eq!(run_story(&mut harness, &mut [&mut first, &mut second], browser).outcome, Outcome::Done);
    assert_eq!(harness.kept().len(), 1);
    assert_eq!(harness.kept()[0].person, 0);
    assert!(harness.kept()[0].value.ends_with(b"/first"));
}

#[test]
fn a_missing_link_deadline_fails_and_command_teardown_settles() {
    let mut cmd = command(&["wait"], true);
    cmd.within = BOUND;
    let run = run_one([Step::Run(cmd), exited(0)], Sent::Page);
    assert_waiting_command_ended(&run);
    let Outcome::Failed { step: 0, report, .. } = run.outcome else {
        panic!("missing link deadline fails: {:?}", run.outcome)
    };
    assert!(report.contains("harness_fixture: Run missing-link/start deadline passed"), "{report}");
    assert!(report.contains("bound 50000000 ns, waited 50000000 ns in simulated time"), "{report}");
    assert!(!report.contains("browser harness deadline passed"), "command teardown settles: {report}");
}

#[test]
fn an_exited_deadline_fails_and_command_teardown_settles() {
    let run = run_one([Step::Run(command(&["wait"], false)), Step::Exited { code: 0, within: BOUND }], Sent::Page);
    assert_waiting_command_ended(&run);
    let Outcome::Failed { step: 1, report, .. } = run.outcome else { panic!("exit deadline fails: {:?}", run.outcome) };
    assert!(report.contains("harness_fixture: Exited deadline passed"), "{report}");
    assert!(report.contains("bound 50000000 ns, waited 50000000 ns in simulated time"), "{report}");
    assert!(!report.contains("browser harness deadline passed"), "command teardown settles: {report}");
}
