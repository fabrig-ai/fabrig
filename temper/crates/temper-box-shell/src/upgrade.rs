//! The box's standalone preflight for an upgrade, without constructing its worker (protocol/binaries.md).
use std::{net::SocketAddr, path::Path};
use temper_engine_shell::{serve as engine, upgrade::Request};
use temper_service as service;
use temper_service_shell::{configuration, jig_directories};
use temper_shell::directories::Directories;

type Prepared = (temper_engine_service::Service, Box<[u8]>);

pub fn run(directories: &Directories, request: &Request) -> Result<u8, String> {
    run_with(directories, request, prepare, temper_engine_shell::upgrade::run_prepared)
}
fn run_with(
    directories: &Directories,
    request: &Request,
    prepare: impl FnOnce(&Directories) -> Result<Prepared, String>,
    run: impl FnOnce(&str, &Directories, &Request, temper_engine_service::Service, &[u8]) -> Result<u8, String>,
) -> Result<u8, String> {
    let (service, deployment) = prepare(directories).map_err(|why| crate::prefixed("temper-box upgrade", &why))?;
    run("temper-box", directories, request, service, &deployment)
}

pub fn prepare(directories: &Directories) -> Result<Prepared, String> {
    prepare_with(directories, &Native)
}

/// The standalone preflight's supplied facts (temper/docs/design/testing/shells.md, 3.10).
trait Host {
    fn read(&self, directories: &Directories) -> Result<configuration::Read, String>;
    fn executable(&self, path: &Path) -> bool;
    fn address(&self) -> Result<SocketAddr, String>;
    fn given(&self, directories: &Directories, address: SocketAddr) -> service::stock::Given;
    fn agent(
        &self,
        bytes: &[u8],
        directories: &Directories,
        environment: &[crate::EnvironmentEntry],
        programs: &temper_config_shell::Programs,
    ) -> Result<Option<temper_worker_shell::agent::Prepared>, String>;
    fn files(&self, named: &mut service::startup::Named) -> Result<engine::Inputs, String>;
    fn runtimes(&self, named: &service::startup::Named) -> Result<engine::Runtimes, String>;
    fn accounts(&self, config: &jig_protocol_config::Engine) -> Result<temper_engine_service::AccountsRuntime, String>;
    fn seeds(&self) -> Result<service::settings::Seeds, String>;
}

struct Native;
impl Host for Native {
    fn read(&self, directories: &Directories) -> Result<configuration::Read, String> {
        configuration::read(directories, temper_config_shell::Shape::Standalone)
    }
    fn executable(&self, path: &Path) -> bool {
        temper_shell::lookup::is_executable(path)
    }
    fn address(&self) -> Result<SocketAddr, String> {
        crate::scripted::address(&[])
    }
    fn given(&self, directories: &Directories, address: SocketAddr) -> service::stock::Given {
        crate::scripted::given(directories, &service::stock::Given::Stock, address)
    }
    fn agent(
        &self,
        bytes: &[u8],
        directories: &Directories,
        environment: &[crate::EnvironmentEntry],
        programs: &temper_config_shell::Programs,
    ) -> Result<Option<temper_worker_shell::agent::Prepared>, String> {
        crate::serve::configured_agent(bytes, directories, environment, programs)
    }
    fn files(&self, named: &mut service::startup::Named) -> Result<engine::Inputs, String> {
        engine::before_root(
            &mut named.files,
            &mut named.asked_files,
            named.sign_in.as_ref(),
            named.bundle.as_deref(),
            named.bundle_limits,
            skein_shell::effective_user(),
            false,
        )
    }
    fn runtimes(&self, named: &service::startup::Named) -> Result<engine::Runtimes, String> {
        engine::runtimes(named.test_github.as_ref())
    }
    fn accounts(&self, config: &jig_protocol_config::Engine) -> Result<temper_engine_service::AccountsRuntime, String> {
        engine::accounts(config)
    }
    fn seeds(&self) -> Result<service::settings::Seeds, String> {
        engine::seeds(None, None)
    }
}

fn prepare_with(directories: &Directories, host: &impl Host) -> Result<Prepared, String> {
    let read = host.read(directories)?;
    let programs = read.programs_with(|path| host.executable(path))?;
    read.prepare(|bytes| {
        let address = host.address()?;
        let given = host.given(directories, address);
        let mut named = crate::setup::check_with(bytes, &jig_directories(directories), &given)?.named;
        let environment = [(Box::from(crate::scripted::ADDRESS), address.to_string().into_bytes().into_boxed_slice())];
        let agent = host.agent(bytes, directories, &environment, &programs)?;
        let engine::Inputs { files, google, bundle } = host.files(&mut named)?;
        let engine::Runtimes { roots, github } = host.runtimes(&named)?;
        let config = engine::settings(bytes, directories, &given);
        let accounts = host.accounts(&config)?;
        let seeds = host.seeds()?;
        let mut service = service::startup::restoring_engine_with(
            bytes,
            &jig_directories(directories),
            &files,
            agent.as_ref().map(|prepared| &prepared.agent),
            &seeds,
            Some(accounts),
            &given,
        )
        .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
        service.configure_forge_runtime(github, roots);
        service.configure_google_endpoints(google);
        if let Some(bundle) = bundle {
            service.adopt_bundle(bundle);
        }
        Ok((service, named.deployment))
    })
}

#[cfg(test)]
mod tests;
