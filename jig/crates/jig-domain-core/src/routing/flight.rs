//! Child replies resume the continuation named by their handle
//! (`jig/docs/design/domain/engine.md`, sections 4.3 and 4.4).

use crate::Core;
use alloc::boxed::Box;
use skein_lib::{Id, Token};

#[derive(Debug)]
pub(crate) enum Flight {
    Party { request: Token },
    PartyPart { application: Token, original: Token, connector: u16 },
    PersonTask { request: Token, route: crate::PersonTaskRoute },
    Goal { request: Token, route: crate::GoalRoute },
    PersonProposal(crate::PersonProposalRoute),
    Call { request: Token, route: crate::RoutedCall, effect: Option<(u16, Token)> },
    Made { request: Token, route: super::Made },
    PersonEscalation(super::PersonEscalation),
    ProcedureProposal { owner: Token, route: crate::effects::ProcedureProposal },
    Internal(Internal),
    Moving { request: Token, task: u64 },
    EndedMark { request: Token, route: super::ended_marks::Loaded },
    Key { request: Token, route: Option<super::party::key_routes::Flight> },
    Application { request: Token, route: super::application_routes::Flight },
    Watch { watcher: Token, binding: crate::WatchBinding },
    Delegating { request: Token, route: crate::Delegating },
    Saying { request: Token, task: u64, question: Option<u64> },
    ResultRead(super::run::results::ResultRead),
    CreatingInputs { request: Token, route: super::party::inputs::Inputs },
    Creating { request: Token, route: super::Creation },
    AwaitingResources(crate::effects::resources::Batch),
    CreatingDelegates { request: Token, route: super::DelegateCreation },
    Workspace { task: u64, attempt: u64 },
    ProcedureCreation { task: u64, step: u64, route: super::ProcedureCreation },
    EffectStaging,
    Effect { route: Box<crate::effects::Flight> },
    Note(super::NoteRoute),
    SettingsOffering { request: Token, by: u64, number: u32, value: crate::settings::Proposal },
    Retired,
}

#[derive(Debug)]
pub(crate) enum Internal {
    Admission,
    Deletion { project: u32 },
    Delegate,
    Repair,
    Acknowledge,
}

impl Core {
    // Party requests may wait on external actions; their active work has separate flights.
    pub(crate) fn flights_idle(&self) -> bool {
        for (_, flight) in &self.flights {
            match flight.kind() {
                Kind::Party | Kind::Key | Kind::Application | Kind::Watch => {}
                Kind::Workspace
                | Kind::ProcedureCreation
                | Kind::Task
                | Kind::EndedMark
                | Kind::ResultRead
                | Kind::CreatingInputs
                | Kind::Creating
                | Kind::AwaitingResources
                | Kind::CreatingDelegates
                | Kind::EffectStaging
                | Kind::Effect
                | Kind::Note
                | Kind::SettingsOffering => return false,
            }
        }
        true
    }

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
        | tasks::Event::CloseProject { .. }
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

impl Core {
    pub(crate) fn take_ended_mark(&mut self, token: Token) -> Option<(Token, super::ended_marks::Loaded)> {
        match self.land_kind(token, Kind::EndedMark) {
            Some(Flight::EndedMark { request, route }) => Some((request, route)),
            Some(_) | None => None,
        }
    }
    pub(crate) fn key_mut(&mut self, token: Token) -> Option<(Token, &mut Option<super::party::key_routes::Flight>)> {
        match self.flights.get_mut(Id::from_token(token)) {
            Some(Flight::Key { request, route }) => Some((*request, route)),
            Some(_) | None => None,
        }
    }
    pub(crate) fn take_key(&mut self, token: Token) -> Option<(Token, Option<super::party::key_routes::Flight>)> {
        match self.land_kind(token, Kind::Key) {
            Some(Flight::Key { request, route }) => Some((request, route)),
            Some(_) | None => None,
        }
    }
    pub(crate) fn application_mut(&mut self, token: Token) -> Option<(Token, &mut super::application_routes::Flight)> {
        match self.flights.get_mut(Id::from_token(token)) {
            Some(Flight::Application { request, route }) => Some((*request, route)),
            Some(_) | None => None,
        }
    }
    pub(crate) fn take_application(&mut self, token: Token) -> Option<(Token, super::application_routes::Flight)> {
        match self.land_kind(token, Kind::Application) {
            Some(Flight::Application { request, route }) => Some((request, route)),
            Some(_) | None => None,
        }
    }
    pub(crate) fn take_creating_inputs(&mut self, token: Token) -> Option<(Token, super::party::inputs::Inputs)> {
        match self.land_kind(token, Kind::CreatingInputs) {
            Some(Flight::CreatingInputs { request, route }) => Some((request, route)),
            Some(_) | None => None,
        }
    }
    pub(crate) fn creation_mut(&mut self, token: Token) -> Option<(Token, &mut super::Creation)> {
        match self.flights.get_mut(Id::from_token(token)) {
            Some(Flight::Creating { request, route }) => Some((*request, route)),
            Some(_) | None => None,
        }
    }
    pub(crate) fn take_creation(&mut self, token: Token) -> Option<(Token, super::Creation)> {
        match self.land_kind(token, Kind::Creating) {
            Some(Flight::Creating { request, route }) => Some((request, route)),
            Some(_) | None => None,
        }
    }
    pub(crate) fn delegate_creation_mut(&mut self, token: Token) -> Option<(Token, &mut super::DelegateCreation)> {
        match self.flights.get_mut(Id::from_token(token)) {
            Some(Flight::CreatingDelegates { request, route }) => Some((*request, route)),
            Some(_) | None => None,
        }
    }
    pub(crate) fn take_delegate_creation(&mut self, token: Token) -> Option<(Token, super::DelegateCreation)> {
        match self.land_kind(token, Kind::CreatingDelegates) {
            Some(Flight::CreatingDelegates { request, route }) => Some((request, route)),
            Some(_) | None => None,
        }
    }
    #[cfg(test)]
    pub(crate) fn key_flights(&self) -> u32 {
        self.count_flights(Kind::Key)
    }
    pub(crate) fn watch_flights(&self) -> u32 {
        self.count_flights(Kind::Watch)
    }
    pub(crate) fn result_read_flights(&self) -> u32 {
        self.count_flights(Kind::ResultRead)
    }
    pub(crate) fn creating_inputs_flights(&self) -> u32 {
        self.count_flights(Kind::CreatingInputs)
    }
    pub(crate) fn creating_flights(&self) -> u32 {
        self.count_flights(Kind::Creating)
    }
    pub(crate) fn awaiting_resources_flights(&self) -> u32 {
        self.count_flights(Kind::AwaitingResources)
    }
    #[cfg(test)]
    pub(crate) fn note_flights(&self) -> u32 {
        self.count_flights(Kind::Note)
    }
    pub(crate) fn settings_offering_flights(&self) -> u32 {
        self.count_flights(Kind::SettingsOffering)
    }
    pub(crate) fn take_result_read(&mut self, token: Token) -> Option<super::run::results::ResultRead> {
        match self.land_kind(token, Kind::ResultRead) {
            Some(Flight::ResultRead(read)) => Some(read),
            Some(_) | None => None,
        }
    }
    pub(crate) fn take_note(&mut self, token: Token) -> Option<super::NoteRoute> {
        match self.land_kind(token, Kind::Note) {
            Some(Flight::Note(route)) => Some(route),
            Some(_) | None => None,
        }
    }
    /// Name a connector payload before handing it to the core's effect route
    /// (`jig/docs/design/domain/engine.md`, section 4.4).
    pub fn effect_owner(&mut self) -> Token {
        self.fly(Flight::EffectStaging)
    }

    pub(crate) fn effect(&self, token: Token) -> Option<&crate::effects::Flight> {
        match self.flights.get(Id::from_token(token)) {
            Some(Flight::Effect { route, .. }) => Some(route),
            Some(_) | None => None,
        }
    }

    pub(crate) fn effect_mut(&mut self, token: Token) -> Option<&mut crate::effects::Flight> {
        match self.flights.get_mut(Id::from_token(token)) {
            Some(Flight::Effect { route, .. }) => Some(route),
            Some(_) | None => None,
        }
    }

    pub(crate) fn take_effect(&mut self, token: Token) -> Option<Box<crate::effects::Flight>> {
        match self.land_kind(token, Kind::Effect) {
            Some(Flight::Effect { route, .. }) => Some(route),
            Some(_) | None => None,
        }
    }
    pub(crate) fn watch_binding(&self, token: Token) -> Option<&crate::WatchBinding> {
        match self.flights.get(Id::from_token(token)) {
            Some(Flight::Watch { binding, .. }) => Some(binding),
            Some(_) | None => None,
        }
    }
    pub(crate) fn watcher(&self, token: Token) -> Option<Token> {
        match self.flights.get(Id::from_token(token)) {
            Some(Flight::Watch { watcher, .. }) => Some(*watcher),
            Some(_) | None => None,
        }
    }
    #[expect(clippy::wildcard_enum_match_arm, reason = "select one continuation by its external identity")]
    pub(crate) fn watch_token(&self, watcher: Token) -> Option<Token> {
        for (id, flight) in &self.flights {
            match flight {
                Flight::Watch { watcher: named, .. } if *named == watcher => return Some(id.token()),
                _ => {}
            }
        }
        None
    }
    #[expect(clippy::wildcard_enum_match_arm, reason = "select one continuation by its external identity")]
    pub(crate) fn take_settings_offering(
        &mut self,
        request: Option<Token>,
    ) -> Option<(u64, u32, crate::settings::Proposal)> {
        let mut found = None;
        for (id, flight) in &self.flights {
            match flight {
                Flight::SettingsOffering { request: original, .. }
                    if request.is_none() || request == Some(*original) =>
                {
                    found = Some(id.token());
                    break;
                }
                _ => {}
            }
        }
        match self.land_kind(found?, Kind::SettingsOffering) {
            Some(Flight::SettingsOffering { by, number, value, .. }) => Some((by, number, value)),
            Some(_) | None => None,
        }
    }
}

#[cfg(test)]
impl Core {
    pub(crate) fn named_watch(&self, watcher: Token) -> Option<&crate::WatchBinding> {
        self.watch_binding(self.watch_token(watcher)?)
    }
    pub(crate) fn named_watch_mut(&mut self, watcher: Token) -> Option<&mut crate::WatchBinding> {
        let token = self.watch_token(watcher)?;
        match self.flights.get_mut(Id::from_token(token)) {
            Some(Flight::Watch { binding, .. }) => Some(binding),
            Some(_) | None => None,
        }
    }
}

impl Core {
    /// The parent application request naming connector state while this party part
    /// awaits its answer (jig/docs/design/domain/connectors.md, 5.5).
    #[must_use]
    pub fn application_part_request(&self, token: Token) -> Option<Token> {
        match self.flights.get(Id::from_token(token)) {
            Some(Flight::PartyPart { application, .. }) => Some(*application),
            Some(_) | None => None,
        }
    }

    pub(crate) fn people_request(&self, token: Token) -> Option<Token> {
        match self.flights.get(Id::from_token(token)) {
            Some(Flight::Party { request }) => Some(*request),
            Some(Flight::PartyPart { original, .. }) => Some(*original),
            Some(_) | None => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Workspace,
    ProcedureCreation,
    Task,
    Party,
    EndedMark,
    Key,
    Application,
    Watch,
    ResultRead,
    CreatingInputs,
    Creating,
    AwaitingResources,
    CreatingDelegates,
    EffectStaging,
    Effect,
    Note,
    SettingsOffering,
}

impl Flight {
    fn kind(&self) -> Kind {
        match self {
            Self::PersonTask { .. }
            | Self::Goal { .. }
            | Self::PersonProposal(_)
            | Self::Call { .. }
            | Self::Made { .. }
            | Self::PersonEscalation(_)
            | Self::ProcedureProposal { .. }
            | Self::Internal(_)
            | Self::Moving { .. }
            | Self::Delegating { .. }
            | Self::Saying { .. }
            | Self::Retired => Kind::Task,
            Self::Party { .. } | Self::PartyPart { .. } => Kind::Party,
            Self::EndedMark { .. } => Kind::EndedMark,
            Self::Key { .. } => Kind::Key,
            Self::Application { .. } => Kind::Application,
            Self::Watch { .. } => Kind::Watch,
            Self::ResultRead(_) => Kind::ResultRead,
            Self::CreatingInputs { .. } => Kind::CreatingInputs,
            Self::Creating { .. } => Kind::Creating,
            Self::AwaitingResources(_) => Kind::AwaitingResources,
            Self::CreatingDelegates { .. } => Kind::CreatingDelegates,
            Self::Workspace { .. } => Kind::Workspace,
            Self::ProcedureCreation { .. } => Kind::ProcedureCreation,
            Self::EffectStaging => Kind::EffectStaging,
            Self::Effect { .. } => Kind::Effect,
            Self::Note(_) => Kind::Note,
            Self::SettingsOffering { .. } => Kind::SettingsOffering,
        }
    }
}

impl Core {
    pub(crate) fn land_kind(&mut self, token: Token, kind: Kind) -> Option<Flight> {
        let id = Id::from_token(token);
        let slot = self.flights.get_mut(id)?;
        if slot.kind() != kind {
            return None;
        }
        let flight = core::mem::replace(slot, Flight::Retired);
        self.flights.retire(id);
        self.flights.reclaim();
        Some(flight)
    }

    pub(crate) fn land_task(&mut self, token: Token) -> Option<Flight> {
        self.land_kind(token, Kind::Task)
    }
    #[expect(clippy::wildcard_enum_match_arm, reason = "select one continuation by its external identity")]
    pub(crate) fn workspace_flight(&self, task: u64) -> Option<(Token, u64)> {
        for (id, flight) in &self.flights {
            match flight {
                Flight::Workspace { task: owner, attempt } if *owner == task => return Some((id.token(), *attempt)),
                _ => {}
            }
        }
        None
    }

    #[expect(clippy::wildcard_enum_match_arm, reason = "select one continuation by its external identity")]
    pub(crate) fn procedure_creation_token(&self, task: u64, step: u64) -> Option<Token> {
        for (id, flight) in &self.flights {
            match flight {
                Flight::ProcedureCreation { task: owner, step: asked, .. } if *owner == task && *asked == step => {
                    return Some(id.token());
                }
                _ => {}
            }
        }
        None
    }
}

impl Core {
    fn count_flights(&self, kind: Kind) -> u32 {
        let mut count = 0_u32;
        for (_, flight) in &self.flights {
            if flight.kind() == kind {
                count = count.checked_add(1).expect("live flight count");
            }
        }
        count
    }
}
