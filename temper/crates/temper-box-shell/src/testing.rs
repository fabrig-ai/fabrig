//! Shared fixtures and native entry points for the box's test binaries (tools/crates/design.md, 3.2).

pub mod fake_backend_fixtures;
pub mod init_fixtures;

pub use crate::fake::created::Created;
#[must_use]
pub fn path(bytes: &[u8]) -> &std::path::Path {
    crate::path(bytes)
}

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

pub mod scripted {
    pub use crate::scripted::{ADDRESS, choose};
}

pub mod serve {
    use skein_io::kernel::Fd;
    use temper_service_shell::{Host, configuration};
    use temper_shell::directories::{Directories, EnvironmentEntry};

    use crate::Service;
    pub use crate::serve::{
        PipeKind, agents_setting_over, configured_agent, local_setting_over, operations_with_copies, pipe_kind,
        prepare_with_limits,
    };
    pub use crate::served::SERVED;

    /// Prepare as `temper-box serve` does without inherited listeners or a supervising box.
    /// The world takes the returned descriptors and supplies its own signal pipe.
    pub fn hosted(
        environment: &[EnvironmentEntry],
        working: &[u8],
        directories: Directories,
        output: impl std::io::Write + 'static,
    ) -> Result<(Host<Service>, Vec<Fd>), String> {
        const BINARY: &str = "temper-box";
        let read = configuration::read(&directories, temper_config_shell::Shape::Standalone)
            .map_err(|why| temper_service_shell::serve_error(BINARY, &why))?;
        let programs = read.programs().map_err(|why| temper_service_shell::serve_error(BINARY, &why))?;
        let mut owned = Vec::new();
        match read.prepare(|bytes| {
            crate::serve::prepare_with_store(
                BINARY,
                environment,
                working,
                &directories,
                bytes,
                &mut owned,
                false,
                &programs,
            )
        }) {
            Ok((mut service, store)) => {
                owned.extend_from_slice(service.engine_component().store().log_descriptors());
                let mut host = Host::with_binary(BINARY, service, SERVED, directories, environment);
                temper_service_shell::testing::hosted_output(&mut host, store, output);
                Ok((host, owned))
            }
            Err(why) => {
                temper_service_shell::close(owned);
                Err(temper_service_shell::serve_error(BINARY, &why))
            }
        }
    }
}

pub mod upgrade {
    pub use crate::upgrade::{prepare, run};
}
