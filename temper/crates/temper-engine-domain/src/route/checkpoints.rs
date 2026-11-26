//! The owner's act waits for the store's terminal (temper/docs/design/protocol/checkpoints.md, 3).
use super::{Domain, applications};
use crate::{CheckpointWhy, Checkpointed};
use jig_domain_people as people;
use skein_lib::{Map, Token};

type Answer = Result<Checkpointed, CheckpointWhy>;

#[derive(Hash, Debug)]
pub(crate) struct State {
    pending: Map<Token, Token>,
    answers: Map<Token, Answer>,
}
impl State {
    pub(crate) fn new(bound: u32) -> Self {
        Self { pending: Map::with_capacity(bound), answers: Map::with_capacity(bound) }
    }
    pub(crate) fn answer(&self, token: Token) -> Option<Answer> {
        self.answers.get(&token).cloned()
    }
}

pub(super) fn admitted(domain: &mut Domain, request: Token, token: Token, person: u64) {
    domain.checkpoints.pending.insert(token, request).expect("one checkpoint per admitted key");
    if domain.engine.core().people.role(person, people::DEPLOYMENT) != Some(people::Role::Owner) {
        terminal(domain, token, Err(CheckpointWhy::NotOwner));
        return;
    }
    assert!(
        super::now(domain, crate::Request::Store(crate::StoreRequest::Checkpoint { token })),
        "checkpoint output door reserved"
    );
}

pub(super) fn terminal(domain: &mut Domain, token: Token, answer: Answer) {
    let Some(request) = domain.checkpoints.pending.remove(&token) else { return };
    if domain.checkpoints.answers.len() == domain.checkpoints.answers.capacity() {
        let oldest = *domain.checkpoints.answers.first().expect("full answers").0;
        domain.checkpoints.answers.remove(&oldest);
    }
    domain.checkpoints.answers.insert(token, answer).expect("bounded checkpoint answers");
    applications::finished(domain, request, Ok(token));
}
