//! Only current init output, with an explicit bundle, is importable in M1.
use super::{Directories, Json, public, resolved};
use crate::directories::jig_directories;
use std::{
    num::NonZeroU16,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

pub(super) struct Source {
    pub marker: Box<[u8]>,
    pub documents: Option<Json>,
    pub bundle: Option<PathBuf>,
    pub bundle_limits: temper_service::limits::Clients,
}
const RESOURCES: &[(&str, bool, &str)] = &[
    ("engine/deployment/directory", false, "deployment"),
    ("engine/hosts/workers/0/digest", false, "worker.digest"),
    ("workers/0/secret", false, "worker.secret"),
    ("application/machine/fake/git", false, "demo/bin/git"),
    ("application/demo/llm/key", false, "demo/llm.key"),
    ("application/workers/0/root", false, "workspaces"),
];
impl Source {
    pub(super) fn read(dirs: &Directories, environment: &[crate::EnvironmentEntry]) -> Result<Self, String> {
        let config = crate::path(&dirs.config);
        let empty = match std::fs::read_dir(config) {
            Ok(mut entries) => entries.next().transpose().map_err(|why| why.to_string())?.is_none(),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => true,
            Err(why) => return Err(format!("source configuration {}: {why}", config.display())),
        };
        let mut marker =
            format!("temper fake-backends v1\nconfig={:?}\ndata={:?}\n", dirs.config, dirs.data).into_bytes();
        if empty {
            marker.extend_from_slice(b"empty\n");
            return Ok(Self {
                marker: marker.into(),
                documents: None,
                bundle: None,
                bundle_limits: temper_service::limits::profile(temper_service::limits::Profile::Small).clients,
            });
        }
        let temper_bytes = public(&dirs.document())?;
        let mut temper = Json::decode(&temper_bytes)?;
        let mut expected = expected_documents(dirs, &temper, environment)?;
        let bundle = match temper.at("application/clients/bundle")? {
            Json::Null => None,
            Json::String(path) => Some(resolved(config, path)?),
            Json::Object(_) | Json::Array(_) | Json::Number(_) | Json::Bool(_) => {
                return Err("application/clients/bundle: expected a directory or null".into());
            }
        };
        // Bundle is an admitted public override. Its directory is checked before sandbox creation.
        expected.set("application/clients/bundle", temper.at("application/clients/bundle")?.clone())?;
        normalize_resources(dirs, &mut temper, &expected)?;
        temper.same(&expected, "temper.json")?;
        let checked = crate::setup::check(&temper.encode()?, &jig_directories(dirs))?;
        marker.extend_from_slice(&skein_lib::sha256::digest(&temper_bytes));
        Ok(Self { marker: marker.into(), documents: Some(temper), bundle, bundle_limits: checked.named.bundle_limits })
    }
    /// Rebase only public paths whose init-equivalence was checked above. Relative data paths
    /// keep their spelling and resolve against D'; absolute spellings are made relative first.
    pub(super) fn rebase(&self, _effective: &Directories) -> Result<Option<Box<[u8]>>, String> {
        self.documents.as_ref().map(Json::encode).transpose()
    }
}
fn expected_documents(
    dirs: &Directories,
    temper: &Json,
    environment: &[crate::EnvironmentEntry],
) -> Result<Json, String> {
    let binary = std::env::current_exe().map_err(|why| why.to_string())?;
    let git = match temper.at("application/workers/0/git")? {
        Json::Null => None,
        Json::String(path) => Some(current_executable(b"git", path, "application/workers/0/git", environment)?),
        Json::Object(_) | Json::Array(_) | Json::Number(_) | Json::Bool(_) => {
            return Err("application/workers/0/git: expected init's executable or null".into());
        }
    };
    let claude = match temper.at("application/workers/0/claude") {
        Ok(Json::String(name)) => {
            let path = crate::path(name);
            if !path.is_absolute() || name.len() > 4096 || name.contains(&0) {
                return Err("application/workers/0/claude: expected a bounded absolute adapter path".into());
            }
            Some(path.to_path_buf())
        }
        Ok(Json::Null | Json::Object(_) | Json::Array(_) | Json::Number(_) | Json::Bool(_)) => {
            return Err("application/workers/0/claude: expected an absolute adapter path".into());
        }
        Err(_) => None,
    };
    let ports = ports(temper)?;
    let login = temper.string("application/machine/owned/person/login")?;
    let generated = temper_init::files(
        &temper_init::Facts {
            sign_in: temper.at("engine/clients/provider")? != &Json::Null,
            binary: binary.as_os_str().as_bytes(),
            git: git.as_ref().map(|path| path.as_os_str().as_bytes()),
            claude: claude.as_ref().map(|path| path.as_os_str().as_bytes()),
            ports,
            login: Some(login),
        },
        &temper_init::Directories { config: &dirs.config, data: &dirs.data },
        &[0; 32],
        &[0; 32],
    )
    .map_err(|why| format!("source init shape: {why:?}"))?;
    documents(generated)
}
fn normalize_resources(dirs: &Directories, temper: &mut Json, expected: &Json) -> Result<(), String> {
    for &(pointer, configuration, name) in RESOURCES {
        equivalent(
            temper,
            expected,
            pointer,
            crate::path(if configuration { &dirs.config } else { &dirs.data }),
            name,
        )?;
    }
    Ok(())
}

fn equivalent(value: &mut Json, expected: &Json, pointer: &str, root: &Path, name: &str) -> Result<(), String> {
    if resolved(root, value.string(pointer)?)? != resolved(root, name.as_bytes())? {
        return Err(format!("{pointer}: only init's {name} resource is supported"));
    }
    value.set(pointer, expected.at(pointer)?.clone())
}
fn current_executable(
    name: &[u8],
    given: &[u8],
    pointer: &str,
    environment: &[crate::EnvironmentEntry],
) -> Result<PathBuf, String> {
    match executable(name, environment) {
        Some(path) if path.as_os_str().as_bytes() == given => Ok(path),
        Some(_) | None => Err(format!(
            "{pointer}: custom external executables are unsupported; expected current init's PATH selection"
        )),
    }
}
fn executable(name: &[u8], environment: &[crate::EnvironmentEntry]) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    crate::init::candidates(environment, name)
        .into_iter()
        .find(|path| std::fs::metadata(path).is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0))
}
fn port(value: &Json, pointer: &str) -> Result<NonZeroU16, String> {
    let socket = std::str::from_utf8(value.string(pointer)?)
        .ok()
        .and_then(|s| s.parse::<std::net::SocketAddr>().ok())
        .filter(|addr| addr.ip() == std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST) && addr.port() != 0)
        .ok_or_else(|| format!("{pointer}: expected init's IPv4 loopback socket"))?;
    Ok(NonZeroU16::new(socket.port()).expect("nonzero checked"))
}
fn ports(value: &Json) -> Result<temper_init::Ports, String> {
    Ok(temper_init::Ports {
        clients: port(value, "application/clients/listener/socket")?,
        owned: port(value, "application/machine/owned/listen/socket")?,
        forge: port(value, "application/machine/fake/listen/socket")?,
        llm: port(value, "application/demo/llm/listen/socket")?,
        webhooks: port(value, "application/webhooks/listener/socket")?,
    })
}
fn documents(answer: temper_init::Answer) -> Result<Json, String> {
    for file in answer.files {
        if file.name == b"temper.json" {
            let temper_init::Content::Bytes { bytes, .. } = file.content else { unreachable!("init document bytes") };
            return Json::decode(&bytes);
        }
    }
    unreachable!("init temper document")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imported_executable_uses_the_given_search_path() {
        use std::os::unix::fs::PermissionsExt;
        let scratch = skein_scratch::Scratch::new("imported-executable");
        let git = scratch.path().join("git");
        std::fs::write(&git, b"#!/bin/sh\nexit 1\n").expect("fixture executable");
        std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o700)).expect("executable mode");
        let environment = [(Box::from(b"PATH".as_slice()), Box::from(scratch.path().as_os_str().as_bytes()))];
        assert_eq!(executable(b"git", &environment), Some(git));
        assert_eq!(executable(b"git", &[]), None);
    }
}
