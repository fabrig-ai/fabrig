//! Family 30's bounded translations (temper/docs/design/protocol/store.md, 2).
use super::{FITS, names, w};
use skein_lib::{List, Wall};
use temper_engine_domain_forge as d;

pub(super) fn encode(row: &d::LandingLog) -> w::LandingLog {
    let mut landings = List::with_capacity(u32::try_from(row.landings.len()).expect(FITS));
    for landing in &row.landings {
        let mut changes = List::with_capacity(u32::try_from(landing.changes.len()).expect(FITS));
        for change in &landing.changes {
            changes
                .push(
                    w::LoggedChange::new(
                        &w::CEILINGS,
                        w::LoggedChangeParts { task: change.task, pull: change.pull, title: change.title.clone() },
                    )
                    .expect(FITS),
                )
                .expect(FITS);
        }
        landings
            .push(
                w::LoggedLanding::new(
                    &w::CEILINGS,
                    w::LoggedLandingParts {
                        commit: landing.commit,
                        branch: landing.branch.clone(),
                        at: landing.at.as_nanos(),
                        changes,
                    },
                )
                .expect(FITS),
            )
            .expect(FITS);
    }
    w::LandingLog::new(
        &w::CEILINGS,
        w::LandingLogParts { repository: names::encode_repository(row.repository), landings },
    )
    .expect(FITS)
}

pub(super) fn decode(row: w::LandingLog) -> d::LandingLog {
    let w::LandingLogParts { repository, landings } = row.into_parts();
    let mut decoded = List::with_capacity(landings.len());
    for landing in landings.into_boxed() {
        let w::LoggedLandingParts { commit, branch, at, changes } = landing.into_parts();
        let mut members = List::with_capacity(changes.len());
        for change in changes.into_boxed() {
            let w::LoggedChangeParts { task, pull, title } = change.into_parts();
            members.push(d::LoggedChange { task, pull, title }).expect(FITS);
        }
        decoded
            .push(d::LoggedLanding { commit, branch, at: Wall::from_nanos(at), changes: members.into_boxed() })
            .expect(FITS);
    }
    d::LandingLog { repository: names::decode_repository(repository), landings: decoded.into_boxed() }
}
