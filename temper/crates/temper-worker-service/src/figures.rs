//! Limits taken by name (temper/docs/design/protocol/configuration/figures.md, 3).
mod live;
use crate::startup::{Problems, What, WorkerPart};
use alloc::boxed::Box;
use jig_protocol_config as config;
use jig_protocol_worker_wire::Figures;
pub(crate) use live::{bound_line, plan};
use skein_lib::{Decimal, List, bytes};
use temper_service_core::limits;

/// Apply over stock, retaining unknown names once for the caller's diagnostic.
pub fn apply(part: &mut WorkerPart, figures: Figures) -> Result<List<Box<[u8]>>, Problems> {
    let mut jig = config::stock();
    let mut own = limits::stock();
    let mut unknown: List<Box<[u8]>> = List::uncapped();
    let mut found = Problems::empty();
    for limit in &figures.limits {
        let name = limit.name.as_ref();
        let value = limit.value;
        match read(&mut jig, &mut own, name, value) {
            Ok(()) => {}
            Err(What::Override(limits::Refused::Unknown)) => {
                let mut seen = false;
                for previous in &unknown {
                    if previous.as_ref() == name {
                        seen = true;
                        break;
                    }
                }
                if !seen {
                    unknown.push(bytes::copy_of(name)).expect("uncapped unknown names");
                }
            }
            Err(what) => found.push(figure_path(name, value), what),
        }
    }
    part.worker.jig.slots = figures.places;
    part.worker.jig.stop_bound = match config::stop_bound(&part.worker.jig.graces, jig.link.ping, jig.link.silence) {
        Some(bound) => bound,
        None => {
            found.push(figure_path(b"link.ping_ms", jig.link.ping.as_millis()), What::Jig(config::What::StopBound));
            return Err(found);
        }
    };
    jig.hub.slots = part.worker.jig.slots;
    jig.hub.source_interval = part.worker.jig.check_interval;
    own.checkout.workspaces = part.worker.jig.slots;
    own.git_component.operations = own.checkout.workspaces;
    part.worker.jig.limits = Some(jig);
    part.limits = crate::worker_settings::Limits::shared(own);
    part.call_answer = temper_service_core::startup::CallAnswer::Explicit;
    part.figures = Some(figures);
    if found.is_empty() { Ok(unknown) } else { Err(found) }
}
fn read(jig: &mut config::Limits, own: &mut limits::Shared, name: &[u8], value: u64) -> Result<(), What> {
    if config::names::find(name).is_some() {
        match config::overrides::apply(jig, name, value) {
            Ok(()) => Ok(()),
            Err(what) => Err(What::Jig(what)),
        }
    } else {
        match limits::apply(own, name, value) {
            Ok(()) => Ok(()),
            Err(what) => Err(What::Override(what)),
        }
    }
}

fn figure_path(name: &[u8], value: u64) -> config::Path {
    config::Path::root().member(&bytes::concat(&[b"the engine's figures: ", name, b" ", Decimal::of(value).as_bytes()]))
}
/// Keep the checks, naming the engine's scalar rather than a removed document member.
#[must_use]
pub fn problems(mut found: Problems, figures: &Figures) -> Problems {
    for problem in found.problems.iter_mut() {
        let name = match problem.path.segments.iter().next_back() {
            Some(config::Segment::Member(name)) => Some(name.as_ref()),
            Some(config::Segment::Index(_)) | None => None,
        };
        let mut changed = false;
        if let Some(name) = name {
            for limit in figures.limits.iter().chain(figures.agents.profile.iter()) {
                if limit.name.as_ref() == name
                    || (name == b"tool_payload" && limit.name.as_ref() == b"declared.tool_payload")
                {
                    problem.path = figure_path(limit.name.as_ref(), limit.value);
                    changed = true;
                    break;
                }
            }
        }
        if !changed {
            problem.path = config::Path::root().member(b"the engine's figures");
        }
    }
    found
}
