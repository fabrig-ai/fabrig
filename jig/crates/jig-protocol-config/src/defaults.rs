//! The application's settings, passed after parsing (configuration.md, 1 and 3).
use crate::{Engine, Policy, Problems, domain};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use skein_lib::List;

/// Give an engine parsed with `Defaults::Passed` the application's defaults, then check
/// its seed projects against them. The core checks the defaults themselves at each start.
/// # Errors
/// The document's problems against the defaults, at the document's paths.
pub fn pass(engine: &mut Engine, defaults: jig_domain_core::settings::Defaults) -> Result<(), Problems> {
    assert_eq!(engine.defaults, crate::Defaults::Passed, "pass requires passed defaults");
    engine.charters = defaults.charters;
    engine.templates = defaults.templates;
    engine.rules = defaults.rules;
    let mut policies = List::with_capacity(defaults.policies.len());
    for (number, policy) in &defaults.policies {
        policies
            .push(Policy {
                number: *number,
                policy: policy.policy.clone(),
                roles: policy.permissions.clone(),
                templates: policy.templates.clone(),
            })
            .expect("one entry per passed policy");
    }
    engine.policies = policies.into_boxed();
    let mut problems = Problems::empty();
    crate::projects::resolve(engine, &mut problems);
    if problems.is_empty() {
        domain::seeds(engine, &mut problems);
    }
    if problems.is_empty() { Ok(()) } else { Err(problems) }
}

pub(crate) fn document(engine: &mut Engine, problems: &mut Problems) {
    if engine.defaults == crate::Defaults::Document {
        crate::projects::resolve(engine, problems);
        if problems.is_empty() {
            domain::seeds(engine, problems);
        }
        domain::document_defaults(engine, problems);
    }
}

pub(crate) fn empty_rules() -> authority::Rules {
    authority::Rules {
        ceiling: authority::Authority {
            tools: authority::Tools(0),
            models: authority::Models::NONE,
            grants: Box::default(),
            delegation: authority::Delegation { kinds: Box::default(), tasks: 0, depth: 0 },
            budget: authority::Budget { spend: 0, deadline: None },
            notes: authority::Scopes(0),
            note_resources: Box::default(),
        },
        period_spend: 0,
        minimum_run_spend: 0,
        maximum_run_spend: 0,
        implies: authority::Implies::new(Box::default(), 0).expect("empty implications"),
        requirements: Box::default(),
    }
}

pub(crate) struct Offers {
    pub charters: Option<Box<[jig_domain_core::Charter]>>,
    pub templates: Option<Box<[jig_domain_core::Template]>>,
}
pub(crate) fn offers(
    document: &skein_json::Document,
    object: &mut crate::read::Object,
    llm: Option<&crate::Llm>,
    limits: &crate::Limits,
    directories: &crate::Directories,
    defaults: crate::Defaults,
    problems: &mut Problems,
) -> Offers {
    let path = crate::Path::root();
    if defaults == crate::Defaults::Passed {
        for name in [b"charters".as_slice(), b"templates", b"policies", b"rules"] {
            if object.member(document, name).is_some() {
                problems.push(path.member(name), crate::What::Stored);
            }
        }
        return Offers { charters: Some(Box::default()), templates: Some(Box::default()) };
    }
    let mut charters = None;
    if let Some(at) = object.required(document, b"charters", &path, problems)
        && let Some(llm) = llm
    {
        charters = crate::charters::parse(document, at, &path.member(b"charters"), llm, limits, directories, problems);
    }
    let configured = match llm {
        Some(llm) => &*llm.models,
        None => &[],
    };
    let mut templates = None;
    if let Some(at) = object.required(document, b"templates", &path, problems) {
        templates = crate::templates::templates(
            document,
            at,
            &path.member(b"templates"),
            charters.as_deref().unwrap_or(&[]),
            configured,
            limits,
            problems,
        );
    }
    Offers { charters, templates }
}
