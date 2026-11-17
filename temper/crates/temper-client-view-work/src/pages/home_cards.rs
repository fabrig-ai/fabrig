//! Home's first three needs (temper/docs/design/client/pages/README.md, Home, The cards).
use super::ask;
use crate::{
    bindings::Bindings,
    look,
    words::{Shell, Times, Word, Words},
};
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    address::Filter,
    pages::{Act, findings, needs},
};

pub fn cards(
    b: &mut Builder,
    root: &Domain,
    needs: &[needs::Need<'_>],
    across: bool,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    let mut open = skein_lib::List::with_capacity(u32::try_from(needs.len()).expect("bounded needs"));
    for need in needs {
        open.push(findings::Need::Existing(*need)).expect("bounded needs");
    }
    all(b, root, open.as_slice(), across, w, bindings, times);
}
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "card placement shares question acts and reasons with groups and keeps the first approval's context"
)]
fn existing(
    b: &mut Builder,
    root: &Domain,
    needs: &[needs::Need<'_>],
    across: bool,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    own_reason: bool,
    first: bool,
) {
    for need in needs {
        if !needs::ready(root, *need) {
            continue;
        }
        let group = root.pages().needs().findings().group(findings::Need::Existing(*need));
        b.open(Role::Slot);
        b.style(look::NEED_CARD);
        if let Some(group) = &group {
            b.style(look::GROUP_CARD);
            b.open(Role::Paragraph);
            b.style(look::NEED_FROM);
            super::findings::group_from(b, root, group, w, times, across);
            if root.pages().needs().later(need.object()) {
                b.text(Run::Plain, &w.say(Word::NeedsLaterMeta, &[]));
            }
            b.close();
        } else {
            from(b, root, *need, across, w, bindings, times);
        }
        let (_, mut question, _) = super::home::need_words(need, w);
        if let needs::Need::Card(card) = need
            && let ask::Kind::Funds { spent, limit } = ask::read(card, w).kind
        {
            question = w.say(Word::NeedsFundsUsed, &[Hole::Number(spent), Hole::Number(limit)]);
        }
        ask::text(b, Role::Paragraph, &question, look::ASK_QUESTION);
        if let Some(group) = &group {
            super::findings::group_rows(b, group, w, bindings, times);
            if let needs::Need::Card(card) = need {
                super::findings::group_context(b, &ask::read(card, w));
            }
        }
        match need {
            needs::Need::Card(card) => {
                if own_reason {
                    super::needs::why_line(b, root, w, bindings, needs::Need::Card(card), super::needs::WhyForm::Short);
                }
            }
            needs::Need::Approval(row) => super::approval::size(b, row, w),
        }
        if first && let needs::Need::Approval(row) = need {
            for line in &super::approval::sentences(row, super::approval::viewer(root), w) {
                ask::text(b, Role::Paragraph, line, look::ASK_CONTEXT);
            }
        }
        b.open(Role::Slot);
        b.style(look::NEED_ACTS);
        match need {
            needs::Need::Card(card) => {
                let mut asked = ask::read(card, w);
                ask::primary_first(&mut asked);
                if let Some(outcome) = root.pages().needs().refused(card.object) {
                    crate::confirm::outcome_words(b, w, outcome);
                }
                if card.key.is_some() || card.outcome.is_some() {
                    crate::confirm::card_controls(b, root, card, w, bindings);
                } else {
                    match asked.kind {
                        ask::Kind::Question => super::needs::choices(b, &asked, w, bindings),
                        ask::Kind::Proposal | ask::Kind::Held | ask::Kind::Funds { .. } | ask::Kind::PersonTask => {
                            crate::confirm::needs_controls(b, root, card, w, bindings);
                        }
                    }
                    match asked.kind {
                        ask::Kind::PersonTask => person_task(b, asked.source, w, bindings),
                        ask::Kind::Question => tell(
                            b,
                            need,
                            w,
                            bindings,
                            match &asked.parsed {
                                Some(parsed) => parsed.choices.is_empty(),
                                None => true,
                            },
                            group.is_some(),
                        ),
                        ask::Kind::Proposal | ask::Kind::Held | ask::Kind::Funds { .. } => {}
                    }
                }
            }
            needs::Need::Approval(row) => {
                if row.yours == temper_client_domain::forge::approvals::Yours::Counts
                    && skein_ui::markdown::safe_url(&row.address)
                {
                    let label = super::approval::review_words(row, w);
                    b.open(Role::Link);
                    b.style(look::RECOMMENDED);
                    b.name(&label);
                    b.external(&row.address);
                    b.text(Run::Plain, &label);
                    b.close();
                }
                tell(b, need, w, bindings, false, false);
            }
        }
        b.close();
        if w.shell == Shell::Browser {
            ask::button(
                b,
                bindings,
                &w.say(Word::HomeCardMore, &[]),
                Act::Needs(needs::Act::Open { object: need.object() }),
                look::NEED_MORE,
            );
        }
        b.close();
    }
}

fn person_task(b: &mut Builder, source: Option<u64>, w: &Words, bindings: &mut Bindings) {
    if let Some(task) = source {
        ask::button(
            b,
            bindings,
            &w.say(Word::HomeOpen, &[]),
            Act::Go { address: Address::Task { task } },
            look::RECOMMENDED,
        );
    }
}
fn tell(b: &mut Builder, need: &needs::Need<'_>, w: &Words, bindings: &mut Bindings, primary: bool, linked: bool) {
    let label = w.say(Word::HomeCardTell, &[]);
    let act = Act::Needs(needs::Act::Open { object: need.object() });
    if linked {
        crate::bindings::link(b, &label, bindings.insert(act));
        b.text(Run::Plain, &label);
        b.style(if primary { look::RECOMMENDED } else { look::ASK_QUIET });
        b.close();
    } else {
        ask::button(b, bindings, &label, act, if primary { look::RECOMMENDED } else { look::ASK_QUIET });
    }
}
fn from(
    b: &mut Builder,
    root: &Domain,
    need: needs::Need<'_>,
    across: bool,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    let (title, source, project, when) = match need {
        needs::Need::Card(card) => {
            let asked = ask::read(card, w);
            (asked.title, asked.source, card.project, ask::need_when(card, w, times))
        }
        needs::Need::Approval(row) => {
            (row.title.clone(), Some(row.task), Some(row.project), super::approval::when(row, w, times))
        }
    };
    b.open(Role::Paragraph);
    b.style(look::NEED_FROM);
    if across
        && let Some(project) = project
        && let Some(name) = super::project_name(root.pages().me(root.client()), project)
    {
        ask::text(b, Role::Text, &name, look::CHIP);
        b.text(Run::Plain, b" ");
    }
    ask::from_title(b, need, &title, source, bindings, true);
    b.text(Run::Plain, &w.say(Word::ToolSeparator, &[]));
    b.text(Run::Plain, &when);
    if root.pages().needs().later(need.object()) {
        b.text(Run::Plain, &w.say(Word::NeedsLaterMeta, &[]));
    }
    b.close();
}

/// Both wide Homes close with the same question field (client/pages/README.md).
pub(crate) fn ask_line(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    if w.shell == Shell::Terminal {
        return;
    }
    b.open(Role::Slot);
    b.style(look::ASK_LINE);
    b.open(Role::TextField);
    b.name(&w.say(Word::AskTitle, &[]));
    b.placeholder(&w.say(Word::HomeAskPlaceholder, &[]));
    b.value(root.pages().ask_line(), temper_client_domain::pages::shown(root.pages()));
    b.state(State::SubmitsOnEnter);
    b.bind(bindings.insert(Act::AskLine { words: root.pages().ask_line().into() }));
    b.close();
    ask::button(
        b,
        bindings,
        &w.say(Word::HomeAsk, &[]),
        Act::AskWith { words: root.pages().ask_line().into() },
        look::ASK_QUIET,
    );
    b.close();
}

/// The first three open cards, with a shared reason only when every drawn reason agrees.
pub(crate) fn all(
    b: &mut Builder,
    root: &Domain,
    open: &[findings::Need<'_>],
    across: bool,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    let mut drawn = skein_lib::List::with_capacity(3);
    for need in open.iter().take(3) {
        if need.ready(root) {
            drawn.push(*need).expect("three cards");
        }
    }
    let mut shared = match drawn.get(0) {
        Some(need) if drawn.len() >= 2 => findings::why(root, *need),
        Some(_) | None => None,
    };
    for need in &drawn {
        if findings::why(root, *need) != shared {
            shared = None;
        }
    }
    for (index, need) in drawn.iter().enumerate() {
        match need {
            findings::Need::Existing(need) => {
                existing(b, root, &[*need], across, w, bindings, times, shared.is_none(), index == 0);
            }
            findings::Need::Finding(card) => {
                super::findings::preview(b, root, card, w, bindings, times, across, shared.is_none());
            }
        }
    }
    more(b, root, open, across, w, bindings, times);
    if let Some(why) = shared
        && let Some(line) = crate::shared::why_words(root, w, &why, true)
        && let Some(first) = drawn.get(0)
    {
        match first {
            findings::Need::Existing(need) => {
                super::needs::reason_line(b, root, w, bindings, *need, super::needs::WhyForm::Short, &line);
            }
            findings::Need::Finding(_) => ask::text(b, Role::Paragraph, &line, look::WHY),
        }
    }
}
fn more(
    b: &mut Builder,
    root: &Domain,
    open: &[findings::Need<'_>],
    across: bool,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    let more = open.len().saturating_sub(3);
    if more == 0 {
        return;
    }
    let label = if more == 1 {
        w.say(Word::HomeCardsOneMore, &[])
    } else if let Some(number) = super::home::number_word(u32::try_from(more).expect("bounded needs")) {
        w.say(Word::HomeCardsNumberMore, &[Hole::Name(w.say(number, &[]))])
    } else {
        w.say(Word::HomeCardsMore, &[Hole::Number(u64::try_from(more).expect("bounded needs"))])
    };
    b.open(Role::Paragraph);
    b.style(look::HOME_MORE);
    ask::link(
        b,
        bindings,
        &label,
        Address::Inbox {
            filter: Filter { project: if across { None } else { root.pages().project() }, ..Filter::ALL },
        },
    );
    if more == 1
        && let Some(need) = open.get(3)
    {
        b.text(Run::Plain, if w.shell == Shell::Terminal { b": " } else { b" " });
        match need {
            findings::Need::Finding(card) => {
                super::findings::severity(b, w, card.row.severity().clone());
                b.text(Run::Plain, &super::findings::short(card, w, times, true));
            }
            findings::Need::Existing(need) => {
                if let Some(group) = root.pages().needs().findings().group(findings::Need::Existing(*need)) {
                    super::findings::group_short(b, &group, w, times);
                    b.close();
                    return;
                }
                let (_, short, _) = super::home::need_words(need, w);
                b.text(Run::Plain, &short);
                b.text(Run::Plain, &w.say(Word::ToolSeparator, &[]));
                let when = match need {
                    needs::Need::Card(card) => ask::need_when(card, w, times),
                    needs::Need::Approval(row) => super::approval::when(row, w, times),
                };
                b.text(Run::Plain, &when);
            }
        }
    }
    b.close();
}
