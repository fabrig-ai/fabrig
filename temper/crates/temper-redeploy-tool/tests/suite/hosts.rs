//! Only this seam test starts host prefixes, and those are sh alone (redeploy.md, 7).
use temper_redeploy_tool::{
    fake::description,
    hosts::{Hosts, Part, Processes, quote},
};

#[test]
fn processes_pass_one_line_and_captured_environment_and_spawn_all_before_waiting() {
    let scratch = skein_scratch::Scratch::new("redeploy-processes");
    let mut description = description(1);
    let environment = vec![(b"CAPTURED".to_vec().into_boxed_slice(), b"it's here".to_vec().into_boxed_slice())];
    let mut hosts = Processes::new(&description, &environment);
    let answer = hosts.run(Part::Engine, "printf '%s' \"$CAPTURED\"; printf '%s' problem >&2; read nothing; exit 7");
    assert_eq!(answer.code, Some(7));
    assert_eq!(answer.stdout, "it's here");
    assert_eq!(answer.stderr, "problem");
    let a = quote(scratch.path().join("a").to_str().unwrap());
    let b = quote(scratch.path().join("b").to_str().unwrap());
    // A bounded shell-only rendezvous: neither can finish successfully unless both were spawned.
    let first = format!(
        "printf a > {a}; n=0; while [ ! -f {b} ]; do n=$((n+1)); [ $n -lt 100000 ] || exit 9; done; printf first"
    );
    let second = format!(
        "printf b > {b}; n=0; while [ ! -f {a} ]; do n=$((n+1)); [ $n -lt 100000 ] || exit 9; done; printf second"
    );
    let answers = hosts.run_each(&[(Part::Engine, first), (Part::Worker(0), second)]);
    assert_eq!(
        answers.iter().map(|answer| (answer.code, answer.stdout.as_str())).collect::<Vec<_>>(),
        vec![(Some(0), "first"), (Some(0), "second")]
    );
    description.engine.run = vec![scratch.path().join("cannot-spawn").to_string_lossy().into_owned()];
    let mut hosts = Processes::new(&description, &environment);
    let answer = hosts.run(Part::Engine, "not run");
    assert_eq!(answer.code, None);
    assert!(answer.stderr.contains("cannot-spawn"));
}
