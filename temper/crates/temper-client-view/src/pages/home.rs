//! A project's first page, and its settings (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    look,
    words::{Shell, TimeForm, Times},
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, ChatsFilter, Domain,
    forge::{
        self,
        forges::{Connection, Kind, Mode},
    },
    pages::{
        Act,
        home::{Facts, Needs, Repository, Status},
    },
};

/// A project's first page (temper/docs/design/client/README.md, 3.5).
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    times: &mut Times,
) {
    let me = root.pages().me(root.client());
    let name = crate::pages::project_name(me, project);
    if me.is_some() && name.is_none() {
        page(b, root, w, bindings, project, false);
        return;
    }
    let fallback = w.say(Word::Project, &[Hole::Number(u64::from(project))]);
    let name = name.as_deref().unwrap_or(&fallback);
    let facts = temper_client_domain::pages::home::facts(root, project);
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::HOME);
    intro(b, root, w, bindings, name, &facts, times);
    if facts.to_begin() {
        to_begin(b, w, bindings, project, name, &facts);
    }
    needs(b, root, w, bindings, project, &facts, times);
    chats(b, root, w, bindings, project, facts.to_begin(), times);
    foot(b, w, bindings, project, &facts, times);
    b.close();
}

struct Intro {
    kicker: Box<[u8]>,
    headline: Box<[u8]>,
    lede: Option<Box<[u8]>>,
    byline: Option<Box<[u8]>>,
}
fn intro_words(w: &Words, name: &[u8], facts: &Facts, times: &mut Times) -> Intro {
    // The kicker is this Home's day and time, not a project's creation time.
    let day = times.say(w, TimeForm::Day, times.now);
    let clock = times.say(w, TimeForm::Clock, times.now);
    let mut kicker = w.say(Word::HomeDayTime, &[Hole::Name(day), Hole::Name(clock)]);
    let minute = skein_lib::Duration::from_secs(60);
    let tick = times.now.as_nanos().checked_div(minute.as_nanos()).expect("minute is nonzero");
    let next = match tick.checked_add(1) {
        Some(tick) => match tick.checked_mul(minute.as_nanos()) {
            Some(nanos) => Some(skein_lib::Wall::from_nanos(nanos)),
            None => None,
        },
        None => None,
    };
    times.next = [times.next, next].into_iter().flatten().min();
    if let Status::Shown(status) = &facts.status
        && let Some(previous) = status.previous()
        && times.now.as_nanos().saturating_sub(*previous) > minute.as_nanos()
    {
        let span = times.say(w, TimeForm::Looked, skein_lib::Wall::from_nanos(*previous));
        kicker = w.say(Word::HomeLooked, &[Hole::Name(kicker), Hole::Name(span)]);
    }
    let needs = if facts.to_begin() {
        w.say(Word::HomeProjectReady, &[Hole::Name(name.into())])
    } else {
        match facts.needs {
            Needs::Unknown => name.into(),
            Needs::Count { count, cut: true } => w.say(Word::HomeNeedsCut, &[Hole::Number(u64::from(count))]),
            Needs::Count { count: 0, cut: false } => w.say(Word::HomeNeedsNone, &[]),
            Needs::Count { count: 1, cut: false } => w.say(Word::HomeNeedsOne, &[]),
            Needs::Count { count, cut: false } => w.say(Word::HomeNeedsMany, &[Hole::Number(u64::from(count))]),
        }
    };
    let landed = if facts.to_begin() { 0 } else { facts.landed_since() };
    let sentence = match landed {
        0 => Box::default(),
        1 => w.say(Word::HomeLandedOne, &[]),
        2..=u64::MAX => w.say(Word::HomeLandedMany, &[Hole::Number(landed)]),
    };
    let headline = if sentence.is_empty() { needs } else { crate::accounts::words::join(&[&sentence, b" ", &needs]) };
    Intro { kicker, headline, lede: None, byline: None }
}
fn intro(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    name: &[u8],
    facts: &Facts,
    times: &mut Times,
) {
    b.open(Role::Slot);
    b.style(look::HOME_INTRO);
    let mut intro = intro_words(w, name, facts, times);
    if facts.to_begin() {
        intro.lede = begin_lede(root, w);
    }
    text(b, Role::Paragraph, &intro.kicker, Some(look::HOME_KICKER));
    text(b, Role::Heading { level: 1 }, &intro.headline, None);
    if facts.to_begin() {
        b.open(Role::Slot);
        b.style(look::HOME_LEDES);
    }
    if let Some(lede) = intro.lede {
        text(b, Role::Paragraph, &lede, Some(look::HOME_LEDE));
    }
    if facts.to_begin() {
        let (_, more) = super::accounts::home_accounts(root);
        go(b, bindings, &w.say(more, &[]), Address::Accounts, None);
        b.close();
    }
    if let Some(byline) = intro.byline {
        text(b, Role::Paragraph, &byline, Some(look::HOME_BYLINE));
    }
    b.close();
}
fn begin_lede(root: &Domain, w: &Words) -> Option<Box<[u8]>> {
    let (key, _) = super::accounts::home_accounts(root);
    let mut lede = Box::default();
    if let Some(account) = super::accounts::first_usable_kind(root, w) {
        lede = if key { w.say(Word::HomeAgentsKey, &[]) } else { w.say(Word::HomeAgentsOn, &[Hole::Name(account)]) };
    }
    for row in &forge::listed::rows(root.forges(), forge::connect::playground(root.forge())) {
        if row.mode == Mode::Fake {
            continue;
        }
        match row.connection {
            Connection::Connected { .. } => {
                let sentence = if row.kind == Kind::Forgejo && row.mode == Mode::Owned {
                    w.say(if lede.is_empty() { Word::HomeForgeHere } else { Word::HomeAndForgeHere }, &[])
                } else {
                    w.say(
                        if lede.is_empty() { Word::HomeForge } else { Word::HomeAndForge },
                        &[Hole::Name(crate::forge::connect::name(w, row, root))],
                    )
                };
                lede = crate::accounts::words::join(&[&lede, &sentence]);
                break;
            }
            Connection::NotConnected
            | Connection::NotSetUp
            | Connection::SettingUp { .. }
            | Connection::SetUpFailed { .. }
            | Connection::Connecting
            | Connection::Stopped { .. } => {}
        }
    }
    if lede.is_empty() { None } else { Some(crate::accounts::words::join(&[&lede, b"."])) }
}
// The first need has a card, followed by at most five other needs and five chats (3.5).
const ROWS: usize = 5;
fn inbox_address(project: u32) -> Address {
    Address::Inbox { filter: temper_client_domain::address::Filter { project: Some(project), kind: None, goal: None } }
}
fn need_title(card: &temper_client_domain::pages::inbox::Card, w: &Words) -> Box<[u8]> {
    if let Some(page) = &card.page
        && let Some(title) = super::inbox::task_title(page)
        && !title.is_empty()
    {
        return title.into();
    }
    match super::inbox::source_task(card.object) {
        Some(task) => w.say(Word::TaskWho, &[Hole::Number(task)]),
        None => w.say(need_kind(card.group), &[]),
    }
}
fn need_kind(group: jig_client::contract::InboxGroup) -> Word {
    use jig_client::contract::InboxGroup;
    match group {
        InboxGroup::Proposals => Word::HomeProposal,
        InboxGroup::Questions => Word::HomeQuestion,
        InboxGroup::Escalations => Word::HomeHeld,
        InboxGroup::PersonTasks => Word::HomePersonTask,
        InboxGroup::Results => Word::InboxUpdates,
        InboxGroup::Replies => Word::Reply,
    }
}
fn need_when(card: &temper_client_domain::pages::inbox::Card, w: &Words, times: &mut Times) -> Box<[u8]> {
    let kind = w.say(need_kind(card.group), &[]);
    let since = match &card.page {
        Some(page) => super::inbox::waiting_since(page),
        None => None,
    };
    match since {
        Some(since) => crate::accounts::words::join(&[
            &kind,
            &w.say(Word::ToolSeparator, &[]),
            &times.say(w, TimeForm::Span, since),
        ]),
        None => kind,
    }
}
fn needs(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    facts: &Facts,
    times: &mut Times,
) {
    let inbox = root.pages().inbox();
    let count = match inbox {
        Some(inbox) => inbox.needs().count(),
        None => 0,
    };
    if count == 0 && facts.to_begin() {
        return;
    }
    b.open(Role::Region);
    b.style(look::HOME_NEEDS);
    b.name(&w.say(Word::NeedsYou, &[]));
    b.open(Role::Slot);
    b.style(look::HOME_HEAD);
    let heading = if count > 0 {
        w.say(Word::HomeNeedsCount, &[Hole::Number(u64::try_from(count).expect("needs count fits"))])
    } else {
        w.say(Word::NeedsYou, &[])
    };
    text(b, Role::Heading { level: 2 }, &heading, if count > 0 { Some(look::HOME_SIGNAL) } else { None });
    if count > 1 && w.shell == Shell::Browser {
        go(b, bindings, &w.say(Word::HomeInARow, &[]), inbox_address(project), Some(look::HOME_IN_A_ROW));
    }
    b.close();
    let first = match inbox {
        Some(inbox) => inbox.needs().next(),
        None => None,
    };
    if let Some(first) = first {
        ask(b, first, w, bindings, project, times);
        if let Some(inbox) = inbox {
            let mut others = inbox.needs().skip(1).take(ROWS).peekable();
            if others.peek().is_some() {
                b.open(Role::List);
                b.style(look::HOME_ROWS);
                for card in others {
                    b.open(Role::Item);
                    let title = need_title(card, w);
                    let when = need_when(card, w, times);
                    let label = crate::accounts::words::join(&[&title, &w.say(Word::ToolSeparator, &[]), &when]);
                    crate::link(b, &label, bindings.insert(Act::Go { address: inbox_address(project) }));
                    row_words(b, w, &title, &when);
                    b.close();
                    b.close();
                }
                b.close();
            }
        }
    } else {
        text(b, Role::Paragraph, &w.say(Word::HomeNeedsEmpty, &[]), Some(look::HOME_EMPTY));
    }
    b.close();
}
fn ask(
    b: &mut Builder,
    card: &temper_client_domain::pages::inbox::Card,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    times: &mut Times,
) {
    let title = need_title(card, w);
    let when = need_when(card, w, times);
    let suffix = crate::accounts::words::join(&[&w.say(Word::ToolSeparator, &[]), &when]);
    b.open(Role::Slot);
    b.style(look::ASK);
    b.open(Role::Paragraph);
    b.style(look::ASK_META);
    b.name(&crate::accounts::words::join(&[&title, &suffix]));
    if let Some(task) = super::inbox::source_task(card.object) {
        go(b, bindings, &title, Address::Task { task, filter: temper_client_domain::PlanFilter::All }, None);
    } else {
        b.text(Run::Plain, &title);
    }
    b.text(Run::Plain, &suffix);
    b.close();
    let Some(page) = &card.page else {
        b.close();
        return;
    };
    let words = super::inbox::card_words(page);
    text(b, Role::Paragraph, &words, Some(look::ASK_QUESTION));
    b.open(Role::Slot);
    b.style(look::ASK_CHOICES);
    let reply = if card.group == jig_client::contract::InboxGroup::Questions { Word::Reply } else { Word::HomeOpen };
    button(b, bindings, &w.say(reply, &[]), Act::Go { address: inbox_address(project) }, look::ASK_QUIET, false);
    b.close();
    b.close();
    if w.shell == Shell::Browser {
        let line = w.say(Word::HomeAskLine, &[Hole::Name(title), Hole::Name(words)]);
        let name = w.say(Word::HomeAnswerName, &[Hole::Name(line.clone())]);
        crate::link(b, &name, bindings.insert(Act::Go { address: inbox_address(project) }));
        b.style(look::HOME_CTA);
        b.text(Run::Plain, &line);
        b.open(Role::Text);
        b.style(look::HOME_CTA_GO);
        b.run(Run::Plain, &w.say(Word::HomeAnswer, &[]));
        b.close();
        b.close();
    }
}
fn row_words(b: &mut Builder, w: &Words, title: &[u8], when: &[u8]) {
    b.text(Run::Plain, title);
    b.open(Role::Text);
    b.style(look::HOME_WHEN);
    if w.shell == Shell::Terminal {
        b.run(Run::Plain, &w.say(Word::ToolSeparator, &[]));
    }
    b.run(Run::Plain, when);
    b.close();
}
fn chats(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    beginning: bool,
    times: &mut Times,
) {
    use jig_client::contract::EndedGroup;
    let chats = root.pages().chats();
    let (live, page) = match chats {
        Some(chats) => (chats.live(root.client()), chats.page.as_ref()),
        None => (None, None),
    };
    let (live_rows, live_cut) = match live {
        Some(live) => (live.rows.as_slice(), live.more),
        None => (&[][..], false),
    };
    let (ended, cut) = match page {
        Some(page) => match &page.ended {
            EndedGroup::Rows { rows, after, .. } => (rows.as_slice(), after.is_some()),
            EndedGroup::None => (&[][..], false),
        },
        None => (&[][..], false),
    };
    let count = live_rows.len().checked_add(ended.len()).expect("two bounded windows fit");
    if count == 0 && beginning {
        return;
    }
    b.open(Role::Region);
    b.style(look::HOME_CHATS);
    b.name(&w.say(Word::Chats, &[]));
    b.open(Role::Slot);
    b.style(look::HOME_HEAD);
    text(b, Role::Heading { level: 2 }, &w.say(Word::Chats, &[]), None);
    if count > 0 {
        let word = if cut || live_cut { Word::HomeAllChatsCut } else { Word::HomeAllChats };
        let label = w.say(word, &[Hole::Number(u64::try_from(count).expect("chat count fits"))]);
        let name = w.say(Word::HomeAllChatsName, &[Hole::Name(label.clone())]);
        crate::link(
            b,
            &name,
            bindings.insert(Act::Go { address: Address::Chats { filter: ChatsFilter::All, start: false } }),
        );
        b.text(Run::Plain, &label);
        b.close();
    }
    b.close();
    if count == 0 {
        text(b, Role::Paragraph, &w.say(Word::NoChats, &[]), Some(look::HOME_EMPTY));
    } else {
        b.open(Role::List);
        b.style(look::HOME_ROWS);
        for line in live_rows.iter().chain(ended.iter()).take(ROWS) {
            b.open(Role::Item);
            let status = match root.pages().me(root.client()) {
                Some(me) => super::chats::status_name(line, me, w),
                None => Box::default(),
            };
            let age = times.say(w, TimeForm::Span, line.ended_at.unwrap_or(line.last_activity));
            let title = line.chip.title.bytes();
            let name = w.say(
                Word::HomeChatName,
                &[Hole::Name(title.into()), Hole::Name(status.clone()), Hole::Name(age.clone())],
            );
            crate::link(b, &name, bindings.insert(Act::Go { address: Address::Chat { task: line.chip.task } }));
            let when = crate::accounts::words::join(&[&status, &w.say(Word::ToolSeparator, &[]), &age]);
            row_words(b, w, title, &when);
            b.close();
            b.close();
        }
        b.close();
    }
    if live.is_some() && page.is_some() && !root.has_goals(project) {
        text(b, Role::Paragraph, &w.say(Word::HomeChatsSentence, &[]), Some(look::HOME_FACTS));
    }
    b.close();
}

fn foot(b: &mut Builder, w: &Words, bindings: &mut Bindings, project: u32, facts: &Facts, times: &mut Times) {
    if facts.to_begin() {
        return;
    }
    let Status::Shown(status) = &facts.status else { return };
    let branch = status.branch();
    let branch = if branch.is_empty() { None } else { Some(branch) };
    let red = [branch.unwrap_or(b"")];
    let landing = super::changes::LandingFacts {
        queued: u64::from(status.queued()),
        last: match status.last_landed() {
            Some(at) => Some(skein_lib::Wall::from_nanos(*at)),
            None => None,
        },
        branch,
        red: if status.red() { &red } else { &[] },
    };
    b.open(Role::Paragraph);
    b.style(look::HOME_FACTS);
    b.text(Run::Plain, &w.say(Word::HomeLanding, &[]));
    super::changes::landing_parts(b, &super::changes::landing_line(&landing, w, times), w);
    b.text(Run::Plain, &w.say(Word::ToolSeparator, &[]));
    go(b, bindings, &w.say(Word::Changes, &[]), Address::Changes { project }, None);
    b.close();
}

fn text(b: &mut Builder, role: Role, words: &[u8], style: Option<skein_ui::Style>) {
    b.open(role);
    if let Some(style) = style {
        b.style(style);
    }
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
fn step(b: &mut Builder, w: &Words, number: u64, done: bool, heading: &[u8], detail: &[u8]) {
    b.open(Role::Item);
    b.style(look::HOME_STEP);
    if done {
        b.style(look::HOME_DONE);
    }
    let heading = match w.shell {
        Shell::Browser => heading.into(),
        Shell::Terminal => {
            let done = if done { w.say(Word::HomeDone, &[]) } else { Box::default() };
            crate::accounts::words::join(&[&w.say(Word::HomeNumber, &[Hole::Number(number)]), &done, heading])
        }
    };
    text(b, Role::Heading { level: 3 }, &heading, None);
    if !detail.is_empty() {
        text(b, Role::Paragraph, detail, None);
    }
}
fn acts(b: &mut Builder) {
    b.open(Role::Slot);
    b.style(look::HOME_ACTS);
}
fn go(b: &mut Builder, bindings: &mut Bindings, label: &[u8], address: Address, style: Option<skein_ui::Style>) {
    crate::link(b, label, bindings.insert(Act::Go { address }));
    if let Some(style) = style {
        b.style(style);
    }
    b.text(Run::Plain, label);
    b.close();
}
fn button(b: &mut Builder, bindings: &mut Bindings, label: &[u8], act: Act, style: skein_ui::Style, disabled: bool) {
    b.open(Role::Button);
    b.style(style);
    b.name(label);
    if disabled {
        b.state(State::Disabled);
    }
    b.bind(bindings.insert(act));
    b.close();
}
fn optional(b: &mut Builder, w: &Words) {
    let optional = w.say(Word::HomeOptional, &[]);
    let optional = match w.shell {
        Shell::Browser => optional,
        Shell::Terminal => crate::accounts::words::join(&[&optional, b" "]),
    };
    b.open(Role::Text);
    b.style(look::HOME_TAG);
    b.run(Run::Plain, &optional);
    b.close();
}
fn begin_head(b: &mut Builder, w: &Words, bindings: &mut Bindings) {
    b.open(Role::Slot);
    b.style(look::HOME_HEAD);
    text(b, Role::Heading { level: 2 }, &w.say(Word::HomeBegin, &[]), None);
    if w.shell == Shell::Browser {
        optional(b, w);
    }
    let skip = w.say(Word::HomeSkip, &[]);
    b.open(Role::Button);
    b.style(look::HOME_SKIP);
    b.name(&skip);
    b.bind(bindings.insert(Act::SkipToBegin));
    if w.shell == Shell::Terminal {
        optional(b, w);
        b.text(Run::Plain, &skip);
    }
    b.close();
    b.close();
}
fn to_begin(b: &mut Builder, w: &Words, bindings: &mut Bindings, project: u32, name: &[u8], facts: &Facts) {
    let heading = w.say(Word::HomeBegin, &[]);
    b.open(Role::Region);
    b.style(look::HOME_BEGIN);
    b.name(&heading);
    begin_head(b, w, bindings);
    let note = match facts.left() {
        0 | 1 => {
            if facts.chat_asked {
                Word::HomeNoteSettings
            } else {
                Word::HomeNoteChats
            }
        }
        2 => {
            if facts.chat_asked {
                Word::HomeNoteTwoSettings
            } else {
                Word::HomeNoteTwo
            }
        }
        3..=u8::MAX => Word::HomeNoteThree,
    };
    text(b, Role::Paragraph, &w.say(note, &[]), Some(look::HOME_NOTE));
    if facts.skip_refused {
        text(b, Role::Alert, &w.say(Word::HomeSkipRefused, &[]), None);
    }
    b.open(Role::List);
    b.style(look::HOME_STEPS);
    step(
        b,
        w,
        1,
        facts.configured,
        &w.say(Word::HomeHow, &[]),
        &if facts.configured { Box::default() } else { w.say(Word::HomeHowDetail, &[]) },
    );
    if !facts.configured {
        acts(b);
        button(
            b,
            bindings,
            &w.say(Word::HomeStart, &[]),
            Act::Go { address: Address::Configure { project } },
            look::HOME_PRIMARY,
            false,
        );
        b.close();
    }
    b.close();
    repository_step(b, w, bindings, project, name, &facts.repository);
    step(
        b,
        w,
        3,
        facts.chat_asked,
        &w.say(Word::HomeChat, &[]),
        &if facts.chat_asked { Box::default() } else { w.say(Word::HomeChatDetail, &[]) },
    );
    if !facts.chat_asked {
        acts(b);
        go(b, bindings, &w.say(Word::HomeNewChat, &[]), Address::Chats { filter: ChatsFilter::All, start: true }, None);
        b.close();
    }
    b.close();
    b.close();
    b.close();
}
fn repository_step(
    b: &mut Builder,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    name: &[u8],
    repository: &Repository,
) {
    let adoption = w.say(Word::Adoption, &[]);
    match repository {
        Repository::Unknown => return,
        Repository::In { owner, name: repo } => {
            step(
                b,
                w,
                2,
                true,
                &w.say(Word::HomeIn, &[Hole::Name(owner.clone()), Hole::Name(repo.clone()), Hole::Name(name.into())]),
                &w.say(Word::HomeInDetail, &[]),
            );
        }
        Repository::Adopting(first) => {
            step(
                b,
                w,
                2,
                false,
                &w.say(
                    Word::HomeAdopting,
                    &[Hole::Name(first.org.clone()), Hole::Name(first.name.clone()), Hole::Name(name.into())],
                ),
                &w.say(Word::HomeAdoptingDetail, &[]),
            );
        }
        Repository::Offer { first, refused } => {
            step(
                b,
                w,
                2,
                false,
                &adoption,
                &w.say(Word::HomeOffer, &[Hole::Name(first.org.clone()), Hole::Name(first.name.clone())]),
            );
            if let Some(answer) = refused {
                let forge = skein_lib::Decimal::of(u64::from(first.forge));
                crate::forge::page::outcome(
                    b,
                    (&first.org, &first.name, b"temper/", forge.as_bytes()),
                    answer,
                    w,
                    w.name_bytes,
                );
            }
            acts(b);
            button(b, bindings, &w.say(Word::HomeAdoptLink, &[]), Act::AdoptFirst, look::HOME_PRIMARY, false);
            b.close();
        }
        Repository::GitHub { account } => {
            let detail = match account {
                Some(account) => w.say(Word::HomeGitHub, &[Hole::Name(account.clone())]),
                None => w.say(Word::HomeNone, &[]),
            };
            step(b, w, 2, false, &adoption, &detail);
            acts(b);
            go(b, bindings, &w.say(Word::HomeAdoptLink, &[]), Address::Adopt { project }, Some(look::HOME_PRIMARY));
            b.close();
        }
        Repository::None => {
            step(b, w, 2, false, &adoption, &w.say(Word::HomeNone, &[]));
            acts(b);
            go(b, bindings, &w.say(Word::HomeAdoptLink, &[]), Address::Adopt { project }, Some(look::HOME_PRIMARY));
            b.close();
        }
    }
    b.close();
}
/// A project's settings (temper/docs/design/client/README.md, 3.5).
pub(crate) fn settings(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    page(b, root, w, bindings, project, true);
}

fn page(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32, settings: bool) {
    b.open(Role::Main);
    b.style(look::MAIN);
    if settings {
        b.style(look::SETTINGS);
    } else {
        b.style(look::HOME);
    }
    let me = root.pages().me(root.client());
    let name = crate::pages::project_name(me, project);
    let fallback = w.say(Word::Project, &[Hole::Number(u64::from(project))]);
    let missing = me.is_some() && name.is_none();
    let name = name.as_deref().unwrap_or(&fallback);
    let heading = if settings { w.say(Word::SettingsFor, &[Hole::Name(name.into())]) } else { name.into() };
    b.open(Role::Heading { level: 1 });
    b.name(&heading);
    b.text(Run::Plain, &heading);
    b.close();
    if missing {
        b.open(Role::Paragraph);
        w.text(b, Word::ProjectNotYours, &[Hole::Number(u64::from(project))]);
        b.close();
        let chats = w.say(Word::Chats, &[]);
        crate::link(
            b,
            &chats,
            bindings.insert(Act::Go { address: Address::Chats { filter: ChatsFilter::All, start: false } }),
        );
        b.text(Run::Plain, &chats);
        b.close();
    } else if settings {
        settings_controls(b, root, w, bindings, project);
    }
    b.close();
}

fn settings_controls(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    b.open(Role::List);
    b.open(Role::Item);
    let task = match root.pages().chats() {
        Some(chats) => chats.open_configuring(root.client(), root.pages().me(root.client()), project),
        None => None,
    };
    let address = match task {
        Some(task) => Address::Chat { task },
        None => Address::Configure { project },
    };
    go(b, bindings, &w.say(Word::ConfigureTitle, &[]), address, None);
    b.close();
    b.open(Role::Item);
    crate::frame::forges_link(b, root, w, bindings);
    b.close();
    b.open(Role::Item);
    crate::frame::accounts_link(b, root, w, bindings);
    b.close();
    b.open(Role::Item);
    let name = w.say(Word::Adoption, &[]);
    crate::link(b, &name, bindings.insert(Act::Go { address: Address::Adopt { project } }));
    b.text(Run::Plain, &name);
    b.close();
    b.close();
    b.close();
    if root.client().link().sign_out_lost {
        b.open(Role::Alert);
        b.name(&w.say(Word::SignOutLost, &[]));
        w.text(b, Word::SignOutLost, &[]);
        b.close();
    }
    if root.pages().me(root.client()).is_some() || root.client().link().sign_out_lost {
        b.open(Role::Button);
        b.name(&w.say(Word::SignOut, &[]));
        b.bind(bindings.insert(Act::SignOut));
        b.close();
    }
}
