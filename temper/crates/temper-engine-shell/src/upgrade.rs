//! Upgrade commands share serve's configuration checks (protocol/binaries.md).
use crate::{path, prefixed, stopped};
use std::os::{fd::IntoRawFd, unix::ffi::OsStrExt};
use temper_engine_service as service;
use temper_service_shell::close;
use temper_shell::directories::{self, Directories, EnvironmentEntry};

mod report;
mod scratch;
pub(crate) mod testing;
#[cfg(test)]
pub(crate) mod tests;
pub(crate) use report::job_counts;
pub use report::{lines, owner, workers_lines};

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Check(Box<[u8]>),
    Workers(Box<[u8]>),
    Back,
}

pub fn configure(
    mode: &str,
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<(Directories, Request), String> {
    let (directories, rest) = directories::resolve(arguments, environment, working).map_err(|why| why.to_string())?;
    let request = match (mode, rest.as_ref()) {
        ("--check" | "--workers", [directory]) if !directory.is_empty() && !directory.starts_with(b"-") => {
            let directory = path(working).join(path(directory));
            let directory = directory.as_os_str().as_bytes().into();
            if mode == "--workers" { Request::Workers(directory) } else { Request::Check(directory) }
        }
        ("--back", []) => Request::Back,
        (_, _) => return Err("invalid arguments".into()),
    };
    Ok((directories, request))
}

pub fn run(binary: &str, directories: &Directories, request: &Request, build_name: &'static str) -> Result<u8, String> {
    if let Request::Check(source) = request {
        return checked(binary, directories, request, source, build_name)
            .map_err(|why| prefixed(&format!("{binary} upgrade"), &why));
    }
    let (mut service, deployment) =
        stopped::prepare(binary, directories).map_err(|why| prefixed(&format!("{binary} upgrade"), &why))?;
    service.engine_component().build_name(build_name);
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
/// The check restores its copy under the limits a start would take from it
/// (configuration/limit-settings.md, 3): its engine is sized after the copy opens.
fn checked(
    binary: &str,
    directories: &Directories,
    request: &Request,
    source: &[u8],
    build_name: &'static str,
) -> Result<u8, String> {
    let mut owned = Vec::new();
    let mut made = None;
    let prepared = stopped::prepare_rehearsal(directories, build_name, |store| {
        let directory = skein_shell::open_root(path(source))
            .map_err(|errno| format!("{}: no readable deployment directory: errno {errno}", path(source).display()))?;
        owned.push(directory);
        let (scratch, held, fd) = rehearsal_scratch(binary, directories, &mut owned)?;
        made = Some((scratch, held));
        store
            .adopt_rehearsal(directory, fd, skein_shell::effective_user())
            .map_err(|_| "store refused its rehearsal roots".to_owned())
    });
    let exit =
        prepared.and_then(|(service, _)| stopped::drive(binary, directories, service, source, report(request), false));
    if let Some((scratch, held)) = made
        && let Some(line) = scratch::remove(binary, &scratch, held)
    {
        println!("{line}");
    }
    if exit.is_err() {
        close(owned);
    }
    exit
}
fn run_inner(
    binary: &str,
    directories: &Directories,
    request: &Request,
    mut service: service::Service,
    deployment: &[u8],
) -> Result<u8, String> {
    let source = match request {
        Request::Check(directory) | Request::Workers(directory) => directory.as_ref(),
        Request::Back => deployment,
    };
    let directory = skein_shell::open_root(path(source))
        .map_err(|errno| format!("{}: no readable deployment directory: errno {errno}", path(source).display()))?;
    let mut owned = vec![directory];
    let scratch = match request {
        Request::Check(_) | Request::Workers(_) => {
            let (scratch, held, fd) = match rehearsal_scratch(binary, directories, &mut owned) {
                Ok(made) => made,
                Err(why) => {
                    close(owned);
                    return Err(why);
                }
            };
            let reach = match request {
                Request::Check(_) => jig_protocol_store::Reach::Whole,
                Request::Workers(_) => jig_protocol_store::Reach::Openings,
                Request::Back => unreachable!("rehearsal branch"),
            };
            service
                .rehearse_store(directory, fd, skein_shell::effective_user(), reach)
                .expect("new rehearsal adopts roots");
            Some((scratch, held))
        }
        Request::Back => {
            service.back_store(directory, skein_shell::effective_user()).expect("new back adopts root");
            None
        }
    };
    let copy_providers = match request {
        Request::Check(_) | Request::Workers(_) => false,
        Request::Back => true,
    };
    let exit = stopped::drive(binary, directories, service, source, report(request), copy_providers);
    if let Some((scratch, held)) = scratch
        && let Some(line) = scratch::remove(binary, &scratch, held)
    {
        println!("{line}");
    }
    if exit.is_err() {
        close(owned);
    }
    exit
}
/// Sweep the cache, then make and lock this rehearsal's scratch; its store adopts the descriptor.
fn rehearsal_scratch(
    binary: &str,
    directories: &Directories,
    owned: &mut Vec<skein_io::kernel::Fd>,
) -> Result<(std::path::PathBuf, std::fs::File, skein_io::kernel::Fd), String> {
    for line in scratch::sweep(binary, path(&directories.cache)) {
        println!("{line}");
    }
    let (scratch, file) = scratch::make(path(&directories.cache))?;
    // The store owns the adopted descriptor; its clone holds the lock through removal.
    let held = match file.try_clone().map_err(|why| scratch::failure(&why)) {
        Ok(held) => held,
        Err(why) => {
            if let Some(line) = scratch::remove(binary, &scratch, file) {
                println!("{line}");
            }
            return Err(why);
        }
    };
    let fd = skein_io::kernel::Fd::new(file.into_raw_fd());
    owned.push(fd);
    Ok((scratch, held, fd))
}
fn report(request: &Request) -> temper_service_shell::host::UpgradeReport<service::Service> {
    match request {
        Request::Workers(_) => |service, binary, directories, source| {
            Some(workers_lines(binary, service.upgrade_report()?, directories, source))
        },
        Request::Check(_) | Request::Back => {
            |service, binary, directories, source| Some(lines(binary, service.upgrade_report()?, directories, source))
        }
    }
}
