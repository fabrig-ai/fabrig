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
fn a_configuring_chat_discovers_the_forge_host_for_charter_documents() {
    use serde_json::json;
    let mut world = repository_projects();
    configuring_repository_text(&mut world, &["rules\n".into(), "words\n".into()]);
    adopt_repository(&mut world, 2, b"other", 28);
    let run = world.chat(CONFIGURING, world.owner, 20);
    let (inventory, _) = read_whole(&mut world.driver, &run, 1, json!({}));
    let row = inventory
        .lines()
        .find_map(|line| line.strip_prefix("  forge "))
        .expect("settings inventory exposes the canonical forge host and repository");
    let (host, repository) = row.split_once(" repository ").expect("repository identity");
    assert_eq!(host, "forge.example:3847", "the canonical host keeps its port");
    assert_eq!(repository, "org/repo");
    assert!(!inventory.contains("org/other"), "another project's repository stays private");
    let mut value = read_charter(&mut world.driver, &run, 10, CHAT);
    value["documents"] = json!([{
        "forge":host,"repository":repository,"branch":"main","path":"docs/direction.md"
    }]);
    let items = json!([{"setting":{"kind":"charter","number":CHAT},"set":value}]);
    let proposal = propose(&mut world.driver, &run, 20, 1, &items);
    world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    let chat = world.chat(CHAT, world.owner, 22);
    let section = chat
        .sections
        .iter()
        .find(|section| {
            matches!(&section.kind,
        engine::BriefKind::Forge(engine::ForgeBriefKind::Document(name))
            if name.as_ref() == b"org/repo docs/direction.md")
        })
        .expect("accepted charter document gathered through the fake forge");
    let engine::BriefBody::Text(text) = &section.body else { panic!("document text: {:?}", section.body) };
    let text = std::str::from_utf8(text).expect("document UTF-8");
    assert!(text.starts_with("At main's head "), "{text}");
    assert!(text.contains("rules\n"), "{text}");
}

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
fn a_project_with_no_ways_reads_at_their_stock() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    for (completion, kind, value) in [
        (1, "project-ways", r#"{"drafts":false,"goals":false,"changes":false,"away":false,"way":0}"#),
        (2, "project-questions", r#""none""#),
    ] {
        let input = json!({"setting":{"kind":kind,"project":PROJECT}});
        let (body, error) = host_tool(&mut world.driver, &run, completion, b"settings", input.to_string().as_bytes());
        let text = std::str::from_utf8(&body).expect("settings UTF-8");
        assert!(!error, "{text}");
        let expected = format!("{kind} {PROJECT}, revision 1\n{value}\nthe stock\nend\n");
        assert_eq!(text, expected);
        let input = json!({"setting":{"kind":kind,"project":PROJECT},"revision":1});
        let (body, error) =
            host_tool(&mut world.driver, &run, completion + 2, b"settings", input.to_string().as_bytes());
        assert!(!error);
        assert_eq!(body.as_ref(), expected.as_bytes(), "the named newest revision also takes the stock");
    }
}

#[test]
fn a_projects_stored_ways_read_as_stored() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let setting = json!({"kind":"project-ways","project":PROJECT});
    let value = json!({"drafts":true,"goals":true,"changes":true,"away":true,"way":1});
    let items = json!([{"setting":setting,"set":value}]);
    let proposal = propose(&mut world.driver, &run, 1, 1, &items);
    world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    let (text, _) = read_whole(&mut world.driver, &run, 2, json!({"setting":setting}));
    let body = text.strip_prefix("project-ways 1, revision 2\n").expect("newest setting heading");
    let (body, state) = body.split_once('\n').expect("value and application state");
    assert_eq!(serde_json::from_str::<serde_json::Value>(body).expect("ways JSON"), value);
    assert!(state.starts_with("changed at revision 2, "), "{text}");
    assert!(!text.contains("the stock"), "{text}");
}

#[test]
fn a_read_that_finds_nothing_says_it_was_not_read() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    world.change(1, 21, Box::new([owner_charter(b"Keep commits small.")]));
    for (completion, input) in [
        (1, json!({"setting":{"kind":"project-ways","project":999}})),
        (2, json!({"setting":{"kind":"project-questions","project":999}})),
        (3, json!({"setting":{"kind":"charter","number":REMOVED}})),
        (4, json!({"setting":{"kind":"project-ways","project":PROJECT},"revision":1})),
        (5, json!({"setting":{"kind":"project-questions","project":PROJECT},"revision":1})),
    ] {
        let (body, error) = host_tool(&mut world.driver, &run, completion, b"settings", input.to_string().as_bytes());
        let text = std::str::from_utf8(&body).expect("settings UTF-8");
        assert!(error, "{input}: {text}");
        assert_eq!(
            text,
            "not read: no such setting at that revision. This says nothing about the settings: read the inventory and name what it lists; project-ways and project-questions take project and no number.\nend\n"
        );
    }
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
    assert_eq!(pages, 1, "the host allowance holds the stock configuring instructions");
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
    let mut ask = world.driver.chat_ask(PROJECT, b"Read without settings authority");
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("chat creation") };
    batch[0].authority.tools &= !core::ToolFamilies::JIG.settings;
    batch[0].budget = people::Budget::AsAllowed { cap: Some(128) };
    world.ask(world.owner, 21, ask);
    let run = world.driver.last_assignment();
    assert_eq!(run.charter, CHAT);
    assert_eq!(run.run.authority.tools.0 & core::ToolFamilies::JIG.settings, 0);
    for (completion, kind) in [(1, "project-ways"), (2, "project-questions")] {
        let input = serde_json::json!({"setting":{"kind":kind,"project":PROJECT}});
        let (_, error) = host_tool(&mut world.driver, &run, completion, b"settings", input.to_string().as_bytes());
        assert!(error, "the stock cannot bypass the settings family");
        let (_, answer) = world.driver.protocol.answers.last().expect("family denial");
        assert!(matches!(answer, engine::CallAnswer::ToolDenied { .. }));
    }
}

#[test]
fn settings_and_named_documents_use_the_host_answer_allowance() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let (inventory, error) = host_tool(&mut world.driver, &run, 1, b"settings", br"{}");
    assert!(!error);
    assert!(inventory.starts_with(b"newest revision 1\n"));
    let input = json!({"setting":{"kind":"policy","number":1}});
    let (whole, pages) = read_whole(&mut world.driver, &run, 2, input.clone());
    assert!(whole.len() > 7000, "stock policy exercises a long JSON value");
    assert_eq!(pages, 1, "the configured host allowance holds the whole stock policy");
    let mut small = limits();
    small.call_answer_bytes = 2048;
    small.authority.segments = 8;
    small.people.people = 4;
    small.people.holdings = 4;
    small.forge.client.answer_bytes = 65_536;
    small.journal.transcript_bytes = 65_536;
    let mut world = World::with_limits(config(), &small);
    configuring_repository(&mut world);
    let run = world.chat(CONFIGURING, world.owner, 20);
    let (first, error) = host_tool(&mut world.driver, &run, 9, b"settings", input.to_string().as_bytes());
    assert!(!error);
    assert!(first.len() > 1000, "a long value fills the page after its heading");
    let (continued, pages) = read_whole(&mut world.driver, &run, 10, input);
    assert!(pages > 1);
    assert_eq!(continued, whole, "continuations reconstruct every original byte");
    configuring_documents(&mut world, &run);
}

fn configuring_repository(world: &mut World) {
    configuring_repository_text(world, &["rules\n".repeat(5334), "words\n".repeat(2167)]);
}

fn configuring_repository_text(world: &mut World, contents: &[String; 2]) {
    use skein_fake_forge_domain::api as raw;
    use temper_engine_domain_world::forge_stand::ForgeStand;
    let paths = ["docs/direction.md", "docs/conventions.md"];
    world.driver.forge = Some(ForgeStand::with_setup(
        [(2, "org/repo"), (3, "org/other")]
            .map(|(id, name)| raw::Setup {
                workflows: Box::new([]),
                id,
                name: name.as_bytes().into(),
                default: b"main".as_slice().into(),
                tree: paths
                    .iter()
                    .zip(contents)
                    .map(|(path, content)| raw::File {
                        path: path.as_bytes().into(),
                        content: content.as_bytes().into(),
                    })
                    .collect(),
                labels: Box::new([]),
                protection: None,
                hooked: false,
                checks: raw::Checks {
                    cues: Box::new([]),
                    contexts: Box::new([]),
                    latency_min: skein_lib::Duration::ZERO,
                    latency_max: skein_lib::Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    reruns: 0,
                    cue: None,
                },
            })
            .into(),
    ));
    adopt_repository(world, PROJECT, b"repo", 29);
}

fn configuring_documents(world: &mut World, run: &engine::Assignment) {
    use serde_json::json;
    let paths = ["docs/direction.md", "docs/conventions.md"];
    let contents = ["rules\n".repeat(5334), "words\n".repeat(2167)];
    let (branch, error) = host_tool(&mut world.driver, run, 30, b"read_branch", br#"{"branch":"main"}"#);
    assert!(!error, "{}", String::from_utf8_lossy(&branch));
    let head = std::str::from_utf8(&branch).expect("head").trim();
    for (index, path) in paths.iter().enumerate() {
        for (offset, max_bytes) in [(0, None), (1, Some(64))] {
            let mut input = json!({"head":head,"path":path});
            if let Some(max_bytes) = max_bytes {
                input["max_bytes"] = json!(max_bytes);
            }
            let (body, error) = host_tool(
                &mut world.driver,
                run,
                31 + u32::try_from(index).expect("document index") * 2 + offset,
                b"read_file",
                input.to_string().as_bytes(),
            );
            assert!(!error, "{}", String::from_utf8_lossy(&body));
            assert!(body.len() <= 2048 - b"read_file".len());
            let text = std::str::from_utf8(&body).expect("file answer");
            assert!(text.contains(path));
            assert!(text.ends_with("[file truncated]"));
            let (_, prefix) = text.split_once(":\n").expect("file metadata");
            let prefix = prefix.strip_suffix("\n[file truncated]").expect("visible cut");
            let prefix = prefix.split_once("\nread_file rendered size ").map_or(prefix, |(content, _)| content);
            assert!(contents[index].starts_with(prefix) && !prefix.is_empty());
            if max_bytes.is_some() {
                assert_eq!(prefix.len(), 64);
            }
        }
    }
}

fn adopt_repository(world: &mut World, project: u32, name: &[u8], key: u8) {
    use temper_engine_domain_forge as forge;
    world.driver.send(Event::Party(PartyInput::Application {
        scope: people::ApplicationScope::Project(project),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.owner,
        key: [key; 16],
        digest: [key; 32],
        carried: None,
        request: engine::Application::Forge(forge::Ask::Adopt(forge::Adoption {
            project,
            home: false,
            forge: 1,
            host: b"forge.example:3847".as_slice().into(),
            owner: b"org".as_slice().into(),
            name: name.into(),
            checks: Box::new([]),
            ci: true,
            landing: b"main".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: forge::Role::Owned,
        })),
    }));
    world.driver.settle();
    let reply = world.driver.delivered.iter().rev().find(|delivery| {
        matches!(delivery,
        Delivery::Application { to, .. } | Delivery::WebReply { to, .. }
            if *to == ReplyTo::new(Token::new(u64::from(key))))
    });
    assert!(
        matches!(
            reply,
            Some(Delivery::Application {
                outcome: Some(engine::ApplicationAnswer::Adopt(forge::ApplicationOutcome::Adopted(_))),
                ..
            })
        ),
        "repository adopted in project {project}: {reply:?}"
    );
}
