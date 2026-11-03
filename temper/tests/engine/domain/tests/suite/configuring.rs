//! Scripted stock configuring chats through the root and jig's store
//! (temper/docs/design/domain/testing.md, 6.4; temper/docs/design/domain/agent.md, 8.1).
mod fixture;
use fixture::*;
use jig_domain_core::{self as core, settings as s};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Event, PartyInput};
use temper_engine_domain_world::host_tool::host_tool;

#[test]
fn an_owners_configuring_chat_proposes_and_hears_it_accepted() {
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    assert_eq!(run.charter, CONFIGURING);
    assert_eq!(
        run.run.policy.instructions.as_ref(),
        include_bytes!("../../../../../crates/temper-service/src/stock/configuring.txt")
    );
    let offered = temper_engine_smith::charter(&run);
    for name in [b"settings".as_slice(), b"propose_settings"] {
        assert!(offered.grants.host_tools.iter().any(|tool| tool.name.as_ref() == name));
    }
    let (inventory, error) = host_tool(&mut world.driver, &run, 1, b"settings", br"{}");
    assert!(!error);
    assert!(inventory.starts_with(b"newest revision 1\n"));
    let value = read_charter(&mut world.driver, &run, 2, CHAT);
    let items = charter_item(value, b"Push early and report the result.");
    let proposal = propose(&mut world.driver, &run, 3, 1, &items);
    let review = world.review(&run, proposal);
    assert_eq!(review.reason.bytes.as_ref(), REASON.as_bytes());
    assert_eq!(world.head().number, 1, "a proposal alone changes no settings");
    world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    assert!(!world.head().moved);
    assert_eq!(
        world.head().proposal,
        Some((people::Proposer::Task(run.task), u32::try_from(proposal).expect("proposal number")))
    );
    let next = world.next(&run);
    assert_heard(&next, b"accepted: revision 2 made");
}
