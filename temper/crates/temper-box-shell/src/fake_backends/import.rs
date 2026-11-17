//! Only current init output, with an explicit bundle, is importable in M1.
use super::{Directories, Json, bytes, public, resolved};
use crate::directories::jig_directories;
use std::{
    num::NonZeroU16,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

type Documents = (Box<[u8]>, Box<[u8]>);

pub(super) struct Source {
    pub marker: Box<[u8]>,
    pub documents: Option<(Json, Json)>,
    pub bundle: Option<PathBuf>,
    pub bundle_limits: temper_service::limits::Clients,
}
const RESOURCES: &[(&str, bool, &str)] = &[
    ("engine/hosts/workers/0/digest", false, "worker.digest"),
    ("workers/0/secret", false, "worker.secret"),
    ("workers/0/agent/configuration", true, "agent.json"),
    ("workers/0/agent/arguments/2", true, "agent.json"),
    ("application/demo/llm/key", false, "demo/llm.key"),
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
        let agent_bytes = public(&config.join("agent.json"))?;
        let mut temper = Json::decode(&temper_bytes)?;
        let mut agent = Json::decode(&agent_bytes)?;
        let (mut expected, expected_agent) = expected_documents(dirs, &temper, &agent, environment)?;
        let bundle = match temper.at("application/clients/bundle")? {
            Json::Null => None,
            Json::String(path) => Some(resolved(config, path)?),
            Json::Object(_) | Json::Array(_) | Json::Number(_) | Json::Bool(_) => {
                return Err("application/clients/bundle: expected a directory or null".into());
            }
        };
        // Bundle is an admitted public override. Its directory is checked before sandbox creation.
        expected.set("application/clients/bundle", temper.at("application/clients/bundle")?.clone())?;
        normalize_resources(dirs, &mut temper, &mut agent, &expected, &expected_agent)?;
        temper.same(&expected, "temper.json")?;
        agent.same(&expected_agent, "agent.json")?;
        let checked = crate::setup::check(&temper.encode()?, &jig_directories(dirs))?;
        marker.extend_from_slice(&skein_lib::sha256::digest(&temper_bytes));
        marker.extend_from_slice(&skein_lib::sha256::digest(&agent_bytes));
        Ok(Self {
            marker: marker.into(),
            documents: Some((temper, agent)),
            bundle,
            bundle_limits: checked.named.bundle_limits,
        })
    }
    /// Rebase only public paths whose init-equivalence was checked above. Relative data paths
    /// keep their spelling and resolve against D'; absolute spellings are made relative first.
    pub(super) fn rebase(&self, effective: &Directories) -> Result<Option<Documents>, String> {
        let Some((mut temper, mut agent)) = self.documents.clone() else { return Ok(None) };
        let agent_name = bytes(&crate::path(&effective.config).join("agent.json"));
        temper.set("workers/0/agent/configuration", Json::String(agent_name.clone()))?;
        temper.set("workers/0/agent/arguments/2", Json::String(agent_name))?;
        if let Ok(program) = agent.at_mut("programs/claude_code/directory") {
            *program = Json::String(bytes(&crate::path(&effective.data).join("programs")));
        }
        Ok(Some((temper.encode()?, agent.encode()?)))
    }
}
fn expected_documents(
    dirs: &Directories,
    temper: &Json,
    agent: &Json,
    environment: &[crate::EnvironmentEntry],
) -> Result<(Json, Json), String> {
    let binary = std::env::current_exe().map_err(|why| why.to_string())?;
    let git = temper_shell::lookup::find(environment, b"git");
    let claude = if agent.at("programs/claude_code").is_ok() {
        let name = agent.string("programs/claude_code/path")?;
        let path = crate::path(name);
        if name == temper_init::FETCHED {
            None
        } else {
            if !path.is_absolute() || name.len() > 4096 || name.contains(&0) {
                return Err("agent.json/programs/claude_code/path: expected a bounded absolute adapter path".into());
            }
            Some(path.to_path_buf())
        }
    } else {
        None
    };
    let ports = ports(temper)?;
    let login = temper
        .string("engine/owners/0/subject")?
        .strip_suffix(b"@localhost")
        .ok_or("engine/owners/0/subject: expected init's LOGIN@localhost")?;
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
fn normalize_resources(
    dirs: &Directories,
    temper: &mut Json,
    agent: &mut Json,
    expected: &Json,
    expected_agent: &Json,
) -> Result<(), String> {
    for &(pointer, configuration, name) in RESOURCES {
        equivalent(
            temper,
            expected,
            pointer,
            crate::path(if configuration { &dirs.config } else { &dirs.data }),
            name,
        )?;
    }
    if agent.at("programs/claude_code").is_ok() {
        equivalent(agent, expected_agent, "programs/claude_code/directory", crate::path(&dirs.data), "programs")?;
    }
    Ok(())
}

fn equivalent(value: &mut Json, expected: &Json, pointer: &str, root: &Path, name: &str) -> Result<(), String> {
    if resolved(root, value.string(pointer)?)? != resolved(root, name.as_bytes())? {
        return Err(format!("{pointer}: only init's {name} resource is supported"));
    }
    value.set(pointer, expected.at(pointer)?.clone())
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
        llm: port(value, "application/demo/llm/listen/socket")?,
        webhooks: port(value, "application/webhooks/listener/socket")?,
    })
}
fn documents(answer: temper_init::Answer) -> Result<(Json, Json), String> {
    let mut temper = None;
    let mut agent = None;
    for file in answer.files {
        if file.name == b"temper.json" || file.name == b"agent.json" {
            let temper_init::Content::Bytes { bytes, .. } = file.content else {
                unreachable!("init documents are bytes")
            };
            if file.name == b"temper.json" {
                temper = Some(Json::decode(&bytes)?);
            } else {
                agent = Some(Json::decode(&bytes)?);
            }
        }
    }
    Ok((temper.expect("init temper document"), agent.expect("init agent document")))
}
