//! Total landing history translations (temper/docs/design/domain/landing-history.md, 2 and 4).
use super::{FITS, names, w};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use temper_engine_domain_forge_vocabulary::history as d;
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
fn decode_unready(value: w::HistoryUnready) -> d::Unready {
    match value {
        w::HistoryUnready::Held => d::Unready::Held,
        w::HistoryUnready::HeadMoved => d::Unready::HeadMoved,
        w::HistoryUnready::Refused => d::Unready::Refused,
        w::HistoryUnready::Dependency => d::Unready::Dependency,
        w::HistoryUnready::Approval => d::Unready::Approval,
        w::HistoryUnready::Conflict => d::Unready::Conflict,
        w::HistoryUnready::Closed => d::Unready::Closed,
    }
}
fn encode_line(value: d::Line) -> w::HistoryLine {
    match value {
        d::Line::Queue => w::HistoryLine::Queue,
        d::Line::Landing => w::HistoryLine::Landing,
        d::Line::ForgeQueue => w::HistoryLine::ForgeQueue,
    }
}
fn decode_line(value: w::HistoryLine) -> d::Line {
    match value {
        w::HistoryLine::Queue => d::Line::Queue,
        w::HistoryLine::Landing => d::Line::Landing,
        w::HistoryLine::ForgeQueue => d::Line::ForgeQueue,
    }
}
fn encode_update_outcome(value: d::UpdateOutcome) -> w::HistoryUpdateOutcome {
    match value {
        d::UpdateOutcome::Updated => w::HistoryUpdateOutcome::Updated,
        d::UpdateOutcome::Conflict => w::HistoryUpdateOutcome::Conflict,
        d::UpdateOutcome::Failed => w::HistoryUpdateOutcome::Failed,
    }
}
fn decode_update_outcome(value: w::HistoryUpdateOutcome) -> d::UpdateOutcome {
    match value {
        w::HistoryUpdateOutcome::Updated => d::UpdateOutcome::Updated,
        w::HistoryUpdateOutcome::Conflict => d::UpdateOutcome::Conflict,
        w::HistoryUpdateOutcome::Failed => d::UpdateOutcome::Failed,
    }
}
fn encode_job(value: d::Job) -> w::HistoryJob {
    match value {
        d::Job::Prepare => w::HistoryJob::Prepare,
        d::Job::Merge => w::HistoryJob::Merge,
    }
}
fn decode_job(value: w::HistoryJob) -> d::Job {
    match value {
        w::HistoryJob::Prepare => d::Job::Prepare,
        w::HistoryJob::Merge => d::Job::Merge,
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
fn decode_job_outcome(value: w::HistoryJobOutcome) -> d::JobOutcome {
    match value {
        w::HistoryJobOutcome::Done => d::JobOutcome::Done,
        w::HistoryJobOutcome::Conflict => d::JobOutcome::Conflict,
        w::HistoryJobOutcome::Failed => d::JobOutcome::Failed,
        w::HistoryJobOutcome::Lost => d::JobOutcome::Lost,
    }
}
fn encode_with(value: d::With) -> w::HistoryWith {
    match value {
        d::With::Branch => w::HistoryWith::Branch,
        d::With::Earlier => w::HistoryWith::Earlier,
    }
}
fn decode_with(value: w::HistoryWith) -> d::With {
    match value {
        w::HistoryWith::Branch => d::With::Branch,
        w::HistoryWith::Earlier => d::With::Earlier,
    }
}
fn encode_ask(value: d::Ask) -> w::HistoryAsk {
    match value {
        d::Ask::Resolution => w::HistoryAsk::Resolution,
        d::Ask::Repair => w::HistoryAsk::Repair,
        d::Ask::Judgement => w::HistoryAsk::Judgement,
    }
}
fn decode_ask(value: w::HistoryAsk) -> d::Ask {
    match value {
        w::HistoryAsk::Resolution => d::Ask::Resolution,
        w::HistoryAsk::Repair => d::Ask::Repair,
        w::HistoryAsk::Judgement => d::Ask::Judgement,
    }
}
fn encode_answer(value: d::Answer) -> w::HistoryAnswer {
    match value {
        d::Answer::Pushed => w::HistoryAnswer::Pushed,
        d::Answer::Passed => w::HistoryAnswer::Passed,
        d::Answer::Refused => w::HistoryAnswer::Refused,
        d::Answer::Nothing => w::HistoryAnswer::Nothing,
        d::Answer::Failed => w::HistoryAnswer::Failed,
        d::Answer::Cancelled => w::HistoryAnswer::Cancelled,
    }
}
fn decode_answer(value: w::HistoryAnswer) -> d::Answer {
    match value {
        w::HistoryAnswer::Pushed => d::Answer::Pushed,
        w::HistoryAnswer::Passed => d::Answer::Passed,
        w::HistoryAnswer::Refused => d::Answer::Refused,
        w::HistoryAnswer::Nothing => d::Answer::Nothing,
        w::HistoryAnswer::Failed => d::Answer::Failed,
        w::HistoryAnswer::Cancelled => d::Answer::Cancelled,
    }
}
fn encode_pause(value: d::Pause) -> w::HistoryPause {
    match value {
        d::Pause::BranchRed => w::HistoryPause::BranchRed,
        d::Pause::TierRed => w::HistoryPause::TierRed,
    }
}
fn decode_pause(value: w::HistoryPause) -> d::Pause {
    match value {
        w::HistoryPause::BranchRed => d::Pause::BranchRed,
        w::HistoryPause::TierRed => d::Pause::TierRed,
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
fn decode_round_cause(value: w::HistoryRoundCause) -> d::RoundCause {
    match value {
        w::HistoryRoundCause::Turned => d::RoundCause::Turned,
        w::HistoryRoundCause::Left => d::RoundCause::Left,
        w::HistoryRoundCause::Stale => d::RoundCause::Stale,
        w::HistoryRoundCause::Cancelled => d::RoundCause::Cancelled,
        w::HistoryRoundCause::Cut => d::RoundCause::Cut,
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
fn decode_gate_verdict(value: w::HistoryGateVerdict) -> d::GateVerdict {
    match value {
        w::HistoryGateVerdict::Passed => d::GateVerdict::Passed,
        w::HistoryGateVerdict::Failed => d::GateVerdict::Failed,
        w::HistoryGateVerdict::Fault => d::GateVerdict::Fault,
        w::HistoryGateVerdict::Timeout => d::GateVerdict::Timeout,
        w::HistoryGateVerdict::Superseded => d::GateVerdict::Superseded,
    }
}
fn encode_recorded_state(value: d::RecordedState) -> w::HistoryRecordedState {
    match value {
        d::RecordedState::Merged => w::HistoryRecordedState::Merged,
        d::RecordedState::Closed => w::HistoryRecordedState::Closed,
    }
}
fn decode_recorded_state(value: w::HistoryRecordedState) -> d::RecordedState {
    match value {
        w::HistoryRecordedState::Merged => d::RecordedState::Merged,
        w::HistoryRecordedState::Closed => d::RecordedState::Closed,
    }
}
fn encode_ending(value: d::Ending) -> w::HistoryEnding {
    match value {
        d::Ending::NoChange => w::HistoryEnding::NoChange,
        d::Ending::Cancelled => w::HistoryEnding::Cancelled,
        d::Ending::Failed => w::HistoryEnding::Failed,
    }
}
fn decode_ending(value: w::HistoryEnding) -> d::Ending {
    match value {
        w::HistoryEnding::NoChange => d::Ending::NoChange,
        w::HistoryEnding::Cancelled => d::Ending::Cancelled,
        w::HistoryEnding::Failed => d::Ending::Failed,
    }
}
fn encode_landing_outcome(value: d::LandingOutcome) -> w::HistoryLandingOutcome {
    match value {
        d::LandingOutcome::Done => w::HistoryLandingOutcome::Done,
        d::LandingOutcome::Cancelled => w::HistoryLandingOutcome::Cancelled,
        d::LandingOutcome::Held => w::HistoryLandingOutcome::Held,
    }
}
fn decode_landing_outcome(value: w::HistoryLandingOutcome) -> d::LandingOutcome {
    match value {
        w::HistoryLandingOutcome::Done => d::LandingOutcome::Done,
        w::HistoryLandingOutcome::Cancelled => d::LandingOutcome::Cancelled,
        w::HistoryLandingOutcome::Held => d::LandingOutcome::Held,
    }
}
fn encode_on(value: d::On) -> w::HistoryOn {
    match value {
        d::On::Change => w::HistoryOn::Change,
        d::On::Landing => w::HistoryOn::Landing,
    }
}
fn decode_on(value: w::HistoryOn) -> d::On {
    match value {
        w::HistoryOn::Change => d::On::Change,
        w::HistoryOn::Landing => d::On::Landing,
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
fn decode_after_verdict(value: w::HistoryAfterVerdict) -> d::AfterVerdict {
    match value {
        w::HistoryAfterVerdict::Verified => d::AfterVerdict::Verified,
        w::HistoryAfterVerdict::Red => d::AfterVerdict::Red,
        w::HistoryAfterVerdict::Cleared => d::AfterVerdict::Cleared,
        w::HistoryAfterVerdict::Uncleared => d::AfterVerdict::Uncleared,
    }
}
pub(super) fn encode_change_history(value: &d::ChangeHistory) -> w::ChangeHistory {
    w::ChangeHistory::new(
        &w::CEILINGS,
        w::ChangeHistoryParts {
            last: value.last,
            ready: value.ready,
            queued: value.queued,
            paused: value.paused,
            held: value.held,
            landing_held: value.landing_held,
            delivered: value.delivered,
        },
    )
    .expect(FITS)
}
pub(super) fn decode_change_history(value: w::ChangeHistory) -> d::ChangeHistory {
    let w::ChangeHistoryParts { last, ready, queued, paused, held, landing_held, delivered } = value.into_parts();
    d::ChangeHistory { last, ready, queued, paused, held, landing_held, delivered }
}
pub(super) fn encode_in_landing(value: &d::InLanding) -> w::HistoryInLanding {
    w::HistoryInLanding::new(
        &w::CEILINGS,
        w::HistoryInLandingParts { task: value.task, round: value.round, unit: value.unit },
    )
    .expect(FITS)
}
pub(super) fn decode_in_landing(value: w::HistoryInLanding) -> d::InLanding {
    let w::HistoryInLandingParts { task, round, unit } = value.into_parts();
    d::InLanding { task, round, unit }
}
pub(super) fn encode_mark(value: &d::Mark) -> w::HistoryMark {
    w::HistoryMark::new(&w::CEILINGS, w::HistoryMarkParts { hour: value.hour.as_nanos(), first: value.first })
        .expect(FITS)
}
pub(super) fn decode_mark(value: w::HistoryMark) -> d::Mark {
    let w::HistoryMarkParts { hour, first } = value.into_parts();
    d::Mark { hour: Wall::from_nanos(hour), first }
}
pub(super) fn encode_project(value: &d::Project) -> w::HistoryProject {
    w::HistoryProject::new(
        &w::CEILINGS,
        w::HistoryProjectParts {
            project: value.project,
            next: value.next,
            kept: value.kept,
            whole_since: match value.whole_since {
                Some(value) => Some(value.as_nanos()),
                None => None,
            },
            marks: encode_marks(&value.marks),
            sweeping: value.sweeping,
            removing: value.removing,
        },
    )
    .expect(FITS)
}
pub(super) fn decode_project(value: w::HistoryProject) -> d::Project {
    let w::HistoryProjectParts { project, next, kept, whole_since, marks, sweeping, removing } = value.into_parts();
    d::Project {
        project,
        next,
        kept,
        whole_since: match whole_since {
            Some(value) => Some(Wall::from_nanos(value)),
            None => None,
        },
        marks: decode_marks(marks),
        sweeping,
        removing,
    }
}
pub(super) fn encode_change_index(value: &d::ChangeIndex) -> w::HistoryChange {
    w::HistoryChange::new(
        &w::CEILINGS,
        w::HistoryChangeParts {
            project: value.project,
            change: value.change,
            number: value.number,
            landing: value.landing,
        },
    )
    .expect(FITS)
}
pub(super) fn decode_change_index(value: w::HistoryChange) -> d::ChangeIndex {
    let w::HistoryChangeParts { project, change, number, landing } = value.into_parts();
    d::ChangeIndex { project, change, number, landing }
}
pub(super) fn encode_landing_index(value: &d::LandingIndex) -> w::HistoryLanding {
    w::HistoryLanding::new(
        &w::CEILINGS,
        w::HistoryLandingParts {
            project: value.project,
            landing: value.landing,
            number: value.number,
            change: value.change,
        },
    )
    .expect(FITS)
}
pub(super) fn decode_landing_index(value: w::HistoryLanding) -> d::LandingIndex {
    let w::HistoryLandingParts { project, landing, number, change } = value.into_parts();
    d::LandingIndex { project, landing, number, change }
}
pub(super) fn encode_last(value: &d::Last) -> w::HistoryLast {
    w::HistoryLast::new(
        &w::CEILINGS,
        w::HistoryLastParts {
            project: value.project,
            number: value.number,
            change: value.change,
            at: value.at.as_nanos(),
        },
    )
    .expect(FITS)
}
pub(super) fn decode_last(value: w::HistoryLast) -> d::Last {
    let w::HistoryLastParts { project, number, change, at } = value.into_parts();
    d::Last { project, number, change, at: Wall::from_nanos(at) }
}
pub(super) fn encode_member(value: &d::Member) -> w::HistoryMember {
    w::HistoryMember::new(
        &w::CEILINGS,
        w::HistoryMemberParts { change: value.change, unit: value.unit, head: value.head },
    )
    .expect(FITS)
}
pub(super) fn decode_member(value: w::HistoryMember) -> d::Member {
    let w::HistoryMemberParts { change, unit, head } = value.into_parts();
    d::Member { change, unit, head }
}
pub(super) fn encode_proof(value: &d::Proof) -> w::HistoryProof {
    w::HistoryProof::new(&w::CEILINGS, w::HistoryProofParts { gate: value.gate, run: value.run }).expect(FITS)
}
pub(super) fn decode_proof(value: w::HistoryProof) -> d::Proof {
    let w::HistoryProofParts { gate, run } = value.into_parts();
    d::Proof { gate, run }
}
pub(super) fn encode_event(value: &d::Event) -> w::HistoryEvent {
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
            repository: names::encode_repository(value.repository),
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
pub(super) fn decode_event(value: w::HistoryEvent) -> d::Event {
    let w::HistoryEventParts {
        project,
        number,
        at,
        source_at,
        change,
        repository,
        landing,
        head,
        base,
        operation,
        attempt,
        what,
    } = value.into_parts();
    d::Event {
        project,
        number,
        at: Wall::from_nanos(at),
        source_at: match source_at {
            Some(value) => Some(Wall::from_nanos(value)),
            None => None,
        },
        change,
        repository: names::decode_repository(repository),
        landing: match landing {
            Some(value) => Some(decode_in_landing(value)),
            None => None,
        },
        head,
        base,
        operation,
        attempt,
        what: decode_what(what),
    }
}
fn encode_marks(values: &[d::Mark]) -> List<w::HistoryMark> {
    let mut out = List::with_capacity(u32::try_from(values.len()).expect("bounded history list"));
    for value in values {
        out.push(encode_mark(value)).expect("bounded history list");
    }
    out
}
fn decode_marks(values: List<w::HistoryMark>) -> Box<[d::Mark]> {
    let mut out = List::with_capacity(values.len());
    for value in values.into_boxed() {
        out.push(decode_mark(value)).expect("bounded history list");
    }
    out.into_boxed()
}
fn encode_members(values: &[d::Member]) -> List<w::HistoryMember> {
    let mut out = List::with_capacity(u32::try_from(values.len()).expect("bounded history list"));
    for value in values {
        out.push(encode_member(value)).expect("bounded history list");
    }
    out
}
fn decode_members(values: List<w::HistoryMember>) -> Box<[d::Member]> {
    let mut out = List::with_capacity(values.len());
    for value in values.into_boxed() {
        out.push(decode_member(value)).expect("bounded history list");
    }
    out.into_boxed()
}
fn encode_proofs(values: &[d::Proof]) -> List<w::HistoryProof> {
    let mut out = List::with_capacity(u32::try_from(values.len()).expect("bounded history list"));
    for value in values {
        out.push(encode_proof(value)).expect("bounded history list");
    }
    out
}
fn decode_proofs(values: List<w::HistoryProof>) -> Box<[d::Proof]> {
    let mut out = List::with_capacity(values.len());
    for value in values.into_boxed() {
        out.push(decode_proof(value)).expect("bounded history list");
    }
    out.into_boxed()
}
fn encode_style(value: skein_forge::api::UpdateStyle) -> w::UpdateStyle {
    match value {
        skein_forge::api::UpdateStyle::Merge => w::UpdateStyle::Merge,
        skein_forge::api::UpdateStyle::Rebase => w::UpdateStyle::Rebase,
    }
}
fn decode_style(value: w::UpdateStyle) -> skein_forge::api::UpdateStyle {
    match value {
        w::UpdateStyle::Merge => skein_forge::api::UpdateStyle::Merge,
        w::UpdateStyle::Rebase => skein_forge::api::UpdateStyle::Rebase,
    }
}
fn encode_departure(value: d::Departure) -> w::HistoryDeparture {
    match value {
        d::Departure::Landing(cause) => w::HistoryDeparture::Landing(
            w::HistoryLandingDeparture::new(
                &w::CEILINGS,
                w::HistoryLandingDepartureParts { cause: super::landing::encode_leave(cause) },
            )
            .expect(FITS),
        ),
        d::Departure::Unready => w::HistoryDeparture::Unready,
        d::Departure::Conflict => w::HistoryDeparture::Conflict,
        d::Departure::Repair => w::HistoryDeparture::Repair,
        d::Departure::Ejected => w::HistoryDeparture::Ejected,
    }
}
fn decode_departure(value: w::HistoryDeparture) -> d::Departure {
    match value {
        w::HistoryDeparture::Landing(value) => {
            d::Departure::Landing(super::landing::decode_leave(value.into_parts().cause))
        }
        w::HistoryDeparture::Unready => d::Departure::Unready,
        w::HistoryDeparture::Conflict => d::Departure::Conflict,
        w::HistoryDeparture::Repair => d::Departure::Repair,
        w::HistoryDeparture::Ejected => d::Departure::Ejected,
    }
}
fn encode_by(value: d::By) -> w::HistoryBy {
    match value {
        d::By::Party(number) => {
            w::HistoryBy::Party(w::Number::new(&w::CEILINGS, w::NumberParts { number }).expect(FITS))
        }
        d::By::Task(number) => w::HistoryBy::Task(w::Number::new(&w::CEILINGS, w::NumberParts { number }).expect(FITS)),
    }
}
fn decode_by(value: w::HistoryBy) -> d::By {
    match value {
        w::HistoryBy::Party(value) => d::By::Party(value.number()),
        w::HistoryBy::Task(value) => d::By::Task(value.number()),
    }
}
#[expect(clippy::too_many_lines, reason = "one exhaustive translation of history event kinds")]
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
#[expect(clippy::too_many_lines, reason = "one exhaustive translation of history event kinds")]
fn decode_what(value: w::HistoryWhat) -> d::What {
    match value {
        w::HistoryWhat::Ready => d::What::Ready,
        w::HistoryWhat::Unready(value) => {
            let w::HistoryUnreadyStepParts { why } = value.into_parts();
            d::What::Unready { why: decode_unready(why) }
        }
        w::HistoryWhat::Joined(value) => {
            let w::HistoryJoinedParts { line, place, priority } = value.into_parts();
            d::What::Joined {
                line: decode_line(line),
                place,
                priority: match priority {
                    Some(value) => Some(i32::from_be_bytes(value.to_be_bytes())),
                    None => None,
                },
            }
        }
        w::HistoryWhat::Departed(value) => {
            let w::HistoryDepartedParts { line, cause } = value.into_parts();
            d::What::Departed { line: decode_line(line), cause: decode_departure(cause) }
        }
        w::HistoryWhat::Selected => d::What::Selected,
        w::HistoryWhat::Update(value) => {
            let w::HistoryUpdateParts { style } = value.into_parts();
            d::What::Update { style: decode_style(style) }
        }
        w::HistoryWhat::Updated(value) => {
            let w::HistoryUpdatedParts { outcome } = value.into_parts();
            d::What::Updated { outcome: decode_update_outcome(outcome) }
        }
        w::HistoryWhat::Job(value) => {
            let w::HistoryJobStartedParts { job, task } = value.into_parts();
            d::What::Job { job: decode_job(job), task }
        }
        w::HistoryWhat::JobEnded(value) => {
            let w::HistoryJobEndedParts { job, outcome } = value.into_parts();
            d::What::JobEnded { job: decode_job(job), outcome: decode_job_outcome(outcome) }
        }
        w::HistoryWhat::Conflict(value) => {
            let w::HistoryConflictParts { with, files } = value.into_parts();
            d::What::Conflict { with: decode_with(with), files }
        }
        w::HistoryWhat::Asked(value) => {
            let w::HistoryAskedParts { ask, task, gate } = value.into_parts();
            d::What::Asked { ask: decode_ask(ask), task, gate }
        }
        w::HistoryWhat::Answered(value) => {
            let w::HistoryAnsweredParts { ask, outcome } = value.into_parts();
            d::What::Answered { ask: decode_ask(ask), outcome: decode_answer(outcome) }
        }
        w::HistoryWhat::Pushed => d::What::Pushed,
        w::HistoryWhat::Delivered(value) => {
            let w::HistoryDeliveredParts { checked } = value.into_parts();
            d::What::Delivered { checked }
        }
        w::HistoryWhat::Paused(value) => {
            let w::HistoryPausedParts { why } = value.into_parts();
            d::What::Paused { why: decode_pause(why) }
        }
        w::HistoryWhat::Resumed => d::What::Resumed,
        w::HistoryWhat::Formed(value) => {
            let w::HistoryFormedParts { members, place } = value.into_parts();
            d::What::Formed { members: decode_members(members), place }
        }
        w::HistoryWhat::RoundEnded(value) => {
            let w::HistoryRoundEndedParts { cause } = value.into_parts();
            d::What::RoundEnded { cause: decode_round_cause(cause) }
        }
        w::HistoryWhat::Gate(value) => {
            let w::HistoryGateParts { gate, key } = value.into_parts();
            d::What::Gate { gate, key }
        }
        w::HistoryWhat::GateEnded(value) => {
            let w::HistoryGateEndedParts { gate, key, verdict } = value.into_parts();
            d::What::GateEnded { gate, key, verdict: decode_gate_verdict(verdict) }
        }
        w::HistoryWhat::Proved(value) => {
            let w::HistoryProvedParts { proofs } = value.into_parts();
            d::What::Proved { proofs: decode_proofs(proofs) }
        }
        w::HistoryWhat::Search => d::What::Search,
        w::HistoryWhat::Searched(value) => {
            let w::HistorySearchedParts { green, red, blamed } = value.into_parts();
            d::What::Searched { green, red, blamed }
        }
        w::HistoryWhat::Probe => d::What::Probe,
        w::HistoryWhat::ProbeEnded(value) => {
            let w::HistoryProbeEndedParts { green } = value.into_parts();
            d::What::ProbeEnded { green }
        }
        w::HistoryWhat::Turn(value) => {
            let w::HistoryTurnParts { members } = value.into_parts();
            d::What::Turn { members }
        }
        w::HistoryWhat::Landed => d::What::Landed,
        w::HistoryWhat::Recorded(value) => {
            let w::HistoryRecordedParts { state } = value.into_parts();
            d::What::Recorded { state: decode_recorded_state(state) }
        }
        w::HistoryWhat::Ended(value) => {
            let w::HistoryEndedParts { ending } = value.into_parts();
            d::What::Ended { ending: decode_ending(ending) }
        }
        w::HistoryWhat::LandingEnded(value) => {
            let w::HistoryLandingEndedParts { outcome } = value.into_parts();
            d::What::LandingEnded { outcome: decode_landing_outcome(outcome) }
        }
        w::HistoryWhat::Held(value) => {
            let w::HistoryHoldParts { on, reason, by } = value.into_parts();
            d::What::Held {
                on: decode_on(on),
                reason,
                by: match by {
                    Some(value) => Some(decode_by(value)),
                    None => None,
                },
            }
        }
        w::HistoryWhat::Released(value) => {
            let w::HistoryHoldParts { on, reason, by } = value.into_parts();
            d::What::Released {
                on: decode_on(on),
                reason,
                by: match by {
                    Some(value) => Some(decode_by(value)),
                    None => None,
                },
            }
        }
        w::HistoryWhat::After(value) => {
            let w::HistoryAfterParts { verdict } = value.into_parts();
            d::What::After { verdict: decode_after_verdict(verdict) }
        }
    }
}

pub(super) fn encode_job_record(job: Option<&d::JobRecord>) -> Option<w::HistoryJobRecord> {
    let job = job?;
    let mut members = List::with_capacity(u32::try_from(job.members.len()).expect(FITS));
    for member in &job.members {
        members.push(*member).expect(FITS);
    }
    Some(
        w::HistoryJobRecord::new(
            &w::CEILINGS,
            w::HistoryJobRecordParts {
                task: job.step.task,
                attempt: job.step.attempt,
                job: encode_job(job.step.job),
                outcome: match job.step.outcome {
                    Some(outcome) => Some(encode_job_outcome(outcome)),
                    None => None,
                },
                round: job.round,
                members,
                head: job.head,
                written: job.written,
            },
        )
        .expect(FITS),
    )
}
pub(super) fn decode_job_record(job: Option<w::HistoryJobRecord>) -> Option<d::JobRecord> {
    let w::HistoryJobRecordParts { task, attempt, job, outcome, round, members, head, written } = job?.into_parts();
    Some(d::JobRecord {
        step: d::JobStep {
            task,
            attempt,
            job: decode_job(job),
            outcome: match outcome {
                Some(outcome) => Some(decode_job_outcome(outcome)),
                None => None,
            },
            head: None,
        },
        round,
        members: members.into_boxed(),
        head,
        written,
    })
}
