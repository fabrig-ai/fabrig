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
        world.driver.task(run.task).expect("configuring task").settings,
        tasks::SettingsRevision { task: 1, run: Some(1) },
        "accepting with running work staying keeps the current task and run on revision 1"
    );
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

#[test]
fn the_configuring_agent_reads_each_kind() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let held = read_charter(&mut world.driver, &run, 1, CHAT);
    world.change(1, 21, Box::new([owner_charter(b"The newest chat instructions.")]));
    let before = world.driver.store.rows.clone();
    let mut completion = 10;
    for (input, heading, content) in [
        (json!({}), "newest revision 2\n", "held revisions:\n  revision 1,"),
        (
            json!({"setting":{"kind":"charter","number":CHAT}}),
            "charter 1, revision 2\n",
            "The newest chat instructions.",
        ),
        (json!({"setting":{"kind":"template","number":1}}), "template 1, revision 2\n", "\"bytes\":\"chat\""),
        (json!({"setting":{"kind":"policy","number":1}}), "policy 1, revision 2\n", "\"roles\":"),
        (json!({"setting":{"kind":"rules"}}), "rules, revision 2\n", "\"ceiling\":"),
        (json!({"setting":{"kind":"landing-rule","number":1}}), "landing-rule 1, revision 2\n", "\"enforce\":true"),
        (json!({"setting":{"kind":"forge-charter","number":1}}), "forge-charter 1, revision 2\n", "default, "),
        (json!({"revision":2}), "revision 2, against revision 1,", "The newest chat instructions."),
        (json!({"history":{}}), "revision 2, against revision 1,", "revision 1, against revision 0,"),
    ] {
        let (text, pages) = read_whole(&mut world.driver, &run, completion, input);
        assert!(text.starts_with(heading), "{text}");
        assert!(text.contains(content), "{text}");
        completion += pages;
    }
    let older = read_value(
        &mut world.driver,
        &run,
        completion,
        json!({"setting":{"kind":"charter","number":CHAT},"revision":1}),
    );
    assert_eq!(older, held, "a live run holds its older charter revision");
    let (text, pages) = read_whole(
        &mut world.driver,
        &run,
        completion + 10,
        json!({"setting":{"kind":"charter","number":CONFIGURING}}),
    );
    assert!(pages > 1, "the stock configuring instructions require continuation from an offset");
    let (_, value) = text.split_once('\n').expect("charter heading");
    let value: serde_json::Value = serde_json::from_str(value).expect("reassembled charter JSON");
    assert_eq!(value["instructions"], std::str::from_utf8(&run.run.policy.instructions).expect("stock instructions"));
    assert_eq!(world.driver.store.rows, before, "settings reads commit nothing");
}

#[test]
fn propose_settings_refuses_a_stale_revision_and_an_item_out_of_shape() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let charter = read_charter(&mut world.driver, &run, 1, CHAT);
    world.change(1, 21, Box::new([owner_charter(b"The owner's instructions.")]));
    let before = world.settings_rows();
    let valid = charter_item(charter.clone(), b"A proposed change.");
    let input = json!({"against":1,"reason":REASON,"items":valid});
    let (body, error) = host_tool(&mut world.driver, &run, 10, b"propose_settings", input.to_string().as_bytes());
    assert!(error);
    assert_eq!(body.as_ref(), b"stale; newest revision 2; read settings show again");
    assert_eq!(world.settings_rows(), before, "a stale call records no proposal or settings change");
    let mut unknown = charter.clone();
    unknown["unknown"] = json!(true);
    let mut missing = charter;
    missing.as_object_mut().expect("charter object").remove("instructions");
    for (completion, value, member) in [(11, unknown, "unknown"), (12, missing, "instructions")] {
        let input = json!({
            "against":2,"reason":REASON,
            "items":[{"setting":{"kind":"charter","number":CHAT},"set":value}]
        });
        let (body, error) =
            host_tool(&mut world.driver, &run, completion, b"propose_settings", input.to_string().as_bytes());
        assert!(error);
        let text = std::str::from_utf8(&body).expect("refusal text");
        assert!(text.contains("item 1"), "{text}");
        assert!(text.contains(member), "{text}");
        assert_eq!(world.settings_rows(), before, "an item out of shape records no proposal or settings change");
    }
    assert_eq!(world.head().number, 2);
}

#[test]
fn a_chat_is_offered_no_settings_tools() {
    let config = config();
    let policy = &config.core.defaults.policies.get(&1).expect("stock policy 1").policy;
    assert_ne!(policy.roles[0].authority.tools.0 & core::ToolFamilies::JIG.settings, 0);
    let mut world = World::new(config);
    let run = world.chat(CHAT, world.owner, 20);
    assert_eq!(run.charter, CHAT);
    let task = world.driver.task(run.task).expect("stock chat task");
    assert_ne!(task.authority.tools.0 & core::ToolFamilies::JIG.settings, 0);
    let offered = temper_engine_smith::charter(&run);
    assert!(!offered.grants.host_tools.is_empty(), "the chat is offered its other host tools");
    for name in [b"settings".as_slice(), b"propose_settings"] {
        assert!(!offered.grants.host_tools.iter().any(|tool| tool.name.as_ref() == name));
    }
}

#[test]
fn a_reference_sized_set_becomes_one_bounded_proposal() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let items = reference_sized_items(&mut world, &run);
    let document = json!({"against":1,"reason":REASON,"items":items}).to_string();
    assert!(document.len() >= 19_748, "reference-sized compact settings");
    assert!(items[0].to_string().len() > CALL_BYTES, "one setting exceeds a call");
    let before = world.settings_rows();
    let mut completion = 10;
    for (offset, text) in document.as_bytes().chunks(4_000).enumerate() {
        let input = json!({"against":1,"fragment":{"offset":offset * 4_000,
            "text":std::str::from_utf8(text).expect("fixture ASCII")}});
        let (body, error) = bounded_proposal(&mut world.driver, &run, completion, &input);
        assert!(!error, "{}", String::from_utf8_lossy(&body));
        assert_eq!(world.settings_rows(), before, "fragments apply nothing and propose nothing");
        completion += 1;
    }
    let input = json!({"against":1,"finish":document.len()});
    let (body, error) = bounded_proposal(&mut world.driver, &run, completion, &input);
    assert!(!error, "{}", String::from_utf8_lossy(&body));
    let proposal = proposal_number(&body);
    let review = world.review(&run, proposal);
    let core::documents::ProposalAction::Settings { review: Some(review), .. } = review.action else {
        panic!("complete review")
    };
    assert_eq!(usize::try_from(review.items.len()).expect("item count"), items.len());
    assert_eq!(world.head().number, 1, "finalization still needs owner acceptance");
    world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2, "one acceptance creates one revision");
    for (index, number) in [1, 2, 3].into_iter().enumerate() {
        assert_eq!(read_charter(&mut world.driver, &run, completion + number, number), items[index]["set"]);
    }
    let revision = world
        .settings_rows()
        .into_iter()
        .find_map(|record| match record {
            s::Record::Revision { history: false, revision } if revision.head.number == 2 => Some(revision),
            _ => None,
        })
        .expect("accepted revision");
    assert_eq!(revision.items.len(), items.len(), "collector sees the complete accepted edit set");
}

#[test]
fn settings_fragments_are_contiguous_bounded_and_retryable() {
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let before = world.settings_rows();
    let (first, error) = stage(&mut world, &run, 1, 1, 0, "aλ");
    assert!(!error);
    assert!(first.ends_with(b"3"), "offset counts UTF-8 bytes");
    let (again, error) = stage(&mut world, &run, 2, 1, 0, "aλ");
    assert!(!error);
    assert_eq!(again, first, "identical retransmission keeps the byte count");
    for (completion, offset, text) in [(3, 4, "gap"), (4, 1, "conflict")] {
        assert!(stage(&mut world, &run, completion, 1, offset, text).1, "noncontiguous fragment");
    }
    assert!(finish(&mut world, &run, 5, 1, 4).1, "length must match");
    assert!(!stage(&mut world, &run, 6, 1, 0, "").1, "empty offset zero discards");
    assert!(finish(&mut world, &run, 7, 1, 3).1, "discard leaves no assembly");
    let part = "x".repeat(8_000);
    for index in 0..8 {
        assert!(!stage(&mut world, &run, 10 + index, 1, usize::try_from(index).unwrap() * 8_000, &part).1);
    }
    assert!(!stage(&mut world, &run, 20, 1, 64_000, &"x".repeat(1_536)).1, "exact ceiling fits");
    assert!(stage(&mut world, &run, 21, 1, 65_536, "x").1, "one byte past is refused");
    assert_eq!(world.settings_rows(), before, "staging never creates settings or proposals");
    assert!(!stage(&mut world, &run, 22, 1, 0, "").1);
}

#[test]
fn a_settings_assembly_is_bound_to_its_task_attempt_and_base() {
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let other = world.chat(CONFIGURING, world.owner, 21);
    let value = read_charter(&mut world.driver, &run, 1, CHAT);
    let document = serde_json::json!({"against":1,"reason":REASON,
        "items":charter_item(value, b"A private proposed change.")})
    .to_string();
    assert!(!stage(&mut world, &run, 2, 1, 0, &document).1);
    assert!(finish(&mut world, &other, 1, 1, document.len()).1, "another task cannot finish it");
    assert!(stage(&mut world, &other, 2, 1, 0, "replace").1, "another task cannot replace it");
    world.park(&run);
    let next = world.wake(&run, 23);
    assert!(finish(&mut world, &next, 1, 1, document.len()).1, "new attempt has no assembly");
    assert!(!stage(&mut world, &next, 2, 1, 0, &document).1);
    world.change(1, 24, Box::new([owner_charter(b"The owner's change.")]));
    assert!(finish(&mut world, &next, 3, 1, document.len()).1, "stale base refused");
    assert!(finish(&mut world, &next, 4, 2, document.len()).1, "stale bytes discarded");
    assert!(!stage(&mut world, &other, 3, 2, 0, "fresh").1, "slot released");
    assert!(!stage(&mut world, &other, 4, 2, 0, "").1);
}

#[test]
fn finalization_validates_whole_values_and_the_staged_base() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let charter = read_charter(&mut world.driver, &run, 1, CHAT);
    let items = charter_item(charter, b"A complete change.");
    let before = world.settings_rows();
    for (index, input) in [
        json!({"against":2,"reason":REASON,"items":items}),
        json!({"against":1,"reason":REASON,"items":[{"setting":{"kind":"charter","number":1},"set":{}}]}),
    ]
    .into_iter()
    .enumerate()
    {
        let input = input.to_string();
        let completion = 10 + u32::try_from(index).unwrap() * 2;
        assert!(!stage(&mut world, &run, completion, 1, 0, &input).1);
        assert!(finish(&mut world, &run, completion + 1, 1, input.len()).1, "invalid finalization refused");
        assert_eq!(world.settings_rows(), before);
    }
    let input = json!({"against":1,"reason":REASON,"items":items}).to_string();
    assert!(!stage(&mut world, &run, 20, 1, 0, &input).1);
    let (body, error) = finish(&mut world, &run, 21, 1, input.len());
    assert!(!error);
    let (replayed, error) = finish(&mut world, &run, 21, 1, input.len());
    assert!(!error, "settled finalization replays without its volatile bytes");
    assert_eq!(replayed, body);
    world.decide(&run, proposal_number(&body), 21, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
}

#[test]
fn settings_fragments_require_the_tasks_authority() {
    let mut config = config();
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling.tools.0 &= !core::ToolFamilies::JIG.settings;
    for role in &mut policy.roles {
        role.authority.tools.0 &= !core::ToolFamilies::JIG.settings;
    }
    let mut world = World::new(config);
    let run = world.chat(CONFIGURING, world.owner, 20);
    assert_eq!(run.run.authority.tools.0 & core::ToolFamilies::JIG.settings, 0);
    assert!(stage(&mut world, &run, 1, 1, 0, "private").1, "authority must hold settings");
    assert!(finish(&mut world, &run, 2, 1, 7).1, "finalization requires the same authority");
    assert_eq!(world.head().number, 1);
}
