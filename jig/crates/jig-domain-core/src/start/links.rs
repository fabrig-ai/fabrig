//! Cross-child required links before sweep (jig/docs/design/domain/engine/restart.md).
//! Kept deletions own remaining project rows (jig/docs/design/domain/people/acts.md, 2).
use crate::{
    Core, CoreKey, Key, accounts, people,
    start::{Failure, Linked},
    tasks,
};

impl Core {
    /// Re-derive a waiting holder through the same selection used while serving.
    pub(crate) fn route_waiting_escalation(
        &mut self,
        env: &skein_lib::Env<crate::Limits>,
        context: &tasks::EscalationContext,
        out: &mut skein_lib::Queue<tasks::Request>,
    ) -> bool {
        let role = self.people.role(context.requester, context.project);
        let Some(holder) = self.escalation_recipient(&env.limits, context, role) else {
            return context.project == people::DEPLOYMENT;
        };
        match context.escalation {
            tasks::Escalation::Waiting { holder: old, .. } if old == holder => return true,
            tasks::Escalation::Waiting { revision: u64::MAX, .. } => return false,
            tasks::Escalation::Unheld { .. }
            | tasks::Escalation::Routing { .. }
            | tasks::Escalation::Waiting { .. }
            | tasks::Escalation::Rejected { .. } => {}
        }
        let Some(entry) = crate::fresh(&mut self.counters, crate::Family::Message) else { return false };
        crate::routing::step::step_tasks(
            self,
            env,
            tasks::Event::RoutedEscalation {
                task: context.task,
                revision: context.escalation.revision(),
                holder,
                entry,
            },
            out,
        );
        true
    }

    /// First live task or call whose required run proof is absent (domain/engine/restart.md).
    #[must_use]
    pub fn missing_proof_link(&self) -> Option<(Key, Key)> {
        for (task, _) in &self.restoring_proofs {
            if !self.proofs.contains_key(task) {
                return Some((Key::Tasks(tasks::Key::Live(*task)), Key::Core(CoreKey::RunProof(*task))));
            }
        }
        for (key, _) in &self.call_parts {
            if !self.proofs.contains_key(&key.task) {
                return Some((Key::Core(CoreKey::Call(*key)), Key::Core(CoreKey::RunProof(key.task))));
            }
        }
        None
    }

    /// First live task whose charter is absent from its held settings (domain/engine/restart.md).
    #[must_use]
    pub fn missing_charter_link(&self) -> Option<(Key, u32, u32)> {
        for view in self.tasks.view_tasks() {
            let row = self.tasks.task(view.number).expect("live view owner");
            if let tasks::Executor::Agent { charter } = row.executor {
                for revision in [Some(row.settings.task), row.settings.run].into_iter().flatten() {
                    if self.held(revision) && self.charter_at(charter, revision).is_none() {
                        return Some((Key::Tasks(tasks::Key::Live(row.number)), charter, revision));
                    }
                }
            }
        }
        None
    }

    pub(super) fn start_cross_links(&self) -> Result<(), Failure> {
        for view in self.tasks.view_tasks() {
            self.link_task_row(self.tasks.task(view.number).expect("live view owner"))?;
        }
        self.link_placed_holds()?;
        self.link_person_proposals()?;
        self.link_settings_proposals()?;
        self.link_projection_tasks()?;
        self.link_resource_projects();
        self.link_people_answers()?;
        self.link_other_children()?;
        Ok(())
    }

    fn link_task_row(&self, row: &tasks::TaskRecord) -> Result<(), Failure> {
        let source = Key::Tasks(tasks::Key::Live(row.number));
        self.link_project(&source, row.project)?;
        self.link_party(&source, row.requester)?;
        self.link_executor(&source, row.executor)?;
        self.link_funder(&source, row.funder)?;
        match row.executor {
            tasks::Executor::Agent { charter } => {
                for revision in [Some(row.settings.task), row.settings.run].into_iter().flatten() {
                    require_link(
                        self.held(revision),
                        &source,
                        Linked::Record(Key::Core(CoreKey::Settings(crate::settings::Key::Held(revision)))),
                    )?;
                    require_link(
                        self.charter_at(charter, revision).is_some(),
                        &source,
                        Linked::Charter { charter, revision },
                    )?;
                }
            }
            tasks::Executor::Person(_) | tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } => {}
        }
        if let Some(number) = row.placed {
            require_link(
                self.tasks.placed_hold(number).is_some(),
                &source,
                Linked::Record(Key::Tasks(tasks::Key::PlacedHold(number))),
            )?;
        }
        if let Some(recurring) = &row.recurring {
            self.link_batch(&source, &recurring.template.batch)?;
        }
        if let Some(person) = row.taken_by {
            self.link_person(&source, person)?;
        }
        if let Some(answer) = row.person_answer {
            self.link_person(&source, answer.by)?;
        }
        if let Some(grace) = &row.grace {
            self.link_party(&source, grace.by)?;
        }
        for added in &row.added {
            self.link_party(&source, added.by)?;
        }
        for message in &row.inbox {
            self.link_party(&source, message.from)?;
        }
        match row.escalation {
            tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Person(person), .. }
            | tasks::Escalation::Rejected { by: person, .. } => self.link_person(&source, person)?,
            tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Role { project, .. }, .. } => {
                self.link_project(&source, project)?;
            }
            tasks::Escalation::Unheld { .. }
            | tasks::Escalation::Routing { .. }
            | tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(_), .. } => {}
        }
        if let Some(proposal) = &row.proposal {
            self.link_proposal_action(&source, proposal)?;
            match proposal.state {
                tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Person(person), .. } => {
                    self.link_person(&source, person)?;
                }
                tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Policy { project, .. }, .. } => {
                    self.link_project(&source, project)?;
                }
                tasks::ProposalState::Accepted { by } | tasks::ProposalState::Rejected { by, .. } => {
                    self.link_party(&source, by)?;
                }
                tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Task(_), .. }
                | tasks::ProposalState::Withdrawn => {}
            }
        }
        match &row.phase {
            tasks::Phase::Closing(closing) | tasks::Phase::Held { was: tasks::Was::Closing(closing), .. } => {
                match closing.ending {
                    tasks::Ending::Cancelled { by, .. } => self.link_party(&source, by)?,
                    tasks::Ending::Done(_) | tasks::Ending::Failed { .. } => {}
                }
            }
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } | tasks::Phase::Ended(_) => {}
        }
        Ok(())
    }

    fn link_placed_holds(&self) -> Result<(), Failure> {
        let mut after = 0;
        while let Some(hold) = self.tasks.next_placed_hold(after) {
            after = hold.number;
            let source = Key::Tasks(tasks::Key::PlacedHold(hold.number));
            self.link_project(&source, hold.project)?;
            match hold.owner {
                tasks::Owner::Party { person } => self.link_person(&source, person)?,
                tasks::Owner::Task { task } => self.link_task(&source, task)?,
            }
        }
        Ok(())
    }

    fn link_person_proposals(&self) -> Result<(), Failure> {
        for (_, proposal) in self.tasks.person_proposals() {
            let source = Key::Tasks(tasks::Key::PersonProposal(proposal.number));
            self.link_person(&source, proposal.proposer)?;
            self.link_project(&source, proposal.project)?;
            match proposal.state {
                tasks::PersonProposalState::Accepted { by }
                | tasks::PersonProposalState::SettingsAccepted { by, .. }
                | tasks::PersonProposalState::Rejected { by, .. } => self.link_party(&source, by)?,
                tasks::PersonProposalState::Pending { .. } | tasks::PersonProposalState::Withdrawn { .. } => {}
            }
            match &proposal.action {
                tasks::PersonAction::Batch(batch) => self.link_batch(&source, batch)?,
                tasks::PersonAction::Amend { amendment, .. } => self.link_amendment(&source, amendment)?,
                tasks::PersonAction::Settings { against, .. } => {
                    self.link_settings_proposal(
                        &source,
                        people::Proposer::Party(proposal.proposer),
                        proposal.number,
                        *against,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn link_settings_proposals(&self) -> Result<(), Failure> {
        for ((proposer, number), proposal) in &self.settings_proposals {
            let source =
                Key::Core(CoreKey::Settings(crate::settings::Key::Proposal { proposer: *proposer, proposal: *number }));
            self.link_revision(&source, proposal.against)?;
            let present = match proposer {
                people::Proposer::Party(person) => {
                    self.link_person(&source, *person)?;
                    match self.tasks.person_proposals().get(&u64::from(*number)) {
                        Some(row) => row.proposer == *person,
                        None => false,
                    }
                }
                people::Proposer::Task(task) => {
                    self.link_task(&source, *task)?;
                    match self.tasks.task(*task) {
                        Some(row) => match &row.proposal {
                            Some(proposal) => proposal.number == u64::from(*number),
                            None => false,
                        },
                        None => false,
                    }
                }
            };
            require_link(
                present,
                &source,
                Linked::Record(match proposer {
                    people::Proposer::Party(_) => Key::Tasks(tasks::Key::PersonProposal(u64::from(*number))),
                    people::Proposer::Task(task) => Key::Tasks(tasks::Key::Live(*task)),
                }),
            )?;
        }
        Ok(())
    }

    fn link_projection_tasks(&self) -> Result<(), Failure> {
        for (goal, projection) in &self.projections {
            for row in &projection.plan {
                match row.phase {
                    tasks::Phase::Ended(_)
                    | tasks::Phase::Closing(_)
                    | tasks::Phase::Held { was: tasks::Was::Closing(_), .. } => {}
                    tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } => {
                        self.link_task(
                            &Key::Core(CoreKey::Projection(crate::ProjectionKey::Task {
                                goal: *goal,
                                task: row.number,
                            })),
                            row.number,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    fn link_resource_projects(&self) {
        for (key, _) in &self.resource_roles {
            assert!(
                key.project == people::DEPLOYMENT
                    || self.authority.policy(key.project).is_some()
                    || self.deletions.contains(key.project),
                "{key:?} links missing project {}",
                key.project
            );
        }
    }

    fn link_people_answers(&self) -> Result<(), Failure> {
        let mut after = None;
        while let Some((key, outcome)) = self.people.next_answer(after) {
            after = Some(key);
            let source = Key::People(people::Key::Answer(key));
            match outcome {
                people::Outcome::Applied { connector, .. } => {
                    require_link(self.connectors.contains(connector), &source, Linked::Connector(*connector))?;
                }
                people::Outcome::Left { task }
                | people::Outcome::Stopped { task }
                | people::Outcome::Cancelled { task }
                | people::Outcome::Released { task }
                | people::Outcome::PersonTaken { task }
                | people::Outcome::PersonHandedBack { task }
                | people::Outcome::PersonAnswered { task }
                | people::Outcome::Moved { task }
                | people::Outcome::Said { task, .. }
                | people::Outcome::QuestionAnswered { task, .. }
                | people::Outcome::Amended { task, .. }
                | people::Outcome::EscalationDecided { task, .. }
                | people::Outcome::ProposalDecided { proposer: people::Proposer::Task(task), .. }
                | people::Outcome::MarkedSeen { seen: people::Seen::Conversation { task, .. } } => {
                    self.link_allocated_task(&source, *task)?;
                }
                people::Outcome::Created { first, last, given } => {
                    self.link_allocated_task(&source, *first)?;
                    self.link_allocated_task(&source, *last)?;
                    if let Some(given) = given {
                        self.link_allocated_task(&source, given.task)?;
                    }
                }
                people::Outcome::Restored { .. }
                | people::Outcome::MarkedSeen { seen: people::Seen::Results { .. } }
                | people::Outcome::Compound { .. }
                | people::Outcome::CompoundRefused { .. }
                | people::Outcome::Again(_)
                | people::Outcome::SignedOut
                | people::Outcome::NoteEdited { .. }
                | people::Outcome::ServiceMade { .. }
                | people::Outcome::Adopted { .. }
                | people::Outcome::Proposed { .. }
                | people::Outcome::Withdrawn { .. }
                | people::Outcome::Lifted { .. }
                | people::Outcome::ProposalDecided { proposer: people::Proposer::Party(_), .. }
                | people::Outcome::RolesSet { .. }
                | people::Outcome::PolicyChanged { .. }
                | people::Outcome::Renamed { .. }
                | people::Outcome::ProjectRemoved { .. }
                | people::Outcome::VisibilitySet { .. }
                | people::Outcome::ProjectMade { .. }
                | people::Outcome::SettingsChanged { .. }
                | people::Outcome::MovedToNewest { .. }
                | people::Outcome::PoolSet { .. }
                | people::Outcome::AccountSignInStarted { .. }
                | people::Outcome::AccountProgramDeclared { .. }
                | people::Outcome::AccountKeyKept { .. }
                | people::Outcome::AccountMovingOut { .. }
                | people::Outcome::AccountFinishing { .. }
                | people::Outcome::AccountMovedIn { .. }
                | people::Outcome::AccountReading { .. }
                | people::Outcome::AccountChanged
                | people::Outcome::AccountRemoved { .. }
                | people::Outcome::AccountSignedOut { .. }
                | people::Outcome::Refused(_) => {}
            }
        }
        Ok(())
    }

    fn link_other_children(&self) -> Result<(), Failure> {
        for (number, account) in self.accounts.accounts_by_number() {
            match account.holder() {
                accounts::Holder::Party(person) => {
                    self.link_person(&Key::Accounts(accounts::Key::Account { number: *number }), person)?;
                }
                accounts::Holder::Deployment => {}
            }
        }
        for (number, proof) in &self.proofs {
            let source = Key::Core(CoreKey::RunProof(*number));
            self.link_task(&source, proof.task)?;
        }
        if let Some((record, target)) = self.missing_proof_link() {
            return Err(Failure::Link { record, linked: Linked::Record(target) });
        }
        for ((party, task), _) in self.people.read_conversations() {
            self.link_task(&Key::People(people::Key::ReadConversation { party: *party, task: *task }), *task)?;
        }
        Ok(())
    }

    fn link_batch(&self, source: &Key, batch: &[tasks::New]) -> Result<(), Failure> {
        for member in batch {
            self.link_executor(source, member.executor)?;
            self.link_funder(source, member.funder)?;
        }
        Ok(())
    }

    fn link_amendment(&self, source: &Key, amendment: &tasks::Amendment) -> Result<(), Failure> {
        if let Some(addressed) = amendment.addressed {
            self.link_executor(source, tasks::Executor::Person(addressed.address))?;
        }
        Ok(())
    }

    fn link_revision(&self, source: &Key, revision: u32) -> Result<(), Failure> {
        require_link(
            self.held(revision),
            source,
            Linked::Record(Key::Core(CoreKey::Settings(crate::settings::Key::Held(revision)))),
        )?;
        Ok(())
    }

    fn link_settings_proposal(
        &self,
        source: &Key,
        proposer: people::Proposer,
        number: u64,
        revision: u32,
    ) -> Result<(), Failure> {
        self.link_revision(source, revision)?;
        let Ok(number) = u32::try_from(number) else {
            return Err(Failure::Link {
                record: source.clone(),
                linked: Linked::SettingsProposal { proposer, proposal: number },
            });
        };
        let target = crate::settings::Key::Proposal { proposer, proposal: number };
        require_link(
            self.settings_proposals.contains_key(&(proposer, number)),
            source,
            Linked::Record(Key::Core(CoreKey::Settings(target))),
        )?;
        Ok(())
    }

    fn link_proposal_action(&self, source: &Key, proposal: &tasks::Proposal) -> Result<(), Failure> {
        match &proposal.action {
            tasks::ProposalAction::Batch(batch) => self.link_batch(source, batch)?,
            tasks::ProposalAction::Amend { amendment, .. } => self.link_amendment(source, amendment)?,
            tasks::ProposalAction::Settings { against, .. } => {
                self.link_settings_proposal(
                    source,
                    people::Proposer::Task(proposal.proposer),
                    proposal.number,
                    *against,
                )?;
            }
            tasks::ProposalAction::Effect { .. }
            | tasks::ProposalAction::ProcedureEffect { .. }
            | tasks::ProposalAction::Widen { .. }
            | tasks::ProposalAction::ReleasePlaced { .. }
            | tasks::ProposalAction::Release { .. } => {}
        }
        Ok(())
    }

    fn link_allocated_task(&self, source: &Key, task: u64) -> Result<(), Failure> {
        // Answer outcomes retain task identities after their live rows end (domain/engine/restart.md).
        require_link(
            task != 0 && task <= self.counters.deployment().tasks,
            source,
            Linked::Record(Key::Tasks(tasks::Key::Ended(task))),
        )?;
        Ok(())
    }

    fn link_person(&self, source: &Key, person: u64) -> Result<(), Failure> {
        require_link(
            self.people.identity(person).is_some(),
            source,
            Linked::Record(Key::People(people::Key::Person(person))),
        )?;
        Ok(())
    }

    fn link_project(&self, source: &Key, project: u32) -> Result<(), Failure> {
        if project == people::DEPLOYMENT {
            return Ok(());
        }
        require_link(self.authority.policy(project).is_some(), source, Linked::Project(project))?;
        Ok(())
    }

    fn link_task(&self, source: &Key, task: u64) -> Result<(), Failure> {
        require_link(self.tasks.task(task).is_some(), source, Linked::Record(Key::Tasks(tasks::Key::Live(task))))?;
        Ok(())
    }

    fn link_party(&self, source: &Key, party: tasks::Party) -> Result<(), Failure> {
        match party {
            tasks::Party::Person(person) => self.link_person(source, person)?,
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => {}
        }
        Ok(())
    }

    fn link_executor(&self, source: &Key, executor: tasks::Executor) -> Result<(), Failure> {
        match executor {
            tasks::Executor::Person(tasks::PersonAddress::Person(person)) => self.link_person(source, person)?,
            tasks::Executor::Agent { .. }
            | tasks::Executor::Procedure { .. }
            | tasks::Executor::Job { .. }
            | tasks::Executor::Person(tasks::PersonAddress::Role(_)) => {}
        }
        Ok(())
    }

    fn link_funder(&self, source: &Key, funder: tasks::Funder) -> Result<(), Failure> {
        match funder {
            tasks::Funder::Pool { person, .. } => self.link_person(source, person)?,
            tasks::Funder::Task(_) | tasks::Funder::Period { .. } | tasks::Funder::Recurring { .. } => {}
        }
        Ok(())
    }
}

fn require_link(present: bool, record: &Key, linked: Linked) -> Result<(), Failure> {
    if present { Ok(()) } else { Err(Failure::Link { record: record.clone(), linked }) }
}
