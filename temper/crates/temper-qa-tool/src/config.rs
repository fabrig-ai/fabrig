//! Home-relative configuration (qa/README.md, 3.2).
use crate::{Refusal, document as d, refused};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Configuration {
    pub owner: String,
    pub git: Git,
    pub agent: Agent,
    pub bounds: Bounds,
    pub builds: PathBuf,
    pub notify: Option<Vec<String>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Git {
    pub directory: PathBuf,
    pub branch: String,
    pub fetch: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Agent {
    pub command: Vec<String>,
    pub triage: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub poll: u64,
    pub triage: u64,
    pub build_wait: u64,
}
impl Configuration {
    pub fn decode(bytes: &[u8], home: &Path) -> Result<Self, Refusal> {
        let decode = || {
            let value: Value = serde_json::from_slice(bytes).map_err(|why| refused(why.to_string()))?;
            let object = d::object(&value, &["owner", "git", "agent", "bounds", "builds", "notify"])?;
            let owner = nonempty(object, "owner")?;
            let git = named_object(object, "git", &["directory", "branch", "fetch"])?;
            let directory = path(home, &nonempty(git, "directory").map_err(|why| refused(format!("git.{why}")))?);
            let branch = git.get("branch").map(d::string).transpose()?.unwrap_or_else(|| "main".into());
            let fetch = git.get("fetch").map(d::string).transpose()?;
            let agent = named_object(object, "agent", &["command", "triage"])?;
            let command = command_line(d::required(agent, "command").map_err(|why| refused(format!("agent.{why}")))?)?;
            let triage = nonempty(agent, "triage").map_err(|why| refused(format!("agent.{why}")))?;
            let empty = Map::new();
            let bounds = match object.get("bounds") {
                Some(bounds) => d::object(bounds, &["poll", "triage", "build_wait"])
                    .map_err(|why| refused(format!("bounds: {why}")))?,
                None => &empty,
            };
            let bounds = Bounds {
                poll: seconds(bounds, "poll", 60)?,
                triage: seconds(bounds, "triage", 900)?,
                build_wait: seconds(bounds, "build_wait", 21600)?,
            };
            let builds =
                path(home, &object.get("builds").map(d::string).transpose()?.unwrap_or_else(|| "builds".into()));
            let notify = object.get("notify").map(command_line).transpose()?;
            Ok(Self {
                owner,
                git: Git { directory, branch, fetch },
                agent: Agent { command, triage },
                bounds,
                builds,
                notify,
            })
        };
        decode().map_err(|why: Refusal| refused(format!("qa.json: {why}")))
    }
}
fn named_object<'a>(
    object: &'a Map<String, Value>,
    name: &str,
    fields: &[&str],
) -> Result<&'a Map<String, Value>, Refusal> {
    d::object(d::required(object, name)?, fields).map_err(|why| refused(format!("{name}: {why}")))
}
fn nonempty(object: &Map<String, Value>, name: &str) -> Result<String, Refusal> {
    let value = d::text(object, name)?;
    if value.trim().is_empty() {
        return Err(refused(format!("{name} is empty")));
    }
    Ok(value)
}
fn command_line(value: &Value) -> Result<Vec<String>, Refusal> {
    let command = d::strings(value)?;
    if command.first().is_none_or(String::is_empty) {
        return Err(refused("command needs a program"));
    }
    Ok(command)
}
fn seconds(object: &Map<String, Value>, name: &str, default: u64) -> Result<u64, Refusal> {
    Ok(object
        .get(name)
        .map(d::number)
        .transpose()
        .map_err(|why| refused(format!("bounds.{name}: {why}")))?
        .unwrap_or(default))
}
fn path(home: &Path, value: &str) -> PathBuf {
    home.join(value)
}
