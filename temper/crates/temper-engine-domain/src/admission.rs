//! Admission at the service boundary: new work waits at the watermark,
//! while completions of admitted work are always taken (skein's
//! `programming-model.md`, 7.1).
use crate::boundary::{Event, Link, PartyInput, StoreInput, WorkerInput};
use crate::domain::{Domain, admits, step};
use crate::limits::Limits;
use skein_lib::Env;
use temper_engine_domain_forge as forge;

/// Whether an input finishes or advances work the root already admitted,
/// so that it is taken at any backlog (domain/root.md, 4). New work (a
/// worker's call, a party's request, a credential, a restart, a forge's
/// binding, installation or hint) waits at the watermark.
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive classification makes each new boundary variant a compile-time decision"
)]
pub fn completes(event: &Event) -> bool {
    match event {
        Event::Store(_)
        | Event::Released(_)
        | Event::Resume
        | Event::Account(_)
        | Event::Setup { .. }
        | Event::Stop
        | Event::Party(
            PartyInput::Unwatch { .. }
            | PartyInput::Delivered { .. }
            | PartyInput::StreamDelivered { .. }
            | PartyInput::ProcedureStep { .. },
        ) => true,
        Event::Party(
            PartyInput::PublicRead { .. }
            | PartyInput::Whose { .. }
            | PartyInput::ProposeSettings { .. }
            | PartyInput::ChangeModel { .. }
            | PartyInput::ChangeSettings { .. }
            | PartyInput::SignOut { .. }
            | PartyInput::Application { .. }
            | PartyInput::StartRecurring { .. }
            | PartyInput::Period { .. }
            | PartyInput::SignedIn { .. }
            | PartyInput::Ask { .. }
            | PartyInput::Key { .. }
            | PartyInput::ForgeRead { .. }
            | PartyInput::Read { .. }
            | PartyInput::Watch { .. }
            | PartyInput::WatchRun { .. },
        )
        | Event::Credential { .. }
        | Event::Restart { .. } => false,
        Event::Worker(worker) => worker_completes(worker),
        Event::Forge(event) => match event.as_ref() {
            forge::Event::Bind { .. } | forge::Event::Installation { .. } | forge::Event::Hint { .. } => false,
            forge::Event::MakingStarted { .. }
            | forge::Event::MakingCopying { .. }
            | forge::Event::MakingMade { .. }
            | forge::Event::MakingStopped { .. }
            | forge::Event::MakingAdopt { .. }
            | forge::Event::MakingAsked { .. }
            | forge::Event::MakingActed { .. }
            | forge::Event::MakingAgain { .. }
            | forge::Event::OrderLanding { .. }
            | forge::Event::JoinLanding { .. }
            | forge::Event::Landing { .. }
            | forge::Event::VetoLanding { .. }
            | forge::Event::StepLanding { .. }
            | forge::Event::TierWords { .. }
            | forge::Event::TierTask { .. }
            | forge::Event::TierRefused { .. }
            | forge::Event::StepTier { .. }
            | forge::Event::VetoTier { .. }
            | forge::Event::Settle
            | forge::Event::AppRecorded { .. }
            | forge::Event::PlaygroundMade { .. }
            | forge::Event::PlaygroundFailed { .. }
            | forge::Event::ResetActAnswered { .. }
            | forge::Event::SecretsErased { .. }
            | forge::Event::BoxRemoved { .. }
            | forge::Event::BoxClosed
            | forge::Event::ConnectAnswered { .. }
            | forge::Event::AccessAnswered { .. }
            | forge::Event::ReachableAnswered { .. }
            | forge::Event::AcceptAnswered { .. }
            | forge::Event::InstallAnswered { .. }
            | forge::Event::ConnectEnded { .. }
            | forge::Event::Owner { .. }
            | forge::Event::PersonOf { .. }
            | forge::Event::BotLogin { .. }
            | forge::Event::SetUpAnswered { .. }
            | forge::Event::FirstMade { .. }
            | forge::Event::FirstFailed { .. }
            | forge::Event::Connection { .. }
            | forge::Event::Fetched { .. }
            | forge::Event::StatusTask { .. }
            | forge::Event::StatusRefused { .. }
            | forge::Event::StatusHeard { .. }
            | forge::Event::AddDevice { .. }
            | forge::Event::RemoveDevice { .. }
            | forge::Event::Pushed { .. }
            | forge::Event::SetDraft { .. }
            | forge::Event::SetAway { .. }
            | forge::Event::Overturn { .. }
            | forge::Event::Decided { .. }
            | forge::Event::Looked { .. }
            | forge::Event::Dismiss { .. }
            | forge::Event::Application { .. }
            | forge::Event::ApplicationAnswered { .. }
            | forge::Event::ForgetProjection { .. }
            | forge::Event::PlanDocument { .. }
            | forge::Event::PlanBrief { .. }
            | forge::Event::GatherPlanned { .. }
            | forge::Event::GatherBrief { .. }
            | forge::Event::GatherBriefHeld { .. }
            | forge::Event::CutBrief { .. }
            | forge::Event::TakeBrief { .. }
            | forge::Event::DropBrief { .. }
            | forge::Event::Adopt { .. }
            | forge::Event::ReleaseProject { .. }
            | forge::Event::ForgetAdoption { .. }
            | forge::Event::Names { .. }
            | forge::Event::Rename { .. }
            | forge::Event::Unnamed { .. }
            | forge::Event::Hold { .. }
            | forge::Event::Claim { .. }
            | forge::Event::Answered { .. }
            | forge::Event::Lost { .. }
            | forge::Event::KeepProposedEffect { .. }
            | forge::Event::DropProposedEffect { .. }
            | forge::Event::Enqueue { .. }
            | forge::Event::KeepProjection { .. }
            | forge::Event::Project { .. }
            | forge::Event::FailProjection { .. }
            | forge::Event::ProjectDesired { .. }
            | forge::Event::DropChange { .. }
            | forge::Event::Change { .. }
            | forge::Event::Priority { .. }
            | forge::Event::QueueRepairStarted { .. }
            | forge::Event::Delegated { .. }
            | forge::Event::DelegateAdmissionRefused { .. }
            | forge::Event::DelegateRefused { .. }
            | forge::Event::DelegateResult { .. }
            | forge::Event::RelayRefused { .. }
            | forge::Event::Relayed { .. }
            | forge::Event::StartConflicted { .. }
            | forge::Event::ReviewRequested { .. }
            | forge::Event::ReviewsChanged { .. }
            | forge::Event::StepChange { .. }
            | forge::Event::SavedWork { .. }
            | forge::Event::Closer { .. }
            | forge::Event::ReadCloserSelection { .. }
            | forge::Event::CheckCloser { .. }
            | forge::Event::ChangeEnding { .. }
            | forge::Event::ReleaseChange { .. }
            | forge::Event::GoalReleased { .. }
            | forge::Event::SettleEffects { .. }
            | forge::Event::Release { .. }
            | forge::Event::ContinueRelease { .. }
            | forge::Event::ContinueGoalBranch { .. }
            | forge::Event::RetryGoalBranch { .. }
            | forge::Event::ReleaseProjection { .. }
            | forge::Event::Committed { .. }
            | forge::Event::Resolve { .. }
            | forge::Event::Withdraw { .. }
            | forge::Event::VetoChange { .. }
            | forge::Event::Subscribe { .. }
            | forge::Event::Unsubscribe { .. }
            | forge::Event::Client(_)
            | forge::Event::Restore { .. }
            | forge::Event::Restored { .. }
            | forge::Event::ReadAfresh
            | forge::Event::ProducedTitle { .. }
            | forge::Event::Follow { .. }
            | forge::Event::FollowChangeAmended { .. }
            | forge::Event::FollowAct { .. }
            | forge::Event::SettleOutbox => true,
        },
    }
}

/// Hand new work back unchanged at the watermark. Store answers and released
/// continuations are always taken. Other completions wait only for earlier
/// hand-offs/store answers and, if they keep a payload, its slab slot.
#[expect(clippy::result_large_err, reason = "the service admission boundary returns the original event by value")]
pub fn try_step(domain: &mut Domain, env: &Env<Limits>, event: Event) -> Result<(), Event> {
    domain.changed();
    crate::domain::invalidate_input(domain, &event);
    match event {
        event @ Event::Store(StoreInput::ReadLoaded { .. } | StoreInput::Switched) => {
            if domain.store_answers.is_empty() && domain.work.is_empty() {
                step(domain, env, event);
            } else {
                domain.store_answers.push(event);
            }
            Ok(())
        }
        event @ (Event::Store(
            StoreInput::MissingLink { .. }
            | StoreInput::Committed { .. }
            | StoreInput::Uncommitted { .. }
            | StoreInput::Loaded { .. }
            | StoreInput::Unloaded { .. },
        )
        | Event::Resume) => {
            step(domain, env, event);
            Ok(())
        }
        Event::Released(released) => {
            step(domain, env, Event::Released(released));
            Ok(())
        }
        Event::Party(PartyInput::PublicRead { reply, read }) => {
            if crate::limits::backlog(domain) >= u64::from(env.limits.public_watermark)
                || domain.public_reads >= env.limits.public_reads
            {
                crate::route::public::busy(domain, reply);
            } else {
                step(domain, env, Event::Party(PartyInput::PublicRead { reply, read }));
            }
            Ok(())
        }
        event @ (Event::Worker(_)
        | Event::Party(_)
        | Event::Account(_)
        | Event::Credential { .. }
        | Event::Forge(_)
        | Event::Setup { .. }
        | Event::Restart { .. }
        | Event::Stop) => {
            let ordered = domain.work.is_empty() && domain.store_answers.is_empty();
            let admitted = if completes(&event) { ordered } else { admits(domain, &env.limits) };
            let keeps_payload = match &event {
                Event::Worker(input) => crate::link::keeps_payload(input),
                Event::Party(_)
                | Event::Account(_)
                | Event::Credential { .. }
                | Event::Forge(_)
                | Event::Setup { .. }
                | Event::Restart { .. }
                | Event::Stop
                | Event::Store(_)
                | Event::Released(_)
                | Event::Resume => false,
            };
            if !admitted || keeps_payload && !crate::link::room(domain) {
                return Err(event);
            }
            step(domain, env, event);
            Ok(())
        }
    }
}

fn worker_completes(worker: &WorkerInput) -> bool {
    match worker {
        WorkerInput::DecodedHostCall { .. }
        | WorkerInput::RenderedHostCall { .. }
        | WorkerInput::ResultRead { .. }
        | WorkerInput::Link { link: Link::Hello { .. } | Link::Lost, .. } => true,
        WorkerInput::Link { link: Link::Up { up, .. }, .. } => !up.new_work(),
    }
}
