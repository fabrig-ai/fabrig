//! Limits taken by name (temper/docs/design/protocol/configuration/figures.md, 3).
use crate::startup::{Problems, What, WorkerPart};
use alloc::boxed::Box;
use jig_protocol_config as config;
use jig_protocol_worker_wire::Figures;
use skein_lib::{Decimal, List, bytes};
use temper_service_core::limits;

/// Apply over stock, retaining unknown names once for the caller's diagnostic.
pub fn apply(part: &mut WorkerPart, figures: Figures) -> Result<List<Box<[u8]>>, Problems> {
    let mut jig = config::profile(config::Profile::Standard);
    let mut own = limits::profile(limits::Profile::Standard);
    let names = config::overrides::list(&jig);
    let mut unknown: List<Box<[u8]>> = List::uncapped();
    let mut found = Problems::empty();
    for limit in figures.limits() {
        let name = limit.name();
        let value = limit.value();
        let mut known = false;
        for &(candidate, _) in &names {
            if candidate.as_bytes() == name {
                known = true;
                break;
            }
        }
        let refusal = if known {
            match config::overrides::apply(&mut jig, name, value) {
                Ok(()) => None,
                Err(what) => Some(What::Jig(what)),
            }
        } else {
            match limits::apply(&mut own, name, value) {
                Ok(()) => None,
                Err(limits::Refused::Range) => Some(What::Override(limits::Refused::Range)),
                Err(limits::Refused::Unknown) => {
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
                    None
                }
            }
        };
        if let Some(what) = refusal {
            found.push(figure_path(name, value), what);
        }
    }
    part.worker.jig.slots = part.worker.jig.slots.min(figures.places());
    part.worker.jig.stop_bound = match config::stop_bound(&part.worker.jig.graces, jig.link.ping, jig.link.silence) {
        Some(bound) => bound,
        None => {
            found.push(figure_path(b"link.ping_ms", jig.link.ping.as_millis()), What::Jig(config::What::StopBound));
            return Err(found);
        }
    };
    part.worker.jig.limits = Some(jig);
    part.limits = crate::worker_settings::Limits::shared(own);
    part.call_answer = temper_service_core::startup::CallAnswer::Explicit;
    part.figures = Some(figures);
    if found.is_empty() { Ok(unknown) } else { Err(found) }
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
            for limit in figures.limits().iter().chain(figures.agents().profile().iter()) {
                if limit.name() == name || (name == b"tool_payload" && limit.name() == b"declared.tool_payload") {
                    problem.path = figure_path(limit.name(), limit.value());
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
