//! The twin tool's binary (temper/docs/design/testing/tools.md, 3).
fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    temper_twin_tool::main(temper_version::VERSION, temper_version::COMMIT)
}
