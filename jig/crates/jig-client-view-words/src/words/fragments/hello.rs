//! The hello refusal a `Words` method draws beyond its own text
//! (jig/docs/design/client/domain/view-crates.md, 3.1).
use crate::words::Words;
use crate::words::deployment_words::DeploymentPhrase as P;
use jig_client::contract::HelloRefusal;
use skein_lib::Duration;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Run};

pub fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    let saying = words.book().say(words.language(), phrase.id(), holes).expect("deployment sayings checked");
    b.text(Run::Plain, &saying.text);
}

pub fn field_number(b: &mut Builder, words: &Words, phrase: P, value: u64) {
    text(b, words, phrase, &[Hole::Number(value)]);
}

pub fn duration(b: &mut Builder, words: &Words, phrase: P, value: Duration) {
    text(b, words, phrase, &[Hole::Span(value)]);
}

pub fn refusal(b: &mut Builder, words: &Words, why: HelloRefusal) {
    match why {
        HelloRefusal::Agent { why } => agent(b, words, why),
        HelloRefusal::JobKinds => text(b, words, P::HelloRefusalJobKinds, &[]),
        HelloRefusal::ReservedChannel => text(b, words, P::HelloRefusalReservedChannel, &[]),
        HelloRefusal::Workers { limit } => {
            text(b, words, P::HelloRefusalWorkers, &[]);
            field_number(b, words, P::Limit, u64::from(limit));
        }
        HelloRefusal::Position { worker } => {
            text(b, words, P::HelloRefusalPosition, &[Hole::Number(u64::from(worker))]);
        }
        HelloRefusal::StopBound { grace } => {
            text(b, words, P::HelloRefusalStopBound, &[]);
            duration(b, words, P::AllowedGrace, grace);
        }
    }
}

fn agent(b: &mut Builder, words: &Words, why: jig_client::contract::AgentShortfall) {
    use crate::words::deployment_words::AgentPhrase as A;
    use jig_client::contract::AgentShortfall as S;
    let phrase = match why {
        S::Model { .. } => A::Model,
        S::Window { .. } => A::Window,
        S::Output { .. } => A::Output,
        S::Turns { .. } => A::Turns,
        S::Spend => A::Spend,
        S::Time { .. } => A::Time,
        S::Waiting { .. } => A::Waiting,
        S::Checks { .. } => A::Checks,
        S::Message => A::Message,
        S::Inbox => A::Inbox,
        S::Unacknowledged => A::Unacknowledged,
    };
    match why {
        S::Checks { charter } => text(b, words, P::Agent(phrase), &[Hole::Number(u64::from(charter))]),
        S::Model { charter, model } | S::Window { charter, model } | S::Output { charter, model } => {
            text(b, words, P::Agent(phrase), &[Hole::Number(u64::from(charter))]);
            text(b, words, P::Agent(A::ModelPosition), &[Hole::Number(u64::from(model))]);
        }
        S::Turns { .. } | S::Spend | S::Time { .. } | S::Waiting { .. } | S::Message | S::Inbox | S::Unacknowledged => {
            text(b, words, P::Agent(phrase), &[]);
        }
    }
}
