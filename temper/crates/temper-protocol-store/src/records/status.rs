//! Status families (temper/docs/design/protocol/store.md, 2).
use super::w;
use skein_lib::Wall;
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode_facts(row: &d::ProjectFacts) -> w::ProjectFacts {
    w::ProjectFacts { project: row.project, landed: row.landed, last_landed: row.last_landed.as_nanos() }
}
pub(super) fn decode_facts(row: w::ProjectFacts) -> d::ProjectFacts {
    let w::ProjectFacts { project, landed, last_landed } = row;
    d::ProjectFacts { project, landed, last_landed: Wall::from_nanos(last_landed) }
}

pub(super) fn encode_person_brief(row: &d::PersonBrief) -> w::PersonBrief {
    let brief = match &row.brief {
        Some(brief) => Some(w::StatusBrief {
            sentence: brief.sentence.clone(),
            lede: brief.lede.clone(),
            at: brief.at.as_nanos(),
            landed: brief.landed,
            findings: brief.findings,
        }),
        None => None,
    };
    let running = match &row.running {
        Some(running) => Some(w::StatusRunning {
            task: running.task,
            landed: running.landed,
            since: running.since.as_nanos(),
            findings: running.findings,
        }),
        None => None,
    };
    w::PersonBrief {
        project: row.project,
        party: row.party,
        unpaid: match row.unpaid {
            Some(wall) => Some(wall.as_nanos()),
            None => None,
        },
        brief,
        running,
    }
}

pub(super) fn decode_person_brief(row: w::PersonBrief) -> d::PersonBrief {
    let w::PersonBrief { party, project, brief, running, unpaid } = row;
    let brief = match brief {
        Some(brief) => {
            let w::StatusBrief { sentence, lede, at, landed, findings } = brief;
            Some(d::Brief { sentence, lede, at: Wall::from_nanos(at), landed, findings })
        }
        None => None,
    };
    let running = match running {
        Some(running) => {
            let w::StatusRunning { task, landed, since, findings } = running;
            Some(d::StatusRunning { task, landed, since: Wall::from_nanos(since), findings })
        }
        None => None,
    };
    d::PersonBrief {
        party,
        project,
        brief,
        running,
        unpaid: match unpaid {
            Some(wall) => Some(Wall::from_nanos(wall)),
            None => None,
        },
    }
}

pub(super) fn encode_looked(row: &d::Looked) -> w::Looked {
    let previous = match row.previous {
        Some(wall) => Some(wall.as_nanos()),
        None => None,
    };
    w::Looked {
        party: row.party,
        project: row.project,
        previous,
        latest: row.latest.as_nanos(),
        seen: row.seen,
        next_seen: row.next_seen,
    }
}

pub(super) fn decode_looked(row: w::Looked) -> d::Looked {
    let w::Looked { party, project, previous, latest, seen, next_seen } = row;
    let previous = match previous {
        Some(wall) => Some(Wall::from_nanos(wall)),
        None => None,
    };
    d::Looked { party, project, previous, latest: Wall::from_nanos(latest), seen, next_seen }
}
