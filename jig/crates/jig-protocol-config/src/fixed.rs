//! Members an application fixes (protocol/configuration.md, 3.4).
use crate::{Clients, Deployment, IdentityProvider, Limits, Path, Problems, Project, What, read};
use alloc::boxed::Box;
use skein_json::Document;
use skein_lib::Duration;

#[derive(Clone, Debug)]
pub struct FixedClients {
    pub heartbeat: Duration,
    pub link: bool,
    pub approval: bool,
}
impl FixedClients {
    pub(crate) fn clients(&self) -> Clients {
        Clients { heartbeat: self.heartbeat, link: self.link, approval: self.approval, provider: None }
    }
}
#[derive(Clone, Debug)]
pub struct Fixed {
    pub deployment: Option<Deployment>,
    pub providers: Option<Box<[IdentityProvider]>>,
    pub connectors: Option<Box<[u16]>>,
    pub acts: Option<Box<[jig_domain_people::ActGrant]>>,
    pub projects: Option<Box<[Project]>>,
    pub clients: Option<FixedClients>,
}
impl Fixed {
    pub const NONE: Self =
        Self { deployment: None, providers: None, connectors: None, acts: None, projects: None, clients: None };
}

pub(crate) fn refuse(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    path: &Path,
    problems: &mut Problems,
) {
    if object.member(document, name).is_some() {
        problems.push(path.member(name), What::Fixed);
    }
}
pub(crate) fn member(
    object: &mut read::Object,
    document: &Document,
    name: &[u8],
    fixed: bool,
    path: &Path,
    problems: &mut Problems,
) -> Option<u32> {
    if fixed {
        refuse(object, document, name, path, problems);
        None
    } else {
        object.required(document, name, path, problems)
    }
}

pub(crate) fn check(fixed: &Fixed, limits: &Limits, problems: &mut Problems) {
    if let Some(deployment) = &fixed.deployment
        && let Err(what) = crate::Directories::working().join(crate::Directory::Data, &deployment.directory)
    {
        problems.push(Path::root().member(b"deployment").member(b"directory"), what);
    }
    if let Some(providers) = &fixed.providers {
        crate::domain::check_providers(providers, &Path::root().member(b"providers"), problems);
    }
    if let Some(connectors) = &fixed.connectors {
        crate::domain::check_connectors(
            connectors,
            limits.core.connectors,
            &Path::root().member(b"connectors"),
            problems,
        );
    }
}
