//! Kernel entry for the program-account fake (smith/docs/design/testing.md, 4).
//! It keeps no story state; `smith_fake_claude::main` owns the scripted program.

fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    smith_fake_claude::main()
}
