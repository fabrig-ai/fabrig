//! Composers through the client link and engine (client/view.md, 4).
use jig_client_kit::harness::{ClientId, Until, World};
use skein_fake_person::{Find, Step};
use skein_ui::{Role, State};
use skein_world::domain::Verdict;
use temper_client_domain::Address;
use temper_client_world::{app::Temper, fixtures as f, front::Joint};

type Pages = World<Temper, Joint>;
fn follow(world: &mut Pages, client: ClientId, steps: Vec<Step<Address>>) {
    world.person(client, steps);
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
}
fn see(words: &[u8]) -> Step<Address> {
    Step::See { find: Find::text(words), within: f::WITHIN }
}
fn type_words(name: &[u8], words: &[u8]) -> Step<Address> {
    Step::Type { find: Find::named(Role::TextArea, name), words: words.into() }
}
fn assert_over(world: &Pages, client: ClientId, area: &[u8], send: &[u8], typed: &[u8]) {
    let nodes = world.tree(client).nodes();
    let area = nodes
        .iter()
        .find(|n| n.role == Role::TextArea && n.name.as_deref() == Some(area))
        .expect("reply or message composer");
    assert_eq!(area.value.as_ref().expect("composers have values").text.as_ref(), typed);
    assert!(!area.states.as_slice().contains(&State::SubmitsOnEnter));
    let send = nodes
        .iter()
        .find(|n| n.role == Role::Button && n.name.as_deref() == Some(send))
        .expect("composer send control");
    assert!(send.states.as_slice().contains(&State::Disabled));
}
fn chat() -> (Pages, ClientId, u64) {
    let mut world = f::long_messages_world(f::SEED);
    world.front().reply_in_pieces(&[b"Working"]);
    let task = f::chat(world.front(), b"Hello");
    let client = f::long_messages_client(&mut world, f::chat_page(task));
    follow(&mut world, client, vec![Step::See { find: Find::named(Role::TextArea, b"Message"), within: f::WITHIN }]);
    (world, client, task)
}
#[test]
fn a_paragraph_pasted_into_a_chat_reads_back() {
    let (mut world, client, task) = chat();
    let words = vec![b'p'; 1024];
    follow(
        &mut world,
        client,
        vec![type_words(b"Message", &words), Step::Send { find: Find::named(Role::TextArea, b"Message") }, see(&words)],
    );
    assert!(world.front().engine.task(task).unwrap().inbox.iter().any(|line| line.words.as_ref() == words));
    // Reopen the page so this is a read through the engine, beyond the local pending line.
    let reader = f::long_messages_client(&mut world, f::chat_page(task));
    follow(&mut world, reader, vec![see(&words)]);
}
#[test]
fn a_paste_past_the_bound_stays_and_says_how_far_over() {
    let (mut world, client, task) = chat();
    let words = vec![b'p'; 8193];
    follow(
        &mut world,
        client,
        vec![type_words(b"Message", &words), see(b"That's 1 character over what one message holds.")],
    );
    assert_over(&world, client, b"Message", b"Send", &words);
    assert!(!world.front().engine.task(task).unwrap().inbox.iter().any(|line| line.words.as_ref() == words));
    let edited = vec![b'e'; 1024];
    follow(
        &mut world,
        client,
        vec![type_words(b"Message", &edited), Step::Press { find: Find::named(Role::Button, b"Send") }, see(&edited)],
    );
    assert!(world.front().engine.task(task).unwrap().inbox.iter().any(|line| line.words.as_ref() == edited));
}
#[test]
fn a_new_chats_paste_stays_with_send_disabled() {
    let mut world = f::long_messages_world(f::SEED);
    let client = f::long_messages_client(&mut world, Address::NewChat { draft: false });
    let words = vec![b'p'; 8193];
    follow(
        &mut world,
        client,
        vec![
            Step::See { find: Find::named(Role::TextArea, b"What do you want done?"), within: f::WITHIN },
            type_words(b"What do you want done?", &words),
            see(b"That's 1 character over what one message holds."),
        ],
    );
    assert_over(&world, client, b"What do you want done?", b"Send", &words);
}

#[test]
fn a_long_reply_is_answered() {
    let mut world = f::long_messages_world(f::SEED);
    let task = f::question_composer(world.front(), b"Which path?");
    let asker = world.front().engine.task(task).unwrap().delegates[0];
    let question = world.front().engine.task(asker).unwrap().questions[0].number;
    let client = f::long_messages_client(&mut world, Address::Home { project: 1 });
    let words = vec![b'r'; 8193];
    follow(
        &mut world,
        client,
        vec![
            Step::See { find: Find::named(Role::Button, b"Tell it something else"), within: f::WITHIN },
            Step::Press { find: Find::named(Role::Button, b"Tell it something else") },
            Step::See { find: Find::named(Role::TextArea, b"Or tell it something else"), within: f::WITHIN },
            type_words(b"Or tell it something else", &words),
            see(b"That's 1 character over what one message holds."),
        ],
    );
    assert_over(&world, client, b"Or tell it something else", b"Send", &words);
    let edited = vec![b'a'; 1024];
    follow(
        &mut world,
        client,
        vec![
            type_words(b"Or tell it something else", &edited),
            Step::Press { find: Find::named(Role::Button, b"Send") },
            Step::Pause { span: temper_client_domain::pages::needs::HOLD },
            Step::Go { address: f::task_page(asker) },
            see(&edited),
        ],
    );
    let answers: Vec<_> = world
        .front()
        .engine
        .store
        .rows
        .values()
        .filter_map(|record| {
            if let temper_engine_domain::Record::Core(jig_domain_core::Record::Tasks(
                jig_domain_tasks::Stored::Activity(line),
            )) = record
                && line.task == asker
                && let jig_domain_tasks::Line::Words {
                    kind: jig_domain_tasks::MessageKind::Answer { question: number },
                    words,
                    ..
                } = &line.line
                && *number == question
            {
                Some(words.as_ref())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(answers, [edited.as_slice()]);
}

#[test]
fn a_new_chats_kilobyte_reads_back_whole() {
    let mut world = first_message_world();
    world.front().reply_in_pieces(&[b"Working"]);
    let mut limits = f::whole_turn_limits();
    limits.client.domain.text = 512;
    world.front().next_client_limits(limits);
    let client = world.client_with(f::PARTY, Address::NewChat { draft: false }, limits, first_message_root);
    let words = vec![b'w'; 1024];
    follow(
        &mut world,
        client,
        vec![
            see(b"Live"),
            Step::See { find: Find::named(Role::TextArea, b"What do you want done?"), within: f::WITHIN },
            type_words(b"What do you want done?", &words),
            Step::Press { find: Find::named(Role::Button, b"Send") },
            see(&words),
        ],
    );
}

fn first_message_root(seed: u64) -> (temper_client_world::app::Root, temper_client_world::app::View) {
    let mut limits = f::whole_turn_limits();
    limits.client.domain.text = 512;
    let mut words = f::phrasebooks(&limits);
    words.shell = temper_client_view::words::Shell::Browser;
    let bounds = f::view_bounds(&limits, &words);
    (
        temper_client_world::app::Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed))),
        temper_client_world::app::View::Person(Box::new(temper_client_view::View::new(
            &bounds,
            &limits,
            words,
            f::FORGE,
        ))),
    )
}

fn first_message_world() -> Pages {
    use jig_client_kit::harness::Settings;
    use temper_engine_domain_world::{store::Store, walking};
    let mut limits = f::whole_turn_limits();
    limits.client.domain.text = 512;
    let small = Joint::with_reading_pages(f::SEED, &limits);
    let mut engine_limits = small.engine.env.limits;
    drop(small);
    engine_limits.documents.text_bytes = 512;
    engine_limits.people.words = 8192;
    engine_limits.people.spec_bytes = 8192;
    engine_limits.people.amendment_bytes = 8192;
    engine_limits.tasks.spec_bytes = 8192;
    engine_limits.tasks.message_bytes = 8192;
    engine_limits.tasks.inbox_bytes = 8192;
    engine_limits.journal.result_bytes = 8192;
    let front = Joint::with_engine(
        f::SEED,
        &limits,
        Store::with_limits(&engine_limits),
        [walking::config(f::SEED), walking::config(f::SEED)],
        &engine_limits,
        1,
    );
    let mut world = World::new(
        Settings { seed: f::SEED, words: f::decision_words(), iterations: 4096, ..Settings::default() },
        front,
    );
    world.randomness(2, 0, 0);
    world
}
