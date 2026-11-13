//! A charter's documents reach its runs through the core, root and forge
//! (temper/docs/design/testing/checks.md, 4;
//! temper/docs/design/protocol/configuration/settings.md, 6).
use super::*;
use jig_domain_core::Document;
use jig_domain_link::EndpointBound;

const CHANNEL: Token = Token::new(7);
const CHAT: Token = Token::new(92);
const CONTINUE: Token = Token::new(93);
const PROJECT: u32 = 1;
const REPOSITORY: &[u8] = b"org/repo";
const BRANCH: &[u8] = b"main";
const PATH: &[u8] = b"AGENTS.md";
const TITLE: &[u8] = b"Document org/repo AGENTS.md";
const WHOLE: &[u8] =
    b"# Guide\nKeep the rules whole.\n## Rules\nRun the focused tests.\n## Other\nAsk about boundaries.\n";
const CHANGED: &[u8] = b"# Guide\nRun the document stories before pushing.\n";

fn document(repository: &[u8], path: &[u8], heading: Option<&[u8]>) -> Document {
    Document {
        connector: 0,
        name: authority::Name {
            segments: [b"forge".as_slice(), b"forge.example", b"org", repository, b"branch", BRANCH]
                .map(Box::from)
                .into(),
        },
        path: path.into(),
        heading: heading.map(Box::from),
    }
}

fn push_document(world: &mut World, path: &[u8], content: &[u8]) -> u64 {
    fake::advance(&mut world.fake, &world.fake_env, REPOSITORY, BRANCH, path, content, 1)
        .expect("the document pushed to main")
}

fn document_world(documents: &[Document]) -> (World, u64) {
    let mut world = World::configured_documents(documents);
    let head = push_document(&mut world, PATH, WHOLE);
    world.adopt();
    world.send(protocol::hello(CHANNEL, Box::new([])));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(CHAT),
        sign_in: world.signed_in.expect("owner session"),
        key: [92; 16],
        ask: world.chat(PROJECT, b"Follow the charter's document"),
    }));
    world.until(Until::Assigned);
    (world, head)
}

fn document_section<'a>(assignment: &'a engine::Assignment, name: &[u8]) -> &'a engine::BriefSection {
    assignment
        .sections
        .iter()
        .find(|section| {
            matches!(&section.kind,
            engine::BriefKind::Forge(engine::ForgeBriefKind::Document(words)) if words.as_ref() == name)
        })
        .unwrap_or_else(|| panic!("No document section: {:?}", assignment.sections))
}

fn assert_document(assignment: &engine::Assignment, title: &[u8], expected: &[u8]) {
    let section = document_section(assignment, title.strip_prefix(b"Document ").expect("document title"));
    let engine::BriefBody::Text(words) = &section.body else {
        panic!("Document has no text: {:?}", assignment.sections)
    };
    assert_eq!(words.as_ref(), expected, "assignment sections: {:?}", assignment.sections);
    let rendered = temper_engine_smith::section(section);
    assert_eq!(rendered.title.as_ref(), title);
    assert_eq!(rendered.text.as_ref(), expected);
}

fn at_head(head: u64, text: &[u8]) -> Vec<u8> {
    [b"At main's head ".as_slice(), &forge_top::commit_text(&translate::commit(head)), b"\n", text].concat()
}

fn next_run(world: &mut World, first: &engine::Assignment) -> engine::Assignment {
    let event = world.protocol.answer(CHANNEL, first.task, first.attempt, 0, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(CONTINUE),
        sign_in: world.signed_in.expect("owner session"),
        key: [93; 16],
        ask: people::Ask::Say { project: PROJECT, task: first.task, words: b"Read the rules again".as_slice().into() },
    }));
    world.until(Until::SecondAssignment);
    world.until(Until::Ready);
    world.assigned[1].clone()
}

fn assert_running(world: &World, assignment: &engine::Assignment) {
    assert!(
        matches!(live_task(world, assignment.task).phase,
            tasks::Phase::Active(tasks::Active::Running { attempt }) if attempt == assignment.attempt),
        "the run starts despite the missing document: {:?}",
        assignment.sections
    );
}

#[test]
fn a_charters_document_reaches_its_runs_brief_at_the_branch_head() {
    let (mut world, head) = document_world(&[document(b"repo", PATH, None)]);
    assert_document(&world.assigned[0], TITLE, &at_head(head, WHOLE));
    world.close();
}

#[test]
fn a_landed_change_to_the_document_reaches_the_next_run() {
    let (mut world, head) = document_world(&[document(b"repo", PATH, None)]);
    let first = world.assigned[0].clone();
    assert_document(&first, TITLE, &at_head(head, WHOLE));
    let next_head = push_document(&mut world, PATH, CHANGED);
    let next = next_run(&mut world, &first);
    assert_eq!(next.task, first.task);
    assert_ne!(next.attempt, first.attempt);
    assert_eq!(next.charter, first.charter);
    assert_eq!(next.run.revision, first.run.revision);
    assert_ne!(next_head, head);
    assert_document(&next, TITLE, &at_head(next_head, CHANGED));
    world.close();
}

#[test]
fn a_documents_heading_gives_its_part_alone() {
    let (mut world, head) = document_world(&[document(b"repo", PATH, Some(b"Rules"))]);
    assert_document(
        &world.assigned[0],
        b"Document org/repo AGENTS.md, \"Rules\"",
        &at_head(head, b"## Rules\nRun the focused tests.\n"),
    );
    world.close();
}

#[test]
fn a_missing_document_says_so_and_the_run_goes_on() {
    const MISSING_PATH: &[u8] = b"MISSING.md";
    const MISSING_TITLE: &[u8] = b"Document org/repo MISSING.md, \"Rules\"";
    let (mut world, head) =
        document_world(&[document(b"repo", MISSING_PATH, Some(b"Rules")), document(b"other", PATH, None)]);
    world.until(Until::Ready);
    let first = world.assigned[0].clone();
    assert_document(
        &first,
        MISSING_TITLE,
        &[
            b"No file MISSING.md in org/repo at main's head ".as_slice(),
            &forge_top::commit_text(&translate::commit(head)),
            b".",
        ]
        .concat(),
    );
    assert_document(&first, b"Document org/other AGENTS.md", b"Repository org/other on forge.example is not adopted.");
    assert_running(&world, &first);
    let next_head = push_document(&mut world, MISSING_PATH, b"# Guide\nNo rules heading here.\n");
    let next = next_run(&mut world, &first);
    assert_document(
        &next,
        MISSING_TITLE,
        &[
            b"No heading \"Rules\" in MISSING.md at main's head ".as_slice(),
            &forge_top::commit_text(&translate::commit(next_head)),
            b".",
        ]
        .concat(),
    );
    assert_running(&world, &next);
    world.close();
}

fn settings_world() -> (World, u64) {
    let model = stock::box_model();
    let llm = jig_protocol_config::Llm {
        endpoints: Box::new([]),
        providers: Box::new([]),
        models: (Box::new([model.clone()]))
            .into_iter()
            .enumerate()
            .map(|(at, model)| jig_domain_core::settings::CatalogueModel {
                first_read: None,
                number: u32::try_from(at).expect("bounded fixture models") + 1,
                model,
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: jig_domain_core::settings::ModelSource::Stock,
                estimated: false,
            })
            .collect(),
    };
    let limits = temper_engine_domain_world::forge_routes::box_limits();
    let defaults = temper_engine_service::stock::with_models(llm.models.clone(), limits.tasks.result_bytes);
    let mut world = World::configured_stock_limits(&defaults, &model, &limits);
    let head = push_document(&mut world, b"README.md", WHOLE);
    world.adopt();
    (world, head)
}

fn shown_charter(world: &mut World, number: u32) -> (u32, serde_json::Value) {
    use jig_domain_core::{reads, settings as s};
    let reads::Answer::Setting(document) = world.party_read(reads::Read::Setting {
        setting: s::Setting::Charter(number),
        revision: temper_engine_domain_world::kinds::revision(&world.store.rows),
        proposal: None,
    }) else {
        panic!("charter read")
    };
    let limits = temper_client_tools::limits::tools();
    let wire = jig_protocol_engine_clients::translate::settings::setting_document(&document, &[], &limits.settings)
        .expect("wire charter");
    let preset = temper_client_tools::limits::client().client;
    let link = jig_client_protocol_link::Limits::new(preset.domain, preset.exchanges).expect("client limits");
    let value =
        jig_client_protocol_link::translate::settings::restore_setting_document(&wire, &link).expect("client charter");
    let text = temper_client_tools::jig::render::settings_text(
        &jig_client::contract::ReadValue::Setting(Box::new(value)),
        None,
        &limits,
    );
    let text = std::str::from_utf8(&text).expect("shown charter text");
    let json = text.lines().find(|line| line.starts_with('{')).expect("shown charter value");
    (document.revision, serde_json::from_str(json).expect("charter JSON"))
}

fn document_items(number: u32, value: &serde_json::Value) -> Box<[jig_domain_core::settings::Item]> {
    use jig_client::contract as c;
    let file = serde_json::to_vec(&serde_json::json!({
        "items": [{"setting": {"kind": "charter", "number": number}, "set": value}]
    }))
    .expect("settings file");
    let limits = temper_client_tools::limits::tools();
    let items = temper_client_tools::jig::settings_file::decode(&file, &limits).expect("shared settings decoder");
    let ask = c::Ask::ChangeSettings(c::ChangeSettings { against: 1, items, running: c::Running::Stays });
    let jig_client_protocol_wire::Ask::ChangeSettings(change) =
        jig_client_protocol_link::translate::asks::ask(&ask, &limits.settings).expect("settings wire")
    else {
        panic!("settings change")
    };
    jig_protocol_engine_clients::translate::settings::items(change.items().into(), &[], &limits.settings)
        .expect("settings items reach the core")
}

fn accept_document(world: &mut World, proposal: u64, against: u32) {
    use jig_domain_core::{documents as d, reads};
    let reads::Answer::Document(document) =
        world.party_read(reads::Read::Proposal { proposer: people::Proposer::Party(1), proposal })
    else {
        panic!("proposal read")
    };
    let d::Document::Proposal(document) = document.as_ref() else { panic!("proposal document") };
    let d::ProposalAction::Settings { review: Some(review), .. } = &document.action else { panic!("settings review") };
    assert_eq!(review.newest, against);
    assert!(matches!(review.items.get(0).expect("reviewed charter").would_be.as_ref(),
        Some(jig_domain_core::settings::Value::Charter(charter)) if charter.documents[0].connector == 1));
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(81)),
        sign_in: world.signed_in.expect("owner"),
        key: [81; 16],
        ask: people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Party(1),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    }));
    world.until(Until::Ready);
    assert!(
        world.web_replies.iter().any(|reply| matches!(
            reply,
            people::Reply::Outcome(people::Outcome::ProposalDecided { choice: people::ProposalChoice::Accepted, .. })
        )),
        "acceptance: {:?}",
        world.web_replies
    );
}

fn chat_with_document(world: &mut World) -> engine::Assignment {
    let mut event = protocol::hello(CHANNEL, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("worker hello")
    };
    let bounds = &mut hello.bounds;
    let model = stock::box_model();
    bounds.endpoints =
        Box::new([EndpointBound { endpoint: model.endpoint, window: model.window, output: model.output }]);
    bounds.turns = 800;
    bounds.time = Duration::from_secs(28_800);
    bounds.spend = 2_500_000;
    bounds.message_bytes = world.env.limits.tasks.message_bytes.max(world.env.limits.tasks.result_bytes);
    bounds.inbox = world.env.limits.tasks.run_inbox;
    world.send(event);
    let ask = world.chat(PROJECT, b"Follow the README document");
    assigned_chat(world, 92, ask)
}

#[test]
fn a_proposed_charter_document_reaches_a_later_runs_brief() {
    let (mut world, head) = settings_world();
    let (against, mut value) = shown_charter(&mut world, 1);
    let document = serde_json::json!({
        "forge": "forge.example", "repository": "org/repo", "branch": "main", "path": "README.md"
    });
    value["documents"] = serde_json::json!([document]);
    world.send(temper_engine_domain_world::kinds::proposal(
        world.signed_in.expect("owner"),
        against,
        80,
        document_items(1, &value),
    ));
    world.until(Until::Ready);
    let proposal = world
        .web_replies
        .iter()
        .find_map(|reply| match reply {
            people::Reply::Outcome(people::Outcome::Proposed { proposal, .. }) => Some(*proposal),
            _ => None,
        })
        .unwrap_or_else(|| panic!("document proposal refused: {:?}", world.web_replies));
    accept_document(&mut world, proposal, against);
    let (revision, shown) = shown_charter(&mut world, 1);
    assert_eq!(revision, against + 1);
    let mut expected = document;
    expected["heading"] = serde_json::Value::Null;
    assert_eq!(shown["documents"], serde_json::json!([expected]));
    let assignment = chat_with_document(&mut world);
    assert_eq!(assignment.run.revision, revision);
    assert_document(&assignment, b"Document org/repo README.md", &at_head(head, WHOLE));
    world.close();
}

#[test]
fn applied_charter_documents_keep_null_and_named_headings() {
    let (mut world, head) = settings_world();
    for number in [1, 2] {
        for heading in [serde_json::Value::Null, serde_json::json!("Rules")] {
            let (against, mut value) = shown_charter(&mut world, number);
            let document = serde_json::json!({
                "forge": "forge.example", "repository": "org/repo", "branch": "main",
                "path": "README.md", "heading": heading,
            });
            value["documents"] = serde_json::json!([document]);
            let key = u8::try_from(against + 70).expect("fixture request key");
            world.send(temper_engine_domain_world::kinds::change(
                world.signed_in.expect("owner"),
                against,
                key,
                document_items(number, &value),
            ));
            world.until(Until::Ready);
            let (revision, shown) = shown_charter(&mut world, number);
            assert_eq!(revision, against + 1, "apply: {:?}", world.web_replies);
            assert_eq!(shown["documents"], serde_json::json!([document]));
        }
    }
    let assignment = chat_with_document(&mut world);
    assert_document(
        &assignment,
        b"Document org/repo README.md, \"Rules\"",
        &at_head(head, b"## Rules\nRun the focused tests.\n"),
    );
    world.close();
}
