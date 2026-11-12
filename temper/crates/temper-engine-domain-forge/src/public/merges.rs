//! Public queue merges from the landing log (temper/docs/design/client/forge/README.md, 9.2).
use super::{Names, PublicAsk, w};
use crate::{ChangeRow, Domain, Limits, LoggedLanding};
use core::cmp::Reverse;
use skein_forge::api::{Commit, Repository};
use skein_lib::{List, Map, Wall};
use temper_engine_domain_forge_change::Way;

type MergeKey = (Reverse<Wall>, Repository, Commit);

impl Domain {
    /// The project's public queue merges, newest first.
    pub(super) fn queue_merges(&self, ask: &PublicAsk<'_>, limits: &Limits) -> Option<Map<MergeKey, &LoggedLanding>> {
        let mut recent = Map::with_capacity(limits.repositories.checked_mul(limits.landings_kept)?);
        for (repository, log) in &self.landing_logs {
            if self.repositories.get(repository)?.project != ask.project {
                continue;
            }
            for logged in &log.landings {
                if self.public_merge(ask, logged) {
                    recent.insert((Reverse(logged.at), *repository, logged.commit), logged).ok()?;
                }
            }
        }
        Some(recent)
    }

    /// Every member handed, at least one, and retained rows of the project and by the queue.
    fn public_merge(&self, ask: &PublicAsk<'_>, logged: &LoggedLanding) -> bool {
        if logged.changes.is_empty() {
            return false;
        }
        for change in &logged.changes {
            if ask.handed(change.task).is_none() {
                return false;
            }
            if let Some(row) = self.changes.get(&change.task)
                && (self.public_change(ask, change.task).is_none() || row.change.way != Way::Queue)
            {
                return false;
            }
            // A dropped member is named by the log; no retained queue or proof facts remain.
        }
        true
    }

    #[expect(clippy::unused_self, reason = "the public merge projection is a Domain method")]
    pub(super) fn merge_line(&self, logged: &LoggedLanding, names: &mut Names) -> Option<w::PublicLandingLine> {
        let l = &w::CEILINGS;
        let mut members = List::with_capacity(l.public_landing_line_members);
        for change in &logged.changes {
            members.push(names.name(change.task)?).ok()?;
        }
        let mut landed = List::with_capacity(l.public_landing_line_landed);
        landed.push(logged.commit).ok()?;
        w::PublicLandingLine::new(
            l,
            w::PublicLandingLineParts { members, phase: w::PublicLandingPhase::Landed, landed },
        )
        .ok()
    }

    /// The merge `change` is in, read from its repository's log; `None` is unknown.
    pub(super) fn make_merge(
        &self,
        ask: &PublicAsk<'_>,
        change: &ChangeRow,
        names: &mut Names,
    ) -> Option<w::PublicLanding> {
        let log = self.landing_logs.get(&change.repository)?;
        for logged in &log.landings {
            for member in &logged.changes {
                if member.task != change.task {
                    continue;
                }
                if !self.public_merge(ask, logged) {
                    return None;
                }
                let l = &w::CEILINGS;
                let mut members = List::with_capacity(l.public_landing_members);
                for member in &logged.changes {
                    let member = w::PublicMember::new(
                        l,
                        w::PublicMemberParts {
                            task: names.name(member.task)?,
                            base: None,
                            candidate: None,
                            landed: Some(logged.commit),
                            gates: List::with_capacity(0),
                        },
                    )
                    .ok()?;
                    members.push(member).ok()?;
                }
                return w::PublicLanding::new(
                    l,
                    w::PublicLandingParts {
                        phase: w::PublicLandingPhase::Landed,
                        hold: None,
                        preparation: 0,
                        members,
                        stale_proofs: 0,

                        round: temper_engine_domain_forge_landing::Round::new(Box::from(b"temper/1/l61".as_slice())),
                        holds_turn: false,
                    },
                )
                .ok();
            }
        }
        None
    }
}
