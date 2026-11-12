//! The box's worker half (temper/docs/design/protocol/components.md).
use crate::directories::Directories;
use crate::{path, printed, program_login};
use temper_engine_shell::serve::{WorkerHalf, WorkerStart};
use temper_service as service;
use temper_shell::directories::EnvironmentEntry;
use temper_worker_shell::{agent, worker};
pub(crate) struct Half {
    pub environment: Box<[EnvironmentEntry]>,
}
impl WorkerHalf for Half {
    fn preflight(&self, named: &service::startup::Named) -> Result<service::settings::Agent, String> {
        agent::preflight(named)
    }
    fn alone(
        &self,
        directories: &Directories,
        bytes: &[u8],
        owned: &mut Vec<skein_io::kernel::Fd>,
    ) -> Result<service::Service, String> {
        worker::alone(directories, bytes, owned, &self.environment)
    }
    fn start(
        &self,
        named: &service::startup::Named,
        directories: &Directories,
        owned: &mut Vec<skein_io::kernel::Fd>,
        before_root: &mut dyn FnMut() -> Result<(), String>,
    ) -> Result<WorkerStart, String> {
        let agent = agent::preflight(named)?;
        before_root()?;
        let worker = named.worker.as_ref().expect("named worker");
        let user = skein_shell::effective_user();
        let runtime = worker::root(&worker.root, user, owned)?;
        let environment = &self.environment;
        let program_login =
            program_login::line(directories, &printed::running_word(environment), skein_shell::HOME.value(environment))
                .map(|line| line.into_bytes().into_boxed_slice());
        Ok(WorkerStart { agent, runtime, program_login })
    }
    fn configure(
        &self,
        service: &mut service::Service,
        agent: &service::settings::Agent,
        named: &service::startup::Named,
        directories: &Directories,
    ) -> Result<(), String> {
        worker::tune(service, &self.environment);
        worker::sources(
            service,
            agent,
            path(&named.worker.as_ref().expect("local agent checked").agent),
            path(&named.worker.as_ref().expect("local agent checked").program),
            path(&directories.cache),
            skein_shell::effective_user(),
        )
    }
}
