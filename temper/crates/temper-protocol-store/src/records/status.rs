//! Status families (temper/docs/design/protocol/store.md, 2).
use super::{FITS, w};
use skein_lib::Wall;
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode_facts(row: &d::ProjectFacts) -> w::ProjectFacts {
    let brief = match &row.brief {
        Some(brief) => Some(
            w::StatusBrief::new(
                &w::CEILINGS,
                w::StatusBriefParts {
                    sentence: brief.sentence.clone(),
                    lede: brief.lede.clone(),
                    at: brief.at.as_nanos(),
                    landed: brief.landed,
                },
            )
            .expect(FITS),
        ),
        None => None,
    };
    let running = match &row.running {
        Some(running) => Some(
            w::StatusRunning::new(
                &w::CEILINGS,
                w::StatusRunningParts { task: running.task, landed: running.landed, since: running.since.as_nanos() },
            )
            .expect(FITS),
        ),
        None => None,
    };
    w::ProjectFacts::new(
        &w::CEILINGS,
        w::ProjectFactsParts {
            project: row.project,
            landed: row.landed,
            last_landed: row.last_landed.as_nanos(),
            brief,
            running,
        },
    )
    .expect(FITS)
}

pub(super) fn decode_facts(row: w::ProjectFacts) -> d::ProjectFacts {
    let w::ProjectFactsParts { project, landed, last_landed, brief, running } = row.into_parts();
    let brief = match brief {
        Some(brief) => {
            let w::StatusBriefParts { sentence, lede, at, landed } = brief.into_parts();
            Some(d::Brief { sentence, lede, at: Wall::from_nanos(at), landed })
        }
        None => None,
    };
    let running = match running {
        Some(running) => {
            let w::StatusRunningParts { task, landed, since } = running.into_parts();
            Some(d::StatusRunning { task, landed, since: Wall::from_nanos(since) })
        }
        None => None,
    };
    d::ProjectFacts { project, landed, last_landed: Wall::from_nanos(last_landed), brief, running }
}

pub(super) fn encode_looked(row: &d::Looked) -> w::Looked {
    let previous = match row.previous {
        Some(wall) => Some(wall.as_nanos()),
        None => None,
    };
    w::Looked::new(
        &w::CEILINGS,
        w::LookedParts {
            party: row.party,
            project: row.project,
            previous,
            latest: row.latest.as_nanos(),
            seen: row.seen,
            next_seen: row.next_seen,
        },
    )
    .expect(FITS)
}

pub(super) fn decode_looked(row: w::Looked) -> d::Looked {
    let w::LookedParts { party, project, previous, latest, seen, next_seen } = row.into_parts();
    let previous = match previous {
        Some(wall) => Some(Wall::from_nanos(wall)),
        None => None,
    };
    d::Looked { party, project, previous, latest: Wall::from_nanos(latest), seen, next_seen }
}
