//! Open needs in order, with one card shown in the browser (temper/docs/design/client/pages/README.md).
use super::ask;
use crate::{
    frame::Bindings,
    look,
    words::{Times, Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act,
        needs::{self, ANSWERED_ON_NEEDS},
        parts, stop_asking,
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
    let facts = parts::needs_facts(root, times.now);
    let parts = parts::needs(&facts);
    let inbox = root.pages().inbox();
    let session = root.pages().needs();
    let lines = session.answered(
        times.now,
        ANSWERED_ON_NEEDS,
        match inbox {
            Some(inbox) => inbox.filter.project,
            None => None,
        },
    );
    if let Some(line) = lines.first() {
        times.next = [
            times.next,
            Some(skein_lib::Wall::from_nanos(line.at.as_nanos().saturating_add(ANSWERED_ON_NEEDS.as_nanos()))),
        ]
        .into_iter()
        .flatten()
        .min();
    }
    if parts.list
        && let Some(inbox) = inbox
        && let Some(need) = session.shown_all(inbox, root.forge().approvals())
    {
        if need.ready(root) {
            flow(b, root, w, bindings, times, need, &lines, rows, sign_in);
        }
    } else if parts.done {
        b.open(Role::Slot);
        b.style(look::NEEDS_INTRO);
        ask::text(b, Role::Paragraph, &w.say(Word::NeedsYou, &[]), look::HOME_KICKER);
        ask::text(b, Role::Heading { level: 1 }, &w.say(Word::InboxNeedsEmpty, &[]), look::FLOW_Q);
        ask::text(b, Role::Paragraph, &w.say(Word::NeedsDone, &[]), look::HOME_LEDE);
        b.close();
        ask::answered_lines(b, root, &lines, w, bindings);
        if let Some(address) = home_address(root) {
            b.open(Role::Paragraph);
            crate::link(b, &w.say(Word::RailHome, &[]), bindings.insert(Act::Go { address }));
            b.style(look::NEEDS_HOME);
            b.text(Run::Plain, &w.say(Word::RailHome, &[]));
            b.close();
            b.close();
        }
    }
    if parts.only {
        super::only_you::main(b, root, w, bindings);
    }
    if parts.decided {
        decided(b, root, w, bindings, times);
    }
    crate::confirm::main(b, root, w, bindings, rows);
    b.close();
}

pub(crate) fn decided(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, times: &mut Times) {
    let facts = parts::needs_facts(root, times.now);
    if !parts::needs(&facts).decided {
        return;
    }
    let project = match root.pages().inbox() {
        Some(inbox) => inbox.filter.project,
        None => None,
    };
    if let Some(shown) = root.pages().decided(project, times.now) {
        super::decided::main(b, root, w, bindings, project, &shown, times);
    }
}
fn home_address(root: &Domain) -> Option<Address> {
    let filter = match root.pages().inbox() {
        Some(inbox) => inbox.filter.project,
        None => None,
    };
    let project = match filter.or(root.pages().project()) {
        Some(project) => project,
        None => root.pages().me(root.client())?.projects.get(0)?.project,
    };
    Some(
        if let Some(me) = root.pages().me(root.client())
            && me.projects.len() >= 2
        {
            Address::AllHome
        } else {
            Address::Home { project }
        },
    )
}

#[expect(clippy::too_many_arguments, reason = "the flow passes the view's row bound to its card facts")]
fn flow(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    need: temper_client_domain::pages::findings::Need<'_>,
    lines: &[&needs::Answered],
    rows: u32,
    sign_in: &crate::forge_sign_in::Forges,
) {
    let inbox = root.pages().inbox().expect("flow inbox");
    let session = root.pages().needs();
    let open = session.all(inbox, root.forge().approvals());
    let total = lines.len().checked_add(open.len()).expect("bounded session and inbox");
    let at = lines.len().checked_add(1).expect("bounded session");
    let head = w.say(
        if *inbox.cut.first().expect("needs cut")
            || root.forge().approvals().more() != 0
            || root.pages().needs().findings().cut_for(inbox.filter)
        {
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
    b.style(look::NEEDS_BAR);
    if let Some(address) = home_address(root) {
        ask::link(b, bindings, &w.say(Word::NeedsYou, &[]), address);
    }
    ask::text(b, Role::Text, &head, look::NEEDS_BAR_AT);
    b.close();
    intro(b, root, w, open.len(), times.now);
    b.open(Role::Slot);
    b.style(look::NEEDS_HEAD);
    let cut = *inbox.cut.first().expect("needs cut")
        || root.forge().approvals().more() != 0
        || root.pages().needs().findings().cut_for(inbox.filter);
    let heading = w.say(
        if cut { Word::NeedsHeadCut } else { Word::HomeNeedsCount },
        &[Hole::Number(u64::try_from(open.len()).expect("bounded needs"))],
    );
    b.open(Role::Heading { level: 2 });
    b.style(look::HOME_HEAD);
    b.name(&heading);
    b.text(Run::Plain, &skein_lib::bytes::concat(&[&w.say(Word::NeedsYou, &[]), &w.say(Word::ToolSeparator, &[])]));
    b.text(Run::Strong, skein_lib::Decimal::of(u64::try_from(open.len()).expect("bounded needs")).as_bytes());
    if cut {
        b.text(Run::Strong, b"+");
    }
    b.close();
    ask::text(b, Role::Text, &w.say(Word::NeedsOldest, &[]), look::HOME_KICKER);
    b.close();
    for other in &open {
        if other.same(need) {
            match other {
                temper_client_domain::pages::findings::Need::Existing(needs::Need::Card(card)) => {
                    card_main(b, root, w, bindings, times, card, rows);
                }
                temper_client_domain::pages::findings::Need::Existing(needs::Need::Approval(row)) => {
                    approval(b, root, row, w, bindings, times, sign_in);
                }
                temper_client_domain::pages::findings::Need::Finding(card) => {
                    super::findings::main(b, root, card, w, bindings, times);
                }
            }
        } else {
            b.open(Role::Button);
            b.style(look::NEED_ROW);
            b.bind(bindings.insert(other.open()));
            from_all(b, root, *other, w, bindings, times);
            ask::text(
                b,
                Role::Text,
                &match other {
                    temper_client_domain::pages::findings::Need::Existing(need) => {
                        crate::shared::need_line(need, w).words
                    }
                    temper_client_domain::pages::findings::Need::Finding(card) => card.row.title().into(),
                },
                look::ASK_QUESTION,
            );
            b.close();
        }
    }
    if !lines.is_empty() {
        b.open(Role::Slot);
        b.style(look::ANSWERED_SECTION);
        ask::text(b, Role::Heading { level: 2 }, &w.say(Word::NeedsAnswered, &[]), look::HOME_HEAD);
        ask::answered_lines(b, root, lines, w, bindings);
        b.close();
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
    let session = root.pages().needs();
    let asked = ask::read(card, w);
    b.open(Role::Slot);
    b.style(look::ASK);
    from(b, root, needs::Need::Card(card), w, bindings, times, true);
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
    why_line(b, root, w, bindings, needs::Need::Card(card), WhyForm::Rule);
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
    match asked.kind {
        ask::Kind::Question => tell(b, root, &asked, w, bindings),
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
    b.close();
}

fn tell(b: &mut Builder, root: &Domain, asked: &ask::Asked<'_>, w: &Words, bindings: &mut Bindings) {
    let card = asked.card;
    let words = match &card.words {
        Some(words) => words.bytes(),
        None => b"",
    };
    let over = match root.pages().me(root.client()) {
        Some(me) => {
            temper_client_domain::pages::composer::over(words, root.pages().needs().reply_beyond, me.words_bytes)
        }
        None => 0,
    };
    b.open(Role::Slot);
    b.style(look::NEED_TELL);
    let label = w.say(Word::NeedsTell, &[]);
    b.open(Role::TextArea);
    b.name(&label);
    b.placeholder(&label);
    b.bind(bindings.insert(Act::CardWords { object: card.object, words: Box::default() }));
    b.value(words, temper_client_domain::pages::shown(root.pages()));
    b.close();
    if !words.trim_ascii().is_empty() {
        b.open(Role::Button);
        b.style(look::RECOMMENDED);
        b.name(&w.say(Word::Send, &[]));
        b.bind(bindings.insert(Act::Needs(needs::Act::SendReply { object: card.object })));
        if over != 0 {
            b.state(State::Disabled);
        }
        b.close();
    }
    super::session::over(b, w, over);
    b.close();
}

fn from(
    b: &mut Builder,
    root: &Domain,
    need: needs::Need<'_>,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    linked: bool,
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
    let later =
        if root.pages().needs().later(need.object()) { w.say(Word::NeedsLaterMeta, &[]) } else { Box::default() };
    let suffix = skein_lib::bytes::concat(&[&w.say(Word::ToolSeparator, &[]), &when, &later]);
    let project = if root.pages().inbox().expect("from inbox").filter.project.is_none() {
        match project {
            Some(project) => super::project_name(root.pages().me(root.client()), project),
            None => None,
        }
    } else {
        None
    };
    let prefix = match &project {
        Some(name) => skein_lib::bytes::concat(&[name, &w.say(Word::ToolSeparator, &[])]),
        None => Box::default(),
    };
    b.open(Role::Text);
    b.style(look::ASK_META);
    b.name(&skein_lib::bytes::concat(&[&prefix, &title, &suffix]));
    if let Some(name) = project {
        ask::text(b, Role::Text, &name, look::CHIP);
    }
    if linked && let Some(task) = source {
        ask::link(b, bindings, &title, Address::Task { task });
    } else {
        b.open(Role::Text);
        b.text(Run::Plain, &title);
        b.close();
    }
    b.open(Role::Text);
    b.text(Run::Plain, &suffix);
    b.close();
    b.close();
}

fn approval(
    b: &mut Builder,
    root: &Domain,
    row: &temper_client_domain::forge::approvals::Row,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    sign_in: &crate::forge_sign_in::Forges,
) {
    b.open(Role::Slot);
    b.style(look::ASK);
    from(b, root, needs::Need::Approval(row), w, bindings, times, true);
    ask::text(b, Role::Heading { level: 1 }, &super::approval::question(row, w), look::FLOW_Q);
    for line in &super::approval::sentences(row, w) {
        ask::text(b, Role::Paragraph, line, look::ASK_CONTEXT);
    }
    b.open(Role::Slot);
    b.style(look::ASK_CHOICES);
    if row.yours == temper_client_domain::forge::approvals::Yours::Counts
        && let Some(address) = super::approval::address(row, sign_in)
    {
        let forge = w.say(
            match row.kind {
                temper_client_domain::forge::forges::Kind::Forgejo => Word::ApprovalForgejo,
                temper_client_domain::forge::forges::Kind::GitHub => Word::ApprovalGitHub,
            },
            &[],
        );
        let label = w.say(Word::ApprovalReview, &[Hole::Name(forge)]);
        b.open(Role::Link);
        b.style(look::RECOMMENDED);
        b.name(&label);
        b.external(&address);
        b.text(Run::Plain, &label);
        b.close();
    }
    b.close();
    ask::button(
        b,
        bindings,
        &w.say(Word::HomeOpen, &[]),
        Act::Go { address: Address::Task { task: row.task } },
        look::ASK_QUIET,
    );
    b.close();
}

fn intro(b: &mut Builder, root: &Domain, w: &Words, count: usize, now: skein_lib::Wall) {
    let inbox = root.pages().inbox().expect("intro inbox");
    let kicker = match inbox.filter.project {
        Some(project) => {
            let name = match super::project_name(root.pages().me(root.client()), project) {
                Some(name) => name,
                None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
            };
            w.say(Word::NeedsProject, &[Hole::Name(name)])
        }
        None => w.say(Word::NeedsAcross, &[]),
    };
    let cut = *inbox.cut.first().expect("needs cut")
        || root.forge().approvals().more() != 0
        || root.pages().needs().findings().cut_for(inbox.filter);
    let sentence = super::home::needs_sentence(
        w,
        temper_client_domain::pages::home::Needs::Count { count: u32::try_from(count).expect("bounded inbox"), cut },
        b"",
    );
    let showing = parts::needs_facts(root, now).showing;
    let sentence = if showing > 0 {
        let decided = if showing == 1 {
            w.say(Word::NeedsDecidedOne, &[])
        } else {
            let number = match super::home::number_word(showing) {
                Some(word) => w.say(word, &[]),
                None => skein_lib::Decimal::of(u64::from(showing)).as_bytes().into(),
            };
            w.say(Word::NeedsDecidedMany, &[Hole::Name(number)])
        };
        crate::accounts::words::join(&[&sentence, b" ", &decided])
    } else {
        sentence
    };
    b.open(Role::Slot);
    b.style(look::NEEDS_INTRO);
    ask::text(b, Role::Paragraph, &kicker, look::HOME_KICKER);
    ask::text(b, Role::Heading { level: 1 }, &sentence, look::HOME_HEAD);
    b.close();
}

pub(crate) fn choices(b: &mut Builder, asked: &ask::Asked<'_>, w: &Words, bindings: &mut Bindings) {
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
                if w.shell == crate::words::Shell::Terminal {
                    b.text(Run::Plain, &choice.words);
                    b.text(Run::Plain, &w.say(Word::ToolSeparator, &[]));
                }
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

/// Needs you names the narrower rule; Home and the terminal inbox keep the short act.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhyForm {
    Rule,
    Short,
}

#[expect(clippy::too_many_lines, reason = "the reason line keeps its two forms and opened offers together")]
pub(crate) fn why_line(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    need: needs::Need<'_>,
    form: WhyForm,
) {
    let Some(why) = crate::shared::why(root, w, need) else { return };
    let offers = stop_asking::offered(root, need, bindings.model_times.now);
    let open = match root.pages().stop_asking() {
        Some(open) if open.object() == need.object() => Some(open),
        Some(_) | None => None,
    };
    let numbered = w.shell == crate::words::Shell::Terminal && offers.is_some();
    b.open(if numbered { Role::Slot } else { Role::Paragraph });
    b.style(look::WHY);
    if numbered {
        ask::text(b, Role::Paragraph, &why, look::WHY);
    } else {
        b.name(&why);
        b.text(Run::Plain, &why);
    }
    if let Some(offers) = &offers
        && let Some(first) = offers.first()
    {
        let label = match form {
            WhyForm::Rule => {
                let Some(needs::Why::Reach { project, reach }) = needs::why(root, need) else { return };
                let Some(classes) = root.forge().classes(project) else { return };
                keep_button(
                    b,
                    w,
                    bindings,
                    need.object(),
                    match open {
                        Some(open) => open.busy(),
                        None => false,
                    },
                    false,
                );
                b.text(Run::Plain, &w.say(Word::ToolSeparator, &[]));
                crate::shared::rule(w, classes, &reach, *first)
            }
            WhyForm::Short => w.say(Word::StopAsking, &[]),
        };
        b.open(Role::Button);
        b.name(&label);
        b.bind(bindings.insert(Act::StopAsking(stop_asking::Act::Open { object: need.object() })));
        if let Some(open) = open {
            b.state(State::Expanded);
            if open.busy() {
                b.state(State::Disabled);
            }
        }
        b.close();
    }
    b.close();
    let (Some(open), Some(offers)) = (open, offers) else { return };
    let (project, reach) = match needs::why(root, need) {
        Some(needs::Why::Reach { project, reach }) => (project, reach),
        Some(needs::Why::Direct | needs::Why::Proposal | needs::Why::Held | needs::Why::Funds) | None => return,
    };
    if let Some(classes) = root.forge().classes(project) {
        b.open(Role::Slot);
        b.style(look::WHY_OFFERS);
        for offer in &offers {
            b.open(Role::Slot);
            b.open(Role::Slot);
            ask::text(b, Role::Paragraph, &crate::shared::offer(w, classes, &reach, *offer), look::WHY);
            match offer {
                temper_protocol_reaches::Offer::Words(_) => {
                    b.open(Role::TextField);
                    b.name(&w.say(Word::StopAskingWhich, &[]));
                    b.placeholder(&w.say(Word::StopAskingPlaceholder, &[]));
                    b.value(open.words(), temper_client_domain::pages::shown(root.pages()));
                    b.bind(bindings.insert(Act::StopAsking(stop_asking::Act::Words { words: Box::default() })));
                    if open.busy() {
                        b.state(State::Disabled);
                    }
                    b.close();
                }
                temper_protocol_reaches::Offer::Decide(_) | temper_protocol_reaches::Offer::Split(_) => {}
            }
            b.close();
            let owner = stop_asking::owner(root, project);
            b.open(Role::Button);
            b.name(&w.say(if owner { Word::StopAskingApply } else { Word::StopAskingPropose }, &[]));
            b.bind(bindings.insert(Act::StopAsking(if owner {
                stop_asking::Act::Apply { offer: *offer }
            } else {
                stop_asking::Act::Propose { offer: *offer }
            })));
            let words_ready = match offer {
                temper_protocol_reaches::Offer::Words(_) => {
                    !open.words().is_empty() && open.words().len() <= temper_protocol_reaches::WORDS_BYTES
                }
                temper_protocol_reaches::Offer::Decide(_) | temper_protocol_reaches::Offer::Split(_) => true,
            };
            if open.busy()
                || !words_ready
                || root.forge().settings_newest().is_none()
                || root.forge().classes_busy(project)
            {
                b.state(State::Disabled);
            }
            b.close();
            b.close();
        }
        if form == WhyForm::Short {
            keep_button(b, w, bindings, need.object(), open.busy(), true);
        }
        if let Some(refused) = open.refused() {
            b.open(Role::Alert);
            crate::cards::refusal(b, w, refused);
            b.close();
        }
        b.close();
    }
}

fn keep_button(
    b: &mut Builder,
    w: &Words,
    bindings: &mut Bindings,
    object: jig_client::contract::ObjectKey,
    busy: bool,
    quiet: bool,
) {
    b.open(Role::Button);
    if quiet {
        b.style(look::ASK_QUIET);
    }
    b.name(&w.say(Word::KeepAsking, &[]));
    b.bind(bindings.insert(Act::StopAsking(stop_asking::Act::Keep { object })));
    if busy {
        b.state(State::Disabled);
    }
    b.close();
}

fn from_all(
    b: &mut Builder,
    root: &Domain,
    need: temper_client_domain::pages::findings::Need<'_>,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    match need {
        temper_client_domain::pages::findings::Need::Existing(need) => from(b, root, need, w, bindings, times, false),
        temper_client_domain::pages::findings::Need::Finding(card) => super::findings::from(b, root, card, w, times),
    }
}
