//! Home's first three needs (temper/docs/design/client/README.md, 3.5, Home, The cards).
use super::ask;
use crate::{
    frame::Bindings,
    look,
    words::{Shell, Times, Word, Words},
};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    address::Filter,
    pages::{Act, needs},
};

pub(crate) fn cards(
    b: &mut Builder,
    root: &Domain,
    needs: &[needs::Need<'_>],
    across: bool,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    for need in needs.iter().take(3) {
        b.open(Role::Slot);
        b.style(look::NEED_CARD);
        from(b, root, *need, across, w, bindings, times);
        let (_, mut question, _) = super::home::need_words(need, w);
        if let needs::Need::Card(card) = need
            && let ask::Kind::Funds { spent, limit } = ask::read(card, w).kind
        {
            question = w.say(Word::NeedsFundsUsed, &[Hole::Number(spent), Hole::Number(limit)]);
        }
        ask::text(b, Role::Paragraph, &question, look::ASK_QUESTION);
        b.open(Role::Slot);
        b.style(look::NEED_ACTS);
        match need {
            needs::Need::Card(card) => {
                let mut asked = ask::read(card, w);
                if let Some(parsed) = &mut asked.parsed
                    && let Some(primary) = parsed.primary()
                {
                    parsed.choices.get_mut(..=primary).expect("primary is a choice").rotate_right(1);
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
                    if let ask::Kind::Question = asked.kind {
                        tell(
                            b,
                            need,
                            w,
                            bindings,
                            match &asked.parsed {
                                Some(parsed) => parsed.choices.is_empty(),
                                None => true,
                            },
                        );
                    }
                }
            }
            needs::Need::Approval(row) => {
                if row.yours == temper_client_domain::forge::approvals::Yours::Counts
                    && skein_ui::markdown::safe_url(&row.address)
                {
                    let forge = w.say(
                        match row.kind {
                            temper_client_domain::forge::forges::Kind::GitHub => Word::ApprovalGitHub,
                            temper_client_domain::forge::forges::Kind::Forgejo => Word::ApprovalForgejo,
                        },
                        &[],
                    );
                    let label = w.say(Word::ApprovalReview, &[Hole::Name(forge)]);
                    b.open(Role::Link);
                    b.style(look::RECOMMENDED);
                    b.name(&label);
                    b.external(&row.address);
                    b.text(Run::Plain, &label);
                    b.close();
                }
                tell(b, need, w, bindings, false);
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
    let more = needs.len().saturating_sub(3);
    if more > 0 {
        let label = if more == 1 {
            w.say(Word::HomeCardsOneMore, &[])
        } else {
            w.say(Word::HomeCardsMore, &[Hole::Number(u64::try_from(more).expect("bounded needs"))])
        };
        let project = if across { None } else { root.pages().project() };
        ask::link(b, bindings, &label, Address::Inbox { filter: Filter { project, ..Filter::ALL } });
    }
}
fn tell(b: &mut Builder, need: &needs::Need<'_>, w: &Words, bindings: &mut Bindings, primary: bool) {
    ask::button(
        b,
        bindings,
        &w.say(Word::HomeCardTell, &[]),
        Act::Needs(needs::Act::Open { object: need.object() }),
        if primary { look::RECOMMENDED } else { look::ASK_QUIET },
    );
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
    if let Some(task) = source {
        ask::link(b, bindings, &title, Address::Task { task });
    } else {
        b.text(Run::Plain, &title);
    }
    b.text(Run::Plain, &w.say(Word::ToolSeparator, &[]));
    b.text(Run::Plain, &when);
    if root.pages().needs().later(need.object()) {
        b.text(Run::Plain, &w.say(Word::NeedsLaterMeta, &[]));
    }
    b.close();
}
