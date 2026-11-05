//! Environment names remain visible without exposing values (domain/tools.md, section 5).

use alloc::boxed::Box;
use alloc::fmt::format;

use crate::{Component, Environment, Spawn, Variable};

const CANARY: &str = "canary-machine-91d2";

fn variable(name: &[u8]) -> Variable {
    Variable { name: Box::from(name), value: Box::from(CANARY.as_bytes()) }
}

fn assert_hidden(debug: &str, pretty: &str) {
    let canary_bytes = format(format_args!("{:?}", CANARY.as_bytes()));
    // Entries can prefix the value's bytes with NAME= inside the same byte list.
    let sequence = canary_bytes.trim_matches(['[', ']']);
    assert!(!debug.contains(CANARY));
    assert!(!debug.contains(sequence));
    assert!(!pretty.contains(CANARY));
    assert!(!pretty.replace([' ', '\n'], "").contains(&sequence.replace(' ', "")));
}

#[test]
fn debug_of_a_variable_shows_its_name_never_its_value() {
    let variable = variable(b"NAME");
    let debug = format(format_args!("{variable:?}"));
    let pretty = format(format_args!("{variable:#?}"));
    assert_hidden(&debug, &pretty);
    assert_eq!(debug, "Variable { name: [78, 65, 77, 69] }");
}

#[test]
fn debug_of_an_environment_shows_names_never_values() {
    // NAME= plus 19 canary bytes and B= plus the same 19 bytes total 45 bytes.
    const EXPECTED: &str = "Environment { names: [[78, 65, 77, 69], [66]], bytes: 45 }";
    let environment = Environment::encode(&[variable(b"NAME"), variable(b"B")], &super::limits())
        .expect("configured environment fits");
    let debug = format(format_args!("{environment:?}"));
    let pretty = format(format_args!("{environment:#?}"));
    assert_hidden(&debug, &pretty);
    assert_eq!(debug, EXPECTED);

    let component = Component::new(&super::limits(), environment);
    let debug = format(format_args!("{component:?}"));
    let pretty = format(format_args!("{component:#?}"));
    assert_hidden(&debug, &pretty);
    assert!(debug.contains(EXPECTED));
}

#[test]
fn debug_of_a_spawn_shows_env_names_never_values() {
    let entry = format(format_args!("NAME={CANARY}"));
    let spawn = Spawn {
        program: Box::from(b"sh".as_slice()),
        args: Box::new([Box::from(b"-c".as_slice())]),
        env: Box::new([Box::from(entry.as_bytes()), Box::from(b"BARE".as_slice())]),
        dir: Box::from(b".".as_slice()),
        pipes: Box::new([skein_io::kernel::Pipe { child: 1, way: skein_io::kernel::Way::Out, parent: None }]),
    };
    let debug = format(format_args!("{spawn:?}"));
    let pretty = format(format_args!("{spawn:#?}"));
    assert_hidden(&debug, &pretty);
    assert_eq!(
        debug,
        "Spawn { program: [115, 104], args: [[45, 99]], env: [[78, 65, 77, 69], [66, 65, 82, 69]], dir: [46], pipes: [Pipe { child: 1, way: Out, parent: None }] }"
    );
}
