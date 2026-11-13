use super::fixtures;

use fixtures::chat::{
    ATTEMPT, BYTES, CLOSE_GRACE, COMMITTED, GRACES, LINES, MESSAGE, PARTY, TASK, TEXT, TURN, chat_header, chat_root,
    closing, messaging_root, projected_tree as tree, proposal_root, provisional, provisional_with_text, rejecting_chat,
};

use jig_client::{contract as c, thread::Thread};

use skein_lib::{List, Wall};

use skein_ui::Role;


pub(super) use super::fixtures::chat_helpers::{
    asking_task, asking_task_root, asking_task_root_owned, assert_run_line, deliver_chat, long_line_page, model_choice,
    model_effort, model_line_page, model_line_root, named_position, pause_chip, structured_thread,
};
use crate::work_tests::chat::{TURN_DAY, full_turn_window, named_count, reading_thread, reading_tree, repeated_markdown, style_position, styled, subtree, timed_drawing, timed_tree, turn_wall};


#[test]
fn no_page_offers_older() {
    let view = fixtures::page_view(&chat_with_history());
    assert!(fixtures::find(view.tree(), Role::Button, b"Older").is_none());
}


#[test]
fn sending_moves_the_logs_end_request() {
    use temper_client_domain::{Action, Event, pages::Act};
    let mut root = messaging_root();
    for (act, expected) in [
        (None, 0),
        (Some(Act::MessageWords { words: b"Hello".as_slice().into() }), 0),
        (Some(Act::Send), 1),
        (Some(Act::Latest), 2),
    ] {
        if let Some(act) = act {
            fixtures::root_event(&mut root, Event::Act { action: Action::Pages(act) });
        }
        let view = fixtures::browser_page_view(&root);
        let log = named_position(view.tree().nodes(), Role::Log, None).unwrap();
        assert_eq!(view.tree().nodes()[log].end_request, Some(expected));
    }
    let (tree, _) = reading_tree(&reading_thread(), false);
    assert_eq!(tree.nodes()[0].end_request, None);
}


#[test]
fn conversation_children_after_the_head_have_distinct_keys() {
    for drawn in [
        &tree(&structured_thread()),
        &timed_tree(&structured_thread(), timed_drawing(true), Some(PARTY), 0).0,
        fixtures::browser_page_view(&proposal_root()).tree(),
        fixtures::browser_page_view(&messaging_root()).tree(),
    ] {
        let log = named_position(drawn.nodes(), Role::Log, None).unwrap();
        let column = style_position(drawn.nodes(), crate::look::THREAD_INNER).unwrap_or(log);
        let end = column + usize::try_from(drawn.nodes()[column].size).unwrap();
        let mut child = column + 1;
        let mut started = false;
        let mut keys = List::with_capacity(drawn.len());
        while child < end {
            let node = &drawn.nodes()[child];
            started |= node.styles.as_slice().contains(&crate::look::TURN);
            if started {
                let key = node.key.expect("child after the head is keyed");
                assert!(!keys.as_slice().contains(&key), "duplicate sibling key {key:?}");
                keys.push(key).unwrap();
            }
            child += usize::try_from(node.size).unwrap();
        }
    }
}


#[test]
fn a_started_or_sent_chat_asks_for_its_composer_and_an_opened_one_does_not() {
    use temper_client_domain::{Action, Event, pages::Act};
    let mut root = messaging_root();
    for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        for node in view.tree().nodes() {
            assert_eq!(node.focus, None);
        }
    }
    fixtures::root_event(&mut root, Event::Act { action: Action::Pages(Act::Send) });
    for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        let at = named_position(view.tree().nodes(), Role::TextArea, Some(b"Message")).expect("composer");
        let composer = &view.tree().nodes()[at];
        assert_eq!(composer.focus, Some(1));
        for node in view.tree().nodes() {
            if node.id != composer.id {
                assert_eq!(node.focus, None);
            }
        }
    }
}


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
        assert!(context > styled(tree, crate::look::THREAD_INNER));
        assert!(tree.nodes()[context].styles.as_slice().contains(&crate::look::META_LINE));
        assert!(skein_lib::bytes::find(&fixtures::node_text(tree, context), "Shop · acme/shop".as_bytes()).is_some());
    }
}


#[test]
fn the_session_heading_keeps_its_name_and_shows_the_title_in_the_browser() {
    const FULL_WORDS: u32 = 102;
    // FULL_WORDS copies of "word " and the final "it" fill this title's bound.
    const TITLE_BYTES: u32 = 512;
    let mut domain = fixtures::domain_limits();
    domain.client.domain.text = TITLE_BYTES;
    domain.client.domain.long_text = TITLE_BYTES;
    domain.client.domain.turn_text = TITLE_BYTES;
    let mut title = skein_lib::Writer::new(usize::try_from(domain.client.domain.long_text).unwrap());
    for _ in 0..FULL_WORDS {
        title.put(b"word ").unwrap();
    }
    title.put(b"it").unwrap();
    let bound_title = title.finish();
    assert_eq!(bound_title.len(), usize::try_from(domain.client.domain.long_text).unwrap());
    for (title, short, hint) in [
        (b"Ship the update".as_slice(), "Ship the update", None),
        (
            b"okay let's design a small http server in python, using std lib. It should show a page with details about the moon cycle, where are we now in the cycle and when the next full moon will happen".as_slice(),
            "okay let's design a small http server in python, using std lib.",
            Some(b"okay let's design a small http server in python, using std lib. It should show a page with details about the moon cycle, where are we now in the cycle and when the next full moon will happen".as_slice()),
        ),
        (
            bound_title.as_ref(),
            "word word word word word word word word word word word word word word…",
            Some(bound_title.as_ref()),
        ),
    ] {
        let (mut root, _, watch) = fixtures::root_at(temper_client_domain::Address::Chat { task: TASK }, &domain);
        let mut task = asking_task();
        task.chip.title = c::DocumentText::complete(fixtures::first_run::text(title));
        deliver_chat(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(task)));
        let whole = crate::accounts::words::join(&["T9 · ".as_bytes(), title]);
        let numbered = crate::accounts::words::join(&["T9 · ".as_bytes(), short.as_bytes()]);
        for (shell, terminal, shown) in [
            (crate::words::Shell::Terminal, false, numbered.as_ref()),
            (crate::words::Shell::Browser, false, short.as_bytes()),
            (crate::words::Shell::Browser, true, short.as_bytes()),
        ] {
            if terminal {
                fixtures::root_event(
                    &mut root,
                    temper_client_domain::Event::Act {
                        action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Terminal(
                            temper_client_domain::pages::terminal::Act::Enter,
                        )),
                    },
                );
            }
            let words = crate::words::Words::english(fixtures::FORGE, domain.client.domain.text, shell).unwrap();
            let bounds = crate::Limits::of(&domain, &words).unwrap();
            let mut view = crate::View::new(&bounds, &domain, words, fixtures::FORGE);
            let tree = fixtures::build(&mut view, &root);
            let heading = named_position(tree.nodes(), Role::Heading { level: 1 }, Some(&whole)).expect("heading");
            assert_eq!(fixtures::node_text(tree, heading).as_ref(), shown);
            assert_eq!(tree.nodes()[heading].hint.as_deref(), hint);
            assert!(fixtures::person_find(tree, Role::Region, &whole).is_none());
            if shell == crate::words::Shell::Terminal {
                let mut print = skein_ui_text::Print::new(&bounds.tree).unwrap();
                skein_ui_text::render(tree, b"temper", None, &mut print);
                let line = crate::accounts::words::join(&[b"== ", &numbered, b" =="]);
                assert!(skein_lib::bytes::find(print.text.as_slice(), &line).is_some(), "terminal heading");
            }
        }
        assert_eq!(root.pages().chat().unwrap().task(root.client()).unwrap().chip.title.bytes(), title);
    }
}


#[test]
fn a_chat_proposal_is_named_from_its_source_task() {
    let view = fixtures::page_view(&proposal_root());
    assert!(fixtures::person_find(view.tree(), Role::Item, b"Proposal from T12").is_some());
}

#[test]
fn a_held_status_line_names_only_its_exceeded_bound() {
    use temper_client_domain::{Address, Event};
    for (reason, expected) in super::chats::HELD_STATUSES {
        let (mut root, _, watch) = fixtures::root_at(Address::Chat { task: TASK }, &fixtures::domain_limits());
        let mut task = asking_task();
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
        assert_eq!(status.styles.as_slice(), [crate::look::STATUS_LINE]);
        assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), expected).is_some());
    }
}

#[test]
fn the_close_confirmation_says_how_the_chat_ends() {
    use c::RunStage as S;
    use skein_lib::Duration;
    let grace = Some(Duration::from_secs(30));
    for (name, stage, delegate, grace, expected) in [
        ("at rest", S::Idle, false, grace, "You can still read it, but it can’t be reopened."),
        ("at rest with delegate", S::Idle, true, grace, "You can still read it, but it can’t be reopened."),
        ("preparing", S::Preparing, false, grace, "You can still read it, but it can’t be reopened."),
        (
            "claimed",
            S::Claimed { attempt: ATTEMPT },
            false,
            grace,
            "The run under way gets 30 seconds to finish, then stops.",
        ),
        (
            "running",
            S::Running { attempt: ATTEMPT },
            false,
            grace,
            "The run under way gets 30 seconds to finish, then stops.",
        ),
        ("running without grace", S::Running { attempt: ATTEMPT }, false, None, "The run under way stops now."),
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
        if expected == "You can still read it, but it can’t be reopened." {
            assert!(skein_lib::bytes::find(&text, b"Close chat: allow").is_none(), "{name}");
        }
        let mut expected_text = List::with_capacity(3);
        expected_text.push("You can still read it, but it can’t be reopened.".as_bytes()).unwrap();
        if expected != "You can still read it, but it can’t be reopened." {
            expected_text.push(expected.as_bytes()).unwrap();
        }
        if delegate {
            expected_text.push(b"T7 is cancelled.".as_slice()).unwrap();
        }
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
    assert!(fixtures::find(fixtures::build(&mut view, &root), Role::Dialog, b"Close this chat?").is_some());
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
    assert!(skein_lib::bytes::find(&said, b"1 task below it is cancelled.").is_some());
    assert!(skein_lib::bytes::find(&said, b"Dangerous action").is_none());
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
    assert!(fixtures::find(tree, Role::Dialog, b"Reject this proposal?").is_some());
}

#[test]
fn a_chat_rejection_confirmation_keeps_the_persons_reason() {
    let root = rejecting_chat();
    let mut view = fixtures::view();
    let tree = fixtures::build(&mut view, &root);
    assert!(skein_lib::bytes::find(&fixtures::text(tree), b"Too costly").is_some());
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
fn the_composer_asks_what_should_change_until_it_is_answered() {
    use temper_client_domain::{Action, Address, Event, pages::Act};
    let check = |root: &temper_client_domain::Domain, expected: &[u8]| {
        for view in [fixtures::page_view(root), fixtures::browser_page_view(root)] {
            let id = fixtures::find(view.tree(), Role::TextArea, b"Message").expect("composer");
            let composer = view.tree().find(id).unwrap();
            assert_eq!(composer.placeholder.as_deref(), Some(expected));
        }
    };
    for send in [true, false] {
        let source = messaging_root();
        let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
        task.proposals = List::with_capacity(1);
        task.proposals.push(fixtures::chat::PROPOSAL).unwrap();
        let (mut root, _, watch) = fixtures::root_at(Address::Chat { task: TASK }, &fixtures::domain_limits());
        let watch = watch.unwrap();
        deliver_chat(&mut root, watch, c::Delivery::Task(Box::new(task.clone())));
        check(&root, b"Reply to T9");
        fixtures::root_event(
            &mut root,
            Event::Act { action: Action::Pages(Act::ChangeProposal { proposal: fixtures::chat::PROPOSAL }) },
        );
        check(&root, b"What should change?");
        if send {
            fixtures::root_event(
                &mut root,
                Event::Act {
                    action: Action::Pages(Act::MessageWords { words: b"Wait three seconds instead".as_slice().into() }),
                },
            );
            fixtures::root_event(&mut root, Event::Act { action: Action::Pages(Act::Send) });
        } else {
            task.proposals.clear();
            deliver_chat(&mut root, watch, c::Delivery::Task(Box::new(task)));
        }
        check(&root, b"Reply to T9");
    }
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
    let mut limits = fixtures::domain_limits();
    limits.client.domain.long_text = u32::try_from(over.len()).unwrap();
    for words in [&over[..bound], &over] {
        let mut view = fixtures::page_view(&root);
        let composer = fixtures::find(view.tree(), Role::TextArea, b"Message").unwrap();
        let action = crate::act(&mut view, composer, skein_ui::Act::Input { value: Box::from(words) }).unwrap();
        temper_client_domain::step(
            &mut root,
            &skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits },
            temper_client_domain::Event::Act { action },
            &mut skein_lib::Queue::uncapped(),
        );
        let view = fixtures::page_view(&root);
        let composer = fixtures::find(view.tree(), Role::TextArea, b"Message").unwrap();
        let mut held = None;
        for node in view.tree().nodes() {
            if node.id == composer {
                held = Some(node.value.as_ref().unwrap().text.as_ref());
            }
        }
        assert_eq!(held, Some(words));
    }
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
        (Role::Button, b"Pause"),
        (Role::Button, b"Close"),
    ] {
        assert!(named_position(nodes, role, Some(name)).is_some(), "{name:?}");
    }
    assert!(named_position(nodes, Role::Paragraph, Some(fixtures::task::document().chip.title.bytes())).is_none());
    assert!(fixtures::find(view.tree(), Role::Button, b"Mark seen").is_none());
    assert!(fixtures::find(view.tree(), Role::Button, b"Terminal, full screen").is_none());
    let browser = fixtures::browser_page_view(&fresh);
    assert!(fixtures::find(browser.tree(), Role::Button, b"Terminal, full screen").is_none());
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
    let row = styled(tree, crate::look::FOOT_ROW);
    assert_eq!(row, entry + usize::try_from(tree.nodes()[entry].size).unwrap());
    assert_eq!(hint, row + 1);
    assert_eq!(
        tree.nodes()[hint].name.as_deref(),
        Some("Enter sends · Shift+Enter for a new line · Ctrl+\\ switches the draft".as_bytes())
    );
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

#[expect(clippy::too_many_lines, reason = "the fixture carries every root obligation exhaustively")]
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
            | Request::Screen { .. }
            | Request::KeepLook { .. }
            | Request::KeepSignIn { .. }
            | Request::Tab { .. }
            | Request::Copy { .. }
            | Request::Push(_)
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
fn reading_is_offered_after_the_first_page_with_more() {
    let view = fixtures::page_view(&chat_with_history());
    let nodes = subtree(view.tree(), styled(view.tree(), crate::look::THREAD));
    assert_eq!(nodes[0].role, Role::Log);
    assert!(nodes[1].styles.as_slice().contains(&crate::look::THREAD_INNER));
    assert!(nodes[0].binding.is_some());
}

pub(crate) fn answered_message(answer: c::Answer) -> temper_client_domain::Domain {
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
    let mut task = asking_task();
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
fn the_status_line_counts_the_delegates_it_waits_on() {
    // One and two distinguish the shared goal phrase's plural forms.
    for (live, expected) in [(1, b"waiting on 1 delegate".as_slice()), (2, b"waiting on 2 delegates")] {
        let view = header_with_status(c::WaitsOn::Delegates, live, c::WakeRule::Never, c::Requester::Person(PARTY));
        let status = fixtures::find(view.tree(), Role::Status, expected).expect("delegate status");
        assert_eq!(view.tree().find(status).unwrap().styles.as_slice(), [crate::look::STATUS_LINE]);
    }
}


#[test]
fn the_status_line_waits_on_the_requesting_persons_words() {
    // Batching also wakes on words, after its configured count or age.
    const BATCH_COUNT: u32 = 2;
    let person = fixtures::root_in_project(None);
    let requester = person.pages().me(person.client()).unwrap().party.number;
    for rule in
        [c::WakeRule::Immediate, c::WakeRule::Batch { count: BATCH_COUNT, age: skein_lib::Duration::from_secs(1) }]
    {
        let view = header_with_status(c::WaitsOn::News, 0, rule, c::Requester::Person(requester));
        let status = fixtures::find(view.tree(), Role::Status, b"waiting on your words").expect("words status");
        assert_eq!(view.tree().find(status).unwrap().styles.as_slice(), [crate::look::STATUS_LINE]);
    }
}


#[test]
fn the_status_line_keeps_jigs_word_without_the_requesters_words_wake() {
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
            "Closed at 00:00",
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
        (
            "report",
            c::ChipEnding::Done(report),
            reported,
            Some(fixtures::NOW),
            "Closed at 00:00 · its report, “A report”",
        ),
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
    assert!(style_position(view.tree().nodes(), crate::look::MODEL).is_some());
    assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_some());
}


#[test]
fn a_chats_end_and_its_phase_keep_the_supplied_clock() {
    use fixtures::first_run::text;
    for (ending, expected) in [
        (
            c::ChipEnding::Done(c::ResultValue::Report(c::DocumentText::complete(text(b"A report")))),
            "Closed at 00:00 · its report, “A report”",
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
        assert!(fixtures::find(view.tree(), Role::Status, b"Closed").is_none());
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


#[test]
fn the_playground_chats_bar_names_the_default_model() {
    let root = fixtures::playground_root(temper_client_domain::Address::Chat { task: TASK }, fixtures::PROJECT, true);
    let view = fixtures::browser_page_view(&root);
    let bar = styled(view.tree(), crate::look::SESSION_TOP);
    let byline = style_position(subtree(view.tree(), bar), crate::look::BYLINE).expect("byline");
    let node = &view.tree().nodes()[bar + 1 + byline];
    assert_eq!(node.description.as_deref().or(node.name.as_deref()), Some(b"T9".as_slice()));
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


fn resting_task() -> c::Task {
    let mut task = asking_task();
    let source = messaging_root();
    task.requester = c::Requester::Person(source.pages().me(source.client()).unwrap().party.number);
    task.waits_on = c::WaitsOn::News;
    task.wake.words = c::WakeRule::Immediate;
    task.chip.details.details.live_delegates = 0;
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Idle);
    task
}

#[test]
fn asking_chats_keep_the_settings_layout_and_controls_in_both_shells() {
    use crate::look;
    use temper_client_domain::{self as d, Address, pages::Act};
    let mut root = asking_task_root(asking_task(), true, 37);
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
            (Role::Link, b"Ask temper"),
            (Role::Paragraph, b"7 spent \xc2\xb7 of 7"),
            (Role::Button, b"Pause"),
            (Role::Button, b"Close"),
        ] {
            assert!(named_position(top_nodes, role, Some(name)).is_some(), "{name:?}");
        }
        let controls = subtree(tree, bar);
        assert!(named_position(controls, Role::Button, Some(b"Terminal, full screen")).is_none());
        let positions = [
            named_position(controls, Role::Button, Some(b"Start over")).unwrap(),
            named_position(controls, Role::Button, Some(b"Pause")).unwrap(),
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
        let feedback = styled(tree, look::CHOICE);
        assert_eq!(feedback, thread + usize::try_from(tree.nodes()[thread].size).unwrap());
        assert_eq!(foot, feedback + usize::try_from(tree.nodes()[feedback].size).unwrap());
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
        let stop = fixtures::find(tree, Role::Button, b"Pause").unwrap();
        let close = fixtures::find(tree, Role::Button, b"Close").unwrap();
        let back = tree.nodes()[styled(tree, look::BACK)].id;
        let hint = styled(tree, look::SESSION_HINT);
        assert!(entry < hint && hint < foot + usize::try_from(tree.nodes()[foot].size).unwrap());
        assert_eq!(
            tree.nodes()[hint].name.as_deref(),
            Some(b"Enter sends \xc2\xb7 Shift+Enter for a new line \xc2\xb7 Ctrl+\\ switches the draft".as_slice())
        );
        for style in [look::ACCOUNT_LINE, look::COMPOSER_AREA, look::STARTERS] {
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
            Some(d::address::print(&Address::Ask { project: fixtures::PROJECT }).as_ref())
        );
    }
    let view = fixtures::page_view(&root);
    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
    skein_ui_text::render(view.tree(), b"temper", None, &mut print);
    let printed = print.text.as_slice();
    for words in
        ["How work runs", "T9 · Hello", "Conversation:", "Message", "Send", "Enter sends · Shift+Enter for a new line"]
    {
        assert!(skein_lib::bytes::find(printed, words.as_bytes()).is_some(), "{words}");
    }
    assert!(skein_lib::bytes::find(printed, b"Change model").is_none());
}


#[test]
fn only_a_delivered_template_five_executor_changes_chat_path_and_navigation() {
    use crate::look;
    for (listed, charter, asking) in [(true, 37, true), (true, 1, false), (false, 37, false), (false, 1, false)] {
        let root = asking_task_root(asking_task(), listed, charter);
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let tree = view.tree();
            assert!(style_position(tree.nodes(), look::SESSION).is_some());
            assert_eq!(style_position(tree.nodes(), look::AGENT_PAGE), None);
            for style in [look::RAIL, look::TABBAR] {
                if let Some(at) = style_position(tree.nodes(), style) {
                    for name in [b"Ask temper".as_slice(), b"Work"] {
                        let nodes = subtree(tree, at);
                        let link = named_position(nodes, Role::Link, Some(name));
                        if name == b"Ask temper" && !listed {
                            assert!(link.is_none());
                            continue;
                        }
                        let link = link.unwrap();
                        assert_eq!(
                            nodes[link].states.as_slice().contains(&skein_ui::State::Current),
                            if name == b"Ask temper" { asking } else { !asking }
                        );
                    }
                }
            }
            assert!(style_position(tree.nodes(), look::SESSION_FOOT).is_some());
            assert!(style_position(tree.nodes(), look::CHOICE).is_some());
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
    for asking in [false, true] {
        for (offered, ended, result) in [(false, false, false), (true, true, false), (true, false, true)] {
            let mut task = asking_task();
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
            let root = asking_task_root(task, true, if asking { 37 } else { 1 });
            for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
                assert_eq!(style_position(view.tree().nodes(), crate::look::SESSION_FOOT), None);
                assert!(fixtures::find(view.tree(), Role::TextArea, b"Message").is_none());
                assert!(fixtures::find(view.tree(), Role::Button, b"Send").is_none());
                assert_eq!(style_position(view.tree().nodes(), crate::look::CHOICE).is_some(), !ended && !result);
                assert!(fixtures::find(view.tree(), Role::Button, b"+ Draft").is_none());
                assert!(style_position(view.tree().nodes(), crate::look::STATUS_LINE).is_none());
                if asking {
                    assert_eq!(fixtures::find(view.tree(), Role::Group, b"The draft").is_some(), !ended && !result);
                }
            }
        }
    }
}


#[test]
fn asking_chats_keep_message_refusals_below_the_entry() {
    use temper_client_domain::{self as d, pages::Act};
    let mut root = asking_task_root(asking_task(), true, 37);
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
fn asking_chats_keep_the_conversation_and_proposals_whole() {
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
    let asking = fixtures::chat::proposal_in_layout(Some(activity()), None, true, true);
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut plain_view = fixtures::view_of(words.clone());
        let mut asking_view = fixtures::view_of(words);
        let plain_tree = fixtures::build(&mut plain_view, &plain);
        let asking_tree = fixtures::build(&mut asking_view, &asking);
        assert!(style_position(asking_tree.nodes(), crate::look::SESSION).is_some());
        assert!(style_position(plain_tree.nodes(), crate::look::SESSION).is_some());
        let original = subtree(plain_tree, styled(plain_tree, crate::look::THREAD));
        let session = subtree(asking_tree, styled(asking_tree, crate::look::THREAD));
        assert_eq!(original.len(), session.len());
        for (a, b) in original.iter().zip(session) {
            assert_eq!((a.role, &a.name, &a.text, &a.styles), (b.role, &b.name, &b.text, &b.styles));
        }
        assert!(style_position(asking_tree.nodes(), crate::look::TASK_CARD).is_some());
        assert!(skein_lib::bytes::find(&fixtures::text(asking_tree), b"Delegated work").is_some());
        assert_eq!(named_count(asking_tree, Role::Button, b"Accept"), 1);
        assert!(fixtures::find(asking_tree, Role::Disclosure, b"Read 1 file").is_some());
        assert!(skein_lib::bytes::find(&fixtures::text(asking_tree), b"Here is my proposal").is_some());
    }
}


fn goals_navigation(root: &temper_client_domain::Domain, shown: bool) {
    let view = fixtures::browser_page_view(root);
    // Work stays in both navigation surfaces while Goals is offered or held.
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
            found |= node.role == Role::Link && node.name.as_deref() == Some(b"Work");
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
fn goals_navigation_keeps_an_existing_goal_when_kind_seven_hides_it() {
    let (mut root, me, _) = fixtures::root_at(
        temper_client_domain::Address::Home { project: fixtures::PROJECT },
        &fixtures::domain_limits(),
    );
    super::rail::with_goal_fact(&mut root, me, Some(true));
    goals_navigation(&root, true);
}


#[test]
fn every_chat_bar_orders_its_byline_allowed_acts_group_and_spend_and_links_its_path() {
    use crate::look;
    use temper_client_domain::Address;
    for asking in [false, true] {
        for allowed in [false, true] {
            let mut task = resting_task();
            if !allowed {
                task.acts.clear();
            }
            let root = asking_task_root(task, true, if asking { 37 } else { 1 });
            for (mut view, browser) in [(fixtures::page_view(&root), false), (fixtures::browser_page_view(&root), true)]
            {
                let tree = view.tree();
                let top = styled(tree, look::SESSION_TOP);
                let right = styled(tree, look::SESSION_RIGHT);
                let controls = subtree(tree, right);
                let byline = style_position(controls, look::BYLINE);
                assert_eq!(byline.is_some(), browser);
                assert!(named_position(controls, Role::Status, None).is_none());
                assert!(named_position(controls, Role::Button, Some(b"+ Draft")).is_none());
                let group = named_position(controls, Role::Group, Some(b"The draft")).expect("group");
                let chat = named_position(controls, Role::Button, Some(b"Chat")).unwrap();
                assert!(controls[chat].states.as_slice().contains(&skein_ui::State::Pressed));
                let spend =
                    named_position(controls, Role::Paragraph, Some("7 spent · of 7".as_bytes())).expect("spend");
                if let Some(byline) = byline {
                    assert!(byline < group);
                }
                assert!(named_position(controls, Role::Button, Some(b"Terminal, full screen")).is_none());
                assert!(group < spend);
                let start_over = named_position(controls, Role::Button, Some(b"Start over"));
                assert_eq!(start_over.is_some(), asking);
                if let Some(start) = start_over {
                    assert!(start < group);
                    let tag = style_position(controls, look::CHATS_TAG).unwrap();
                    assert!(tag < start);
                }
                for name in [b"Pause".as_slice(), b"Close"] {
                    assert_eq!(named_position(controls, Role::Button, Some(name)).is_some(), allowed);
                }
                if allowed {
                    let stop = named_position(controls, Role::Button, Some(b"Pause")).unwrap();
                    let close = named_position(controls, Role::Button, Some(b"Close")).unwrap();
                    assert!(stop < close && close < group);
                    if let Some(start) = start_over {
                        assert!(start < stop);
                    }
                }
                let path = if asking { b"Ask temper".as_slice() } else { b"Chats" };
                let trail = subtree(tree, styled(tree, look::TRAIL));
                assert_eq!(trail.len(), 2, "a chat's trail is one link, naming no project");
                assert_eq!(trail[0].role, Role::Link);
                assert_eq!(trail[0].name.as_deref(), Some(path));
                let crumbs = subtree(tree, styled(tree, look::CRUMBS));
                assert_eq!(crumbs[0].styles.as_slice(), [look::BACK]);
                let trail_at = usize::try_from(crumbs[0].size).unwrap();
                assert_eq!(crumbs[trail_at].styles.as_slice(), [look::TRAIL]);
                let heading_at = trail_at.checked_add(usize::try_from(crumbs[trail_at].size).unwrap()).unwrap();
                assert_eq!(crumbs[heading_at].role, Role::Heading { level: 1 });
                let mut links = List::with_capacity(2);
                for node in subtree(tree, top) {
                    if node.role == Role::Link && node.name.as_deref() == Some(path) {
                        links.push(node.id).expect("phone back and desktop path");
                    }
                }
                assert_eq!(links.len(), 2, "phone back and desktop path");
                let address = if asking {
                    Address::Ask { project: fixtures::PROJECT }
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
fn the_status_line_is_the_composers_first_row_at_rest_and_working() {
    use crate::look;
    for working in [false, true] {
        let mut task = resting_task();
        if working {
            task.waits_on = c::WaitsOn::Nothing;
            task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
        }
        let root = asking_task_root(task, true, 37);
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            let tree = view.tree();
            let foot = styled(tree, look::SESSION_FOOT);
            let line = styled(tree, look::STATUS_LINE);
            assert_eq!(line, foot + 1);
            assert_eq!(tree.nodes()[line].role, Role::Status);
            let expected = if working { b"Working".as_slice() } else { b"waiting on your words" };
            assert_eq!(tree.nodes()[line].name.as_deref(), Some(expected));
            assert_eq!(tree.nodes()[line].styles.as_slice().contains(&look::STATUS_WORKING), working);
            assert!(!tree.nodes()[line].styles.as_slice().contains(&look::STATE_NEEDS));
            let star = styled(tree, look::STATUS_STAR);
            assert_eq!(star, line + 1);
            assert_eq!(fixtures::node_text(tree, star).as_ref(), "✦".as_bytes());
            assert!(line < styled(tree, look::SESSION_COMPOSER));
        }
    }
}


#[test]
fn the_session_send_arrow_waits_for_words_in_every_chat() {
    use temper_client_domain::{self as d, pages::Act};
    for asking in [false, true] {
        let mut root = asking_task_root(asking_task(), true, if asking { 37 } else { 1 });
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
fn working_dots_follow_each_status_in_the_browser_alone_including_asking_chats() {
    use crate::look;
    for (line, token) in super::chats::status_rows() {
        for asking in [false, true] {
            let mut task = asking_task();
            task.chip = line.chip.clone();
            task.waits_on = line.waits_on.clone();
            let root = asking_task_root(task, true, if asking { 37 } else { 1 });
            for (view, browser) in [(fixtures::page_view(&root), false), (fixtures::browser_page_view(&root), true)] {
                let tree = view.tree();
                let dots = style_position(tree.nodes(), look::TYPING);
                assert_eq!(
                    dots.is_some(),
                    browser && token == look::STATUS_ACTIVE,
                    "{token:?}, asking={asking}, browser={browser}"
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


fn full_window(dropped: bool) -> (temper_client_domain::Domain, temper_client_domain::Limits) {
    let mut limits = fixtures::domain_limits();
    limits.client.domain.rows = LINES;
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
            after: if dropped { Some(c::Cursor::new(b"newer".as_slice().into(), TEXT).unwrap()) } else { None },
            cut: false,
        }),
        None,
    );
    deliver_chat(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(asking_task())));
    (root, limits)
}

#[test]
fn the_log_has_one_column_for_its_turns() {
    let (root, limits) = full_window(false);
    for words in [fixtures::words(), fixtures::browser_words()] {
        let browser = words.shell == crate::words::Shell::Browser;
        let bounds = crate::Limits::of(&limits, &words).unwrap();
        let mut view = crate::View::new(&bounds, &limits, words, fixtures::FORGE);
        let tree = fixtures::build(&mut view, &root);
        let thread = styled(tree, crate::look::THREAD);
        let log = named_position(tree.nodes(), Role::Log, Some(b"Conversation")).unwrap();
        let inner = styled(tree, crate::look::THREAD_INNER);
        assert_eq!(log, thread + 1);
        assert_eq!(tree.nodes()[thread].size, tree.nodes()[log].size + if browser { 2 } else { 1 });
        assert_eq!(inner, log + 1);
        assert_eq!(tree.nodes()[log].size, tree.nodes()[inner].size + 1);
        let column = subtree(tree, inner);
        assert!(named_position(column, Role::Paragraph, Some(b"A retained message")).is_some());
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
    deliver_chat(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(asking_task())));
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


#[test]
fn latest_is_a_browsers_alone_and_goes_to_the_log_end() {
    let root = messaging_root();
    for (mut view, browser) in [(fixtures::page_view(&root), false), (fixtures::browser_page_view(&root), true)] {
        let found = fixtures::find(view.tree(), Role::Button, b"Latest");
        assert_eq!(found.is_some(), browser);
        if let Some(id) = found {
            let nodes = view.tree().nodes();
            let pill = named_position(nodes, Role::Button, Some(b"Latest")).unwrap();
            let log = named_position(nodes, Role::Log, None).unwrap();
            assert_eq!(pill, log + usize::try_from(nodes[log].size).unwrap());
            assert!(nodes[pill].styles.as_slice().contains(&crate::look::LATEST));
            assert!(nodes[pill].states.as_slice().contains(&skein_ui::State::ToLogEnd));
            let thread = styled(view.tree(), crate::look::THREAD);
            assert_eq!(pill + 1, thread + usize::try_from(nodes[thread].size).unwrap());
            assert!(crate::act(&mut view, id, skein_ui::Act::Press).is_none());
        }
    }
}


#[test]
fn latest_is_bound_only_where_the_window_let_the_end_go() {
    use temper_client_domain::{Action, pages::Act};
    for dropped in [false, true] {
        let (root, limits) = full_window(dropped);
        let words = fixtures::browser_words();
        let bounds = crate::Limits::of(&limits, &words).unwrap();
        let mut view = crate::View::new(&bounds, &limits, words, fixtures::FORGE);
        fixtures::build(&mut view, &root);
        let id = fixtures::find(view.tree(), Role::Button, b"Latest").expect("latest");
        assert_eq!(
            crate::act(&mut view, id, skein_ui::Act::Press),
            if dropped { Some(Action::Pages(Act::Latest)) } else { None }
        );
    }
}


#[test]
fn the_head_says_when_an_earlier_read_is_out() {
    use jig_client::thread::{Reading, window::Edge};
    for (reading, at_start, loading) in [
        (None, false, false),
        (Some(Reading::Older), false, true),
        (Some(Reading::Newer), false, false),
        (Some(Reading::Latest { known: 0 }), false, false),
        (None, true, false),
        (Some(Reading::Older), true, false),
    ] {
        let mut thread =
            Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT, live: TEXT });
        thread.reading = reading;
        thread.lines.edges.after =
            if at_start { Edge::End } else { Edge::More(c::Cursor::new(b"older".as_slice().into(), TEXT).unwrap()) };
        let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
        crate::pages::chat::older(&mut b, &thread, &fixtures::browser_words());
        let tree = b.finish();
        let status = fixtures::find(&tree, Role::Status, b"Loading earlier messages");
        assert_eq!(status.is_some(), loading);
        assert!(fixtures::find(&tree, Role::Button, b"Older").is_none());
        if let Some(status) = status {
            let node = tree.find(status).unwrap();
            assert_eq!(node.styles.as_slice(), [crate::look::THREAD_LOADING]);
            assert_eq!(fixtures::text(&tree).as_ref(), "Loading earlier messages…".as_bytes());
        }
    }
    let mut root = chat_with_history();
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Older),
        },
    );
    let view = fixtures::browser_page_view(&root);
    let nodes = view.tree().nodes();
    let inner = styled(view.tree(), crate::look::THREAD_INNER);
    assert_eq!(nodes[inner + 1].name.as_deref(), Some(b"Loading earlier messages".as_slice()));
}


#[test]
fn a_closed_chat_says_what_it_ran_on() {
    use crate::look;
    for (latest, own, preview, charter, caught_up, closed, expected) in [
        (
            Some(model_effort("gpt-6-astra", Some("low"))),
            Some(model_choice("different-next-run", Some("high"))),
            None,
            1,
            true,
            true,
            Some("T9 · GPT-6 Astra · low"),
        ),
        (Some(model_effort("gpt-6-astra", None)), None, None, 1, true, true, Some("T9 · GPT-6 Astra")),
        (
            Some(model_effort("claude-opus-5-5", Some("high"))),
            None,
            None,
            1,
            true,
            true,
            Some("T9 · Claude Opus 5.5 · high"),
        ),
        (
            Some(model_effort("unoffered-model", Some("medium"))),
            None,
            None,
            1,
            true,
            true,
            Some("T9 · unoffered-model · medium"),
        ),
        (None, Some(model_choice("gpt-6-astra", None)), None, 1, true, true, Some("T9 · GPT-6 Astra · high")),
        (None, None, Some(1), 1, true, true, Some("T9 · GPT-6 Astra · high")),
        (None, None, Some(5), 37, true, true, Some("T9 · GPT-6 Astra · high")),
        (None, None, None, 37, true, true, Some("T9 · Using the default model")),
        (None, None, Some(1), 37, true, true, Some("T9 · Using the default model")),
        (Some(model_effort("gpt-6-astra", Some("low"))), None, Some(1), 1, false, true, None),
        (Some(model_effort("gpt-6-astra", Some("low"))), None, Some(1), 1, true, false, None),
    ] {
        let mut task = asking_task();
        task.chip.executor = c::ChipExecutor::Agent { charter };
        task.activity = if caught_up { 1 } else { 2 };
        task.llm = own;
        if !closed {
            task.acts = List::with_capacity(2);
            for act in [c::ActKind::Message, c::ActKind::Amend] {
                task.acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).unwrap();
            }
        }
        if closed {
            task.result = Some(c::TaskResult::Ending(Box::new(cancelled_by(task.requester))));
        }
        let root = model_line_root(task, latest, preview);
        assert_eq!(root.pages().chat().unwrap().thread.caught_up(), caught_up);
        for (view, browser) in [(fixtures::browser_page_view(&root), true), (fixtures::page_view(&root), false)] {
            let tree = view.tree();
            let bar = subtree(tree, styled(tree, look::SESSION_TOP));
            let byline = style_position(bar, look::BYLINE);
            let expected = expected.or(if browser { Some("T9") } else { None });
            assert_eq!(byline.is_some(), expected.is_some());
            if let Some(expected) = expected {
                let node = &bar[byline.unwrap()];
                assert_eq!(node.role, Role::Paragraph);
                assert_eq!(node.name.as_deref(), Some(expected.as_bytes()));
                let at = styled(tree, look::BYLINE);
                assert_eq!(fixtures::node_text(tree, at).as_ref(), expected.as_bytes());
            }
            if closed {
                assert!(fixtures::find(tree, Role::Disclosure, b"Model").is_none());
                assert!(style_position(tree.nodes(), look::ACCOUNT_LINE).is_none());
            } else {
                assert!(fixtures::find(tree, Role::Disclosure, b"Model").is_some());
            }
        }
    }
}


#[test]
fn a_closed_chat_ends_on_one_line() {
    use temper_client_domain::{Action, Address, Event, pages::Act};
    let mut rows = List::with_capacity(1);
    rows.push(crate::tests::documents::line(1, Wall::EPOCH, c::Activity::Decided { revision: 1, proposal: None }))
        .unwrap();
    let activity = c::ActivityPage { rows, before: None, after: None, cut: false };
    let (mut root, _, watch, _, _) = fixtures::root_at_with_activity(
        Address::Chat { task: TASK },
        &fixtures::domain_limits(),
        b"Shop",
        None,
        Some(activity),
        None,
    );
    let source = messaging_root();
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    task.activity = 1;
    task.requester = c::Requester::Person(PARTY);
    let watch = watch.unwrap();
    deliver_chat(&mut root, watch, c::Delivery::Task(Box::new(task.clone())));
    fixtures::root_event(&mut root, Event::Act { action: Action::Pages(Act::Close) });
    let mut out = skein_lib::Queue::uncapped();
    chat_event(&mut root, Event::Act { action: Action::Pages(Act::Confirm) }, &mut out);
    let mut attempt = None;
    for request in &out {
        if let temper_client_domain::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt: sent, .. } },
        } = request
        {
            attempt = Some(*sent);
        }
    }
    let attempt = attempt.expect("close sent");
    fixtures::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                        c::DocumentOutcome::Amended { task: TASK },
                    )))),
                },
            },
        },
    );
    let at = Wall::from_nanos(75_900_000_000_000); // 21:05 UTC on the fixture's epoch day.
    task.result = Some(c::TaskResult::Ending(Box::new(cancelled_by(task.requester))));
    task.chip.details.details.ended_at = Some(at);
    deliver_chat(&mut root, watch, c::Delivery::Task(Box::new(task)));
    let view = fixtures::page_view(&root);
    let tree = view.tree();
    assert!(fixtures::find(tree, Role::Paragraph, b"Closed at 21:05").is_some());
    assert!(fixtures::find(tree, Role::Status, b"Closed").is_none());
    let mut ends = 0_u32;
    for node in tree.nodes() {
        if node.styles.as_slice().contains(&crate::look::ENDED) {
            ends = ends.saturating_add(1);
        }
    }
    assert_eq!(ends, 1);
    let text = fixtures::text(tree);
    assert!(skein_lib::bytes::find(&text, b"Applied").is_none());
    assert!(skein_lib::bytes::find(&text, b"Decision made").is_none());
}


#[test]
fn every_chats_run_wait_says_why_above_the_composer_without_working_dots() {
    use crate::look;
    for (wait, owned, reason) in [
        (
            c::TaskRunWait::Account { account: None, source: None, why: c::TaskWaitWhy::Empty },
            true,
            b"No account can run Scripted model yet.".as_slice(),
        ),
        (
            c::TaskRunWait::Account { account: Some(7), source: Some(8), why: c::TaskWaitWhy::Host },
            true,
            b"LLM account's login isn't signed in on a host that can run this.",
        ),
        (c::TaskRunWait::Credential, true, b"This work needs a credential from its forge."),
        (c::TaskRunWait::Host, true, b"Waiting for other work to finish. It starts by itself."),
        (c::TaskRunWait::Host, false, b"No host has room for this, or none is connected. It starts when one does."),
        (
            c::TaskRunWait::Refused(c::TaskWaitRefused::Bounds {
                kind: c::HostKind::Worker,
                why: Some(c::AgentShortfall::Turns { charter: 1 }),
                figures: Some(c::Figures { required: 100_000, offered: 800 }),
            }),
            true,
            b"No worker can take this: its charter asks 100,000 turns, and the largest worker takes 800.",
        ),
    ] {
        super::choice::wait_pointers(reason);
        for asking in [false, true] {
            let mut task = asking_task();
            task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Due);
            task.run_wait = Some(wait);
            task.llm = Some(c::LlmChoice {
                later: None,
                account: None,
                main: c::ModelEffort { model: fixtures::first_run::text(b"demo"), effort: None },
                subagents: List::with_capacity(0),
            });
            let root = asking_task_root_owned(task, true, if asking { 37 } else { 1 }, owned);
            for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
                let tree = view.tree();
                super::choice::wait_pointers(&fixtures::text(tree));
                let feedback = named_position(tree.nodes(), Role::Paragraph, Some(reason)).expect("wait reason");
                let composer = styled(tree, look::SESSION_COMPOSER);
                assert!(feedback < composer);
                assert!(style_position(tree.nodes(), look::TYPING).is_none());
                if wait == c::TaskRunWait::Credential {
                    let link = fixtures::find(tree, Role::Link, b"Forges").expect("forge action");
                    assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/forges".as_slice()));
                }
                if let c::TaskRunWait::Account { why: c::TaskWaitWhy::Empty, .. } = wait {
                    let link = fixtures::find(view.tree(), Role::Link, b"Connect a model").expect("connect action");
                    assert_eq!(
                        crate::act(&mut view, link, skein_ui::Act::Press),
                        Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
                            address: temper_client_domain::Address::ConnectModel { way: None }
                        }))
                    );
                }
            }
        }
    }
}


pub(crate) fn assert_long_line(
    root: &temper_client_domain::Domain,
    domain: &temper_client_domain::Limits,
    bytes: &[u8],
) {
    for shell in [crate::words::Shell::Browser, crate::words::Shell::Terminal] {
        let words = crate::words::Words::english(fixtures::FORGE, domain.client.domain.text, shell).unwrap();
        let bounds = crate::Limits::of(domain, &words).unwrap();
        let mut view = crate::View::new(&bounds, domain, words, fixtures::FORGE);
        let tree = fixtures::build(&mut view, root);
        let mut whole = false;
        let mut sent = None;
        for (index, node) in tree.nodes().iter().enumerate() {
            if node.role == Role::Text && fixtures::node_text(tree, index).as_ref() == bytes {
                whole = true;
            }
            if node.styles.as_slice().contains(&crate::look::GOAL_SENT) {
                sent = Some(index);
            }
        }
        assert!(whole, "the line's whole words are a tree text");
        if let Some(page) = root.pages().task()
            && page.shape == temper_client_domain::pages::task::Shape::Tracked
        {
            let sent = sent.expect("the goal's sent words");
            assert_eq!(fixtures::node_text(tree, sent).as_ref(), bytes, "the reader's words draw in .sent");
        }
    }
}


#[test]
fn a_chats_long_line_draws_whole() {
    for length in [1149, 10_660] {
        let bytes = super::limits::repeated(b'x', length);
        let (root, domain) = long_line_page(
            temper_client_domain::Address::Chat { task: TASK },
            c::Activity::Words {
                message: 1,
                by: c::Requester::Person(PARTY),
                kind: c::MessageKind::Words,
                words: fixtures::first_run::text(&bytes),
            },
            false,
        );
        assert_long_line(&root, &domain, &bytes);
    }
}


// client/pages/README.md: this hold is the reader's own Stop on this chat.

fn pause_root(paused: bool, own: bool, release: bool) -> temper_client_domain::Domain {
    let (mut root, _, watch) =
        fixtures::root_at(temper_client_domain::Address::Chat { task: TASK }, &fixtures::domain_limits());
    let mut task = fixtures::task::document();
    task.chip.task = TASK;
    if paused {
        pause_chip(&mut task.chip);
        if !own {
            task.chip.details.details.hold.as_mut().unwrap().placed.as_mut().unwrap().owner =
                c::HoldOwner::Task { task: TASK };
        }
    }
    task.acts = List::with_capacity(4);
    for act in [c::ActKind::Stop, c::ActKind::Cancel, c::ActKind::Message, c::ActKind::Release] {
        if act != c::ActKind::Release || release {
            task.acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).unwrap();
        }
    }
    fixtures::root_event(
        &mut root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
    );
    root
}

#[test]
fn a_running_chat_says_pause_and_close() {
    let root = pause_root(false, true, true);
    for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        for (name, act) in [
            (b"Pause".as_slice(), temper_client_domain::pages::Act::Stop),
            (b"Close", temper_client_domain::pages::Act::Close),
        ] {
            let id = fixtures::find(view.tree(), Role::Button, name).expect("bar action");
            assert_eq!(crate::act(&mut view, id, skein_ui::Act::Press), Some(temper_client_domain::Action::Pages(act)));
        }
    }
}

#[test]
fn a_paused_chat_says_resume_paused_and_its_hint() {
    let root = pause_root(true, true, true);
    for mut view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        let id = fixtures::find(view.tree(), Role::Button, b"Resume").expect("resume");
        assert_eq!(
            crate::act(&mut view, id, skein_ui::Act::Press),
            Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Resume))
        );
        assert!(fixtures::find(view.tree(), Role::Button, b"Pause").is_none());
        assert!(fixtures::find(view.tree(), Role::Button, b"Close").is_some());
        let status = named_position(view.tree().nodes(), Role::Status, Some(b"Paused")).expect("paused status");
        assert_eq!(view.tree().nodes()[status].styles.as_slice(), [crate::look::STATUS_LINE]);
        let words = if fixtures::find(view.tree(), Role::Group, b"The draft").is_some() {
            b"Sending resumes this chat. \xc2\xb7 Ctrl+\\ switches the draft".as_slice()
        } else {
            b"Sending resumes this chat."
        };
        let hint = fixtures::find(view.tree(), Role::Paragraph, words).expect("resuming hint");
        assert!(view.tree().find(hint).unwrap().styles.as_slice().contains(&crate::look::HINT));
    }
}

#[test]
fn a_chat_held_otherwise_says_held_and_has_no_resume() {
    let root = pause_root(true, false, true);
    for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        assert!(fixtures::find(view.tree(), Role::Status, b"Held").is_some());
        assert!(fixtures::find(view.tree(), Role::Button, b"Resume").is_none());
        assert!(fixtures::find(view.tree(), Role::Button, b"Pause").is_some());
        assert!(
            fixtures::find(
                view.tree(),
                Role::Paragraph,
                b"Sending resumes this chat. \xc2\xb7 Ctrl+\\ switches the draft"
            )
            .is_none()
        );
    }
}

#[test]
fn a_pause_without_release_has_no_resume_or_resuming_hint() {
    let root = pause_root(true, true, false);
    for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        assert!(fixtures::find(view.tree(), Role::Button, b"Resume").is_none());
        assert!(fixtures::find(view.tree(), Role::Button, b"Pause").is_none());
        assert!(
            fixtures::find(
                view.tree(),
                Role::Paragraph,
                b"Sending resumes this chat. \xc2\xb7 Ctrl+\\ switches the draft"
            )
            .is_none()
        );
    }
}

fn answered_pause(resume: bool, answer: c::Answer) -> temper_client_domain::Domain {
    let mut root = pause_root(resume, true, true);
    let mut out = skein_lib::Queue::uncapped();
    chat_event(
        &mut root,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(if resume {
                temper_client_domain::pages::Act::Resume
            } else {
                temper_client_domain::pages::Act::Stop
            }),
        },
        &mut out,
    );
    chat_event(&mut root, temper_client_domain::Event::Client { event: jig_client::Event::Stored }, &mut out);
    let mut attempt = None;
    for request in &out {
        if let temper_client_domain::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt: token, .. } },
        } = request
        {
            attempt = Some(*token);
        }
    }
    let attempt = attempt.expect("sent act");
    chat_event(
        &mut root,
        temper_client_domain::Event::Client {
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
    root
}

#[test]
fn an_accepted_pause_or_resume_says_no_applied() {
    for resume in [false, true] {
        let root = answered_pause(
            resume,
            c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Said { task: TASK, message: MESSAGE })),
        );
        for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
            assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"Applied").is_none());
        }
    }
}

#[test]
fn a_refused_resume_says_why() {
    let root = answered_pause(true, c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None });
    for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"not offered").is_some());
    }
}


// client/view.md, 4: the message composers share the bound.
fn bounded_composers(browser: bool) -> Box<[temper_client_domain::Domain]> {
    use temper_client_domain::Address;
    let mut agent = fixtures::task::document();
    agent.chip.task = TASK;
    agent.chip.executor = c::ChipExecutor::Agent { charter: 1 };
    agent.priority = None;
    agent.acts = List::uncapped();
    agent.acts.push(c::Offered { act: c::ActKind::Message, admission: c::Admission::Allowed, effect: None }).unwrap();
    let agent_root = fixtures::task::supplied_root(agent.clone(), false);
    agent.priority = Some(1);
    let goal_root = fixtures::task::supplied_root(agent, false);
    let mut roots = List::with_capacity(5);
    roots.push(messaging_root()).expect("five composers");
    roots
        .push(fixtures::root_at(Address::NewChat { draft: false }, &fixtures::domain_limits()).0)
        .expect("five composers");
    roots.push(agent_root).expect("five composers");
    roots.push(goal_root).expect("five composers");
    if browser {
        roots
            .push(super::needs::fixture("Which path?", false, 1, false, temper_client_domain::address::Filter::ALL))
            .expect("five composers");
    }
    roots.into_boxed()
}

fn composer_limits() -> temper_client_domain::Limits {
    let mut limits = fixtures::domain_limits();
    limits.client.domain.long_text = 20_000;
    limits
}

fn composer_view(root: &temper_client_domain::Domain, browser: bool) -> crate::View {
    let words = if browser { fixtures::browser_words() } else { fixtures::words() };
    let limits = crate::Limits::of(&composer_limits(), &words).unwrap();
    let mut view = crate::View::new(&limits, &composer_limits(), words, fixtures::FORGE);
    fixtures::build(&mut view, root);
    view
}

fn composer_node(tree: &skein_ui::Tree, role: Role) -> &skein_ui::Node {
    for node in tree.nodes() {
        if node.role == role
            && (role != Role::Button
                || node.name.as_deref() == Some(b"Send")
                || node.name.as_deref() == Some(b"Send reply"))
        {
            return node;
        }
    }
    panic!("composer control {role:?} is absent");
}

fn composer_words(bytes: usize) -> Box<[u8]> {
    let mut words = skein_lib::bytes::zeroed(bytes);
    words.fill(b'x');
    words
}

fn composer_edit(root: &mut temper_client_domain::Domain, view: &mut crate::View, words: &[u8]) {
    let area = composer_node(view.tree(), Role::TextArea).id;
    let action = crate::act(view, area, skein_ui::Act::Input { value: words.into() }).unwrap();
    temper_client_domain::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits: composer_limits() },
        temper_client_domain::Event::Act { action },
        &mut skein_lib::Queue::uncapped(),
    );
}

#[test]
fn past_the_bound_send_is_disabled_and_a_line_says_how_far() {
    for browser in [false, true] {
        for (index, mut root) in bounded_composers(browser).into_iter().enumerate() {
            for (excess, line) in [
                (1, "That's 1 character over what one message holds."),
                (3200, "That's 3,200 characters over what one message holds."),
            ] {
                let mut view = composer_view(&root, browser);
                let bound = usize::try_from(root.pages().me(root.client()).unwrap().words_bytes).unwrap();
                let words = composer_words(bound + excess);
                composer_edit(&mut root, &mut view, &words);
                fixtures::build(&mut view, &root);
                let status = fixtures::find(view.tree(), Role::Status, line.as_bytes());
                assert!(status.is_some(), "composer {index}: {line}");
                assert!(view.tree().find(status.unwrap()).unwrap().styles.is_empty());
                let area = composer_node(view.tree(), Role::TextArea);
                assert_eq!(area.value.as_ref().unwrap().text.as_ref(), words.as_ref());
                assert!(!area.states.as_slice().contains(&skein_ui::State::SubmitsOnEnter));
                let send = composer_node(view.tree(), Role::Button);
                assert!(send.states.as_slice().contains(&skein_ui::State::Disabled), "composer {index}");
                let area = area.id;
                let send = send.id;
                assert_eq!(crate::act(&mut view, area, skein_ui::Act::Submit), None);
                assert_eq!(crate::act(&mut view, send, skein_ui::Act::Press), None);
            }
        }
    }
}

#[test]
fn within_the_bound_nothing_changes() {
    for (index, mut root) in bounded_composers(true).into_iter().enumerate() {
        let mut view = composer_view(&root, true);
        composer_edit(&mut root, &mut view, b"x");
        fixtures::build(&mut view, &root);
        let before = composer_node(view.tree(), Role::TextArea).value.as_ref().unwrap().written;
        let area_states = composer_node(view.tree(), Role::TextArea).states.clone();
        let send_states = composer_node(view.tree(), Role::Button).states.clone();
        let bound = usize::try_from(root.pages().me(root.client()).unwrap().words_bytes).unwrap();
        composer_edit(&mut root, &mut view, &composer_words(bound));
        fixtures::build(&mut view, &root);
        let area = composer_node(view.tree(), Role::TextArea);
        assert_eq!(area.states, area_states);
        assert_eq!(composer_node(view.tree(), Role::Button).states, send_states);
        if index == 0 {
            assert_eq!(area.value.as_ref().unwrap().written, before);
        }
        assert!(skein_lib::bytes::find(&fixtures::text(view.tree()), b"over what one message holds").is_none());
    }
}
