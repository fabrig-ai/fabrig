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
    pub places: Vec<String>,
    pub surfaces: Vec<String>,
    pub areas: std::collections::BTreeMap<String, String>,
    pub request: RequestConfiguration,
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
    pub pass: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestConfiguration {
    pub command: Vec<String>,
    pub state: Option<Vec<String>>,
    pub area: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub poll: u64,
    pub triage: u64,
    pub build_wait: u64,
    pub pass: u64,
}
impl Configuration {
    #[must_use]
    pub fn finding_area(&self, surface: &str) -> &str {
        self.areas.get(surface).map_or(self.owner.as_str(), String::as_str)
    }
    pub fn place_directory(&self, home: &Path, name: &str) -> Result<PathBuf, Refusal> {
        if !self.places.iter().any(|place| place == name) {
            return Err(refused(format!("unknown place {name}")));
        }
        Ok(path(home, name))
    }
    pub fn decode(bytes: &[u8], home: &Path) -> Result<Self, Refusal> {
        let decode = || {
            let value: Value = serde_json::from_slice(bytes).map_err(|why| refused(why.to_string()))?;
            let object = d::object(
                &value,
                &["owner", "git", "agent", "bounds", "builds", "notify", "places", "surfaces", "areas", "request"],
            )?;
            let owner = nonempty(object, "owner")?;
            let git = named_object(object, "git", &["directory", "branch", "fetch"])?;
            let directory = path(home, &nonempty(git, "directory").map_err(|why| refused(format!("git.{why}")))?);
            let branch = git.get("branch").map(d::string).transpose()?.unwrap_or_else(|| "main".into());
            let fetch = git.get("fetch").map(d::string).transpose()?;
            let agent = named_object(object, "agent", &["command", "triage", "pass"])?;
            let command = command_line(d::required(agent, "command").map_err(|why| refused(format!("agent.{why}")))?)?;
            let triage = nonempty(agent, "triage").map_err(|why| refused(format!("agent.{why}")))?;
            let pass = nonempty(agent, "pass").map_err(|why| refused(format!("agent.pass: {why}")))?;
            let request = named_object(object, "request", &["command", "state", "area"])?;
            let request = RequestConfiguration {
                command: command_line(
                    d::required(request, "command").map_err(|why| refused(format!("request.{why}")))?,
                )?,
                state: request.get("state").map(command_line).transpose()?,
                area: request.get("area").map(d::string).transpose()?.unwrap_or_else(|| owner.clone()),
            };
            let places = object.get("places").map(d::strings).transpose()?.unwrap_or_else(|| vec!["places/1".into()]);
            if places.is_empty() || places.iter().any(|place| place.trim().is_empty()) {
                return Err(refused("places needs at least one directory"));
            }
            let unique: std::collections::BTreeSet<_> = places.iter().collect();
            if unique.len() != places.len() {
                return Err(refused("places directories must be distinct"));
            }
            let surfaces = object.get("surfaces").map(d::strings).transpose()?.unwrap_or_else(|| vec!["web".into()]);
            for surface in &surfaces {
                if surface != "web" && surface != "tui" {
                    return Err(refused(format!("surfaces: {surface} is neither web nor tui")));
                }
            }
            let mut areas = std::collections::BTreeMap::new();
            if let Some(value) = object.get("areas") {
                for (surface, area) in d::object(value, &["web", "tui"])? {
                    areas.insert(surface.clone(), d::string(area)?);
                }
            }
            let empty = Map::new();
            let bounds = match object.get("bounds") {
                Some(bounds) => d::object(bounds, &["poll", "triage", "build_wait", "pass"])
                    .map_err(|why| refused(format!("bounds: {why}")))?,
                None => &empty,
            };
            let bounds = Bounds {
                poll: seconds(bounds, "poll", 60)?,
                triage: seconds(bounds, "triage", 900)?,
                build_wait: seconds(bounds, "build_wait", 21600)?,
                pass: seconds(bounds, "pass", 7200)?,
            };
            let builds =
                path(home, &object.get("builds").map(d::string).transpose()?.unwrap_or_else(|| "builds".into()));
            let notify = object.get("notify").map(command_line).transpose()?;
            Ok(Self {
                owner,
                git: Git { directory, branch, fetch },
                agent: Agent { command, triage, pass },
                bounds,
                builds,
                notify,
                places,
                surfaces,
                areas,
                request,
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
