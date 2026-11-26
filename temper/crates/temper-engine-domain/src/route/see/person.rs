//! Handoffs for reads as the chat's requester (temper/docs/design/domain/routing.md, 1).
use super::{Address, Decision, Domain, Env, FlightRender, Limits, ReplyTo, SeeAnswer, SeeFlight, SeeReading, Work};
use crate::SeeNotRead;
use alloc::boxed::Box;
use jig_domain_core::{
    self as core,
    reads::{Answer, Read},
};
use skein_lib::Token;
use temper_engine_domain_forge as forge;
use temper_engine_domain_see::person::{self, Gathered, Part, Step, next};

#[expect(clippy::too_many_arguments, reason = "the person read carries its call and parsed address")]
pub(super) fn read(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: crate::CallKey,
    line: Box<[u8]>,
    from: u32,
    address: Address,
) {
    let context = domain.core.tasks.delegation(key.task).expect("authorized chat delegation");
    match context.requester {
        jig_domain_tasks::Party::Person(_) => {}
        jig_domain_tasks::Party::Task(_) | jig_domain_tasks::Party::Deployment { .. } => {
            super::deliver(env, decision, to, line, from, SeeAnswer::NotRead(SeeNotRead::NoPerson));
            return;
        }
    }
    let gathered = Gathered::new(domain.config.forge_connector);
    let read = match next(&address, context.project, &gathered) {
        Step::Read(read) => read,
        Step::Deliver(_) => unreachable!("first person read"),
    };
    let right = to.into_token();
    domain.call_renders.get_mut(&right).expect("decoded see flight").render = FlightRender::Settings {
        see: Some(Box::new(SeeFlight { line, reading: SeeReading::Person(Box::new(gathered)) })),
        from,
        requested: None,
        asked: None,
    };
    ask(domain, ReplyTo::new(right), key, read);
}

fn ask(domain: &mut Domain, to: ReplyTo, key: crate::CallKey, read: Read) {
    domain.work.push(Work::Core(core::Event::NamedAction {
        to,
        key,
        action: core::NamedAction::PersonRead(Box::new(read)),
    }));
}

pub(crate) fn is_reading(domain: &Domain, right: Token) -> bool {
    let Some(flight) = domain.call_renders.get(&right) else { return false };
    match &flight.render {
        FlightRender::Settings { see: Some(see), .. } => match see.reading {
            SeeReading::Person(_) => true,
            SeeReading::Settings | SeeReading::Head | SeeReading::File | SeeReading::Proposal => false,
        },
        FlightRender::Plain
        | FlightRender::ProjectRead
        | FlightRender::TaskRead
        | FlightRender::ForgeRead(_)
        | FlightRender::BranchFile { .. }
        | FlightRender::Settings { see: None, .. }
        | FlightRender::Amended(_)
        | FlightRender::Change(_)
        | FlightRender::PersonRoles(_)
        | FlightRender::Document(_)
        | FlightRender::Remember(_) => false,
    }
}

pub(crate) fn answered(domain: &mut Domain, to: ReplyTo, answer: Answer) {
    answered_part(domain, to, Part::Jig(answer));
}

pub(crate) fn document(domain: &mut Domain, to: ReplyTo, document: Option<forge::Document>) {
    let document = match document {
        Some(document) => Some(Box::new(document)),
        None => None,
    };
    answered_part(domain, to, Part::Forge(document));
}

fn flight_context(render: &mut FlightRender) -> (&mut SeeFlight, u32) {
    match render {
        FlightRender::Settings { see: Some(see), from, .. } => (see, *from),
        FlightRender::Plain
        | FlightRender::ProjectRead
        | FlightRender::TaskRead
        | FlightRender::ForgeRead(_)
        | FlightRender::BranchFile { .. }
        | FlightRender::Settings { see: None, .. }
        | FlightRender::Amended(_)
        | FlightRender::Change(_)
        | FlightRender::PersonRoles(_)
        | FlightRender::Document(_)
        | FlightRender::Remember(_) => unreachable!("see person context"),
    }
}

fn answered_part(domain: &mut Domain, to: ReplyTo, answer: Part) {
    let right = to.into_token();
    let Some(flight) = domain.call_renders.get_mut(&right) else { return };
    let key = flight.key.expect("decoded person read");
    let context = domain.core.tasks.delegation(key.task).expect("person read chat");
    let project = context.project;
    let person = match context.requester {
        jig_domain_tasks::Party::Person(person) => person,
        jig_domain_tasks::Party::Task(_) | jig_domain_tasks::Party::Deployment { .. } => {
            unreachable!("person read requester")
        }
    };
    let (see, _) = flight_context(&mut flight.render);
    let address = super::address(&see.line, project);
    let gathered = match &mut see.reading {
        SeeReading::Person(gathered) => gathered.as_mut(),
        SeeReading::Settings | SeeReading::Head | SeeReading::File | SeeReading::Proposal => {
            unreachable!("see person flight")
        }
    };
    gathered.push(&address, answer);
    let answer = match next(&address, project, gathered) {
        Step::Read(read) => {
            ask(domain, ReplyTo::new(right), key, read);
            return;
        }
        Step::Deliver(Some(why)) => SeeAnswer::NotRead(why),
        Step::Deliver(None) => {
            let gathered = ::core::mem::replace(gathered, Gathered::new(domain.config.forge_connector));
            SeeAnswer::Person(Box::new(person::answer(&domain.core, &domain.forge, address, person, project, gathered)))
        }
    };
    let flight = domain.call_renders.get_mut(&right).expect("see person flight");
    let (see, from) = flight_context(&mut flight.render);
    let request = crate::boundary::delivery_output(super::Delivery::Host(Box::new(crate::HostDelivery::See {
        to: ReplyTo::new(right),
        line: see.line.clone(),
        from,
        answer: Box::new(answer),
    })));
    assert!(crate::domain::now(domain, request), "see person render door reserved");
}
