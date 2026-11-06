use crate::support;
use temper_forge_setup::prepare::Exit;
use temper_forge_setup_shell::command;
#[test]
fn a_command_ends_with_its_status_and_both_streams() {
    let scratch = support::Scratch::new("command_ends");
    let spec = support::script(scratch.path(), "echo out; echo err >&2; exit 3");
    let ran = command::run_one(&support::io(), &spec).expect("it runs");
    assert_eq!((ran.exit, ran.stdout.expose(), ran.stderr.expose()), (Exit::Code(3), &b"out\n"[..], &b"err\n"[..]));
    scratch.remove();
}
#[test]
fn output_past_the_bound_is_cut_and_the_command_ends() {
    let scratch = support::Scratch::new("command_bounded");
    let mut spec = support::script(scratch.path(), "head -c 100000 /dev/zero; head -c 100000 /dev/zero >&2");
    spec.output = 17;
    let ran = command::run_one(&support::io(), &spec).expect("it runs");
    assert_eq!((ran.exit, ran.stdout.expose(), ran.stderr.expose()), (Exit::Code(0), &[0; 17][..], &[0; 17][..]));
    scratch.remove();
}
#[test]
fn a_program_that_is_not_there_is_reported_not_run() {
    let scratch = support::Scratch::new("command_missing");
    let mut spec = support::script(scratch.path(), "exit 0");
    spec.program = b"/no-such-temper-program".as_slice().into();
    assert!(matches!(command::run_one(&support::io(), &spec), Err(command::Error::Spawn(skein_io::Error::Other))));
    scratch.remove();
}

#[test]
fn a_source_command_deadline_closes_a_live_child_and_settles_both_pipes() {
    let scratch = support::Scratch::new("command_deadline");
    let spec = support::script(scratch.path(), "sleep 0.1");
    let until = skein_shell::Clock::new().now().now.saturating_add(skein_lib::Duration::from_millis(10));
    let answer = command::run_until(&support::io(), &spec, until);
    assert!(matches!(answer, Err(command::Error::Kernel(ref why)) if why == "source deadline expired"));
    scratch.remove();
}
