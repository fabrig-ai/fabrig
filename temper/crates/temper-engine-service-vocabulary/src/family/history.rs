//! Client history records (temper/docs/design/domain/landing-history.md, 2 and 5.1).
use temper_client_protocol_wire::v1::Record;
use super::w;
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use temper_engine_domain_forge::history as d;
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
fn encode_job_failure(value: d::JobFailure) -> w::HistoryJobFailure {
    match value {
        d::JobFailure::Unprepared => w::HistoryJobFailure::Unprepared,
        d::JobFailure::Run => w::HistoryJobFailure::Run,
        d::JobFailure::Result => w::HistoryJobFailure::Result,
        d::JobFailure::Unstarted => w::HistoryJobFailure::Unstarted,
        d::JobFailure::Undelivered => w::HistoryJobFailure::Undelivered,
        d::JobFailure::Unknown => w::HistoryJobFailure::Unknown,
    }
}
fn encode_job_outcome(value: d::JobOutcome) -> w::HistoryJobOutcome {
    match value {
        d::JobOutcome::Done => w::HistoryJobOutcome::Done,
        d::JobOutcome::Conflict => w::HistoryJobOutcome::Conflict,
        d::JobOutcome::Failed { class } => w::HistoryJobOutcome::Failed(
            w::HistoryJobFailed { class: encode_job_failure(class) },
        ),
        d::JobOutcome::Timeout => w::HistoryJobOutcome::Timeout,
        d::JobOutcome::Cancelled => w::HistoryJobOutcome::Cancelled,
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
    w::HistoryInLanding { task: value.task, round: value.round, unit: value.unit }
}
fn encode_member(value: &d::Member) -> w::HistoryMember {
    w::HistoryMember { change: value.change, unit: value.unit, head: value.head }
}
fn encode_proof(value: &d::Proof) -> w::HistoryProof {
    w::HistoryProof { gate: value.gate, run: value.run }
}
fn encode_event(value: &d::Event) -> w::HistoryEvent {
    w::HistoryEvent {
            project: value.project,
            number: value.number,
            at: value.at.as_nanos(),
            source_at: match value.source_at {
                Some(value) => Some(value.as_nanos()),
                None => None,
            },
            change: value.change,
            repository: w::HistoryRepository { forge: value.repository.forge, repository: value.repository.repository },
            landing: match value.landing {
                Some(value) => Some(encode_in_landing(&value)),
                None => None,
            },
            head: value.head,
            base: value.base,
            operation: value.operation,
            attempt: value.attempt,
            what: encode_what(&value.what),
        }
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
            w::HistoryLandingDeparture { cause: encode_leave(cause) },
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
            w::HistoryBy::Party(w::HistoryNumber { number })
        }
        d::By::Task(number) => {
            w::HistoryBy::Task(w::HistoryNumber { number })
        }
    }
}
#[expect(clippy::too_many_lines, reason = "one exhaustive translation of history kinds")]
fn encode_what(value: &d::What) -> w::HistoryWhat {
    match value {
        d::What::Ready => w::HistoryWhat::Ready,
        d::What::Unready { why } => w::HistoryWhat::Unready(
            w::HistoryUnreadyStep { why: encode_unready(*why) },
        ),
        d::What::Joined { line, place, priority } => w::HistoryWhat::Joined(
            w::HistoryJoined {
                    line: encode_line(*line),
                    place: *place,
                    priority: match *priority {
                        Some(value) => Some(u32::from_be_bytes(value.to_be_bytes())),
                        None => None,
                    },
                },
        ),
        d::What::Departed { line, cause } => w::HistoryWhat::Departed(
            w::HistoryDeparted { line: encode_line(*line), cause: encode_departure(*cause) },
        ),
        d::What::Selected => w::HistoryWhat::Selected,
        d::What::Update { style, forge_base } => w::HistoryWhat::Update(
            w::HistoryUpdate { style: encode_style(*style), forge_base: *forge_base },
        ),
        d::What::Updated { outcome, forge_base } => w::HistoryWhat::Updated(
            w::HistoryUpdated { outcome: encode_update_outcome(*outcome), forge_base: *forge_base },
        ),
        d::What::Job { job, task } => w::HistoryWhat::Job(
            w::HistoryJobStarted { job: encode_job(*job), task: *task },
        ),
        d::What::JobEnded { job, outcome } => w::HistoryWhat::JobEnded(
            w::HistoryJobEnded { job: encode_job(*job), outcome: encode_job_outcome(*outcome) },
        ),
        d::What::Conflict { with, files } => w::HistoryWhat::Conflict(
            w::HistoryConflict { with: encode_with(*with), files: *files },
        ),
        d::What::Asked { ask, task, gate } => w::HistoryWhat::Asked(
            w::HistoryAsked { ask: encode_ask(*ask), task: *task, gate: *gate },
        ),
        d::What::Answered { ask, outcome } => w::HistoryWhat::Answered(
            w::HistoryAnswered { ask: encode_ask(*ask), outcome: encode_answer(*outcome) },
        ),
        d::What::Pushed => w::HistoryWhat::Pushed,
        d::What::Delivered { checked } => w::HistoryWhat::Delivered(
            w::HistoryDelivered { checked: *checked },
        ),
        d::What::Paused { why } => w::HistoryWhat::Paused(
            w::HistoryPaused { why: encode_pause(*why) },
        ),
        d::What::Resumed => w::HistoryWhat::Resumed,
        d::What::Formed { members, place } => w::HistoryWhat::Formed(
            w::HistoryFormed { members: encode_members(members), place: *place },
        ),
        d::What::RoundEnded { cause } => w::HistoryWhat::RoundEnded(
            w::HistoryRoundEnded { cause: encode_round_cause(*cause) },
        ),
        d::What::Gate { gate, key } => w::HistoryWhat::Gate(
            w::HistoryGate { gate: *gate, key: *key },
        ),
        d::What::GateEnded { gate, key, verdict } => w::HistoryWhat::GateEnded(
            w::HistoryGateEnded { gate: *gate, key: *key, verdict: encode_gate_verdict(*verdict) },
        ),
        d::What::Proved { proofs } => w::HistoryWhat::Proved(
            w::HistoryProved { proofs: encode_proofs(proofs) },
        ),
        d::What::Search => w::HistoryWhat::Search,
        d::What::Searched { green, red, blamed } => w::HistoryWhat::Searched(
            w::HistorySearched { green: *green, red: *red, blamed: *blamed },
        ),
        d::What::Probe => w::HistoryWhat::Probe,
        d::What::ProbeEnded { green } => w::HistoryWhat::ProbeEnded(
            w::HistoryProbeEnded { green: *green },
        ),
        d::What::Turn { members } => w::HistoryWhat::Turn(
            w::HistoryTurn { members: *members },
        ),
        d::What::Landed => w::HistoryWhat::Landed,
        d::What::Recorded { state } => w::HistoryWhat::Recorded(
            w::HistoryRecorded { state: encode_recorded_state(*state) },
        ),
        d::What::Ended { ending } => w::HistoryWhat::Ended(
            w::HistoryEnded { ending: encode_ending(*ending) },
        ),
        d::What::LandingEnded { outcome } => w::HistoryWhat::LandingEnded(
            w::HistoryLandingEnded { outcome: encode_landing_outcome(*outcome) },
        ),
        d::What::Held { on, reason, by } => w::HistoryWhat::Held(
            w::HistoryHold {
                    on: encode_on(*on),
                    reason: reason.clone(),
                    by: match *by {
                        Some(value) => Some(encode_by(value)),
                        None => None,
                    },
                },
        ),
        d::What::Released { on, reason, by } => w::HistoryWhat::Released(
            w::HistoryHold {
                    on: encode_on(*on),
                    reason: reason.clone(),
                    by: match *by {
                        Some(value) => Some(encode_by(value)),
                        None => None,
                    },
                },
        ),
        d::What::After { verdict } => w::HistoryWhat::After(
            w::HistoryAfter { verdict: encode_after_verdict(*verdict) },
        ),
    }
}
fn encode_leave(why: temper_engine_domain_forge::LandingLeave) -> w::HistoryLeave {
    match why {
        temper_engine_domain_forge::LandingLeave::Conflict => w::HistoryLeave::Conflict,
        temper_engine_domain_forge::LandingLeave::EarlierConflict => w::HistoryLeave::EarlierConflict,
        temper_engine_domain_forge::LandingLeave::Unit { change } => w::HistoryLeave::Unit(
            w::HistoryLeftChange { change },
        ),
        temper_engine_domain_forge::LandingLeave::Dependency { change } => w::HistoryLeave::Dependency(
            w::HistoryLeftChange { change },
        ),
        temper_engine_domain_forge::LandingLeave::Blamed { gate, run, red, green } => w::HistoryLeave::Blamed(
            w::HistoryBlamed { gate, run, red, green },
        ),
        temper_engine_domain_forge::LandingLeave::HeadMoved { head } => w::HistoryLeave::HeadMoved(
            w::HistoryCommit { commit: head },
        ),
        temper_engine_domain_forge::LandingLeave::Cancelled => w::HistoryLeave::Cancelled,
        temper_engine_domain_forge::LandingLeave::Held => w::HistoryLeave::Held,
        temper_engine_domain_forge::LandingLeave::Refused { gate } => w::HistoryLeave::Refused(
            w::HistoryGateRefused { gate },
        ),
        temper_engine_domain_forge::LandingLeave::Failed { gate, run } => w::HistoryLeave::Failed(
            w::HistoryGateFailed { gate, run },
        ),
    }
}

pub(super) fn query(value: w::HistoryRead) -> Option<d::Query> {
    let w::HistoryRead { project, of, since, until, after } = value;
    Some(d::Query {
        project,
        of: match of {
            w::HistoryOf::Project => d::Of::Project,
            w::HistoryOf::Change(task) => d::Of::Change(task.task),
            w::HistoryOf::Landing(task) => d::Of::Landing(task.task),
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
            w::HistoryReply::Gone(w::HistoryGone { from, first })
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
                w::HistoryPage {
                        events: rows,
                        after,
                        upper,
                        whole_since: match whole_since {
                            Some(at) => Some(at.as_nanos()),
                            None => None,
                        },
                    },
            )
        }
    };
    (w::HistoryAnswer { result }).encode(limits).ok()
}
