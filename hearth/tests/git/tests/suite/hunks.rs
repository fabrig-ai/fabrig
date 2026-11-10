use hearth_git_world::{
    fixtures, generate,
    world::{self, DiffOutcome, Settings},
};
use skein_lib::Rng;
#[test]
fn generated_hunks_decode_to_their_lines_whatever_the_cuts() {
    for (input, expected) in fixtures::HUNKS {
        for seed in 0..16 {
            let stdout = generate::hunks(&mut Rng::new(seed), input);
            let DiffOutcome::Done(hunks) = world::diff(&stdout, &fixtures::SMALL, &Settings::cut(seed)) else {
                panic!("seed {seed}: valid hunks")
            };
            let lines: Vec<_> = hunks.lines.iter().map(|line| (line.kind, line.text.as_ref())).collect();
            assert_eq!(lines.as_slice(), *expected, "seed {seed}");
        }
    }
}
const EXIT_CASES: &[(hearth_protocol_git::invocation::Exit, bool)] = &[
    (hearth_protocol_git::invocation::Exit::Code(0), false),
    (hearth_protocol_git::invocation::Exit::Code(1), true),
    (hearth_protocol_git::invocation::Exit::Code(2), false),
    (hearth_protocol_git::invocation::Exit::Signal(9), false),
];
#[test]
fn only_diff_exit_one_permits_its_hunks() {
    use hearth_protocol_git::invocation::{self, Command};
    let command = Command::Diff { store: b".", before: &[0x11; 32], after: &[0x22; 32] };
    for (exit, expected) in EXIT_CASES {
        assert_eq!(invocation::succeeded(command, *exit), *expected);
    }
}

/// An outcome answers its request (hearth/docs/design/domain/render/README.md, 5.5).
#[test]
fn an_end_or_a_fault_before_the_read_answers_the_read() {
    use hearth_protocol_git::hunks::{self, Diff, Error, Event, Request};
    use skein_lib::{
        Env, Queue, Time, Wall,
        stream::{Fault, Up},
    };

    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fixtures::SMALL };
    for (outcome, error) in [(Up::End, Error::Malformed), (Up::Failed(Fault::Reset), Error::Stream(Fault::Reset))] {
        let mut machine = Diff::new(&env.limits);
        let mut above = Queue::uncapped();
        let mut below = Queue::uncapped();
        hunks::up(&mut machine, &env, outcome, &mut above, &mut below);
        assert!(above.is_empty(), "outcome waits for Read");
        assert!(below.is_empty(), "no read before Read");
        hunks::down(&mut machine, &env, Request::Read, &mut above, &mut below);
        assert_eq!(above.pop(), Some(Event::Failed(error)), "Read gets the held outcome");
        assert!(above.is_empty() && below.is_empty(), "one answer and no demand");
    }
}
