//! Startup resource counts for the local shell (protocol/limits.md, sections 3–4).
//! Keeps the selected sources and fronts until construction, knows no resource
//! descriptors, and derives kernel operation counts before resource adoption.
use super::{Local, configuration_failure, derivation_refusal};
use smith_agent_service as agent;
use smith_local_service as service;

/// Product startup owners prepared together before a destination is adopted.
pub(crate) struct Admission {
    pub handed: Option<crate::credentials::Prepared>,
    pub front: Option<crate::front::Front>,
    pub producer: Option<agent::Config>,
    pub programs: Option<smith_protocol_program::Programs>,
}

/// Startup resource needs without any credential file locations.
#[derive(Clone, Copy)]
pub(crate) struct AccountNeeds {
    pub handed: bool,
    pub programs: bool,
}

impl AccountNeeds {
    pub(crate) fn prepared(prepared: Option<&crate::credentials::Prepared>) -> Self {
        Self { handed: prepared.is_some(), programs: prepared.is_some_and(has_programs) }
    }
}

impl Local {
    /// Validate service limits and derive kernel operations before adoption.
    pub(crate) fn startup_operations(
        limits: &service::Limits,
        lower_routes: u32,
        accounts: AccountNeeds,
        result_file: bool,
    ) -> Result<u32, crate::startup::Failure> {
        let mut io_limits = limits.process.io;
        let programs = accounts.programs;
        if programs {
            io_limits.sockets = io_limits
                .sockets
                .checked_add(crate::credentials::PROGRAM_SOCKETS)
                .ok_or_else(|| configuration_failure("program I/O count overflows".into()))?;
        }
        let operations = Self::kernel_operations(&io_limits, lower_routes, accounts.handed)
            .and_then(|operations| {
                operations.checked_add(if result_file { crate::compose::RESULT_FILE_OPERATIONS } else { 0 })
            })
            .ok_or_else(|| configuration_failure(derivation_refusal("local_routes")))?;
        if !service::valid(limits) {
            return Err(configuration_failure(derivation_refusal("local_limits")));
        }
        if programs {
            let mut checked = limits.clone();
            checked.process.io = io_limits;
            if !service::valid(&checked) {
                return Err(configuration_failure("invalid program check limits".into()));
            }
        }
        Ok(operations)
    }
}

pub(super) fn has_programs(prepared: &crate::credentials::Prepared) -> bool {
    prepared.locations.iter().any(|location| matches!(location.source, crate::credentials::Source::Program { .. }))
}
