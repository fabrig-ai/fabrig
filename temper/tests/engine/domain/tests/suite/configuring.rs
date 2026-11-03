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

#[test]
fn a_members_proposal_waits_for_an_owner() {
    let mut world = World::new(member_config());
    let member = world.member();
    let run = world.chat(CONFIGURING, member, 20);
    let value = read_charter(&mut world.driver, &run, 1, CHAT);
    let items = charter_item(value, b"Push early and report the result.");
    let proposal = propose(&mut world.driver, &run, 2, 1, &items);
    assert_eq!(world.head().number, 1);
    let review = world.review(&run, proposal);
    assert_eq!(
        review.waiting.holder,
        tasks::ProposalHolder::Policy { project: people::DEPLOYMENT, kind: tasks::ProposalKind::Settings }
    );
    world.driver.send(Event::Party(PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(91)),
        sign_in: world.owner,
        read: core::reads::Read::Inbox {
            group: core::reads::Group::Proposals,
            filter: core::reads::Filter { project: None, goal: None },
            from: None,
        },
    }));
    world.driver.settle();
    let inbox = world
        .driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let Delivery::Read { answer: core::reads::Answer::Page(page), .. } = delivery {
                Some(page)
            } else {
                None
            }
        })
        .expect("owner's proposal inbox");
    assert!(inbox.rows.iter().any(|row| matches!(row,
        core::reads::PageRow::Inbox(entry) if entry.object == core::documents::InboxObject::Proposal {
            proposer: people::Proposer::Task(run.task), number: proposal
        }
    )));
    world.ask(
        member,
        21,
        people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Task(run.task),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    );
    assert!(matches!(world.reply(21), people::Reply::Outcome(people::Outcome::Refused(_))));
    assert_eq!(world.head().number, 1, "a project role cannot accept the deployment's settings");
    assert_eq!(world.review(&run, proposal).waiting, review.waiting);
    world.decide(&run, proposal, 22, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    assert_heard(&world.next(&run), b"accepted: revision 2 made");
}

#[test]
fn a_stale_proposal_is_withdrawn_read_again_and_proposed_afresh() {
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let value = read_charter(&mut world.driver, &run, 1, CHAT);
    let items = charter_item(value, b"Push early.");
    let stale = propose(&mut world.driver, &run, 2, 1, &items);
    world.change(1, 21, Box::new([owner_charter(b"Keep commits small.")]));
    let next = world.next(&run);
    assert_heard(&next, b"withdrawn as stale: the newest is 2");
    let (inventory, error) = host_tool(&mut world.driver, &next, 1, b"settings", br"{}");
    assert!(!error);
    assert!(inventory.starts_with(b"newest revision 2\n"));
    let value = read_charter(&mut world.driver, &next, 2, CHAT);
    assert_eq!(value["instructions"], "Keep commits small.");
    let items = charter_item(value, b"Keep commits small. Push early.");
    let fresh = propose(&mut world.driver, &next, 3, 2, &items);
    assert_ne!(fresh, stale);
    world.decide(&next, fresh, 22, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 3);
    assert_heard(&world.next(&next), b"accepted: revision 3 made");
}

#[test]
fn a_rejected_proposal_is_revised() {
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let value = read_charter(&mut world.driver, &run, 1, CHAT);
    let items = charter_item(value, b"Push every edit.");
    let first = propose(&mut world.driver, &run, 2, 1, &items);
    world.decide(
        &run,
        first,
        21,
        people::ProposalDecision::Reject { reason: b"Push each complete change instead.".as_slice().into() },
        people::Running::Stays,
    );
    assert_eq!(world.head().number, 1, "rejecting changes no settings");
    let next = world.next(&run);
    assert_heard(&next, b"rejected: Push each complete change instead.");
    let value = read_charter(&mut world.driver, &next, 1, CHAT);
    let items = charter_item(value, b"Push each complete change.");
    let revised = propose(&mut world.driver, &next, 2, 1, &items);
    assert_ne!(first, revised);
    let review = world.review(&next, revised);
    let core::documents::ProposalAction::Settings { review: Some(review), .. } = review.action else {
        panic!("settings proposal review")
    };
    assert!(review.items.iter().any(|item| matches!(&item.would_be,
        Some(s::Value::Charter(charter)) if charter.policy.instructions.as_ref() == b"Push each complete change."
    )));
    world.decide(&next, revised, 22, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    assert_heard(&world.next(&next), b"accepted: revision 2 made");
}

#[test]
fn an_accepted_proposal_moves_running_work() {
    let mut world = World::new(moving_config());
    let chat = world.chat(CHAT, world.owner, 20);
    let removed = world.chat(REMOVED, world.owner, 21);
    world.park(&chat);
    world.park(&removed);
    let run = world.chat(CONFIGURING, world.owner, 22);
    let value = read_charter(&mut world.driver, &run, 1, CHAT);
    let mut items = charter_item(value, b"Push each complete change.");
    let mut rules = read_value(&mut world.driver, &run, 10, serde_json::json!({"setting":{"kind":"rules"}}));
    omit_removable(&mut rules["ceiling"]);
    let mut policy =
        read_value(&mut world.driver, &run, 30, serde_json::json!({"setting":{"kind":"policy","number":1}}));
    omit_removable(&mut policy["ceiling"]);
    for role in policy["roles"].as_array_mut().expect("policy roles") {
        omit_removable(&mut role["authority"]);
    }
    // Removing a charter also removes its references, in the same proposal.
    items.as_array_mut().expect("items").extend([
        serde_json::json!({"setting":{"kind":"charter","number":REMOVED},"edit":"removed"}),
        serde_json::json!({"setting":{"kind":"template","number":REMOVED},"edit":"removed"}),
        serde_json::json!({"setting":{"kind":"rules"},"set":rules}),
        serde_json::json!({"setting":{"kind":"policy","number":1},"set":policy}),
        serde_json::json!({"setting":{"kind":"project-policy","project":PROJECT},"set":policy}),
    ]);
    let proposal = propose(&mut world.driver, &run, 50, 1, &items);
    world.decide(&run, proposal, 23, people::ProposalDecision::Accept, people::Running::Moves);
    assert_eq!(world.head().number, 2);
    assert!(world.head().moved);
    assert_eq!(
        world.driver.task(chat.task).expect("stock chat").settings,
        tasks::SettingsRevision { task: 2, run: None }
    );
    assert_eq!(
        world.driver.task(removed.task).expect("removed charter's task").settings,
        tasks::SettingsRevision { task: 1, run: None }
    );
    assert_eq!(
        world.driver.task(run.task).expect("configuring task").settings,
        tasks::SettingsRevision { task: 2, run: Some(1) },
        "the live run keeps its prepared revision"
    );
    let next = world.wake(&chat, 24);
    assert_eq!(next.run.revision, 2);
    assert_eq!(next.run.policy.instructions.as_ref(), b"Push each complete change.");
    let stayed = world.wake(&removed, 25);
    assert_eq!(stayed.run.revision, 1);
    assert_eq!(stayed.run.policy.instructions, removed.run.policy.instructions);
}
