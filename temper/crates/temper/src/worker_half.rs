//! The box's worker half (temper/docs/design/protocol/README.md, 5).
use crate::directories::Directories;
use crate::{agent, claude_code, path, printed, private_root, program_login};
use std::os::unix::ffi::OsStrExt;
use temper_engine_shell::serve::{WorkerHalf, WorkerStart};
use temper_service as service;
pub(crate) struct Half;
impl WorkerHalf for Half {
    fn alone(
        &self,
        directories: &Directories,
        bytes: &[u8],
        owned: &mut Vec<skein_io::kernel::Fd>,
    ) -> Result<service::Service, String> {
        crate::worker_alone::prepare(directories, bytes, owned)
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
        let root_path = path(&worker.root);
        if std::fs::symlink_metadata(root_path).is_ok() {
            private_root(root_path, user).map_err(|why| format!("application.workers[0].root: {why}"))?;
        }
        let root = skein_shell::make_root(root_path, 0o700)
            .map_err(|errno| format!("application.workers[0].root: errno {errno}"))?;
        owned.push(root);
        private_root(root_path, user).map_err(|why| format!("application.workers[0].root: {why}"))?;
        let runtime = service::WorkerRuntime { root, path: root_path.as_os_str().as_bytes().into() };
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
        if crate::workspace_reuse() {
            service.reuse_workspaces();
        }
        if let Some(directory) = crate::dump_llm() {
            service.dump_llm(directory.as_os_str().as_bytes());
        }
        configure_sources(
            service,
            agent,
            path(&named.worker.as_ref().expect("local agent checked").agent),
            path(&named.worker.as_ref().expect("local agent checked").program),
            path(&directories.cache),
            skein_shell::effective_user(),
        )
    }
}
fn configure_sources(
    service: &mut service::Service,
    agent: &service::settings::Agent,
    configuration: &std::path::Path,
    agent_program: &std::path::Path,
    cache: &std::path::Path,
    user: u32,
) -> Result<(), String> {
    if let Some(program) = agent.programs.first() {
        if agent::launcher(path(&program.directory)).as_deref() == Some(path(&program.path)) {
            claude_code::place(path(&program.path).parent().expect("launcher parent"), cache, agent_program)
                .map_err(|why| format!("programs.claude_code: {why}"))?;
        }
        // Source commands use the agent's explicit command environment.
        let environment = agent::configuration(configuration)?.service.environment.command_entries();
        let parent =
            skein_shell::open_root(path(&program.directory).parent().unwrap_or(std::path::Path::new("/"))).ok();
        let resources = service::program_sources::configured(program, environment, user, parent)
            .ok_or("workers[0].agent.configuration: program source resources exceed their bounds")?;
        service.configure_program_sources(resources).map_err(|_| "worker refused program source resources")?;
    }
    Ok(())
}
