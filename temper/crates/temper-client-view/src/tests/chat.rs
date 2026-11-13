use super::fixtures;
use fixtures::chat::{
    ATTEMPT, BYTES, CLOSE_GRACE, COMMITTED, GRACES, LINES, MESSAGE, PARTY, TASK, TEXT, TURN, chat_header, chat_root,
    closing, messaging_root, projected_tree as tree, proposal_root, provisional, provisional_with_text, rejecting_chat,
};
use jig_client::{contract as c, thread::Thread};
use skein_lib::{List, Wall};
use skein_ui::Role;

fn reading_tree(thread: &Thread, projected: bool) -> (skein_ui::Tree, crate::frame::Bindings) {
    let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::frame::Bindings::new(fixtures::tree_limits().bindings);
    crate::pages::chat::conversation(
        &mut b,
        thread,
        &fixtures::browser_words(),
        (LINES, TEXT),
        crate::pages::chat::Speakers { person: None, requester: false },
        timed_drawing(projected),
        &mut bindings,
    );
    (b.finish(), bindings)
}

fn reading_thread() -> Thread {
    Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT, live: TEXT })
}

#[test]
fn no_page_offers_older() {
    let view = fixtures::page_view(&chat_with_history());
    assert!(fixtures::find(view.tree(), Role::Button, b"Older").is_none());
}

#[test]
fn the_log_is_bound_while_it_can_read_on() {
    use jig_client::thread::window::Edge;
    for (after, before, bound) in [
        (Edge::End, Edge::End, false),
        (Edge::Dropped, Edge::End, true),
        (Edge::End, Edge::Dropped, true),
        (Edge::Dropped, Edge::Dropped, true),
    ] {
        let mut thread = reading_thread();
        thread.lines.edges.after = after;
        thread.lines.edges.before = before;
        for projected in [false, true] {
            let (tree, _) = reading_tree(&thread, projected);
            assert_eq!(tree.nodes()[0].binding.is_some(), bound);
        }
    }
}

#[test]
fn near_decodes_to_older_and_newer() {
    use skein_ui::{Act, LogEdge};
    use temper_client_domain::{Action, pages::Act as PageAct};
    let mut thread = reading_thread();
    thread.lines.edges.after = jig_client::thread::window::Edge::Dropped;
    for projected in [false, true] {
        let (tree, bindings) = reading_tree(&thread, projected);
        let binding = tree.nodes()[0].binding.expect("reading binding");
        for (edge, meaning) in [(LogEdge::Head, PageAct::Older), (LogEdge::Foot, PageAct::Newer)] {
            assert_eq!(bindings.decode(binding.token, Act::Near { edge }), Some(Action::Pages(meaning)));
        }
        assert!(bindings.decode(binding.token, Act::Press).is_none());
    }
}

#[test]
fn a_log_continues_while_the_window_let_its_end_go() {
    use jig_client::thread::window::Edge;
    for (before, continues) in [(Edge::End, false), (Edge::Dropped, true)] {
        let mut thread = reading_thread();
        thread.lines.edges.before = before;
        for projected in [false, true] {
            let (tree, _) = reading_tree(&thread, projected);
            assert_eq!(tree.nodes()[0].states.as_slice().contains(&skein_ui::State::Continues), continues);
        }
    }
}

#[test]
fn a_log_is_busy_while_a_read_is_out_or_failed() {
    use jig_client::thread::{Reading, window::End};
    for (reading, failed, busy) in [
        (None, None, false),
        (Some(Reading::Older), None, true),
        (Some(Reading::Newer), None, true),
        (Some(Reading::Latest { known: 0 }), None, true),
        (None, Some(End::After), true),
        (None, Some(End::Before), true),
    ] {
        let mut thread = reading_thread();
        thread.reading = reading;
        thread.read_failed = failed;
        for projected in [false, true] {
            let (tree, _) = reading_tree(&thread, projected);
            assert_eq!(tree.nodes()[0].states.as_slice().contains(&skein_ui::State::Busy), busy);
        }
    }
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

fn page_words(positions: &[u64]) -> c::ActivityPage {
    let mut rows = List::with_capacity(u32::try_from(positions.len()).unwrap());
    for position in positions {
        rows.push(super::documents::line(
            *position,
            Wall::EPOCH,
            c::Activity::Words {
                message: *position,
                by: c::Requester::Person(PARTY),
                kind: c::MessageKind::Words,
                words: fixtures::first_run::text(if *position == 1 { b"Earlier words" } else { b"Retained words" }),
            },
        ))
        .unwrap();
    }
    c::ActivityPage { rows, before: None, after: None, cut: false }
}

#[test]
fn an_earlier_page_only_inserts() {
    use jig_client::thread::Reading;
    // Retain either the same day's lines or the next day's: both must stay put.
    const NEXT_DAY: Wall = Wall::from_nanos(86_400_000_000_000);
    for retained_day in [Wall::EPOCH, NEXT_DAY] {
        let limits = fixtures::tree_limits();
        let mut patches = skein_lib::Queue::with_capacity(limits.patches);
        let mut next = 0;
        let mut thread = reading_thread();
        thread.reading = Some(Reading::Latest { known: 0 });
        let mut page = page_words(&[2, 3]);
        for line in page.rows.iter_mut() {
            line.source.at = retained_day;
        }
        thread.page(page);
        let mut before = timed_tree(&thread, timed_drawing(true), Some(PARTY), 0).0;
        skein_ui::diff(&skein_ui::Tree::new(limits.nodes), &mut before, &limits, &mut next, &mut patches);
        while patches.pop().is_some() {}
        thread.reading = Some(Reading::Older);
        thread.page(page_words(&[1]));
        let mut after = timed_tree(&thread, timed_drawing(true), Some(PARTY), 0).0;
        skein_ui::diff(&before, &mut after, &limits, &mut next, &mut patches);
        let mut inserts = 0_u32;
        while let Some(patch) = patches.pop() {
            match patch {
                skein_ui::Patch::Insert { .. } => inserts = inserts.checked_add(1).unwrap(),
                skein_ui::Patch::Text { .. } | skein_ui::Patch::Remove { .. } => {
                    panic!("retained line changed: {patch:?}")
                }
                skein_ui::Patch::Move { .. } | skein_ui::Patch::Attribute { .. } | skein_ui::Patch::Value { .. } => {}
            }
        }
        assert!(inserts > 0);
    }
}

#[test]
fn reading_past_the_bound_only_removes_released_lines() {
    use jig_client::thread::Reading;
    let limits = fixtures::tree_limits();
    let mut patches = skein_lib::Queue::with_capacity(limits.patches);
    let mut next = 0;
    let page = page_words(&[2, 3]);
    // Two retained messages exactly fill the byte window.
    let bytes = page.rows.as_slice()[0].weight().checked_mul(2).unwrap();
    let mut thread = Thread::new(TASK, &jig_client::thread::Limits { bytes, lines: LINES, text: TEXT, live: TEXT });
    thread.reading = Some(Reading::Latest { known: 0 });
    thread.page(page);
    let mut before = tree(&thread);
    skein_ui::diff(&skein_ui::Tree::new(limits.nodes), &mut before, &limits, &mut next, &mut patches);
    while patches.pop().is_some() {}
    let mut released = None;
    for node in before.nodes() {
        if node.styles.as_slice().contains(&crate::look::TURN) && node.key.is_some() {
            released = Some(node.id);
        }
    }
    let released = released.expect("last retained line");
    thread.reading = Some(Reading::Older);
    thread.page(page_words(&[1]));
    let mut after = tree(&thread);
    skein_ui::diff(&before, &mut after, &limits, &mut next, &mut patches);
    let mut removed = List::with_capacity(limits.nodes);
    while let Some(patch) = patches.pop() {
        match patch {
            skein_ui::Patch::Remove { node } => removed.push(node).unwrap(),
            skein_ui::Patch::Text { .. } => panic!("retained line rewritten: {patch:?}"),
            skein_ui::Patch::Insert { .. }
            | skein_ui::Patch::Move { .. }
            | skein_ui::Patch::Attribute { .. }
            | skein_ui::Patch::Value { .. } => {}
        }
    }
    assert_eq!(removed.as_slice(), [released]);
}

#[test]
fn a_run_of_calls_takes_its_first_turns_key() {
    use fixtures::first_run as run;
    let mut thread = reading_thread();
    let mut rows = List::with_capacity(2);
    for number in [TURN, TURN + 1] {
        rows.push(run::turn(
            u64::from(number),
            ATTEMPT,
            number,
            &[run::message(c::Speaker::Assistant, &[run::call(u64::from(number), b"read_docs", b"{}", None)])],
            512,
        ))
        .unwrap();
    }
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    let drawn = tree(&thread);
    let mut turns = List::with_capacity(drawn.len());
    for node in drawn.nodes() {
        if node.styles.as_slice().contains(&crate::look::TURN) {
            turns.push(node).unwrap();
        }
    }
    assert_eq!(turns.len(), 1);
    assert_eq!(
        turns.as_slice()[0].key,
        Some(skein_ui::Key { owner: crate::frame::owner(), kind: 40, a: ATTEMPT, b: u64::from(TURN) })
    );
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
fn a_provisional_turn_and_its_committed_one_share_a_key() {
    let live = tree(&provisional());
    let committed = tree(&conversation_turn(
        &[fixtures::first_run::message(c::Speaker::Assistant, &[c::Block::Text(fixtures::first_run::text(b"Hello"))])],
        false,
    ));
    let key = live.nodes()[styled(&live, crate::look::TURN)].key.expect("provisional key");
    assert_eq!(committed.nodes()[styled(&committed, crate::look::TURN)].key, Some(key));
}

#[test]
fn a_failed_read_says_so_at_the_head() {
    use jig_client::thread::window::{Edge, End};
    for (after, failed, says) in [
        (Edge::Dropped, Some(End::After), true),
        (Edge::Dropped, Some(End::Before), false),
        (Edge::End, Some(End::After), false),
        (Edge::End, Some(End::Before), false),
    ] {
        let mut thread = conversation_turn(
            &[fixtures::first_run::message(
                c::Speaker::Assistant,
                &[c::Block::Text(fixtures::first_run::text(b"Kept"))],
            )],
            false,
        );
        thread.lines.edges.after = after;
        thread.read_failed = failed;
        let mut builder = skein_ui::Builder::new(&fixtures::tree_limits());
        crate::pages::chat::older(&mut builder, &thread, &fixtures::browser_words());
        let drawn = builder.finish();
        let status = fixtures::find(&drawn, Role::Status, b"Loading earlier messages");
        assert_eq!(status.is_some(), says);
        if let Some(id) = status {
            assert_eq!(drawn.find(id).unwrap().styles.as_slice(), [crate::look::THREAD_LOADING]);
            let at = named_position(drawn.nodes(), Role::Status, Some(b"Loading earlier messages")).unwrap();
            assert_eq!(fixtures::node_text(&drawn, at).as_ref(), b"Couldn't load earlier messages");
        }
    }
}

#[test]
fn a_failed_first_read_says_so() {
    let mut thread = reading_thread();
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    thread.failed();
    let mut builder = skein_ui::Builder::new(&fixtures::tree_limits());
    crate::pages::chat::older(&mut builder, &thread, &fixtures::browser_words());
    let drawn = builder.finish();
    let status = named_position(drawn.nodes(), Role::Status, Some(b"Loading earlier messages"))
        .expect("a failed first read says why the empty conversation cannot load");
    assert_eq!(fixtures::node_text(&drawn, status).as_ref(), b"Couldn't load earlier messages");
}

fn answer_tree(thread: &Thread, words: &crate::words::Words, projected: bool) -> skein_ui::Tree {
    let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
    crate::pages::chat::conversation(
        &mut b,
        thread,
        words,
        (LINES, TEXT),
        crate::pages::chat::Speakers { person: None, requester: false },
        if projected {
            crate::pages::chat::Drawn::Chat { request: None, delegations: None }
        } else {
            crate::pages::chat::Drawn::Lines
        },
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
    );
    b.finish()
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
fn a_chats_heading_is_its_first_sentence_or_line_cut_on_a_word() {
    for (title, short) in [
        ("Ship the update", "Ship the update"),
        (
            "okay let's design a small http server in python, using std lib. It should show a page with details about the moon cycle, where are we now in the cycle and when the next full moon will happen",
            "okay let's design a small http server in python, using std lib.",
        ),
        ("Why does it fail? It ran yesterday.", "Why does it fail?"),
        ("Ship it! Then rest.", "Ship it!"),
        ("Fix the build\nthen the docs. And more", "Fix the build"),
        ("Fix it\r\nthen", "Fix it"),
        ("Read 3.5 in lib.rs first. Then", "Read 3.5 in lib.rs first."),
        ("  \n\tIndented first line\nsecond", "Indented first line"),
        (
            "Show the moon cycle on one page, with where we are now and the next moon",
            "Show the moon cycle on one page, with where we are now and the next moon",
        ),
        (
            "Show the moon cycle on one page, with where we are now and the next one again soon",
            "Show the moon cycle on one page, with where we are now and the next one…",
        ),
        (
            "Please write a small http server in python for the moon cycle page, using the standard library",
            "Please write a small http server in python for the moon cycle page…",
        ),
        (&"x".repeat(100), "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx…"),
        (&"é".repeat(100), "ééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé…"),
        (&"é".repeat(72), "éééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééééé"),
        ("", ""),
    ] {
        assert_eq!(crate::pages::chat::short_title(title.as_bytes()).as_ref(), short.as_bytes(), "{title:?}");
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
fn the_end_goes_when_the_turn_commits() {
    let mut thread = provisional();
    assert!(fixtures::find(&tree(&thread), Role::Status, "▍".as_bytes()).is_some());
    thread.reading = Some(jig_client::thread::Reading::Latest { known: COMMITTED.into() });
    let mut rows = List::with_capacity(TURN);
    rows.push(crate::tests::documents::turn(
        TURN.into(),
        Wall::EPOCH,
        c::TurnName { attempt: ATTEMPT, number: TURN },
        c::TurnRead::Decoded(c::Turn {
            messages: {
                let mut messages = List::with_capacity(1);
                messages
                    .push(fixtures::first_run::message(
                        c::Speaker::Assistant,
                        &[c::Block::Text(fixtures::first_run::text(b"Hello"))],
                    ))
                    .unwrap();
                messages
            },
            used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
            spent: 0,
            read_through: None,
            llm: None,
            cut: false,
        }),
    ))
    .expect("turn");
    thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
    assert_eq!(fixtures::find(&tree(&thread), Role::Status, "▍".as_bytes()), None);
}

#[test]
fn a_committed_redraw_keeps_the_nodes_of_the_words_being_read() {
    use fixtures::first_run as run;
    const WORDS: &[u8] = b"First.\n\nSecond.\n\nThird.";
    let limits = fixtures::tree_limits();
    let mut patches = skein_lib::Queue::with_capacity(limits.patches);
    let mut next = 0;
    let mut thread = provisional_with_text(WORDS);
    let mut live = tree(&thread);
    thread.run(&c::RunState { task: TASK, attempt: None, committed_turn: TURN, writing: None, call: None }, None);
    assert_eq!(tree(&thread).nodes(), live.nodes(), "a run's end alone redraws nothing");
    skein_ui::diff(&skein_ui::Tree::new(limits.nodes), &mut live, &limits, &mut next, &mut patches);
    while patches.pop().is_some() {}
    let says = styled(&live, crate::look::SAYS);
    let end = styled(&live, crate::look::WRITING_END);
    let mark = live.nodes()[end].id;
    let turn = live.nodes()[styled(&live, crate::look::TURN)].id;
    let mut committed =
        tree(&conversation_turn(&[run::message(c::Speaker::Assistant, &[c::Block::Text(run::text(WORDS))])], false));
    skein_ui::diff(&live, &mut committed, &limits, &mut next, &mut patches);
    assert_eq!(committed.nodes()[styled(&committed, crate::look::TURN)].id, turn, "the turn's slot stays");
    for word in subtree(&live, says) {
        if word.id != mark && !subtree(&live, end).contains(word) {
            assert!(committed.find(word.id).is_some(), "a word's node was replaced");
        }
    }
    while let Some(patch) = patches.pop() {
        match patch {
            skein_ui::Patch::Remove { node } => {
                for word in subtree(&live, says) {
                    if word.id != mark && !subtree(&live, end).contains(word) {
                        assert_ne!(node, word.id, "a word's node was removed");
                    }
                }
            }
            skein_ui::Patch::Insert { parent, .. } => {
                for word in subtree(&committed, styled(&committed, crate::look::SAYS)) {
                    assert_ne!(parent, word.id, "nodes were inserted under the words");
                }
                assert_ne!(parent, committed.nodes()[styled(&committed, crate::look::SAYS)].id);
            }
            skein_ui::Patch::Move { .. }
            | skein_ui::Patch::Text { .. }
            | skein_ui::Patch::Attribute { .. }
            | skein_ui::Patch::Value { .. } => {}
        }
    }
}

// Compare the shell-visible nodes, discounting only the live end and its size.
fn said_without_end(tree: &skein_ui::Tree) -> List<skein_ui::Made> {
    let says = styled(tree, crate::look::SAYS);
    let children = subtree(tree, says);
    let end = style_position(children, crate::look::WRITING_END);
    let mut result = List::with_capacity(tree.len());
    for (at, node) in children.iter().enumerate() {
        let mut node = skein_ui::Made::of(node);
        if let Some(end) = end {
            let end_size = usize::try_from(children[end].size).unwrap();
            if (end..end.checked_add(end_size).unwrap()).contains(&at) {
                continue;
            }
            if at < end && at.checked_add(usize::try_from(node.size).unwrap()).unwrap() > end {
                node.size = node.size.checked_sub(u32::try_from(end_size).unwrap()).unwrap();
            }
        }
        result.push(node).unwrap();
    }
    result
}

#[test]
fn a_turn_being_written_draws_as_its_committed_turn_with_its_end() {
    use fixtures::first_run as run;
    for input in [
        b"A **bold** reply.\n\nLast words.".as_slice(),
        b"- first\n- last",
        b"> quoted",
        b"```\ncode\n```",
        b"| a | b |\n| - | - |\n| c | d |",
    ] {
        let live = provisional_with_text(input);
        let committed =
            conversation_turn(&[run::message(c::Speaker::Assistant, &[c::Block::Text(run::text(input))])], false);
        for (live, committed) in
            [(tree(&live), tree(&committed)), (fixtures::chat::tree(&live), fixtures::chat::tree(&committed))]
        {
            let message = &live.nodes()[styled(&live, crate::look::MESSAGE)];
            assert!(message.styles.as_slice().contains(&crate::look::BY_AGENT));
            let who = subtree(&live, styled(&live, crate::look::WHO));
            assert_eq!(who[0].name.as_deref(), Some(b"Temper".as_slice()));
            let mark = named_position(live.nodes(), Role::Status, Some("▍".as_bytes())).expect("live end");
            let end = styled(&live, crate::look::WRITING_END);
            assert_eq!(fixtures::find(&live, Role::Status, b"Writing..."), None);
            assert!(mark > end && mark < end.checked_add(usize::try_from(live.nodes()[end].size).unwrap()).unwrap());
            let mut block = None;
            for (at, node) in live.nodes()[..end].iter().enumerate() {
                if [Role::Paragraph, Role::Heading { level: 3 }, Role::Item, Role::Quote, Role::Cell, Role::Code]
                    .contains(&node.role)
                    && at.checked_add(usize::try_from(node.size).unwrap()).unwrap() > end
                {
                    block = Some(at);
                }
            }
            assert!(block.is_some(), "end inside the last word's block: {input:?}");
            let mut has_words = false;
            for node in &live.nodes()[block.unwrap()..end] {
                has_words |= !node.text.is_empty();
            }
            assert!(has_words);
            assert_eq!(said_without_end(&live), said_without_end(&committed));
        }
    }
}

#[test]
fn a_closed_block_emits_no_patch_as_words_arrive_after_it() {
    let limits = fixtures::tree_limits();
    let mut patches = skein_lib::Queue::with_capacity(limits.patches);
    let mut next = 0;
    let mut before = tree(&provisional_with_text(b"Closed **bold**.\n\nNext"));
    skein_ui::diff(&skein_ui::Tree::new(limits.nodes), &mut before, &limits, &mut next, &mut patches);
    while patches.pop().is_some() {}
    let says = styled(&before, crate::look::SAYS);
    let block = says.checked_add(1).unwrap();
    let through = block.checked_add(usize::try_from(before.nodes()[block].size).unwrap()).unwrap();
    let mut closed = List::with_capacity(before.len());
    for node in &before.nodes()[block..through] {
        closed.push(skein_ui::Made::of(node)).unwrap();
    }
    for input in [b"Closed **bold**.\n\nNext word".as_slice(), b"Closed **bold**.\n\nNext words arrive."] {
        let mut after = tree(&provisional_with_text(input));
        skein_ui::diff(&before, &mut after, &limits, &mut next, &mut patches);
        for (expected, node) in closed.iter().zip(&after.nodes()[block..through]) {
            assert_eq!(*expected, skein_ui::Made::of(node));
        }
        let mut changed = false;
        while let Some(patch) = patches.pop() {
            let target = match patch {
                skein_ui::Patch::Insert { parent, .. } => parent,
                skein_ui::Patch::Remove { node }
                | skein_ui::Patch::Move { node, .. }
                | skein_ui::Patch::Text { node, .. }
                | skein_ui::Patch::Attribute { node, .. }
                | skein_ui::Patch::Value { node, .. } => node,
            };
            for node in &closed {
                assert_ne!(node.id, target, "closed block patched");
            }
            changed = true;
        }
        assert!(changed, "the growing block changes");
        before = after;
    }
}

#[test]
fn a_turn_being_written_shows_no_raw_marker() {
    for (input, expected, run) in [
        (b"**bo".as_slice(), b"bo".as_slice(), Some(skein_ui::Run::Strong)),
        (b"##", b"", None),
        (b"```rust\ncode", b"code", Some(skein_ui::Run::Code)),
    ] {
        for tree in [tree(&provisional_with_text(input)), fixtures::chat::tree(&provisional_with_text(input))] {
            let nodes = subtree(&tree, styled(&tree, crate::look::SAYS));
            let mut said = List::with_capacity(TEXT);
            for node in nodes {
                for text in &node.text {
                    if text.text.as_ref() == "▍".as_bytes() {
                        continue;
                    }
                    if let Some(run) = run {
                        assert_eq!(text.run, run);
                    }
                    for byte in &text.text {
                        said.push(*byte).unwrap();
                    }
                }
            }
            assert_eq!(said.as_slice(), expected);
        }
    }
}

fn working_tree(thread: &Thread) -> skein_ui::Tree {
    use crate::pages::chat::{Drawn, Speakers, conversation_with_times};
    let mut task = asking_task();
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Due);
    let root = asking_task_root(task, true, 1);
    let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
    conversation_with_times(
        &mut b,
        thread,
        &fixtures::browser_words(),
        (LINES, TEXT),
        Speakers { person: Some(PARTY), requester: true },
        (Drawn::ChatPage { request: None, root: &root }, Some(&mut crate::words::Times::new(fixtures::NOW))),
        &mut crate::frame::Bindings::new(fixtures::tree_limits().bindings),
    );
    b.finish()
}

#[test]
fn a_turn_with_no_words_draws_nothing_and_the_dots_stand() {
    let empty = provisional_with_text(b"");
    for tree in [tree(&empty), fixtures::chat::tree(&empty), working_tree(&empty)] {
        assert!(style_position(tree.nodes(), crate::look::MESSAGE).is_none());
        assert!(style_position(tree.nodes(), crate::look::TURN).is_none());
    }
    assert!(style_position(working_tree(&empty).nodes(), crate::look::TYPING).is_some());
}

#[test]
fn the_first_word_takes_the_dots_place() {
    let tree = working_tree(&provisional());
    assert!(style_position(tree.nodes(), crate::look::TYPING).is_none());
    assert!(fixtures::find(&tree, Role::Status, "▍".as_bytes()).is_some());
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
    assert!(skein_lib::bytes::find(&text, b"T12 is cancelled.").is_some());
    assert!(skein_lib::bytes::find(&text, b"T13 moves to you.").is_some());
}
#[test]
fn the_close_confirmation_offers_each_live_delegates_choice() {
    let tree = closing(Some(CLOSE_GRACE));
    assert!(fixtures::find(&tree, Role::Button, b"Take it instead").is_some());
    assert!(fixtures::find(&tree, Role::Button, b"Cancel it instead").is_some());
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

const LONG_PROVISIONAL: &[u8] = b"a234567890123456789012345678901\xc3\xa9 and more";
#[test]
fn a_long_provisional_text_says_nothing_of_what_was_cut() {
    let thread = provisional_with_text(LONG_PROVISIONAL);
    for drawn in [fixtures::chat::projected_tree(&thread), fixtures::chat::tree(&thread)] {
        let said = fixtures::text(&drawn);
        assert!(skein_lib::bytes::find(&said, b"Earlier text omitted").is_none());
        assert!(skein_lib::bytes::find(&said, b"a234567890123456789012345678901").is_some());
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
        llm: None,
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
fn a_request_being_read_ends_in_the_mark() {
    let thread = owner_words_thread();
    for (kept, marked) in [(b"Kept request".as_slice(), "Kept request…"), (b"".as_slice(), "…")] {
        let mut request = c::SpecificationDocument::complete(c::Specification {
            asks: false,
            words: fixtures::first_run::text(kept),
            parameters: List::with_capacity(0),
            inputs: List::with_capacity(0),
        });
        request.specification.words.cut = true;
        let drawn = fixtures::chat::requested_tree(&thread, &request);
        assert!(fixtures::find(&drawn, Role::Paragraph, marked.as_bytes()).is_some());
        assert!(skein_lib::bytes::find(&fixtures::text(&drawn), b"Earlier text omitted").is_none());
    }
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
fn each_turn_carries_the_turn_token() {
    let tree = tree(&provisional());
    let turn = styled(&tree, crate::look::TURN);
    assert_eq!(tree.nodes()[turn + 1].role, Role::Region);
    let turns = fixtures::styles(&tree, Role::Slot);
    let mut count = 0_u32;
    for styles in &turns {
        if styles.as_ref() == [crate::look::TURN] {
            count = count.checked_add(1).unwrap();
        }
    }
    assert_eq!(count, 1);
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
                    llm: None,
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
    assert_run_line(&tree, b"Read 1 file", b"", &[b"Read: Done"]);
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
pub(super) fn answered_message(answer: c::Answer) -> temper_client_domain::Domain {
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
    assert_run_line(
        &tree,
        "Read 2 files, wait".as_bytes(),
        b"",
        &[b"read: Read the first file", b"wait", b"read: Read the second file"],
    );
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Temper").is_none());
}
#[test]
fn a_fold_names_two_repeated_calls_and_two_distinct_calls() {
    use fixtures::first_run as run;
    for (tool, description) in
        [(b"read".as_slice(), "Read 2 files"), (b"edit".as_slice(), "Read 1 file, edited 1 file")]
    {
        let tree = assistant_blocks(&[run::call(1, b"read", b"{}", None), run::call(2, tool, b"{}", None)]);
        assert_run_line(&tree, description.as_bytes(), b"", &[b"read", tool]);
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
    let mut name = skein_lib::Writer::new(256);
    name.put(tool.get(..253).unwrap()).unwrap();
    name.put("…".as_bytes()).unwrap();
    let mut row = skein_lib::Writer::new(160);
    row.put(tool.get(..157).unwrap()).unwrap();
    row.put("…".as_bytes()).unwrap();
    assert_run_line(&tree, &name.finish(), b"", &[&row.finish()]);
}
// The fake person sees the fold and all its content, without an Expanded state.
pub(super) fn assert_run_line(tree: &skein_ui::Tree, name: &[u8], description: &[u8], lines: &[&[u8]]) {
    assert!(fixtures::person_find(tree, Role::Disclosure, name).is_some());
    let at = named_position(tree.nodes(), Role::Disclosure, Some(name)).expect("disclosure node");
    let fold = &tree.nodes()[at];
    assert_eq!(fold.description.as_deref().unwrap_or(b""), description);
    assert!(!fold.states.as_slice().contains(&skein_ui::State::Expanded));
    let slot = at.checked_add(1).unwrap();
    assert!(tree.nodes()[slot].styles.as_slice().contains(&crate::look::WORK_CALLS));
    let mut child = slot.checked_add(1).unwrap();
    for line in lines {
        let node = &tree.nodes()[child];
        assert_eq!(node.role, Role::Paragraph);
        assert_eq!(fixtures::node_text(tree, child).as_ref(), *line);
        child = child.checked_add(usize::try_from(node.size).unwrap()).unwrap();
    }
    assert_eq!(child, at.checked_add(usize::try_from(fold.size).unwrap()).unwrap());
}
#[test]
fn an_opaque_block_draws_nothing_and_no_shell() {
    use fixtures::first_run as run;
    for speaker in [c::Speaker::Assistant, c::Speaker::Person, c::Speaker::Tool, c::Speaker::System] {
        let mut thread =
            Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT, live: TEXT });
        thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
        let mut rows = List::with_capacity(1);
        rows.push(run::turn(
            1,
            ATTEMPT,
            1,
            &[run::message(speaker, &[c::Block::Opaque { dialect: run::text(b"opaque-v1"), size: 42 }])],
            512,
        ))
        .unwrap();
        thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
        for words in [fixtures::words(), fixtures::browser_words()] {
            for projected in [true, false] {
                let drawn = answer_tree(&thread, &words, projected);
                assert_eq!(drawn.nodes().len(), 1, "only the conversation log remains");
                assert_eq!(drawn.nodes()[0].role, Role::Log);
                assert!(fixtures::text(&drawn).is_empty());
            }
        }
    }
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
        assert!(fixtures::find(view.tree(), Role::Status, b"Closed").is_some());
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
    let message = styled(&tree, crate::look::MESSAGE);
    let fold = styled(&tree, crate::look::WORK_RUN);
    let card = styled(&tree, crate::look::CARD);
    assert!(fold >= message + usize::try_from(tree.nodes()[message].size).unwrap());
    assert!(fold + usize::try_from(tree.nodes()[fold].size).unwrap() <= card, "run before proposal card");
    assert!(turn < fold);
    assert_eq!(named_count(&tree, Role::Button, b"Accept"), 1);
    assert_run_line(&tree, "Read 2 files".as_bytes(), b"", &[b"read", b"read"]);
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

pub(super) fn asking_task_root(task: c::Task, template: bool, charter: u32) -> temper_client_domain::Domain {
    asking_task_root_owned(task, template, charter, true)
}
fn asking_task_root_owned(
    mut task: c::Task,
    template: bool,
    charter: u32,
    owned: bool,
) -> temper_client_domain::Domain {
    let (mut root, me_watch, task_watch) =
        fixtures::root_at(temper_client_domain::Address::Chat { task: TASK }, &fixtures::domain_limits());
    let source = messaging_root();
    let mut me = source.pages().me(source.client()).expect("me").clone();
    if template {
        me = fixtures::asking_me(me);
    }
    me.projects.get_mut(0).unwrap().models = super::choice::line_models(true);
    fixtures::deliver_me(&mut root, me_watch, me);
    if !owned {
        super::connect_forge::reread_deployment(&mut root, me_watch, owned);
    }
    task.chip.executor = c::ChipExecutor::Agent { charter };
    deliver_chat(&mut root, task_watch.expect("task watch"), c::Delivery::Task(Box::new(task)));
    root
}
pub(super) fn asking_task() -> c::Task {
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
            (Role::Status, b"Parked"),
            (Role::Paragraph, b"7 spent \xc2\xb7 of 7"),
            (Role::Button, b"Pause"),
            (Role::Button, b"Close"),
        ] {
            assert!(named_position(top_nodes, role, Some(name)).is_some(), "{name:?}");
        }
        let controls = subtree(tree, bar);
        assert!(named_position(controls, Role::Button, Some(b"Terminal, full screen")).is_none());
        let positions = [
            named_position(controls, Role::Status, Some(b"Parked")).unwrap(),
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
            Some(b"Enter sends \xc2\xb7 Shift+Enter for a new line".as_slice())
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
                if asking {
                    assert_eq!(fixtures::find(view.tree(), Role::Button, b"+ Draft").is_some(), !ended && !result);
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
fn every_chat_bar_orders_its_byline_status_allowed_acts_and_spend_and_links_its_path() {
    use crate::look;
    use temper_client_domain::Address;
    for asking in [false, true] {
        for allowed in [false, true] {
            let mut task = asking_task();
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
                let status = named_position(controls, Role::Status, Some(b"Parked")).expect("status");
                let spend =
                    named_position(controls, Role::Paragraph, Some("7 spent · of 7".as_bytes())).expect("spend");
                if let Some(byline) = byline {
                    assert!(byline < status);
                }
                assert!(named_position(controls, Role::Button, Some(b"Terminal, full screen")).is_none());
                assert!(status < spend);
                for name in [b"Pause".as_slice(), b"Close"] {
                    assert_eq!(named_position(controls, Role::Button, Some(name)).is_some(), allowed);
                }
                if allowed {
                    let stop = named_position(controls, Role::Button, Some(b"Pause")).unwrap();
                    let close = named_position(controls, Role::Button, Some(b"Close")).unwrap();
                    assert!(status < stop && stop < close && close < spend);
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
fn the_chat_bar_marks_yours_and_waiting_states_as_needing_the_person() {
    use crate::look;
    for (line, token) in super::chats::status_rows() {
        let mut task = asking_task();
        task.chip = line.chip;
        task.waits_on = line.waits_on;
        let root = asking_task_root(task, true, 1);
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

#[test]
fn provisional_text_replaces_working_dots_and_agent_task_logs_never_have_them() {
    use crate::pages::chat::{Drawn, Speakers, conversation_with_times};
    let mut task = asking_task();
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Due);
    let root = asking_task_root(task, true, 1);
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
                assert!(fixtures::find(&tree, Role::Status, "▍".as_bytes()).is_some());
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

#[test]
fn a_cut_text_says_nothing() {
    use fixtures::first_run as run;
    for projected in [true, false] {
        let mut blocks = List::with_capacity(2);
        blocks.push(run::call(1, b"note", b"", Some("Note 1 revision 1 written · 12 lines".as_bytes()))).unwrap();
        blocks.push(c::Block::Text(run::text(b"A long reply, also cut"))).unwrap();
        let mut line = run::turn(1, ATTEMPT, 1, &[run::message(c::Speaker::Assistant, blocks.as_slice())], 512);
        let Some(c::ArchivedTurn { turn: c::TurnRead::Decoded(turn), .. }) = &mut line.decoded else {
            panic!("decoded turn")
        };
        turn.cut = true;
        line.source.task = TASK;
        line.decoded.as_mut().unwrap().task = TASK;
        let mut thread =
            Thread::new(TASK, &jig_client::thread::Limits { lines: LINES, bytes: BYTES, text: TEXT, live: TEXT });
        thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
        let mut rows = List::with_capacity(1);
        rows.push(line).unwrap();
        thread.page(c::ActivityPage { rows, before: None, after: None, cut: false });
        let tree = answer_tree(&thread, &fixtures::words(), projected);
        assert!(fixtures::find(&tree, Role::Status, b"Earlier text omitted").is_none());
        assert!(fixtures::find(&tree, Role::Button, b"Show").is_some());
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
        let page = full_turn_window(&domain, &input, domain.client.domain.rows);
        thread.page(page);
        assert_eq!(thread.lines().len(), domain.client.domain.rows);
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

fn repeated_markdown(pattern: &[u8], length: usize) -> Box<[u8]> {
    let mut input = skein_lib::bytes::zeroed(length);
    for (at, byte) in input.iter_mut().enumerate() {
        *byte = *pattern.get(at.checked_rem(pattern.len()).unwrap()).unwrap();
    }
    input
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

pub(super) fn model_effort(model: &str, effort: Option<&str>) -> c::ModelEffort {
    c::ModelEffort {
        model: fixtures::first_run::text(model.as_bytes()),
        effort: match effort {
            Some(effort) => Some(fixtures::first_run::text(effort.as_bytes())),
            None => None,
        },
    }
}
pub(super) fn model_choice(model: &str, effort: Option<&str>) -> c::LlmChoice {
    c::LlmChoice { later: None, account: None, main: model_effort(model, effort), subagents: List::with_capacity(0) }
}
pub(super) fn model_line_page(latest: Option<c::ModelEffort>) -> c::ActivityPage {
    let mut row = crate::tests::documents::line(1, Wall::EPOCH, c::Activity::Decided { revision: 1, proposal: None });
    if let Some(llm) = latest {
        row.source.line = c::Activity::Turn { reply: 1, attempt: ATTEMPT, turn: TURN, read: None };
        row.decoded = Some(c::ArchivedTurn {
            task: TASK,
            name: c::TurnName { attempt: ATTEMPT, number: TURN },
            cumulative: 0,
            read: None,
            at: Wall::EPOCH,
            turn: c::TurnRead::Decoded(c::Turn {
                messages: List::with_capacity(0),
                used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                spent: 0,
                read_through: None,
                cut: false,
                llm: Some(llm),
            }),
        });
    }
    c::ActivityPage { rows: List::from_boxed(Box::new([row])), before: None, after: None, cut: false }
}
pub(super) fn model_line_root(
    task: c::Task,
    latest: Option<c::ModelEffort>,
    preview: Option<u32>,
) -> temper_client_domain::Domain {
    let (mut root, me_watch, watch, _, _) = fixtures::root_at_with_activity(
        temper_client_domain::Address::Chat { task: TASK },
        &fixtures::domain_limits(),
        b"Shop",
        None,
        Some(model_line_page(latest)),
        None,
    );
    let (source, _) = super::choice::provider_byline_root(
        temper_client_domain::Address::NewChat { draft: false },
        [c::Readiness::Usable; 2],
        [true; 2],
        true,
    );
    let mut me = fixtures::asking_me(source.pages().me(source.client()).unwrap().clone());
    let project = me.projects.get_mut(0).unwrap();
    project.models = List::with_capacity(2);
    for (raw, shown, provider) in [("gpt-6-astra", "GPT-6 Astra", 17), ("claude-opus-5-5", "Opus 5.5", 18)] {
        let mut model = super::choice::line_models(true).get(0).unwrap().clone();
        model.name = fixtures::first_run::text(raw.as_bytes());
        model.shown = Some(fixtures::first_run::text(shown.as_bytes()));
        model.effort = Some(fixtures::first_run::text(b"high"));
        model.provider = Some(provider);
        project.models.push(model).unwrap();
    }
    project.defaults = List::with_capacity(2);
    if let Some(template) = preview {
        project.defaults.push(c::MeDefault { template, choice: model_choice("gpt-6-astra", None) }).unwrap();
        if template != 1 {
            project
                .defaults
                .push(c::MeDefault { template: 1, choice: model_choice("wrong-chat-preview", None) })
                .unwrap();
        }
    }
    fixtures::deliver_me(&mut root, me_watch, me);
    deliver_chat(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(task)));
    root
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
    assert!(fixtures::find(tree, Role::Status, b"Closed").is_some());
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

pub(super) fn long_line_page(
    address: temper_client_domain::Address,
    line: c::Activity,
    tracked: bool,
) -> (temper_client_domain::Domain, temper_client_domain::Limits) {
    let mut domain = fixtures::domain_limits();
    domain.client.domain.text = 512;
    domain.client.domain.long_text = 131_072;
    domain.client.domain.turn_text = 131_072;
    domain.client.domain.document_bytes = 1_048_576;
    domain.client.domain.turn_bytes = 1_048_576;
    domain.client.domain.requests.bytes = 4_194_304;
    domain.client.domain.ask_bytes = 1_048_576;
    let mut rows = List::with_capacity(1);
    rows.push(super::documents::line(1, Wall::EPOCH, line)).unwrap();
    let (mut root, _, watch, _, _) = fixtures::root_at_with_activity(
        address,
        &domain,
        b"Shop",
        None,
        Some(c::ActivityPage { rows, before: None, after: None, cut: false }),
        None,
    );
    let mut task = fixtures::task::document();
    task.chip.task = TASK;
    task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
    task.priority = tracked.then_some(1);
    task.activity = 1;
    temper_client_domain::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: Wall::EPOCH, limits: domain },
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
        &mut skein_lib::Queue::uncapped(),
    );
    (root, domain)
}

pub(super) fn assert_long_line(
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
pub(super) fn pause_chip(chip: &mut c::Chip) {
    chip.phase = c::Phase::Held;
    chip.details.phase = c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 19 };
    chip.details.details.hold = Some(c::ChipHold {
        placed: Some(c::PlacedHold {
            number: 1,
            owner: c::HoldOwner::Party { person: PARTY },
            mode: c::HoldMode::Stop,
            scope: c::HoldScope::Subtree { task: chip.task },
            reason: Box::default(),
            placed_at: Wall::EPOCH,
            revision: c::Revision { value: 19 },
            lifted: false,
            may_lift: true,
        }),
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: c::HoldCategory::Stopped,
        revision: 19,
        before: c::PriorPhase::Active(c::RunStage::Idle),
        escalation: None,
        tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0, infrastructure: 0 },
        hold_wait_since: None,
    });
}
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
        assert_eq!(view.tree().nodes()[status].styles.as_slice(), [crate::look::STATUS_QUIET]);
        let hint = fixtures::find(view.tree(), Role::Paragraph, b"Sending resumes this chat.").expect("resuming hint");
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
        assert!(fixtures::find(view.tree(), Role::Paragraph, b"Sending resumes this chat.").is_none());
    }
}
#[test]
fn a_pause_without_release_has_no_resume_or_resuming_hint() {
    let root = pause_root(true, true, false);
    for view in [fixtures::page_view(&root), fixtures::browser_page_view(&root)] {
        assert!(fixtures::find(view.tree(), Role::Button, b"Resume").is_none());
        assert!(fixtures::find(view.tree(), Role::Button, b"Pause").is_none());
        assert!(fixtures::find(view.tree(), Role::Paragraph, b"Sending resumes this chat.").is_none());
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

// Placement fixtures use empty call objects, as a model's tool calls do.
const PLACE_TEXT: u32 = 512;
// The two-proposal history retains six activity rows.
const PLACE_ROWS: u32 = 6;
const FIRST_CARD: c::ProposalKey = c::ProposalKey { proposer: c::Proposer::Task(TASK), number: 1 };
const SECOND_CARD: c::ProposalKey = c::ProposalKey { number: FIRST_CARD.number + 1, ..FIRST_CARD };

fn place_turn(position: u64, at: u64, words: &[u8]) -> c::DecodedActivityLine {
    use fixtures::first_run as run;
    let mut line = run::turn(
        position,
        ATTEMPT,
        u32::try_from(position).unwrap(),
        &[run::message(
            c::Speaker::Assistant,
            &[c::Block::Text(run::text(words)), run::call(position, b"propose_settings", b"{}", None)],
        )],
        PLACE_TEXT,
    );
    line.source.task = TASK;
    line.source.at = turn_wall(at);
    line.decoded.as_mut().unwrap().task = TASK;
    line.decoded.as_mut().unwrap().at = turn_wall(at);
    line
}
fn place_made(position: u64, at: u64, key: c::ProposalKey) -> c::DecodedActivityLine {
    let mut line = super::documents::line(
        position,
        turn_wall(at),
        c::Activity::Decided { revision: 1, proposal: Some(c::DecidedProposal { number: key.number, made: true }) },
    );
    line.source.task = TASK;
    line
}
fn place_proposal(key: c::ProposalKey, since: u64) -> c::ReadValue {
    let c::ReadValue::Proposal(mut proposal) = super::cards::batch_proposal() else { panic!("proposal") };
    proposal.key = key;
    proposal.waiting.since = turn_wall(since);
    proposal.action = c::ProposalAction::Settings(Box::new(c::ProposalActionSettings {
        against: 1,
        items: List::with_capacity(0),
        review: None,
        made: None,
    }));
    c::ReadValue::Proposal(proposal)
}
fn place_accepted(key: c::ProposalKey) -> c::ReadValue {
    c::ReadValue::Ended(Box::new(c::Ended {
        key: c::ObjectKey::Proposal(key),
        at: None,
        by: None,
        value: c::EndedValue::Proposal(c::ProposalHistory {
            number: key.number,
            proposer: TASK,
            as_holder: true,
            reason: c::DocumentText::complete(fixtures::first_run::text(b"Keep the settings")),
            state: c::ProposalHistoryState::Accepted { by: c::Requester::Person(PARTY) },
            action: c::RetainedAction::Settings(c::RetainedActionSettings {
                projects: List::with_capacity(0),
                against: 1,
                items: List::with_capacity(0),
                made: Some(2),
            }),
        }),
    }))
}
fn place_root(
    rows: &[c::DecodedActivityLine],
    pages: &[(c::ProposalKey, c::ReadValue)],
) -> temper_client_domain::Domain {
    place_read_root(rows, pages, false)
}
fn place_read_root(
    rows: &[c::DecodedActivityLine],
    pages: &[(c::ProposalKey, c::ReadValue)],
    gone: bool,
) -> temper_client_domain::Domain {
    use temper_client_domain::{Address, Event, Place, Request};
    let mut limits = fixtures::domain_limits();
    limits.client.domain.rows = PLACE_ROWS;
    limits.client.domain.turn_text = PLACE_TEXT;
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: fixtures::NOW, limits };
    let (mut root, _, _) = fixtures::root_at(Address::Settings { project: fixtures::PROJECT }, &limits);
    let mut out = skein_lib::Queue::uncapped();
    temper_client_domain::step(
        &mut root,
        &env,
        Event::Went { place: Place::Address(Address::Chat { task: TASK }) },
        &mut out,
    );
    let mut task = fixtures::task::document();
    task.chip.task = TASK;
    task.activity = match rows.last() {
        Some(row) => row.source.position,
        None => 0,
    };
    task.proposals = List::with_capacity(u32::try_from(pages.len()).unwrap());
    // Live keys arrive in reverse number order; ended keys come from activity alone.
    for (key, page) in pages.iter().rev() {
        if let c::ReadValue::Ended(_) = page {
        } else {
            task.proposals.push(*key).unwrap();
        }
    }
    while let Some(request) = out.pop() {
        let up = match request {
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Open {
                                watch,
                                subject: jig_client::Watchable::Jig(c::Subject::Task { task: TASK }),
                            },
                    },
            } => Some(jig_client::Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task.clone()))),
            }),
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Activity { .. }) },
                    },
            } => Some(jig_client::Up::ReadDone {
                read,
                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(
                    c::ActivityPage { rows: place_rows(rows), before: None, after: None, cut: false },
                )))),
            }),
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Proposal { proposal }) },
                    },
            } => Some(jig_client::Up::ReadDone {
                read,
                result: jig_client::Readout::Jig(if gone {
                    c::ReadResult::None(c::ReadNone::Gone)
                } else {
                    c::ReadResult::Value(place_page(pages, proposal).clone())
                }),
            }),
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
            | Request::Fact { .. } => None,
        };
        if let Some(up) = up {
            temper_client_domain::step(
                &mut root,
                &env,
                Event::Client { event: jig_client::Event::Up { up } },
                &mut out,
            );
        }
    }
    root
}
fn place_tree(root: &temper_client_domain::Domain, thread: &Thread, shell: crate::words::Shell) -> skein_ui::Tree {
    let mut limits = fixtures::tree_limits();
    // Two settings cards and four full turns share this drawing.
    limits.nodes = 512;
    let mut b = skein_ui::Builder::new(&limits);
    let words = if shell == crate::words::Shell::Browser { fixtures::browser_words() } else { fixtures::words() };
    crate::pages::chat::conversation_with_times(
        &mut b,
        thread,
        &words,
        (PLACE_ROWS, PLACE_TEXT),
        crate::pages::chat::Speakers { person: Some(PARTY), requester: true },
        (
            crate::pages::chat::Drawn::ChatPage { request: None, root },
            Some(&mut crate::words::Times::new(fixtures::NOW)),
        ),
        &mut crate::frame::Bindings::new(limits.bindings),
    );
    b.finish()
}
fn place_rows(rows: &[c::DecodedActivityLine]) -> List<c::DecodedActivityLine> {
    let mut kept = List::with_capacity(u32::try_from(rows.len()).unwrap());
    for row in rows {
        kept.push(row.clone()).unwrap();
    }
    kept
}
fn place_page(pages: &[(c::ProposalKey, c::ReadValue)], wanted: c::ProposalKey) -> &c::ReadValue {
    for (key, page) in pages {
        if *key == wanted {
            return page;
        }
    }
    unreachable!("requested proposal belongs to the fixture")
}
fn place_text(tree: &skein_ui::Tree, words: &[u8]) -> usize {
    for (index, node) in tree.nodes().iter().enumerate() {
        for run in &node.text {
            if run.text.as_ref() == words {
                return index;
            }
        }
    }
    unreachable!("named words belong to the fixture")
}
fn placed_card(tree: &skein_ui::Tree, wanted: c::ProposalKey) -> &skein_ui::Node {
    let key = crate::pages::inbox::card_key(c::ObjectKey::Proposal(wanted));
    for node in tree.nodes() {
        if node.key == Some(key) {
            return node;
        }
    }
    unreachable!("card has its proposal key")
}
fn place_order(tree: &skein_ui::Tree) -> List<&'static str> {
    let mut order = List::with_capacity(tree.len());
    for node in tree.nodes() {
        let text = match node.text.as_slice().first() {
            Some(run) => Some(run.text.as_ref()),
            None => None,
        };
        if let Some(name) = node.name.as_deref().or(text) {
            for word in ["First reply", "Making turn", "Later reply", "Second reply", "Third reply", "Still writing"] {
                if name == word.as_bytes() {
                    order.push(word).unwrap();
                }
            }
        }
        for key in [FIRST_CARD, SECOND_CARD] {
            if node.key == Some(crate::pages::inbox::card_key(c::ObjectKey::Proposal(key))) {
                order.push(if key == FIRST_CARD { "First card" } else { "Second card" }).unwrap();
            }
        }
    }
    order
}
fn assert_place_order(root: &temper_client_domain::Domain, expected: &[&str]) {
    for shell in [crate::words::Shell::Terminal, crate::words::Shell::Browser] {
        let tree = place_tree(root, &root.pages().chat().unwrap().thread, shell);
        assert_eq!(place_order(&tree).as_slice(), expected, "{shell:?}");
    }
}
#[test]
fn a_card_stands_after_the_turn_that_made_it_and_a_reply_after_it() {
    let root = place_root(
        &[
            place_turn(1, 1, b"First reply"),
            place_made(2, 2, FIRST_CARD),
            place_turn(3, 3, b"Making turn"),
            place_turn(4, 4, b"Later reply"),
        ],
        &[(FIRST_CARD, place_proposal(FIRST_CARD, 2))],
    );
    assert_place_order(&root, &["First reply", "Making turn", "First card", "Later reply"]);
    for shell in [crate::words::Shell::Terminal, crate::words::Shell::Browser] {
        let tree = place_tree(&root, &root.pages().chat().unwrap().thread, shell);
        let making = place_text(&tree, b"Making turn");
        let mut folded = false;
        for node in &tree.nodes()[making..] {
            if node.key == Some(crate::pages::inbox::card_key(c::ObjectKey::Proposal(FIRST_CARD))) {
                break;
            }
            folded |= node.styles.as_slice().contains(&crate::look::WORK_RUN);
        }
        assert!(folded, "the making turn's tool line precedes its card");
    }
}
#[test]
fn a_new_card_is_last_while_its_turn_runs() {
    let root = place_root(&[place_made(1, 1, FIRST_CARD)], &[(FIRST_CARD, place_proposal(FIRST_CARD, 1))]);
    let mut thread = provisional_with_text(b"Still writing");
    thread.lines.rows = List::from_boxed(Box::new([place_made(1, 1, FIRST_CARD)]));
    for shell in [crate::words::Shell::Terminal, crate::words::Shell::Browser] {
        assert_eq!(place_order(&place_tree(&root, &thread, shell)).as_slice(), ["Still writing", "First card"]);
    }
}
#[test]
fn a_second_proposal_stands_below_the_first() {
    let root = place_root(
        &[
            place_made(1, 1, FIRST_CARD),
            place_turn(2, 2, b"First reply"),
            place_turn(3, 3, b"Second reply"),
            place_made(4, 4, SECOND_CARD),
            place_turn(5, 5, b"Third reply"),
            place_turn(6, 6, b"Later reply"),
        ],
        &[(FIRST_CARD, place_accepted(FIRST_CARD)), (SECOND_CARD, place_proposal(SECOND_CARD, 4))],
    );
    assert_place_order(
        &root,
        &["First reply", "First card", "Second reply", "Third reply", "Second card", "Later reply"],
    );
}
#[test]
fn a_card_with_no_line_goes_by_its_making_time() {
    let root = place_root(
        &[place_turn(1, 1, b"First reply"), place_turn(2, 3, b"Making turn"), place_turn(3, 4, b"Later reply")],
        &[(FIRST_CARD, place_proposal(FIRST_CARD, 2))],
    );
    assert_place_order(&root, &["First reply", "Making turn", "First card", "Later reply"]);
}
#[test]
fn a_tie_goes_by_the_line_and_not_the_clock() {
    let root = place_root(
        &[
            place_turn(1, 1, b"First reply"),
            place_made(2, 1, FIRST_CARD),
            place_turn(3, 1, b"Making turn"),
            place_turn(4, 1, b"Later reply"),
        ],
        &[(FIRST_CARD, place_proposal(FIRST_CARD, 1))],
    );
    assert_place_order(&root, &["First reply", "Making turn", "First card", "Later reply"]);
}
#[test]
fn a_card_past_the_window_s_end_is_not_drawn() {
    let root = place_root(&[place_turn(1, 1, b"First reply")], &[(FIRST_CARD, place_proposal(FIRST_CARD, 2))]);
    let mut thread = reading_thread();
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    thread.page(c::ActivityPage {
        rows: List::from_boxed(Box::new([place_turn(1, 1, b"First reply")])),
        after: Some(c::Cursor::new(b"newer".as_slice().into(), TEXT).unwrap()),
        before: None,
        cut: false,
    });
    for shell in [crate::words::Shell::Terminal, crate::words::Shell::Browser] {
        let tree = place_tree(&root, &thread, shell);
        assert_eq!(place_order(&tree).as_slice(), ["First reply"]);
        assert_eq!(named_count(&tree, Role::Button, b"Accept"), 0);
    }
    thread.reading = Some(jig_client::thread::Reading::Newer);
    thread.page(c::ActivityPage { rows: List::with_capacity(0), before: None, after: None, cut: false });
    for shell in [crate::words::Shell::Terminal, crate::words::Shell::Browser] {
        assert_eq!(place_order(&place_tree(&root, &thread, shell)).as_slice(), ["First reply", "First card"]);
    }
}
#[test]
fn a_call_naming_its_proposal_keeps_the_card_at_the_call() {
    let mut turn = place_turn(1, 1, b"First reply");
    let c::TurnRead::Decoded(decoded) = &mut turn.decoded.as_mut().unwrap().turn else { panic!("decoded") };
    let c::Block::Call(call) = decoded.messages.get_mut(0).unwrap().blocks.get_mut(1).unwrap() else { panic!("call") };
    call.objects = List::from_boxed(Box::new([c::ObjectKey::Proposal(FIRST_CARD)]));
    let root = place_root(
        &[turn, place_made(2, 2, FIRST_CARD), place_turn(3, 3, b"Later reply")],
        &[(FIRST_CARD, place_proposal(FIRST_CARD, 2))],
    );
    for shell in [crate::words::Shell::Terminal, crate::words::Shell::Browser] {
        let tree = place_tree(&root, &root.pages().chat().unwrap().thread, shell);
        let card = named_position(tree.nodes(), Role::Region, Some(b"Proposal from T9")).expect("call's region");
        let reply = place_text(&tree, b"Later reply");
        assert!(card < reply);
        assert_eq!(named_count(&tree, Role::Button, b"Accept"), 1);
    }
}
#[test]
fn a_making_line_draws_no_decision_row() {
    for (proposal, expected) in [
        (Some(c::DecidedProposal { number: FIRST_CARD.number, made: true }), 0),
        (Some(c::DecidedProposal { number: FIRST_CARD.number, made: false }), 1),
        (None, 1),
    ] {
        let line = super::documents::line(1, Wall::EPOCH, c::Activity::Decided { revision: 1, proposal });
        let thread = timed_thread(&[(1, line)]);
        for projected in [true, false] {
            let browser = reading_tree(&thread, projected).0;
            let terminal = timed_tree(&thread, timed_drawing(projected), None, 1).0;
            for tree in [browser, terminal] {
                assert_eq!(skein_lib::bytes::count(&fixtures::text(&tree), b"Decision made", u32::MAX), expected);
            }
        }
    }
}
#[test]
fn cards_at_one_place_stand_in_number_order() {
    let root = place_root(
        &[place_made(1, 1, SECOND_CARD), place_made(2, 2, FIRST_CARD), place_turn(3, 3, b"First reply")],
        &[(FIRST_CARD, place_proposal(FIRST_CARD, 2)), (SECOND_CARD, place_proposal(SECOND_CARD, 1))],
    );
    assert_place_order(&root, &["First reply", "First card", "Second card"]);
}

#[test]
fn a_card_keeps_its_element_as_lines_arrive_around_it() {
    let rows = [place_made(2, 2, FIRST_CARD), place_turn(3, 3, b"Making turn")];
    let pages = [(FIRST_CARD, place_proposal(FIRST_CARD, 2))];
    let before = place_root(&rows, &pages);
    let after = place_root(
        &[place_turn(1, 1, b"First reply"), rows[0].clone(), rows[1].clone(), place_turn(4, 4, b"Later reply")],
        &pages,
    );
    for shell in [crate::words::Shell::Terminal, crate::words::Shell::Browser] {
        let limits = fixtures::tree_limits();
        let mut patches = skein_lib::Queue::with_capacity(limits.patches);
        let mut next = 0;
        let mut first = place_tree(&before, &before.pages().chat().unwrap().thread, shell);
        skein_ui::diff(&skein_ui::Tree::new(limits.nodes), &mut first, &limits, &mut next, &mut patches);
        while patches.pop().is_some() {}
        let id = placed_card(&first, FIRST_CARD).id;
        let mut second = place_tree(&after, &after.pages().chat().unwrap().thread, shell);
        skein_ui::diff(&first, &mut second, &limits, &mut next, &mut patches);
        assert_eq!(placed_card(&second, FIRST_CARD).id, id);
        while let Some(patch) = patches.pop() {
            if let skein_ui::Patch::Remove { node } = patch {
                assert_ne!(node, id);
            }
        }
    }
}

fn place_decision(position: u64, key: c::ProposalKey) -> c::DecodedActivityLine {
    let mut line = place_made(position, position, key);
    let c::Activity::Decided { proposal: Some(proposal), .. } = &mut line.source.line else { panic!("decision") };
    proposal.made = false;
    line
}
#[test]
fn a_decided_card_stands_at_its_place_after_a_reload() {
    let root = place_root(
        &[
            place_turn(1, 1, b"First reply"),
            place_made(2, 2, FIRST_CARD),
            place_turn(3, 3, b"Making turn"),
            place_decision(4, FIRST_CARD),
            place_turn(5, 5, b"Later reply"),
        ],
        &[(FIRST_CARD, place_accepted(FIRST_CARD))],
    );
    assert_place_order(&root, &["First reply", "Making turn", "First card", "Later reply"]);
}
#[test]
fn where_a_card_stands_there_is_no_decision_row() {
    let root = place_root(
        &[place_made(1, 1, FIRST_CARD), place_turn(2, 2, b"Making turn"), place_decision(3, FIRST_CARD)],
        &[(FIRST_CARD, place_accepted(FIRST_CARD))],
    );
    for shell in [crate::words::Shell::Terminal, crate::words::Shell::Browser] {
        let tree = place_tree(&root, &root.pages().chat().unwrap().thread, shell);
        assert_eq!(place_order(&tree).as_slice(), &["Making turn", "First card"]);
        assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Decision made").is_none());
    }
}
#[test]
fn a_row_stays_where_no_card_stands() {
    let made = [place_made(1, 1, FIRST_CARD), place_turn(2, 2, b"Making turn"), place_decision(3, FIRST_CARD)];
    let decision = [place_decision(3, FIRST_CARD)];
    let pages = [(FIRST_CARD, place_accepted(FIRST_CARD))];
    for (root, rows) in [
        (Box::new(place_root(&decision, &[])), decision.as_slice()),
        (Box::new(place_root(&made, &pages)), decision.as_slice()),
        (Box::new(place_read_root(&made, &pages, true)), made.as_slice()),
    ] {
        let mut thread = Thread::new(
            TASK,
            &jig_client::thread::Limits { lines: PLACE_ROWS, bytes: BYTES, text: PLACE_TEXT, live: PLACE_TEXT },
        );
        thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
        thread.page(c::ActivityPage { rows: place_rows(rows), before: None, after: None, cut: false });
        for shell in [crate::words::Shell::Terminal, crate::words::Shell::Browser] {
            let tree = place_tree(&root, &thread, shell);
            assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Decision made").is_some());
            for node in tree.nodes() {
                assert_ne!(node.key, Some(crate::pages::inbox::card_key(c::ObjectKey::Proposal(FIRST_CARD))));
            }
        }
    }
}
#[test]
fn a_decided_proposal_of_another_kind_keeps_its_row_and_draws_no_card() {
    let mut ended = place_accepted(FIRST_CARD);
    let c::ReadValue::Ended(value) = &mut ended else { panic!("ended") };
    let c::EndedValue::Proposal(proposal) = &mut value.value else { panic!("proposal") };
    proposal.action = c::RetainedAction::Batch(List::with_capacity(0));
    let root = place_root(
        &[place_made(1, 1, FIRST_CARD), place_turn(2, 2, b"Making turn"), place_decision(3, FIRST_CARD)],
        &[(FIRST_CARD, ended)],
    );
    for shell in [crate::words::Shell::Terminal, crate::words::Shell::Browser] {
        let tree = place_tree(&root, &root.pages().chat().unwrap().thread, shell);
        assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Decision made").is_some());
        assert_eq!(place_order(&tree).as_slice(), &["Making turn"]);
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
#[test]
fn a_whole_request_draws_whole() {
    let request = c::SpecificationDocument::complete(c::Specification {
        asks: false,
        words: fixtures::first_run::text(b"Whole request"),
        parameters: List::with_capacity(0),
        inputs: List::with_capacity(0),
    });
    let drawn = fixtures::chat::requested_tree(&owner_words_thread(), &request);
    assert!(fixtures::find(&drawn, Role::Paragraph, b"Whole request").is_some());
    assert!(skein_lib::bytes::find(&fixtures::text(&drawn), "…".as_bytes()).is_none());
}
