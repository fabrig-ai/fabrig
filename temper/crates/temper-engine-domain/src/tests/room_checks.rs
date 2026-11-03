//! Charter values are checked before reaching the core
//! (temper/docs/design/domain/agent.md, 4.3).
use super::resume_fixture;
use crate::{Delivery, Domain, Event, Limits, PartyInput, PartyRequest, Request};
use alloc::boxed::Box;
use jig_domain_core::settings::{self as settings, Edit, Item, Setting, Value};
use jig_domain_people as people;
use skein_lib::{Env, Queue, ReplyTo, Token};

// A synthetic host part isolates the admission boundary from smith's measured table.
const HOST: crate::room::HostPart = crate::room::HostPart {
    bound: 3_048,
    fixed: 0,
    declared: 0,
    settings: 0,
    inspect: 0,
    modify: 0,
    shell: 0,
    agents: 0,
};
const CHARTER: u32 = 1;
const REPLY: Token = Token::new(9);
// 3,048 - (999 instruction bytes + two line-ending bytes) = 2,047.
const SHORT_INSTRUCTIONS: u32 = 999;
const WORDS: &[u8] = b"charter 1 leaves its brief 2,047 bytes of its host part's 3,048; it needs at least 2,048";

fn root(host: Option<crate::room::HostPart>) -> (Domain, Env<Limits>) {
    let mut config = resume_fixture::config(7);
    config.host_part = host;
    resume_fixture::root_with_config(config)
}
fn items() -> Box<[Item]> {
    let mut charter = resume_fixture::charter();
    charter.policy.instructions = super::bytes(SHORT_INSTRUCTIONS);
    Box::new([
        Item { setting: Setting::Rules, edit: Edit::Reset },
        Item { setting: Setting::Charter(CHARTER), edit: Edit::Set(Value::Charter(Box::new(charter))) },
    ])
}
fn change() -> PartyInput {
    PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(REPLY),
        sign_in: 1,
        key: [1; 16],
        digest: [2; 32],
        kinds: Box::new([]),
        change: Box::new(settings::Change { against: 1, items: items(), running: settings::Running::Stays }),
    }
}
fn proposal() -> PartyInput {
    PartyInput::ProposeSettings {
        reply_to: ReplyTo::new(REPLY),
        sign_in: 1,
        key: [1; 16],
        digest: [2; 32],
        kinds: Box::new([]),
        change: Box::new(settings::Proposed { against: 1, items: items(), reason: b"a charter".as_slice().into() }),
    }
}
fn refused(input: PartyInput) -> people::Refusal {
    let (mut root, env) = root(Some(HOST));
    crate::try_step(&mut root, &env, Event::Party(input)).unwrap();
    let mut out = Queue::uncapped();
    crate::release(&mut root, &env, &mut out);
    crate::release(&mut root, &env, &mut out);
    let Some(Request::Party(PartyRequest::Deliver(Delivery::WebReply { to, reply: people::Reply::Refused(why) }))) =
        out.pop()
    else {
        panic!("charter refused at the settings entrance");
    };
    assert_eq!(to.into_token(), REPLY);
    assert!(out.is_empty());
    why
}
fn judged() -> people::Refusal {
    people::Refusal::Judged { at: 1, setting: Setting::Charter(CHARTER), why: WORDS.into() }
}

#[test]
fn a_partys_change_refuses_the_short_charter_item() {
    assert_eq!(refused(change()), judged());
}

#[test]
fn a_partys_proposal_refuses_the_short_charter_item() {
    assert_eq!(refused(proposal()), judged());
}

#[test]
fn a_reset_carries_its_previously_admitted_value() {
    let reset = [Item { setting: Setting::Charter(CHARTER), edit: Edit::Reset }];
    assert_eq!(crate::route::room_refusal(Some(HOST), &reset), None);
}

#[test]
fn a_root_without_an_agent_checks_no_room() {
    let (root, _) = root(None);
    assert_eq!(crate::route::room_refusal(root.host_part, &items()), None);
}
