//! Advertised host capabilities and the reason a hello was refused.
use super::{
    Builder, HelloRefusal, HelloRefusalLine, Hole, Key, Owner, P, Words, duration, field_number, region, text,
};
pub(super) fn hello(b: &mut Builder, words: &Words, value: &HelloRefusalLine, position: u64) {
    region(
        b,
        words,
        P::Host,
        &[Hole::Number(value.channel)],
        Some(Key { owner: Owner::Jig.opaque(), kind: 97, a: position, b: value.channel }),
    );
    field_number(b, words, P::Host, value.channel);
    field_number(b, words, P::Slots, u64::from(value.slots));
    text(b, words, P::At, &[Hole::Wall(words.adjusted_wall(value.at))]);
    duration(b, words, P::StopBound, value.stop_bound);
    refusal(b, words, value.why);
    b.close();
}
pub(crate) fn refusal(b: &mut Builder, words: &Words, why: HelloRefusal) {
    match why {
        HelloRefusal::Agent { why } => agent(b, words, why),
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
        S::Turns => A::Turns,
        S::Spend => A::Spend,
        S::Time => A::Time,
        S::Waiting => A::Waiting,
        S::Checks => A::Checks,
        S::Message => A::Message,
        S::Inbox => A::Inbox,
        S::Unacknowledged => A::Unacknowledged,
    };
    match why {
        S::Model { charter, model } | S::Window { charter, model } | S::Output { charter, model } => {
            text(b, words, P::Agent(phrase), &[Hole::Number(u64::from(charter))]);
            text(b, words, P::Agent(A::ModelPosition), &[Hole::Number(u64::from(model))]);
        }
        S::Turns | S::Spend | S::Time | S::Waiting | S::Checks | S::Message | S::Inbox | S::Unacknowledged => {
            text(b, words, P::Agent(phrase), &[]);
        }
    }
}
