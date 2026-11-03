//! ctl makes the named project, then adopts its first repository (temper/docs/design/protocol/setup.md, 6.7).
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
    /// Only `ProjectMade` permits adoption. Every other outcome stays in jig's words.
    pub fn answered(&mut self, outcome: &Outcome) -> Option<super::Adoption> {
        self.outcome = Some(outcome.clone());
        if let Outcome::Answered(Answered::Jig(c::Answer::Done(c::Outcome::Document(
            c::DocumentOutcome::ProjectMade { project },
        )))) = outcome
        {
            self.adoption.project = *project;
            Some(self.adoption.clone())
        } else {
            None
        }
    }
}
