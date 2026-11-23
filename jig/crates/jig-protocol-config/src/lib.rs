//! Strict configuration values and operator problems (protocol/configuration.md, 1 and 9).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
mod acts;
mod fixed;
pub use fixed::{Fixed, FixedClients};
mod charters;
mod core;
mod templates;
pub use core::core;
mod directories;
pub use directories::{Directories, Directory};
mod defaults;
mod domain;
pub use defaults::{pass, pass_providers};
mod projects;
pub use domain::{
    Deployment, Provider as IdentityProvider, ProviderKind, amendment_bytes_max, check_effective_limits,
    policy_bytes_max,
};
pub use projects::{Policy, Project, policies};
mod check;
mod worker;
pub use worker::{Agent, FixedWorker, Graces, Worker, stop_bound, worker, worker_over};
pub mod clients;
mod files;
pub mod hosts;
pub mod llm;
pub use clients::Clients;
pub use files::{File, Files, files, worker_files};
pub use hosts::Hosts;
pub use llm::{Endpoint, EndpointAddress, Header, Llm, Provider, ProviderSetting, Registration, Transport};
pub mod address;
mod document;
pub mod members;
pub mod names;
pub mod overrides;
pub use overrides::with_overrides;
pub mod profile;
pub use profile::{JournalSizes, Limits, Link, stock};
pub mod forms;
mod problem;
pub mod read;
pub use address::Address;
pub use document::{DocumentLimits, document};
pub use forms::{digest, secret};
pub use problem::{CodecBound, PROBLEMS, Path, Problem, Problems, Segment, What};
/// Where an engine's settings defaults come from (configuration.md, 1;
/// domain/engine/settings.md, 1 and 7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Defaults {
    /// The document's `charters`, `templates`, `policies` and `rules`.
    Document,
    /// The application's defaults, given by `pass`; the four members are refused.
    Passed,
    /// All defaults, including providers, passed by the application.
    PassedProviders,
}
/// The engine's protocol configuration (protocol/configuration-engine.md, 1).
#[derive(Clone, Debug)]
pub struct Engine {
    /// With `Passed`, the four settings fields are empty until `pass` gives them.
    pub defaults: Defaults,
    pub deployment: Deployment,
    pub owners: Box<[jig_domain_people::InitialOwner]>,
    pub acts: Box<[jig_domain_people::ActGrant]>,
    pub providers: Box<[IdentityProvider]>,
    pub rules: jig_domain_authority::Rules,
    pub projects: Box<[Project]>,
    /// Policy numbers awaiting defaults, by seed index; empty once the copy is resolved.
    pub seed_policies: Box<[Option<u32>]>,
    pub policies: Box<[Policy]>,
    pub connectors: Box<[u16]>,
    pub memory: Option<u64>,
    pub charters: Box<[jig_domain_core_vocabulary::Charter]>,
    pub templates: Box<[jig_domain_core_vocabulary::Template]>,
    pub llm: Llm,
    pub provider_defaults: Option<Box<[ProviderSetting]>>,
    pub fixed_llm_providers: Box<[ProviderSetting]>,
    pub hosts: Hosts,
    pub clients: Clients,
    pub limits: Limits,
}
/// Parse jig's engine member before startup constructs any component.
pub fn engine(
    document: &skein_json::Document,
    at: u32,
    directories: &Directories,
    defaults: Defaults,
    fixed: &Fixed,
) -> Result<Engine, Problems> {
    engine_over(document, at, directories, defaults, fixed, &stock())
}

/// Parse jig's engine member over an application's base (protocol/limits.md, the stock).
#[expect(clippy::too_many_lines, reason = "one ordered read before the engine whole-check")]
#[expect(clippy::manual_map, reason = "step code uses no closures")]
pub fn engine_over(
    document: &skein_json::Document,
    at: u32,
    directories: &Directories,
    defaults: Defaults,
    fixed: &Fixed,
    base: &Limits,
) -> Result<Engine, Problems> {
    let path = Path::root();
    let mut problems = Problems::empty();
    let Some(mut object) = check::object(document, at, &path, &mut problems, "engine") else {
        return Err(problems);
    };
    let mut limits = *base;
    if let Some(at) = object.member(document, b"limits") {
        let parsed = overrides::parse_over(document, at, &path.member(b"limits"), base);
        limits = parsed.limits;
        for problem in &parsed.problems.problems {
            problems.push(problem.path.clone(), problem.what);
        }
        problems.past = problems.past.saturating_add(parsed.problems.past);
    }
    let mut llm = None;
    if defaults == Defaults::PassedProviders {
        if object.member(document, b"llm").is_some() {
            problems.push(path.member(b"llm"), What::Stored);
        }
        llm = Some(Llm { models: Box::new([]), providers: Box::new([]), endpoints: Box::new([]) });
    } else if let Some(at) = object.required(document, b"llm", &path, &mut problems) {
        llm = llm::parse(document, at, &path.member(b"llm"), directories, defaults, &mut problems);
    }
    let mut hosts = None;
    if let Some(at) = object.required(document, b"hosts", &path, &mut problems) {
        hosts = hosts::parse(document, at, &path.member(b"hosts"), &limits, directories, &mut problems);
    }
    let mut clients = None;
    let clients_at = if fixed.clients.is_some() {
        object.member(document, b"clients")
    } else {
        object.required(document, b"clients", &path, &mut problems)
    };
    if let Some(at) = clients_at {
        clients =
            clients::parse(document, at, &path.member(b"clients"), directories, fixed.clients.as_ref(), &mut problems);
    } else if let Some(given) = &fixed.clients {
        clients = Some(given.clients());
    }
    let offers = defaults::offers(document, &mut object, llm.as_ref(), &limits, directories, defaults, &mut problems);
    let configured = match &llm {
        Some(llm) => llm.models(),
        None => Box::default(),
    };
    let mut deployment = fixed.deployment.clone();
    if let Some(at) =
        fixed::member(&mut object, document, b"deployment", fixed.deployment.is_some(), &path, &mut problems)
    {
        deployment = domain::deployment(document, at, &path.member(b"deployment"), directories, &mut problems);
    }
    let mut providers = fixed.providers.clone();
    if let Some(at) =
        fixed::member(&mut object, document, b"providers", fixed.providers.is_some(), &path, &mut problems)
    {
        providers = domain::providers(document, at, &path.member(b"providers"), &mut problems);
    }
    let mut rules = if defaults == Defaults::Document { None } else { Some(defaults::empty_rules()) };
    if defaults == Defaults::Document
        && let Some(at) = object.required(document, b"rules", &path, &mut problems)
    {
        let authority = &limits.core.authority;
        rules = projects::rules(document, at, &path.member(b"rules"), authority, &configured, &mut problems);
    }
    let mut projects = match &fixed.projects {
        Some(values) => Some(projects::Projects { values: values.clone(), policies: Box::default() }),
        None => None,
    };
    if let Some(at) = fixed::member(&mut object, document, b"projects", fixed.projects.is_some(), &path, &mut problems)
    {
        projects = projects::projects(document, at, &path.member(b"projects"), &limits, &configured, &mut problems);
    }
    let policies = if defaults == Defaults::Document {
        projects::configured_policies(&mut object, document, &limits, &configured, &mut problems)
    } else {
        Some(Box::default())
    };
    let mut connectors = fixed.connectors.clone();
    if let Some(at) =
        fixed::member(&mut object, document, b"connectors", fixed.connectors.is_some(), &path, &mut problems)
    {
        connectors =
            domain::connectors(document, at, &path.member(b"connectors"), limits.core.connectors, &mut problems);
    }
    let acts = if let Some(given) = &fixed.acts {
        fixed::refuse(&mut object, document, b"acts", &path, &mut problems);
        acts::check(given, connectors.as_deref().unwrap_or(&[]), &path.member(b"acts"), &mut problems);
        given.clone()
    } else {
        acts::parse(&mut object, document, &path, connectors.as_deref().unwrap_or(&[]), &mut problems)
    };
    fixed::check(fixed, &limits, &mut problems);
    limits.core.people.act_grants = u32::try_from(acts.len()).expect("act-grant count fits u32");
    let memory = check::optional_integer(&mut object, document, b"memory_bytes", &path, &mut problems);
    let owners = object.required(document, b"owners", &path, &mut problems);
    object.finish(document, &path, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    let projects = projects.expect("required member checked");
    let mut engine = Engine {
        defaults,
        deployment: deployment.expect("required member checked"),
        providers: providers.expect("required member checked"),
        rules: rules.expect("required member checked"),
        projects: projects.values,
        seed_policies: projects.policies,
        policies: policies.expect("optional policies checked"),
        connectors: connectors.expect("required member checked"),
        owners: Box::default(),
        acts,
        memory,
        charters: offers.charters.expect("required member checked"),
        templates: offers.templates.expect("required member checked"),
        llm: llm.expect("required member checked"),
        provider_defaults: None,
        fixed_llm_providers: Box::new([]),
        hosts: hosts.expect("required member checked"),
        clients: clients.expect("required member checked"),
        limits,
    };
    engine.owners = domain::owners(document, owners.expect("required member checked"), &engine, &mut problems);
    clients::check_provider(&engine, &mut problems);
    domain::whole(&engine, &mut problems);
    defaults::document(&mut engine, &mut problems);
    if !problems.is_empty() {
        return Err(problems);
    }
    Ok(engine)
}
#[cfg(test)]
mod tests;
