//! The worker's binary (temper/docs/design/protocol/README.md, 4.5).
fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    temper_worker_shell::main()
}
