//! The engine's entry point (temper/docs/design/cli/README.md, 3).
fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    temper_engine_shell::main(temper_version::VERSION, temper_version::COMMIT)
}
