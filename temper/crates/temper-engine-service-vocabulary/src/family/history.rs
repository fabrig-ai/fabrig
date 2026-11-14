//! Client history records (temper/docs/design/domain/landing-history.md, 2 and 5.1).
use super::{w, wire};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use temper_engine_domain_forge::history as d;
const FITS: &str = "bounded history event fits the client wire";
fn encode_unready(value: d::Unready) -> w::HistoryUnready {
    match value {
        d::Unready::Held => w::HistoryUnready::Held,
        d::Unready::HeadMoved => w::HistoryUnready::HeadMoved,
        d::Unready::Refused => w::HistoryUnready::Refused,
        d::Unready::Dependency => w::HistoryUnready::Dependency,
        d::Unready::Approval => w::HistoryUnready::Approval,
        d::Unready::Conflict => w::HistoryUnready::Conflict,
        d::Unready::Closed => w::HistoryUnready::Closed,
    }
}
fn encode_line(value: d::Line) -> w::HistoryLine {
    match value {
        d::Line::Queue => w::HistoryLine::Queue,
        d::Line::Landing => w::HistoryLine::Landing,
        d::Line::ForgeQueue => w::HistoryLine::ForgeQueue,
    }
}
fn encode_update_outcome(value: d::UpdateOutcome) -> w::HistoryUpdateOutcome {
    match value {
        d::UpdateOutcome::Updated => w::HistoryUpdateOutcome::Updated,
        d::UpdateOutcome::Conflict => w::HistoryUpdateOutcome::Conflict,
        d::UpdateOutcome::Failed => w::HistoryUpdateOutcome::Failed,
    }
}
fn encode_job(value: d::Job) -> w::HistoryJob {
    match value {
        d::Job::Prepare => w::HistoryJob::Prepare,
        d::Job::Merge => w::HistoryJob::Merge,
    }
}
fn encode_job_outcome(value: d::JobOutcome) -> w::HistoryJobOutcome {
    match value {
        d::JobOutcome::Done => w::HistoryJobOutcome::Done,
        d::JobOutcome::Conflict => w::HistoryJobOutcome::Conflict,
        d::JobOutcome::Failed => w::HistoryJobOutcome::Failed,
        d::JobOutcome::Lost => w::HistoryJobOutcome::Lost,
    }
}
fn encode_with(value: d::With) -> w::HistoryWith {
    match value {
        d::With::Branch => w::HistoryWith::Branch,
        d::With::Earlier => w::HistoryWith::Earlier,
    }
}
fn encode_ask(value: d::Ask) -> w::HistoryAsk {
    match value {
        d::Ask::Resolution => w::HistoryAsk::Resolution,
        d::Ask::Repair => w::HistoryAsk::Repair,
        d::Ask::Judgement => w::HistoryAsk::Judgement,
    }
}
fn encode_answer(value: d::Answer) -> w::HistoryDelegateOutcome {
    match value {
        d::Answer::Pushed => w::HistoryDelegateOutcome::Pushed,
        d::Answer::Passed => w::HistoryDelegateOutcome::Passed,
        d::Answer::Refused => w::HistoryDelegateOutcome::Refused,
        d::Answer::Nothing => w::HistoryDelegateOutcome::Nothing,
        d::Answer::Failed => w::HistoryDelegateOutcome::Failed,
        d::Answer::Cancelled => w::HistoryDelegateOutcome::Cancelled,
    }
}
fn encode_pause(value: d::Pause) -> w::HistoryPause {
    match value {
        d::Pause::BranchRed => w::HistoryPause::BranchRed,
        d::Pause::TierRed => w::HistoryPause::TierRed,
    }
}
fn encode_round_cause(value: d::RoundCause) -> w::HistoryRoundCause {
    match value {
        d::RoundCause::Turned => w::HistoryRoundCause::Turned,
        d::RoundCause::Left => w::HistoryRoundCause::Left,
        d::RoundCause::Stale => w::HistoryRoundCause::Stale,
        d::RoundCause::Cancelled => w::HistoryRoundCause::Cancelled,
        d::RoundCause::Cut => w::HistoryRoundCause::Cut,
    }
}
fn encode_gate_verdict(value: d::GateVerdict) -> w::HistoryGateVerdict {
    match value {
        d::GateVerdict::Passed => w::HistoryGateVerdict::Passed,
        d::GateVerdict::Failed => w::HistoryGateVerdict::Failed,
        d::GateVerdict::Fault => w::HistoryGateVerdict::Fault,
        d::GateVerdict::Timeout => w::HistoryGateVerdict::Timeout,
        d::GateVerdict::Superseded => w::HistoryGateVerdict::Superseded,
    }
}
fn encode_recorded_state(value: d::RecordedState) -> w::HistoryRecordedState {
    match value {
        d::RecordedState::Merged => w::HistoryRecordedState::Merged,
        d::RecordedState::Closed => w::HistoryRecordedState::Closed,
    }
}
fn encode_ending(value: d::Ending) -> w::HistoryEnding {
    match value {
        d::Ending::NoChange => w::HistoryEnding::NoChange,
        d::Ending::Cancelled => w::HistoryEnding::Cancelled,
        d::Ending::Failed => w::HistoryEnding::Failed,
    }
}
fn encode_landing_outcome(value: d::LandingOutcome) -> w::HistoryLandingOutcome {
    match value {
        d::LandingOutcome::Done => w::HistoryLandingOutcome::Done,
        d::LandingOutcome::Cancelled => w::HistoryLandingOutcome::Cancelled,
        d::LandingOutcome::Held => w::HistoryLandingOutcome::Held,
    }
}
fn encode_on(value: d::On) -> w::HistoryOn {
    match value {
        d::On::Change => w::HistoryOn::Change,
        d::On::Landing => w::HistoryOn::Landing,
    }
}
fn encode_after_verdict(value: d::AfterVerdict) -> w::HistoryAfterVerdict {
    match value {
        d::AfterVerdict::Verified => w::HistoryAfterVerdict::Verified,
        d::AfterVerdict::Red => w::HistoryAfterVerdict::Red,
        d::AfterVerdict::Cleared => w::HistoryAfterVerdict::Cleared,
        d::AfterVerdict::Uncleared => w::HistoryAfterVerdict::Uncleared,
    }
}
fn encode_in_landing(value: &d::InLanding) -> w::HistoryInLanding {
    w::HistoryInLanding::new(
        &w::CEILINGS,
        w::HistoryInLandingParts { task: value.task, round: value.round, unit: value.unit },
    )
    .expect(FITS)
}
fn encode_member(value: &d::Member) -> w::HistoryMember {
    w::HistoryMember::new(
        &w::CEILINGS,
        w::HistoryMemberParts { change: value.change, unit: value.unit, head: value.head },
    )
    .expect(FITS)
}
fn encode_proof(value: &d::Proof) -> w::HistoryProof {
    w::HistoryProof::new(&w::CEILINGS, w::HistoryProofParts { gate: value.gate, run: value.run }).expect(FITS)
}
fn encode_event(value: &d::Event) -> w::HistoryEvent {
    w::HistoryEvent::new(
        &w::CEILINGS,
        w::HistoryEventParts {
            project: value.project,
            number: value.number,
            at: value.at.as_nanos(),
            source_at: match value.source_at {
                Some(value) => Some(value.as_nanos()),
                None => None,
            },
            change: value.change,
            repository: w::HistoryRepository::new(
                &w::CEILINGS,
                w::HistoryRepositoryParts { forge: value.repository.forge, repository: value.repository.repository },
            )
            .expect(FITS),
            landing: match value.landing {
                Some(value) => Some(encode_in_landing(&value)),
                None => None,
            },
            head: value.head,
            base: value.base,
            operation: value.operation,
            attempt: value.attempt,
            what: encode_what(&value.what),
        },
    )
    .expect(FITS)
}
fn encode_members(values: &[d::Member]) -> List<w::HistoryMember> {
    let mut out = List::with_capacity(u32::try_from(values.len()).expect("bounded history list"));
    for value in values {
        out.push(encode_member(value)).expect("bounded history list");
    }
    out
}
fn encode_proofs(values: &[d::Proof]) -> List<w::HistoryProof> {
    let mut out = List::with_capacity(u32::try_from(values.len()).expect("bounded history list"));
    for value in values {
        out.push(encode_proof(value)).expect("bounded history list");
    }
    out
}
fn encode_style(value: skein_forge::api::UpdateStyle) -> w::HistoryUpdateStyle {
    match value {
        skein_forge::api::UpdateStyle::Merge => w::HistoryUpdateStyle::Merge,
        skein_forge::api::UpdateStyle::Rebase => w::HistoryUpdateStyle::Rebase,
    }
}
fn encode_departure(value: d::Departure) -> w::HistoryDeparture {
    match value {
        d::Departure::Landing(cause) => w::HistoryDeparture::Landing(
            w::HistoryLandingDeparture::new(
                &w::CEILINGS,
                w::HistoryLandingDepartureParts { cause: encode_leave(cause) },
            )
            .expect(FITS),
        ),
        d::Departure::Unready => w::HistoryDeparture::Unready,
        d::Departure::Conflict => w::HistoryDeparture::Conflict,
        d::Departure::Repair => w::HistoryDeparture::Repair,
        d::Departure::Ejected => w::HistoryDeparture::Ejected,
    }
}
fn encode_by(value: d::By) -> w::HistoryBy {
    match value {
        d::By::Party(number) => {
            w::HistoryBy::Party(w::HistoryNumber::new(&w::CEILINGS, w::HistoryNumberParts { number }).expect(FITS))
        }
        d::By::Task(number) => {
            w::HistoryBy::Task(w::HistoryNumber::new(&w::CEILINGS, w::HistoryNumberParts { number }).expect(FITS))
        }
    }
}
#[expect(clippy::too_many_lines, reason = "one exhaustive translation of history kinds")]
fn encode_what(value: &d::What) -> w::HistoryWhat {
    match value {
        d::What::Ready => w::HistoryWhat::Ready,
        d::What::Unready { why } => w::HistoryWhat::Unready(
            w::HistoryUnreadyStep::new(&w::CEILINGS, w::HistoryUnreadyStepParts { why: encode_unready(*why) })
                .expect(FITS),
        ),
        d::What::Joined { line, place, priority } => w::HistoryWhat::Joined(
            w::HistoryJoined::new(
                &w::CEILINGS,
                w::HistoryJoinedParts {
                    line: encode_line(*line),
                    place: *place,
                    priority: match *priority {
                        Some(value) => Some(u32::from_be_bytes(value.to_be_bytes())),
                        None => None,
                    },
                },
            )
            .expect(FITS),
        ),
        d::What::Departed { line, cause } => w::HistoryWhat::Departed(
            w::HistoryDeparted::new(
                &w::CEILINGS,
                w::HistoryDepartedParts { line: encode_line(*line), cause: encode_departure(*cause) },
            )
            .expect(FITS),
        ),
        d::What::Selected => w::HistoryWhat::Selected,
        d::What::Update { style } => w::HistoryWhat::Update(
            w::HistoryUpdate::new(&w::CEILINGS, w::HistoryUpdateParts { style: encode_style(*style) }).expect(FITS),
        ),
        d::What::Updated { outcome } => w::HistoryWhat::Updated(
            w::HistoryUpdated::new(&w::CEILINGS, w::HistoryUpdatedParts { outcome: encode_update_outcome(*outcome) })
                .expect(FITS),
        ),
        d::What::Job { job, task } => w::HistoryWhat::Job(
            w::HistoryJobStarted::new(&w::CEILINGS, w::HistoryJobStartedParts { job: encode_job(*job), task: *task })
                .expect(FITS),
        ),
        d::What::JobEnded { job, outcome } => w::HistoryWhat::JobEnded(
            w::HistoryJobEnded::new(
                &w::CEILINGS,
                w::HistoryJobEndedParts { job: encode_job(*job), outcome: encode_job_outcome(*outcome) },
            )
            .expect(FITS),
        ),
        d::What::Conflict { with, files } => w::HistoryWhat::Conflict(
            w::HistoryConflict::new(&w::CEILINGS, w::HistoryConflictParts { with: encode_with(*with), files: *files })
                .expect(FITS),
        ),
        d::What::Asked { ask, task, gate } => w::HistoryWhat::Asked(
            w::HistoryAsked::new(
                &w::CEILINGS,
                w::HistoryAskedParts { ask: encode_ask(*ask), task: *task, gate: *gate },
            )
            .expect(FITS),
        ),
        d::What::Answered { ask, outcome } => w::HistoryWhat::Answered(
            w::HistoryAnswered::new(
                &w::CEILINGS,
                w::HistoryAnsweredParts { ask: encode_ask(*ask), outcome: encode_answer(*outcome) },
            )
            .expect(FITS),
        ),
        d::What::Pushed => w::HistoryWhat::Pushed,
        d::What::Delivered { checked } => w::HistoryWhat::Delivered(
            w::HistoryDelivered::new(&w::CEILINGS, w::HistoryDeliveredParts { checked: *checked }).expect(FITS),
        ),
        d::What::Paused { why } => w::HistoryWhat::Paused(
            w::HistoryPaused::new(&w::CEILINGS, w::HistoryPausedParts { why: encode_pause(*why) }).expect(FITS),
        ),
        d::What::Resumed => w::HistoryWhat::Resumed,
        d::What::Formed { members, place } => w::HistoryWhat::Formed(
            w::HistoryFormed::new(
                &w::CEILINGS,
                w::HistoryFormedParts { members: encode_members(members), place: *place },
            )
            .expect(FITS),
        ),
        d::What::RoundEnded { cause } => w::HistoryWhat::RoundEnded(
            w::HistoryRoundEnded::new(&w::CEILINGS, w::HistoryRoundEndedParts { cause: encode_round_cause(*cause) })
                .expect(FITS),
        ),
        d::What::Gate { gate, key } => w::HistoryWhat::Gate(
            w::HistoryGate::new(&w::CEILINGS, w::HistoryGateParts { gate: *gate, key: *key }).expect(FITS),
        ),
        d::What::GateEnded { gate, key, verdict } => w::HistoryWhat::GateEnded(
            w::HistoryGateEnded::new(
                &w::CEILINGS,
                w::HistoryGateEndedParts { gate: *gate, key: *key, verdict: encode_gate_verdict(*verdict) },
            )
            .expect(FITS),
        ),
        d::What::Proved { proofs } => w::HistoryWhat::Proved(
            w::HistoryProved::new(&w::CEILINGS, w::HistoryProvedParts { proofs: encode_proofs(proofs) }).expect(FITS),
        ),
        d::What::Search => w::HistoryWhat::Search,
        d::What::Searched { green, red, blamed } => w::HistoryWhat::Searched(
            w::HistorySearched::new(
                &w::CEILINGS,
                w::HistorySearchedParts { green: *green, red: *red, blamed: *blamed },
            )
            .expect(FITS),
        ),
        d::What::Probe => w::HistoryWhat::Probe,
        d::What::ProbeEnded { green } => w::HistoryWhat::ProbeEnded(
            w::HistoryProbeEnded::new(&w::CEILINGS, w::HistoryProbeEndedParts { green: *green }).expect(FITS),
        ),
        d::What::Turn { members } => w::HistoryWhat::Turn(
            w::HistoryTurn::new(&w::CEILINGS, w::HistoryTurnParts { members: *members }).expect(FITS),
        ),
        d::What::Landed => w::HistoryWhat::Landed,
        d::What::Recorded { state } => w::HistoryWhat::Recorded(
            w::HistoryRecorded::new(&w::CEILINGS, w::HistoryRecordedParts { state: encode_recorded_state(*state) })
                .expect(FITS),
        ),
        d::What::Ended { ending } => w::HistoryWhat::Ended(
            w::HistoryEnded::new(&w::CEILINGS, w::HistoryEndedParts { ending: encode_ending(*ending) }).expect(FITS),
        ),
        d::What::LandingEnded { outcome } => w::HistoryWhat::LandingEnded(
            w::HistoryLandingEnded::new(
                &w::CEILINGS,
                w::HistoryLandingEndedParts { outcome: encode_landing_outcome(*outcome) },
            )
            .expect(FITS),
        ),
        d::What::Held { on, reason, by } => w::HistoryWhat::Held(
            w::HistoryHold::new(
                &w::CEILINGS,
                w::HistoryHoldParts {
                    on: encode_on(*on),
                    reason: reason.clone(),
                    by: match *by {
                        Some(value) => Some(encode_by(value)),
                        None => None,
                    },
                },
            )
            .expect(FITS),
        ),
        d::What::Released { on, reason, by } => w::HistoryWhat::Released(
            w::HistoryHold::new(
                &w::CEILINGS,
                w::HistoryHoldParts {
                    on: encode_on(*on),
                    reason: reason.clone(),
                    by: match *by {
                        Some(value) => Some(encode_by(value)),
                        None => None,
                    },
                },
            )
            .expect(FITS),
        ),
        d::What::After { verdict } => w::HistoryWhat::After(
            w::HistoryAfter::new(&w::CEILINGS, w::HistoryAfterParts { verdict: encode_after_verdict(*verdict) })
                .expect(FITS),
        ),
    }
}
fn encode_leave(why: temper_engine_domain_forge::LandingLeave) -> w::HistoryLeave {
    match why {
        temper_engine_domain_forge::LandingLeave::Conflict => w::HistoryLeave::Conflict,
        temper_engine_domain_forge::LandingLeave::EarlierConflict => w::HistoryLeave::EarlierConflict,
        temper_engine_domain_forge::LandingLeave::Unit { change } => w::HistoryLeave::Unit(
            w::HistoryLeftChange::new(&w::CEILINGS, w::HistoryLeftChangeParts { change }).expect(FITS),
        ),
        temper_engine_domain_forge::LandingLeave::Dependency { change } => w::HistoryLeave::Dependency(
            w::HistoryLeftChange::new(&w::CEILINGS, w::HistoryLeftChangeParts { change }).expect(FITS),
        ),
        temper_engine_domain_forge::LandingLeave::Blamed { gate, run, red, green } => w::HistoryLeave::Blamed(
            w::HistoryBlamed::new(&w::CEILINGS, w::HistoryBlamedParts { gate, run, red, green }).expect(FITS),
        ),
        temper_engine_domain_forge::LandingLeave::HeadMoved { head } => w::HistoryLeave::HeadMoved(
            w::HistoryCommit::new(&w::CEILINGS, w::HistoryCommitParts { commit: head }).expect(FITS),
        ),
        temper_engine_domain_forge::LandingLeave::Cancelled => w::HistoryLeave::Cancelled,
        temper_engine_domain_forge::LandingLeave::Held => w::HistoryLeave::Held,
        temper_engine_domain_forge::LandingLeave::Refused { gate } => w::HistoryLeave::Refused(
            w::HistoryGateRefused::new(&w::CEILINGS, w::HistoryGateRefusedParts { gate }).expect(FITS),
        ),
        temper_engine_domain_forge::LandingLeave::Failed { gate, run } => w::HistoryLeave::Failed(
            w::HistoryGateFailed::new(&w::CEILINGS, w::HistoryGateFailedParts { gate, run }).expect(FITS),
        ),
    }
}

pub(super) fn query(value: w::HistoryRead) -> Option<d::Query> {
    let w::HistoryReadParts { project, of, since, until, after } = value.into_parts();
    Some(d::Query {
        project,
        of: match of {
            w::HistoryOf::Project => d::Of::Project,
            w::HistoryOf::Change(task) => d::Of::Change(task.task()),
            w::HistoryOf::Landing(task) => d::Of::Landing(task.task()),
        },
        since: match since {
            Some(n) => Some(Wall::from_nanos(n)),
            None => None,
        },
        until: match until {
            Some(n) => Some(Wall::from_nanos(n)),
            None => None,
        },
        after: match after {
            Some(bytes) => Some(d::Cursor::decode(&bytes)?),
            None => None,
        },
    })
}
pub(super) fn answer(value: d::Reply, limits: &w::Limits) -> Option<Box<[u8]>> {
    let result = match value {
        d::Reply::Busy => w::HistoryReply::Busy,
        d::Reply::Gone { from, first } => {
            w::HistoryReply::Gone(w::HistoryGone::new(limits, w::HistoryGoneParts { from, first }).ok()?)
        }
        d::Reply::Page { events, after, upper, whole_since } => {
            let mut rows = List::with_capacity(limits.history_page_events);
            for event in &events {
                rows.push(encode_event(event)).ok()?;
            }
            let after = match after {
                Some(cursor) => Some(cursor.encode().as_ref().try_into().expect("17-byte cursor")),
                None => None,
            };
            w::HistoryReply::Page(
                w::HistoryPage::new(
                    limits,
                    w::HistoryPageParts {
                        events: rows,
                        after,
                        upper,
                        whole_since: match whole_since {
                            Some(at) => Some(at.as_nanos()),
                            None => None,
                        },
                    },
                )
                .ok()?,
            )
        }
    };
    Some(wire::history_answer(&w::HistoryAnswer::new(limits, w::HistoryAnswerParts { result }).ok()?))
}
