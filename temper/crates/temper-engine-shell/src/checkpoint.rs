//! Checkpoints over a stopped deployment (temper/docs/design/protocol/checkpoints.md, 2).
use crate::{path, prefixed, stopped};
use jig_protocol_store::{CheckpointName, host::Adoption};
use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
use temper_engine_service as service;
use temper_shell::directories::{self, Directories, EnvironmentEntry};

/// The absolute checkpoint directory supplied to the held child.
pub const DIRECTORY: skein_shell::Variable = skein_shell::Variable {
    name: "TEMPER_CHECKPOINT",
    set: skein_shell::Set::Public,
    form: "DIRECTORY",
    line: "The held checkpoint's absolute directory, set for checkpoint --hold's child.",
    absent: "Set for every held command; an inherited value is replaced.",
    notes: "The checkpoint lock stays held until the child is closed.",
    way: skein_shell::Way::Sets,
};

mod report;
pub use report::lines;
#[cfg(test)]
pub(crate) mod tests;

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Take,
    Hold { name: CheckpointName, command: Box<[Box<[u8]>]> },
    Remove(CheckpointName),
}

pub fn configure(
    arguments: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
) -> Result<(Directories, Request), String> {
    let separator = arguments.iter().position(|word| word.as_ref() == b"--");
    let (options, command) = match separator {
        Some(at) => (&arguments[..at], Some(&arguments[at + 1..])),
        None => (arguments, None),
    };
    let (directories, rest) = directories::resolve(options, environment, working).map_err(|why| why.to_string())?;
    let invalid = || "invalid arguments".to_string();
    let request = match (rest.as_ref(), command) {
        ([], None) => Request::Take,
        ([flag, name], None) if flag.as_ref() == b"--remove" => {
            Request::Remove(CheckpointName::parse(name).ok_or_else(invalid)?)
        }
        ([flag, name], Some(command))
            if flag.as_ref() == b"--hold" && command.first().is_some_and(|word| !word.is_empty()) =>
        {
            Request::Hold { name: CheckpointName::parse(name).ok_or_else(invalid)?, command: command.into() }
        }
        _ => return Err(invalid()),
    };
    Ok((directories, request))
}

pub fn run(
    binary: &str,
    directories: &Directories,
    request: &Request,
    environment: &[EnvironmentEntry],
    working: &[u8],
    build_name: &'static str,
) -> Result<u8, String> {
    let (mut service, deployment) =
        stopped::prepare(binary, directories).map_err(|why| prefixed(&format!("{binary} checkpoint"), &why))?;
    service.engine_component().build_name(build_name);
    run_prepared(binary, directories, request, environment, working, service, &deployment)
}

/// The box supplies its standalone configuration without constructing a worker.
pub fn run_prepared(
    binary: &str,
    directories: &Directories,
    request: &Request,
    environment: &[EnvironmentEntry],
    working: &[u8],
    mut service: service::Service,
    deployment: &[u8],
) -> Result<u8, String> {
    let mut owned = Vec::new();
    let mut working_root = None;
    let result = (|| {
        let directory = skein_shell::open_root(path(deployment)).map_err(|errno| {
            format!("{}: no readable deployment directory: errno {errno}", path(deployment).display())
        })?;
        owned.push(directory);
        let (adoption, child) = match request {
            Request::Take => {
                let drawn = skein_shell::crypto_seed().map_err(|errno| format!("checkpoint draw: errno {errno}"))?;
                let mut draw = [0; 16];
                draw.copy_from_slice(drawn.expose().get(..16).expect("32-byte draw"));
                (Adoption::Checkpoint { draw }, None)
            }
            Request::Remove(name) => (Adoption::RemoveCheckpoint { name: *name }, None),
            Request::Hold { name, command } => {
                let root = skein_shell::open_root(path(working))
                    .map_err(|errno| format!("working directory: errno {errno}"))?;
                owned.push(root);
                working_root = Some(root);
                let checkpoint = path(working).join(path(deployment)).join("checkpoints").join(path(&name.bytes()));
                let spawn = spawn(command, environment, working, &checkpoint, root);
                (
                    Adoption::HoldCheckpoint { name: *name },
                    Some((command.first().expect("hold command").clone(), spawn)),
                )
            }
        };
        let signals = skein_shell::open_termination_signals().map_err(|errno| format!("signals: errno {errno}"))?;
        owned.push(signals);
        service.adopt_signals(signals).expect("new checkpoint adopts signals");
        service
            .checkpoint_store(directory, skein_shell::effective_user(), adoption, child)
            .expect("new checkpoint adopts root");
        stopped::drive(binary, directories, service, deployment, |service, binary, _, deployment| {
            Some(lines(binary, service.checkpoint_outcome()?, deployment))
        })
    })();
    // The store and signal source close their adopted descriptors on success.
    match result {
        Ok(code) => {
            if let Some(root) = working_root {
                temper_service_shell::close(vec![root]);
            }
            Ok(code)
        }
        Err(why) => {
            temper_service_shell::close(owned);
            Err(prefixed(&format!("{binary} checkpoint"), &why))
        }
    }
}

fn executable(file: &std::path::Path) -> bool {
    std::fs::metadata(file).is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}
fn program(command: &[u8], environment: &[EnvironmentEntry], working: &[u8]) -> Option<Box<[u8]>> {
    let file = if command.contains(&b'/') {
        let file = path(working).join(path(command));
        executable(&file).then_some(file)?
    } else {
        temper_shell::environment::PATH
            .value(environment)?
            .split(|byte| *byte == b':')
            .map(|entry| path(working).join(path(entry)).join(path(command)))
            .find(|file| executable(file))?
    };
    Some(file.as_os_str().as_bytes().into())
}
fn spawn(
    command: &[Box<[u8]>],
    environment: &[EnvironmentEntry],
    working: &[u8],
    checkpoint: &std::path::Path,
    root: skein_io::kernel::Fd,
) -> Result<skein_io::kernel::Spawn, ()> {
    use skein_io::kernel::{Pipe, Spawn, Way};
    let (name, args) = command.split_first().ok_or(())?;
    let program = program(name, environment, working).ok_or(())?;
    let mut env: Vec<Box<[u8]>> = environment
        .iter()
        .filter(|(name, _)| name.as_ref() != DIRECTORY.name.as_bytes())
        .map(|(name, value)| [name.as_ref(), b"=", value.as_ref()].concat().into())
        .collect();
    env.push([DIRECTORY.name.as_bytes(), b"=", checkpoint.as_os_str().as_bytes()].concat().into());
    Ok(Spawn {
        program,
        args: args.into(),
        env: env.into(),
        root,
        dir: Box::from(b".".as_slice()),
        pipes: (0..3).map(|child| Pipe { child, way: Way::Inherit, parent: None }).collect(),
    })
}
