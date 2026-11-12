//! The worker's binary (temper/docs/design/protocol/binaries.md, 5).
fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    temper_worker_shell::main(temper_version::VERSION, temper_version::COMMIT)
}
