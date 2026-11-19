//! Open needs in order, with one card shown in the browser (temper/docs/design/client/pages/README.md).
#[cfg(test)]
mod limit_tests;
use super::ask;
use crate::{
    bindings::Bindings,
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

pub fn main(
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
        if need.ready(root) || matches_row(need) {
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
            crate::bindings::link(b, &w.say(Word::RailHome, &[]), bindings.insert(Act::Go { address }));
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
    if parts.tell
        && let Some(project) = match inbox {
            Some(inbox) => inbox.filter.project,
            None => None,
        }
    {
        super::tell::region(b, root, w, bindings, project);
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
    let counted = session.counted(inbox, root.forge().approvals());
    let (count, cut) = super::home::needs_count(counted);
    let open_count = match counted {
        temper_client_domain::pages::home::Needs::Count { count, .. } => count,
        temper_client_domain::pages::home::Needs::Unknown => 0,
    };
    let total = lines
        .len()
        .checked_add(usize::try_from(open_count).expect("bounded needs"))
        .expect("bounded session and inbox");
    let mut position = 0_usize;
    for other in &open {
        if other.same(need) {
            break;
        }
        position = position.checked_add(1).expect("bounded open needs");
    }
    let at = lines.len().checked_add(position).expect("bounded session").checked_add(1).expect("bounded session");
    let head = w.say(
        if cut { Word::NeedsProgressCut } else { Word::NeedsProgress },
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
    intro(b, root, w, counted, times.now);
    b.open(Role::Slot);
    b.style(look::NEEDS_HEAD);
    let heading = w.say(if cut { Word::NeedsHeadCut } else { Word::HomeNeedsCount }, &[Hole::Number(u64::from(count))]);
    b.open(Role::Heading { level: 2 });
    b.style(look::HOME_HEAD);
    b.name(&heading);
    b.text(Run::Plain, &skein_lib::bytes::concat(&[&w.say(Word::NeedsYou, &[]), &w.say(Word::ToolSeparator, &[])]));
    b.text(Run::Strong, skein_lib::Decimal::of(u64::from(count)).as_bytes());
    if cut {
        b.text(Run::Strong, b"+");
    }
    b.close();
    ask::text(b, Role::Text, &w.say(Word::NeedsOldest, &[]), look::HOME_KICKER);
    b.close();
    for other in &open {
        if other.same(need) && other.ready(root) {
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
                temper_client_domain::pages::findings::Need::Row(_) => {}
            }
        } else {
            b.open(Role::Button);
            b.style(look::NEED_ROW);
            b.bind(bindings.insert(other.open()));
            from_all(b, root, *other, w, bindings, times);
            let question = match other {
                temper_client_domain::pages::findings::Need::Existing(needs::Need::Card(card)) => {
                    let asked = ask::read(card, w);
                    match asked.kind {
                        ask::Kind::Held => Some(crate::shared::hold_sentence(card.document(), w)),
                        ask::Kind::PersonTask => None,
                        ask::Kind::Question if ask::title_in_from(card) => None,
                        ask::Kind::Question | ask::Kind::Funds { .. } | ask::Kind::Proposal => {
                            Some(crate::shared::need_line(&needs::Need::Card(card), w).words)
                        }
                    }
                }
                temper_client_domain::pages::findings::Need::Existing(need) => {
                    Some(crate::shared::need_line(need, w).words)
                }
                temper_client_domain::pages::findings::Need::Finding(card) => Some(card.row.title.into()),
                temper_client_domain::pages::findings::Need::Row(_) => None,
            };
            if let Some(question) = question {
                ask::text(b, Role::Text, &question, look::ASK_QUESTION);
            }
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

fn row_from(b: &mut Builder, root: &Domain, row: &jig_client::contract::InboxEntry, w: &Words, times: &mut Times) {
    let title = super::inbox::row_title(row, w);
    let when = super::inbox::row_when(row, w, times);
    let later =
        if root.pages().needs().later(row.card_key()) { w.say(Word::NeedsLaterMeta, &[]) } else { Box::default() };
    let suffix = skein_lib::bytes::concat(&[&w.say(Word::ToolSeparator, &[]), &when, &later]);
    b.open(Role::Text);
    b.style(look::ASK_META);
    if root.pages().inbox().expect("row inbox").filter.project.is_none()
        && let Some(project) = row.project
        && let Some(name) = super::project_name(root.pages().me(root.client()), project)
    {
        ask::text(b, Role::Text, &name, look::CHIP);
    }
    b.run(Run::Plain, &title);
    b.run(Run::Plain, &suffix);
    b.close();
}

fn matches_row(need: temper_client_domain::pages::findings::Need<'_>) -> bool {
    match need {
        temper_client_domain::pages::findings::Need::Row(_) => true,
        temper_client_domain::pages::findings::Need::Existing(_)
        | temper_client_domain::pages::findings::Need::Finding(_) => false,
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
    let mut asked = ask::read(card, w);
    ask::primary_first(&mut asked);
    let group =
        session.findings().group(temper_client_domain::pages::findings::Need::Existing(needs::Need::Card(card)));
    b.open(Role::Slot);
    b.style(look::ASK);
    from(b, root, needs::Need::Card(card), w, bindings, times, true);
    if let Some(question) = ask::question(&asked, w) {
        ask::text(b, Role::Heading { level: 1 }, &question, look::FLOW_Q);
    }
    if let Some(group) = &group {
        super::findings::group_rows(b, group, w, bindings, times);
    }
    context(b, root, &asked, w, rows);
    ask::whole_parts(b, &asked, w, bindings);
    why_line(b, root, w, bindings, needs::Need::Card(card), WhyForm::Rule);
    limit_hold(b, root, w, bindings, card);
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

pub(super) fn limit_hold(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    card: &temper_client_domain::pages::inbox::Card,
) {
    use temper_client_domain::pages::limit_settings;
    let state = root.pages().limit_settings();
    let Some(held) = state.hold(card) else { return };
    limit_details(
        b,
        w,
        bindings,
        held,
        limit_settings::owner(root.pages().me(root.client())),
        state.ready() && card.key.is_none(),
        state.refused(held.task),
    );
}
fn limit_details(
    b: &mut Builder,
    w: &Words,
    bindings: &mut Bindings,
    held: &temper_client_protocol_wire::v1::LimitHold,
    owner: bool,
    ready: bool,
    refused: Option<&jig_client::Outcome>,
) {
    use temper_client_domain::pages::limit_settings;
    ask::text(
        b,
        Role::Paragraph,
        &w.say(
            Word::NeedsLimit,
            &[Hole::Name(held.limit.into()), Hole::Number(held.configured), Hole::Number(held.needed)],
        ),
        look::SETTINGS_DETAIL,
    );
    if owner {
        if let Some(figure) = limit_settings::raise_figure(held.configured, held.needed) {
            let label = w.say(Word::NeedsLimitRaise, &[Hole::Name(held.limit.into()), Hole::Number(figure)]);
            b.open(Role::Button);
            b.name(&label);
            b.bind(bindings.insert(Act::LimitSettings(limit_settings::Act::Raise {
                task: held.task,
                revision: held.revision,
            })));
            if !ready {
                b.state(State::Disabled);
            }
            b.close();
        }
    } else {
        ask::text(b, Role::Paragraph, &w.say(Word::NeedsLimitOwner, &[]), look::SETTINGS_DETAIL);
    }
    if let Some(outcome) = refused {
        crate::confirm::outcome_words(b, w, outcome);
    }
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
    if linked
        && let Some(group) =
            root.pages().needs().findings().group(temper_client_domain::pages::findings::Need::Existing(need))
    {
        b.open(Role::Paragraph);
        b.style(look::NEED_FROM);
        super::findings::group_from(
            b,
            root,
            &group,
            w,
            times,
            root.pages().inbox().expect("from inbox").filter.project.is_none(),
        );
        if let needs::Need::Card(card) = need {
            let asked = ask::read(card, w);
            b.text(Run::Plain, &w.say(Word::ToolSeparator, &[]));
            if let Some(task) = asked.source {
                ask::link(b, bindings, &asked.title, Address::Task { task });
            } else {
                b.text(Run::Plain, &asked.title);
            }
        }
        if root.pages().needs().later(need.object()) {
            b.text(Run::Plain, &w.say(Word::NeedsLaterMeta, &[]));
        }
        b.close();
        return;
    }
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
    b.name(&skein_lib::bytes::concat(&[&prefix, &ask::from_name(need, &title), &suffix]));
    if let Some(name) = project {
        ask::text(b, Role::Text, &name, look::CHIP);
    }
    ask::from_title(b, need, &title, source, bindings, linked);
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
    for line in &super::approval::sentences(row, super::approval::viewer(root), w) {
        ask::text(b, Role::Paragraph, line, look::ASK_CONTEXT);
    }
    super::approval::size(b, row, w);
    b.open(Role::Slot);
    b.style(look::ASK_CHOICES);
    if row.yours == temper_client_domain::forge::approvals::Yours::Counts
        && let Some(address) = super::approval::address(row, sign_in)
    {
        let label = super::approval::review_words(row, w);
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

fn intro(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    counted: temper_client_domain::pages::home::Needs,
    now: skein_lib::Wall,
) {
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
    let sentence = super::home::needs_sentence(w, counted, b"");
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
            b.style(if parsed.primary() == Some(i) { look::RECOMMENDED } else { look::ASK_QUIET });
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
        crate::cards::specification_words(b, &held.task.spec, w, ask::title_in_from(asked.card));
        match asked.kind {
            ask::Kind::Funds { .. } => {
                crate::cards::hold(b, held.task.chip.task, &jig_client::contract::HoldCategory::Budget, w);
            }
            ask::Kind::Question | ask::Kind::Proposal | ask::Kind::Held | ask::Kind::PersonTask => {}
        }
    }
    if let jig_client::contract::ReadValue::PersonTask(task) = page {
        if ask::title_in_from(asked.card) {
            skein_ui::markdown::read(crate::shared::after_title(&task.spec.specification.words), 3, b);
            crate::cards::specification_facts(b, &task.spec, w);
        } else {
            crate::cards::specification_after_title(b, &task.spec, w);
        }
    }
    match asked.kind {
        ask::Kind::Proposal => {
            crate::confirm::facts_at(b, page, w, rows, crate::cards::Place::Inbox);
            if let jig_client::contract::ObjectKey::Proposal(key) = asked.card.object {
                crate::cards::proposal::owned_approval(b, root, key.number, w);
            }
        }
        ask::Kind::Question | ask::Kind::PersonTask | ask::Kind::Held | ask::Kind::Funds { .. } => {}
    }
    b.close();
}

/// Needs you names the narrower rule; Home and the terminal inbox keep the short act.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WhyForm {
    Rule,
    Short,
}

pub fn why_line(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    need: needs::Need<'_>,
    form: WhyForm,
) {
    let Some(why) = crate::shared::why(root, w, need) else { return };
    reason_line(b, root, w, bindings, need, form, &why);
}

#[expect(clippy::too_many_lines, reason = "the reason line keeps its two forms and opened offers together")]
pub(crate) fn reason_line(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    need: needs::Need<'_>,
    form: WhyForm,
    why: &[u8],
) {
    let offers = stop_asking::offered(root, need, bindings.model_times.now);
    let open = match root.pages().stop_asking() {
        Some(open) if open.object() == need.object() => Some(open),
        Some(_) | None => None,
    };
    let numbered = w.shell == crate::words::Shell::Terminal && offers.is_some();
    b.open(if numbered { Role::Slot } else { Role::Paragraph });
    b.style(look::WHY);
    if numbered {
        ask::text(b, Role::Paragraph, why, look::WHY);
    } else {
        b.name(why);
        b.text(Run::Plain, why);
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
        Some(
            needs::Why::Direct
            | needs::Why::Proposal
            | needs::Why::Held
            | needs::Why::Funds
            | needs::Why::Finding { .. },
        )
        | None => return,
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
        temper_client_domain::pages::findings::Need::Row(row) => row_from(b, root, row, w, times),
    }
}
