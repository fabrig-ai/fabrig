//! The box's binary (temper/docs/design/cli/README.md, 2).
fn main() -> std::process::ExitCode {
    skein_fail_stop::install();
    temper_box_shell::main(temper_version::VERSION, temper_version::COMMIT)
}
