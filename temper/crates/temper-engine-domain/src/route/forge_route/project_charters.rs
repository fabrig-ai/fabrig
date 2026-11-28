//! Give jig the project's charter set (temper/docs/design/domain/agent/project-charters.md, 3–4).
use alloc::boxed::Box;
use jig_domain_contract::{Up, lifecycle};
use jig_domain_core::{CharterPart, Event, ProjectCharter, ProjectCharters};
use jig_domain_tasks::{REASON_NAMES, Reasons};
use skein_lib::{Env, List, bytes};

use super::{Decision, Domain, Limits, commit_text, forge, repository_name};
use crate::{Delivery, Fact, ProjectChartersDiagnostic, ProjectChartersFault};

const WAYS_TOML: &[u8] = b".temper/ways.toml";

pub(super) fn moved(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, row: &forge::DotTemperRevision) {
    let offered = has_ways(row);
    if offered || domain.engine.core().project_charters(row.project) != 0 {
        let set = read(domain, row, offered);
        give(domain, env, decision, set);
    }
}

pub(super) fn begin_restore(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision) {
    if let Some(project) = next(domain, 0) {
        resume(env, decision, Some(project));
    } else {
        restored(domain);
    }
}

/// Each backfill gets its own decision; the core advances past the forge only
/// once the last of these decisions is durable.
pub(crate) fn restore(domain: &mut Domain, env: &Env<Limits>, decision: &mut Decision, project: Option<u32>) {
    if domain.engine.core().stopping() || domain.settling.active() {
        return;
    }
    if let Some(project) = project {
        let row = domain.forge.temper_revision(project).expect("the loaded revision waits for this backfill");
        let set = read(domain, row, true);
        give(domain, env, decision, set);
        resume(env, decision, next(domain, project));
    } else {
        restored(domain);
    }
}

fn next(domain: &Domain, mut after: u32) -> Option<u32> {
    while let Some(row) = domain.forge.next_temper_revision(after) {
        if has_ways(row) && row.number != domain.engine.core().project_charters(row.project) {
            return Some(row.project);
        }
        after = row.project;
    }
    None
}

fn resume(env: &Env<Limits>, decision: &mut Decision, project: Option<u32>) {
    super::emit(decision, &env.limits, Delivery::Resume(crate::StartResume::ProjectCharters(project)));
}

fn restored(domain: &mut Domain) {
    crate::route::forge_up(domain, Up::Lifecycle(lifecycle::Up::Done { stage: forge::RestartStage::Restored }));
}

fn has_ways(row: &forge::DotTemperRevision) -> bool {
    for file in &row.files {
        if file.path.as_ref() == WAYS_TOML {
            return true;
        }
    }
    false
}

fn give(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    set: Result<Option<ProjectCharters>, ProjectChartersDiagnostic>,
) {
    match set {
        Ok(Some(set)) => {
            let out = super::step_core(domain, env, Event::ProjectCharters(set));
            crate::route::route_engine_requests(domain, env, decision, out);
        }
        Ok(None) => {}
        Err(fault) => super::super::diagnostics::emit(domain, Fact::ProjectCharters(fault)),
    }
}

fn read(
    domain: &Domain,
    row: &forge::DotTemperRevision,
    offered: bool,
) -> Result<Option<ProjectCharters>, ProjectChartersDiagnostic> {
    let Some(commit) = &row.commit else { return Ok(None) };
    let Some(home) = domain.forge.repository(row.home) else { return Ok(None) };
    if home.project != row.project || domain.forge.home(row.project).is_none() {
        return Ok(None);
    }
    let charters = if offered { read_files(row)? } else { Box::new([]) };
    let digits = commit_text(commit);
    let prefix = digits.get(..12).expect("a commit has at least twelve hex digits");
    let source = bytes::concat(&[&repository_name(home), b" at ", prefix]);
    Ok(Some(ProjectCharters { project: row.project, revision: row.number, source, charters }))
}

#[expect(clippy::disallowed_methods, reason = "the shared charter reader takes the text of a judged file")]
fn read_files(row: &forge::DotTemperRevision) -> Result<Box<[ProjectCharter]>, ProjectChartersDiagnostic> {
    let mut reasons = [""; REASON_NAMES.len()];
    for (text, name) in reasons.iter_mut().zip(REASON_NAMES) {
        *text = core::str::from_utf8(name).expect("jig's reason names are ASCII");
    }
    let mut charters = List::uncapped();
    for file in &row.files {
        let Some(path) = file.path.strip_prefix(b".temper/charters/") else { continue };
        let Some(name) = path.strip_suffix(b".md") else { continue };
        if name.contains(&b'/') {
            continue;
        }
        let read = match core::str::from_utf8(&file.bytes) {
            Ok(text) => match temper_charter_set::read_charter(text, &reasons) {
                Ok(read) => read,
                Err(fault) => {
                    return Err(unreadable(row, file, fault.line, alloc::fmt::format(format_args!("{fault}")).into()));
                }
            },
            Err(_error) => return Err(unreadable(row, file, 0, "not UTF-8".into())),
        };
        let settings = match read.settings {
            Some(settings) => settings.into_boxed_bytes(),
            None => Box::from(name),
        };
        let mut parts = List::with_capacity(u32::try_from(read.parts.len()).expect("parts are bounded by file bytes"));
        for part in read.parts {
            let mut on = Reasons::EMPTY;
            for reason in part.reasons {
                on = on.union(Reasons::named(reason.as_bytes()).expect("the reader checked jig's reason names"));
            }
            parts.push(CharterPart { on, instructions: part.text.into_boxed_bytes() }).expect("one part per heading");
        }
        let charter = ProjectCharter {
            name: Box::from(name),
            settings,
            instructions: read.instructions.into_boxed_bytes(),
            parts: parts.into_boxed(),
        };
        charters.push(charter).expect("one charter per file");
    }
    Ok(charters.into_boxed())
}

fn unreadable(
    row: &forge::DotTemperRevision,
    file: &forge::DotTemperFile,
    line: u32,
    why: Box<str>,
) -> ProjectChartersDiagnostic {
    ProjectChartersDiagnostic {
        project: row.project,
        revision: row.number,
        fault: ProjectChartersFault::Unreadable { path: file.path.clone(), line, why },
    }
}
