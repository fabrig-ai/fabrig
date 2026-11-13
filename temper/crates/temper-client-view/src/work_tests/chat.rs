use super::fixtures;

use super::fixtures::chat::{
    ATTEMPT, BYTES, COMMITTED, LINES, MESSAGE, PARTY, TASK, TEXT, TURN, chat_header, projected_tree as tree, provisional, provisional_with_text,
};

use jig_client::{contract as c, thread::Thread};

use skein_lib::{List, Wall};

use skein_ui::Role;


pub(super) use super::fixtures::chat_helpers::{
    asking_task, asking_task_root, assert_run_line, named_position, pause_chip, structured_thread,
};


fn reading_tree(thread: &Thread, projected: bool) -> (skein_ui::Tree, crate::bindings::Bindings) {
    let mut b = skein_ui::Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::bindings::Bindings::new(fixtures::tree_limits().bindings);
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
    use super::fixtures::first_run as run;
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
        Some(skein_ui::Key { owner: crate::look::owner(), kind: 40, a: ATTEMPT, b: u64::from(TURN) })
    );
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
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
    );
    b.finish()
}


#[test]
fn a_chats_bar_draws_no_status() {
    let tree = chat_header();
    assert!(fixtures::find(&tree, Role::Status, b"Parked").is_none());
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
    rows.push(super::documents::turn(
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
    use super::fixtures::first_run as run;
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
    use super::fixtures::first_run as run;
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
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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
    use super::fixtures::first_run as run;
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
    use super::fixtures::first_run as run;
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
    use super::fixtures::first_run as run;
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
    use super::fixtures::first_run as run;
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
    use super::fixtures::first_run as run;
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


fn styled(tree: &skein_ui::Tree, token: skein_ui::Style) -> usize {
    style_position(tree.nodes(), token).expect("styled node")
}

fn subtree(tree: &skein_ui::Tree, at: usize) -> &[skein_ui::Node] {
    let from = at.checked_add(1).expect("child index");
    let through = at.checked_add(usize::try_from(tree.nodes()[at].size).expect("size")).expect("subtree end");
    &tree.nodes()[from..through]
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


fn assistant_blocks(blocks: &[c::Block]) -> skein_ui::Tree {
    use super::fixtures::first_run as run;
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
    use super::fixtures::first_run as run;
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
    use super::fixtures::first_run as run;
    for (tool, description) in
        [(b"read".as_slice(), "Read 2 files"), (b"edit".as_slice(), "Read 1 file, edited 1 file")]
    {
        let tree = assistant_blocks(&[run::call(1, b"read", b"{}", None), run::call(2, tool, b"{}", None)]);
        assert_run_line(&tree, description.as_bytes(), b"", &[b"read", tool]);
    }
}

#[test]
fn a_tool_name_using_the_whole_turn_budget_fits_its_fold_description() {
    use super::fixtures::first_run as run;
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
        &mut crate::bindings::Bindings::new(bounds.tree.bindings),
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

#[test]
fn an_opaque_block_draws_nothing_and_no_shell() {
    use super::fixtures::first_run as run;
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
    rows.push(super::documents::line(
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
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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
    use super::fixtures::first_run as run;
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
    super::documents::line(
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
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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
    let began = super::documents::line(
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
        super::documents::turn(
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
fn a_proposals_fold_precedes_its_card_after_the_message() {
    use super::fixtures::first_run as run;
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
        &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
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
            let mut bindings = crate::bindings::Bindings::new(fixtures::tree_limits().bindings);
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


#[test]
fn a_cut_text_says_nothing() {
    use super::fixtures::first_run as run;
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


fn full_turn_window(domain: &temper_client_domain::Limits, input: &[u8], count: u32) -> c::ActivityPage {
    use super::fixtures::first_run as run;
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
            &mut crate::bindings::Bindings::new(bounds.tree.bindings),
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


fn repeated_markdown(pattern: &[u8], length: usize) -> Box<[u8]> {
    let mut input = skein_lib::bytes::zeroed(length);
    for (at, byte) in input.iter_mut().enumerate() {
        *byte = *pattern.get(at.checked_rem(pattern.len()).unwrap()).unwrap();
    }
    input
}


// Placement fixtures use empty call objects, as a model's tool calls do.
const PLACE_TEXT: u32 = 512;

// The two-proposal history retains six activity rows.
const PLACE_ROWS: u32 = 6;

const FIRST_CARD: c::ProposalKey = c::ProposalKey { proposer: c::Proposer::Task(TASK), number: 1 };

const SECOND_CARD: c::ProposalKey = c::ProposalKey { number: FIRST_CARD.number + 1, ..FIRST_CARD };


fn place_turn(position: u64, at: u64, words: &[u8]) -> c::DecodedActivityLine {
    use super::fixtures::first_run as run;
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
        &mut crate::bindings::Bindings::new(limits.bindings),
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
