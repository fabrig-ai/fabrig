use super::fixtures;

use jig_client::contract::*;

use skein_lib::{Queue, Wall};

use skein_ui::{Builder, Role};

use temper_client_domain::ChatsFilter;



pub(super) use super::fixtures::chats::{
    HELD_STATUSES, TASK, line, row_status_styles, status_rows,
};


pub(super) fn tree(line: &TaskLine) -> (skein_ui::Tree, crate::bindings::Bindings) {
    tree_at(line, fixtures::NOW)
}

fn tree_at(line: &TaskLine, now: Wall) -> (skein_ui::Tree, crate::bindings::Bindings) {
    let mut b = Builder::new(&fixtures::tree_limits());
    let mut bindings = crate::bindings::Bindings::new(fixtures::tree_limits().bindings);
    crate::pages::chats::row(
        &mut b,
        &fixtures::root_in_project(Some(fixtures::PROJECT)),
        line,
        &fixtures::words(),
        &mut bindings,
        &mut crate::words::Times::new(now),
    );
    (b.finish(), bindings)
}

fn style_index(tree: &skein_ui::Tree, style: skein_ui::Style) -> usize {
    for (index, node) in tree.nodes().iter().enumerate() {
        if node.styles.as_slice().contains(&style) {
            return index;
        }
    }
    panic!("missing style {style:?}");
}


#[test]
fn a_chats_row_says_its_status_from_jigs_facts() {
    let (tree, _) = tree(&line());
    assert!(fixtures::find(&tree, Role::Status, b"Parked").is_some());
}


#[test]
fn a_held_chats_row_names_only_its_exceeded_bound() {
    for (reason, expected) in HELD_STATUSES {
        let mut line = line();
        let ChipPhaseFacts { phase, .. } = line.chip.details.as_mut();
        *phase = ChipPhase::Held { reason, revision: 1 };
        let (tree, _) = tree(&line);
        let mut status = None;
        for node in tree.nodes() {
            if node.role == Role::Status && node.name.as_deref() == Some(expected) {
                status = Some(node);
                break;
            }
        }
        let status = status.expect("held status");
        assert_eq!(status.styles.as_slice(), [crate::look::STATUS_HELD]);
        assert!(skein_lib::bytes::find(&fixtures::text(&tree), expected).is_some());
    }
}


#[test]
fn a_chats_row_says_its_spend_and_delegates() {
    let (tree, _) = tree(&line());
    assert!(fixtures::find(&tree, Role::Paragraph, "7 spent".as_bytes()).is_some());
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), "7 spent · 3 delegates".as_bytes()).is_some());
}


#[test]
fn a_chats_row_offers_its_number_as_a_terminal_command() {
    // client/pages/README.md: "TN", a link to its page; 6.5: numbered commands.
    for ended in [false, true] {
        let mut line = line();
        if ended {
            line.chip.phase = Phase::Ended;
            line.ended_at = Some(Wall::EPOCH);
        }
        let (mut tree, bindings) = tree(&line);
        let limits = fixtures::tree_limits();
        skein_ui::diff(
            &skein_ui::Tree::new(limits.nodes),
            &mut tree,
            &limits,
            &mut 0,
            &mut Queue::with_capacity(limits.patches),
        );
        let control = fixtures::find(&tree, Role::Link, b"T9").expect("chat number link");
        let (node, first) = fixtures::terminal_command(&tree, &limits, control, b"T9", b"Hello");
        let mut binding = None;
        for item in tree.nodes() {
            if item.id == node {
                binding = item.binding;
            }
        }
        assert_eq!(
            bindings.decode(binding.expect("chat link binding").token, first),
            Some(temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Go {
                address: temper_client_domain::Address::Chat { task: TASK }
            }))
        );
    }
}

#[test]
fn a_chats_row_carries_the_row_token() {
    let (tree, _) = tree(&line());
    assert_eq!(fixtures::styles(&tree, Role::Item).as_ref(), [Box::from([crate::look::CHATS_ROW])]);
    let nodes = tree.nodes();
    assert_eq!(nodes[0].key, Some(skein_ui::Key { owner: crate::look::owner(), kind: 10, a: TASK, b: 0 }));
    assert_eq!(nodes[1].styles.as_slice(), [crate::look::CHATS_TOP]);
    let title = style_index(&tree, crate::look::CHATS_LINE);
    assert_eq!(title, 1 + usize::try_from(nodes[1].size).unwrap());
    assert_eq!(nodes[title].name.as_deref(), Some(b"Hello".as_slice()));
    assert!(style_index(&tree, crate::look::DETAILS) > title);

    for ended_at in [false, true] {
        let mut ended = line();
        if ended_at {
            ended.ended_at = Some(Wall::EPOCH);
        } else {
            ended.chip.phase = Phase::Ended;
        }
        let (tree, _) = self::tree(&ended);
        assert_eq!(
            fixtures::styles(&tree, Role::Item).as_ref(),
            [Box::from([crate::look::CHATS_ROW, crate::look::CHATS_ROW_ENDED])]
        );
        assert!(fixtures::find(&tree, Role::Status, b"Closed").is_some());
    }
}


#[test]
fn each_status_is_styled_by_its_meaning() {
    for (line, token) in status_rows() {
        let (tree, _) = tree(&line);
        assert_eq!(fixtures::styles(&tree, Role::Status).as_ref(), [row_status_styles(token)]);
        for node in tree.nodes() {
            assert!(node.styles.len() <= 2);
        }
        let root = fixtures::root_in_project(None);
        let mut b = Builder::new(&fixtures::tree_limits());
        crate::pages::chats::status(
            &mut b,
            &line.chip,
            (&line.waits_on, None, None),
            (line.project, line.requester),
            None,
            root.pages().me(root.client()).unwrap(),
            &fixtures::words(),
        );
        assert_eq!(fixtures::styles(&b.finish(), Role::Status).as_ref(), [Box::from([token])]);
    }
}

#[test]
fn a_row_shows_the_real_activity_or_end_time() {
    let mut live = line();
    let now = Wall::from_nanos(720_000_000_000);
    let (shown, _) = tree_at(&live, now);
    assert_eq!(fixtures::node_text(&shown, style_index(&shown, crate::look::TIME)).as_ref(), b"12 min");
    live.ended_at = Some(Wall::from_nanos(360_000_000_000));
    let (shown, _) = tree_at(&live, now);
    assert_eq!(fixtures::node_text(&shown, style_index(&shown, crate::look::TIME)).as_ref(), b"6 min");
}


#[test]
fn a_chats_line_counts_the_delegates_it_waits_on() {
    let mut line = line();
    line.waits_on = WaitsOn::Delegates;
    let (tree, _) = tree(&line);
    assert!(fixtures::find(&tree, Role::Status, b"waiting on 3 delegates").is_some());
    assert_eq!(
        fixtures::styles(&tree, Role::Status).as_ref(),
        [Box::from([crate::look::STATUS_WAITING, crate::look::STATE_NEEDS])]
    );
}


#[test]
fn a_chats_line_reads_its_owner_wake_policy_for_news() {
    let mut line = line();
    line.waits_on = WaitsOn::News;
    let ChipPhaseFacts { details, .. } = line.chip.details.as_mut();
    details.wake.words = WakeRule::Immediate;
    details.live_delegates = 0;
    let (tree, _) = tree(&line);
    assert!(fixtures::find(&tree, Role::Status, b"waiting on your words").is_some());
}


#[test]
fn an_open_chat_at_rest_says_waiting_on_your_words_on_its_line() {
    let mut line = line();
    line.flags.ended_by = EndedBy::Requester;
    line.waits_on = WaitsOn::News;
    let ChipPhaseFacts { phase, details } = line.chip.details.as_mut();
    *phase = ChipPhase::Active(RunStage::Idle);
    details.wake.words = WakeRule::Immediate;
    details.live_delegates = 0;
    let (tree, _) = tree(&line);
    assert!(fixtures::find(&tree, Role::Status, b"waiting on your words").is_some());
}

#[test]
fn a_closed_chat_says_closed_on_its_line() {
    for (by, expected) in [
        (Some(Requester::Person(73)), b"Closed".as_slice()),
        (Some(Requester::Person(74)), b"Closed"),
        (None, b"Closed"),
    ] {
        let mut line = line();
        let ChipPhaseFacts { phase, details } = line.chip.details.as_mut();
        *phase = ChipPhase::Ended;
        details.ended_at = Some(Wall::EPOCH);
        details.ending = Some(match by {
            Some(by) => ChipEnding::Cancelled {
                by,
                reason: DocumentText::complete(fixtures::first_run::text(b"Chat closed")),
                result: None,
                at: Wall::EPOCH,
            },
            None => {
                ChipEnding::Done(ResultValue::Report(DocumentText::complete(fixtures::first_run::text(b"A report"))))
            }
        });
        let (tree, _) = tree(&line);
        assert!(fixtures::find(&tree, Role::Status, expected).is_some());
        assert_eq!(fixtures::styles(&tree, Role::Status).as_ref(), [Box::from([crate::look::STATUS_QUIET])]);
    }
}

#[test]
fn delegate_counts_are_plural_and_absent_at_zero() {
    for (count, expected) in [(0, None), (1, Some(b"1 delegate".as_slice())), (2, Some(b"2 delegates".as_slice()))] {
        let mut line = line();
        let ChipPhaseFacts { details, .. } = line.chip.details.as_mut();
        details.live_delegates = count;
        let (tree, _) = tree(&line);
        if let Some(words) = expected {
            assert!(skein_lib::bytes::find(&fixtures::text(&tree), words).is_some());
        } else {
            let mut paragraphs = 0_u32;
            for node in tree.nodes() {
                if node.role == Role::Paragraph {
                    paragraphs += 1;
                }
            }
            assert_eq!(paragraphs, 3, "status and age, title, spend; number is a command container");
        }
    }
}


#[test]
fn done_and_failed_chats_say_closed_in_the_list_and_bar() {
    for ending in [
        ChipEnding::Done(ResultValue::Report(DocumentText::complete(fixtures::first_run::text(b"Done")))),
        ChipEnding::Failed(DocumentText::complete(fixtures::first_run::text(b"Failed"))),
    ] {
        let mut line = line();
        let ChipPhaseFacts { phase, details } = line.chip.details.as_mut();
        *phase = ChipPhase::Ended;
        details.live_delegates = 0;
        details.ended_at = Some(Wall::EPOCH);
        details.ending = Some(ending);
        let (tree, _) = tree(&line);
        assert!(fixtures::find(&tree, Role::Status, b"Closed").is_some());
        let root = fixtures::root_in_project(None);
        let me = root.pages().me(root.client()).unwrap();
        let mut bar = Builder::new(&fixtures::tree_limits());
        crate::pages::chats::status(
            &mut bar,
            &line.chip,
            (&line.waits_on, Some(0), None),
            (line.project, line.requester),
            None,
            me,
            &fixtures::words(),
        );
        assert!(fixtures::find(&bar.finish(), Role::Status, b"Closed").is_some());
    }
}


#[test]
fn rows_omit_their_project_and_name_only_kept_ended_transcripts() {
    for ended in [false, true] {
        for kept in [false, true] {
            let mut line = line();
            if ended {
                line.ended_at = Some(Wall::EPOCH);
            }
            line.flags.retention = if kept { Retention::Kept } else { Retention::ForgottenOnceRead };
            let (tree, _) = tree(&line);
            assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"Shop").is_none());
            assert_eq!(
                fixtures::find(&tree, Role::Paragraph, b"Closed. Its transcript is kept.").is_some(),
                ended && kept
            );
        }
    }
}


#[test]
fn a_kept_ended_row_without_a_clock_is_closed_even_with_stale_wait_metadata() {
    let mut line = line();
    line.chip.phase = Phase::Ended;
    line.waits_on = WaitsOn::Delegates;
    let (tree, _) = tree(&line);
    assert!(fixtures::find(&tree, Role::Status, b"Closed").is_some());
    assert!(fixtures::find(&tree, Role::Paragraph, b"Closed. Its transcript is kept.").is_some());
    assert!(skein_lib::bytes::find(&fixtures::text(&tree), b"3 delegates").is_none());
}


#[test]
fn asking_rows_name_the_chip_executor_in_the_top_line_in_both_shells() {
    use crate::look;
    let source = fixtures::chat::messaging_root();
    for listed in [false, true] {
        let (mut root, me_watch, _) = fixtures::root_at(
            temper_client_domain::Address::Chats { filter: ChatsFilter::All },
            &fixtures::domain_limits(),
        );
        let mut me = source.pages().me(source.client()).expect("me").clone();
        if listed {
            me = fixtures::asking_me(me);
        }
        fixtures::deliver_me(&mut root, me_watch, me);
        for (executor, tagged) in [
            (ChipExecutor::Agent { charter: 37 }, listed),
            (ChipExecutor::Agent { charter: 1 }, false),
            (ChipExecutor::Party { party: 37 }, false),
        ] {
            for words in [fixtures::words(), fixtures::browser_words()] {
                let mut line = line();
                line.chip.executor = executor;
                let mut b = Builder::new(&fixtures::tree_limits());
                crate::pages::chats::row(
                    &mut b,
                    &root,
                    &line,
                    &words,
                    &mut crate::bindings::Bindings::new(16),
                    &mut crate::words::Times::new(fixtures::NOW),
                );
                let tree = b.finish();
                let top = style_index(&tree, look::CHATS_TOP);
                let top_text = fixtures::node_text(&tree, top);
                assert_eq!(skein_lib::bytes::find(&top_text, "T9 · Ask temper".as_bytes()).is_some(), tagged);
                assert_eq!(tree.nodes()[top].role, Role::Slot);
                let mut print = skein_ui_text::Print::new(&fixtures::tree_limits()).unwrap();
                skein_ui_text::render(&tree, b"temper", None, &mut print);
                assert_eq!(skein_lib::bytes::find(print.text.as_slice(), b"Ask temper").is_some(), tagged);
                if tagged && words.shell == crate::words::Shell::Terminal {
                    assert!(
                        skein_lib::bytes::find(print.text.as_slice(), "[1] T9 · Ask temper\n".as_bytes()).is_some()
                    );
                }
                assert!(skein_lib::bytes::find(print.text.as_slice(), b"A renamed role").is_none());
            }
        }
    }
}


#[test]
fn a_paused_chat_s_row_says_paused() {
    let mut line = line();
    super::chat::pause_chip(&mut line.chip);
    for words in [fixtures::words(), fixtures::browser_words()] {
        let mut b = Builder::new(&fixtures::tree_limits());
        crate::pages::chats::row(
            &mut b,
            &fixtures::root_in_project(Some(fixtures::PROJECT)),
            &line,
            &words,
            &mut crate::bindings::Bindings::new(fixtures::tree_limits().bindings),
            &mut crate::words::Times::new(fixtures::NOW),
        );
        let tree = b.finish();
        let mut status = None;
        for node in tree.nodes() {
            if node.role == Role::Status && node.name.as_deref() == Some(b"Paused") {
                status = Some(node);
                break;
            }
        }
        let status = status.expect("paused row");
        assert_eq!(status.styles.as_slice(), [crate::look::STATUS_QUIET]);
    }
}
