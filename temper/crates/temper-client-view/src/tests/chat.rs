use super::fixtures;
use fixtures::chat::{
    ATTEMPT, BYTES, CLOSE_GRACE, COMMITTED, GRACES, LINES, MESSAGE, PARTY, TASK, TEXT, TURN, chat_header, chat_root,
    closing, messaging_root, projected_tree as tree, proposal_root, provisional, provisional_with_text, rejecting_chat,
};
use jig_client::{contract as c, thread::Thread};
use skein_lib::{List, Wall};
use skein_ui::Role;

#[test]
fn the_playground_project_is_named_playground() {
    let root = fixtures::playground_root(temper_client_domain::Address::Chat { task: TASK }, fixtures::PROJECT, true);
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut view = fixtures::view_of(words);
        let tree = fixtures::build(&mut view, &root);
        let mut context = None;
        for (index, node) in tree.nodes().iter().enumerate() {
            if node.styles.as_slice().contains(&crate::look::CONTEXT) {
                context = Some(index);
                break;
            }
        }
        let context = context.expect("chat context");
        assert_eq!(context, styled(tree, crate::look::THREAD_INNER) + 1);
        assert!(tree.nodes()[context].styles.as_slice().contains(&crate::look::META_LINE));
        assert!(skein_lib::bytes::find(&fixtures::node_text(tree, context), "Shop · acme/shop".as_bytes()).is_some());
    }
}

#[test]
fn the_session_heading_keeps_its_name_and_shows_the_title_in_the_browser() {
    let root = fixtures::playground_root(temper_client_domain::Address::Chat { task: TASK }, fixtures::PROJECT, true);
    for (view, shown) in
        [(fixtures::page_view(&root), "T9 · Ship the update"), (fixtures::browser_page_view(&root), "Ship the update")]
    {
        let heading =
            named_position(view.tree().nodes(), Role::Heading { level: 1 }, Some("T9 · Ship the update".as_bytes()))
                .expect("heading");
        assert_eq!(fixtures::node_text(view.tree(), heading).as_ref(), shown.as_bytes());
        assert!(fixtures::person_find(view.tree(), Role::Region, "T9 · Ship the update".as_bytes()).is_none());
    }
}

#[test]
fn a_chat_proposal_is_named_from_its_source_task() {
    let view = fixtures::page_view(&proposal_root());
    assert!(fixtures::person_find(view.tree(), Role::Item, b"Proposal from T12").is_some());
}

#[test]
fn a_chats_header_says_its_status_and_spend() {
    let tree = chat_header();
    assert!(fixtures::find(&tree, Role::Status, b"Parked").is_some());
}
#[test]
fn a_held_chatbar_names_only_its_exceeded_bound() {
    use temper_client_domain::{Address, Event};
    for (reason, expected) in super::chats::HELD_STATUSES {
        let (mut root, _, watch) = fixtures::root_at(Address::Chat { task: TASK }, &fixtures::domain_limits());
        let mut task = fixtures::task::document();
        task.chip.task = TASK;
        let c::ChipPhaseFacts { phase, .. } = task.chip.details.as_mut();
        *phase = c::ChipPhase::Held { reason, revision: 1 };
        fixtures::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: watch.expect("task watch"),
                        event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                    },
                },
            },
        );
        let view = fixtures::page_view(&root);
        let mut status = None;
        for node in view.tree().nodes() {
            if node.role == Role::Status && node.name.as_deref() == Some(expected) {
                status = Some(node);
                break;
            }
        }
        let status = status.expect("held status");
        assert_eq!(status.styles.as_slice(), [crate::look::STATUS_HELD]);
        assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), expected).is_some());
    }
}

#[test]
fn a_chats_header_says_its_spend_including_delegates() {
    let tree = chat_header();
    assert!(fixtures::find(&tree, Role::Paragraph, "7 spent · of 7".as_bytes()).is_some());
}

#[test]
fn provisional_text_is_marked_until_its_turn_commits() {
    let mut thread = provisional();
    assert!(fixtures::find(&tree(&thread), Role::Status, b"Writing...").is_some());
    thread.reading = Some(jig_client::thread::Reading::Latest { known: COMMITTED.into() });
    let mut rows = List::with_capacity(TURN);
    rows.push(crate::tests::documents::turn(
        TURN.into(),
        Wall::EPOCH,
        c::TurnName { attempt: ATTEMPT, number: TURN },
        c::TurnRead::Undecodable { why: c::TurnProblem::Malformed },
    ))
    .expect("turn");
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    assert_eq!(fixtures::find(&tree(&thread), Role::Status, b"Writing..."), None);
}

#[test]
fn an_owner_activity_message_is_shown_in_the_chat() {
    let thread = owner_words_thread();
    assert!(skein_lib::bytes::find(&fixtures::text(&tree(&thread)), b"An owner message").is_some());
}

#[test]
fn an_inbox_answer_is_acknowledged_in_the_chat_conversation() {
    let mut thread =
        Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT, live: TEXT });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: COMMITTED.into() });
    let mut rows = List::with_capacity(1);
    rows.push(jig_client::contract::DecodedActivityLine {
        source: Box::new(c::ActivityLine {
            task: TASK,
            position: 1,
            at: Wall::EPOCH,
            line: c::Activity::Heard(c::Word {
                number: MESSAGE,
                from: c::Requester::Task(TASK),
                kind: c::MessageKind::QuestionAnswered { question: 42 },
                words: c::Text::new(Box::new([]), TEXT).expect("empty notice words"),
                at: Wall::EPOCH,
                hits: 1,
                eligible: true,
            }),
        }),
        decoded: None,
    })
    .expect("notice");
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    let tree = tree(&thread);
    assert!(fixtures::find(&tree, Role::Log, b"Conversation").is_some());
    assert!(fixtures::find(&tree, Role::Paragraph, b"You answered question 42 in your inbox").is_some());
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"You answered question 42 in your inbox").is_some());
}

// The chat gallery's T10: a chat started with no spec, its request a words line.
#[test]
fn a_chat_whose_spec_is_empty_draws_no_opening_turn() {
    let thread = owner_words_thread();
    let spec = |words: &[u8]| {
        c::SpecificationDocument::complete(c::Specification {
            asks: false,
            words: c::Text::new(Box::from(words), TEXT).expect("spec words"),
            parameters: List::with_capacity(0),
            inputs: List::with_capacity(0),
        })
    };
    let person_turns = |tree: &skein_ui::Tree| {
        let mut count = 0_u32;
        for node in tree.nodes() {
            if node.styles.as_slice().contains(&crate::look::BY_PERSON) {
                count = count.checked_add(1).expect("person turn count");
            }
        }
        count
    };
    assert_eq!(person_turns(&fixtures::chat::requested_tree(&thread, &spec(b"A request"))), 2, "the request opens");
    let tree = fixtures::chat::requested_tree(&thread, &spec(b""));
    assert_eq!(person_turns(&tree), 1, "the words line alone");
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"An owner message").is_some());
}

fn owner_words_thread() -> Thread {
    let mut thread =
        Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT, live: TEXT });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: COMMITTED.into() });
    let mut rows = List::with_capacity(TURN);
    rows.push(jig_client::contract::DecodedActivityLine {
        source: Box::new(c::ActivityLine {
            task: TASK,
            position: TURN.into(),
            at: Wall::EPOCH,
            line: c::Activity::Words {
                message: MESSAGE,
                by: c::Requester::Person(PARTY),
                kind: c::MessageKind::Words,
                words: c::Text::new(Box::from(*b"An owner message"), TEXT).expect("words"),
            },
        }),
        decoded: None,
    })
    .expect("line");
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    thread
}

#[test]
fn the_close_confirmation_names_each_live_delegates_fate() {
    let tree = closing(Some(CLOSE_GRACE));
    let text = fixtures::text(&tree);
    assert!(skein_lib::bytes::find(&text, b"Delegate 12 will be cancelled").is_some());
    assert!(skein_lib::bytes::find(&text, b"Delegate 13 will be moved to you").is_some());
}
#[test]
fn the_close_confirmation_offers_each_live_delegates_choice() {
    let tree = closing(Some(CLOSE_GRACE));
    assert!(fixtures::find(&tree, Role::Button, b"Cancel delegate 12").is_some());
    assert!(fixtures::find(&tree, Role::Button, b"Take delegate 13").is_some());
}
#[test]
fn the_close_confirmation_says_how_the_chat_ends() {
    use c::RunStage as S;
    use skein_lib::Duration;
    let grace = Some(Duration::from_secs(30));
    for (name, stage, delegate, grace, expected) in [
        ("at rest", S::Idle, false, grace, "Close chat now"),
        ("at rest with delegate", S::Idle, true, grace, "Close chat now"),
        ("preparing", S::Preparing, false, grace, "Close chat now"),
        (
            "claimed",
            S::Claimed { attempt: ATTEMPT },
            false,
            grace,
            "Close chat: allow 30 seconds to finish, then cancel",
        ),
        (
            "running",
            S::Running { attempt: ATTEMPT },
            false,
            grace,
            "Close chat: allow 30 seconds to finish, then cancel",
        ),
        ("running without grace", S::Running { attempt: ATTEMPT }, false, None, "Close chat now"),
    ] {
        let source = chat_root();
        let mut me = source.pages().me(source.client()).unwrap().clone();
        me.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap().grace = grace;
        let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
        let c::ChipPhaseFacts { phase, .. } = task.chip.details.as_mut();
        *phase = c::ChipPhase::Active(stage);
        task.delegates = List::with_capacity(1);
        if delegate {
            let mut chip = task.chip.clone();
            chip.task = 7;
            task.delegates.push(c::LiveDelegate { chip, takeover_budget: 0 }).unwrap();
        }
        let (mut root, me_watch, watch) =
            fixtures::root_at(temper_client_domain::Address::Chat { task: TASK }, &fixtures::domain_limits());
        for (watch, delivery) in
            [(me_watch, c::Delivery::Me(Box::new(me))), (watch.unwrap(), c::Delivery::Task(Box::new(task)))]
        {
            deliver_chat(&mut root, watch, delivery);
        }
        fixtures::root_event(
            &mut root,
            temper_client_domain::Event::Act {
                action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Close),
            },
        );
        let view = fixtures::page_view(&root);
        let text = fixtures::text(view.tree());
        if expected == "Close chat now" {
            assert!(skein_lib::bytes::find(&text, b"Close chat: allow").is_none(), "{name}");
        }
        let mut expected_text = List::with_capacity(3);
        if delegate {
            expected_text.push(b"Delegate 7 will be cancelled".as_slice()).unwrap();
        }
        expected_text.push(expected.as_bytes()).unwrap();
        expected_text.push(b"A closed chat can't be reopened.".as_slice()).unwrap();
        let mut remaining = text.as_ref();
        for line in &expected_text {
            let Some(at) = skein_lib::bytes::find(remaining, line) else { panic!("{name}: missing {line:?}") };
            remaining = &remaining[at.checked_add(line.len()).unwrap()..];
        }
    }
    for (grace, expected) in GRACES {
        assert!(skein_lib::bytes::find(&fixtures::text(&closing(grace)), expected).is_some(), "{grace:?}");
    }
}
fn deliver_chat(root: &mut temper_client_domain::Domain, watch: skein_lib::Token, delivery: c::Delivery) {
    fixtures::root_event(
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) },
            },
        },
    );
}
fn messaging_task(task: c::Task) -> temper_client_domain::Domain {
    let (mut root, _, watch) =
        fixtures::root_at(temper_client_domain::Address::Chat { task: TASK }, &fixtures::domain_limits());
    deliver_chat(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(task)));
    root
}
#[test]
fn an_open_chat_at_rest_says_waiting_on_your_words_and_keeps_its_composer() {
    let source = messaging_root();
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    task.flags.ended_by = c::EndedBy::Requester;
    task.waits_on = c::WaitsOn::News;
    task.wake.words = c::WakeRule::Immediate;
    task.requester = c::Requester::Person(PARTY);
    let c::ChipPhaseFacts { phase, details } = task.chip.details.as_mut();
    *phase = c::ChipPhase::Active(c::RunStage::Idle);
    details.live_delegates = 0;
    let view = fixtures::page_view(&messaging_task(task));
    assert!(fixtures::find(view.tree(), Role::Status, b"waiting on your words").is_some());
    assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_some());
    assert!(fixtures::find(view.tree(), Role::Button, b"Send").is_some());
}

#[test]
fn pressing_close_opens_the_chat_confirmation() {
    let mut root = chat_root();
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Close),
        },
    );
    assert!(root.pages().confirmation().is_some(), "Close prepared the compound request");
    let mut view = fixtures::view();
    assert!(fixtures::find(fixtures::build(&mut view, &root), Role::Dialog, b"Close").is_some());
}

#[test]
fn the_close_confirmation_says_the_cancellations_facts() {
    let mut root = chat_root();
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Close),
        },
    );
    let mut view = fixtures::view();
    let said = fixtures::text(fixtures::build(&mut view, &root));
    let dangerous =
        fixtures::words().say_word(jig_client::words::Word::Decision(jig_client::words::DecisionWord::Dangerous), &[]);
    assert!(skein_lib::bytes::find(&said, &dangerous).is_some(), "irreversible cancellation");
    let impact = fixtures::words().say_word(
        jig_client::words::Word::Decision(jig_client::words::DecisionWord::TasksRunning),
        &[skein_ui::phrases::Hole::Number(1)],
    );
    assert!(skein_lib::bytes::find(&said, &impact).is_some(), "supplied running-task impact");
}

#[test]
fn an_unanchored_proposal_is_shown_once_in_the_conversation_tail() {
    let root = proposal_root();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    let mut conversation_end = None;
    let mut accept = None;
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.role == Role::Log {
            conversation_end = Some(index + usize::try_from(node.size).expect("log size"));
        }
        if node.role == Role::Button && node.name.as_deref() == Some(b"Accept".as_slice()) {
            accept = Some(index);
        }
    }
    assert_eq!(named_count(tree, Role::Button, b"Accept"), 1);
    assert!(
        accept.expect("proposal's accept act") < conversation_end.expect("conversation"),
        "card inside conversation"
    );
}
#[test]
fn a_chat_proposal_offers_rejection_with_a_reason() {
    let root = proposal_root();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Button, b"Reject").is_some());
}
#[test]
fn rejecting_in_the_chat_shows_the_reason_in_its_confirmation() {
    let root = rejecting_chat();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(fixtures::find(tree, Role::Dialog, b"Reject with reason").is_some());
}
#[test]
fn a_chat_rejection_confirmation_keeps_the_persons_reason() {
    let root = rejecting_chat();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(skein_lib::bytes::find(&fixtures::text(tree), b"Too costly").is_some());
}

const LONG_PROVISIONAL: &[u8] = b"1234567890123456789012345678901\xc3\xa9 and more";
#[test]
fn a_long_provisional_text_says_nothing_of_what_was_cut() {
    let thread = provisional_with_text(LONG_PROVISIONAL);
    for drawn in [fixtures::chat::projected_tree(&thread), fixtures::chat::tree(&thread)] {
        let said = fixtures::text(&drawn);
        assert!(skein_lib::bytes::find(&said, b"Earlier text omitted").is_none());
        assert!(skein_lib::bytes::find(&said, b"1234567890123456789012345678901").is_some());
        assert!(skein_lib::bytes::find(&said, "…".as_bytes()).is_none());
    }
}

fn conversation_turn(messages: &[c::Message], cut: bool) -> Thread {
    let mut kept = List::with_capacity(u32::try_from(messages.len()).expect("message count"));
    for message in messages {
        kept.push(message.clone()).expect("message");
    }
    let turn = c::Turn {
        messages: kept,
        used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
        spent: 0,
        read_through: None,
        cut,
    };
    let mut rows = List::with_capacity(1);
    rows.push(super::documents::turn(
        1,
        Wall::EPOCH,
        c::TurnName { attempt: ATTEMPT, number: TURN },
        c::TurnRead::Decoded(turn),
    ))
    .expect("turn");
    let mut thread =
        Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT, live: TEXT });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    thread
}

#[test]
fn a_cut_committed_turn_says_nothing_of_what_was_cut() {
    use fixtures::first_run as run;
    let thread = conversation_turn(
        &[run::message(c::Speaker::Assistant, &[c::Block::Text(run::text("Kept…".as_bytes()))])],
        true,
    );
    for drawn in [fixtures::chat::projected_tree(&thread), fixtures::chat::tree(&thread)] {
        let said = fixtures::text(&drawn);
        assert!(skein_lib::bytes::find(&said, b"Earlier text omitted").is_none());
        assert!(skein_lib::bytes::find(&said, "Kept…".as_bytes()).is_some());
    }
}

#[test]
fn a_turns_opaque_blocks_are_not_drawn() {
    use fixtures::first_run as run;
    let thread = conversation_turn(
        &[run::message(
            c::Speaker::Assistant,
            &[c::Block::Opaque { dialect: run::text(b"3"), size: 2164 }, c::Block::Text(run::text(b"Kept words"))],
        )],
        false,
    );
    for drawn in [fixtures::chat::projected_tree(&thread), fixtures::chat::tree(&thread)] {
        let said = fixtures::text(&drawn);
        assert!(skein_lib::bytes::find(&said, b"Content in").is_none());
        assert!(skein_lib::bytes::find(&said, b"bytes").is_none());
        assert!(skein_lib::bytes::find(&said, b"Kept words").is_some());
    }
}

#[test]
fn a_message_of_opaque_blocks_alone_draws_no_message() {
    use fixtures::first_run as run;
    for speaker in [c::Speaker::Assistant, c::Speaker::Person, c::Speaker::Tool, c::Speaker::System] {
        let thread = conversation_turn(
            &[run::message(speaker, &[c::Block::Opaque { dialect: run::text(b"3"), size: 2164 }])],
            false,
        );
        for drawn in [fixtures::chat::projected_tree(&thread), fixtures::chat::tree(&thread)] {
            assert!(style_position(drawn.nodes(), crate::look::MESSAGE).is_none(), "opaque message has no shell");
            assert!(fixtures::text(&drawn).is_empty(), "opaque message has no words");
        }
    }
}

#[test]
fn an_agents_result_line_is_its_first_line() {
    use fixtures::first_run as run;
    for summary in [b"first\nsecond".as_slice(), b"\n\r\n \t\r\nfirst\r\nsecond"] {
        let thread = conversation_turn(
            &[run::message(
                c::Speaker::Tool,
                &[c::Block::Result { call: 1, summary: run::text(summary), failed: false }],
            )],
            false,
        );
        let drawn = fixtures::chat::tree(&thread);
        assert!(fixtures::find(&drawn, Role::Paragraph, b"first").is_some());
        assert!(skein_lib::bytes::find(&fixtures::text(&drawn), b"second").is_none());
    }
}

#[test]
fn a_conversation_past_its_rows_says_nothing_of_what_was_cut() {
    use fixtures::first_run as run;
    let block = c::Block::Text(run::text(b"Kept"));
    let mut blocks = List::with_capacity(LINES + 1);
    let mut messages = List::with_capacity(LINES + 1);
    for _ in 0..=LINES {
        blocks.push(block.clone()).expect("block");
        messages.push(run::message(c::Speaker::Assistant, core::slice::from_ref(&block))).expect("message");
    }
    for thread in [
        conversation_turn(&[run::message(c::Speaker::Assistant, blocks.as_slice())], false),
        conversation_turn(messages.as_slice(), false),
    ] {
        for drawn in [fixtures::chat::projected_tree(&thread), fixtures::chat::tree(&thread)] {
            assert!(skein_lib::bytes::find(&fixtures::text(&drawn), b"Earlier text omitted").is_none());
            assert_eq!(skein_lib::bytes::count(&fixtures::text(&drawn), b"Kept", LINES + 1), LINES);
        }
    }
}

#[test]
fn a_cut_request_says_nothing_of_what_was_cut() {
    let thread = owner_words_thread();
    let mut request = c::SpecificationDocument::complete(c::Specification {
        asks: false,
        words: fixtures::first_run::text(b"Kept request"),
        parameters: List::with_capacity(0),
        inputs: List::with_capacity(0),
    });
    request.specification.words.cut = true;
    let drawn = fixtures::chat::requested_tree(&thread, &request);
    assert!(fixtures::find(&drawn, Role::Paragraph, b"Kept request").is_some());
    assert!(skein_lib::bytes::find(&fixtures::text(&drawn), b"Earlier text omitted").is_none());
}

#[test]
fn the_message_composer_keeps_its_name_and_has_a_numbered_placeholder() {
    let view = fixtures::page_view(&messaging_root());
    let id = fixtures::person_find(view.tree(), Role::TextArea, b"Message").expect("composer named Message");
    let composer = view.tree().find(id).expect("composer node");
    assert_eq!(composer.name.as_deref(), Some(b"Message".as_slice()));
    // messaging_root's chat is TASK, T9.
    assert_eq!(composer.placeholder.as_deref(), Some(b"Reply to T9".as_slice()));
}

#[test]
fn enter_in_the_message_composer_sends() {
    let mut view = fixtures::page_view(&messaging_root());
    let composer = fixtures::find(view.tree(), Role::TextArea, b"Message").expect("composer");
    assert_eq!(
        crate::act(&mut view, composer, skein_ui::Act::Submit),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Send)),
    );
}
#[test]
fn a_line_break_counts_within_the_message_bound() {
    let mut root = messaging_root();
    let bound = usize::try_from(temper_client_protocol_wire::CUE_BYTES).expect("bound fits");
    let mut over = skein_lib::bytes::zeroed(bound.checked_add(1).unwrap());
    over.fill(b'a');
    over[3] = b'\n';
    let words = &over[..bound];
    assert_eq!(&*fixtures::type_into(&mut root, b"Message", words), words);
    assert_eq!(&*fixtures::type_into(&mut root, b"Message", &over), words);
}
#[test]
fn enter_in_a_cards_reason_does_nothing() {
    let mut view = fixtures::page_view(&proposal_root());
    let reason = fixtures::find(view.tree(), Role::TextArea, b"Reason").expect("reason");
    assert_eq!(crate::act(&mut view, reason, skein_ui::Act::Submit), None);
}
#[test]
fn a_chats_card_carries_the_card_token() {
    let view = fixtures::page_view(&proposal_root());
    let cards = fixtures::styles(view.tree(), Role::Item);
    assert!(!cards.is_empty());
    for card in &cards {
        assert!(card.as_ref().contains(&crate::look::CARD));
        assert!(card.as_ref().contains(&crate::look::KIND_PROPOSAL));
    }
}
#[test]
fn each_turn_carries_the_turn_token() {
    let tree = tree(&provisional());
    let turns = fixtures::styles(&tree, Role::Region);
    assert_eq!(turns.len(), 1);
    assert_eq!(turns.as_ref(), [Box::from([crate::look::TURN])]);
}

fn style_position(nodes: &[skein_ui::Node], token: skein_ui::Style) -> Option<usize> {
    for (index, node) in nodes.iter().enumerate() {
        if node.styles.as_slice().contains(&token) {
            return Some(index);
        }
    }
    None
}
pub(super) fn named_position(nodes: &[skein_ui::Node], role: Role, name: Option<&[u8]>) -> Option<usize> {
    for (index, node) in nodes.iter().enumerate() {
        if node.role == role && (name.is_none() || node.name.as_deref() == name) {
            return Some(index);
        }
    }
    None
}
fn styled(tree: &skein_ui::Tree, token: skein_ui::Style) -> usize {
    style_position(tree.nodes(), token).expect("styled node")
}
fn subtree(tree: &skein_ui::Tree, at: usize) -> &[skein_ui::Node] {
    let from = at.checked_add(1).expect("child index");
    let through = at.checked_add(usize::try_from(tree.nodes()[at].size).expect("size")).expect("subtree end");
    &tree.nodes()[from..through]
}
#[test]
fn the_chat_bar_contains_identity_spend_and_acts() {
    use temper_client_domain::Event;
    let root = messaging_root();
    // The standard task fixture offers Close; a checked task delivery adds Stop.
    let mut task = root.pages().chat().expect("chat").task(root.client()).expect("task").clone();
    let mut offers = List::with_capacity(3);
    for act in [c::ActKind::Message, c::ActKind::Stop, c::ActKind::Cancel] {
        offers.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).expect("offer");
    }
    task.acts = offers;
    // Re-enter with a task watch whose token is supplied by the fixture.
    let (mut fresh, _, watch) =
        fixtures::root_at(temper_client_domain::Address::Chat { task: TASK }, &fixtures::domain_limits());
    fixtures::root_event(
        &mut fresh,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("task watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
    );
    let view = fixtures::page_view(&fresh);
    let bar = styled(view.tree(), crate::look::SESSION_TOP);
    assert_eq!(view.tree().nodes()[bar].role, Role::Slot);
    let nodes = subtree(view.tree(), bar);
    let heading = fixtures::words().say(
        crate::words::Word::ChatTitle,
        &[
            skein_ui::phrases::Hole::Number(9),
            skein_ui::phrases::Hole::Name(skein_lib::bytes::copy_of(fixtures::task::document().chip.title.bytes())),
        ],
    );
    for (role, name) in [
        (Role::Heading { level: 1 }, heading.as_ref()),
        (Role::Paragraph, "7 spent · of 7".as_bytes()),
        (Role::Status, b"Parked"),
        (Role::Button, b"Stop"),
        (Role::Button, b"Close"),
    ] {
        assert!(named_position(nodes, role, Some(name)).is_some(), "{name:?}");
    }
    assert!(named_position(nodes, Role::Paragraph, Some(fixtures::task::document().chip.title.bytes())).is_none());
    assert!(fixtures::find(view.tree(), Role::Button, b"Mark seen").is_none());
}
#[test]
fn the_chat_entry_has_the_shared_hint_after_message_and_send() {
    let view = fixtures::page_view(&messaging_root());
    let tree = view.tree();
    let area = styled(tree, crate::look::SESSION_FOOT);
    let entry = styled(tree, crate::look::SESSION_COMPOSER);
    assert_eq!(tree.nodes()[area].role, Role::Form);
    assert!(entry > area);
    let nodes = subtree(tree, entry);
    assert_eq!(nodes[0].role, Role::TextArea);
    assert!(nodes[0].states.as_slice().contains(&skein_ui::State::SubmitsOnEnter));
    assert_eq!(nodes[1].role, Role::Button);
    assert_eq!(nodes[1].name.as_deref(), Some(b"Send".as_slice()));
    let hint = styled(tree, crate::look::SESSION_HINT);
    assert_eq!(hint, entry + usize::try_from(tree.nodes()[entry].size).unwrap());
    assert_eq!(tree.nodes()[hint].name.as_deref(), Some("Enter sends · Shift+Enter for a new line".as_bytes()));
}

#[test]
fn a_chat_without_message_has_no_composer_area() {
    let view = fixtures::page_view(&chat_root());
    assert!(style_position(view.tree().nodes(), crate::look::SESSION_FOOT).is_none());
}
#[test]
fn chat_cards_are_items_inside_the_logs_column_after_the_turns() {
    let view = fixtures::page_view(&proposal_root());
    let tree = view.tree();
    let cards = styled(tree, crate::look::CARDS);
    assert_eq!(tree.nodes()[cards].role, Role::List);
    let card = style_position(subtree(tree, cards), crate::look::CARD).expect("card");
    assert_eq!(subtree(tree, cards)[card].role, Role::Item);
    let log = named_position(tree.nodes(), Role::Log, None).expect("log");
    let inner = styled(tree, crate::look::THREAD_INNER);
    assert_eq!(inner, log + 1);
    assert!(cards > inner && cards < log + usize::try_from(tree.nodes()[log].size).unwrap());
}

fn structured_messages() -> List<c::Message> {
    let mut messages = List::with_capacity(4);
    for speaker in [c::Speaker::Person, c::Speaker::Assistant, c::Speaker::Tool, c::Speaker::System] {
        let mut blocks = List::with_capacity(2);
        blocks
            .push(c::Block::Call(c::Call {
                number: 1,
                tool: c::Text::new(Box::from(*b"Read"), TEXT).expect("tool"),
                input: c::Text::new(Box::from([]), TEXT).expect("input"),
                outcome: None,
                failed: false,
                objects: List::with_capacity(0),
                parts: List::with_capacity(0),
            }))
            .expect("call");
        blocks
            .push(c::Block::Result {
                call: 1,
                summary: c::Text::new(Box::from(*b"Done"), TEXT).expect("summary"),
                failed: false,
            })
            .expect("result");
        messages.push(c::Message { speaker, blocks }).expect("message");
    }
    messages
}
pub(super) fn structured_thread() -> Thread {
    let mut thread = Thread::new(TASK, &jig_client::thread::Limits { lines: 12, bytes: BYTES, text: 128, live: 128 });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    let messages = structured_messages();
    let mut rows = List::with_capacity(5);
    for (position, event) in [
        (
            5,
            crate::tests::documents::line(
                1,
                Wall::EPOCH,
                jig_client::contract::Activity::RunEnded {
                    attempt: ATTEMPT,
                    end: jig_client::contract::RunEnd::Finished,
                    spent: 0,
                },
            ),
        ),
        (
            4,
            crate::tests::documents::turn(
                1,
                Wall::EPOCH,
                c::TurnName { attempt: ATTEMPT, number: 1 },
                c::TurnRead::Decoded(c::Turn {
                    messages,
                    used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                    spent: 0,
                    read_through: None,
                    cut: false,
                }),
            ),
        ),
        (
            3,
            crate::tests::documents::line(
                1,
                Wall::EPOCH,
                jig_client::contract::Activity::Words {
                    message: 1,
                    by: jig_client::contract::Requester::Person(
                        (c::Party {
                            number: PARTY,
                            kind: c::PartyKind::Person,
                            name: c::Text::new(Box::from(*b"Pat"), TEXT).expect("person"),
                        })
                        .number,
                    ),
                    kind: jig_client::contract::MessageKind::Words,
                    words: c::Text::new(Box::from(*b"Hello"), TEXT).expect("words"),
                },
            ),
        ),
        (
            2,
            crate::tests::documents::line(
                1,
                Wall::EPOCH,
                jig_client::contract::Activity::RunBegan {
                    attempt: ATTEMPT,
                    why: jig_client::contract::RunWhy::Made,
                    resumed: false,
                },
            ),
        ),
    ] {
        rows.push({
            let mut line = event;
            line.source.position = position;
            line.source.at = Wall::EPOCH;
            line
        })
        .expect("line");
    }
    thread.page(c::ActivityPage {
        rows,
        after: None,
        before: Some(c::Cursor::new(Box::from(*b"older"), TEXT).expect("cursor")),
        cut: true,
    });
    thread.left = 1;
    thread
}
#[test]
fn person_words_keep_their_shell_and_only_assistant_calls_keep_work() {
    use crate::look;
    let tree = tree(&structured_thread());
    {
        let at = styled(&tree, look::BY_PERSON);
        assert_eq!(tree.nodes()[at].role, Role::Slot);
        assert!(tree.nodes()[at].styles.as_slice().contains(&look::MESSAGE));
        let nodes = subtree(&tree, at);
        let who = style_position(nodes, look::WHO).expect("who");
        let says = style_position(nodes, look::SAYS).expect("says");
        assert!(who < says);
        assert_eq!(nodes[who].role, Role::Slot);
        assert_eq!(nodes[says].role, Role::Slot);
    }
    let mut works = 0_u32;
    for (at, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&look::WORK) {
            if node.role == Role::Slot {
                assert!(named_position(subtree(&tree, at), Role::Paragraph, None).is_some());
            } else {
                assert_eq!(node.role, Role::Disclosure);
            }
            works = works.checked_add(1).expect("work count");
        }
    }
    assert_eq!(works, 2);
    for hidden in [look::BY_TOOL, look::BY_SYSTEM, look::BY_AGENT] {
        assert!(style_position(tree.nodes(), hidden).is_none(), "inputs and call-only assistant have no shell");
    }
    assert_call_fold(&tree, b"1 tool call", b"Read", &[b"Read"]);
}
#[test]
fn only_omission_markers_keep_the_turn_region() {
    let tree = tree(&structured_thread());
    let mut markers = 0_u32;
    for node in tree.nodes() {
        if node.styles.as_slice().contains(&crate::look::MARKER) {
            assert_eq!(node.role, Role::Region);
            assert!(node.styles.as_slice().contains(&crate::look::TURN));
            markers = markers.checked_add(1).expect("marker count");
        }
    }
    assert_eq!(markers, 1);
}

fn chat_event(
    root: &mut temper_client_domain::Domain,
    event: temper_client_domain::Event,
    out: &mut skein_lib::Queue<temper_client_domain::Request>,
) {
    temper_client_domain::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: Wall::EPOCH, limits: fixtures::domain_limits() },
        event,
        out,
    );
}
fn chat_with_history() -> temper_client_domain::Domain {
    history().0
}
fn history() -> (temper_client_domain::Domain, skein_lib::Queue<temper_client_domain::Request>) {
    use temper_client_domain::{Address, Event, Place, Request};
    let (mut root, _, _) = fixtures::root_at(
        Address::Inbox { filter: temper_client_domain::address::Filter::ALL },
        &fixtures::domain_limits(),
    );
    let mut out = skein_lib::Queue::uncapped();
    chat_event(&mut root, Event::Went { place: Place::Address(Address::Chat { task: TASK }) }, &mut out);
    let mut watch = None;
    let mut activity = None;
    while let Some(request) = out.pop() {
        match request {
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Open {
                                watch: token,
                                subject: jig_client::Watchable::Jig(c::Subject::Task { .. }),
                            },
                    },
            } => watch = Some(token),
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Activity { .. }) },
                    },
            } => activity = Some(read),
            Request::Client { .. }
            | Request::Unlinked { .. }
            | Request::Show { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Fact { .. } => {}
        }
    }
    let template = messaging_root();
    let mut task = template.pages().chat().expect("chat").task(template.client()).expect("task").clone();
    task.activity = 4;
    chat_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
        &mut out,
    );
    let mut rows = List::with_capacity(2);
    rows.push(jig_client::contract::DecodedActivityLine {
        source: Box::new(c::ActivityLine {
            task: TASK,
            position: 3,
            at: Wall::EPOCH,
            line: c::Activity::Turn { reply: 1, attempt: ATTEMPT, turn: TURN, read: None },
        }),
        decoded: None,
    })
    .expect("owner turn");
    rows.push(crate::tests::documents::line(
        4,
        Wall::EPOCH,
        jig_client::contract::Activity::Words {
            message: 1,
            by: jig_client::contract::Requester::Person(
                (c::Party {
                    number: PARTY,
                    kind: c::PartyKind::Person,
                    name: c::Text::new(Box::from(*b"Pat"), TEXT).expect("name"),
                })
                .number,
            ),
            kind: jig_client::contract::MessageKind::Words,
            words: c::Text::new(Box::from(*b"Hello"), TEXT).expect("words"),
        },
    ))
    .expect("line");
    chat_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read: activity.expect("activity"),
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(
                        c::ActivityPage {
                            rows,
                            before: Some(c::Cursor::new(Box::from(*b"older"), TEXT).expect("cursor")),
                            after: None,
                            cut: false,
                        },
                    )))),
                },
            },
        },
        &mut out,
    );
    (root, out)
}
#[test]
fn older_is_not_offered_while_the_window_is_filling() {
    let view = fixtures::page_view(&chat_with_history());
    let nodes = subtree(view.tree(), styled(view.tree(), crate::look::THREAD));
    assert_eq!(nodes[0].role, Role::Log);
    assert!(nodes[1].styles.as_slice().contains(&crate::look::THREAD_INNER));
    assert!(fixtures::find(view.tree(), Role::Button, b"Older").is_none());
}
fn answered_message(answer: c::Answer) -> temper_client_domain::Domain {
    use temper_client_domain::{Action, Event, Request, pages::Act};
    let (mut root, mut out) = history();
    let mut seen = None;
    while let Some(request) = out.pop() {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Send { attempt, request: jig_client::Ask::Jig(ask), .. },
                },
        } = request
            && let c::Ask::Seen { .. } = *ask
        {
            seen = Some(attempt);
        }
    }
    let seen = seen.expect("history already emitted automatic Seen");
    for act in [Act::MessageWords { words: Box::from(*b"Hello") }, Act::Send] {
        chat_event(&mut root, Event::Act { action: Action::Pages(act) }, &mut out);
    }
    assert_eq!(root.pages().chat().expect("chat").pending.len(), 2);
    let pending = fixtures::page_view(&root);
    assert!(fixtures::find(pending.tree(), Role::Status, b"Being sent").is_some());
    let thread = styled(pending.tree(), crate::look::THREAD);
    let area = styled(pending.tree(), crate::look::SESSION_FOOT);
    let log = named_position(pending.tree().nodes(), Role::Log, None).expect("log");
    let end = log.checked_add(usize::try_from(pending.tree().nodes()[log].size).expect("log size")).expect("log end");
    assert!(named_position(&pending.tree().nodes()[end..area], Role::Status, None).is_none());
    assert!(thread < log && end <= area, "thread={thread} log={log} end={end} area={area}");
    chat_event(&mut root, Event::Client { event: jig_client::Event::Stored }, &mut out);
    let mut attempt = None;
    for request in &out {
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt: token, .. } },
        } = request
        {
            attempt = Some(*token);
        }
    }
    let attempt = attempt.expect("attempt");
    chat_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(answer)),
                    why: None,
                },
            },
        },
        &mut out,
    );
    chat_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: seen,
                    envelope: skein_lib::RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                        c::DocumentOutcome::Amended { task: 9 },
                    )))),
                    why: None,
                },
            },
        },
        &mut out,
    );
    root
}
#[test]
fn a_messages_progress_and_acceptance_stay_in_the_conversation() {
    let root = answered_message(c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Said {
        task: TASK,
        message: MESSAGE,
    })));
    let answered = fixtures::page_view(&root);
    assert!(fixtures::find(answered.tree(), Role::Status, b"Sent").is_some());
    let applied = fixtures::words().say_card(crate::cards::P::Applied, &[]);
    assert!(named_position(answered.tree().nodes(), Role::Status, Some(&applied)).is_none());
}
#[test]
fn a_refused_message_draws_its_refusal_after_send() {
    let root = answered_message(c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
    let view = fixtures::page_view(&root);
    let nodes = view.tree().nodes();
    let send = named_position(nodes, Role::Button, Some(b"Send")).unwrap();
    let status = named_position(nodes, Role::Status, Some(b"This act is not offered")).unwrap();
    assert!(send < status);
    let area = styled(view.tree(), crate::look::SESSION_FOOT);
    assert!(status < area + usize::try_from(nodes[area].size).unwrap());
}

fn header_with_status(waits_on: c::WaitsOn, live: u32, wake: c::WakeRule, requester: c::Requester) -> crate::View {
    use temper_client_domain::{Address, Event};
    let (mut root, _, watch) = fixtures::root_at(Address::Chat { task: TASK }, &fixtures::domain_limits());
    let mut task = fixtures::task::document();
    task.chip.task = TASK;
    task.waits_on = waits_on;
    task.requester = requester;
    task.wake.words = wake;
    let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
    details.live_delegates = live;
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("task watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
    );
    fixtures::page_view(&root)
}

#[test]
fn the_chatbar_counts_the_delegates_it_waits_on() {
    // One and two distinguish the shared goal phrase's plural forms.
    for (live, expected) in [(1, b"waiting on 1 delegate".as_slice()), (2, b"waiting on 2 delegates")] {
        let view = header_with_status(c::WaitsOn::Delegates, live, c::WakeRule::Never, c::Requester::Person(PARTY));
        let status = fixtures::find(view.tree(), Role::Status, expected).expect("delegate status");
        assert_eq!(
            view.tree().find(status).unwrap().styles.as_slice(),
            [crate::look::STATUS_WAITING, crate::look::STATE_NEEDS]
        );
    }
}

#[test]
fn the_chatbar_waits_on_the_requesting_persons_words() {
    // Batching also wakes on words, after its configured count or age.
    const BATCH_COUNT: u32 = 2;
    let person = fixtures::root_in_project(None);
    let requester = person.pages().me(person.client()).unwrap().party.number;
    for rule in
        [c::WakeRule::Immediate, c::WakeRule::Batch { count: BATCH_COUNT, age: skein_lib::Duration::from_secs(1) }]
    {
        let view = header_with_status(c::WaitsOn::News, 0, rule, c::Requester::Person(requester));
        let status = fixtures::find(view.tree(), Role::Status, b"waiting on your words").expect("words status");
        assert_eq!(
            view.tree().find(status).unwrap().styles.as_slice(),
            [crate::look::STATUS_WAITING, crate::look::STATE_NEEDS]
        );
    }
}

#[test]
fn the_chatbar_keeps_jigs_word_without_the_requesters_words_wake() {
    const OTHER_PERSON: u64 = 74;
    let person = fixtures::root_in_project(None);
    let requester = person.pages().me(person.client()).unwrap().party.number;
    for (rule, by) in [
        (c::WakeRule::Never, c::Requester::Person(requester)),
        (c::WakeRule::Immediate, c::Requester::Person(OTHER_PERSON)),
        (c::WakeRule::Immediate, c::Requester::Task(TASK)),
    ] {
        let view = header_with_status(c::WaitsOn::News, 0, rule, by);
        assert!(fixtures::find(view.tree(), Role::Status, b"Parked").is_some());
    }
}

fn assistant_blocks(blocks: &[c::Block]) -> skein_ui::Tree {
    use fixtures::first_run as run;
    let mut thread =
        Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: 512, live: 512 });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    let mut rows = List::with_capacity(1);
    rows.push(run::turn(1, ATTEMPT, 1, &[run::message(c::Speaker::Assistant, blocks)], 512)).expect("turn");
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    tree(&thread)
}
#[test]
fn an_agents_pipe_table_keeps_its_rows_cells_and_empty_header() {
    let tree =
        assistant_blocks(&[c::Block::Text(fixtures::first_run::text(b"| | |\n|---|---|\n| Harness | Running |"))]);
    let message = styled(&tree, crate::look::BY_AGENT);
    let says = style_position(subtree(&tree, message), crate::look::SAYS).expect("agent's words");
    let words = subtree(&tree, message + 1 + says);
    let table = named_position(words, Role::Table, None).expect("table in agent's turn");
    let table_at = message + 1 + says + 1 + table;
    let nodes = subtree(&tree, table_at);
    let mut rows = 0_u32;
    let mut headers = 0_u32;
    let mut cells = List::with_capacity(2);
    for (offset, node) in nodes.iter().enumerate() {
        if node.role == Role::Row {
            rows += 1;
        }
        if node.role == Role::ColumnHeader {
            headers += 1;
            assert_eq!(node.size, 1, "empty header has no child");
            if let Some(name) = &node.name {
                assert!(name.is_empty());
            }
            assert!(node.text.is_empty());
        }
        if node.role == Role::Cell {
            cells.push(offset).expect("two body cells");
        }
    }
    assert_eq!(rows, 2);
    assert_eq!(headers, 2);
    assert_eq!(cells.len(), 2);
    for (offset, expected) in cells.iter().zip([b"Harness".as_slice(), b"Running"]) {
        assert_eq!(fixtures::node_text(&tree, table_at + 1 + offset).as_ref(), expected);
    }
}
#[test]
fn repeated_calls_fold_in_first_name_order_with_each_outcome_inside() {
    use fixtures::first_run as run;
    let tree = assistant_blocks(&[
        run::call(1, b"read", b"{}", Some(b"Read the first file")),
        run::call(2, b"wait", b"{}", None),
        run::call(3, b"read", b"{}", Some(b"Read the second file")),
    ]);
    assert_call_fold(
        &tree,
        b"3 tool calls",
        "read ×2 · wait".as_bytes(),
        &[b"readRead the first file", b"wait", b"readRead the second file"],
    );
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Temper").is_none());
}
#[test]
fn a_fold_names_two_repeated_calls_and_two_distinct_calls() {
    use fixtures::first_run as run;
    for (tool, description) in [(b"read".as_slice(), "read ×2"), (b"edit".as_slice(), "read · edit")] {
        let tree = assistant_blocks(&[run::call(1, b"read", b"{}", None), run::call(2, tool, b"{}", None)]);
        assert_call_fold(&tree, b"2 tool calls", description.as_bytes(), &[b"read", tool]);
    }
}
#[test]
fn a_tool_name_using_the_whole_turn_budget_fits_its_fold_description() {
    use fixtures::first_run as run;
    const TURN_BYTES: u32 = 4096;
    let mut receiving = fixtures::domain_limits();
    receiving.client.domain.turn_text = TURN_BYTES;
    let words = fixtures::words();
    let bounds = crate::Limits::of(&receiving, &words).expect("view bounds");
    let mut tool = skein_lib::bytes::zeroed(usize::try_from(TURN_BYTES).unwrap());
    tool.fill(b'x');
    let mut call = run::call(1, b"read", b"", None);
    let c::Block::Call(source) = &mut call else { panic!("call") };
    source.tool = c::Text::new(Box::from(tool.as_ref()), TURN_BYTES).unwrap();
    let mut thread = Thread::new(
        TASK,
        &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TURN_BYTES, live: TURN_BYTES },
    );
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    let mut rows = List::with_capacity(1);
    rows.push(run::turn(1, ATTEMPT, 1, &[run::message(c::Speaker::Assistant, &[call])], TURN_BYTES)).unwrap();
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    let mut b = skein_ui::Builder::new(&bounds.tree);
    crate::pages::chat::conversation(
        &mut b,
        &thread,
        &words,
        (receiving.client.domain.rows, TURN_BYTES),
        crate::pages::chat::Speakers { person: None, requester: false },
        crate::pages::chat::Drawn::Chat { request: None, delegations: None },
        &mut crate::frame::Bindings::new(bounds.tree.bindings),
    );
    let tree = b.finish();
    assert_call_fold(&tree, b"1 tool call", &tool, &[&tool]);
}
// The fake person sees the fold and all its content, without an Expanded state.
pub(super) fn assert_call_fold(tree: &skein_ui::Tree, name: &[u8], description: &[u8], lines: &[&[u8]]) {
    assert!(fixtures::person_find(tree, Role::Disclosure, name).is_some());
    let at = named_position(tree.nodes(), Role::Disclosure, Some(name)).expect("disclosure node");
    let fold = &tree.nodes()[at];
    assert_eq!(fold.description.as_deref(), Some(description));
    assert!(!fold.states.as_slice().contains(&skein_ui::State::Expanded));
    let mut child = at.checked_add(1).unwrap();
    for line in lines {
        let node = &tree.nodes()[child];
        assert_eq!(node.role, Role::Slot);
        assert!(node.styles.as_slice().contains(&crate::look::WORK));
        assert_eq!(fixtures::node_text(tree, child).as_ref(), *line);
        child = child.checked_add(usize::try_from(node.size).expect("work line size")).unwrap();
    }
    assert_eq!(child, at.checked_add(usize::try_from(fold.size).expect("fold size")).unwrap());
}
#[test]
fn a_persons_words_are_paragraphs_without_a_read_label() {
    let mut thread = structured_thread();
    let mut rows = List::with_capacity(1);
    rows.push(crate::tests::documents::line(
        6,
        Wall::EPOCH,
        jig_client::contract::Activity::Words {
            message: 2,
            by: jig_client::contract::Requester::Person(
                (c::Party { number: PARTY, kind: c::PartyKind::Person, name: fixtures::first_run::text(b"Pat") })
                    .number,
            ),
            kind: jig_client::contract::MessageKind::Words,
            words: fixtures::first_run::text(b"First line\nSecond line"),
        },
    ))
    .expect("words");
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 5 });
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    let tree = tree(&thread);
    assert!(fixtures::find(&tree, Role::Paragraph, b"First line").is_some());
    assert!(fixtures::find(&tree, Role::Paragraph, b"Second line").is_some());
    assert!(fixtures::find(&tree, Role::Status, b"Read").is_none());
}

#[test]
fn an_owners_person_words_have_a_person_shell_and_one_paragraph_per_line() {
    let mut thread =
        Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: 512, live: 512 });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    let mut rows = List::with_capacity(1);
    rows.push(jig_client::contract::DecodedActivityLine {
        source: Box::new(c::ActivityLine {
            task: TASK,
            position: 1,
            at: Wall::EPOCH,
            line: c::Activity::Words {
                message: MESSAGE,
                by: c::Requester::Person(PARTY),
                kind: c::MessageKind::Words,
                words: fixtures::first_run::text(b"First line\nSecond line"),
            },
        }),
        decoded: None,
    })
    .expect("owner words");
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    let tree = tree(&thread);
    let person = styled(&tree, crate::look::BY_PERSON);
    let said = fixtures::node_text(&tree, person);
    assert!(skein_lib::bytes::find(&said, b"PersonFirst lineSecond line").is_some());
    assert!(fixtures::find(&tree, Role::Paragraph, b"First line").is_some());
    assert!(fixtures::find(&tree, Role::Paragraph, b"Second line").is_some());
    assert!(fixtures::find(&tree, Role::Status, b"Read").is_none());
}

#[test]
fn a_tasks_words_have_an_agent_shell_named_by_the_task() {
    let mut thread =
        Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT, live: TEXT });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    let mut rows = List::with_capacity(1);
    rows.push(jig_client::contract::DecodedActivityLine {
        source: Box::new(c::ActivityLine {
            task: TASK,
            position: 1,
            at: Wall::EPOCH,
            line: c::Activity::Words {
                message: MESSAGE,
                by: c::Requester::Task(2),
                kind: c::MessageKind::Question,
                words: fixtures::first_run::text(b"First line\nSecond line"),
            },
        }),
        decoded: None,
    })
    .expect("owner words");
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
    crate::pages::chat::conversation(
        &mut b,
        &thread,
        &fixtures::words(),
        (LINES, TEXT),
        crate::pages::chat::Speakers { person: Some(PARTY), requester: true },
        crate::pages::chat::Drawn::Chat { request: None, delegations: None },
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
    );
    let tree = b.finish();
    let turn = styled(&tree, crate::look::TURN);
    let message = styled(&tree, crate::look::BY_DELEGATE);
    #[expect(clippy::disallowed_methods, reason = "the view assertion searches the bounded turn's nodes")]
    {
        assert!(subtree(&tree, turn).iter().any(|node| node.styles.as_slice().contains(&crate::look::BY_DELEGATE)));
    }
    let mut paragraphs = List::with_capacity(3);
    for node in subtree(&tree, message) {
        assert_ne!(node.role, Role::Status);
        if node.role == Role::Paragraph {
            paragraphs.push(node.name.as_deref().expect("named paragraph")).expect("three paragraphs");
        }
    }
    assert_eq!(paragraphs.as_slice(), [b"T2".as_slice(), b"First line", b"Second line"]);
    let agent = fixtures::chat::tree(&thread);
    assert!(style_position(agent.nodes(), crate::look::BY_AGENT).is_some());
    assert!(style_position(agent.nodes(), crate::look::BY_DELEGATE).is_none());
}

#[test]
fn committed_text_exceeds_field_text_on_both_conversation_pages() {
    use fixtures::first_run as run;
    let value = [b'x'; 256];
    let mut thread =
        Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT, live: TEXT });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    let mut rows = List::with_capacity(1);
    rows.push(run::turn(
        1,
        ATTEMPT,
        1,
        &[run::message(c::Speaker::Assistant, &[c::Block::Text(run::text(&value))])],
        2048,
    ))
    .expect("committed turn");
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    for projected in [true, false] {
        let drawn = if projected { fixtures::chat::projected_tree(&thread) } else { fixtures::chat::tree(&thread) };
        assert!(skein_lib::bytes::find(&fixtures::text(&drawn), &value).is_some(), "chat projection {projected}");
        assert!(fixtures::find(&drawn, Role::Status, b"Earlier text omitted").is_none());
    }
}

fn result_view(result: c::TaskResult) -> crate::View {
    result_view_at(result, None)
}
fn result_view_at(result: c::TaskResult, at: Option<Wall>) -> crate::View {
    use temper_client_domain::{Address, Event};
    let (mut root, _, watch) = fixtures::root_at(Address::Chat { task: TASK }, &fixtures::domain_limits());
    let live = messaging_root();
    let mut task = live.pages().chat().expect("chat").task(live.client()).expect("task").clone();
    // Keep Message offered: the same result which draws the end must suppress the whole foot.
    task.requester = c::Requester::Person(PARTY);
    task.result = Some(result);
    {
        let c::ChipPhaseFacts { details, .. } = task.chip.details.as_mut();
        details.ended_at = at;
    }
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("task watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
    );
    fixtures::page_view(&root)
}

fn result_document(ending: c::Ending) -> c::TaskResult {
    c::TaskResult::Document(Box::new(c::Result {
        task: fixtures::task::document().chip,
        ending,
        value: None,
        at: None,
        details: None,
        acts: List::with_capacity(0),
    }))
}

#[test]
fn every_chat_end_line_carries_ended_and_omits_the_whole_composer() {
    use fixtures::first_run::text;
    let report = c::ResultValue::Report(c::DocumentText::complete(text(b"A report")));
    let cancelled = c::ChipEnding::Cancelled {
        reason: c::DocumentText::complete(text(b"No longer needed")),
        result: None,
        by: c::Requester::Person(74),
        at: Wall::EPOCH,
    };
    for (result, expected) in [
        (c::TaskResult::Ending(Box::new(c::ChipEnding::Done(report.clone()))), "Closed · its report, “A report”"),
        (
            c::TaskResult::Ending(Box::new(c::ChipEnding::Failed(c::DocumentText::complete(text(b"Checks failed"))))),
            "Failed · Checks failed",
        ),
        (c::TaskResult::Ending(Box::new(cancelled.clone())), "Cancelled · No longer needed"),
        (result_document(c::Ending::Task(Box::new(c::ChipEnding::Done(report)))), "Closed · its report, “A report”"),
        (result_document(c::Ending::Task(Box::new(cancelled))), "Cancelled · No longer needed"),
        (result_document(c::Ending::Failed { reason: Some(text(b"Checks failed")) }), "Failed · Checks failed"),
        (result_document(c::Ending::Failed { reason: None }), "Failed · "),
        (
            result_document(c::Ending::Cancelled {
                reason: Some(text(b"No longer needed")),
                by: c::Actor::Party(74),
                at: None,
            }),
            "Cancelled · No longer needed",
        ),
        (result_document(c::Ending::Withdrawn { reason: text(b"Withdrawn") }), "Cancelled · Withdrawn"),
        (result_document(c::Ending::Done), "Closed"),
        (result_document(c::Ending::Deleted), "Closed"),
        (result_document(c::Ending::Decided { by: PARTY, party: None, how: None, at: Wall::EPOCH }), "Closed"),
    ] {
        let view = result_view(result);
        let end = fixtures::find(view.tree(), Role::Paragraph, expected.as_bytes()).expect("end line");
        assert_eq!(view.tree().find(end).expect("end").styles.as_slice(), [crate::look::ENDED]);
        let log = named_position(view.tree().nodes(), Role::Log, None).unwrap();
        let end_at = styled(view.tree(), crate::look::ENDED);
        assert_eq!(
            end_at + usize::try_from(view.tree().nodes()[end_at].size).unwrap(),
            log + usize::try_from(view.tree().nodes()[log].size).unwrap(),
            "end line is inside the log after the turns"
        );
        for token in [crate::look::CHOICE, crate::look::SESSION_FOOT, crate::look::SESSION_COMPOSER] {
            assert!(style_position(view.tree().nodes(), token).is_none(), "ended foot: {token:?}");
        }
        assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_none());
    }
}

fn cancelled_by(by: c::Requester) -> c::ChipEnding {
    c::ChipEnding::Cancelled {
        by,
        reason: c::DocumentText::complete(fixtures::first_run::text(b"No longer needed")),
        result: None,
        at: Wall::EPOCH,
    }
}
fn cancelled_document(by: c::Actor) -> c::TaskResult {
    result_document(c::Ending::Cancelled { by, reason: Some(fixtures::first_run::text(b"No longer needed")), at: None })
}
#[test]
fn a_chats_end_line_says_who_ended_it() {
    use fixtures::first_run::text;
    let report = c::ResultValue::Report(c::DocumentText::complete(text(b"A report")));
    let mut reported = result_document(c::Ending::Done);
    let c::TaskResult::Document(doc) = &mut reported else { panic!("document") };
    doc.value = Some(report.clone());
    for (name, owner, document, clock, expected) in [
        (
            "requester",
            cancelled_by(c::Requester::Person(PARTY)),
            cancelled_document(c::Actor::Party(PARTY)),
            Some(fixtures::NOW),
            "Closed 00:00",
        ),
        (
            "requester without clock",
            cancelled_by(c::Requester::Person(PARTY)),
            cancelled_document(c::Actor::Party(PARTY)),
            None,
            "Closed",
        ),
        (
            "another person",
            cancelled_by(c::Requester::Person(74)),
            cancelled_document(c::Actor::Party(74)),
            Some(fixtures::NOW),
            "Cancelled 00:00 · No longer needed",
        ),
        (
            "a task",
            cancelled_by(c::Requester::Task(7)),
            cancelled_document(c::Actor::Task(7)),
            Some(fixtures::NOW),
            "Cancelled 00:00 · No longer needed",
        ),
        ("report", c::ChipEnding::Done(report), reported, Some(fixtures::NOW), "Closed 00:00 · its report, “A report”"),
        (
            "failure",
            c::ChipEnding::Failed(c::DocumentText::complete(text(b"Checks failed"))),
            result_document(c::Ending::Failed { reason: Some(text(b"Checks failed")) }),
            Some(fixtures::NOW),
            "Failed 00:00 · Checks failed",
        ),
    ] {
        for result in [
            c::TaskResult::Ending(Box::new(owner.clone())),
            result_document(c::Ending::Task(Box::new(owner))),
            document,
        ] {
            let view = result_view_at(result, clock);
            assert!(fixtures::find(view.tree(), Role::Paragraph, expected.as_bytes()).is_some(), "{name}");
            for token in [crate::look::SESSION_FOOT, crate::look::CHOICE, crate::look::SESSION_COMPOSER] {
                assert!(style_position(view.tree().nodes(), token).is_none(), "{name}");
            }
            assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_none());
            assert!(fixtures::find(view.tree(), Role::Button, b"Send").is_none());
        }
    }
}

#[test]
fn a_live_chat_keeps_its_choice_line_and_composer() {
    let view = fixtures::page_view(&messaging_root());
    assert!(style_position(view.tree().nodes(), crate::look::CHOICE).is_some());
    assert!(style_position(view.tree().nodes(), crate::look::BYLINE).is_some());
    assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_some());
}

#[test]
fn a_chats_end_and_its_phase_keep_the_supplied_clock() {
    use fixtures::first_run::text;
    for (ending, expected) in [
        (
            c::ChipEnding::Done(c::ResultValue::Report(c::DocumentText::complete(text(b"A report")))),
            "Closed 00:00 · its report, “A report”",
        ),
        (c::ChipEnding::Failed(c::DocumentText::complete(text(b"Checks failed"))), "Failed 00:00 · Checks failed"),
        (
            c::ChipEnding::Cancelled {
                reason: c::DocumentText::complete(text(b"No longer needed")),
                result: None,
                by: c::Requester::Person(74),
                at: Wall::EPOCH,
            },
            "Cancelled 00:00 · No longer needed",
        ),
    ] {
        let view = result_view_at(c::TaskResult::Ending(Box::new(ending)), Some(fixtures::NOW));
        assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), expected.as_bytes()).is_some());
        assert!(fixtures::find(view.tree(), Role::Status, b"Closed 00:00").is_some());
        assert_eq!(view.next_change(), Some(Wall::from_nanos(86_400_000_000_000)));
    }
}

#[test]
fn an_automatic_seen_draws_no_progress() {
    let root = chat_with_history();
    let chat = root.pages().chat().expect("chat");
    let mut seeing = false;
    for (key, kind) in &chat.pending {
        if *kind == temper_client_domain::pages::chat::Sending::Seen {
            assert!(root.client().pending(*key).is_some(), "request table has Seen progress");
            seeing = true;
        }
    }
    assert!(seeing, "automatic Seen pending");
    let view = fixtures::page_view(&root);
    let words = fixtures::words();
    for phrase in
        [crate::cards::P::InFlight, crate::cards::P::Retrying, crate::cards::P::Parked, crate::cards::P::Unknown]
    {
        let name = words.say_card(phrase, &[]);
        assert!(fixtures::find(view.tree(), Role::Status, &name).is_none(), "automatic request progress: {name:?}");
    }
    assert!(fixtures::find(view.tree(), Role::Button, b"Mark seen").is_none());
}

// Thursday 22 October 2026, noon UTC, and its neighbouring days.
const TURN_NOON: u64 = 1_792_670_400;
const TURN_DAY: u64 = 86_400;
const TURN_MIDNIGHT: u64 = TURN_NOON - TURN_DAY / 2;
fn turn_wall(seconds: u64) -> Wall {
    Wall::from_nanos(seconds.checked_mul(1_000_000_000).expect("fixture wall fits"))
}
fn timed_thread(events: &[(u64, jig_client::contract::DecodedActivityLine)]) -> Thread {
    let mut thread = Thread::new(TASK, &jig_client::thread::Limits { lines: 12, bytes: BYTES, text: 512, live: 512 });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    let mut rows = List::with_capacity(u32::try_from(events.len()).unwrap());
    for (index, (at, event)) in events.iter().enumerate().rev() {
        rows.push({
            let mut line = event.clone();
            line.source.position = u64::try_from(index).unwrap().checked_add(1).expect("position fits");
            line.source.at = turn_wall(*at);
            line
        })
        .unwrap();
    }
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    thread
}
fn timed_words() -> jig_client::contract::DecodedActivityLine {
    crate::tests::documents::line(
        1,
        Wall::EPOCH,
        jig_client::contract::Activity::Words {
            message: MESSAGE,
            by: jig_client::contract::Requester::Person(
                (c::Party { number: PARTY, kind: c::PartyKind::Person, name: fixtures::first_run::text(b"Pat") })
                    .number,
            ),
            kind: jig_client::contract::MessageKind::Words,
            words: fixtures::first_run::text(b"A message"),
        },
    )
}
fn timed_turn() -> jig_client::contract::DecodedActivityLine {
    fixtures::first_run::turn(
        1,
        ATTEMPT,
        TURN,
        &[fixtures::first_run::message(
            c::Speaker::Assistant,
            &[c::Block::Text(fixtures::first_run::text(b"An answer"))],
        )],
        512,
    )
}
fn timed_tree(
    thread: &Thread,
    drawn: crate::pages::chat::Drawn<'_>,
    person: Option<u64>,
    now: u64,
) -> (skein_ui::Tree, Option<Wall>) {
    let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
    let mut times = crate::words::Times::new(turn_wall(now));
    crate::pages::chat::conversation_with_times(
        &mut b,
        thread,
        &fixtures::words(),
        (LINES, TEXT),
        crate::pages::chat::Speakers { person, requester: true },
        (drawn, Some(&mut times)),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
    );
    (b.finish(), times.next)
}
fn timed_drawing(projected: bool) -> crate::pages::chat::Drawn<'static> {
    if projected {
        crate::pages::chat::Drawn::Chat { request: None, delegations: None }
    } else {
        crate::pages::chat::Drawn::Lines
    }
}
fn timed_style_positions(tree: &skein_ui::Tree, style: skein_ui::Style) -> List<usize> {
    let mut positions = List::with_capacity(fixtures::tree_limits().nodes);
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&style) {
            positions.push(index).expect("positions fit tree nodes");
        }
    }
    positions
}
#[test]
fn turn_clocks_follow_separately_named_speakers_on_both_paths() {
    let thread = timed_thread(&[(TURN_NOON, timed_words()), (TURN_NOON + 60, timed_turn())]);
    for projected in [true, false] {
        for (person, label) in [(Some(PARTY), b"You".as_slice()), (None, b"Person".as_slice())] {
            let (tree, _) = timed_tree(&thread, timed_drawing(projected), person, TURN_NOON);
            let roles = timed_style_positions(&tree, crate::look::WHO);
            for (index, (name, clock)) in roles.iter().zip([(label, b"12:00"), (b"Temper".as_slice(), b"12:01")]) {
                let children = subtree(&tree, *index);
                assert_eq!(children[0].name.as_deref(), Some(name));
                let clock_at = usize::try_from(children[0].size).unwrap();
                assert_eq!(children[clock_at].role, Role::Time);
                assert_eq!(children[clock_at].name.as_deref(), Some(clock.as_slice()));
                assert_eq!(children[clock_at].size, 2, "the time holds only its text leaf");
                assert_eq!(children[clock_at + 1].role, Role::Text);
            }
            assert_eq!(roles.len(), 2);
        }
    }
}
#[test]
fn pending_words_and_provisional_text_have_no_clock_or_day() {
    let mut pending =
        Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT, live: TEXT });
    assert!(pending.words.begin(
        jig_client::Key { asker: jig_client::Asker::JIG, number: 1 },
        None,
        fixtures::first_run::text(b"Pending")
    ));
    for thread in [&pending, &provisional()] {
        for projected in [true, false] {
            let (tree, next) = timed_tree(thread, timed_drawing(projected), None, TURN_NOON);
            for node in tree.nodes() {
                assert_ne!(node.role, Role::Time);
                assert!(!node.styles.as_slice().contains(&crate::look::DAY));
            }
            assert_eq!(next, None);
        }
    }
}
#[test]
fn the_request_uses_the_first_timed_line_and_shares_its_day() {
    let began = crate::tests::documents::line(
        1,
        Wall::EPOCH,
        jig_client::contract::Activity::RunBegan {
            attempt: ATTEMPT,
            why: jig_client::contract::RunWhy::Made,
            resumed: false,
        },
    );
    let thread = timed_thread(&[(TURN_NOON - 60, began), (TURN_NOON, timed_words())]);
    let request = c::SpecificationDocument::complete(c::Specification {
        asks: false,
        words: fixtures::first_run::text(b"The request"),
        parameters: List::with_capacity(0),
        inputs: List::with_capacity(0),
    });
    for (thread, expected) in [(&thread, Some(b"12:00".as_slice())), (&provisional(), None)] {
        let (tree, _) = timed_tree(
            thread,
            crate::pages::chat::Drawn::Chat { request: Some(&request), delegations: None },
            None,
            TURN_NOON,
        );
        let who = styled(&tree, crate::look::WHO);
        let children = subtree(&tree, who);
        assert_eq!(children[0].name.as_deref(), Some(b"You".as_slice()));
        let mut clock = None;
        for child in children {
            if child.role == Role::Time {
                clock = child.name.as_deref();
            }
        }
        assert_eq!(clock, expected);
        assert_eq!(timed_style_positions(&tree, crate::look::DAY).len(), u32::from(expected.is_some()));
    }
}
#[test]
fn each_days_first_turn_has_one_day_line_with_the_calendar_words() {
    // Wednesday 31 December 2025: a year must be said before this year's October.
    let last_year = 1_767_139_200;
    let thread = timed_thread(&[
        (last_year, timed_words()),
        (TURN_NOON - 2 * TURN_DAY, timed_words()),
        (TURN_NOON - TURN_DAY, timed_words()),
        (TURN_NOON, timed_words()),
        (TURN_NOON + 60, timed_turn()),
    ]);
    for projected in [true, false] {
        let (tree, next) = timed_tree(&thread, timed_drawing(projected), None, TURN_NOON);
        let days = timed_style_positions(&tree, crate::look::DAY);
        for (index, said) in days.iter().zip([
            "Wednesday 31 December 2025",
            "Tuesday 20 October",
            "Yesterday · Wednesday 21 October",
            "Today · Thursday 22 October",
        ]) {
            let node = &tree.nodes()[*index];
            assert_eq!(node.name.as_deref(), Some(said.as_bytes()));
            assert!(
                tree.nodes()[index + usize::try_from(node.size).unwrap()]
                    .styles
                    .as_slice()
                    .contains(&crate::look::TURN),
                "day precedes its first turn"
            );
        }
        assert_eq!(days.len(), 4);
        assert_eq!(next, Some(turn_wall(TURN_MIDNIGHT + TURN_DAY)));
    }
}
#[test]
fn an_undecodable_turn_keeps_its_clock_and_day_on_both_paths() {
    let thread = timed_thread(&[(
        TURN_NOON,
        crate::tests::documents::turn(
            1,
            Wall::EPOCH,
            c::TurnName { attempt: ATTEMPT, number: TURN },
            c::TurnRead::Undecodable { why: c::TurnProblem::Malformed },
        ),
    )]);
    for projected in [true, false] {
        let (tree, _) = timed_tree(&thread, timed_drawing(projected), None, TURN_NOON);
        assert!(fixtures::find(&tree, Role::Time, b"12:00").is_some());
        assert!(fixtures::find(&tree, Role::Paragraph, "Today · Thursday 22 October".as_bytes()).is_some());
    }
}
#[test]
fn a_chat_rebuilds_its_day_at_midnight_and_keeps_the_turn_clock() {
    let root = chat_with_history();
    let shown = temper_client_domain::shown(&root);
    let mut view = fixtures::view();
    let mut patches = skein_lib::Queue::with_capacity(view.limits().tree.patches);
    for (now, day, next) in [
        (TURN_DAY - 1, "Today · Thursday 1 January", TURN_DAY),
        (TURN_DAY, "Yesterday · Thursday 1 January", 2 * TURN_DAY),
    ] {
        crate::build(&mut view, &root, turn_wall(now), &mut patches);
        assert!(fixtures::find(view.tree(), Role::Paragraph, day.as_bytes()).is_some());
        assert!(fixtures::find(view.tree(), Role::Time, b"00:00").is_some());
        assert_eq!(view.next_change(), Some(turn_wall(next)));
        assert_eq!(temper_client_domain::shown(&root), shown);
        while patches.pop().is_some() {}
    }
}

#[test]
fn the_chatbar_and_chats_line_share_every_status_phrase() {
    const STATES: u32 = 18;
    let mut rows = List::with_capacity(STATES);
    for (line, _) in super::chats::status_rows() {
        rows.push(line).unwrap();
    }
    for (waits_on, live, wake) in [
        (c::WaitsOn::Delegates, 1, c::WakeRule::Never),
        (c::WaitsOn::Delegates, 2, c::WakeRule::Never),
        (c::WaitsOn::News, 0, c::WakeRule::Immediate),
        (c::WaitsOn::News, 0, c::WakeRule::Never),
        (c::WaitsOn::News, 2, c::WakeRule::Never),
        (c::WaitsOn::Delegates, 0, c::WakeRule::Immediate),
    ] {
        let mut line = super::chats::line();
        line.waits_on = waits_on;
        let c::ChipPhaseFacts { details, .. } = line.chip.details.as_mut();
        details.live_delegates = live;
        details.wake.words = wake;
        rows.push(line).unwrap();
    }
    let mut closed = super::chats::line();
    let c::ChipPhaseFacts { phase, details } = closed.chip.details.as_mut();
    *phase = c::ChipPhase::Ended;
    details.ended_at = Some(Wall::EPOCH);
    details.ending = Some(c::ChipEnding::Cancelled {
        by: closed.requester,
        reason: c::DocumentText::complete(fixtures::first_run::text(b"Any reason")),
        result: None,
        at: Wall::EPOCH,
    });
    rows.push(closed).unwrap();
    let expected = [
        b"waiting on an answer from you".as_slice(),
        b"Held",
        b"Retrying",
        b"Starting",
        b"Writing",
        b"Using tool search",
        b"Working",
        b"Waiting for another party",
        b"Parked",
        b"Closing",
        b"Closed",
        b"waiting on 1 delegate",
        b"waiting on 2 delegates",
        b"waiting on your words",
        b"Parked",
        b"waiting on 2 delegates",
        b"waiting on your words",
        b"Closed",
    ];
    for (line, expected) in rows.iter().zip(expected) {
        let (line_tree, _) = super::chats::tree(line);
        assert!(fixtures::find(&line_tree, Role::Status, expected).is_some(), "chats line: {expected:?}");
        let c::ChipPhaseFacts { details, .. } = line.chip.details.as_ref();
        let root = fixtures::root_in_project(None);
        let me = root.pages().me(root.client()).unwrap();
        let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
        crate::pages::chats::status(
            &mut b,
            &line.chip,
            (&line.waits_on, Some(details.live_delegates), None),
            (line.project, line.requester),
            Some(&details.wake),
            me,
            &fixtures::words(),
        );
        assert!(fixtures::find(&b.finish(), Role::Status, expected).is_some(), "chatbar: {expected:?}");
    }
}

#[test]
fn the_chatbar_draws_the_supplied_budget_including_zero() {
    use temper_client_domain::{Address, Event};
    for (budget, phrase) in [(0, "7 spent · of 0".as_bytes()), (100, "7 spent · of 100".as_bytes())] {
        let (mut root, _, watch) = fixtures::root_at(Address::Chat { task: TASK }, &fixtures::domain_limits());
        let mut task = fixtures::task::document();
        task.chip.task = TASK;
        task.budget.numbers.budget = budget;
        fixtures::root_event(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch: watch.unwrap(),
                        event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                    },
                },
            },
        );
        let view = fixtures::page_view(&root);
        assert!(fixtures::find(view.tree(), Role::Paragraph, phrase).is_some());
    }
}

#[test]
fn a_proposals_fold_precedes_its_card_after_the_message() {
    use fixtures::first_run as run;
    let object = c::ObjectKey::Proposal(fixtures::chat::PROPOSAL);
    let mut proposal = run::call(3, b"propose", b"{}", None);
    let c::Block::Call(call) = &mut proposal else { panic!("call") };
    call.objects = List::with_capacity(1);
    call.objects.push(object).unwrap();
    let mut message = run::message(
        c::Speaker::Assistant,
        &[
            c::Block::Text(run::text(b"A proposal follows")),
            run::call(1, b"read", b"{}", None),
            run::call(2, b"read", b"{}", None),
        ],
    );
    let mut blocks = List::with_capacity(4);
    for block in &message.blocks {
        blocks.push(block.clone()).unwrap();
    }
    blocks.push(proposal.clone()).unwrap();
    message.blocks = blocks;
    let mut rows = List::with_capacity(2);
    let mut first = run::turn(1, ATTEMPT, 1, &[], 512);
    let Some(c::ArchivedTurn { turn: c::TurnRead::Decoded(turn), .. }) = &mut first.decoded else {
        panic!("decoded turn")
    };
    turn.messages = List::with_capacity(1);
    turn.messages.push(message).unwrap();
    rows.push(first).unwrap();
    rows.push(run::turn(2, ATTEMPT, 2, &[run::message(c::Speaker::Assistant, &[proposal])], 512)).unwrap();
    let root = fixtures::chat::proposal_root_with_activity(Some(c::ActivityPage {
        rows,
        before: None,
        after: None,
        cut: false,
    }));
    let mut builder = skein_ui::Builder::new(&fixtures::tree_limits());
    crate::pages::chat::main(
        &mut builder,
        &root,
        &fixtures::words(),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
        4,
        TEXT,
        &mut crate::words::Times::new(fixtures::NOW),
    );
    let tree = builder.finish();
    let mut turn = None;
    for (at, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&crate::look::TURN)
            && skein_lib::bytes::find(&fixtures::node_text(&tree, at), b"A proposal follows").is_some()
        {
            turn = Some(at);
            break;
        }
    }
    assert!(
        turn.is_some(),
        "assistant turn: {:?}; rows {:?}",
        fixtures::text(&tree),
        root.pages().chat().unwrap().thread.lines.rows
    );
    let turn = turn.unwrap();
    let children = subtree(&tree, turn);
    let card = style_position(children, crate::look::CARD).expect("proposal inside first turn");
    let message = style_position(children, crate::look::MESSAGE).expect("message");
    assert!(card >= message + usize::try_from(children[message].size).unwrap());
    let fold = style_position(children, crate::look::WORK).expect("fold in the same turn");
    assert!(fold >= message + usize::try_from(children[message].size).unwrap());
    assert!(fold + usize::try_from(children[fold].size).unwrap() <= card, "fold before proposal card");
    assert_eq!(named_count(&tree, Role::Button, b"Accept"), 1);
    assert_call_fold(&tree, b"2 tool calls", "read ×2".as_bytes(), &[b"read", b"read"]);
    let said = fixtures::text(&tree);
    assert!(skein_lib::bytes::find(&said, b"propose").is_none());
}

#[test]
fn a_chat_proposal_links_to_the_inbox_filtered_to_proposals() {
    let mut view = fixtures::page_view(&proposal_root());
    let link = fixtures::find(view.tree(), Role::Link, "Open in inbox ↗".as_bytes()).expect("inbox link");
    assert_eq!(
        crate::act(&mut view, link, skein_ui::Act::Press),
        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
            address: temper_client_domain::Address::Inbox {
                filter: temper_client_domain::address::Filter {
                    kind: Some(temper_client_domain::address::Kind::Proposal),
                    ..temper_client_domain::address::Filter::ALL
                }
            }
        }))
    );
}

fn named_count(tree: &skein_ui::Tree, role: Role, name: &[u8]) -> u32 {
    let mut count = 0_u32;
    for node in tree.nodes() {
        if node.role == role && node.name.as_deref() == Some(name) {
            count = count.checked_add(1).unwrap();
        }
    }
    count
}

#[test]
fn the_playground_chats_bar_names_the_default_model() {
    let root = fixtures::playground_root(temper_client_domain::Address::Chat { task: TASK }, fixtures::PROJECT, true);
    let view = fixtures::page_view(&root);
    let bar = styled(view.tree(), crate::look::SESSION_TOP);
    let byline = style_position(subtree(view.tree(), bar), crate::look::BYLINE).expect("byline");
    let node = &view.tree().nodes()[bar + 1 + byline];
    assert_eq!(node.description.as_deref().or(node.name.as_deref()), Some("T9 · Scripted model · Default".as_bytes()));
}

#[test]
fn unavailable_models_block_send_and_enter_in_the_view() {
    let root = super::choice::no_model_root(true);
    let mut view = fixtures::page_view(&root);
    for (role, name, event) in [
        (Role::Button, b"Send".as_slice(), skein_ui::Act::Press),
        (Role::TextArea, b"Message".as_slice(), skein_ui::Act::Submit),
    ] {
        let node = fixtures::find(view.tree(), role, name).expect("composer control");
        assert_eq!(crate::act(&mut view, node, event), None);
    }
}

fn configuring_task_root(mut task: c::Task, template: bool, charter: u32) -> temper_client_domain::Domain {
    let (mut root, me_watch, task_watch) =
        fixtures::root_at(temper_client_domain::Address::Chat { task: TASK }, &fixtures::domain_limits());
    let source = messaging_root();
    let mut me = source.pages().me(source.client()).expect("me").clone();
    if template {
        me = fixtures::configuring_me(me);
    }
    me.projects.get_mut(0).unwrap().models = super::choice::line_models(true);
    fixtures::deliver_me(&mut root, me_watch, me);
    task.chip.executor = c::ChipExecutor::Agent { charter };
    deliver_chat(&mut root, task_watch.expect("task watch"), c::Delivery::Task(Box::new(task)));
    root
}
fn configuring_task() -> c::Task {
    let source = messaging_root();
    let mut task = source.pages().chat().expect("chat").task(source.client()).expect("task").clone();
    task.chip.title = c::DocumentText::complete(fixtures::first_run::text(b"Hello"));
    task.acts = List::with_capacity(3);
    for act in [c::ActKind::Message, c::ActKind::Stop, c::ActKind::Cancel] {
        task.acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).unwrap();
    }
    task
}
#[test]
fn configuring_chats_keep_the_settings_layout_and_controls_in_both_shells() {
    use crate::look;
    use temper_client_domain::{self as d, Address, pages::Act};
    let mut root = configuring_task_root(configuring_task(), true, 37);
    fixtures::root_event(
        &mut root,
        d::Event::Act { action: d::Action::Pages(Act::MessageWords { words: b"My words".as_slice().into() }) },
    );
    for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        let tree = view.tree();
        let main = styled(tree, look::SESSION);
        assert_eq!(tree.nodes()[main].styles.as_slice(), [look::MODE_CHAT, look::SESSION]);
        assert_eq!(style_position(tree.nodes(), look::AGENT_PAGE), None);
        let top = styled(tree, look::SESSION_TOP);
        let bar = styled(tree, look::SESSION_RIGHT);
        assert!(top < bar && bar < top + usize::try_from(tree.nodes()[top].size).unwrap());
        let top_nodes = subtree(tree, top);
        for (role, name) in [
            (Role::Heading { level: 1 }, b"T9 \xc2\xb7 Hello".as_slice()),
            (Role::Link, b"Settings"),
            (Role::Status, b"Parked"),
            (Role::Paragraph, b"7 spent \xc2\xb7 of 7"),
            (Role::Button, b"Stop"),
            (Role::Button, b"Close"),
        ] {
            assert!(named_position(top_nodes, role, Some(name)).is_some(), "{name:?}");
        }
        let controls = subtree(tree, bar);
        let positions = [
            named_position(controls, Role::Status, Some(b"Parked")).unwrap(),
            named_position(controls, Role::Button, Some(b"Stop")).unwrap(),
            named_position(controls, Role::Button, Some(b"Close")).unwrap(),
            named_position(controls, Role::Paragraph, Some(b"7 spent \xc2\xb7 of 7")).unwrap(),
        ];
        for pair in positions.windows(2) {
            assert!(pair[0] < pair[1]);
        }
        let convo = styled(tree, look::CONVO);
        assert_eq!(tree.nodes()[convo].role, Role::Region);
        let thread = styled(tree, look::THREAD);
        let foot = styled(tree, look::SESSION_FOOT);
        assert_eq!(thread, convo + 1);
        assert_eq!(foot, thread + usize::try_from(tree.nodes()[thread].size).unwrap());
        assert!(foot < convo + usize::try_from(tree.nodes()[convo].size).unwrap());
        assert_eq!(tree.nodes()[foot].role, Role::Form);
        let entry = styled(tree, look::SESSION_COMPOSER);
        let area = named_position(tree.nodes(), Role::TextArea, Some(b"Message")).unwrap();
        assert_eq!(area, entry + 1);
        assert_eq!(tree.nodes()[area].placeholder.as_deref(), Some(b"Reply to T9".as_slice()));
        assert_eq!(tree.nodes()[area].value.as_ref().unwrap().text.as_ref(), b"My words");
        assert!(tree.nodes()[area].states.as_slice().contains(&skein_ui::State::SubmitsOnEnter));
        let send = fixtures::find(tree, Role::Button, b"Send").unwrap();
        let area_id = tree.nodes()[area].id;
        let stop = fixtures::find(tree, Role::Button, b"Stop").unwrap();
        let close = fixtures::find(tree, Role::Button, b"Close").unwrap();
        let back = tree.nodes()[styled(tree, look::BACK)].id;
        let hint = styled(tree, look::SESSION_HINT);
        assert!(entry < hint && hint < foot + usize::try_from(tree.nodes()[foot].size).unwrap());
        assert_eq!(
            tree.nodes()[hint].name.as_deref(),
            Some(b"Enter sends \xc2\xb7 Shift+Enter for a new line".as_slice())
        );
        for style in [look::ACCOUNT_LINE, look::CHOICE, look::COMPOSER_AREA, look::STARTERS] {
            assert_eq!(style_position(tree.nodes(), style), None);
        }
        assert_eq!(crate::act(&mut view, area_id, skein_ui::Act::Submit), Some(d::Action::Pages(Act::Send)));
        assert_eq!(
            crate::act(&mut view, area_id, skein_ui::Act::Input { value: b"Changed".as_slice().into() }),
            Some(d::Action::Pages(Act::MessageWords { words: b"Changed".as_slice().into() }))
        );
        for (id, act) in [(send, Act::Send), (stop, Act::Stop), (close, Act::Close)] {
            assert_eq!(crate::act(&mut view, id, skein_ui::Act::Press), Some(d::Action::Pages(act)));
        }
        assert_eq!(
            crate::href(&mut view, back).as_deref(),
            Some(d::address::print(&Address::Settings { project: fixtures::PROJECT }).as_ref())
        );
    }
    let view = fixtures::page_view(&root);
    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", None, &mut print);
    let printed = print.text.as_slice();
    for words in
        ["Settings", "T9 · Hello", "Conversation:", "Message", "Send", "Enter sends · Shift+Enter for a new line"]
    {
        assert!(skein_lib::bytes::find(printed, words.as_bytes()).is_some(), "{words}");
    }
    assert!(skein_lib::bytes::find(printed, b"Change model").is_none());
}

#[test]
fn only_a_delivered_template_three_executor_changes_chat_path_and_navigation() {
    use crate::look;
    for (listed, charter, configuring) in [(true, 37, true), (true, 1, false), (false, 37, false), (false, 1, false)] {
        let root = configuring_task_root(configuring_task(), listed, charter);
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let tree = view.tree();
            assert!(style_position(tree.nodes(), look::SESSION).is_some());
            assert_eq!(style_position(tree.nodes(), look::AGENT_PAGE), None);
            for style in [look::RAIL, look::TABBAR] {
                if let Some(at) = style_position(tree.nodes(), style) {
                    for name in [b"Settings".as_slice(), b"Chats"] {
                        let nodes = subtree(tree, at);
                        let link = named_position(nodes, Role::Link, Some(name)).unwrap();
                        assert_eq!(
                            nodes[link].states.as_slice().contains(&skein_ui::State::Current),
                            if name == b"Settings" { configuring } else { !configuring }
                        );
                    }
                }
            }
            assert!(style_position(tree.nodes(), look::SESSION_FOOT).is_some());
            assert_eq!(style_position(tree.nodes(), look::CHOICE).is_none(), configuring);
        }
    }
    let (root, _, _) =
        fixtures::root_at(temper_client_domain::Address::Chat { task: TASK }, &fixtures::domain_limits());
    let view = fixtures::browser_page_view(&root);
    assert!(style_position(view.tree().nodes(), look::SESSION).is_some(), "shared layout before delivery");
    assert!(style_position(view.tree().nodes(), look::AGENT_PAGE).is_none());
}

#[test]
fn every_chat_omits_the_foot_when_ended_or_message_is_not_offered() {
    for configuring in [false, true] {
        for (offered, ended, result) in [(false, false, false), (true, true, false), (true, false, true)] {
            let mut task = configuring_task();
            if ended {
                task.chip.phase = c::Phase::Ended;
                task.chip.details.phase = c::ChipPhase::Ended;
            }
            if !offered {
                task.acts.clear();
            }
            if result {
                task.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Done(c::ResultValue::Report(
                    c::DocumentText::complete(fixtures::first_run::text(b"Configured")),
                )))));
            }
            let root = configuring_task_root(task, true, if configuring { 37 } else { 1 });
            for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
                assert_eq!(style_position(view.tree().nodes(), crate::look::SESSION_FOOT), None);
                assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_none());
                assert!(fixtures::find(view.tree(), Role::Button, b"Send").is_none());
                assert_eq!(
                    style_position(view.tree().nodes(), crate::look::CHOICE).is_some(),
                    !configuring && !ended && !result
                );
            }
        }
    }
}

#[test]
fn configuring_chats_keep_message_refusals_below_the_entry() {
    use temper_client_domain::{self as d, pages::Act};
    let mut root = configuring_task_root(configuring_task(), true, 37);
    let mut out = skein_lib::Queue::uncapped();
    for event in [
        d::Event::Act { action: d::Action::Pages(Act::MessageWords { words: b"My words".as_slice().into() }) },
        d::Event::Act { action: d::Action::Pages(Act::Send) },
        d::Event::Client { event: jig_client::Event::Stored },
    ] {
        chat_event(&mut root, event, &mut out);
    }
    let mut message_attempt = None;
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
        } = request
        {
            message_attempt = Some(attempt);
        }
    }
    let attempt = message_attempt.expect("message sent");
    chat_event(
        &mut root,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Refused {
                        refusal: c::Refusal::NotOffered,
                        at: None,
                    })),
                },
            },
        },
        &mut skein_lib::Queue::uncapped(),
    );
    for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        let tree = view.tree();
        let foot = styled(tree, crate::look::SESSION_FOOT);
        let children = subtree(tree, foot);
        let send = named_position(children, Role::Button, Some(b"Send")).unwrap();
        let refusal = named_position(children, Role::Status, Some(b"This act is not offered")).unwrap();
        assert!(send < refusal);
        assert!(named_position(children, Role::TextArea, Some(b"Message")).is_some());
    }
}

#[test]
fn configuring_chats_keep_the_conversation_and_proposals_whole() {
    use fixtures::first_run as run;
    let activity = || {
        let mut rows = List::with_capacity(1);
        rows.push(run::turn(
            1,
            ATTEMPT,
            1,
            &[run::message(
                c::Speaker::Assistant,
                &[c::Block::Text(run::text(b"Here is my proposal")), run::call(1, b"read", b"{}", None)],
            )],
            512,
        ))
        .unwrap();
        c::ActivityPage { rows, before: None, after: None, cut: false }
    };
    let plain = fixtures::chat::proposal_in_layout(Some(activity()), None, false, true);
    let configuring = fixtures::chat::proposal_in_layout(Some(activity()), None, true, true);
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut plain_view = fixtures::view_of(words.clone());
        let mut configuring_view = fixtures::view_of(words);
        let plain_tree = fixtures::build(&mut plain_view, &plain);
        let configuring_tree = fixtures::build(&mut configuring_view, &configuring);
        assert!(style_position(configuring_tree.nodes(), crate::look::SESSION).is_some());
        assert!(style_position(plain_tree.nodes(), crate::look::SESSION).is_some());
        let original = subtree(plain_tree, styled(plain_tree, crate::look::THREAD));
        let session = subtree(configuring_tree, styled(configuring_tree, crate::look::THREAD));
        assert_eq!(original.len(), session.len());
        for (a, b) in original.iter().zip(session) {
            assert_eq!((a.role, &a.name, &a.text, &a.styles), (b.role, &b.name, &b.text, &b.styles));
        }
        assert!(style_position(configuring_tree.nodes(), crate::look::TASK_CARD).is_some());
        assert!(skein_lib::bytes::find(&fixtures::text(configuring_tree), b"Delegated work").is_some());
        assert_eq!(named_count(configuring_tree, Role::Button, b"Accept"), 1);
        assert!(fixtures::find(configuring_tree, Role::Disclosure, b"1 tool call").is_some());
        assert!(skein_lib::bytes::find(&fixtures::text(configuring_tree), b"Here is my proposal").is_some());
    }
}

fn goals_navigation(root: &temper_client_domain::Domain, shown: bool) {
    let view = fixtures::browser_page_view(root);
    // The rail and the phone's tab bar follow the same Goals visibility.
    for style in [crate::look::RAIL_NAV, crate::look::TABBAR] {
        let mut at = None;
        for (index, node) in view.tree().nodes().iter().enumerate() {
            if node.styles.as_slice().contains(&style) {
                at = Some(index);
                break;
            }
        }
        let children = subtree(view.tree(), at.unwrap());
        let mut found = false;
        for node in children {
            found |= node.role == Role::Link && node.name.as_deref() == Some(b"Goals");
        }
        assert_eq!(found, shown);
    }
}
#[test]
fn goals_navigation_shows_when_the_setting_is_on() {
    let root = fixtures::root_with_ways(
        temper_client_domain::Address::Home { project: fixtures::PROJECT },
        Some(temper_client_domain::forge::ways::Ways { goals: true, ..temper_client_domain::forge::ways::Ways::STOCK }),
    );
    goals_navigation(&root, true);
}
#[test]
fn goals_navigation_hides_an_existing_goal_until_kind_seven_shows_it() {
    let (mut root, me, _) = fixtures::root_at(
        temper_client_domain::Address::Home { project: fixtures::PROJECT },
        &fixtures::domain_limits(),
    );
    super::rail::with_goal_fact(&mut root, me, Some(true));
    goals_navigation(&root, false);
}

#[test]
fn every_chat_bar_orders_its_byline_status_allowed_acts_and_spend_and_links_its_path() {
    use crate::look;
    use temper_client_domain::Address;
    for configuring in [false, true] {
        for allowed in [false, true] {
            let mut task = configuring_task();
            if !allowed {
                task.acts.clear();
            }
            let root = configuring_task_root(task, true, if configuring { 37 } else { 1 });
            for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
                let tree = view.tree();
                let top = styled(tree, look::SESSION_TOP);
                let right = styled(tree, look::SESSION_RIGHT);
                let controls = subtree(tree, right);
                let byline = style_position(controls, look::BYLINE).expect("byline in the right part");
                let status = named_position(controls, Role::Status, Some(b"Parked")).expect("status");
                let spend =
                    named_position(controls, Role::Paragraph, Some("7 spent · of 7".as_bytes())).expect("spend");
                assert!(byline < status && status < spend);
                for name in [b"Stop".as_slice(), b"Close"] {
                    assert_eq!(named_position(controls, Role::Button, Some(name)).is_some(), allowed);
                }
                if allowed {
                    let stop = named_position(controls, Role::Button, Some(b"Stop")).unwrap();
                    let close = named_position(controls, Role::Button, Some(b"Close")).unwrap();
                    assert!(status < stop && stop < close && close < spend);
                }
                let path = if configuring { b"Settings".as_slice() } else { b"Chats" };
                let mut links = List::with_capacity(2);
                for node in subtree(tree, top) {
                    if node.role == Role::Link && node.name.as_deref() == Some(path) {
                        links.push(node.id).expect("phone back and desktop path");
                    }
                }
                assert_eq!(links.len(), 2, "phone back and desktop path");
                let address = if configuring {
                    Address::Settings { project: fixtures::PROJECT }
                } else {
                    Address::Chats { filter: temper_client_domain::ChatsFilter::All }
                };
                for link in &links {
                    assert_eq!(
                        crate::href(&mut view, *link).as_deref(),
                        Some(temper_client_domain::address::print(&address).as_ref())
                    );
                }
            }
        }
    }
}

#[test]
fn the_chat_bar_marks_yours_and_waiting_states_as_needing_the_person() {
    use crate::look;
    for (line, token) in super::chats::status_rows() {
        let mut task = configuring_task();
        task.chip = line.chip;
        task.waits_on = line.waits_on;
        let root = configuring_task_root(task, true, 1);
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let controls = subtree(view.tree(), styled(view.tree(), look::SESSION_RIGHT));
            let status = &controls[named_position(controls, Role::Status, None).expect("bar status")];
            assert_eq!(
                status.styles.as_slice().contains(&look::STATE_NEEDS),
                token == look::STATUS_YOURS || token == look::STATUS_WAITING
            );
            assert!(status.styles.as_slice().contains(&token));
        }
    }
}

#[test]
fn the_session_send_arrow_waits_for_words_in_every_chat() {
    use temper_client_domain::{self as d, pages::Act};
    for configuring in [false, true] {
        let mut root = configuring_task_root(configuring_task(), true, if configuring { 37 } else { 1 });
        for (words, disabled) in [(b"".as_slice(), true), (b"Hello", false)] {
            fixtures::root_event(
                &mut root,
                d::Event::Act { action: d::Action::Pages(Act::MessageWords { words: words.into() }) },
            );
            for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
                let send = fixtures::find(view.tree(), Role::Button, b"Send").expect("send arrow");
                assert_eq!(
                    view.tree().find(send).unwrap().states.as_slice().contains(&skein_ui::State::Disabled),
                    disabled
                );
            }
        }
    }
}

#[test]
fn working_dots_follow_each_status_in_the_browser_alone_including_configuring_chats() {
    use crate::look;
    for (line, token) in super::chats::status_rows() {
        for configuring in [false, true] {
            let mut task = configuring_task();
            task.chip = line.chip.clone();
            task.waits_on = line.waits_on.clone();
            let root = configuring_task_root(task, true, if configuring { 37 } else { 1 });
            for (view, browser) in [(fixtures::page_view(&root), false), (fixtures::browser_page_view(&root), true)] {
                let tree = view.tree();
                let dots = style_position(tree.nodes(), look::TYPING);
                assert_eq!(
                    dots.is_some(),
                    browser && token == look::STATUS_ACTIVE,
                    "{token:?}, configuring={configuring}, browser={browser}"
                );
                if let Some(dots) = dots {
                    assert_eq!(tree.nodes()[dots].name.as_deref(), Some(b"Writing".as_slice()));
                    assert_eq!(tree.nodes()[dots].role, Role::Slot);
                    assert_eq!(subtree(tree, dots).len(), 3, "three dots");
                    let log = named_position(tree.nodes(), Role::Log, None).unwrap();
                    assert_eq!(dots + 4, log + usize::try_from(tree.nodes()[log].size).unwrap(), "dots end the log");
                }
            }
        }
    }
}

#[test]
fn provisional_text_replaces_working_dots_and_agent_task_logs_never_have_them() {
    use crate::pages::chat::{Drawn, Speakers, conversation_with_times};
    let mut task = configuring_task();
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Due);
    let root = configuring_task_root(task, true, 1);
    for provisional_text in [false, true] {
        let thread = if provisional_text {
            provisional()
        } else {
            Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT, live: TEXT })
        };
        for agent_page in [false, true] {
            let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
            let mut bindings = crate::frame::Bindings::new(fixtures::tree_limits().bindings);
            conversation_with_times(
                &mut b,
                &thread,
                &fixtures::browser_words(),
                (LINES, TEXT),
                Speakers { person: Some(PARTY), requester: true },
                (
                    if agent_page { Drawn::Lines } else { Drawn::ChatPage { request: None, root: &root } },
                    Some(&mut crate::words::Times::new(fixtures::NOW)),
                ),
                &mut bindings,
            );
            let tree = b.finish();
            assert_eq!(style_position(tree.nodes(), crate::look::TYPING).is_some(), !provisional_text && !agent_page);
            assert_eq!(style_position(tree.nodes(), crate::look::THREAD_INNER).is_some(), !agent_page);
            if provisional_text {
                assert!(style_position(tree.nodes(), crate::look::BY_AGENT).is_some());
                assert!(fixtures::find(&tree, Role::Status, b"Writing...").is_some());
            }
        }
    }
}

#[test]
fn the_log_has_one_column_with_older_before_turns() {
    let mut limits = fixtures::domain_limits();
    limits.client.domain.rows = temper_client_domain::pages::chat::LINES;
    let mut rows = List::with_capacity(limits.client.domain.rows);
    for position in 1..=limits.client.domain.rows {
        rows.push(crate::tests::documents::line(
            u64::from(position),
            Wall::EPOCH,
            c::Activity::Words {
                message: u64::from(position),
                by: c::Requester::Person(PARTY),
                kind: c::MessageKind::Words,
                words: fixtures::first_run::text(b"A retained message"),
            },
        ))
        .unwrap();
    }
    let (mut root, _, watch, _, _, _) = fixtures::root_at_with_forges(
        temper_client_domain::Address::Chat { task: TASK },
        &limits,
        b"Shop",
        None,
        Some(c::ActivityPage {
            rows,
            before: Some(c::Cursor::new(Box::from(*b"older"), TEXT).unwrap()),
            after: None,
            cut: false,
        }),
        None,
    );
    deliver_chat(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(configuring_task())));
    for words in [fixtures::words(), fixtures::browser_words()] {
        let bounds = crate::Limits::of(&limits, &words).unwrap();
        let mut view = crate::View::new(&bounds, &limits, words, fixtures::FORGE);
        let tree = fixtures::build(&mut view, &root);
        let thread = styled(tree, crate::look::THREAD);
        let log = named_position(tree.nodes(), Role::Log, Some(b"Conversation")).unwrap();
        let inner = styled(tree, crate::look::THREAD_INNER);
        assert_eq!(log, thread + 1);
        assert_eq!(tree.nodes()[thread].size, tree.nodes()[log].size + 1);
        assert_eq!(inner, log + 1);
        assert_eq!(tree.nodes()[log].size, tree.nodes()[inner].size + 1);
        let column = subtree(tree, inner);
        let older = named_position(column, Role::Button, Some(b"Older")).unwrap();
        assert!(column[older].styles.as_slice().contains(&crate::look::CHATS_MORE));
        let message = named_position(column, Role::Paragraph, Some(b"A retained message")).unwrap();
        assert!(older < message);
    }
}

#[test]
fn a_note_turn_draws_no_cut_words() {
    use fixtures::first_run as run;
    let thread = conversation_turn(
        &[run::message(
            c::Speaker::Assistant,
            &[
                run::call(1, b"note", b"", Some("Note 1 revision 1 written · 12 lines".as_bytes())),
                c::Block::Text(run::text(b"A long reply, also cut")),
            ],
        )],
        true,
    );
    let tree = tree(&thread);
    assert!(fixtures::find(&tree, Role::Status, b"Earlier text omitted").is_none());
    assert!(fixtures::find(&tree, Role::Button, b"Show").is_some());
}

#[test]
fn whole_turn_limits_fit_the_shells_receiving_values() {
    let domain = whole_turn_shell_limits();
    for shell in [crate::words::Shell::Browser, crate::words::Shell::Terminal] {
        let words = crate::words::Words::english(fixtures::FORGE, domain.client.domain.text, shell).unwrap();
        let limits = crate::Limits::of(&domain, &words).expect("whole shell turns have representable bounds");
        assert!(crate::valid(&limits, &domain, &words));
    }
}

fn whole_turn_shell_limits() -> temper_client_domain::Limits {
    let mut domain = fixtures::domain_limits();
    let receiving = &mut domain.client.domain;
    receiving.text = 512;
    receiving.turn_text = 131_072;
    receiving.long_text = 131_072;
    receiving.rows = 16;
    receiving.reads = 32;
    receiving.delegates = 80;
    receiving.turn_bytes = 1_048_576;
    receiving.document_bytes = 1_048_576;
    receiving.requests.bytes = 16_777_216;
    receiving.ask_bytes = 4_194_304;
    domain
}

fn full_turn_window(domain: &temper_client_domain::Limits, input: &[u8], count: u32) -> c::ActivityPage {
    use fixtures::first_run as run;
    let mut rows = List::with_capacity(count);
    let text = c::Text::new(input.into(), domain.client.domain.turn_text).unwrap();
    for position in 1..=count {
        rows.push(run::turn(
            u64::from(position),
            ATTEMPT,
            position,
            &[run::message(c::Speaker::Assistant, &[c::Block::Text(text.clone())])],
            domain.client.domain.turn_text,
        ))
        .unwrap();
    }
    c::ActivityPage { rows, before: None, after: None, cut: false }
}

#[test]
fn a_full_window_of_dense_turn_markdown_fits_the_derived_bound() {
    const TURN_BYTES: u32 = 8192;
    let mut domain = fixtures::domain_limits();
    domain.client.domain.turn_text = TURN_BYTES;
    domain.client.domain.turn_bytes = 1_048_576;
    domain.client.domain.document_bytes = 1_048_576;
    let words = fixtures::words();
    let bounds = crate::Limits::of(&domain, &words).unwrap();
    // Short list items and quotes make many nodes; links duplicate their labels
    // and retain targets, stressing the reader's text price as well.
    for pattern in [b"- x\n".as_slice(), b"> x\n", b"[x](https://x) "] {
        let length = usize::try_from(TURN_BYTES).unwrap();
        let input = repeated_markdown(pattern, length);
        let mut thread = Thread::new(TASK, &temper_client_domain::pages::chat::thread_limits(&domain));
        thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
        thread.page(full_turn_window(&domain, &input, temper_client_domain::pages::chat::LINES));
        assert_eq!(thread.lines().len(), temper_client_domain::pages::chat::LINES);
        let mut b = skein_ui::Builder::new(&bounds.tree);
        crate::pages::chat::conversation(
            &mut b,
            &thread,
            &words,
            (domain.client.domain.rows, domain.client.domain.text),
            crate::pages::chat::Speakers { person: None, requester: false },
            crate::pages::chat::Drawn::Chat { request: None, delegations: None },
            &mut crate::frame::Bindings::new(bounds.tree.bindings),
        );
        let tree = b.finish();
        assert!(tree.len() <= bounds.tree.nodes);
        let mut bytes = 0_u32;
        for node in tree.nodes() {
            bytes = bytes.checked_add(node.text_bytes()).unwrap();
        }
        assert!(bytes <= bounds.tree.text_bytes);
    }
}

#[test]
fn a_whole_markdown_turn_builds_and_prints_at_the_shells_limits() {
    let domain = whole_turn_shell_limits();
    let input = b"Ordinary **Markdown** words.\n\n";
    let length = usize::try_from(domain.client.domain.turn_text).unwrap();
    let text = repeated_markdown(input, length);
    let activity = full_turn_window(&domain, &text, 1);
    let (mut root, _, watch, _, _, _) = fixtures::root_at_with_forges(
        temper_client_domain::Address::Chat { task: TASK },
        &domain,
        b"Shop",
        None,
        Some(activity),
        None,
    );
    deliver_chat(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(configuring_task())));
    for shell in [crate::words::Shell::Browser, crate::words::Shell::Terminal] {
        let words = crate::words::Words::english(fixtures::FORGE, domain.client.domain.text, shell).unwrap();
        let bounds = crate::Limits::of(&domain, &words).unwrap();
        let mut view = crate::View::new(&bounds, &domain, words, fixtures::FORGE);
        let tree = fixtures::build(&mut view, &root);
        let mut print = skein_ui_text::Print::new(&bounds.tree).expect("shell print bounds");
        skein_ui_text::render(tree, b"temper", None, &mut print);
        assert!(skein_lib::bytes::find(print.text.as_slice(), b"Ordinary Markdown words.").is_some());
        assert!(print.text.len() > 100_000);
    }
}

fn repeated_markdown(pattern: &[u8], length: usize) -> Box<[u8]> {
    let mut input = skein_lib::bytes::zeroed(length);
    for (at, byte) in input.iter_mut().enumerate() {
        *byte = *pattern.get(at.checked_rem(pattern.len()).unwrap()).unwrap();
    }
    input
}
