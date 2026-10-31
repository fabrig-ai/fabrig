//! Cross-child required links before sweep (jig/docs/design/domain/engine.md, 6).
use crate::{Core, CoreKey, Key, accounts, people, tasks};

impl Core {
    /// Re-derive a waiting holder through the same selection used while serving.
    pub(crate) fn route_waiting_escalation(
        &mut self,
        env: &skein_lib::Env<crate::Limits>,
        context: &tasks::EscalationContext,
        out: &mut skein_lib::Queue<tasks::Request>,
    ) -> bool {
        let role = self.people.role(context.requester, context.project);
        let Some(holder) = self.escalation_recipient(&env.limits, context, role) else { return false };
        match context.escalation {
            tasks::Escalation::Waiting { holder: old, .. } if old == holder => return true,
            tasks::Escalation::Waiting { revision: u64::MAX, .. } => return false,
            tasks::Escalation::Unheld { .. }
            | tasks::Escalation::Routing { .. }
            | tasks::Escalation::Waiting { .. }
            | tasks::Escalation::Rejected { .. } => {}
        }
        let Some(entry) = crate::fresh(&mut self.counters, crate::Family::Message) else { return false };
        tasks::step(
            &mut self.tasks,
            &skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.tasks },
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

    /// First live task or call whose required run proof is absent (engine.md, 6).
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

    /// First live task whose charter is absent from its held settings (engine.md, 6).
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

    pub(super) fn start_cross_links(&self) {
        for view in self.tasks.view_tasks() {
            self.link_task_row(self.tasks.task(view.number).expect("live view owner"));
        }
        self.link_placed_holds();
        self.link_person_proposals();
        self.link_settings_proposals();
        self.link_projection_tasks();
        self.link_resource_projects();
        self.link_people_answers();
        self.link_other_children();
    }

    fn link_task_row(&self, row: &tasks::TaskRecord) {
        let source = Key::Tasks(tasks::Key::Live(row.number));
        self.link_project(&source, row.project);
        self.link_party(&source, row.requester);
        self.link_executor(&source, row.executor);
        self.link_funder(&source, row.funder);
        match row.executor {
            tasks::Executor::Agent { charter } => {
                for revision in [Some(row.settings.task), row.settings.run].into_iter().flatten() {
                    assert!(
                        self.held(revision),
                        "{source:?} links missing {:?}",
                        Key::Core(CoreKey::Settings(crate::settings::Key::Held(revision)))
                    );
                    assert!(
                        self.charter_at(charter, revision).is_some(),
                        "{source:?} links missing charter {charter} at revision {revision}"
                    );
                }
            }
            tasks::Executor::Person(_) | tasks::Executor::Procedure { .. } => {}
        }
        if let Some(number) = row.placed {
            assert!(
                self.tasks.placed_hold(number).is_some(),
                "{source:?} links missing {:?}",
                Key::Tasks(tasks::Key::PlacedHold(number))
            );
        }
        if let Some(recurring) = &row.recurring {
            self.link_batch(&source, &recurring.template.batch);
        }
        if let Some(person) = row.taken_by {
            self.link_person(&source, person);
        }
        if let Some(answer) = row.person_answer {
            self.link_person(&source, answer.by);
        }
        if let Some(grace) = &row.grace {
            self.link_party(&source, grace.by);
        }
        for added in &row.added {
            self.link_party(&source, added.by);
        }
        for message in &row.inbox {
            self.link_party(&source, message.from);
        }
        match row.escalation {
            tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Person(person), .. }
            | tasks::Escalation::Rejected { by: person, .. } => self.link_person(&source, person),
            tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Role { project, .. }, .. } => {
                self.link_project(&source, project);
            }
            tasks::Escalation::Unheld { .. }
            | tasks::Escalation::Routing { .. }
            | tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(_), .. } => {}
        }
        if let Some(proposal) = &row.proposal {
            self.link_proposal_action(&source, proposal);
            match proposal.state {
                tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Person(person), .. } => {
                    self.link_person(&source, person);
                }
                tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Policy { project, .. }, .. } => {
                    self.link_project(&source, project);
                }
                tasks::ProposalState::Accepted { by } | tasks::ProposalState::Rejected { by, .. } => {
                    self.link_party(&source, by);
                }
                tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Task(_), .. }
                | tasks::ProposalState::Withdrawn => {}
            }
        }
        match &row.phase {
            tasks::Phase::Closing(closing) | tasks::Phase::Held { was: tasks::Was::Closing(closing), .. } => {
                match closing.ending {
                    tasks::Ending::Cancelled { by, .. } => self.link_party(&source, by),
                    tasks::Ending::Done(_) | tasks::Ending::Failed { .. } => {}
                }
            }
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Held { .. } | tasks::Phase::Ended(_) => {}
        }
    }

    fn link_placed_holds(&self) {
        let mut after = 0;
        while let Some(hold) = self.tasks.next_placed_hold(after) {
            after = hold.number;
            let source = Key::Tasks(tasks::Key::PlacedHold(hold.number));
            self.link_project(&source, hold.project);
            match hold.owner {
                tasks::Owner::Party { person } => self.link_person(&source, person),
                tasks::Owner::Task { task } => self.link_task(&source, task),
            }
        }
    }

    fn link_person_proposals(&self) {
        for (_, proposal) in self.tasks.person_proposals() {
            let source = Key::Tasks(tasks::Key::PersonProposal(proposal.number));
            self.link_person(&source, proposal.proposer);
            self.link_project(&source, proposal.project);
            match proposal.state {
                tasks::PersonProposalState::Accepted { by }
                | tasks::PersonProposalState::SettingsAccepted { by, .. }
                | tasks::PersonProposalState::Rejected { by, .. } => self.link_party(&source, by),
                tasks::PersonProposalState::Pending { .. } | tasks::PersonProposalState::Withdrawn { .. } => {}
            }
            match &proposal.action {
                tasks::PersonAction::Batch(batch) => self.link_batch(&source, batch),
                tasks::PersonAction::Amend { amendment, .. } => self.link_amendment(&source, amendment),
                tasks::PersonAction::Settings { against, .. } => {
                    self.link_settings_proposal(
                        &source,
                        people::Proposer::Party(proposal.proposer),
                        proposal.number,
                        *against,
                    );
                }
            }
        }
    }

    fn link_settings_proposals(&self) {
        for ((proposer, number), proposal) in &self.settings_proposals {
            let source =
                Key::Core(CoreKey::Settings(crate::settings::Key::Proposal { proposer: *proposer, proposal: *number }));
            self.link_revision(&source, proposal.against);
            let present = match proposer {
                people::Proposer::Party(person) => {
                    self.link_person(&source, *person);
                    match self.tasks.person_proposals().get(&u64::from(*number)) {
                        Some(row) => row.proposer == *person,
                        None => false,
                    }
                }
                people::Proposer::Task(task) => {
                    self.link_task(&source, *task);
                    match &self.tasks.task(*task).expect("linked proposer").proposal {
                        Some(row) => row.number == u64::from(*number),
                        None => false,
                    }
                }
            };
            assert!(present, "{source:?} links missing proposer {proposer:?} proposal {number}");
        }
    }

    fn link_projection_tasks(&self) {
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
                        );
                    }
                }
            }
        }
    }

    fn link_resource_projects(&self) {
        for (key, _) in &self.resource_roles {
            assert!(
                key.project == people::DEPLOYMENT || self.authority.policy(key.project).is_some(),
                "{key:?} links missing project {}",
                key.project
            );
        }
    }

    fn link_people_answers(&self) {
        let mut after = None;
        while let Some((key, outcome)) = self.people.next_answer(after) {
            after = Some(key);
            let source = Key::People(people::Key::Answer(key));
            match outcome {
                people::Outcome::Applied { connector, .. } => {
                    assert!(self.connectors.contains(connector), "{source:?} links missing connector {connector}");
                }
                people::Outcome::Stopped { task }
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
                    self.link_allocated_task(&source, *task);
                }
                people::Outcome::Created { first, last, given } => {
                    self.link_allocated_task(&source, *first);
                    self.link_allocated_task(&source, *last);
                    if let Some(given) = given {
                        self.link_allocated_task(&source, given.task);
                    }
                }
                people::Outcome::MarkedSeen { seen: people::Seen::Results { .. } }
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
                | people::Outcome::AccountChanged
                | people::Outcome::AccountRemoved { .. }
                | people::Outcome::AccountSignedOut { .. }
                | people::Outcome::Refused(_) => {}
            }
        }
    }

    fn link_other_children(&self) {
        for (number, account) in self.accounts.accounts_by_number() {
            match account.holder() {
                accounts::Holder::Party(person) => {
                    self.link_person(&Key::Accounts(accounts::Key::Account { number: *number }), person);
                }
                accounts::Holder::Deployment => {}
            }
        }
        for (number, proof) in &self.proofs {
            let source = Key::Core(CoreKey::RunProof(*number));
            self.link_task(&source, proof.task);
        }
        if let Some((record, target)) = self.missing_proof_link() {
            assert!(self.missing_proof_link().is_none(), "{record:?} links missing {target:?}");
        }
        for ((party, task), _) in self.people.read_conversations() {
            self.link_task(&Key::People(people::Key::ReadConversation { party: *party, task: *task }), *task);
        }
    }

    fn link_batch(&self, source: &Key, batch: &[tasks::New]) {
        for member in batch {
            self.link_executor(source, member.executor);
            self.link_funder(source, member.funder);
        }
    }

    fn link_amendment(&self, source: &Key, amendment: &tasks::Amendment) {
        if let Some(addressed) = amendment.addressed {
            self.link_executor(source, tasks::Executor::Person(addressed.address));
        }
    }

    fn link_revision(&self, source: &Key, revision: u32) {
        assert!(
            self.held(revision),
            "{source:?} links missing {:?}",
            Key::Core(CoreKey::Settings(crate::settings::Key::Held(revision)))
        );
    }

    fn link_settings_proposal(&self, source: &Key, proposer: people::Proposer, number: u64, revision: u32) {
        self.link_revision(source, revision);
        let target_number = u32::try_from(number);
        let linked = target_number.is_ok();
        assert!(linked, "{source:?} links missing settings proposal {proposer:?}/{number}");
        let number = target_number.expect("linked settings proposal key");
        let target = crate::settings::Key::Proposal { proposer, proposal: number };
        assert!(
            self.settings_proposals.contains_key(&(proposer, number)),
            "{source:?} links missing {:?}",
            Key::Core(CoreKey::Settings(target))
        );
    }

    fn link_proposal_action(&self, source: &Key, proposal: &tasks::Proposal) {
        match &proposal.action {
            tasks::ProposalAction::Batch(batch) => self.link_batch(source, batch),
            tasks::ProposalAction::Amend { amendment, .. } => self.link_amendment(source, amendment),
            tasks::ProposalAction::Settings { against, .. } => {
                self.link_settings_proposal(
                    source,
                    people::Proposer::Task(proposal.proposer),
                    proposal.number,
                    *against,
                );
            }
            tasks::ProposalAction::Effect { .. }
            | tasks::ProposalAction::ProcedureEffect { .. }
            | tasks::ProposalAction::Widen { .. }
            | tasks::ProposalAction::ReleasePlaced { .. }
            | tasks::ProposalAction::Release { .. } => {}
        }
    }

    fn link_allocated_task(&self, source: &Key, task: u64) {
        // Answer outcomes retain task identities after their live rows end (engine.md, 6).
        assert!(
            task != 0 && task <= self.counters.deployment().tasks,
            "{source:?} links missing {:?}",
            Key::Tasks(tasks::Key::Ended(task))
        );
    }

    fn link_person(&self, source: &Key, person: u64) {
        assert!(
            self.people.identity(person).is_some(),
            "{source:?} links missing {:?}",
            Key::People(people::Key::Person(person))
        );
    }

    fn link_project(&self, source: &Key, project: u32) {
        if project == people::DEPLOYMENT {
            return;
        }
        assert!(self.authority.policy(project).is_some(), "{source:?} links missing project {project}");
    }

    fn link_task(&self, source: &Key, task: u64) {
        assert!(self.tasks.task(task).is_some(), "{source:?} links missing {:?}", Key::Tasks(tasks::Key::Live(task)));
    }

    fn link_party(&self, source: &Key, party: tasks::Party) {
        match party {
            tasks::Party::Person(person) => self.link_person(source, person),
            tasks::Party::Task(_) | tasks::Party::Deployment { .. } => {}
        }
    }

    fn link_executor(&self, source: &Key, executor: tasks::Executor) {
        match executor {
            tasks::Executor::Person(tasks::PersonAddress::Person(person)) => self.link_person(source, person),
            tasks::Executor::Agent { .. }
            | tasks::Executor::Procedure { .. }
            | tasks::Executor::Person(tasks::PersonAddress::Role(_)) => {}
        }
    }

    fn link_funder(&self, source: &Key, funder: tasks::Funder) {
        match funder {
            tasks::Funder::Pool { person, .. } => self.link_person(source, person),
            tasks::Funder::Task(_) | tasks::Funder::Period { .. } | tasks::Funder::Recurring { .. } => {}
        }
    }
}
