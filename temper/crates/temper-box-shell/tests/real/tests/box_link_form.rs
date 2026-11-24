use temper_box_machine::terminal::LinkForm::{Bare, Osc8};
use temper_box_shell::testing::box_command::link_form;

#[test]
fn the_box_reads_its_stdout_and_term() {
    let terminal = std::fs::OpenOptions::new().read(true).write(true).open("/dev/ptmx").expect("terminal");
    let null = std::fs::File::open("/dev/null").expect("null");
    let environment = [(Box::from(b"TERM".as_slice()), Box::from(b"xterm-256color".as_slice()))];
    assert_eq!(link_form(&terminal, &environment), Osc8);
    assert_eq!(link_form(&null, &environment), Bare);
}
