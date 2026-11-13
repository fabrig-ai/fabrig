//! Upgrade commands share serve's configuration checks (protocol/binaries.md).
use crate::{path, prefixed, setup};
use skein_shell::{Clock, Config, Kernel};
use std::os::unix::ffi::OsStrExt;
use temper_engine_service as service;
use temper_service_shell::{close, jig_directories};
use temper_shell::directories::{self, Directories, EnvironmentEntry};

mod report;
#[cfg(test)]
mod tests;
pub use report::{lines, owner};

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Check(Box<[u8]>),
    Back,
}

pub fn configure(
    check: bool,
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<(Directories, Request), String> {
    let (directories, rest) = directories::resolve(arguments, environment, working).map_err(|why| why.to_string())?;
    let request = match (check, rest.as_ref()) {
        (true, [directory]) if !directory.is_empty() && !directory.starts_with(b"-") => {
            let directory = path(working).join(path(directory));
            Request::Check(directory.as_os_str().as_bytes().into())
        }
        (false, []) => Request::Back,
        (true | false, _) => return Err("invalid arguments".into()),
    };
    Ok((directories, request))
}

/// No worker root, listener or external system is started.
fn prepare(binary: &str, directories: &Directories) -> Result<(service::Service, Box<[u8]>), String> {
    let bytes = directories.read()?;
    if matches!(
        service::startup::nearest(&bytes),
        Some(service::startup::Shape::Standalone | service::startup::Shape::WorkerAlone)
    ) {
        return Err(format!("temper.json names a worker, and {binary} runs none"));
    }
    let mut named = setup::check(&bytes, &jig_directories(directories))?.named;
    let crate::serve::Inputs { files, google, bundle } = crate::serve::before_root(
        &mut named.files,
        &mut named.asked_files,
        named.sign_in.as_ref(),
        named.bundle.as_deref(),
        named.bundle_limits,
        skein_shell::effective_user(),
        false,
    )?;
    let crate::serve::Runtimes { roots, github } = crate::serve::runtimes(binary, &named.machine)?;
    let engine = crate::serve::settings(&bytes, directories, &service::stock::Given::Stock);
    let accounts = crate::serve::accounts(&engine)?;
    let seeds = crate::serve::seeds(None, None)?;
    let mut service = service::startup::restoring_with(
        &bytes,
        &jig_directories(directories),
        &files,
        None,
        &seeds,
        Some(accounts),
        &service::stock::Given::Stock,
    )
    .map_err(|why| String::from_utf8_lossy(&service::startup::lines(&why)).into_owned())?;
    service.configure_forge_runtime(github, roots);
    service.configure_google_endpoints(google);
    if let Some(bundle) = bundle {
        service.adopt_bundle(bundle);
    }
    Ok((service, named.deployment))
}

pub fn run(binary: &str, directories: &Directories, request: &Request) -> Result<u8, String> {
    let (service, deployment) =
        prepare(binary, directories).map_err(|why| prefixed(&format!("{binary} upgrade"), &why))?;
    run_prepared(binary, directories, request, service, &deployment)
}
/// Drive an engine checked and prepared by its binary, without a worker or listeners.
pub fn run_prepared(
    binary: &str,
    directories: &Directories,
    request: &Request,
    service: service::Service,
    deployment: &[u8],
) -> Result<u8, String> {
    run_inner(binary, directories, request, service, deployment)
        .map_err(|why| prefixed(&format!("{binary} upgrade"), &why))
}
fn run_inner(
    binary: &str,
    directories: &Directories,
    request: &Request,
    mut service: service::Service,
    deployment: &[u8],
) -> Result<u8, String> {
    let source = match request {
        Request::Check(directory) => directory.as_ref(),
        Request::Back => deployment,
    };
    let directory = skein_shell::open_root(path(source))
        .map_err(|errno| format!("{}: no readable deployment directory: errno {errno}", path(source).display()))?;
    let mut owned = vec![directory];
    let scratch = match request {
        Request::Check(_) => {
            let seed = match skein_shell::seed() {
                Ok(seed) => seed,
                Err(errno) => {
                    close(owned);
                    return Err(format!("scratch seed: errno {errno}"));
                }
            };
            let scratch = path(&directories.cache).join(format!("upgrade-{seed:016x}"));
            let fd = match skein_shell::make_root(&scratch, 0o700) {
                Ok(fd) => fd,
                Err(errno) => {
                    close(owned);
                    return Err(format!("scratch: errno {errno}"));
                }
            };
            owned.push(fd);
            service.rehearse_store(directory, fd, skein_shell::effective_user()).expect("new rehearsal adopts roots");
            Some(scratch)
        }
        Request::Back => {
            service.back_store(directory, skein_shell::effective_user()).expect("new back adopts root");
            None
        }
    };
    let mut kernel = match Kernel::open(Config { operations: service.operations() }) {
        Ok(kernel) => kernel,
        Err(why) => {
            close(owned);
            return Err(why.to_string());
        }
    };
    let mut host = temper_service_shell::Host::for_upgrade(
        binary,
        service,
        crate::served::SERVED,
        directories.clone(),
        source.into(),
        |service, binary, directories, source| Some(lines(binary, service.upgrade_report()?, directories, source)),
    );
    let exit = skein_shell::drive(&mut kernel, &Clock::new(), &mut host);
    if let Some(scratch) = scratch {
        let _removed = std::fs::remove_dir(scratch);
    }
    Ok(match exit {
        skein_io::kernel::Exit::Code(code) => code,
        skein_io::kernel::Exit::Signal(_) => 1,
    })
}
