#![expect(clippy::disallowed_methods, reason = "ordinary proofs inspect public checked languages and exact trees")]
pub(in crate::tests) use crate::tests::kit::Goldens;
use crate::tests::kit::text;
use crate::tree::fragments::activity as view;
use crate::words::Words;
use crate::words::{activity_words as data, activity_words::ActivityPhrase as P};
use skein_ui::Limits;
use skein_ui::phrases::{Invalid, PhraseLimits, Phrasebook, Segment, SpanUnit, WallFormat};
use skein_ui::{Builder, Tree};

use jig_client::contract::*;
use skein_lib::{List, Wall, bytes::copy_of};

fn fields() -> List<ResultField> {
    crate::tests::kit::list_in(
        3,
        [
            ResultField { name: copy_of(&[255, 0]), value: copy_of(b"opaque") },
            ResultField { name: copy_of(b"a/b"), value: copy_of(&[254, 1]) },
        ],
    )
}
fn fixture(kind: MessageKind) -> Word {
    Word {
        number: u64::MAX,
        from: Requester::Person(u64::MAX),
        kind,
        words: text(UTF8_TEXTS[0]),
        at: Wall::from_nanos(u64::MAX),
        hits: u32::MAX,
        eligible: true,
    }
}
fn activity(line: Activity) -> ActivityLine {
    ActivityLine { task: u64::MAX, position: u64::MAX, at: Wall::from_nanos(u64::MAX), line }
}
fn client_limits() -> crate::tests::fixtures::Profile {
    crate::tests::fixtures::Profile {
        view: crate::Config { ..(crate::tests::tree::cards::CLIENT).view },
        client: jig_client::DomainLimits { rows: 3, text: 8, ..(crate::tests::tree::cards::CLIENT).client },
    }
}
fn phrase_limits() -> PhraseLimits {
    PhraseLimits {
        languages: 2,
        phrases: crate::words::document_words::phrase_count(),
        holes: 4,
        segments: 8000,
        text_bytes: 150_000,
        hole_bytes: 8,
    }
}
fn normal_words() -> &'static Words {
    static WORDS: std::sync::OnceLock<Words> = std::sync::OnceLock::new();
    WORDS.get_or_init(|| words(false))
}
fn words(selected: bool) -> Words {
    let mut english = crate::words::phrases::english::language();
    english.wall = WallFormat::Nanos;
    english.span_unit = SpanUnit::Nanos;
    let languages: Box<[skein_ui::phrases::Language]> = if selected {
        let translated = crate::tests::kit::translated_language(b"selected activity: ");
        Box::new([english, translated])
    } else {
        Box::new([english])
    };
    Words::new(
        Phrasebook::load(crate::words::phrases::jig::shapes(8), languages, &phrase_limits()).unwrap(),
        u16::from(selected),
        0,
    )
}

fn build(value: &ActivityLine, words: &Words) -> Tree {
    let f = crate::tests::render_limits(&client_limits(), words).unwrap();
    let limits = f;

    let mut b = Builder::new(&limits);
    view::render(&mut b, value, words);
    let tree = b.finish();
    envelope(&tree, f);
    tree
}
fn build_word(value: &Word, words: &Words) -> Tree {
    let f = crate::tests::render_limits(&client_limits(), words).unwrap();
    let mut b = Builder::new(&f);
    view::word(&mut b, value, words);
    let tree = b.finish();
    envelope(&tree, f);
    tree
}
fn envelope(tree: &Tree, f: Limits) {
    assert!(tree.len() <= f.nodes);
    let mut total = 0_u64;
    for node in tree.nodes() {
        assert!(crate::tests::text_bytes(node) <= f.text_per_node);
        total = total.checked_add(u64::from(crate::tests::text_bytes(node))).unwrap();
        if let Some(name) = &node.name {
            assert!(name.len() <= usize::try_from(f.text_per_node).unwrap());
            total = total.checked_add(u64::try_from(name.len()).unwrap()).unwrap();
        }
        assert!(node.binding.is_none() && node.slot.is_none());
        if node.role == skein_ui::Role::Region {
            assert!(node.name.is_some());
        }
    }
    assert!(total <= u64::from(f.text_bytes));
}

fn kinds() -> Box<[MessageKind]> {
    Box::new([
        MessageKind::Escalation { task: u64::MAX, revision: u64::MAX },
        MessageKind::Proposal { proposer: u64::MAX, proposal: u64::MAX, kind: MessageProposalKind::Batch },
        MessageKind::PersonStalled { task: u64::MAX },
        MessageKind::ProposalDecision { proposal: u64::MAX, accepted: true },
        MessageKind::SettingsDecision { proposal: u64::MAX, outcome: SettingsOutcome::Accepted { revision: u32::MAX } },
        MessageKind::SettingsDecision { proposal: u64::MAX, outcome: SettingsOutcome::Rejected },
        MessageKind::SettingsDecision { proposal: u64::MAX, outcome: SettingsOutcome::Stale { newest: u32::MAX } },
        MessageKind::QuestionAnswered { question: u64::MAX },
        MessageKind::QuestionEnded { question: u64::MAX },
        MessageKind::QuestionOwed,
        MessageKind::Words,
        MessageKind::Met { dependency: 7, removed: false },
        MessageKind::EffectNotMade { entry: u64::MAX },
        MessageKind::Amendment { revision: u64::MAX },
        MessageKind::Question,
        MessageKind::Answer { question: u64::MAX },
        MessageKind::Notice { subscription: u64::MAX, target: u64::MAX, state: NoticeState::Held },
        MessageKind::Timer { subscription: u64::MAX },
        MessageKind::News { subscription: u64::MAX, class: NewsClass::Wakes },
        MessageKind::Result(WordResult::Report),
        MessageKind::NothingAwaited,
    ])
}
#[test]
fn owner_activity_all_nine_lines_keep_task_commit_clock_and_exact_variant_facts() {
    let mut goldens = Goldens::new("tree/fragments/activity/owner_activity_all_nine");
    let lines: Box<[Activity]> = Box::new([
        Activity::Words {
            message: u64::MAX,
            by: Requester::Task(u64::MAX),
            kind: MessageKind::Words,
            words: text(UTF8_TEXTS[0]),
        },
        Activity::Heard(fixture(MessageKind::Question)),
        Activity::Notified {
            recipient: Requester::Deployment { project: u32::MAX },
            word: fixture(MessageKind::Words),
        },
        Activity::RunBegan { attempt: u64::MAX, why: RunWhy::Made, resumed: false },
        Activity::RunEnded { attempt: u64::MAX, end: RunEnd::Refused, spent: u64::MAX },
        Activity::Turn { reply: u64::MAX, attempt: u64::MAX, turn: u32::MAX, read: Some(u64::MAX) },
        Activity::Decided { revision: u64::MAX },
        Activity::Milestone { position: u64::MAX },
        Activity::Kept { attempt: u64::MAX, pending: List::with_capacity(0), question: None },
    ]);
    assert_eq!(lines.len(), 9);
    for line in lines {
        let value = activity(line);
        let original = value.clone();
        let tree = build(&value, normal_words());
        goldens.check(&tree);
        assert_eq!(value, original);
    }
}

#[test]
fn owner_activity_all_message_kinds_keep_whole_word_hits_clock_and_flags() {
    let mut goldens = Goldens::new("tree/fragments/activity/owner_activity_all_message");
    assert_eq!(kinds().len(), 21);
    for kind in kinds() {
        let word = fixture(kind);
        let original = word.clone();
        let tree = build_word(&word, normal_words());
        goldens.check(&tree);
        assert_eq!(word, original);
        let mut ineligible = word;
        ineligible.eligible = false;
        ineligible.hits = 0;
        let tree = build_word(&ineligible, normal_words());
        goldens.check(&tree);
    }
}

#[test]
fn owner_activity_equal_sender_numbers_keep_task_person_and_deployment_distinct() {
    let mut goldens = Goldens::new("tree/fragments/activity/owner_activity_equal");
    let senders: Box<[Requester]> =
        Box::new([Requester::Task(7), Requester::Person(7), Requester::Deployment { project: 7 }]);
    let mut retained = List::with_capacity(3);
    for from in senders {
        let word = Word { from, ..fixture(MessageKind::Words) };
        let tree = build_word(&word, normal_words());
        goldens.check(&tree);
        for previous in &retained {
            assert_ne!(&word, previous);
        }
        let notified = activity(Activity::Notified { recipient: from, word: word.clone() });
        goldens.check(&build(&notified, normal_words()));
        retained.push(word).unwrap();
    }
    assert_eq!(retained.len(), 3);
    let tree = build_word(
        &Word { from: Requester::Deployment { project: u32::MAX }, ..fixture(MessageKind::Words) },
        normal_words(),
    );
    goldens.check(&tree);
}
#[test]
fn owner_activity_closed_run_causes_and_ends_keep_all_six_failure_classes_and_counters() {
    let mut goldens = Goldens::new("tree/fragments/activity/owner_activity_closed");
    let tries = FailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: u32::MAX };
    let causes: Box<[RunWhy]> = Box::new([
        RunWhy::Made,
        RunWhy::Ready,
        RunWhy::Message { message: u64::MAX },
        RunWhy::Released,
        RunWhy::Retry { tries },
        RunWhy::Narrowed,
        RunWhy::Changed,
    ]);
    for why in causes {
        for resumed in [false, true] {
            let value = activity(Activity::RunBegan { attempt: u64::MAX, why, resumed });
            let tree = build(&value, normal_words());
            goldens.check(&tree);
        }
    }
    let tree = build(
        &activity(Activity::RunBegan { attempt: 9, why: RunWhy::Retry { tries }, resumed: true }),
        normal_words(),
    );
    goldens.check(&tree);
    let ends: Box<[RunEnd]> = Box::new([RunEnd::Parked, RunEnd::Finished, RunEnd::Cancelled, RunEnd::Refused]);
    for end in ends {
        let tree = build(&activity(Activity::RunEnded { attempt: u64::MAX, end, spent: u64::MAX }), normal_words());
        goldens.check(&tree);
    }
    assert_max_tries();
    let classes: Box<[HoldFailure]> = Box::new([
        HoldFailure::Transient,
        HoldFailure::Permanent,
        HoldFailure::Run,
        HoldFailure::Agent,
        HoldFailure::Lost,
        HoldFailure::Invalid,
    ]);
    for class in classes {
        let tree = build(
            &activity(Activity::RunEnded { attempt: 7, end: RunEnd::Failed { class }, spent: 0 }),
            normal_words(),
        );
        goldens.check(&tree);
    }
}
fn assert_max_tries() {
    let mut goldens = Goldens::new("tree/fragments/activity/assert_max_tries");
    let tries = FailureTries {
        transient: u32::MAX,
        permanent: u32::MAX,
        run: u32::MAX,
        agent: u32::MAX,
        lost: u32::MAX,
        invalid: u32::MAX,
    };
    let tree = build(
        &activity(Activity::RunBegan { attempt: 0, why: RunWhy::Retry { tries }, resumed: false }),
        normal_words(),
    );
    goldens.check(&tree);
}
#[test]
fn owner_activity_archive_read_none_zero_and_max_are_distinct_without_decoding_or_clock_ordering() {
    let mut goldens = Goldens::new("tree/fragments/activity/owner_activity_archive");
    let mut retained = List::with_capacity(3);
    for read in [None, Some(0), Some(u64::MAX)] {
        let value = activity(Activity::Turn { reply: u64::MAX, attempt: u64::MAX, turn: u32::MAX, read });
        let tree = build(&value, normal_words());
        goldens.check(&tree);
        for previous in &retained {
            assert_ne!(&value, previous);
        }
        retained.push(value).unwrap();
    }
    let first = ActivityLine { position: 9, at: Wall::from_nanos(100), ..activity(Activity::Decided { revision: 3 }) };
    let later = ActivityLine { position: 10, at: Wall::from_nanos(0), ..first.clone() };
    goldens.check(&build(&first, normal_words()));
    goldens.check(&build(&later, normal_words()));
    assert_eq!(first.position, 9);
    assert_eq!(later.position, 10);
    assert_time_presentation();
}
fn assert_time_presentation() {
    let mut goldens = Goldens::new("tree/fragments/activity/assert_time_presentation");
    let value = Word { at: Wall::from_nanos(123), ..fixture(MessageKind::Words) };
    let presentations: Box<[i32]> = Box::new([1_i32, 0_i32, -1_i32]);
    for offset in presentations {
        let mut words = normal_words().clone();
        words.set_offset_minutes(offset);
        goldens.check(&build_word(&value, &words));
        let line = ActivityLine { at: value.at, ..activity(Activity::Heard(value.clone())) };
        goldens.check(&build(&line, &words));
        assert_eq!(value.at.as_nanos(), 123);
    }
}
#[test]
fn owner_activity_all_six_result_metadata_keep_approved_presence_opaque_fields_and_change_identity() {
    let mut goldens = Goldens::new("tree/fragments/activity/owner_activity_all_six");
    let variants: Box<[WordResult]> = Box::new([
        WordResult::Approved { state: text(UTF8_TEXTS[1]), words_present: true },
        WordResult::Report,
        WordResult::Verdict { code: u32::MAX, fields: fields() },
        WordResult::Change { connector: u16::MAX, kind: u16::MAX, resource: u64::MAX },
        WordResult::Failed,
        WordResult::Cancelled,
    ]);
    for result in variants {
        let tree = build_word(&fixture(MessageKind::Result(result)), normal_words());
        goldens.check(&tree);
    }
    let tree = build_word(
        &fixture(MessageKind::Result(WordResult::Verdict { code: u32::MAX, fields: fields() })),
        normal_words(),
    );
    goldens.check(&tree);
    let codes: Box<[&[u8]]> = tree
        .nodes()
        .iter()
        .flat_map(|n| &n.text)
        .filter(|r| r.run == skein_ui::Run::Code)
        .map(|r| r.text.as_ref())
        .collect();
    assert_eq!(codes.as_ref(), [b"\\xFF\\x00".as_slice(), b"opaque", b"a/b", b"\\xFE\\x01", UTF8_TEXTS[0]].as_slice());
    let tree = build_word(
        &fixture(MessageKind::Result(WordResult::Change { connector: u16::MAX, kind: u16::MAX, resource: u64::MAX })),
        normal_words(),
    );
    goldens.check(&tree);
    for present in [false, true] {
        let word = Word {
            kind: MessageKind::Result(WordResult::Approved { state: text(b""), words_present: present }),
            words: text(b""),
            ..fixture(MessageKind::Words)
        };
        goldens.check(&build_word(&word, normal_words()));
    }
}
#[test]
fn owner_activity_nested_message_proposal_notice_news_and_decision_values_remain_closed() {
    let mut goldens = Goldens::new("tree/fragments/activity/owner_activity_nested");
    let proposals: Box<[MessageProposalKind]> = Box::new([
        MessageProposalKind::Batch,
        MessageProposalKind::Effect,
        MessageProposalKind::Amend,
        MessageProposalKind::Widen,
        MessageProposalKind::Release,
    ]);
    for kind in proposals {
        let tree =
            build_word(&fixture(MessageKind::Proposal { proposer: u64::MAX, proposal: 7, kind }), normal_words());
        goldens.check(&tree);
    }
    let states: Box<[NoticeState]> =
        Box::new([NoticeState::Held, NoticeState::Done, NoticeState::Failed, NoticeState::Cancelled]);
    for state in states {
        let tree = build_word(
            &fixture(MessageKind::Notice { subscription: u64::MAX, target: u64::MAX, state }),
            normal_words(),
        );
        goldens.check(&tree);
    }
    let classes: Box<[NewsClass]> = Box::new([NewsClass::Wakes, NewsClass::Kept, NewsClass::Dropped]);
    for class in classes {
        let tree = build_word(&fixture(MessageKind::News { subscription: 8, class }), normal_words());
        goldens.check(&tree);
    }
    for accepted in [false, true] {
        let tree = build_word(&fixture(MessageKind::ProposalDecision { proposal: u64::MAX, accepted }), normal_words());
        goldens.check(&tree);
    }
}
#[test]
fn owner_activity_selected_normal_language_and_full_opaque_tree_envelope_refuse_whole_excess() {
    let mut goldens = Goldens::new("tree/fragments/activity/owner_activity_selected");
    let selected = words(true);
    let mut word = fixture(MessageKind::Words);
    word.kind = MessageKind::Result(WordResult::Verdict { code: u32::MAX, fields: List::with_capacity(3) });
    if let MessageKind::Result(WordResult::Verdict { fields, .. }) = &mut word.kind {
        for _ in 0_u32..3 {
            fields.push(ResultField { name: copy_of(&[254; 8]), value: copy_of(&[253; 8]) }).unwrap();
        }
    }
    word.words = text(UTF8_TEXTS[2]);
    let value =
        activity(Activity::Notified { recipient: Requester::Deployment { project: u32::MAX }, word: word.clone() });
    let tree = build(&value, &selected);
    goldens.check(&tree);
    let cloned = selected.clone();
    assert_eq!(build(&value, &cloned).nodes(), tree.nodes());
    for phrase in data::ALL {
        assert!(selected.book().capacity(phrase.id()).is_some());
    }
}
#[test]
fn owner_activity_checked_normal_book_rejects_missing_duplicate_wrong_number_and_wall_holes() {
    let mut language = crate::words::phrases::english::language();
    language.phrases = language.phrases.into_vec().into_iter().filter(|p| p.id != P::Activity.id()).collect();
    assert_eq!(
        Phrasebook::load(crate::words::phrases::jig::shapes(8), Box::new([language]), &phrase_limits()).err(),
        Some(Invalid::Missing { id: P::Activity.id() })
    );
    let mut language = crate::words::phrases::english::language();
    language.phrases.iter_mut().find(|p| p.id == P::Activity.id()).unwrap().id = P::Word.id();
    assert_eq!(
        Phrasebook::load(crate::words::phrases::jig::shapes(8), Box::new([language]), &phrase_limits()).err(),
        Some(Invalid::Duplicate { id: P::Word.id() })
    );
    for phrase in [P::Task, P::At] {
        let mut language = crate::words::phrases::english::language();
        language.phrases.iter_mut().find(|p| p.id == phrase.id()).unwrap().forms[0].segments =
            Box::new([Segment::Text(copy_of(b"missing typed hole"))]);
        assert_eq!(
            Phrasebook::load(crate::words::phrases::jig::shapes(8), Box::new([language]), &phrase_limits()).err(),
            Some(Invalid::Holes { id: phrase.id() })
        );
    }
}

const UTF8_TEXTS: [&[u8]; 3] = [b"~\0", b"!\0", &[b'~'; 8]];

#[test]
fn a_kept_run_is_parked_and_met_names_the_ended_or_removed_dependency() {
    let mut goldens = Goldens::new("tree/fragments/activity/a_kept_run");
    let pending = crate::tests::kit::list([7, 8]);
    let tree = build(&activity(Activity::Kept { attempt: 3, pending, question: None }), normal_words());
    goldens.check(&tree);
    for removed in [false, true] {
        let tree =
            build(&activity(Activity::Heard(fixture(MessageKind::Met { dependency: 7, removed }))), normal_words());
        goldens.check(&tree);
    }
}

#[test]
fn dependency_refusals_name_the_circle_the_ending_and_the_kept_answer() {
    let mut goldens = Goldens::new("tree/fragments/activity/dependency_refusals_name");
    let pending = crate::tests::kit::list([7, 5]);
    for refusal in [
        Refusal::Circle { tasks: pending.clone(), cut: true },
        Refusal::Kept { pending },
        Refusal::Dependency { dependency: 7, how: TaskStatus::Failed },
        Refusal::Dependency { dependency: 5, how: TaskStatus::Cancelled },
    ] {
        let f = crate::tests::render_limits(&client_limits(), normal_words()).unwrap();
        let mut b = Builder::new(&f);
        normal_words().refusal(&mut b, &refusal, client_limits().client.text);
        goldens.check(&b.finish());
    }
}

#[test]
fn a_kept_activity_names_its_question_as_a_question() {
    let mut goldens = Goldens::new("tree/fragments/activity/a_kept_activity");
    let tree = build(
        &activity(Activity::Kept { attempt: 3, pending: List::with_capacity(0), question: Some(12) }),
        normal_words(),
    );
    goldens.check(&tree);
}
