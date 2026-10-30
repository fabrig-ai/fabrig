//! Cross-child required links before sweep (jig/docs/design/domain/engine.md, 6).
use crate::{Core, CoreKey, Key, accounts, people, tasks};

impl Core {
    pub(super) fn start_cross_links(&self) {
        for view in self.tasks.view_tasks() {
            let row = self.tasks.task(view.number).expect("live view owner");
            let source = Key::Tasks(tasks::Key::Live(row.number));
            self.link_project(&source, row.project);
            self.link_party(&source, row.requester);
            self.link_executor(&source, row.executor);
            self.link_funder(&source, row.funder);
            match row.executor {
                tasks::Executor::Agent { charter } => {
                    for revision in [Some(row.settings.task), row.settings.run].into_iter().flatten() {
                        assert!(self.held(revision), "{source:?} links missing {:?}", Key::Core(CoreKey::Settings(crate::settings::Key::Held(revision))));
                        assert!(self.charter_at(charter, revision).is_some(), "{source:?} links missing charter {charter} at revision {revision}");
                    }
                }
                tasks::Executor::Person(_) | tasks::Executor::Procedure { .. } => {}
            }
            if let Some(person) = row.taken_by { self.link_person(&source, person); }
            if let Some(answer) = row.person_answer { self.link_person(&source, answer.by); }
            if let Some(grace) = &row.grace { self.link_party(&source, grace.by); }
            for added in &row.added { self.link_party(&source, added.by); }
            for message in &row.inbox { self.link_party(&source, message.from); }
            match row.escalation {
                tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Person(person), .. }
                | tasks::Escalation::Rejected { by: person, .. } => self.link_person(&source, person),
                tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Role { project, .. }, .. } => self.link_project(&source, project),
                tasks::Escalation::Unheld { .. } | tasks::Escalation::Routing { .. }
                | tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(_), .. } => {}
            }
            if let Some(proposal) = &row.proposal {
                match proposal.state {
                    tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Person(person), .. } => self.link_person(&source, person),
                    tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Policy { project, .. }, .. } => self.link_project(&source, project),
                    tasks::ProposalState::Accepted { by } | tasks::ProposalState::Rejected { by, .. } => self.link_party(&source, by),
                    tasks::ProposalState::Pending { holder: tasks::ProposalHolder::Task(_), .. } | tasks::ProposalState::Withdrawn => {}
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
        for (number, account) in self.accounts.accounts_by_number() {
            match account.holder() {
                accounts::Holder::Party(person) => self.link_person(&Key::Accounts(accounts::Key::Account { number: *number }), person),
                accounts::Holder::Deployment => {}
            }
        }
        for (number, proof) in &self.proofs {
            let source = Key::Core(CoreKey::RunProof(*number));
            self.link_task(&source, proof.task);
        }
        for (task, _) in &self.restoring_proofs {
            assert!(self.proofs.contains_key(task), "{:?} links missing {:?}", Key::Tasks(tasks::Key::Live(*task)), Key::Core(CoreKey::RunProof(*task)));
        }
        for (key, _) in &self.call_parts {
            let source = Key::Core(CoreKey::Call(*key));
            assert!(self.proofs.contains_key(&key.task), "{source:?} links missing {:?}", Key::Core(CoreKey::RunProof(key.task)));
        }
        for ((party, task), _) in self.people.read_conversations() {
            self.link_task(&Key::People(people::Key::ReadConversation { party: *party, task: *task }), *task);
        }
    }

    fn link_person(&self, source: &Key, person: u64) {
        assert!(self.people.identity(person).is_some(), "{source:?} links missing {:?}", Key::People(people::Key::Person(person)));
    }

    fn link_project(&self, source: &Key, project: u32) {
        if project == people::DEPLOYMENT { return; }
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
            tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. }
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

