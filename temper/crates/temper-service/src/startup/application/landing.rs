//! Project landing ways, checks and tiers (temper/docs/design/protocol/configuration.md, 3).
use super::{Problems, What, limits, read};
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path};
use skein_json::Document;
use skein_lib::{List, Map};
use temper_engine_domain as engine;
use temper_engine_domain_forge::{CHECK_NAME_BYTES, WORKFLOW_BYTES};
use temper_engine_domain_forge_tiers::Format;

#[expect(clippy::type_complexity, reason = "the document yields project landing ways and their order together")]
pub(super) fn parse(
    document: &Document,
    at: u32,
    jig: &config::Engine,
    limits: &limits::Limits,
    problems: &mut Problems,
) -> Option<(Map<u32, engine::ProjectLanding>, Box<[u32]>)> {
    let path = Path::root().member(b"application").member(b"landing");
    let mut object = read::object(document, at, &path, problems)?;
    if object.member(document, b"deployment").is_some() {
        problems.push(path.member(b"deployment"), What::LandingStored);
    }
    let project_ats =
        read::list(&mut object, document, b"projects", jig.limits.core.authority.projects, &path, problems);
    read::finish(object, document, &path, problems);
    let mut order = List::with_capacity(jig.limits.core.authority.projects);
    let mut landings = Map::with_capacity(jig.limits.core.authority.projects);
    if let Some(ats) = project_ats {
        for (index, at) in ats.iter().enumerate() {
            let path = read::index(&path.member(b"projects"), index);
            let Some(mut object) = read::object(document, *at, &path, problems) else { continue };
            let project = read::number(&mut object, document, b"project", &path, problems);
            if object.member(document, b"rules").is_some() {
                problems.push(path.member(b"rules"), What::ProjectLandingStored);
            }
            let landing = project_landing(&mut object, document, &path, &limits.forge, problems);
            read::finish(object, document, &path, problems);
            if let Some(project) = project {
                let mut found = false;
                for configured in &jig.projects {
                    if configured.number == project {
                        found = true;
                    }
                }
                // temper/docs/design/protocol/configuration.md, 3: either way
                // can configure a restored project; policy rules live in the store.
                let restored = landing.is_some();
                if !found && !restored {
                    problems.push(path.member(b"project"), What::Jig(config::What::Refers));
                }
                if landings.contains_key(&project) {
                    problems.push(path.member(b"project"), What::Jig(config::What::Unique));
                }
                if let Some(landing) = landing {
                    landings.insert(project, landing).expect("bounded projects");
                }
                order.push(project).expect("bounded projects");
            }
        }
    }
    Some((landings, order.into_boxed()))
}

fn optional_list(
    object: &mut config::read::Object,
    document: &Document,
    name: &[u8],
    max: u32,
    path: &Path,
    problems: &mut Problems,
) -> Option<List<u32>> {
    let Some(at) = object.member(document, name) else { return Some(List::with_capacity(0)) };
    match config::read::list(document, at, max) {
        Ok(ats) => Some(ats),
        Err(what) => {
            problems.push(path.member(name), What::Jig(what));
            None
        }
    }
}
fn optional_integer(
    object: &mut config::read::Object,
    document: &Document,
    name: &[u8],
    default: u64,
    max: u64,
    path: &Path,
    problems: &mut Problems,
) -> Option<u64> {
    let Some(at) = object.member(document, name) else { return Some(default) };
    match config::read::integer(document, at, max) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), What::Jig(what));
            None
        }
    }
}
fn optional_word(
    object: &mut config::read::Object,
    document: &Document,
    name: &[u8],
    default: &[u8],
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    let Some(at) = object.member(document, name) else { return Some(default.into()) };
    match config::read::text(document, at, 1_048_576) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.member(name), What::Jig(what));
            None
        }
    }
}
fn checked_text(
    object: &mut config::read::Object,
    document: &Document,
    name: &[u8],
    max: u32,
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    let at = read::required(object, document, name, path, problems)?;
    match config::read::text(document, at, max) {
        Ok(value) => {
            read::form(!value.is_empty(), path.member(name), problems);
            Some(value)
        }
        Err(what) => {
            problems.push(path.member(name), What::Jig(what));
            None
        }
    }
}
// The merge command is relative, with no parent segment or NUL
// (temper/docs/design/protocol/configuration.md, 3).
fn optional_merge_job(
    object: &mut config::read::Object,
    document: &Document,
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    let at = object.member(document, b"merge_job")?;
    let value = match config::read::text(document, at, 4_096) {
        Ok(value) => value,
        Err(what) => {
            problems.push(path.member(b"merge_job"), What::Jig(what));
            return None;
        }
    };
    let mut valid = !value.is_empty() && value.first() != Some(&b'/') && !value.contains(&0);
    let mut length = 0_usize;
    let mut dots = true;
    for byte in &value {
        if *byte == b'/' {
            valid &= length != 2 || !dots;
            length = 0;
            dots = true;
        } else {
            length = length.saturating_add(1);
            dots &= *byte == b'.';
        }
    }
    valid &= length != 2 || !dots;
    read::form(valid, path.member(b"merge_job"), problems);
    Some(value)
}
// Ignored paths are whole segments; no empty segment, dot or parent segment.
fn valid_ignored_path(value: &[u8]) -> bool {
    let mut length = 0_usize;
    let mut dots = true;
    for byte in value {
        if *byte == b'/' {
            if length == 0 || (dots && length <= 2) {
                return false;
            }
            length = 0;
            dots = true;
        } else {
            length = length.saturating_add(1);
            dots &= *byte == b'.';
        }
    }
    length > 0 && (!dots || length > 2)
}

struct Check {
    name: Box<[u8]>,
    workflow: Box<[u8]>,
    ignored: Box<[Box<[u8]>]>,
}
fn check(
    object: &mut config::read::Object,
    document: &Document,
    path: &Path,
    limits: &temper_engine_domain_forge::Limits,
    problems: &mut Problems,
) -> Option<Check> {
    let name = checked_text(
        object,
        document,
        b"name",
        u32::try_from(CHECK_NAME_BYTES).expect("check name bound"),
        path,
        problems,
    );
    let workflow = checked_text(
        object,
        document,
        b"workflow",
        u32::try_from(WORKFLOW_BYTES).expect("workflow bound"),
        path,
        problems,
    );
    let ats = optional_list(object, document, b"ignored", limits.ignored_paths, path, problems);
    let mut ignored = List::with_capacity(limits.ignored_paths);
    if let Some(ats) = ats {
        for (index, at) in ats.iter().enumerate() {
            let path = read::index(&path.member(b"ignored"), index);
            match config::read::text(document, *at, limits.name_bytes) {
                Ok(value) => {
                    read::form(valid_ignored_path(&value), path, problems);
                    ignored.push(value).expect("bounded ignored paths");
                }
                Err(what) => problems.push(path, What::Jig(what)),
            }
        }
    }
    let name = name?;
    for byte in &name {
        if !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_' {
            read::form(false, path.member(b"name"), problems);
            break;
        }
    }
    let workflow = workflow?;
    read::form(!workflow.contains(&b'/'), path.member(b"workflow"), problems);
    Some(Check { name, workflow, ignored: ignored.into_boxed() })
}
fn tier(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &temper_engine_domain_forge::Limits,
    problems: &mut Problems,
) -> Option<engine::TierCheck> {
    let mut object = read::object(document, at, path, problems)?;
    let check = check(&mut object, document, path, limits, problems);
    let artifact = checked_text(
        &mut object,
        document,
        b"artifact",
        u32::try_from(CHECK_NAME_BYTES).expect("artifact bound"),
        path,
        problems,
    );
    let format = read::text(&mut object, document, b"format", path, problems);
    let follows = match object.member(document, b"branch") {
        Some(at) => match config::read::text(document, at, 1_048_576) {
            Ok(value) => {
                read::form(!value.is_empty(), path.member(b"branch"), problems);
                Some(if value.as_ref() == b"goal" {
                    temper_engine_domain_forge::Follows::Goals
                } else {
                    temper_engine_domain_forge::Follows::Branch(value)
                })
            }
            Err(what) => {
                problems.push(path.member(b"branch"), What::Jig(what));
                None
            }
        },
        None => Some(temper_engine_domain_forge::Follows::Landing),
    };
    let span_ms = optional_integer(&mut object, document, b"span_ms", 0, u64::MAX, path, problems);
    read::finish(object, document, path, problems);
    if let Some(artifact) = &artifact {
        read::form(!artifact.contains(&b'/'), path.member(b"artifact"), problems);
    }
    let format = match format.as_deref() {
        Some(b"junit") => Some(Format::Junit),
        Some(b"lines") => Some(Format::Lines),
        Some(_) => {
            read::form(false, path.member(b"format"), problems);
            None
        }
        None => None,
    };
    let check = check?;
    Some(engine::TierCheck {
        name: check.name,
        workflow: check.workflow,
        ignored: check.ignored,
        artifact: artifact?,
        format: format?,
        follows: follows?,
        span_ms: span_ms?,
    })
}
fn repository_checks(
    document: &Document,
    at: u32,
    path: &Path,
    limits: &temper_engine_domain_forge::Limits,
    problems: &mut Problems,
) -> Option<engine::RepositoryChecks> {
    let mut object = read::object(document, at, path, problems)?;
    let forge = read::short(&mut object, document, b"forge", path, problems);
    let repository = read::number(&mut object, document, b"repository", path, problems);
    let gate_ats = optional_list(&mut object, document, b"gates", limits.gates, path, problems);
    let tier_ats = optional_list(&mut object, document, b"tiers", limits.tiers.tiers, path, problems);
    read::finish(object, document, path, problems);
    let mut gates = List::with_capacity(limits.gates);
    if let Some(ats) = gate_ats {
        for (index, at) in ats.iter().enumerate() {
            let path = read::index(&path.member(b"gates"), index);
            let Some(mut object) = read::object(document, *at, &path, problems) else { continue };
            let check = check(&mut object, document, &path, limits, problems);
            let blocking = read::boolean(&mut object, document, b"blocking", &path, problems);
            read::finish(object, document, &path, problems);
            if let Some(check) = check
                && let Some(blocking) = blocking
            {
                for previous in &gates {
                    let previous: &engine::GateCheck = previous;
                    if previous.name == check.name {
                        problems.push(path.member(b"name"), What::Jig(config::What::Unique));
                    }
                }
                gates
                    .push(engine::GateCheck {
                        name: check.name,
                        workflow: check.workflow,
                        ignored: check.ignored,
                        blocking,
                    })
                    .expect("bounded gates");
            }
        }
    }
    let mut tiers = List::with_capacity(limits.tiers.tiers);
    if let Some(ats) = tier_ats {
        for (index, at) in ats.iter().enumerate() {
            let path = read::index(&path.member(b"tiers"), index);
            if let Some(tier) = tier(document, *at, &path, limits, problems) {
                for previous in &gates {
                    if previous.name == tier.name {
                        problems.push(path.member(b"name"), What::Jig(config::What::Unique));
                    }
                }
                for previous in &tiers {
                    let previous: &engine::TierCheck = previous;
                    if previous.name == tier.name {
                        problems.push(path.member(b"name"), What::Jig(config::What::Unique));
                    }
                }
                tiers.push(tier).expect("bounded tiers");
            }
        }
    }
    Some(engine::RepositoryChecks {
        repository: skein_forge::api::Repository { forge: forge?, repository: repository? },
        gates: gates.into_boxed(),
        tiers: tiers.into_boxed(),
    })
}
fn project_landing(
    object: &mut config::read::Object,
    document: &Document,
    path: &Path,
    limits: &temper_engine_domain_forge::Limits,
    problems: &mut Problems,
) -> Option<engine::ProjectLanding> {
    let retries = optional_integer(object, document, b"tier_retries", 2, u64::from(u8::MAX), path, problems);
    let bound = optional_integer(object, document, b"verification_bound_ms", 7_200_000, u64::MAX, path, problems);
    let by = optional_word(object, document, b"by", b"queue", path, problems)?;
    let merge_job = optional_merge_job(object, document, path, problems);
    let by = match by.as_ref() {
        b"queue" => engine::Way::Queue,
        b"turn" => engine::Way::Turn,
        _ => {
            read::form(false, path.member(b"by"), problems);
            return None;
        }
    };
    let ats = optional_list(object, document, b"gates", limits.repositories, path, problems)?;
    let mut repositories = List::with_capacity(limits.repositories);
    for (index, at) in ats.iter().enumerate() {
        let path = read::index(&path.member(b"gates"), index);
        if let Some(checks) = repository_checks(document, *at, &path, limits, problems) {
            for previous in &repositories {
                let previous: &engine::RepositoryChecks = previous;
                if previous.repository == checks.repository {
                    problems.push(path.member(b"repository"), What::Jig(config::What::Unique));
                }
            }
            repositories.push(checks).expect("bounded repositories");
        }
    }
    Some(engine::ProjectLanding {
        by,
        merge_job,
        repositories: repositories.into_boxed(),
        after: engine::AfterLandingCheck {
            tier_retries: u8::try_from(retries?).expect("bounded retries"),
            verification_bound_ms: bound?,
        },
    })
}
