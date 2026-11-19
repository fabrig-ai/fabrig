//! The probe's observations and spawn refusals, through the machine seam.
use crate::probe::{self, Answer as Observation, Ids, Question, Report};
use crate::{How, Item, Landlock, Machine, Opened};
use skein_io::kernel::{Error, Op, Pipe, Reach, Rights, Rule, Spawn, Submit, Unenforced, Way};
use skein_lib::{Queue, Token};
use skein_sim::{Ask, Call, Config, Handle, Program, Reply, Sim};

fn laid() -> (Machine, Opened) {
    let mut machine = Machine::new();
    let root = machine.lay(&[
        Item::directory(b"granted"),
        Item::file(b"granted/file", b"data"),
        Item::directory(b"granted/inner"),
        Item::file(b"granted/inner/file", b"data"),
        Item::link(b"granted/out", b"../canary"),
        Item::file(b"granted/run", b"").mode(0o755),
        Item::file(b"granted/locked", b"").mode(0),
        Item::file(b"canary", b"canary"),
        Item::link(b"granted/in", b"inner/file"),
    ]);
    (machine, root)
}

fn call(root: Opened, restricted: bool, questions: &[Question]) -> Call {
    let mut sim = Sim::new(7, Config::calm());
    let parent = sim.spawn_process();
    let root = sim.root(parent, Handle::new(root.raw()));
    let reach = restricted.then(|| {
        Box::new(Reach {
            abi: Reach::ABI,
            rules: Box::new([Rule { root, path: Box::from(b"granted".as_slice()), rights: Rights::FULL }]),
        })
    });
    let mut submissions = Queue::with_capacity(1);
    submissions.push(Submit {
        op: Token::new(1),
        kind: Op::Spawn {
            spawn: Box::new(Spawn {
                root,
                reach,
                program: Box::from(b"/fixtures/reach_probe".as_slice()),
                args: probe::arguments(questions),
                env: Box::new([]),
                dir: Box::from(b"granted".as_slice()),
                pipes: Box::new([
                    Pipe { child: 9, way: Way::Out, parent: None },
                    Pipe { child: 0, way: Way::Null, parent: None },
                ]),
            }),
        },
    });
    sim.submit(parent, &mut submissions);
    let mut calls = Queue::with_capacity(1);
    sim.calls(&mut calls);
    calls.pop().expect("spawn reaches machine")
}

fn answer(machine: &mut Machine, call: Call) -> Result<Reply, Error> {
    let mut answers = Queue::with_capacity(1);
    crate::step(machine, call, &mut answers);
    answers.pop().expect("one answer").result
}

fn report(machine: &mut Machine, call: Call) -> Report {
    let Reply::Program(Program::Report { output, bytes }) = answer(machine, call).expect("probe runs") else {
        panic!("probe report")
    };
    assert_eq!(output, 9);
    probe::decode(&bytes).expect("probe report text")
}

#[test]
fn restricted_probe_reports_access_privilege_signals_and_descriptors() {
    use Observation::{Granted, Refused};
    skein_rows::check(
        [
            ("read granted", (true, vec![Question::Read(b"file".as_slice().into())], vec![Granted])),
            ("write granted", (true, vec![Question::Write(b"file".as_slice().into())], vec![Granted])),
            ("list granted", (true, vec![Question::List(b".".as_slice().into())], vec![Granted])),
            ("execute granted", (true, vec![Question::Execute(b"run".as_slice().into())], vec![Granted])),
            (
                "read outside",
                (true, vec![Question::Read(b"../canary".as_slice().into())], vec![Refused(Error::Permission)]),
            ),
            ("list outside", (true, vec![Question::List(b"..".as_slice().into())], vec![Refused(Error::Permission)])),
            ("link outside", (true, vec![Question::Read(b"out".as_slice().into())], vec![Refused(Error::Permission)])),
            ("link inside", (true, vec![Question::Read(b"in".as_slice().into())], vec![Granted])),
            (
                "lookup first",
                (true, vec![Question::Read(b"missing".as_slice().into())], vec![Refused(Error::NotFound)]),
            ),
            (
                "owner still checked",
                (true, vec![Question::Read(b"locked".as_slice().into())], vec![Refused(Error::Permission)]),
            ),
            (
                "execute bit",
                (true, vec![Question::Execute(b"file".as_slice().into())], vec![Refused(Error::Permission)]),
            ),
            ("no privilege", (true, vec![Question::NoNewPrivs], vec![Observation::NoNewPrivs(true)])),
            ("parent signal", (true, vec![Question::SignalParent], vec![Refused(Error::Permission)])),
            ("set-id", (true, vec![Question::SetId(b"run".as_slice().into())], vec![Observation::Ids(Ids::Kept)])),
            ("descriptors", (true, vec![Question::Descriptors], vec![Observation::Descriptors(Box::new([0, 9]))])),
            (
                "unrestricted",
                (
                    false,
                    vec![Question::Read(b"../canary".as_slice().into()), Question::SignalParent, Question::NoNewPrivs],
                    vec![Granted, Granted, Observation::NoNewPrivs(false)],
                ),
            ),
        ],
        |(restricted, questions, expected)| {
            let (mut machine, root) = laid();
            let actual = report(&mut machine, call(root, restricted, &questions));
            assert_eq!(actual.answers.as_ref(), expected);
        },
    );
}

#[test]
fn nested_reach_intersects_layers_without_narrowing_the_parent() {
    skein_rows::check(
        [
            ("child narrows", (Rights::FULL, Rights::READ)),
            ("child cannot broaden parent", (Rights::READ, Rights::FULL)),
        ],
        |(outer, inner)| {
            let (mut machine, root) = laid();
            let asked = [
                Question::Nest {
                    rules: Box::new([probe::Rule { path: b"inner".as_slice().into(), rights: inner }]),
                    questions: Box::new([
                        Question::Read(b"inner/file".as_slice().into()),
                        Question::Read(b"file".as_slice().into()),
                        Question::Write(b"inner/file".as_slice().into()),
                        Question::NoNewPrivs,
                    ]),
                },
                Question::Read(b"file".as_slice().into()),
            ];
            let mut call = call(root, true, &asked);
            let Ask::Spawn { reach, .. } = &mut call.ask else { panic!("spawn") };
            reach.as_mut().expect("reach").rules[0].rights = outer;
            assert_eq!(
                report(&mut machine, call),
                Report {
                    answers: Box::new([
                        Observation::Nested(Report {
                            answers: Box::new([
                                Observation::Granted,
                                Observation::Refused(Error::Permission),
                                Observation::Refused(Error::Permission),
                                Observation::NoNewPrivs(true)
                            ])
                        }),
                        Observation::Granted,
                    ])
                }
            );
        },
    );
}

#[test]
fn reach_refusals_precede_program_selection_in_linux_order() {
    skein_rows::check(
        [
            ("absent", (Landlock::Absent, 6, b".".as_slice(), Unenforced::Absent)),
            ("disabled", (Landlock::Disabled, 6, b".".as_slice(), Unenforced::Disabled)),
            ("older ABI", (Landlock::Abi(5), 6, b".".as_slice(), Unenforced::Abi { found: 5, needs: 6 })),
            ("newer record", (Landlock::Abi(6), 7, b".".as_slice(), Unenforced::Abi { found: 6, needs: 7 })),
            (
                "missing second rule",
                (Landlock::Abi(6), 6, b"missing".as_slice(), Unenforced::Rule { index: 1, errno: 2_i32 }),
            ),
            ("absent before path", (Landlock::Absent, 6, b"missing".as_slice(), Unenforced::Absent)),
            ("escape", (Landlock::Abi(6), 6, b"../outside".as_slice(), Unenforced::Rule { index: 1, errno: 18_i32 })),
            (
                "not directory",
                (Landlock::Abi(6), 6, b"canary/child".as_slice(), Unenforced::Rule { index: 1, errno: 20_i32 }),
            ),
        ],
        |(landlock, abi, path, expected)| {
            let (mut machine, root) = laid();
            machine.set_landlock(landlock);
            let mut call = call(root, true, &[]);
            let Ask::Spawn { reach, program, .. } = &mut call.ask else { panic!("spawn") };
            let reach = reach.as_mut().expect("reach");
            reach.abi = abi;
            reach.rules = Box::new([
                reach.rules[0].clone(),
                skein_sim::machine::Rule { root: Handle::new(root.raw()), path: path.into(), rights: Rights::FULL },
            ]);
            *program = b"unknown-program".as_slice().into();
            let handles = machine.open_handles();
            assert_eq!(answer(&mut machine, call), Err(Error::Reach(expected)));
            assert_eq!(machine.open_handles(), handles, "a refusal opens nothing");
        },
    );
}

#[test]
fn file_rules_and_resolved_ancestry_do_not_grant_siblings() {
    let (mut machine, root) = laid();
    let mut call =
        call(root, true, &[Question::Read(b"file".as_slice().into()), Question::Read(b"inner/file".as_slice().into())]);
    let Ask::Spawn { reach, .. } = &mut call.ask else { panic!("spawn") };
    let reach = reach.as_mut().expect("reach");
    reach.rules[0].path = b"granted/file".as_slice().into();
    reach.rules[0].rights = Rights::FULL;
    assert_eq!(
        report(&mut machine, call).answers.as_ref(),
        [Observation::Granted, Observation::Refused(Error::Permission)]
    );
    let granted = machine.open(root, b"granted", How::Directory).expect("cwd handle");
    let mut from_cwd = self::call(root, true, &[Question::Read(b"../canary".as_slice().into())]);
    let Ask::Spawn { root, dir, .. } = &mut from_cwd.ask else { panic!("spawn") };
    *root = Handle::new(granted.raw());
    *dir = b".".as_slice().into();
    assert_eq!(report(&mut machine, from_cwd).answers.as_ref(), [Observation::Refused(Error::Permission)]);
}
