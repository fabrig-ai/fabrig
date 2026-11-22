//! Findings at the client boundary (temper/docs/design/client/forge/records.md; domain/areas.md, 5 and 7).
use super::{forge, root, w};
use alloc::boxed::Box;
use skein_lib::List;
use temper_client_protocol_wire::v1::Record;

pub(super) fn request(value: w::Request) -> Option<root::Application> {
    match value {
        w::Request::FileFinding(value) => Some(root::Application::FileFinding(forge::FileFinding {
            project: value.project,
            kind: kind_from(value.kind),
            severity: severity_from(value.severity),
            area: value.area,
            title: value.title,
            body: value.body,
        })),
        w::Request::HandFinding(value) => Some(root::Application::HandFinding(forge::HandFinding {
            project: value.project,
            number: value.number,
            area: value.area,
            why: value.why,
        })),
        w::Request::TriageFinding(value) => Some(root::Application::TriageFinding(forge::TriageFinding {
            project: value.project,
            number: value.number,
            act: match value.act {
                w::Triage::Route(value) => forge::Triage::Route(value.task),
                w::Triage::Backlog => forge::Triage::Backlog,
                w::Triage::Drop(value) => forge::Triage::Drop(value.reason),
                w::Triage::Repair(value) => {
                    forge::Triage::Repair(forge::RepairAct { more: value.more.into_boxed(), words: value.words })
                }
                w::Triage::Blame(value) => {
                    let landing = match value.landing {
                        Some(bytes) => {
                            let mut commit = [0; 32];
                            for (cell, byte) in commit.iter_mut().zip(&bytes) {
                                *cell = *byte;
                            }
                            Some(commit)
                        }
                        None => None,
                    };
                    forge::Triage::Blame(forge::BlameAct { landing, why: value.why })
                }
            },
        })),
        w::Request::Adopt(_)
        | w::Request::SetUp(_)
        | w::Request::Connect(_)
        | w::Request::Install(_)
        | w::Request::Reachable(_)
        | w::Request::Accept(_)
        | w::Request::RemoveForge(_)
        | w::Request::StartPlayground
        | w::Request::EndPlayground
        | w::Request::Replace(_)
        | w::Request::Dismiss(_)
        | w::Request::MakeRepository(_)
        | w::Request::DismissMaking(_)
        | w::Request::Looked(_)
        | w::Request::SetAway(_)
        | w::Request::Overturn(_)
        | w::Request::SetDraft(_)
        | w::Request::OpenChange(_)
        | w::Request::TierAct(_)
        | w::Request::FollowAct(_)
        | w::Request::AddDevice(_)
        | w::Request::RemoveDevice(_)
        | w::Request::Checkpoint
        | w::Request::TryAccount(_) => None,
    }
}

pub(super) fn filter(value: &w::FindingsFilter) -> forge::FindingsFilter {
    match value {
        w::FindingsFilter::Triage(value) => forge::FindingsFilter::Triage {
            state: match &value.state {
                Some(state) => Some(state_from(state.clone())),
                None => None,
            },
            mine: value.mine,
        },
        w::FindingsFilter::Goal(value) => forge::FindingsFilter::Goal(value.goal),
        w::FindingsFilter::Milestone(value) => forge::FindingsFilter::Milestone(value.milestone),
    }
}

fn kind_from(value: w::FindingKind) -> forge::FindingKind {
    match value {
        w::FindingKind::Qa => forge::FindingKind::Qa,
        w::FindingKind::Flake => forge::FindingKind::Flake,
        w::FindingKind::Red => forge::FindingKind::Red,
        w::FindingKind::Review => forge::FindingKind::Review,
        w::FindingKind::Feedback => forge::FindingKind::Feedback,
    }
}

fn kind_to(value: forge::FindingKind) -> w::FindingKind {
    match value {
        forge::FindingKind::Qa => w::FindingKind::Qa,
        forge::FindingKind::Flake => w::FindingKind::Flake,
        forge::FindingKind::Red => w::FindingKind::Red,
        forge::FindingKind::Review => w::FindingKind::Review,
        forge::FindingKind::Feedback => w::FindingKind::Feedback,
    }
}

fn severity_from(value: w::Severity) -> forge::Severity {
    match value {
        w::Severity::Low => forge::Severity::Low,
        w::Severity::Medium => forge::Severity::Medium,
        w::Severity::High => forge::Severity::High,
    }
}

fn severity_to(value: forge::Severity) -> w::Severity {
    match value {
        forge::Severity::Low => w::Severity::Low,
        forge::Severity::Medium => w::Severity::Medium,
        forge::Severity::High => w::Severity::High,
    }
}

fn state_from(value: w::FindingState) -> forge::FindingState {
    match value {
        w::FindingState::New => forge::FindingState::New,
        w::FindingState::Routed => forge::FindingState::Routed,
        w::FindingState::Backlog => forge::FindingState::Backlog,
        w::FindingState::Dropped => forge::FindingState::Dropped,
    }
}

fn state_to(value: forge::FindingState) -> w::FindingState {
    match value {
        forge::FindingState::New => w::FindingState::New,
        forge::FindingState::Routed => w::FindingState::Routed,
        forge::FindingState::Backlog => w::FindingState::Backlog,
        forge::FindingState::Dropped => w::FindingState::Dropped,
    }
}

fn holder(value: forge::Holder, limits: &w::Limits) -> Option<w::Holder> {
    let value = {
        Some(match value {
            forge::Holder::Task(number) => w::Holder::Task(w::HolderTask { number }),
            forge::Holder::Party(number) => w::Holder::Party(w::HolderParty { number }),
            forge::Holder::Role(number) => w::Holder::Role(w::HolderRole { number }),
        })
    }?;
    value.measure(limits).ok()?;
    Some(value)
}

fn owner(value: forge::settings::AreaOwner, limits: &w::Limits) -> Option<w::AreaOwner> {
    let value = {
        use forge::settings::AreaOwner;
        Some(match value {
            AreaOwner::Task(number) => w::AreaOwner::Task(w::AreaTask { number }),
            AreaOwner::Party(number) => w::AreaOwner::Party(w::AreaParty { number }),
            AreaOwner::Role(number) => w::AreaOwner::Role(w::AreaRole { number }),
            AreaOwner::Charter(number) => w::AreaOwner::Charter(w::AreaCharter { number }),
        })
    }?;
    value.measure(limits).ok()?;
    Some(value)
}

fn refused(value: forge::FindingRefusal, limits: &w::Limits) -> Option<w::Outcome> {
    let reason = match value {
        forge::FindingRefusal::NoArea(names) => {
            let mut areas = List::with_capacity(limits.finding_areas_areas);
            for name in names {
                areas.push(name).ok()?;
            }
            w::FindingRefusal::NoArea(w::FindingAreas { areas })
        }
        forge::FindingRefusal::Full(count) => w::FindingRefusal::Full(w::FindingsFull { count }),
        forge::FindingRefusal::NotYours(value) => {
            w::FindingRefusal::NotYours(w::FindingNotYours { holder: holder(value, limits)? })
        }
        forge::FindingRefusal::NoFinding => w::FindingRefusal::NoFinding,
        forge::FindingRefusal::NoGoal => w::FindingRefusal::NoGoal,
        forge::FindingRefusal::Ended => w::FindingRefusal::Ended,
        forge::FindingRefusal::Words(field) => w::FindingRefusal::Words(w::FindingWords { field }),
    };
    Some(w::Outcome::FindingRefused(w::FindingRefused { reason }))
}

pub(super) fn file_answer(value: forge::FileFindingAnswer, limits: &w::Limits) -> Option<w::Outcome> {
    match value {
        forge::FileFindingAnswer::Filed { number } => Some(w::Outcome::Filed(w::Filed { number })),
        forge::FileFindingAnswer::Refused(value) => refused(value, limits),
    }
}
pub(super) fn hand_answer(value: forge::HandFindingAnswer, limits: &w::Limits) -> Option<w::Outcome> {
    match value {
        forge::HandFindingAnswer::Handed => Some(w::Outcome::Handed),
        forge::HandFindingAnswer::Refused(value) => refused(value, limits),
    }
}
pub(super) fn triage_answer(value: forge::TriageFindingAnswer, limits: &w::Limits) -> Option<w::Outcome> {
    match value {
        forge::TriageFindingAnswer::Triaged => Some(w::Outcome::Triaged(w::Triaged { goal: None })),
        forge::TriageFindingAnswer::Repaired(goal) => Some(w::Outcome::Triaged(w::Triaged { goal: Some(goal) })),
        forge::TriageFindingAnswer::Refused(value) => refused(value, limits),
    }
}

fn row(value: forge::FindingRow, limits: &w::Limits) -> Option<w::FindingRow> {
    {
        let value = w::FindingRow {
            number: value.number,
            kind: kind_to(value.kind),
            severity: severity_to(value.severity),
            area: value.area,
            state: state_to(value.state),
            title: value.title,
            holder: holder(value.holder, limits)?,
            at: value.at.as_nanos(),
            why: value.why,
            goal: value.goal,
            reason: value.reason,
            cleared: match value.cleared {
                Some(value) => Some(w::FindingClearing { tier: value.tier, run: value.run, at: value.at.as_nanos() }),
                None => None,
            },
            hander: match value.hander {
                Some(value) => Some(filer(value, limits)?),
                None => None,
            },
        };
        value.measure(limits).ok()?;
        Some(value)
    }
}

pub(super) fn document(value: forge::Findings, limits: &w::Limits) -> Option<Box<[u8]>> {
    let mut rows = List::with_capacity(limits.findings_rows);
    for value in value.rows {
        rows.push(row(value, limits)?).ok()?;
    }
    let mut areas = List::with_capacity(limits.findings_areas);
    for value in value.areas {
        let holder = match value.holder {
            Some(value) => Some(holder(value, limits)?),
            None => None,
        };
        areas
            .push(w::AreaRow { name: value.name, words: value.words, owner: owner(value.owner, limits)?, holder })
            .ok()?;
    }
    let triager = match value.triager {
        Some(value) => Some(owner(value, limits)?),
        None => None,
    };
    let record = w::Findings {
        project: value.project,
        rows,
        areas,
        triager,
        holder: holder(value.holder, limits)?,
        cut: value.cut,
    };
    record.encode(limits).ok()
}

pub(super) fn read_answer(value: Option<Box<forge::FindingAnswer>>, limits: &w::Limits) -> Option<Box<[u8]>> {
    let result = match value {
        None => w::FindingResult::NoFinding,
        Some(value) => {
            let filer = filer(value.filer, limits)?;
            w::FindingResult::Finding(Box::new(w::FindingDetail {
                row: row(value.row, limits)?,
                body: value.body,
                filer,
                blame: match value.blame {
                    Some(value) => Some(blame(value, limits)?),
                    None => None,
                },
            }))
        }
    };
    let record = w::FindingAnswer { result };
    record.encode(limits).ok()
}

fn blame(value: forge::FindingBlame, limits: &w::Limits) -> Option<w::FindingBlame> {
    let strength = match value.strength {
        forge::BlameStrength::Owned => w::BlameStrength::Owned,
        forge::BlameStrength::Suspected => w::BlameStrength::Suspected,
        forge::BlameStrength::Unblamed => w::BlameStrength::Unblamed,
        forge::BlameStrength::Moved => w::BlameStrength::Moved,
    };
    let mut landings = List::with_capacity(limits.finding_blame_landings);
    for landing in value.landings {
        let mut goals = List::with_capacity(limits.blamed_landing_goals);
        for goal in landing.goals {
            goals.push(w::BlamedGoal { goal: goal.goal, milestone: goal.milestone, person: goal.person }).ok()?;
        }
        landings.push(w::BlamedLanding { commit: landing.commit, goals }).ok()?;
    }
    let blame = w::FindingBlame { strength, landings };
    blame.measure(limits).ok()?;
    Some(blame)
}

fn filer(value: forge::Filer, limits: &w::Limits) -> Option<w::Filer> {
    let value = {
        Some(match value {
            forge::Filer::Task(number) => w::Filer::Task(w::HolderTask { number }),
            forge::Filer::Party(number) => w::Filer::Party(w::HolderParty { number }),
        })
    }?;
    value.measure(limits).ok()?;
    Some(value)
}
