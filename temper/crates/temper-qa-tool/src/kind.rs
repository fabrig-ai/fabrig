//! A place and a setup chain share one kind (qa/places.md, 2; qa/triage.md, 3).
use crate::{
    Refusal,
    faces::{FilesFace, PathKind, RealFiles},
    refused,
};
use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Box,
    Deployment,
}
impl Kind {
    /// Also the name of this kind's scratch directory (qa/kinds.md, 3).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Box => "box",
            Self::Deployment => "deployment",
        }
    }
    #[must_use]
    pub const fn pinned(self) -> &'static str {
        match self {
            Self::Box => "data/box/fake-backends/data/workspaces/agent",
            Self::Deployment => "data/worker/workspaces/agent",
        }
    }
    #[must_use]
    pub const fn offers_fresh(self) -> bool {
        match self {
            Self::Box => true,
            Self::Deployment => false,
        }
    }
    pub(crate) fn check_fresh(self) -> Result<(), Refusal> {
        if self.offers_fresh() {
            Ok(())
        } else {
            Err(refused("a deployment's place has no fresh setup: its setups start from their root's config/"))
        }
    }
    #[must_use]
    pub fn drive_arguments(self, root: Option<&Path>) -> Vec<OsString> {
        match self {
            Self::Box => Vec::new(),
            Self::Deployment => {
                vec!["--deployment".into(), root.expect("deployment setup's root").join("config").into()]
            }
        }
    }
    pub(crate) fn parse(name: &str) -> Result<Self, Refusal> {
        match name {
            "box" => Ok(Self::Box),
            "deployment" => Ok(Self::Deployment),
            other => Err(refused(format!("unknown kind {other}"))),
        }
    }
    pub(crate) const fn stored(self) -> Option<&'static str> {
        match self {
            Self::Box => None,
            Self::Deployment => Some("deployment"),
        }
    }
}

pub(crate) struct Setup {
    pub parent: Option<String>,
    pub config: bool,
}

/// Missing members, cycles and more than eight parents cannot establish a kind.
pub(crate) fn chain_kind(
    name: &str,
    mut read: impl FnMut(&str) -> Result<Option<Setup>, Refusal>,
) -> Result<Option<Kind>, Refusal> {
    let mut name = name.to_owned();
    let mut visited = Vec::new();
    loop {
        if visited.len() >= 9 || visited.contains(&name) {
            return Ok(None);
        }
        let Some(setup) = read(&name)? else { return Ok(None) };
        visited.push(name);
        match setup.parent {
            Some(parent) => name = parent,
            None => return Ok(Some(if setup.config { Kind::Deployment } else { Kind::Box })),
        }
    }
}

pub(crate) fn parent(recipe: &str) -> Option<&str> {
    recipe
        .lines()
        .nth(1)
        .and_then(|line| line.strip_prefix("after "))
        .map(|after| after.strip_prefix('"').and_then(|name| name.strip_suffix('"')).unwrap_or(after))
}

/// The pass's local chain, root first, shared by setup and the holder (qa/kinds.md, 6).
pub(crate) struct Chain {
    pub members: Vec<ChainMember>,
    pub root: PathBuf,
}
pub(crate) struct ChainMember {
    pub name: String,
    pub recipe: String,
    pub scripts: Option<PathBuf>,
}
impl Chain {
    pub fn kind(&self) -> Kind {
        if self.root.join("config").is_dir() { Kind::Deployment } else { Kind::Box }
    }
}
#[derive(Clone, Copy)]
pub(crate) enum ChainUse {
    Setup,
    Holder,
}
impl ChainUse {
    fn loop_refusal(self, member: &str, parent: &str, why: &str) -> Refusal {
        match self {
            Self::Setup => refused(format!("recipe {parent}: cycle or more than eight parents")),
            Self::Holder => refused(format!("recipe {member}: {why}")),
        }
    }
}
pub(crate) fn read_chain(setups: &Path, name: &str, usage: ChainUse) -> Result<Chain, Refusal> {
    read_chain_with(setups, name, usage, &mut RealFiles)
}

pub(crate) fn read_chain_with(
    setups: &Path,
    name: &str,
    usage: ChainUse,
    files: &mut impl FilesFace,
) -> Result<Chain, Refusal> {
    let mut name = name.to_owned();
    let mut visited = Vec::new();
    let mut members = Vec::new();
    loop {
        let fail = |why| refused(format!("recipe {name}: {why}"));
        visited.push(name.clone());
        let directory = setups.join(&name);
        let recipe = files.read_to_string(&directory.join("recipe")).map_err(|why| fail(why.to_string()))?;
        if recipe.len() > 64 * 1024 || recipe.lines().count() > 1024 {
            return Err(fail("recipe exceeds its size or line bound".into()));
        }
        let mut lines = recipe.lines();
        if lines.next() != Some("recipe 1") {
            return Err(fail("expected recipe 1".into()));
        }
        let script = directory.join("scripts.json");
        let scripts = match files.facts(&script, true) {
            Ok(facts) if facts.kind == PathKind::File => {
                files.open_read(&script).map_err(|why| fail(why.to_string()))?;
                Some(script)
            }
            Ok(_) => return Err(fail("scripts.json is not a file".into())),
            Err(why) if why.kind() == io::ErrorKind::NotFound => None,
            Err(why) => return Err(fail(why.to_string())),
        };
        let Some(second) = lines.next() else {
            return Err(fail("no closing check".into()));
        };
        if second == "after" || second.starts_with("after\t") {
            return Err(fail("invalid after record".into()));
        }
        let next = if let Some(after) = second.strip_prefix("after ") {
            let parent = after.strip_prefix('"').and_then(|value| value.strip_suffix('"')).unwrap_or(after);
            if !crate::place::recipe_name(parent) {
                return Err(fail("invalid after recipe name".into()));
            }
            if visited.iter().any(|name| name == parent) {
                return Err(usage.loop_refusal(&name, parent, "cycle in after chain"));
            }
            if visited.len() == 9 {
                return Err(usage.loop_refusal(&name, parent, "more than eight parents"));
            }
            Some(parent.to_owned())
        } else {
            None
        };
        members.push(ChainMember { name: name.clone(), recipe, scripts });
        let Some(parent) = next else { break };
        name = parent;
    }
    members.reverse();
    let root = setups.join(name);
    Ok(Chain { members, root })
}
