//! Shared words match the browser's trees (temper/docs/design/client/tui/pages/README.md).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary tests inspect semantic trees and bindings"
)]
use super::fixtures as f;
use crate::{shared, words::Words};
use jig_client::contract as c;
use skein_lib::{List, Wall};
use temper_client_domain::{self as d, Event, pages::Act};

pub(crate) fn act(root: &mut d::Domain, act: Act) {
    f::root_event(root, Event::Act { action: d::Action::Pages(act) });
}

#[derive(Debug, PartialEq, Eq)]
enum NeedAction {
    Answer(shared::Question),
    Review { words: Box<[u8]>, address: Box<[u8]> },
    Web,
}
#[derive(Debug, PartialEq, Eq)]
struct NeedValue {
    object: c::ObjectKey,
    project: Option<u32>,
    mark: shared::NeedMark,
    line: shared::NeedLine,
    act: NeedAction,
}
fn need_value(object: c::ObjectKey, mark: shared::NeedMark, words: &[u8], act: NeedAction) -> NeedValue {
    NeedValue {
        object,
        project: Some(f::PROJECT),
        mark,
        line: shared::NeedLine {
            kind: if mark == shared::NeedMark::Question { shared::NeedKind::Question } else { shared::NeedKind::Other },
            words: words.into(),
        },
        act,
    }
}
fn prompt_question(message: u64, admitted: bool) -> (c::ObjectKey, c::Read, c::ReadValue) {
    let mut chip = f::task::document().chip;
    chip.task = f::INBOX_TASK;
    chip.title = f::task::document_words(b"Parser");
    (
        c::ObjectKey::Question { task: f::INBOX_TASK, message },
        c::Read::Question { task: f::INBOX_TASK, message },
        c::ReadValue::Question(Box::new(c::Question {
            task: chip,
            from: None,
            message,
            words: c::DocumentText::complete(f::first_run::text(
                b"Which parser?\nChoice: Keep it\nChoice: Replace it (recommended)",
            )),
            since: Wall::EPOCH,
            acts: if admitted {
                List::from_boxed(Box::new([c::Offered {
                    act: c::ActKind::Answer,
                    admission: c::Admission::Allowed,
                    effect: None,
                }]))
            } else {
                List::with_capacity(0)
            },
            ending: None,
        })),
    )
}
fn question_need(message: u64) -> NeedValue {
    let object = c::ObjectKey::Question { task: f::INBOX_TASK, message };
    need_value(
        object,
        shared::NeedMark::Question,
        b"Parser: Which parser?",
        NeedAction::Answer(shared::Question {
            object,
            head: b"Which parser?".as_slice().into(),
            choices: Box::new([
                shared::QuestionChoice { words: b"Keep it".as_slice().into(), recommended: false },
                shared::QuestionChoice { words: b"Replace it".as_slice().into(), recommended: true },
            ]),
            primary: Some(1),
        }),
    )
}
fn held_needs(reason: c::HoldCategory) -> d::Domain {
    let held = f::needs::held(Some(reason));
    let task = held.task.chip.task;
    f::needs::kinds(&[(
        c::ObjectKey::Escalation(task),
        c::Read::Escalation { task },
        c::ReadValue::Escalation(Box::new(held)),
    )])
}
fn approval_needs(row: &d::forge::approvals::Row) -> d::Domain {
    let mut root = f::needs::kinds(&[]);
    f::needs::approvals(&mut root, core::slice::from_ref(row), 0);
    root
}
fn review_need(row: &d::forge::approvals::Row) -> NeedValue {
    need_value(
        c::ObjectKey::Task(row.task),
        shared::NeedMark::SignOff,
        b"A change: Approve A change on Forgejo",
        NeedAction::Review {
            words: "Review on Forgejo ↗".as_bytes().into(),
            address: b"https://forge.example/work/hello/pulls/24/files".as_slice().into(),
        },
    )
}
struct NeedCase {
    name: &'static str,
    root: Box<d::Domain>,
    expected: Box<[NeedValue]>,
}
fn need_case(name: &'static str, root: d::Domain, expected: Box<[NeedValue]>) -> NeedCase {
    NeedCase { name, root: Box::new(root), expected }
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table checks each need's mark, prompt and merged order")]
fn need_prompts_say_each_kind() {
    use shared::NeedMark;
    let w = f::browser_words();
    let approval = f::needs::approval_row();
    let no_account = d::forge::approvals::Row { yours: d::forge::approvals::Yours::NoAccount, ..approval.clone() };
    let mut proposal = f::needs::proposal(c::Proposer::Task(f::INBOX_TASK));
    proposal.action = c::ProposalAction::Settings(Box::new(c::ProposalActionSettings {
        against: 1,
        items: List::with_capacity(0),
        review: None,
        made: None,
    }));
    let proposal_object = c::ObjectKey::Proposal(proposal.key);
    let proposal_read = c::Read::Proposal { proposal: proposal.key };
    let mut ordered = f::needs::kinds(&[prompt_question(1, true), prompt_question(2, true)]);
    f::needs::approvals(&mut ordered, core::slice::from_ref(&approval), 0);
    act(
        &mut ordered,
        Act::Needs(d::pages::needs::Act::Later { object: c::ObjectKey::Question { task: f::INBOX_TASK, message: 1 } }),
    );
    let mut no_inbox = d::unlinked(&f::domain_limits(), f::SEED);
    f::needs::approvals(&mut no_inbox, core::slice::from_ref(&approval), 0);
    let rows = [
        need_case(
            "question choices and primary",
            f::needs::kinds(&[prompt_question(1, true)]),
            Box::new([question_need(1)]),
        ),
        need_case(
            "question not admitted",
            f::needs::kinds(&[prompt_question(1, false)]),
            Box::new([need_value(
                c::ObjectKey::Question { task: f::INBOX_TASK, message: 1 },
                NeedMark::Question,
                b"Parser: Which parser?",
                NeedAction::Web,
            )]),
        ),
        need_case(
            "budget hold",
            held_needs(c::HoldCategory::Budget),
            Box::new([need_value(
                c::ObjectKey::Escalation(f::INBOX_TASK),
                NeedMark::Money,
                b"first-run-projects: 31 of 40 used",
                NeedAction::Web,
            )]),
        ),
        need_case(
            "other hold",
            held_needs(c::HoldCategory::Stalled),
            Box::new([need_value(
                c::ObjectKey::Escalation(f::INBOX_TASK),
                NeedMark::Other,
                b"first-run-projects: first-run-projects",
                NeedAction::Web,
            )]),
        ),
        need_case(
            "settings proposal",
            f::needs::kinds(&[(proposal_object, proposal_read, c::ReadValue::Proposal(Box::new(proposal)))]),
            Box::new([need_value(
                proposal_object,
                NeedMark::Other,
                b"Task 9: Accept how temper will run your work? The agent follows this plan.",
                NeedAction::Web,
            )]),
        ),
        need_case("counting approval", approval_needs(&approval), Box::new([review_need(&approval)])),
        need_case(
            "approval without account",
            approval_needs(&no_account),
            Box::new([need_value(
                c::ObjectKey::Task(approval.task),
                NeedMark::SignOff,
                b"A change: Approve A change on Forgejo",
                NeedAction::Web,
            )]),
        ),
        need_case(
            "cards then approvals then later",
            ordered,
            Box::new([question_need(2), review_need(&approval), question_need(1)]),
        ),
        need_case("no inbox", no_inbox, Box::default()),
    ];
    let mut failed = Vec::new();
    for NeedCase { name, root, expected } in rows {
        let prompts = shared::need_prompts(&root, &w)
            .into_vec()
            .into_iter()
            .map(|prompt| NeedValue {
                object: prompt.object,
                project: prompt.project,
                mark: prompt.mark,
                line: prompt.line,
                act: match prompt.act {
                    shared::NeedAct::Answer(question) => NeedAction::Answer(question),
                    shared::NeedAct::Review { words, address } => NeedAction::Review { words, address },
                    shared::NeedAct::Web => NeedAction::Web,
                },
            })
            .collect::<Vec<_>>();
        if prompts.as_slice() != expected.as_ref() {
            failed.push((name, prompts, expected));
        }
    }
    assert!(failed.is_empty(), "need prompt rows: {failed:?}");
}

#[test]
fn need_prompts_keep_the_wait_and_the_shared_reason() {
    use temper_protocol_reaches::{Class, Kind, Reach, Reason};
    let w = f::browser_words();
    let (mut question, _, watch) =
        f::needs::fixture_watched("Which parser?", false, 1, false, d::address::Filter::ALL, None);
    f::needs::deliver_asked(
        &mut question,
        watch,
        Some(Reach {
            kind: Kind::Implementation,
            class: Class::Ask,
            reason: Reason::Kind,
            under: 0,
            sets: 0,
            matched: 0,
            away: false,
        }),
    );
    let mut proposal = f::needs::proposal(c::Proposer::Task(f::INBOX_TASK));
    proposal.waiting.since = Wall::from_nanos(123);
    let proposal_root = f::needs::kinds(&[(
        c::ObjectKey::Proposal(proposal.key),
        c::Read::Proposal { proposal: proposal.key },
        c::ReadValue::Proposal(Box::new(proposal)),
    )]);
    let mut approval = f::needs::approval_row();
    approval.since = Wall::from_nanos(456);
    let person = f::needs::person();
    let without_wait = f::needs::kinds(&[(
        c::ObjectKey::PersonTask(f::INBOX_TASK),
        c::Read::PersonTask { task: f::INBOX_TASK },
        c::ReadValue::PersonTask(Box::new(person)),
    )]);
    let approval_root = approval_needs(&approval);
    for (root, since, why) in [
        (
            &question,
            Some(Wall::EPOCH),
            Some("Reached you because it is an implementation choice, which asks you unless you’re away."),
        ),
        (
            &proposal_root,
            Some(Wall::from_nanos(123)),
            Some("Reached you because a proposal always waits for whoever may accept it."),
        ),
        (&approval_root, Some(approval.since), None),
        (&without_wait, None, None),
    ] {
        let prompts = shared::need_prompts(root, &w);
        assert_eq!(prompts.len(), 1, "{since:?} {why:?}");
        let prompt = &prompts[0];
        assert_eq!(prompt.since, since);
        assert_eq!(prompt.why.as_deref(), why.map(str::as_bytes));
        let needs = root.pages().needs().needs(root.pages().inbox().unwrap(), root.forge().approvals());
        assert_eq!(prompt.why, shared::why(root, &w, needs[0]));
    }
}

#[test]
fn shared_person_line_names_the_reader_and_waits_only_when_read() {
    use d::pages::team::{Person, Waits};
    let w = Words::english(1, 4096, crate::words::Shell::Browser).expect("English");
    let mut person = Person {
        party: 1,
        you: true,
        name: b"Reader".as_slice().into(),
        role: c::HeldRole::Member,
        waits: None,
        signed: None,
    };
    assert_eq!(shared::person_line(&w, &person), None);
    person.waits = Some(Waits { questions: 2, reviews: 0, proposals: 0, held: 0, person_tasks: 0, cut: false });
    assert_eq!(shared::person_line(&w, &person).as_deref(), Some(b"Two questions wait on you.".as_slice()));
    person.waits.as_mut().expect("read waits").questions = 0;
    assert_eq!(shared::person_line(&w, &person).as_deref(), Some(b"Nothing waits on you.".as_slice()));
    person.you = false;
    person.name = b"Sam".as_slice().into();
    person.waits.as_mut().expect("read waits").reviews = 1;
    assert_eq!(shared::person_line(&w, &person).as_deref(), Some(b"One review waits on Sam.".as_slice()));
}

#[test]
fn shared_finding_refusal_says_full_and_context_free_owners_and_areas() {
    use temper_client_protocol_wire::v1 as w;
    let words = Words::english(1, 4096, crate::words::Shell::Browser).expect("English");
    let full = w::FindingRefusal::Full(w::FindingsFull { count: 128 });
    assert_eq!(shared::finding_refusal(&words, &full).as_ref(), b"the project's findings are full: 128 new");
    let areas = w::FindingRefusal::NoArea(w::FindingAreas {
        areas: List::from_boxed(Box::new([b"tests".as_slice().into(), b"core".as_slice().into()])),
    });
    assert_eq!(shared::finding_refusal(&words, &areas).as_ref(), b"No such area; its areas: tests, core");
    let held =
        w::FindingRefusal::NotYours(w::FindingNotYours { holder: w::Holder::Task(w::HolderTask { number: 41 }) });
    assert_eq!(shared::finding_refusal(&words, &held).as_ref(), b"Not yours: its owner is task 41");
}

#[test]
fn findings_reach_words_name_the_triager_and_the_work() {
    use d::pages::needs::Why;
    use temper_protocol_reaches::{Class, Classes, Item, Kind, Offer, Reach, Reason, Subject};
    let w = f::browser_words();
    let root = d::unlinked(&f::domain_limits(), f::SEED);
    let subject = Subject::Kind(Kind::Findings);
    let classes = Classes {
        items: Box::new([Item { subject, words: b"small repairs".as_slice().into(), class: Class::Ask }]),
        sets: Box::default(),
    };
    let reach = Reach {
        kind: Kind::Findings,
        class: Class::Ask,
        reason: Reason::Kind,
        under: 0,
        sets: 0,
        matched: 0,
        away: false,
    };
    for (plural, expected) in [
        (false, "Reached you because it starts work on findings, which asks you unless you’re away."),
        (true, "These reached you because they start work on findings, which asks you unless you’re away."),
    ] {
        assert_eq!(
            shared::why_words(&root, &w, &Why::Reach { project: f::PROJECT, reach }, plural).as_deref(),
            Some(expected.as_bytes())
        );
    }
    for (under, object, allowed, offer, rule) in [
        (
            0,
            "starting work on findings",
            "Allowed: starting work on findings.",
            "Let the triager start goals for findings, then tell you.",
            "Stop asking: let the triager start goals for findings",
        ),
        (
            1,
            "starting work on findings under “small repairs”",
            "Allowed: starting work on findings under “small repairs”.",
            "Let the triager start goals for findings under “small repairs”, then tell you.",
            "Stop asking: let the triager start goals for findings under “small repairs”",
        ),
    ] {
        let reach = Reach { under, ..reach };
        assert_eq!(shared::object(&w, subject, under, Some(&classes)).as_ref(), object.as_bytes());
        assert_eq!(
            shared::allowed(&w, &Reach { class: Class::Decide, ..reach }, Some(&classes)).as_ref(),
            allowed.as_bytes()
        );
        assert_eq!(shared::offer(&w, &classes, &reach, Offer::Decide(subject)).as_ref(), offer.as_bytes());
        assert_eq!(shared::rule(&w, &classes, &reach, Offer::Decide(subject)).as_ref(), rule.as_bytes());
    }
}

#[test]
fn delegation_headings_and_shared_lines_cut_titles_in_both_shells() {
    use skein_ui::{Builder, Role};
    for (title, cut) in f::row_titles() {
        let mut task = f::delegation::change();
        task.chip.title = f::delegation::document(title);
        for words in [f::browser_words(), f::words()] {
            assert_eq!(shared::delegation_line(&task, &words).title.as_ref(), cut);
            for terminal in [false, true] {
                let mut b = Builder::new(&f::tree_limits());
                let mut bindings = crate::bindings::Bindings::new(f::tree_limits().bindings);
                if terminal {
                    crate::cards::delegation::terminal_card(&mut b, &task, &words, &mut bindings);
                } else {
                    crate::cards::delegation::card(&mut b, &task, &words, &mut bindings);
                }
                let tree = b.finish();
                let heading = tree.nodes().iter().find(|node| node.role == Role::Heading { level: 3 }).unwrap();
                assert_eq!(heading.name.as_deref(), Some(cut));
                assert_eq!(f::text(&tree).windows(cut.len()).filter(|bytes| *bytes == cut).count(), 1);
            }
        }
    }
}

#[test]
fn proposal_titles_stay_whole_while_shared_lines_keep_their_cut() {
    let root = f::chat::proposal_root();
    let mut card = root.pages().cards().unwrap().as_slice().first().unwrap().clone();
    for (title, cut) in f::row_titles() {
        let mut page = f::cards::batch_proposal();
        let c::ReadValue::Proposal(proposal) = &mut page else { unreachable!() };
        let c::ProposalAction::Batch(tasks) = &mut proposal.action else { unreachable!() };
        tasks.get_mut(0).unwrap().spec.specification.words =
            f::delegation::document(&skein_lib::bytes::concat(&[title, b"\nThe rest is not the title"]));
        card.page = Some(page);
        assert_eq!(shared::proposal_title(&card).unwrap().as_ref(), title);
        for words in [f::browser_words(), f::words()] {
            assert_eq!(shared::proposal_line(&card, &words).unwrap().as_ref(), cut);
        }
    }
    let mut proposal = f::needs::proposal(c::Proposer::Task(f::INBOX_TASK));
    proposal.action = c::ProposalAction::Settings(Box::new(c::ProposalActionSettings {
        against: 1,
        items: List::with_capacity(0),
        review: None,
        made: None,
    }));
    let title = f::row_titles()[0].0;
    proposal.reason = f::delegation::document(&skein_lib::bytes::concat(&[title, b"\nReason body"]));
    card.page = Some(c::ReadValue::Proposal(Box::new(proposal)));
    assert_eq!(shared::proposal_title(&card).unwrap().as_ref(), title);
    assert_eq!(shared::proposal_line(&card, &f::words()).unwrap().as_ref(), title);
    let c::ReadValue::Proposal(proposal) = card.page.as_mut().unwrap() else { unreachable!() };
    proposal.action = c::ProposalAction::Batch(List::uncapped());
    assert_eq!(shared::proposal_title(&card).unwrap().as_ref(), title);
    assert_eq!(shared::proposal_line(&card, &f::words()).unwrap().as_ref(), title);
    card.page = Some(c::ReadValue::Task(Box::new(f::task::document())));
    assert_eq!(shared::proposal_title(&card), None);
    assert_eq!(shared::proposal_line(&card, &f::words()), None);
    card.page = None;
    assert_eq!(shared::proposal_title(&card), None);
    assert_eq!(shared::proposal_line(&card, &f::words()), None);
}

#[test]
fn result_cards_and_close_delegate_rows_cut_titles_in_both_shells() {
    use skein_ui::{Builder, Role};
    for (title, cut) in f::row_titles() {
        let mut task = f::delegation::change();
        task.chip.title = f::delegation::document(title);
        let result = c::Result {
            task: task.chip.clone(),
            ending: c::Ending::Done,
            value: None,
            at: None,
            details: None,
            acts: List::uncapped(),
        };
        let mut parent = f::task::document();
        parent.delegates =
            List::from_boxed(Box::new([c::LiveDelegate { chip: task.chip.clone(), takeover_budget: 0 }]));
        for words in [f::browser_words(), f::words()] {
            let mut b = Builder::new(&f::tree_limits());
            crate::cards::result(&mut b, &result, &words);
            let tree = b.finish();
            assert!(f::find(&tree, Role::Heading { level: 3 }, cut).is_some());
            assert!(skein_lib::bytes::find(&f::text(&tree), cut).is_some());
            let mut b = Builder::new(&f::tree_limits());
            let mut bindings = crate::bindings::Bindings::new(f::tree_limits().bindings);
            crate::confirm::fates(
                &mut b,
                &parent,
                &[c::Operation::Cancel {
                    project: f::PROJECT,
                    task: task.chip.task,
                    reason: f::delegation::text(b"closed"),
                    grace: None,
                }],
                crate::words::Word::CloseTitle,
                &words,
                &mut bindings,
            );
            let tree = b.finish();
            let fate = skein_lib::bytes::concat(&[b"T2 ", cut, b" is cancelled."]);
            assert!(skein_lib::bytes::find(&f::text(&tree), &fate).is_some());
        }
    }
}

#[test]
fn the_chat_switcher_cuts_titles_without_changing_its_destinations() {
    for (title, cut) in f::row_titles() {
        let mut row = f::chats::line();
        row.chip.title = f::delegation::document(title);
        let root = f::chats::listed(&[row], &[], false);
        for words in [f::browser_words(), f::words()] {
            let choices = shared::chats(&root, &words);
            let choice = choices
                .iter()
                .find(|choice| choice.act == Act::Go { address: d::Address::Chat { task: f::chats::TASK } })
                .unwrap();
            assert_eq!(choice.words.as_ref(), cut);
        }
    }
}
