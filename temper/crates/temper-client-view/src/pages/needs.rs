//! One question at a time in the browser (temper/docs/design/client/README.md, 3.5).
use super::ask;
use crate::{
    frame::Bindings,
    look,
    words::{Times, Word, Words},
};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act,
        needs::{self, ANSWERED_ON_NEEDS},
    },
};

pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    rows: u32,
    times: &mut Times,
    sign_in: &crate::forge_sign_in::Forges,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::NEEDS);
    let Some(inbox) = root.pages().inbox() else {
        b.close();
        return;
    };
    let mut have_page = inbox.known();
    for card in &inbox.cards {
        have_page |= card.page.is_some();
    }
    if !have_page {
        b.close();
        return;
    }
    let session = root.pages().needs();
    let lines = session.answered(times.now, ANSWERED_ON_NEEDS, inbox.filter.project);
    if let Some(line) = lines.first() {
        times.next = [
            times.next,
            Some(skein_lib::Wall::from_nanos(line.at.as_nanos().saturating_add(ANSWERED_ON_NEEDS.as_nanos()))),
        ]
        .into_iter()
        .flatten()
        .min();
    }
    if let Some(need) = session.shown(inbox, root.forge().approvals()) {
        flow(b, root, w, bindings, times, need, &lines, rows, sign_in);
    } else {
        b.open(Role::Slot);
        b.style(look::NEEDS_INTRO);
        ask::text(b, Role::Paragraph, &w.say(Word::NeedsYou, &[]), look::HOME_KICKER);
        ask::text(b, Role::Heading { level: 1 }, &w.say(Word::InboxNeedsEmpty, &[]), look::FLOW_Q);
        ask::text(b, Role::Paragraph, &w.say(Word::NeedsDone, &[]), look::HOME_LEDE);
        b.close();
        ask::answered_lines(b, root, &lines, w, bindings);
        if let Some(shown) = root.pages().decided(inbox.filter.project, times.now) {
            super::decided::main(b, root, w, bindings, inbox.filter.project, &shown, times);
        }
        let project = home_project(root, inbox.filter.project);
        if let Some(project) = project {
            b.open(Role::Paragraph);
            crate::link(
                b,
                &w.say(Word::RailHome, &[]),
                bindings.insert(Act::Go { address: Address::Home { project } }),
            );
            b.style(look::NEEDS_HOME);
            b.text(Run::Plain, &w.say(Word::RailHome, &[]));
            b.close();
            b.close();
        }
    }
    crate::confirm::main(b, root, w, bindings, rows);
    b.close();
}

fn home_project(root: &Domain, filter: Option<u32>) -> Option<u32> {
    if let Some(project) = filter.or(root.pages().project()) {
        return Some(project);
    }
    Some(root.pages().me(root.client())?.projects.get(0)?.project)
}

#[expect(clippy::too_many_arguments, reason = "the flow passes the view's row bound to its card facts")]
fn flow(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    need: needs::Need<'_>,
    lines: &[&needs::Answered],
    rows: u32,
    sign_in: &crate::forge_sign_in::Forges,
) {
    let inbox = root.pages().inbox().expect("flow inbox");
    let session = root.pages().needs();
    let open = session.needs(inbox, root.forge().approvals());
    let total = lines.len().checked_add(open.len()).expect("bounded session and inbox");
    let at = lines.len().checked_add(1).expect("bounded session");
    let head = w.say(
        if *inbox.cut.first().expect("needs cut") || root.forge().approvals().more() != 0 {
            Word::NeedsProgressCut
        } else {
            Word::NeedsProgress
        },
        &[
            Hole::Number(u64::try_from(at).expect("bounded count")),
            Hole::Number(u64::try_from(total).expect("bounded count")),
        ],
    );
    b.open(Role::Slot);
    b.style(look::FLOW_HEAD);
    b.open(Role::Figure);
    b.style(look::DOTS);
    b.name(&head);
    for i in 0..total {
        b.open(Role::Text);
        b.style(look::DOT);
        if i < lines.len() {
            b.style(look::DOT_DONE);
        }
        b.close();
    }
    b.close();
    ask::text(b, Role::Paragraph, &head, look::HOME_KICKER);
    b.close();
    match need {
        needs::Need::Card(card) => card_main(b, root, w, bindings, times, card, rows),
        needs::Need::Approval(row) => super::approval::card(b, root, row, w, bindings, times, sign_in, true),
    }
    let mut next = None;
    for other in &open {
        if other.object() != need.object() {
            next = Some(match other {
                needs::Need::Card(card) => {
                    let asked = ask::read(card, w);
                    (asked.title, asked.short)
                }
                needs::Need::Approval(row) => (row.title.clone(), super::approval::question(row, w)),
            });
            break;
        }
    }
    let next = match next {
        Some(next) => w.say(Word::NeedsNext, &[Hole::Name(next.0), Hole::Name(next.1)]),
        None => w.say(Word::NeedsLast, &[]),
    };
    ask::text(b, Role::Paragraph, &next, look::NEXT_UP);
    if !lines.is_empty() {
        b.open(Role::Slot);
        b.style(look::ANSWERED_SECTION);
        ask::text(b, Role::Heading { level: 2 }, &w.say(Word::NeedsAnswered, &[]), look::HOME_HEAD);
        ask::answered_lines(b, root, lines, w, bindings);
        b.close();
    }
    if let Some(shown) = root.pages().decided(inbox.filter.project, times.now) {
        super::decided::main(b, root, w, bindings, inbox.filter.project, &shown, times);
    }
}

fn card_main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    card: &temper_client_domain::pages::inbox::Card,
    rows: u32,
) {
    let inbox = root.pages().inbox().expect("card inbox");
    let session = root.pages().needs();
    let asked = ask::read(card, w);
    b.open(Role::Slot);
    b.style(look::ASK);
    let project = if inbox.filter.project.is_none() {
        match card.project {
            Some(project) => super::project_name(root.pages().me(root.client()), project),
            None => None,
        }
    } else {
        None
    };
    ask::meta(b, &asked, project.as_deref(), session.later(card.object), w, bindings, times);
    let question = match asked.kind {
        ask::Kind::Funds { spent, limit } => w.say(Word::NeedsFundsUsed, &[Hole::Number(spent), Hole::Number(limit)]),
        ask::Kind::Question => match &asked.parsed {
            Some(parsed) => parsed.question.clone(),
            None => asked.short.clone(),
        },
        ask::Kind::Proposal | ask::Kind::Held | ask::Kind::PersonTask => asked.short.clone(),
    };
    ask::text(b, Role::Heading { level: 1 }, &question, look::FLOW_Q);
    context(b, root, &asked, w, rows);
    ask::whole_parts(b, &asked, w, bindings);
    if let Some(outcome) = session.refused(card.object) {
        crate::confirm::outcome_words(b, w, outcome);
    }
    if card.key.is_some() || card.outcome.is_some() {
        crate::confirm::card_controls(b, root, card, w, bindings);
        b.close();
        return;
    }
    match asked.kind {
        ask::Kind::Question => choices(b, &asked, w, bindings),
        ask::Kind::Proposal | ask::Kind::Held | ask::Kind::Funds { .. } | ask::Kind::PersonTask => {
            b.open(Role::Slot);
            b.style(look::ASK_CHOICES);
            crate::confirm::needs_controls(b, root, card, w, bindings);
            b.close();
        }
    }
    b.open(Role::Slot);
    b.style(look::PAIR);
    match asked.kind {
        ask::Kind::Question => ask::button(
            b,
            bindings,
            &w.say(Word::NeedsReply, &[]),
            Act::Needs(needs::Act::OpenReply { object: card.object }),
            look::ASK_QUIET,
        ),
        ask::Kind::Proposal | ask::Kind::Held | ask::Kind::Funds { .. } | ask::Kind::PersonTask => {
            if let Some(task) = super::inbox::source_task(card.object) {
                ask::button(
                    b,
                    bindings,
                    &w.say(Word::HomeOpen, &[]),
                    Act::Go { address: Address::Task { task } },
                    look::ASK_QUIET,
                );
            }
        }
    }
    ask::button(
        b,
        bindings,
        &w.say(Word::NeedsLater, &[]),
        Act::Needs(needs::Act::Later { object: card.object }),
        look::ASK_QUIET,
    );
    b.close();
    ask::reply_form(b, root, &asked, w, bindings);
    b.close();
}

fn choices(b: &mut Builder, asked: &ask::Asked<'_>, w: &Words, bindings: &mut Bindings) {
    let card = asked.card;
    if let Some(parsed) = &asked.parsed
        && !parsed.choices.is_empty()
    {
        b.open(Role::Slot);
        b.style(look::ASK_CHOICES);
        for (i, choice) in parsed.choices.iter().enumerate() {
            b.open(Role::Button);
            b.name(&choice.words);
            b.bind(
                bindings.insert(Act::Needs(needs::Act::Choose { object: card.object, words: choice.words.clone() })),
            );
            if parsed.primary() == Some(i) {
                b.style(look::RECOMMENDED);
            }
            if choice.recommended {
                ask::text(b, Role::Text, &w.say(Word::NeedsRecommended, &[]), look::REC);
            }
            b.close();
        }
        b.close();
    }
}

fn context(b: &mut Builder, root: &Domain, asked: &ask::Asked<'_>, w: &Words, rows: u32) {
    let Some(page) = &asked.card.page else { return };
    match asked.kind {
        ask::Kind::Question => return,
        ask::Kind::Proposal | ask::Kind::Held | ask::Kind::Funds { .. } | ask::Kind::PersonTask => {}
    }
    b.open(Role::Slot);
    b.style(look::ASK_CONTEXT);
    if let jig_client::contract::ReadValue::Escalation(held) = page {
        crate::cards::specification(b, &held.task.spec, w);
    }
    if let jig_client::contract::ReadValue::PersonTask(task) = page {
        crate::cards::specification(b, &task.spec, w);
    }
    match asked.kind {
        ask::Kind::Proposal | ask::Kind::Held | ask::Kind::Funds { .. } => {
            crate::confirm::facts_at(b, page, w, rows, crate::cards::Place::Inbox);
            if let jig_client::contract::ObjectKey::Proposal(key) = asked.card.object {
                crate::cards::proposal::owned_approval(b, root, key.number, w);
            }
        }
        ask::Kind::Question | ask::Kind::PersonTask => {}
    }
    b.close();
}
