//! Endpoint declarations retained by the root at startup. This module
//! knows no destinations or credentials. Configuration is checked before the
//! root starts; each charter is checked against its declared endpoint quantities.
//! Contracts: smith/docs/design/domain/run.md, section 4; smith/docs/design/protocol/limits.md, section 2.1.

use alloc::boxed::Box;
use core::mem::size_of;

use skein_lib::Duration;

use smith_domain_run::{
    Invalid,
    charter::{Endpoint, Llm},
};

use crate::Limits;

/// One endpoint's quantities, supplied at startup and retained until root teardown.
#[derive(Clone, Debug)]
pub struct EndpointLimits {
    pub endpoint: Endpoint,
    /// Declared usable input tokens.
    pub window: u32,
    /// Declared maximum completion tokens, thinking included.
    pub output: u32,
    /// Endpoint-specific fixed schemas, provider framing and configured option bytes.
    pub protocol_allowance: u64,
    pub head: Duration,
    pub idle: Duration,
}

/// Endpoint declarations supplied at startup separately from admission capacities.
#[derive(Clone, Debug)]
pub struct Config {
    /// Unique endpoint names, at most `Limits::endpoints`.
    pub endpoints: Box<[Endpoint]>,
    /// Endpoints whose LLM a program runs (smith/docs/design/protocol/programs.md 3), each once.
    pub programs: Box<[Program]>,
    /// One limits declaration per endpoint; declarations fit `Limits::configured_endpoint_bytes`.
    pub limits: Box<[EndpointLimits]>,
}

/// One endpoint's program name, supplied at startup for turns and events (smith/docs/design/protocol/programs.md 3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub endpoint: Endpoint,
    /// The backend's name in turns and events: `claude_code`.
    pub name: Box<[u8]>,
}

impl Config {
    /// Whether declarations fit their capacities and names are unique.
    #[must_use]
    pub fn valid(&self, limits: &Limits) -> bool {
        let Ok(count) = u32::try_from(self.endpoints.len()) else {
            return false;
        };
        if count > limits.endpoints {
            return false;
        }
        for (index, endpoint) in self.endpoints.iter().enumerate() {
            for other in self.endpoints.get(index.saturating_add(1)..).unwrap_or_default() {
                if endpoint == other {
                    return false;
                }
            }
        }
        let mut bytes = 0_u64;
        for (index, program) in self.programs.iter().enumerate() {
            if !self.contains(program.endpoint) || program.name.is_empty() {
                return false;
            }
            for other in self.programs.get(index.saturating_add(1)..).unwrap_or_default() {
                if program.endpoint == other.endpoint {
                    return false;
                }
            }
            let owned = skein_lib::len_u64(program.name.len())
                .checked_add(u64::try_from(size_of::<Program>()).expect("program cell"));
            let Some(owned) = owned else {
                return false;
            };
            bytes = match bytes.checked_add(owned) {
                Some(bytes) => bytes,
                None => return false,
            };
        }
        if self.limits.len() != self.endpoints.len() {
            return false;
        }
        for (index, endpoint) in self.limits.iter().enumerate() {
            if endpoint.window == 0
                || endpoint.output == 0
                || endpoint.protocol_allowance > limits.session.protocol_allowance
                || endpoint.head == Duration::ZERO
                || endpoint.idle == Duration::ZERO
                || !self.contains(endpoint.endpoint)
            {
                return false;
            }
            bytes = match bytes.checked_add(u64::try_from(size_of::<EndpointLimits>()).expect("endpoint cell")) {
                Some(bytes) => bytes,
                None => return false,
            };
            for other in self.limits.get(index.saturating_add(1)..).unwrap_or_default() {
                if endpoint.endpoint == other.endpoint {
                    return false;
                }
            }
        }
        bytes <= limits.configured_endpoint_bytes
    }

    /// The program configured for an endpoint, if any (smith/docs/design/protocol/programs.md 3).
    #[must_use]
    #[expect(clippy::manual_find, reason = "programming-model.md 10.1 keeps domain lookups free of closures")]
    pub fn program(&self, endpoint: Endpoint) -> Option<&Program> {
        for program in &self.programs {
            if program.endpoint == endpoint {
                return Some(program);
            }
        }
        None
    }

    /// Whether this endpoint name was configured.
    pub(crate) fn contains(&self, endpoint: Endpoint) -> bool {
        self.endpoints.contains(&endpoint)
    }

    /// Progress deadlines for an admitted endpoint, retained only by the root.
    pub(crate) fn progress(&self, llm: &Llm) -> crate::session::Progress {
        for endpoint in &self.limits {
            if llm.endpoint == endpoint.endpoint {
                return crate::session::Progress { head: endpoint.head, idle: endpoint.idle };
            }
        }
        unreachable!("admission checked the endpoint")
    }

    pub(crate) fn protocol_allowance(&self, llm: &Llm) -> u64 {
        for endpoint in &self.limits {
            if llm.endpoint == endpoint.endpoint {
                return endpoint.protocol_allowance;
            }
        }
        unreachable!("admission checked the endpoint")
    }

    /// Check a charter's effective quantities against its endpoint.
    pub(crate) fn check_model(&self, llm: &Llm) -> Result<(), Invalid> {
        if !self.contains(llm.endpoint) {
            return Err(Invalid::Endpoint);
        }
        for endpoint in &self.limits {
            if llm.endpoint == endpoint.endpoint {
                if llm.window == 0 || llm.output == 0 || llm.window > endpoint.window || llm.output > endpoint.output {
                    return Err(Invalid::Llm);
                }
                if let Some(effort) = &llm.effort
                    && effort.len() > 32
                {
                    return Err(Invalid::Effort);
                }
                return Ok(());
            }
        }
        Err(Invalid::Endpoint)
    }
}
