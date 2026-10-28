//! The common web and ctl start: make the named project, then adopt its repository.
use alloc::boxed::Box;
use jig_client::{Answered, Ask, Outcome, contract as c};

#[derive(Debug)]
pub struct StartIn {
    pub name: Box<[u8]>,
    pub adoption: super::Adoption,
    pub outcome: Option<Outcome>,
}
impl StartIn {
    #[must_use]
    pub const fn new(adoption: super::Adoption, name: Box<[u8]>) -> Self {
        Self { name, adoption, outcome: None }
    }
    #[must_use]
    pub fn ask(&self, limits: &super::Limits) -> Option<Ask> {
        Some(Ask::Jig(Box::new(c::Ask::MakeProject {
            name: c::Text::new(self.name.clone(), limits.field_bytes).ok()?,
            policy: 1,
        })))
    }
    /// After a reload attempt zero can dedup to an earlier `ProjectMade`.
    /// Wait for its banner name; only the name asked now permits reusing it.
    #[must_use]
    pub(crate) fn same_project_name(&self, client: &jig_client::Client) -> Option<bool> {
        if self.adoption.project == 0 {
            return None;
        }
        let document = client.document(super::me::handle())?;
        let Some(jig_client::Latest::Jig(c::Delivery::Me(me))) = document.latest else { return None };
        for project in &me.projects {
            if project.project == self.adoption.project {
                return Some(project.name.bytes() == self.name.as_ref());
            }
        }
        None
    }
    /// Only `ProjectMade` permits adoption. Every other outcome stays in jig's words.
    pub fn answered(&mut self, outcome: &Outcome) -> Option<super::Adoption> {
        self.outcome = Some(outcome.clone());
        if let Outcome::Answered(Answered::Jig(c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::ProjectMade {
            project,
        })))) = outcome
        {
            self.adoption.project = *project;
            Some(self.adoption.clone())
        } else {
            None
        }
    }
}
