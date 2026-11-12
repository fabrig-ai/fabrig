//! The deployment's client (temper/docs/design/cli/README.md, 2).
fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    temper_client_shell::main(&temper_version::line(temper_client_shell::NAME))
}
