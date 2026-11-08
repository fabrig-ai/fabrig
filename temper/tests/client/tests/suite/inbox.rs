use jig_client_kit::harness::Until;
use skein_fake_person::{Face, Find, Found, Step, TreeFace};
use skein_ui::Role;
use skein_world::domain::Verdict;
use temper_client_domain::{
    Action, Address, Event,
    pages::{Act, all_home, home, needs, parts},
};
use temper_client_world::fixtures;

const QUESTION: &[u8] = b"Export widget's table as CSV? Go ahead?";
const ANSWER: &[u8] = b"Go ahead.";
const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;

#[test]
fn all_home_answers_one_projects_card_and_keeps_the_other_open() {
    let mut world = fixtures::world(fixtures::SEED);
    *world.front() = fixtures::across_people(fixtures::domain_limits());
    let first = fixtures::project_question(world.front(), PROJECT, b"Keep the first project's fallback?");
    let second = fixtures::project_question(world.front(), OTHER_PROJECT, QUESTION);
    let asker = world.front().engine.task(second).expect("second chat").delegates[0];
    let question = world.front().engine.task(asker).expect("second delegate").questions[0].number;
    let person = world.client(fixtures::PARTY, fixtures::inbox());
    let until = world.now().saturating_add(fixtures::WITHIN);
    assert_eq!(world.run(Until::Time(until)).verdict, Verdict::Passed);
    assert!(
        world.root(person).person().pages().inbox().expect("inbox page").known(),
        "inbox reads: {:?}",
        world.root(person).person().pages().inbox()
    );
    let cards = |world: &jig_client_kit::harness::World<
        temper_client_world::app::Temper,
        temper_client_world::front::Joint,
    >| {
        let root = world.root(person).person();
        root.pages()
            .needs()
            .open(root.pages().inbox().expect("unfiltered inbox"))
            .iter()
            .map(|card| (card.project, card.object))
            .collect::<Vec<_>>()
    };
    let open = cards(&world);
    assert_eq!(open.len(), 2);
    assert_eq!(open.iter().map(|(project, _)| *project).collect::<Vec<_>>(), [Some(PROJECT), Some(OTHER_PROJECT)]);
    assert_eq!(parts::needs_facts(world.root(person).person(), fixtures::NOW).open, 2);
    let object = open[1].1;
    assert_eq!(object, jig_client::contract::ObjectKey::Question { task: second, message: question });
    world.go(person, Address::AllHome);
    let until = world.now().saturating_add(fixtures::WITHIN);
    assert_eq!(world.run(Until::Time(until)).verdict, Verdict::Passed);
    assert_eq!(
        all_home::facts(world.root(person).person(), fixtures::NOW).needs,
        home::Needs::Count { count: 2, cut: false }
    );
    world.event(
        person,
        Event::Act { action: Action::Pages(Act::Needs(needs::Act::Choose { object, words: ANSWER.into() })) }.into(),
    );
    assert_eq!(cards(&world), [open[0]]);
    assert_eq!(parts::needs_facts(world.root(person).person(), fixtures::NOW).open, 1);
    assert!(
        !world
            .front()
            .engine
            .task(asker)
            .expect("asking delegate")
            .inbox
            .iter()
            .any(|message| message.kind == jig_domain_tasks::MessageKind::Answer { question }),
        "the answer stays held before five seconds"
    );
    let held_until = world.now().saturating_add(needs::HOLD).saturating_add(fixtures::WITHIN);
    assert_eq!(world.run(Until::Time(held_until)).verdict, Verdict::Passed);
    assert!(world.front().engine.task(asker).expect("answered delegate").inbox.iter().any(|message| {
        message.from == jig_domain_tasks::Party::Task(second)
            && message.kind == jig_domain_tasks::MessageKind::Answer { question }
            && message.words.as_ref() == ANSWER
    }));
    assert!(
        world
            .front()
            .engine
            .task(first)
            .expect("first chat")
            .inbox
            .iter()
            .any(|message| message.kind == jig_domain_tasks::MessageKind::Question)
    );
    world.go(person, fixtures::inbox());
    world.person(
        person,
        vec![Step::See { find: Find::text(b"Keep the first project's fallback?"), within: fixtures::WITHIN }],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(cards(&world), [open[0]]);
    let facts = parts::needs_facts(world.root(person).person(), fixtures::NOW);
    assert!(facts.known);
    assert_eq!(facts.open, 1);
}

#[test]
fn a_delegates_question_answered_in_the_inbox_reaches_the_delegate() {
    let mut world = fixtures::world(fixtures::SEED);
    let chat = fixtures::agreed_question(world.front(), QUESTION);
    let (question, delegate) = {
        let task = world.front().engine.task(chat).expect("asked chat");
        let question = task
            .inbox
            .iter()
            .find(|message| {
                message.kind == jig_domain_tasks::MessageKind::Question && message.words.as_ref() == QUESTION
            })
            .expect("delegate's question");
        let jig_domain_tasks::Party::Task(delegate) = question.from else { panic!("question from a task") };
        (question.number, delegate)
    };
    assert!(world.front().engine.task(delegate).expect("agreed goal").spec.asks);
    let person = world.client(fixtures::PARTY, fixtures::inbox());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Heading { level: 2 }, b"Needs you"), within: fixtures::WITHIN },
            Step::See { find: Find::exactly_named(Role::Paragraph, b"Question"), within: fixtures::WITHIN },
            Step::See { find: Find::text(QUESTION), within: fixtures::WITHIN },
            Step::See { find: Find::exactly_named(Role::Link, b"Needs you 1"), within: fixtures::WITHIN },
            Step::Type { find: Find::named(Role::TextArea, b"Answer words"), words: ANSWER.into() },
            Step::Press { find: Find::named(Role::Button, b"Answer") },
            Step::See { find: Find::exactly_named(Role::Status, b"Answered"), within: fixtures::WITHIN },
            Step::See { find: Find::exactly_named(Role::Link, b"Needs you"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    assert!(world.front().engine.task(delegate).expect("answered delegate").inbox.iter().any(|message| {
        message.from == jig_domain_tasks::Party::Task(chat)
            && message.kind == jig_domain_tasks::MessageKind::Answer { question }
            && message.words.as_ref() == ANSWER
    }));

    assert!(
        world
            .front()
            .engine
            .task(chat)
            .expect("chat notice")
            .inbox
            .iter()
            .any(|word| word.kind == jig_domain_tasks::MessageKind::QuestionAnswered { question })
    );
    world.go(person, fixtures::chat_page(chat));
    world.person(person, vec![Step::See { find: Find::text(QUESTION), within: fixtures::WITHIN }]);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    let tree = world.tree(person);
    assert!(
        turn_holds(tree, format!("T{delegate}").as_bytes(), QUESTION),
        "delegate T{delegate}'s question turn missing"
    );
    assert!(!turn_holds(tree, b"You", ANSWER), "the answer goes directly to its asker");

    world.go(person, fixtures::task_page(delegate));
    world.person(
        person,
        vec![
            Step::See {
                find: Find::exactly_named(Role::Heading { level: 1 }, b"Planning: reading what was asked."),
                within: fixtures::WITHIN,
            },
            Step::See { find: Find::text(b"Ask the person"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");

    world.go(person, fixtures::inbox());
    world.person(
        person,
        vec![
            Step::See { find: Find::named(Role::Heading { level: 2 }, b"Needs you"), within: fixtures::WITHIN },
            Step::Gone { find: Find::text(QUESTION), within: fixtures::WITHIN },
            Step::Gone { find: Find::exactly_named(Role::Paragraph, b"Question"), within: fixtures::WITHIN },
        ],
    );
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}

fn turn_holds(tree: &skein_ui::Tree, label: &[u8], words: &[u8]) -> bool {
    let Some((start, log)) = tree
        .nodes()
        .iter()
        .enumerate()
        .find(|(_, node)| node.role == Role::Log && node.name.as_deref() == Some(b"Conversation"))
    else {
        return false;
    };
    let conversation = &tree.nodes()[start..start + usize::try_from(log.size).expect("log size")];
    conversation.iter().filter(|node| node.role == Role::Region).any(|turn| {
        let mut face = TreeFace::subtree(tree, turn.id).expect("conversation turn");
        matches!(face.find(&Find::exactly_named(Role::Paragraph, label)), Found::Node(_))
            && matches!(face.find(&Find::exactly_named(Role::Paragraph, words)), Found::Node(_))
    })
}

#[test]
fn a_held_chats_escalation_released_from_the_inbox() {
    let mut world = fixtures::world(fixtures::SEED);
    let task = fixtures::held_chat(world.front());
    let address = fixtures::inbox();
    let person = world.client(fixtures::PARTY, address);
    world.person(
        person,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Release task"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Release task") },
            Step::Press {
                find: Find::named(Role::Button, b"Release")
                    .within(Box::new([skein_fake_person::Target::Role { role: Role::Dialog, name: Box::new([]) }])),
            },
            Step::See { find: Find::text(b"Released"), within: fixtures::WITHIN },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
    assert_eq!(world.front().engine.task(task).expect("released chat").tries, jig_domain_tasks::Tries::NONE);
}

#[test]
fn releasing_says_what_releasing_does_before_it_is_sent() {
    let mut world = fixtures::world(fixtures::SEED);
    fixtures::held_chat(world.front());
    let person = world.client(fixtures::PARTY, fixtures::inbox());
    world.person(
        person,
        vec![
            Step::See { find: Find::exactly_named(Role::Button, b"Release task"), within: fixtures::WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Release task") },
            Step::See {
                find: Find::text(b"The next run resumes its conversation; tries count again from zero"),
                within: fixtures::WITHIN,
            },
        ],
    );
    assert_eq!(world.run(Until::Done).verdict, Verdict::Passed);
}
