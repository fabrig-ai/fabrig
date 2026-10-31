//! Child replies resume the continuation named by their handle
//! (`jig/docs/design/domain/engine.md`, sections 4.3 and 4.4).

use crate::Core;
use skein_lib::{Id, Token};

#[derive(Debug)]
pub(crate) enum Flight {
    PersonTask { request: Token, route: crate::PersonTaskRoute },
    Goal { request: Token, route: crate::GoalRoute },
    PersonProposal(crate::PersonProposalRoute),
    Call { request: Token, route: crate::RoutedCall, effect: Option<(u16, Token)> },
    Made { request: Token, route: crate::routing::Made },
    PersonEscalation(crate::routing::PersonEscalation),
    ProcedureProposal { owner: Token, route: crate::effects::ProcedureProposal },
    Internal(Internal),
    Moving { request: Token, task: u64 },
    Retired,
}

#[derive(Debug)]
pub(crate) enum Internal {
    Admission,
    Delegate,
    Repair,
    Acknowledge,
}

impl Core {
    pub(crate) fn person_task_flights(&self) -> u32 {
        let mut count = 0_u32;
        for (_, flight) in &self.flights {
            if let Flight::PersonTask { .. } = flight {
                count = count.checked_add(1).expect("live flight count");
            }
        }
        count
    }

    pub(crate) fn procedure_proposal_flights(&self) -> u32 {
        let mut count = 0_u32;
        for (_, flight) in &self.flights {
            if let Flight::ProcedureProposal { .. } = flight {
                count = count.checked_add(1).expect("live flight count");
            }
        }
        count
    }

    pub(crate) fn fly_person_task(&mut self, request: Token, route: crate::PersonTaskRoute) -> Token {
        self.fly(Flight::PersonTask { request, route })
    }

    pub(crate) fn fly_goal(&mut self, request: Token, route: crate::GoalRoute) -> Token {
        self.fly(Flight::Goal { request, route })
    }

    pub(crate) fn fly_person_proposal(&mut self, route: crate::PersonProposalRoute) -> Token {
        self.fly(Flight::PersonProposal(route))
    }

    pub(crate) fn fly_call(&mut self, request: Token, route: crate::RoutedCall) -> Token {
        self.fly(Flight::Call { request, route, effect: None })
    }

    pub(crate) fn internal(&mut self, reason: Internal) -> Token {
        self.fly(Flight::Internal(reason))
    }

    pub(crate) fn fly(&mut self, flight: Flight) -> Token {
        self.flights.insert(flight).expect("continuations grow with admitted work").token()
    }

    pub(crate) fn land(&mut self, token: Token) -> Option<Flight> {
        let id = Id::from_token(token);
        let flight = core::mem::replace(self.flights.get_mut(id)?, Flight::Retired);
        self.flights.retire(id);
        self.flights.reclaim();
        Some(flight)
    }
}

/// Bind a task request built during admission to the continuation allocated for it.
pub(crate) fn bind(mut event: jig_domain_tasks::Event, token: Token) -> jig_domain_tasks::Event {
    if let Some(reply) = reply_to(&mut event) {
        *reply = skein_lib::ReplyTo::new(token);
    }
    event
}

/// Inspect the core's correlation while retaining the child's one reply right.
pub(crate) fn request_token(event: &mut jig_domain_tasks::Event) -> Option<Token> {
    let reply = reply_to(event)?;
    let token = core::mem::replace(reply, skein_lib::ReplyTo::new(Token::new(0))).into_token();
    *reply = skein_lib::ReplyTo::new(token);
    Some(token)
}

fn reply_to(event: &mut jig_domain_tasks::Event) -> Option<&mut skein_lib::ReplyTo> {
    use jig_domain_tasks as tasks;
    match event {
        tasks::Event::Place { reply_to, .. }
        | tasks::Event::Lift { reply_to, .. }
        | tasks::Event::Compound { reply_to, .. }
        | tasks::Event::ProposePerson { reply_to, .. }
        | tasks::Event::DecidePersonProposal { reply_to, .. }
        | tasks::Event::Procedure { reply_to, .. }
        | tasks::Event::TakePerson { reply_to, .. }
        | tasks::Event::HandBackPerson { reply_to, .. }
        | tasks::Event::AnswerPerson { reply_to, .. }
        | tasks::Event::Propose { reply_to, .. }
        | tasks::Event::DecideProposal { reply_to, .. }
        | tasks::Event::WithdrawProposal { reply_to, .. }
        | tasks::Event::Control { reply_to, .. }
        | tasks::Event::Amend { reply_to, .. }
        | tasks::Event::Move { reply_to, .. }
        | tasks::Event::Subscribe { reply_to, .. }
        | tasks::Event::SubscribeTopic { reply_to, .. }
        | tasks::Event::Unsubscribe { reply_to, .. }
        | tasks::Event::Introduce { reply_to, .. }
        | tasks::Event::Message { reply_to, .. }
        | tasks::Event::PersonQuestion { reply_to, .. }
        | tasks::Event::PersonQuestionAnswer { reply_to, .. }
        | tasks::Event::AuthorizedMessage { reply_to, .. }
        | tasks::Event::InspectEscalations { reply_to, .. }
        | tasks::Event::RecheckEscalations { reply_to, .. }
        | tasks::Event::DecideEscalation { reply_to, .. }
        | tasks::Event::OpenPeriod { reply_to, .. }
        | tasks::Event::CarvePool { reply_to, .. }
        | tasks::Event::ResizePool { reply_to, .. }
        | tasks::Event::ResizePeriod { reply_to, .. }
        | tasks::Event::Turn { reply_to, .. }
        | tasks::Event::Make { reply_to, .. }
        | tasks::Event::MakeResultFollowups { reply_to, .. }
        | tasks::Event::Prepare { reply_to, .. }
        | tasks::Event::Claim { reply_to, .. }
        | tasks::Event::EffectInFlight { reply_to, .. }
        | tasks::Event::Activation { reply_to, .. }
        | tasks::Event::CancelQuestion { reply_to, .. } => Some(reply_to),
        tasks::Event::WithdrawPersonProposal { reply_to, .. }
        | tasks::Event::DecideSettingsProposal { reply_to, .. } => reply_to.as_mut(),
        tasks::Event::MoveToNewest { .. }
        | tasks::Event::Charters { .. }
        | tasks::Event::Resource { .. }
        | tasks::Event::ResourceAccess { .. }
        | tasks::Event::Kinds { .. }
        | tasks::Event::Slots { .. }
        | tasks::Event::AllocationGone { .. }
        | tasks::Event::WakeProcedure { .. }
        | tasks::Event::TickRecurring { .. }
        | tasks::Event::RenewStanding { .. }
        | tasks::Event::RecurringBatch { .. }
        | tasks::Event::StalledProposal { .. }
        | tasks::Event::Notice { .. }
        | tasks::Event::DelegateResult { .. }
        | tasks::Event::RememberStub { .. }
        | tasks::Event::RoutedEscalation { .. }
        | tasks::Event::PeriodNext { .. }
        | tasks::Event::ChargeEffect { .. }
        | tasks::Event::Refusals { .. }
        | tasks::Event::ReadAfresh { .. }
        | tasks::Event::EffectSettled { .. }
        | tasks::Event::Started { .. }
        | tasks::Event::PreparationWaiting { .. }
        | tasks::Event::PreparationFailed { .. }
        | tasks::Event::PreparationUnread { .. }
        | tasks::Event::Hold { .. }
        | tasks::Event::HoldFrom { .. }
        | tasks::Event::EffectsSettled { .. }
        | tasks::Event::Retained { .. }
        | tasks::Event::Settled { .. }
        | tasks::Event::PersonStallNotice { .. }
        | tasks::Event::QuestionEndedNotice { .. }
        | tasks::Event::QuestionOwedNotice { .. }
        | tasks::Event::MetNotice { .. }
        | tasks::Event::Restore { .. }
        | tasks::Event::Check
        | tasks::Event::Restored => None,
    }
}
