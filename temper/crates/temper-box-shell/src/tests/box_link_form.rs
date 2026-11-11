use crate::box_command::link_form;
use temper_box_machine::terminal::LinkForm::{Bare, Osc8};

#[test]
fn the_box_reads_its_stdout_and_term() {
    let terminal = std::fs::OpenOptions::new().read(true).write(true).open("/dev/ptmx").expect("terminal");
    let scratch = skein_scratch::Scratch::new("box-link-form");
    let file = std::fs::File::create(scratch.path().join("stdout")).expect("scratch stdout");
    let null = std::fs::File::open("/dev/null").expect("null");
    for (stdout, environment, expected) in [
        (&terminal, vec!["TERM=xterm-256color"], Osc8),
        (&terminal, vec!["HOME=/h", "TERMCAP=dumb"], Osc8),
        (&terminal, vec!["TERM=dumb"], Bare),
        (&file, vec!["TERM=xterm-256color"], Bare),
        (&null, vec!["TERM=xterm-256color"], Bare),
    ] {
        let environment: Vec<_> = environment
            .iter()
            .map(|entry| {
                let (name, value) = entry.split_once('=').expect("environment pair");
                (Box::from(name.as_bytes()), Box::from(value.as_bytes()))
            })
            .collect();
        assert_eq!(link_form(stdout, &environment), expected, "environment={environment:?}");
    }
}
