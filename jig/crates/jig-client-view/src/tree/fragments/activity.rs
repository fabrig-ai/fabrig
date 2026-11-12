//! Standalone immutable activity facts (client/domain/objects.md, 10;
//! client/domain/contract/documents.md and client/domain/contract/README.md, 5). Archive references stay references; no decoded turn,
//! sender name, fresh tail or run failure explanation is synthesized.
use crate::words::Words;
use crate::words::activity_words::ActivityPhrase as P;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use jig_client::contract::{
    Activity, ActivityLine, FailureTries, HoldFailure, MessageKind, MessageProposalKind, NewsClass, NoticeState,
    Requester, RunEnd, RunWhy, Word, WordResult,
};

fn say(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    let value =
        words.book().say(words.language(), phrase.id(), holes).expect("checked source activity sayings and holes");
    b.text(Run::Plain, &value.text);
}
fn caption(b: &mut Builder, words: &Words, phrase: P) {
    say(b, words, phrase, &[]);
}
fn number(b: &mut Builder, words: &Words, phrase: P, value: u64) {
    say(b, words, phrase, &[Hole::Number(value)]);
}
fn region(b: &mut Builder, words: &Words, phrase: P) {
    b.open(Role::Region);
    let value = words.book().say(words.language(), phrase.id(), &[]).expect("checked activity region name");
    b.name(&value.text);
}
fn identity(b: &mut Builder, value: Requester, words: &Words) {
    match value {
        Requester::Task(value) => number(b, words, P::TaskIdentity, value),
        Requester::Person(value) => number(b, words, P::PersonIdentity, value),
        Requester::Deployment { project } => number(b, words, P::DeploymentIdentity, u64::from(project)),
    }
}
fn proposal_kind(b: &mut Builder, value: MessageProposalKind, words: &Words) {
    let phrase = match value {
        MessageProposalKind::Settings => return,
        MessageProposalKind::Batch => P::Batch,
        MessageProposalKind::Effect => P::Effect,
        MessageProposalKind::Amend => P::Amend,
        MessageProposalKind::Widen => P::Widen,
        MessageProposalKind::Release => P::Release,
    };
    caption(b, words, phrase);
}
fn notice(b: &mut Builder, value: NoticeState, words: &Words) {
    let phrase = match value {
        NoticeState::Held => P::Held,
        NoticeState::Done => P::Done,
        NoticeState::Failed => P::NoticeFailed,
        NoticeState::Cancelled => P::NoticeCancelled,
    };
    caption(b, words, phrase);
}
fn news(b: &mut Builder, value: NewsClass, words: &Words) {
    let phrase = match value {
        NewsClass::Wakes => P::Wakes,
        NewsClass::Kept => P::Kept,
        NewsClass::Dropped => P::Dropped,
    };
    caption(b, words, phrase);
}
fn result(b: &mut Builder, value: &WordResult, words: &Words) {
    match value {
        WordResult::Approved { state, words_present } => {
            caption(b, words, P::Approved);
            caption(b, words, P::State);
            crate::words::literal_segment::render(b, state.bytes());
            caption(b, words, if *words_present { P::WordsPresent } else { P::WordsAbsent });
        }
        WordResult::Report => caption(b, words, P::Report),
        WordResult::Verdict { code, fields } => {
            caption(b, words, P::Verdict);
            number(b, words, P::Code, u64::from(*code));
            for field in fields {
                caption(b, words, P::FieldName);
                crate::words::literal_segment::render(b, &field.name);
                caption(b, words, P::FieldValue);
                crate::words::literal_segment::render(b, &field.value);
            }
        }
        WordResult::Change { connector, kind, resource } => {
            caption(b, words, P::Change);
            number(b, words, P::Connector, u64::from(*connector));
            number(b, words, P::ChangeKind, u64::from(*kind));
            number(b, words, P::Resource, *resource);
        }
        WordResult::Failed => caption(b, words, P::ResultFailed),
        WordResult::Cancelled => caption(b, words, P::ResultCancelled),
    }
}
fn kind(b: &mut Builder, value: &MessageKind, words: &Words) {
    match value {
        MessageKind::Escalation { task, revision } => {
            caption(b, words, P::Escalation);
            number(b, words, P::TaskIdentity, *task);
            number(b, words, P::Revision, *revision);
        }
        MessageKind::Proposal { proposer, proposal, kind } => {
            caption(b, words, P::Proposal);
            number(b, words, P::Proposer, *proposer);
            number(b, words, P::ProposalNumber, *proposal);
            proposal_kind(b, *kind, words);
        }
        MessageKind::PersonStalled { task } => {
            caption(b, words, P::PersonStalled);
            number(b, words, P::TaskIdentity, *task);
        }
        MessageKind::SettingsDecision { proposal, outcome } => {
            caption(b, words, P::SettingsDecision);
            number(b, words, P::ProposalNumber, *proposal);
            match outcome {
                jig_client::contract::SettingsOutcome::Accepted { revision } => {
                    caption(b, words, P::Accepted);
                    number(b, words, P::Revision, u64::from(*revision));
                }
                jig_client::contract::SettingsOutcome::Rejected => caption(b, words, P::Rejected),
                jig_client::contract::SettingsOutcome::Stale { newest } => {
                    caption(b, words, P::SettingsStale);
                    number(b, words, P::Newest, u64::from(*newest));
                }
            }
        }
        MessageKind::ProposalDecision { proposal, accepted } => {
            caption(b, words, P::ProposalDecision);
            number(b, words, P::ProposalNumber, *proposal);
            caption(b, words, if *accepted { P::Accepted } else { P::Rejected });
        }
        MessageKind::Words => caption(b, words, P::MessageWords),
        MessageKind::EffectNotMade { entry } => {
            caption(b, words, P::EffectNotMade);
            number(b, words, P::Entry, *entry);
        }
        MessageKind::Amendment { revision } => {
            caption(b, words, P::Amendment);
            number(b, words, P::Revision, *revision);
        }
        MessageKind::QuestionAnswered { question } => number(b, words, P::QuestionAnswered, *question),
        MessageKind::QuestionOwed => caption(b, words, P::QuestionOwed),
        MessageKind::NothingAwaited => caption(b, words, P::NothingAwaited),
        MessageKind::QuestionEnded { question } => number(b, words, P::QuestionEnded, *question),
        MessageKind::Met { dependency, removed } => words.text(
            b,
            if *removed {
                crate::words::phrases::jig::JigPhrase::DependencyRemoved
            } else {
                crate::words::phrases::jig::JigPhrase::DependencyMet
            },
            &[Hole::Number(*dependency)],
        ),
        MessageKind::Question => caption(b, words, P::Question),
        MessageKind::Answer { question } => {
            caption(b, words, P::Answer);
            number(b, words, P::QuestionNumber, *question);
        }
        MessageKind::Notice { subscription, target, state } => {
            caption(b, words, P::Notice);
            number(b, words, P::Subscription, *subscription);
            number(b, words, P::Target, *target);
            notice(b, *state, words);
        }
        MessageKind::Timer { subscription } => {
            caption(b, words, P::Timer);
            number(b, words, P::Subscription, *subscription);
        }
        MessageKind::News { subscription, class } => {
            caption(b, words, P::News);
            number(b, words, P::Subscription, *subscription);
            news(b, *class, words);
        }
        MessageKind::Result(value) => {
            caption(b, words, P::Result);
            result(b, value, words);
        }
    }
}
/// Render every whole admitted message field. Absence of cut is not a fabricated loss flag.
pub fn word(b: &mut Builder, value: &Word, words: &Words) {
    region(b, words, P::Word);
    number(b, words, P::Message, value.number);
    caption(b, words, P::From);
    identity(b, value.from, words);
    kind(b, &value.kind, words);
    caption(b, words, P::WholeWords);
    crate::words::literal_segment::render(b, value.words.bytes());
    say(b, words, P::At, &[Hole::Wall(words.adjusted_wall(value.at))]);
    number(b, words, P::Hits, u64::from(value.hits));
    caption(b, words, if value.eligible { P::Eligible } else { P::Ineligible });
    b.close();
}
fn tries(b: &mut Builder, value: FailureTries, words: &Words) {
    number(b, words, P::TransientTries, u64::from(value.transient));
    number(b, words, P::PermanentTries, u64::from(value.permanent));
    number(b, words, P::RunTries, u64::from(value.run));
    number(b, words, P::AgentTries, u64::from(value.agent));
    number(b, words, P::LostTries, u64::from(value.lost));
    number(b, words, P::InvalidTries, u64::from(value.invalid));
    number(b, words, P::InfrastructureTries, u64::from(value.infrastructure));
}
fn why(b: &mut Builder, value: RunWhy, words: &Words) {
    match value {
        RunWhy::Made => caption(b, words, P::Made),
        RunWhy::Ready => caption(b, words, P::Ready),
        RunWhy::Message { message } => {
            caption(b, words, P::MessageCause);
            number(b, words, P::Message, message);
        }
        RunWhy::Released => caption(b, words, P::Released),
        RunWhy::Retry { tries: value } => {
            caption(b, words, P::Retry);
            tries(b, value, words);
        }
        RunWhy::Narrowed => caption(b, words, P::Narrowed),
        RunWhy::Changed => caption(b, words, P::Changed),
    }
}
fn failure(b: &mut Builder, value: HoldFailure, words: &Words) {
    let phrase = match value {
        HoldFailure::Transient => P::Transient,
        HoldFailure::Permanent => P::Permanent,
        HoldFailure::Run => P::RunFailure,
        HoldFailure::Agent => P::AgentFailure,
        HoldFailure::Lost => P::Lost,
        HoldFailure::Invalid => P::Invalid,
        HoldFailure::Infrastructure => P::Infrastructure,
    };
    caption(b, words, phrase);
}
fn end(b: &mut Builder, value: RunEnd, words: &Words) {
    match value {
        RunEnd::Parked => caption(b, words, P::Parked),
        RunEnd::Finished => caption(b, words, P::Finished),
        RunEnd::Cancelled => caption(b, words, P::CancelledEnd),
        RunEnd::Refused => caption(b, words, P::Refused),
        RunEnd::Failed { class, .. } => {
            caption(b, words, P::FailedEnd);
            failure(b, class, words);
        }
    }
}
fn line(b: &mut Builder, value: &Activity, words: &Words) {
    match value {
        Activity::Words { message, by, kind: value, words: text } => {
            caption(b, words, P::Words);
            number(b, words, P::Message, *message);
            caption(b, words, P::From);
            identity(b, *by, words);
            kind(b, value, words);
            caption(b, words, P::WholeWords);
            crate::words::literal_segment::render(b, text.bytes());
        }
        Activity::Heard(value) => {
            caption(b, words, P::Heard);
            word(b, value, words);
        }
        Activity::Notified { recipient, word: value } => {
            caption(b, words, P::Notified);
            caption(b, words, P::Recipient);
            identity(b, *recipient, words);
            word(b, value, words);
        }
        Activity::RunBegan { attempt, why: value, resumed } => {
            caption(b, words, P::RunBegan);
            number(b, words, P::Attempt, *attempt);
            why(b, *value, words);
            caption(b, words, if *resumed { P::Resumed } else { P::NotResumed });
        }
        Activity::RunEnded { attempt, end: value, spent } => {
            caption(b, words, P::RunEnded);
            number(b, words, P::Attempt, *attempt);
            end(b, *value, words);
            number(b, words, P::Spent, *spent);
        }
        Activity::Turn { reply, attempt, turn, read } => {
            caption(b, words, P::Turn);
            number(b, words, P::Reply, *reply);
            number(b, words, P::Attempt, *attempt);
            number(b, words, P::TurnNumber, u64::from(*turn));
            match read {
                Some(value) => number(b, words, P::Read, *value),
                None => caption(b, words, P::ReadAbsent),
            }
        }
        Activity::Decided { revision, .. } => {
            caption(b, words, P::Decided);
            number(b, words, P::Revision, *revision);
        }
        Activity::NowWritten { attempt, line } => {
            number(b, words, P::Attempt, *attempt);
            caption(b, words, P::NowWritten);
            crate::words::literal_segment::render(b, line.bytes());
        }
        Activity::Stood { attempt, words: text } => {
            caption(b, words, P::Parked);
            number(b, words, P::Attempt, *attempt);
            caption(b, words, P::WholeWords);
            crate::words::literal_segment::render(b, text.bytes());
        }
        Activity::TalkOpened { party } => {
            caption(b, words, P::TalkOpened);
            number(b, words, P::PersonIdentity, *party);
        }
        Activity::TalkEnded { lapsed, .. } => caption(b, words, if *lapsed { P::TalkLapsed } else { P::TalkEnded }),
        Activity::SessionEnded { attempt, end } => {
            match end {
                jig_client::contract::SessionEnd::Talk => caption(b, words, P::SessionTalk),
                jig_client::contract::SessionEnd::Left => caption(b, words, P::SessionLeft),
                jig_client::contract::SessionEnd::Quiet
                | jig_client::contract::SessionEnd::Bound
                | jig_client::contract::SessionEnd::Unoffered
                | jig_client::contract::SessionEnd::Fresh
                | jig_client::contract::SessionEnd::Closed
                | jig_client::contract::SessionEnd::Rules => {}
            }
            caption(b, words, P::RunEnded);
            number(b, words, P::Attempt, *attempt);
        }
        Activity::Kept { attempt, pending, question } => {
            if let Some(question) = question {
                number(b, words, P::KeptQuestion, *question);
            }
            words.text(b, crate::words::phrases::jig::JigPhrase::DependencyParked, &[]);
            number(b, words, P::Attempt, *attempt);
            for dependency in pending {
                words.text(b, crate::words::phrases::jig::JigPhrase::KeptUntil, &[Hole::Number(*dependency)]);
            }
        }
        Activity::Milestone { position } => {
            caption(b, words, P::Milestone);
            number(b, words, P::Position, *position);
        }
    }
}
/// Whole supplied task, commit position and clock plus its closed activity line.
pub fn render(b: &mut Builder, value: &ActivityLine, words: &Words) {
    region(b, words, P::Activity);
    number(b, words, P::Task, value.task);
    number(b, words, P::Position, value.position);
    say(b, words, P::At, &[Hole::Wall(words.adjusted_wall(value.at))]);
    line(b, &value.line, words);
    b.close();
}
