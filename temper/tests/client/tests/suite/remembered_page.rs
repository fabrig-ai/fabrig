//! Ask temper's kept line reaches its person's page and its Delete button
//! (temper/docs/design/client/README.md, 5.17; temper/docs/design/domain/agent.md, 8.4.4).
use jig_client_kit::harness::Until;
use skein_fake_person::{Find, Step};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_domain::{Address, pages::remembered::Row};
use temper_client_world::{fixtures as f, front::HostAnswer};
use temper_engine_domain as engine;

#[test]
fn the_page_reads_and_deletes_what_ask_temper_kept() {
    let mut world = f::asking_world(f::SEED, false);
    let agent = f::asking_agent(&mut world);
    let request = f::give(
        &mut world,
        agent,
        &[
            b"chat",
            b"start",
            b"--project",
            b"1",
            b"--template",
            b"5",
            b"Remember that I want short answers.",
            b"--key",
            b"remember-chat",
        ],
    );
    let started = f::answered(&mut world, agent, request);
    assert_eq!(started.status, 0, "{started:?}");
    let task = f::done_task(&started);
    assert_eq!(world.front().assignments.last().unwrap().charter, 11);
    let answer = world.front().host_tool(
        task,
        1,
        engine::Tool::Remember {
            action: engine::RememberAction::Add { text: b"I want short answers.".as_slice().into() },
        },
    );
    let HostAnswer::Tool(engine::CallAnswer::Remember(answer)) = answer else {
        panic!("remember answered: {answer:?}")
    };
    let engine::Remembered::Done { did, line, lines, .. } = *answer else {
        panic!("remember kept the line: {answer:?}")
    };
    assert_eq!(did, engine::RememberDid::Kept);
    assert_eq!(line, 1);
    assert_eq!(lines.as_slice(), &[Box::<[u8]>::from(b"I want short answers.".as_slice())]);
    world.front().asking_reply(task, b"I will keep answers short.");
    let person = world.client(f::PARTY, Address::Remembered);
    world.person(
        person,
        vec![
            Step::See {
                find: Find::exactly_named(Role::Heading { level: 1 }, b"What temper remembers about you"),
                within: f::WITHIN,
            },
            Step::See { find: Find::text(b"I want short answers."), within: f::WITHIN },
            Step::See { find: Find::text(format!("Ask temper chat {task}").as_bytes()), within: f::WITHIN },
            Step::See { find: Find::exactly_named(Role::Button, b"Delete"), within: f::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let page = world.root(person).person().pages().remembered().unwrap();
    assert!(page.answered());
    let rows: Vec<_> = page.rows().collect();
    let [Row::Task { place, line, chat, .. }] = rows.as_slice() else { panic!("one kept row: {rows:?}") };
    assert_eq!(*place, 1);
    assert_eq!(*line, b"I want short answers.");
    assert_eq!(*chat, Some(task));
    world.person(
        person,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Delete") },
            Step::See { find: Find::text(b"Nothing kept yet"), within: f::WITHIN },
            Step::Gone { find: Find::text(b"I want short answers."), within: f::WITHIN },
            Step::Gone { find: Find::exactly_named(Role::Button, b"Delete"), within: f::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let page = world.root(person).person().pages().remembered().unwrap();
    assert!(page.answered());
    assert!(page.rows().next().is_none());
    assert!(page.refusal().is_none());
    // A fresh read proves the empty page comes from the engine's deletion.
    world.go(person, Address::Home { project: 1 });
    world.go(person, Address::Remembered);
    world.person(person, vec![Step::See { find: Find::text(b"Nothing kept yet"), within: f::WITHIN }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let page = world.root(person).person().pages().remembered().unwrap();
    assert!(page.answered());
    assert!(page.rows().next().is_none());
}
