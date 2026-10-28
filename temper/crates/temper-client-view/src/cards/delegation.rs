//! A delegation's card in a chat (client/README.md, 5.7).
use super::{name, text};
use crate::{
    frame::Bindings,
    look,
    words::{Word, Words},
};
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::PlanFilter;
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
        Kind::Approval => Word::DelegationApproval,
    };
    text(b, Role::Paragraph, &words.say(kind, &[Hole::Number(facts.task)]));
    status(b, facts.state, words);
    b.close();
    b.open(Role::Heading { level: 3 });
    b.style(look::TITLE);
    b.name(facts.title);
    b.text(Run::Plain, facts.title);
    b.close();
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
    crate::link(
        b,
        &name,
        bindings.insert(Act::Go { address: Address::Task { task: facts.task, filter: PlanFilter::All } }),
    );
    b.open(Role::Text);
    b.run(Run::Plain, &name);
    b.run(Run::Plain, " ↗".as_bytes());
    b.close();
    b.close();
    b.close();
}
pub(crate) fn status(b: &mut Builder, state: State, words: &Words) {
    let (phase, token) = match state {
        State::Running => (c::Phase::Running, look::STATUS_ACTIVE),
        State::Held => (c::Phase::Held, look::STATUS_HELD),
        State::Landed | State::Done => (c::Phase::Done, look::STATUS_DONE),
        State::Failed => (c::Phase::Failed, look::STATUS_HELD),
        State::Cancelled => (c::Phase::Cancelled, look::STATUS_QUIET),
        State::Ended => (c::Phase::Ended, look::STATUS_QUIET),
    };
    let name = if state == State::Landed {
        words.say(Word::Landed, &[])
    } else {
        words.say_word(jig_client::words::Word::Phase(phase), &[])
    };
    b.open(Role::Status);
    b.style(token);
    b.name(&name);
    b.text(Run::Plain, &name);
    b.close();
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
