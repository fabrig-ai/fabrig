//! The deployment's client entry point (temper/docs/design/protocol/README.md, section 4).
#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    skein_fail_stop::install();
    let arguments = skein_shell::arguments();
    if arguments.as_ref() == [Box::from(b"--version".as_slice())] {
        print!("{}", temper_client::version());
        return ExitCode::SUCCESS;
    }
    let words = arguments.iter().map(|word| String::from_utf8_lossy(word)).collect::<Vec<_>>().join(" ");
    let words = if arguments.is_empty() { String::new() } else { format!(" {words}") };
    eprintln!("{}: no command{words}; {} --help lists them", temper_client::NAME, temper_client::NAME);
    ExitCode::FAILURE
}
