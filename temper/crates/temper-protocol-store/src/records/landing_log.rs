//! Family 30's bounded translations (temper/docs/design/protocol/store.md, 2).
use super::{FITS, names, w};
use alloc::boxed::Box;
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
                        w::LoggedChangeParts {
                            task: change.task,
                            pull: change.pull,
                            title: change.title.clone(),
                            signed: encode_signed(&change.signed),
                        },
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
            let w::LoggedChangeParts { task, pull, title, signed } = change.into_parts();
            members.push(d::LoggedChange { task, pull, title, signed: decode_signed(signed) }).expect(FITS);
        }
        decoded
            .push(d::LoggedLanding { commit, branch, at: Wall::from_nanos(at), changes: members.into_boxed() })
            .expect(FITS);
    }
    d::LandingLog { repository: names::decode_repository(repository), landings: decoded.into_boxed() }
}

pub(super) fn encode_signed(signed: &[d::SignOff]) -> List<w::SignOff> {
    let mut result = List::with_capacity(u32::try_from(d::SIGN_OFFS).expect(FITS));
    for sign in signed {
        result
            .push(
                w::SignOff::new(
                    &w::CEILINGS,
                    w::SignOffParts { person: sign.person, head: sign.head, at: sign.at.as_nanos() },
                )
                .expect(FITS),
            )
            .expect(FITS);
    }
    result
}

pub(super) fn decode_signed(signed: List<w::SignOff>) -> Box<[d::SignOff]> {
    let mut result = List::with_capacity(signed.len());
    for sign in signed.into_boxed() {
        let w::SignOffParts { person, head, at } = sign.into_parts();
        result.push(d::SignOff { person, head, at: Wall::from_nanos(at) }).expect(FITS);
    }
    result.into_boxed()
}
