//! A delegation's card in a chat (client/view.md, 5).
use super::{name, text};
use crate::{
    bindings::Bindings,
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address,
    pages::{
        Act,
        delegation::{self, Kind, Said, State},
    },
};

pub(crate) fn card(b: &mut Builder, task: &c::Task, words: &Words, bindings: &mut Bindings) {
    let facts = delegation::facts(task);
    b.open(Role::Region);
    b.name(&words.say(Word::Task, &[Hole::Number(facts.task)]));
    b.style(look::CARD);
    b.style(look::TASK_CARD);
    b.open(Role::Slot);
    b.style(look::META);
    let kind = match facts.kind {
        Kind::Change => Word::DelegationChange,
        Kind::Report => Word::DelegationReport,
        Kind::Verdict => Word::DelegationVerdict,
        Kind::Job => Word::DelegationJob,
        Kind::Approval => Word::DelegationApproval,
    };
    text(b, Role::Paragraph, &words.say(kind, &[Hole::Number(facts.task)]));
    if words.shell == Shell::Terminal {
        status(b, facts.state, words);
    }
    b.close();
    if words.shell == Shell::Browser {
        line_kind(b, facts, words);
    }
    b.open(Role::Heading { level: 3 });
    b.style(look::TITLE);
    if facts.goal {
        b.style(look::TOOL_FILE);
    }
    b.name(facts.title);
    b.text(Run::Plain, facts.title);
    b.close();
    if words.shell == Shell::Browser {
        status(b, facts.state, words);
    }
    if let Some(said) = facts.said {
        self::said(b, said, words);
    }
    if let Some(done) = facts.steps {
        b.open(Role::List);
        b.name(&words.say(Word::Steps, &[]));
        b.style(look::STEPS);
        for (index, step) in
            [Word::BranchPushed, Word::PullRequestOpened, Word::CiPassed, Word::Landed].iter().enumerate()
        {
            b.open(Role::Item);
            b.style(look::STEP);
            if index < usize::from(done) {
                b.style(look::STEP_DONE);
                b.state(skein_ui::State::Completed);
            }
            words.text(b, *step, &[]);
            b.close();
        }
        b.close();
    }
    let name = words.say(Word::Task, &[Hole::Number(facts.task)]);
    crate::bindings::link(b, &name, bindings.insert(Act::Go { address: Address::Task { task: facts.task } }));
    b.open(Role::Text);
    match words.shell {
        Shell::Browser => words.text(b, Word::DelegationOpen, &[]),
        Shell::Terminal => {
            b.run(Run::Plain, &name);
            b.run(Run::Plain, " ↗".as_bytes());
        }
    }
    b.close();
    b.close();
    b.close();
}
fn line_kind(b: &mut Builder, facts: delegation::Facts<'_>, words: &Words) {
    let word = if facts.goal {
        Word::DelegationLineGoal
    } else {
        match facts.kind {
            Kind::Change => Word::DelegationLineChange,
            Kind::Report => Word::DelegationLineReport,
            Kind::Verdict => Word::DelegationLineVerdict,
            Kind::Job => Word::DelegationLineJob,
            Kind::Approval => Word::DelegationLineApproval,
        }
    };
    text(b, Role::Paragraph, &words.say(word, &[]));
}
pub(crate) fn status(b: &mut Builder, state: State, words: &Words) {
    let token = status_token(state);
    let name = state_words(state, words);
    status_name(b, token, &name);
}
fn status_name(b: &mut Builder, token: skein_ui::Style, name: &[u8]) {
    b.open(Role::Status);
    b.style(token);
    b.name(name);
    b.text(Run::Plain, name);
    b.close();
}
fn status_token(state: State) -> skein_ui::Style {
    match state {
        State::Running => look::STATUS_ACTIVE,
        State::Held | State::Failed => look::STATUS_HELD,
        State::Landed | State::Cleared | State::Uncleared | State::Done => look::STATUS_DONE,
        State::Cancelled | State::Ended => look::STATUS_QUIET,
    }
}

pub(crate) fn said(b: &mut Builder, said: Said<'_>, words: &Words) {
    match said {
        Said::Merge(commit) => {
            b.open(Role::Paragraph);
            b.style(look::DETAILS);
            words.text(b, Word::LandedMerge, &[name(commit)]);
            b.close();
        }
        Said::Words(bytes) => {
            b.open(Role::Slot);
            b.style(look::DETAILS);
            skein_ui::markdown::read(bytes, 3, b);
            b.close();
        }
    }
}

/// The browser's terminal needs one line; the task's page holds its details (client/pages/README.md).
pub(crate) fn terminal_card(b: &mut Builder, task: &c::Task, words: &Words, bindings: &mut Bindings) {
    let facts = delegation::facts(task);
    b.open(Role::Region);
    b.style(look::TERMINAL_CARD);
    b.name(&words.say(Word::Task, &[Hole::Number(facts.task)]));
    b.open(Role::Slot);
    b.style(look::TERMINAL_CARD_LINE);
    let line = crate::shared::delegation_line(task, words);
    text(b, Role::Paragraph, &line.kind);
    text(b, Role::Heading { level: 3 }, &line.title);
    let token = status_token(facts.state);
    status_name(b, token, &line.state);
    let name = words.say(Word::Task, &[Hole::Number(facts.task)]);
    crate::bindings::link(b, &name, bindings.insert(Act::Go { address: Address::Task { task: facts.task } }));
    words.text(b, Word::DelegationOpen, &[]);
    b.close();
    b.close();
    b.close();
}

/// A terminal delegation's data (temper/docs/design/client/pages/README.md).
pub(crate) fn line(task: &c::Task, words: &Words) -> crate::shared::DelegationLine {
    let facts = delegation::facts(task);
    let kind = match facts.kind {
        Kind::Change => Word::DelegationChange,
        Kind::Report => Word::DelegationReport,
        Kind::Verdict => Word::DelegationVerdict,
        Kind::Job => Word::DelegationJob,
        Kind::Approval => Word::DelegationApproval,
    };
    crate::shared::DelegationLine {
        kind: words.say(kind, &[Hole::Number(facts.task)]),
        title: facts.title.into(),
        state: state_words(facts.state, words),
    }
}
fn state_words(state: State, words: &Words) -> Box<[u8]> {
    let phase = match state {
        State::Running => c::Phase::Running,
        State::Held => c::Phase::Held,
        State::Landed | State::Cleared | State::Uncleared | State::Done => c::Phase::Done,
        State::Failed => c::Phase::Failed,
        State::Cancelled => c::Phase::Cancelled,
        State::Ended => c::Phase::Ended,
    };
    if state == State::Landed {
        words.say(Word::Landed, &[])
    } else if state == State::Cleared || state == State::Uncleared {
        use temper_client_domain::forge::words::ChangeWord as W;
        crate::pages::change::say(words, if state == State::Cleared { W::ClearedTag } else { W::UnclearedTag }, &[])
    } else {
        words.say_word(jig_client::words::Word::Phase(phase), &[])
    }
}
