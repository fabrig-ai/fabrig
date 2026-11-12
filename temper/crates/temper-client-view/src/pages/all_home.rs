//! Home on All projects (temper/docs/design/client/pages/home.md, 1, Home on All projects).
use super::{ask, home};
use crate::{
    frame::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    address::Filter,
    pages::{all_home, home::Status, parts},
};

pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, times: &mut Times, rows: u32) {
    let facts = all_home::facts(root, times.now);
    if let Some(inbox) = root.pages().inbox()
        && inbox.filter == Filter::ALL
    {
        page(b, &facts, Some(root), w, bindings, times, rows);
    } else {
        draw(b, &facts, w, bindings, times);
    }
}
pub(crate) fn draw(b: &mut Builder, facts: &all_home::Facts, w: &Words, bindings: &mut Bindings, times: &mut Times) {
    page(b, facts, None, w, bindings, times, 0);
}
fn page(
    b: &mut Builder,
    facts: &all_home::Facts,
    root: Option<&Domain>,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    rows: u32,
) {
    let parts = parts::home(&facts.parts_facts());
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::HOME);
    intro(b, facts, parts.brief, w, times);
    if parts.needs {
        section(b, w, Word::NeedsYou);
        b.style(look::HOME_NEEDS);
        b.open(Role::Slot);
        b.style(look::HOME_HEAD);
        let (count, cut) = match facts.needs {
            temper_client_domain::pages::home::Needs::Count { count, cut } => (count, cut),
            temper_client_domain::pages::home::Needs::Unknown => (0, false),
        };
        ask::text(
            b,
            Role::Heading { level: 2 },
            &w.say(if cut { Word::NeedsHeadCut } else { Word::HomeNeedsCount }, &[Hole::Number(u64::from(count))]),
            look::HOME_SIGNAL,
        );
        ask::link(b, bindings, &w.say(Word::AllHomeAllNeeds, &[]), Address::Inbox { filter: Filter::ALL });
        b.close();
        if let Some(root) = root
            && let Some(inbox) = root.pages().inbox()
        {
            let session = root.pages().needs();
            let answered = session.answered(times.now, temper_client_domain::pages::needs::ANSWERED_ON_HOME, None);
            if let Some(line) = answered.first() {
                let expires = skein_lib::Wall::from_nanos(
                    line.at.as_nanos().saturating_add(temper_client_domain::pages::needs::ANSWERED_ON_HOME.as_nanos()),
                );
                times.next = [times.next, Some(expires)].into_iter().flatten().min();
            }
            ask::answered_lines(b, root, &answered, w, bindings);
            let needs = session.all(inbox, root.forge().approvals());
            super::home_cards::all(b, root, &needs, true, w, bindings, times);
        }
        b.close();
    }
    if parts.decided {
        decided(b, facts, root, w, bindings, times);
    }
    if parts.projects {
        projects(b, facts, w, bindings);
    }
    if parts.ask
        && !facts.projects.is_empty()
        && let Some(root) = root
    {
        super::home_cards::ask_line(b, root, w, bindings);
    }
    if let Some(root) = root {
        crate::confirm::main(b, root, w, bindings, rows);
    }
    b.close();
}
fn intro(b: &mut Builder, facts: &all_home::Facts, brief: bool, w: &Words, times: &mut Times) {
    let intro = all_home::intro(facts);
    let base = home::kicker(w, None, times);
    let previous = match intro.last_looked {
        Some(at) => Some(at.as_nanos()),
        None => None,
    };
    let mut kicker = home::kicker(w, previous, times);
    if let Some(away) = intro.away {
        let since = w.turn_clock(away.since);
        let off = w.turn_clock(away.off);
        kicker = w.say(Word::AllHomeAway, &[Hole::Name(kicker), Hole::Name(since), Hole::Name(off)]);
    }
    let landed = landed(w, &intro.landed, times);
    let needs = home::needs_sentence(w, intro.needs, b"");
    let headline = headline_words(&landed, &needs);
    let brief = if brief { intro.brief.as_ref() } else { None };
    b.open(Role::Slot);
    b.style(look::HOME_INTRO);
    if w.shell == Shell::Terminal {
        ask::text(b, Role::Heading { level: 1 }, &headline, look::HOME_HEAD);
        b.open(Role::Paragraph);
        b.text(Run::Plain, &kicker);
        if !headline.is_empty() {
            b.text(Run::Plain, b" ");
            b.text(Run::Plain, &headline);
        }
        if let Some(brief) = brief {
            b.text(Run::Plain, b" ");
            lede(b, brief, Role::Text);
        }
        b.close();
    } else {
        form_text(b, Role::Paragraph, &kicker, look::HOME_KICKER, Some(look::HOME_WIDE));
        let phone = match intro.last_looked {
            Some(at) if times.now.as_nanos().saturating_sub(at.as_nanos()) > 60_000_000_000 => {
                let span = times.say(w, TimeForm::Span, at);
                w.say(Word::AllHomePhoneLooked, &[Hole::Name(base), Hole::Name(span)])
            }
            Some(_) | None => base,
        };
        form_text(b, Role::Paragraph, &phone, look::HOME_KICKER, Some(look::HOME_PHONE));
        form_text(b, Role::Heading { level: 1 }, &headline, look::HOME_HEAD, Some(look::HOME_WIDE));
        let phone_landed = match &intro.landed {
            all_home::Landed::In { count: 1, .. } | all_home::Landed::Across { count: 1, .. } => {
                w.say(Word::AllHomePhoneOne, &[])
            }
            all_home::Landed::In { count, .. } | all_home::Landed::Across { count, .. } => {
                w.say(Word::AllHomePhoneLanded, &[Hole::Number(*count)])
            }
            all_home::Landed::Last { .. } | all_home::Landed::Nothing => landed,
        };
        let phone_headline = headline_words(&phone_landed, &needs);
        form_text(b, Role::Heading { level: 1 }, &phone_headline, look::HOME_HEAD, Some(look::HOME_PHONE));
        if let Some(brief) = brief {
            lede(b, brief, Role::Paragraph);
        }
    }
    if let Some(brief) = brief
        && let Some(newest) = brief.lines.first()
    {
        let at = times.say(w, TimeForm::Clock, brief.at);
        form_text(
            b,
            Role::Paragraph,
            &w.say(Word::AllHomeByline, &[Hole::Name(newest.name.clone()), Hole::Name(at.clone())]),
            look::HOME_BYLINE,
            if w.shell == Shell::Browser { Some(look::HOME_WIDE) } else { None },
        );
        if w.shell == Shell::Browser {
            form_text(
                b,
                Role::Paragraph,
                &w.say(Word::AllHomePhoneByline, &[Hole::Name(at)]),
                look::HOME_BYLINE,
                Some(look::HOME_PHONE),
            );
        }
    }
    b.close();
}
fn lede(b: &mut Builder, brief: &all_home::Brief, role: Role) {
    b.open(role);
    b.style(look::HOME_LEDE);
    for (i, line) in brief.lines.iter().take(4).enumerate() {
        if i != 0 {
            b.text(Run::Plain, b" ");
        }
        b.text(Run::Strong, &skein_lib::bytes::concat(&[&line.name, b": "]));
        b.text(Run::Plain, &line.sentence);
    }
    b.close();
}

fn headline_words(landed: &[u8], needs: &[u8]) -> Box<[u8]> {
    if landed.is_empty() {
        needs.into()
    } else if needs.is_empty() {
        landed.into()
    } else {
        skein_lib::bytes::concat(&[landed, b" ", needs])
    }
}
fn landed(w: &Words, landed: &all_home::Landed, times: &mut Times) -> Box<[u8]> {
    match landed {
        all_home::Landed::Nothing => Box::default(),
        all_home::Landed::Last { at } => {
            if times.now.as_nanos().saturating_sub(at.as_nanos()) < 60_000_000_000 {
                w.say(Word::HomeLandedNow, &[])
            } else {
                let span = times.say(w, TimeForm::Looked, *at);
                w.say(Word::HomeLandedLast, &[Hole::Name(span)])
            }
        }
        all_home::Landed::In { name, count: 1, .. } => w.say(Word::AllHomeOne, &[Hole::Name(name.clone())]),
        all_home::Landed::In { name, count, .. } => {
            w.say(Word::AllHomeIn, &[Hole::Number(*count), Hole::Name(name.clone())])
        }
        all_home::Landed::Across { count, projects } => {
            let number = match home::number_word(*projects) {
                Some(word) => {
                    let mut number = w.say(word, &[]);
                    number.make_ascii_lowercase();
                    number
                }
                None => skein_lib::Decimal::of(u64::from(*projects)).as_bytes().into(),
            };
            w.say(Word::AllHomeAcross, &[Hole::Number(*count), Hole::Name(number)])
        }
    }
}
fn section(b: &mut Builder, w: &Words, word: Word) {
    b.open(Role::Region);
    b.name(&w.say(word, &[]));
}
fn decided(
    b: &mut Builder,
    facts: &all_home::Facts,
    root: Option<&Domain>,
    w: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    let more = match root {
        Some(root) => match root.pages().decided(None, times.now) {
            Some(shown) => {
                times.next = [times.next, Some(shown.expires)].into_iter().flatten().min();
                shown.more
            }
            None => false,
        },
        None => false,
    };
    let head = w.say(
        if more { Word::DecidedCountMore } else { Word::HomeDecidedCount },
        &[Hole::Number(u64::from(facts.showing))],
    );
    section(b, w, Word::AllHomeDecidedTitle);
    if w.shell == Shell::Browser {
        b.style(look::HOME_WIDE);
    }
    b.open(Role::Slot);
    b.style(look::DECIDED_HEAD);
    ask::text(b, Role::Heading { level: 2 }, &head, look::HOME_HEAD);
    let open = match root {
        Some(root) => root.pages().project(),
        None => None,
    };
    let project = match open {
        Some(project) => Some(project),
        None => match facts.projects.first() {
            Some(project) => Some(project.project),
            None => None,
        },
    };
    if let Some(project) = project {
        ask::link(b, bindings, &w.say(Word::AllHomeReaches, &[]), Address::Settings { project });
    }
    b.close();
    b.open(Role::Slot);
    b.style(look::DECIDED);
    let busy = match root {
        Some(root) => root.pages().overturn_busy(),
        None => false,
    };
    for row in facts.decided.iter().take(7) {
        let mut name = None;
        for project in &facts.projects {
            if project.project == row.project {
                name = Some(project.name.as_ref());
                break;
            }
        }
        super::decided::decision(b, root, w, bindings, &row.line, times, name, busy);
        let expires = skein_lib::Wall::from_nanos(row.line.at.as_nanos().saturating_add(86_400_000_000_000));
        if expires > times.now {
            times.next = [times.next, Some(expires)].into_iter().flatten().min();
        }
    }
    b.close();
    b.close();
    if w.shell == Shell::Browser {
        b.open(Role::Slot);
        b.style(look::HOME_PHONE);
        ask::link(
            b,
            bindings,
            &w.say(Word::AllHomeDecided, &[Hole::Name(head)]),
            Address::Inbox { filter: Filter::ALL },
        );
        b.close();
    }
}
fn projects(b: &mut Builder, facts: &all_home::Facts, w: &Words, bindings: &mut Bindings) {
    section(b, w, Word::AllHomeProjects);
    if w.shell == Shell::Browser {
        b.style(look::HOME_WIDE);
    }
    ask::text(b, Role::Heading { level: 2 }, &w.say(Word::AllHomeProjects, &[]), look::HOME_HEAD);
    b.open(Role::List);
    b.style(look::PROJECTS);
    for row in &facts.projects {
        b.open(Role::Item);
        ask::link(b, bindings, &row.name, Address::Home { project: row.project });
        if let Status::Shown(status) = &row.status {
            if w.shell == Shell::Terminal {
                b.text(Run::Plain, &w.say(Word::ToolSeparator, &[]));
            }
            let branch =
                if status.branch().is_empty() { w.say(Word::ChangesLanding, &[]) } else { status.branch().into() };
            let branch = if status.red() { w.say(Word::HomeBranchRed, &[Hole::Name(branch)]) } else { branch };
            b.open(Role::Text);
            b.style(look::CHIP);
            b.style(if status.red() { look::CHIP_SIGNAL } else { look::CHIP_GOOD });
            b.text(Run::Plain, &branch);
            b.close();
        }
        b.close();
    }
    b.close();
    b.close();
}

fn form_text(b: &mut Builder, role: Role, words: &[u8], style: skein_ui::Style, form: Option<skein_ui::Style>) {
    b.open(role);
    b.style(style);
    if let Some(form) = form {
        b.style(form);
    }
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
