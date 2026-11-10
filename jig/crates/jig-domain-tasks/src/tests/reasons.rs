//! Run reasons and offered kinds (jig/docs/design/domain/tasks/messages.md, 6).
use crate::{
    ALWAYS_OFFERED_NAMES, MessageKind as M, NewsClass, NoticeState, Party, REASON_NAMES, Reasons as R, ResultKind,
    Word, offered,
};
use skein_lib::Wall;

const SOURCE: u64 = 2;
const MESSAGE: u64 = 3;

#[test]
fn reasons_classify_every_kind_and_its_offering() {
    for (kind, from, bit, empty, own) in [
        (M::Question, Party::Task(SOURCE), R::QUESTION, false, true),
        (M::QuestionAnswered { question: MESSAGE }, Party::Task(SOURCE), R::QUESTION_ANSWERED, false, true),
        (M::QuestionEnded { question: MESSAGE }, Party::Task(SOURCE), R::QUESTION_ENDED, false, true),
        (M::QuestionOwed, Party::Task(SOURCE), R::QUESTION_OWED, false, true),
        (M::Escalation { task: SOURCE, revision: 1 }, Party::Task(SOURCE), R::ESCALATION, true, true),
        (
            M::Proposal { proposer: SOURCE, proposal: MESSAGE, kind: crate::ProposalKind::Batch },
            Party::Task(SOURCE),
            R::PROPOSAL,
            true,
            true,
        ),
        (M::PersonStalled { task: SOURCE }, Party::Task(SOURCE), R::PERSON_STALLED, false, true),
        (M::Result(ResultKind::Report), Party::Task(SOURCE), R::RESULT, false, true),
        (
            M::Notice { subscription: MESSAGE, target: SOURCE, state: NoticeState::Done },
            Party::Task(SOURCE),
            R::NOTICE,
            false,
            true,
        ),
        (M::NothingAwaited, Party::Task(SOURCE), R::NOTHING_AWAITED, false, true),
        (M::News { subscription: MESSAGE, class: NewsClass::Wakes }, Party::Task(SOURCE), R::NEWS, false, true),
        (M::Timer { subscription: MESSAGE }, Party::Task(SOURCE), R::TIMER, false, true),
        (M::Words, Party::Task(SOURCE), R::WORDS, false, true),
        (M::Words, Party::Person(SOURCE), R::WORDS, true, true),
        (M::Answer { question: MESSAGE }, Party::Task(SOURCE), R::EMPTY, true, true),
        (M::ProposalDecision { proposal: MESSAGE, accepted: true }, Party::Task(SOURCE), R::EMPTY, true, true),
        (
            M::SettingsDecision { proposal: MESSAGE, outcome: crate::SettingsOutcome::Rejected },
            Party::Task(SOURCE),
            R::EMPTY,
            true,
            true,
        ),
        (M::EffectNotMade { entry: MESSAGE }, Party::Task(SOURCE), R::EMPTY, true, true),
        (M::Met { dependency: SOURCE, removed: false }, Party::Task(SOURCE), R::EMPTY, true, true),
        (M::Amendment { revision: 1 }, Party::Task(SOURCE), R::EMPTY, true, true),
        (M::FinishRequested { revision: 1 }, Party::Task(SOURCE), R::EMPTY, true, true),
    ] {
        assert_eq!(R::of(&kind), bit, "{kind:?}: reason");
        let word = Word { number: MESSAGE, from, kind, words: Box::new([]), at: Wall::EPOCH, hits: 1, eligible: false };
        assert_eq!(offered(&word, R::EMPTY), empty, "{:?} from {from:?}: empty offering", word.kind);
        assert_eq!(offered(&word, bit), own, "{:?} from {from:?}: own offering", word.kind);
    }
    for (name, bit) in REASON_NAMES.into_iter().zip([
        R::FIRST,
        R::QUESTION,
        R::QUESTION_ANSWERED,
        R::QUESTION_ENDED,
        R::QUESTION_OWED,
        R::ESCALATION,
        R::PROPOSAL,
        R::PERSON_STALLED,
        R::RESULT,
        R::NOTICE,
        R::NOTHING_AWAITED,
        R::NEWS,
        R::TIMER,
        R::WORDS,
    ]) {
        assert_eq!(R::named(name), Some(bit), "{name:?}: name");
    }
    for name in ALWAYS_OFFERED_NAMES.into_iter().chain([b"unknown".as_slice()]) {
        assert_eq!(R::named(name), None, "{name:?}: not a reason");
    }
    assert_eq!(R::FIRST.union(R::QUESTION), R(3));
    assert!(R::ALL.contains(R::WORDS));
    assert!(!R::FIRST.meets(R::QUESTION));
    assert!(R::EMPTY.is_empty());
    assert!(!R(1 << 14).is_known(), "bit 14 is outside the stored vocabulary");
}
