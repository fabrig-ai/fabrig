//! Findings at the client boundary (temper/docs/design/client/forge/records.md; domain/areas.md, 5 and 7).
use super::{forge, root, w, wire};
use alloc::boxed::Box;
use skein_lib::List;

pub(super) fn request(value: w::Request) -> Option<root::Application> {
    match value {
        w::Request::FileFinding(value) => {
            let value = value.into_parts();
            Some(root::Application::FileFinding(forge::FileFinding {
                project: value.project,
                kind: kind_from(value.kind),
                severity: severity_from(value.severity),
                area: value.area,
                title: value.title,
                body: value.body,
            }))
        }
        w::Request::HandFinding(value) => {
            let value = value.into_parts();
            Some(root::Application::HandFinding(forge::HandFinding {
                project: value.project,
                number: value.number,
                area: value.area,
                why: value.why,
            hander: match value.hander {
                Some(value) => Some(filer(value, limits)?),
                None => None,
            },
            }))
        }
        w::Request::TriageFinding(value) => {
            let value = value.into_parts();
            Some(root::Application::TriageFinding(forge::TriageFinding {
                project: value.project,
                number: value.number,
                act: match value.act {
                    w::Triage::Route(value) => forge::Triage::Route(value.task()),
                    w::Triage::Backlog => forge::Triage::Backlog,
                    w::Triage::Drop(value) => forge::Triage::Drop(value.into_parts().reason),
                },
            }))
        }
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
        | w::Request::TryAccount(_) => None,
    }
}

pub(super) fn state(value: Option<&w::FindingState>) -> Option<forge::FindingState> {
    match value {
        Some(value) => Some(state_from(value.clone())),
        None => None,
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
    Some(match value {
        forge::Holder::Task(number) => w::Holder::Task(w::HolderTask::new(limits, w::HolderTaskParts { number }).ok()?),
        forge::Holder::Party(number) => {
            w::Holder::Party(w::HolderParty::new(limits, w::HolderPartyParts { number }).ok()?)
        }
        forge::Holder::Role(number) => w::Holder::Role(w::HolderRole::new(limits, w::HolderRoleParts { number }).ok()?),
    })
}

fn owner(value: forge::settings::AreaOwner, limits: &w::Limits) -> Option<w::AreaOwner> {
    use forge::settings::AreaOwner;
    Some(match value {
        AreaOwner::Task(number) => w::AreaOwner::Task(w::AreaTask::new(limits, w::AreaTaskParts { number }).ok()?),
        AreaOwner::Party(number) => w::AreaOwner::Party(w::AreaParty::new(limits, w::AreaPartyParts { number }).ok()?),
        AreaOwner::Role(number) => w::AreaOwner::Role(w::AreaRole::new(limits, w::AreaRoleParts { number }).ok()?),
        AreaOwner::Charter(number) => {
            w::AreaOwner::Charter(w::AreaCharter::new(limits, w::AreaCharterParts { number }).ok()?)
        }
    })
}

fn refused(value: forge::FindingRefusal, limits: &w::Limits) -> Option<w::Outcome> {
    let reason = match value {
        forge::FindingRefusal::NoArea(names) => {
            let mut areas = List::with_capacity(limits.finding_areas_areas);
            for name in names {
                areas.push(name).ok()?;
            }
            w::FindingRefusal::NoArea(w::FindingAreas::new(limits, w::FindingAreasParts { areas }).ok()?)
        }
        forge::FindingRefusal::Full(count) => {
            w::FindingRefusal::Full(w::FindingsFull::new(limits, w::FindingsFullParts { count }).ok()?)
        }
        forge::FindingRefusal::NotYours(value) => w::FindingRefusal::NotYours(
            w::FindingNotYours::new(limits, w::FindingNotYoursParts { holder: holder(value, limits)? }).ok()?,
        ),
        forge::FindingRefusal::NoFinding => w::FindingRefusal::NoFinding,
        forge::FindingRefusal::NoGoal => w::FindingRefusal::NoGoal,
        forge::FindingRefusal::Ended => w::FindingRefusal::Ended,
        forge::FindingRefusal::Words(field) => {
            w::FindingRefusal::Words(w::FindingWords::new(limits, w::FindingWordsParts { field }).ok()?)
        }
    };
    Some(w::Outcome::FindingRefused(w::FindingRefused::new(limits, w::FindingRefusedParts { reason }).ok()?))
}

pub(super) fn file_answer(value: forge::FileFindingAnswer, limits: &w::Limits) -> Option<w::Outcome> {
    match value {
        forge::FileFindingAnswer::Filed { number } => {
            Some(w::Outcome::Filed(w::Filed::new(limits, w::FiledParts { number }).ok()?))
        }
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
        forge::TriageFindingAnswer::Triaged => Some(w::Outcome::Triaged),
        forge::TriageFindingAnswer::Refused(value) => refused(value, limits),
    }
}

fn row(value: forge::FindingRow, limits: &w::Limits) -> Option<w::FindingRow> {
    w::FindingRow::new(
        limits,
        w::FindingRowParts {
            number: value.number,
            kind: kind_to(value.kind),
            severity: severity_to(value.severity),
            area: value.area,
            state: state_to(value.state),
            title: value.title,
            holder: holder(value.holder, limits)?,
            at: value.at.as_nanos(),
            why: value.why,
            hander: match value.hander {
                Some(value) => Some(filer(value, limits)?),
                None => None,
            },
        },
    )
    .ok()
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
            .push(
                w::AreaRow::new(
                    limits,
                    w::AreaRowParts {
                        name: value.name,
                        words: value.words,
                        owner: owner(value.owner, limits)?,
                        holder,
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    let triager = match value.triager {
        Some(value) => Some(owner(value, limits)?),
        None => None,
    };
    let record = w::Findings::new(
        limits,
        w::FindingsParts {
            project: value.project,
            rows,
            areas,
            triager,
            holder: holder(value.holder, limits)?,
            cut: value.cut,
        },
    )
    .ok()?;
    Some(wire::findings(&record))
}

pub(super) fn read_answer(value: Option<forge::FindingAnswer>, limits: &w::Limits) -> Option<Box<[u8]>> {
    let result = match value {
        None => w::FindingResult::NoFinding,
        Some(value) => {
            let filer = filer(value.filer, limits)?;
            w::FindingResult::Finding(
                w::FindingDetail::new(
                    limits,
                    w::FindingDetailParts {
                        row: row(value.row, limits)?,
                        body: value.body,
                        filer,
                        goal: value.goal,
                        reason: value.reason,
                    },
                )
                .ok()?,
            )
        }
    };
    let record = w::FindingAnswer::new(limits, w::FindingAnswerParts { result }).ok()?;
    Some(wire::finding_answer(&record))
}

fn filer(value: forge::Filer, limits: &w::Limits) -> Option<w::Filer> {
    Some(match value {
        forge::Filer::Task(number) => w::Filer::Task(w::HolderTask::new(limits, w::HolderTaskParts { number }).ok()?),
        forge::Filer::Party(number) => w::Filer::Party(w::HolderParty::new(limits, w::HolderPartyParts { number }).ok()?),
    })
}
