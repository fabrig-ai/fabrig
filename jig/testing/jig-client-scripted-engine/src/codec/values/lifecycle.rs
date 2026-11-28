//! Wire projections of attempts and holds (client/domain/testing.md, 4).
use super::super::{c, facts, w};
use super::{wire_hold_category, wire_part, wire_prior_phase, wire_run_end};
pub(in crate::codec) fn wire_attempt_line(value: &c::AttemptLine) -> w::AttemptLine {
    w::AttemptLine {
        task: value.task,
        attempt: value.attempt,
        claimed: value.claimed.map(skein_lib::Wall::as_nanos),
        started: value.started.map(skein_lib::Wall::as_nanos),
        ended: value.ended.map(skein_lib::Wall::as_nanos),
        end: value.end.as_ref().map(wire_run_end),
        phases: value.phases,
        sources: value.sources.as_ref().map(|source| w::RunSources {
            build: source.build.as_bytes().into(),
            settings: source.settings,
            charters: source.charters,
            source: source.source.clone(),
            guides: source.guides.as_ref().map(|guides| {
                skein_lib::List::from_boxed(
                    guides
                        .iter()
                        .map(|guide| w::Guide {
                            repository: guide.repository.as_bytes().into(),
                            commit: guide.commit.as_bytes().into(),
                            path: guide.path.as_bytes().into(),
                            digest: guide.digest,
                            bytes: guide.bytes,
                            whole: guide.whole,
                        })
                        .collect(),
                )
            }),
        }),
    }
}
pub(in crate::codec) fn wire_hold_line(value: &c::HoldLine, limits: &w::Limits) -> Result<w::HoldLine, w::Problem> {
    Ok(w::HoldLine {
        task: value.task,
        position: value.position,
        at: value.at.as_nanos(),
        before: wire_prior_phase(&value.before),
        why: wire_hold_category(&value.why),
        source: value.source.as_ref().map(wire_part),
        ending: value.ending.as_ref().map(|item| facts::chip_ending(item, limits)).transpose()?,
    })
}
