//! A place and a setup chain share one kind (qa/places.md, 2; qa/triage.md, 3).
use crate::{Refusal, refused};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Box,
    Deployment,
}
impl Kind {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Box => "box",
            Self::Deployment => "deployment",
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
