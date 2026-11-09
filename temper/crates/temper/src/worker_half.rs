//! The box's worker half (temper/docs/design/protocol/README.md, 5).
use crate::directories::Directories;
use crate::{path, printed, program_login};
use temper_engine_shell::serve::{WorkerHalf, WorkerStart};
use temper_service as service;
use temper_worker_shell::{agent, worker};
pub(crate) struct Half;
impl WorkerHalf for Half {
    fn alone(
        &self,
        directories: &Directories,
        bytes: &[u8],
        owned: &mut Vec<skein_io::kernel::Fd>,
    ) -> Result<service::Service, String> {
        worker::alone(directories, bytes, owned)
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "cli C2: the process environment configures worker startup; temper-box's table will pass it"
    )]
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
        let environment = skein_shell::environment();
        let program_login = program_login::line(
            directories,
            &printed::running_word(&environment),
            environment.iter().find(|(name, _)| name.as_ref() == b"HOME").map(|(_, value)| value.as_ref()),
        )
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
        worker::tune(service);
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
