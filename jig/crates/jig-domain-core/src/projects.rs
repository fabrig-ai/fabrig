//! Project names and configured-policy creation (people.md, 5.1–5.3; core.md, 6.1).
use crate::{Core, Family, Limits, acts, fresh, policy};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use skein_lib::Queue;

pub(crate) struct Made {
    pub(crate) project: u32,
    pub(crate) name: Box<str>,
    pub(crate) policy: people::PolicyValue,
}

impl Core {
    /// A deletion row closes the project immediately (playground-reset contracts, J3).
    #[must_use]
    pub fn deleting(&self, project: u32) -> bool {
        self.people.deletion(project).is_some()
    }

    pub(crate) fn visible_role(&self, party: u64, project: u32) -> Option<people::Role> {
        if self.deleting(project) { None } else { self.people.role(party, project) }
    }

    pub(crate) fn deleting_task(&self, task: u64) -> bool {
        match self.tasks.task(task) {
            Some(row) => self.deleting(row.project),
            None => match self.archived_chats.get(&task) {
                Some(row) => self.deleting(row.project),
                None => false,
            },
        }
    }

    pub(crate) fn deleting_subject(&self, subject: &crate::documents::Subject) -> bool {
        use crate::documents::{Proposer, Subject};
        match subject {
            Subject::Application { scope, .. } => match scope {
                people::ApplicationScope::Project(project) => self.deleting(*project),
                people::ApplicationScope::Own | people::ApplicationScope::Deployment => false,
            },
            Subject::Project { project } => self.deleting(*project),
            Subject::Task { task }
            | Subject::Tree { task }
            | Subject::Run { task }
            | Subject::Escalation { task }
            | Subject::Question { task, .. }
            | Subject::PersonTask { task }
            | Subject::Result { task }
            | Subject::Proposal { proposer: Proposer::Task(task), .. } => self.deleting_task(*task),
            Subject::Proposal { proposer: Proposer::Party(party), number } => {
                match self.tasks.person_proposal(*party, *number) {
                    Some(row) => self.deleting(row.project),
                    None => false,
                }
            }
            Subject::Query { query } => match query.project {
                Some(project) => self.deleting(project),
                None => false,
            },
            Subject::Chips { tasks } => {
                for task in tasks {
                    if self.deleting_task(*task) {
                        return true;
                    }
                }
                false
            }
            Subject::Frame { .. }
            | Subject::Inbox { .. }
            | Subject::Accounts { .. }
            | Subject::Settings
            | Subject::Deployment
            | Subject::Note { .. } => false,
        }
    }

    pub(crate) fn rename(
        &mut self,
        limits: &Limits,
        person: u64,
        project: u32,
        name: Box<str>,
    ) -> Result<(), people::Refusal> {
        self.role_allowed_for(person, project, limits.tasks.tree_tasks, acts::Act::Rename)?;
        name_shape(&name)?;
        for (other, held) in &self.names {
            if *other != project && people::same_name(held, &name) {
                return Err(people::Refusal::NameTaken);
            }
        }
        self.names.insert(project, name).expect("existing project's name");
        Ok(())
    }

    pub(crate) fn make_project(
        &mut self,
        limits: &Limits,
        maker: u64,
        name: Box<str>,
        number: u32,
    ) -> Result<Made, people::Refusal> {
        let held = self.people.role(maker, people::DEPLOYMENT);
        if held != Some(people::Role::Owner) {
            return Err(people::Refusal::Role { held });
        }
        name_shape(&name)?;
        let configured = self.current.policy(number).ok_or(people::Refusal::Unknown)?;
        if self.projects.len() >= limits.authority.projects
            || !self.people.project_room()
            || limits.people.holdings == 0
            || self.counters.deployment.projects >= u64::from(u32::MAX)
        {
            return Err(people::Refusal::Limit { which: people::LimitKind::Projects });
        }
        for (_, held) in &self.names {
            if people::same_name(held, &name) {
                return Err(people::Refusal::NameTaken);
            }
        }
        let value = policy::snapshot(&configured.policy, &configured.permissions, &configured.templates)
            .expect("checked configured policy snapshot");
        let policy = configured.policy.clone();
        let permissions = configured.permissions.clone();
        let templates = configured.templates.clone();
        let project = u32::try_from(fresh(&mut self.counters, Family::Project).expect("project room checked"))
            .expect("project number room checked");
        let mut facts = Queue::uncapped();
        authority::step(&mut self.authority, authority::Event::Policy { project, policy }, &mut facts);
        assert!(facts.pop() == Some(authority::PolicyFact::Added { project }), "checked configured policy admission");
        self.projects.push(project).expect("project room checked");
        self.names.insert(project, name.clone()).expect("project name room checked");
        self.permission_roles.insert(project, permissions).expect("project permissions room checked");
        self.template_overrides.insert(project, templates).expect("project templates room checked");
        Ok(Made { project, name, policy: value })
    }
}

fn name_shape(name: &str) -> Result<(), people::Refusal> {
    match people::name_rule(name) {
        Some(rule) => Err(people::Refusal::NotWellFormed { at: 0, why: people::Malformed::Name(rule) }),
        None => Ok(()),
    }
}
