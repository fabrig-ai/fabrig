//! Application reads (jig/docs/design/domain/hosts/hub.md, 10). No retry or clock.
use crate::{AskAnswer, Hub, Limits, LinkUp, Request};
use alloc::boxed::Box;
use skein_lib::{Env, Queue, Set, Token, length::len_u64};

#[derive(Debug)]
pub(crate) struct Pending {
    last: u64,
    numbers: Set<u64>,
}
impl Pending {
    pub(crate) fn new() -> Self {
        Self { last: 0, numbers: Set::with_capacity(2) }
    }
    pub(crate) fn clear(&mut self) {
        while self.numbers.pop_first().is_some() {}
    }
}

fn answer(run: Token, attempt: Token, number: u64, answer: AskAnswer, out: &mut Queue<Request>) {
    out.push(Request::Asked { run, attempt, number, answer });
}

pub(crate) fn ask(
    domain: &mut Hub,
    env: &Env<Limits>,
    run: Token,
    attempt: Token,
    bytes: Box<[u8]>,
    out: &mut Queue<Request>,
) {
    let Some(&id) = domain.names.get(&run) else {
        answer(run, attempt, 0, AskAnswer::Refused, out);
        return;
    };
    let entry = domain.hosted.get_mut(id).expect("named attempt");
    if entry.attempt != attempt {
        answer(run, attempt, 0, AskAnswer::Refused, out);
        return;
    }
    let Some(number) = entry.asks.last.checked_add(1) else {
        answer(run, attempt, 0, AskAnswer::Refused, out);
        return;
    };
    entry.asks.last = number;
    if entry.asks.numbers.len() >= 2 || len_u64(bytes.len()) > u64::from(env.limits.application_ask_bytes) {
        answer(run, attempt, number, AskAnswer::Refused, out);
        return;
    }
    entry.asks.numbers.insert(number).expect("two outstanding asks");
    out.push(Request::Up(LinkUp::Ask { run, attempt, number, bytes }));
}

pub(crate) fn asked(
    domain: &mut Hub,
    env: &Env<Limits>,
    run: Token,
    attempt: Token,
    number: u64,
    mut value: AskAnswer,
    out: &mut Queue<Request>,
) {
    let Some(&id) = domain.names.get(&run) else { return };
    let entry = domain.hosted.get_mut(id).expect("named attempt");
    if entry.attempt != attempt || !entry.asks.numbers.remove(&number) {
        return;
    }
    match &value {
        AskAnswer::Bytes(bytes) if len_u64(bytes.len()) > u64::from(env.limits.application_answer_bytes) => {
            value = AskAnswer::Refused;
        }
        AskAnswer::Bytes(_) | AskAnswer::Refused => {}
    }
    answer(run, attempt, number, value, out);
}

pub(crate) fn lost(domain: &mut Hub, out: &mut Queue<Request>) {
    for (_, id) in &domain.names {
        let entry = domain.hosted.get_mut(*id).expect("hosted attempt");
        while let Some(number) = entry.asks.numbers.pop_first() {
            answer(entry.run, entry.attempt, number, AskAnswer::Refused, out);
        }
    }
}
