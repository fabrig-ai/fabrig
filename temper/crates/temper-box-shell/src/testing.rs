//! Shared fixtures and native entry points for the box's test binaries (tools/crates/design.md, 3.2).

pub mod fake_backend_fixtures;
pub mod init_fixtures;

pub use crate::fake::created::Created;
#[must_use]
pub fn path(bytes: &[u8]) -> &std::path::Path {
    crate::path(bytes)
}

#[cfg(target_os = "linux")]
pub fn login(user: u32) -> Result<Box<[u8]>, String> {
    crate::login(user)
}

pub mod box_command {
    use skein_io::kernel::Fd;
    use std::{net::SocketAddr, num::NonZeroU16};

    pub fn held_listeners(held: Vec<(Fd, NonZeroU16)>) -> Result<Vec<(Fd, SocketAddr)>, String> {
        crate::box_command::held_listeners(held)
    }

    #[must_use]
    pub fn link_form(
        output: &impl std::io::IsTerminal,
        environment: &[crate::EnvironmentEntry],
    ) -> temper_box_machine::terminal::LinkForm {
        crate::box_command::link_form(output, environment)
    }
}

pub mod fake_backends {
    pub use crate::fake_backends::{
        ADAPTER, CHOSEN_PORTS, Chosen, Options, Prepared, bytes, chosen_ports, configured_claude, reserve_from,
        validate, write,
    };
}

pub mod init {
    pub use crate::init::{Io, Written, write, write_bare, write_with_key};
}

pub mod program_login {
    pub use crate::program_login::line;
}

#[cfg(target_os = "linux")]
pub mod scripted {
    pub use crate::scripted::{ADDRESS, choose};
}

#[cfg(target_os = "linux")]
pub mod serve {
    pub use crate::serve::{configured_agent, prepare_with_limits};
}

pub mod upgrade {
    pub use crate::upgrade::{prepare, run};
}
