//! A charter's documents reach its runs through the core, root and forge
//! (temper/docs/design/domain/testing.md, 6.4;
//! temper/docs/design/protocol/configuration.md, 13.7).
use super::*;
use jig_domain_core::Document;

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
