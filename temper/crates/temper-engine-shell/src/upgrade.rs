//! Upgrade commands share serve's configuration checks (protocol/binaries.md).
use crate::{path, prefixed, serve::WorkerHalf, setup};
use skein_shell::{Clock, Config, Kernel};
use std::os::unix::ffi::OsStrExt;
use temper_service::{self as service, settings::Seeds};
use temper_service_shell::{close, jig_directories};
use temper_shell::directories::{self, Directories, EnvironmentEntry};

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
fn prepare(
    binary: &str,
    directories: &Directories,
    worker: Option<&dyn WorkerHalf>,
) -> Result<(service::Service, Box<[u8]>), String> {
    let bytes = directories.read()?;
    let mut named = setup::check(&bytes, &jig_directories(directories))?.named;
    if named.worker.is_some() && worker.is_none() {
        return Err("temper.json names a worker, and temper-engine runs none".into());
    }
    let agent =
        if named.worker.is_some() { Some(worker.expect("checked worker half").preflight(&named)?) } else { None };
    let crate::serve::Inputs { files, google, bundle } =
        crate::serve::before_root(&mut named, skein_shell::effective_user())?;
    let roots = crate::serve::trust(None)?;
    let github = crate::serve::github_runtime(binary, named.machine.github.as_ref(), &roots, crate::address)?;
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 1_048_576, depth: 32 },
    )
    .expect("named checked document");
    let engine =
        service::startup::engine(&document, crate::member(&document, 0, b"engine"), &jig_directories(directories))
            .expect("named checked engine");
    let accounts = crate::serve::accounts(&engine)?;
    let drawn = skein_shell::crypto_seed().map_err(|errno| format!("cryptographic seed: errno {errno}"))?;
    let mut installation = [0; 16];
    installation.copy_from_slice(drawn.expose().get(..16).expect("32-byte seed"));
    let mut service = service::startup::restoring_with(
        &bytes,
        &jig_directories(directories),
        &files,
        agent.as_ref(),
        &Seeds {
            seed: skein_shell::seed().map_err(|errno| format!("random seed: errno {errno}"))?,
            crypto: drawn,
            deployment: installation,
            installation,
            link_secret: None,
            local_secret: None,
        },
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

pub fn run(
    binary: &str,
    directories: &Directories,
    request: &Request,
    worker: Option<&dyn WorkerHalf>,
) -> Result<u8, String> {
    run_inner(binary, directories, request, worker).map_err(|why| prefixed(&format!("{binary} upgrade"), &why))
}
fn run_inner(
    binary: &str,
    directories: &Directories,
    request: &Request,
    worker: Option<&dyn WorkerHalf>,
) -> Result<u8, String> {
    let (mut service, deployment) = prepare(binary, directories, worker)?;
    let source = match request {
        Request::Check(directory) => directory,
        Request::Back => &deployment,
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
        source.clone(),
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
