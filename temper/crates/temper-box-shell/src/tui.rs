//! The box binary's TUI start (temper/docs/design/client/tui/README.md, 1 and 2).
use skein_shell::Clock;
use temper_client_shell_tui::start;

#[cfg(test)]
pub(crate) use start::{Refused, terminal_refusal};

#[expect(clippy::needless_pass_by_value, reason = "the command transfers its owned address to the TUI entry point")]
pub(crate) fn run(argument: Option<Box<[u8]>>, clock: &Clock) -> u8 {
    start::run("temper", argument.as_deref(), clock)
}
