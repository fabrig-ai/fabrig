//! A project's first page (temper/docs/design/client/pages/README.md).
//! A stopped try (temper/docs/design/domain/trial.md, 6.3).
use crate::{
    frame::Bindings,
    look,
    words::{Shell, TimeForm, Times},
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, Shortcut, State, phrases::Hole};
use temper_client_domain::{
    Address, ChatsFilter, Domain, GoalsFilter,
    forge::{
        self,
        forges::{Connection, Kind, Mode, Start},
    },
    pages::{
        Act,
        home::{AddField, AddRefused, AddSheet, CopyLine, CopyState, Facts, Needs, Repository, Status},
        needs::{self, ANSWERED_ON_HOME},
        parts,
    },
};

/// A project's first page (temper/docs/design/client/pages/README.md).
#[expect(
    clippy::too_many_arguments,
    reason = "Home draws page controls and a trial retry with their respective bindings"
)]
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    accounts: &mut crate::accounts::Bindings,
    project: u32,
    times: &mut Times,
    rows: u32,
    _sign_in: &crate::forge_sign_in::Forges,
) {
    let me = root.pages().me(root.client());
    let name = crate::pages::project_name(me, project);
    if me.is_some() && name.is_none() {
        missing(b, root, w, bindings, project);
        return;
    }
    let fallback = w.say(Word::Project, &[Hole::Number(u64::from(project))]);
    let name = name.as_deref().unwrap_or(&fallback);
    let facts = temper_client_domain::pages::home::facts(root, project, times.now);
    let parts = parts::home(&facts.parts_facts());
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::HOME);
    intro(b, root, w, bindings, accounts, name, &facts, times);
    if facts.to_begin() {
        to_begin(b, root, w, bindings, project, name, &facts);
    }
    if parts.decided
        && let Some(shown) = &facts.decided
    {
        super::decided::main(b, root, w, bindings, Some(project), shown, times);
    }
    if !facts.to_begin() {
        if parts.needs {
            needs(b, root, w, bindings, project, &facts, times);
        }
        if root.has_goals(project) {
            goals(b, root, w, bindings, project, times);
        } else {
            chats(b, root, w, bindings, project, times);
        }
    }
    foot(b, w, bindings, project, &facts, times);
    if parts.ask {
        super::home_cards::ask_line(b, root, w, bindings);
    }
    crate::confirm::main(b, root, w, bindings, rows);
    add_sheet(b, root, w, bindings, name);
    b.close();
}

pub(crate) struct Intro {
    kicker: Box<[u8]>,
    pub(crate) headline: Box<[u8]>,
    lede: Option<Box<[u8]>>,
    byline: Option<Box<[u8]>>,
}
pub(crate) fn kicker(w: &Words, previous: Option<u64>, times: &mut Times) -> Box<[u8]> {
    // The kicker is this Home's day and time, not a project's creation time.
    let day = times.say(w, TimeForm::Weekday, times.now);
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
    if let Some(previous) = previous
        && times.now.as_nanos().saturating_sub(previous) > minute.as_nanos()
    {
        let span = times.say(w, TimeForm::Looked, skein_lib::Wall::from_nanos(previous));
        kicker = w.say(Word::HomeLooked, &[Hole::Name(kicker), Hole::Name(span)]);
    }
    kicker
}
pub(crate) fn needs_sentence(w: &Words, needs: Needs, unknown: &[u8]) -> Box<[u8]> {
    match needs {
        Needs::Unknown => unknown.into(),
        Needs::Count { count, cut: true } => w.say(Word::HomeNeedsCut, &[Hole::Number(u64::from(count))]),
        Needs::Count { count: 0, cut: false } => w.say(Word::HomeNeedsNone, &[]),
        Needs::Count { count: 1, cut: false } => w.say(Word::HomeNeedsOne, &[]),
        Needs::Count { count, cut: false } => {
            let number = match number_word(count) {
                Some(word) => w.say(word, &[]),
                None => skein_lib::Decimal::of(u64::from(count)).as_bytes().into(),
            };
            w.say(Word::HomeNeedsMany, &[Hole::Name(number)])
        }
    }
}
pub(crate) fn intro_words(w: &Words, name: &[u8], facts: &Facts, times: &mut Times) -> Intro {
    let previous = match &facts.status {
        Status::Shown(status) => *status.previous(),
        Status::Unknown => None,
    };
    let mut kicker = kicker(w, previous, times);
    let needs = if facts.to_begin() {
        w.say(Word::HomeProjectReady, &[Hole::Name(name.into())])
    } else {
        needs_sentence(w, facts.needs, name)
    };
    let minute = skein_lib::Duration::from_secs(60);
    let landed = if facts.to_begin() { 0 } else { facts.landed_since() };
    let brief = if facts.to_begin() { None } else { facts.brief() };
    let sentence = match brief {
        Some(brief) => brief.sentence().into(),
        None => match landed {
            0 => match if facts.to_begin() { None } else { facts.last_landed() } {
                Some(last) => {
                    if times.now.as_nanos().saturating_sub(last.as_nanos()) < minute.as_nanos() {
                        w.say(Word::HomeLandedNow, &[])
                    } else {
                        let span = times.say(w, TimeForm::Looked, last);
                        w.say(Word::HomeLandedLast, &[Hole::Name(span)])
                    }
                }
                None => Box::default(),
            },
            1 => w.say(Word::HomeLandedOne, &[]),
            2..=u64::MAX => w.say(Word::HomeLandedMany, &[Hole::Number(landed)]),
        },
    };
    let (lede, byline) = match brief {
        Some(brief) => {
            let lede = if brief.lede().is_empty() { None } else { Some(brief.lede().into()) };
            let at = times.say(w, TimeForm::Clock, skein_lib::Wall::from_nanos(brief.at()));
            (lede, Some(w.say(Word::HomeByline, &[Hole::Name(at)])))
        }
        None => (None, None),
    };
    if let Some(spell) = &facts.spell {
        if let Some(end) = spell.end {
            let expires = skein_lib::Wall::from_nanos(end.as_nanos().saturating_add(86_400_000_000_000));
            times.next = [times.next, Some(expires)].into_iter().flatten().min();
        }
        let since = times.say(w, TimeForm::Clock, spell.since);
        let end = match spell.end {
            Some(end) => times.say(w, TimeForm::Clock, end),
            None => w.say(Word::HomeAwayNow, &[]),
        };
        kicker = w.say(Word::HomeAwaySpan, &[Hole::Name(since), Hole::Name(end)]);
        let decided = match &facts.decided {
            Some(shown) if shown.showing == 1 => w.say(Word::HomeDecidedOne, &[]),
            Some(shown) => {
                let number = match number_word(shown.showing) {
                    Some(word) => w.say(word, &[]),
                    None => skein_lib::Decimal::of(u64::from(shown.showing)).as_bytes().into(),
                };
                w.say(Word::HomeDecidedMany, &[Hole::Name(number)])
            }
            None => needs,
        };
        let landed = match spell.landed {
            None | Some(0) => Box::default(),
            Some(1) => w.say(Word::HomeAwayLandedOne, &[]),
            Some(count) => w.say(Word::HomeAwayLandedMany, &[Hole::Number(count)]),
        };
        let headline =
            if landed.is_empty() { decided } else { crate::accounts::words::join(&[&landed, b" ", &decided]) };
        return Intro { kicker, headline, lede, byline };
    }
    let headline = if sentence.is_empty() { needs } else { crate::accounts::words::join(&[&sentence, b" ", &needs]) };
    Intro { kicker, headline, lede, byline }
}
#[expect(
    clippy::too_many_arguments,
    reason = "the intro keeps page links and the trial retry in their respective bindings"
)]
fn intro(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    accounts: &mut crate::accounts::Bindings,
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
    if w.shell == Shell::Terminal && !facts.to_begin() {
        // The shell uses the heading as its title; the intro also reads whole in its body.
        text(b, Role::Heading { level: 1 }, &intro.headline, None);
        b.open(Role::Paragraph);
        text(b, Role::Text, &intro.kicker, Some(look::HOME_KICKER));
        b.text(Run::Plain, b" ");
        b.text(Run::Plain, &intro.headline);
        if let Some(lede) = &intro.lede {
            b.text(Run::Plain, b" ");
            text(b, Role::Text, lede, Some(look::HOME_LEDE));
        }
        b.close();
        if let Some(byline) = &intro.byline {
            text(b, Role::Paragraph, byline, Some(look::HOME_BYLINE));
        }
        stopped_try(b, root, w, accounts);
        b.close();
        return;
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
    stopped_try(b, root, w, accounts);
    b.close();
}
fn stopped_try(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut crate::accounts::Bindings) {
    if let Some((who, account)) = super::model_check::stopped(root, w) {
        text(b, Role::Paragraph, &w.say(Word::HomeTryStopped, &[Hole::Name(who)]), Some(look::HOME_BYLINE));
        if let Some(to) = bindings.insert(crate::accounts::Meaning::Try { account }) {
            crate::flow::control(b, crate::flow::Control::Button { name: &w.say(Word::WelcomeTryAgain, &[]), to });
        }
    }
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
pub(crate) fn number_word(count: u32) -> Option<Word> {
    match count {
        2 => Some(Word::HomeTwo),
        3 => Some(Word::HomeThree),
        4 => Some(Word::HomeFour),
        5 => Some(Word::HomeFive),
        6 => Some(Word::HomeSix),
        7 => Some(Word::HomeSeven),
        8 => Some(Word::HomeEight),
        9 => Some(Word::HomeNine),
        0 | 1 | 10.. => None,
    }
}
// The first need has a card, followed by at most five other needs and five chats (3.5).
const ROWS: usize = 5;
fn inbox_address(project: u32) -> Address {
    Address::Inbox { filter: temper_client_domain::address::Filter { project: Some(project), kind: None, goal: None } }
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
    let Some(inbox) = root.pages().inbox() else { return };
    let session = root.pages().needs();
    let open = session.all(inbox, root.forge().approvals());
    let (count, cut) = match facts.needs {
        Needs::Count { count, cut } => (count, cut),
        Needs::Unknown => (0, false),
    };
    b.open(Role::Region);
    b.style(look::HOME_NEEDS);
    b.name(&w.say(Word::NeedsYou, &[]));
    b.open(Role::Slot);
    b.style(look::HOME_HEAD);
    let heading = match (facts.spell.is_some(), cut) {
        (true, false) => Word::HomeStillNeedsCount,
        (true, true) => Word::HomeStillNeedsCountMore,
        (false, false) => Word::HomeNeedsCount,
        (false, true) => Word::NeedsHeadCut,
    };
    text(b, Role::Heading { level: 2 }, &w.say(heading, &[Hole::Number(u64::from(count))]), Some(look::HOME_SIGNAL));
    go(b, bindings, &w.say(Word::AllHomeAllNeeds, &[]), inbox_address(project), None);
    b.close();
    let lines = session.answered(times.now, ANSWERED_ON_HOME, Some(project));
    if let Some(line) = lines.first() {
        times.next = [
            times.next,
            Some(skein_lib::Wall::from_nanos(line.at.as_nanos().saturating_add(ANSWERED_ON_HOME.as_nanos()))),
        ]
        .into_iter()
        .flatten()
        .min();
    }
    super::ask::answered_lines(b, root, &lines, w, bindings);
    super::home_cards::all(b, root, &open, false, w, bindings, times);
    b.close();
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
fn goals(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32, times: &mut Times) {
    let Some(goals) = root.pages().goals() else { return };
    b.open(Role::Region);
    b.style(look::GOALS_SECTION);
    b.name(&w.say(Word::Goals, &[]));
    b.open(Role::Slot);
    b.style(look::HOME_HEAD);
    text(b, Role::Heading { level: 2 }, &w.say(Word::Goals, &[]), None);
    let label = w.say(Word::HomeAllChats, &[Hole::Number(u64::from(goals.count(root.client())))]);
    go(b, bindings, &label, Address::Goals { project, filter: GoalsFilter::All }, None);
    b.close();
    let live = match goals.live(root.client()) {
        Some(live) => live.rows.as_slice(),
        None => &[],
    };
    b.open(Role::Slot);
    b.style(look::GOALS_ROWS);
    for line in live.iter().chain(goals.ended()).take(ROWS.max(live.len())) {
        let report = goals.report(line.chip.task);
        let state = temper_client_domain::pages::goals::state(
            &line.chip,
            goals.needs_person(line.chip.task),
            report.is_some(),
            line.delegates_total,
        );
        super::goals::row(b, w, bindings, times, line, state, report);
    }
    b.close();
    b.close();
}

fn chats(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32, times: &mut Times) {
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
        crate::link(b, &name, bindings.insert(Act::Go { address: Address::Chats { filter: ChatsFilter::All } }));
        b.text(Run::Plain, &label);
        b.close();
    }
    b.close();
    if count == 0 {
        if let Some(chats) = chats
            && chats.known(root.client())
        {
            text(b, Role::Paragraph, &w.say(Word::NoChats, &[]), Some(look::HOME_EMPTY));
        }
    } else {
        b.open(Role::List);
        b.style(look::HOME_ROWS);
        for line in live_rows.iter().chain(ended.iter()).take(ROWS) {
            b.open(Role::Item);
            let status = match root.pages().me(root.client()) {
                Some(me) => crate::shared::chat_state(line, me, w),
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
fn to_begin(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
    name: &[u8],
    facts: &Facts,
) {
    let heading = w.say(Word::HomeBegin, &[]);
    b.open(Role::Region);
    b.style(look::HOME_BEGIN);
    b.name(&heading);
    begin_head(b, w, bindings);
    let note = if facts.left() == 2 {
        Word::HomeNoteTwo
    } else if facts.configured {
        Word::HomeNoteSettings
    } else {
        Word::HomeNoteAsk
    };
    text(b, Role::Paragraph, &w.say(note, &[]), Some(look::HOME_NOTE));
    if facts.skip_refused {
        text(b, Role::Alert, &w.say(Word::HomeSkipRefused, &[]), None);
    }
    b.open(Role::List);
    b.style(look::HOME_STEPS);
    repository_step(b, w, bindings, project, name, &facts.repository);
    step(
        b,
        w,
        2,
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
            Act::Go { address: crate::frame::ask_address(root, project) },
            look::HOME_PRIMARY,
            false,
        );
        b.close();
    }
    b.close();
    b.close();
    b.close();
}
#[expect(clippy::too_many_lines, reason = "the repository step draws each state in its precedence order")]
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
        Repository::Copying { lines } => {
            let mut title = Word::HomeCopyAct;
            for line in lines {
                if line.state == CopyState::Making {
                    title = Word::HomeAddTitle;
                }
            }
            step(b, w, 1, false, &w.say(title, &[]), &[]);
            for line in lines {
                copy_line(b, w, bindings, name, line);
            }
        }
        Repository::CopiesIn { names } => {
            let (title, detail) = if names.len() == 1 {
                (Word::HomeCopyIn, Word::HomeCopyInDetail)
            } else {
                (Word::HomeCopiesIn, Word::HomeCopiesInDetail)
            };
            step(
                b,
                w,
                1,
                true,
                &w.say(title, &[Hole::Name(join_copies(names)), Hole::Name(name.into())]),
                &w.say(detail, &[]),
            );
        }
        Repository::CopyOffer { addresses, .. } => {
            let Some(first) = addresses.first() else { return };
            step(b, w, 1, false, &w.say(Word::HomeCopyOffer, &[Hole::Name(first.clone())]), &[]);
            for address in addresses.iter().skip(1) {
                text(b, Role::Paragraph, address, None);
            }
            acts(b);
            button(b, bindings, &w.say(Word::HomeCopyAct, &[]), Act::CopyOffered, look::HOME_PRIMARY, false);
            b.close();
        }
        Repository::In { owner, name: repo } => {
            step(
                b,
                w,
                1,
                true,
                &w.say(Word::HomeIn, &[Hole::Name(owner.clone()), Hole::Name(repo.clone()), Hole::Name(name.into())]),
                &w.say(Word::HomeInDetail, &[]),
            );
        }
        Repository::Adopting(first) => {
            step(
                b,
                w,
                1,
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
                1,
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
            step(b, w, 1, false, &adoption, &detail);
            acts(b);
            go(b, bindings, &w.say(Word::HomeAdoptLink, &[]), Address::Adopt { project }, Some(look::HOME_PRIMARY));
            b.close();
        }
        Repository::Add { .. } => add_step(b, w, bindings, project),
        Repository::None => {
            step(b, w, 1, false, &adoption, &w.say(Word::HomeNone, &[]));
            acts(b);
            go(b, bindings, &w.say(Word::HomeAdoptLink, &[]), Address::Adopt { project }, Some(look::HOME_PRIMARY));
            b.close();
        }
    }
    b.close();
}
fn add_step(b: &mut Builder, w: &Words, bindings: &mut Bindings, project: u32) {
    step(b, w, 1, false, &w.say(Word::HomeAddTitle, &[]), &w.say(Word::HomeAddDetail, &[]));
    acts(b);
    for (start, word) in [(Start::Empty, Word::HomeAddEmpty), (Start::Copies, Word::HomeAddCopy)] {
        button(b, bindings, &w.say(word, &[]), Act::AddOpen { start }, look::HOME_SMALL, false);
    }
    go(b, bindings, &w.say(Word::HomeAddOther, &[]), Address::Adopt { project }, Some(look::HOME_QUIET));
    b.close();
}
fn join_copies(names: &[Box<[u8]>]) -> Box<[u8]> {
    let mut joined: Box<[u8]> = Box::default();
    for name in names {
        let separator = if joined.is_empty() { b"".as_slice() } else { b", ".as_slice() };
        joined = skein_lib::bytes::concat(&[&joined, separator, name]);
    }
    joined
}
fn copy_line(b: &mut Builder, w: &Words, bindings: &mut Bindings, project: &[u8], line: &CopyLine) {
    b.open(Role::Slot);
    b.style(look::HOME_COPY);
    let label = match &line.state {
        CopyState::Running { host } => w.say(
            Word::HomeCopying,
            &[Hole::Name(line.org.clone()), Hole::Name(line.name.clone()), Hole::Name(host.clone())],
        ),
        CopyState::Making => w.say(Word::HomeMaking, &[Hole::Name(line.org.clone()), Hole::Name(line.name.clone())]),
        CopyState::Waiting => w.say(Word::HomeCopyWaiting, &[]),
        CopyState::Adopting => w.say(
            Word::HomeAdopting,
            &[Hole::Name(line.org.clone()), Hole::Name(line.name.clone()), Hole::Name(project.into())],
        ),
        CopyState::In => {
            b.style(look::HOME_DONE);
            let name = skein_lib::bytes::concat(&[&line.org, b"/", &line.name]);
            let label = w.say(Word::HomeCopyIn, &[Hole::Name(name), Hole::Name(project.into())]);
            match w.shell {
                Shell::Browser => label,
                Shell::Terminal => crate::accounts::words::join(&[&w.say(Word::HomeDone, &[]), &label]),
            }
        }
        CopyState::Failed { words } => words.clone(),
    };
    text(b, Role::Paragraph, &label, None);
    match &line.state {
        CopyState::Failed { .. } => {
            acts(b);
            button(
                b,
                bindings,
                &w.say(Word::HomeCopyAgain, &[]),
                Act::CopyAgain { org: line.org.clone(), name: line.name.clone() },
                look::HOME_QUIET,
                false,
            );
            b.close();
        }
        CopyState::Running { .. } | CopyState::Making | CopyState::Waiting | CopyState::Adopting | CopyState::In => {}
    }
    b.close();
}

fn add_sheet(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: &[u8]) {
    let Some(home) = root.pages().home() else { return };
    let Some(sheet) = &home.add else { return };
    let title = w.say(Word::HomeAddSheet, &[Hole::Name(project.into())]);
    b.open(Role::Dialog);
    b.style(look::WELCOME_SHEET);
    b.name(&title);
    b.open(Role::Slot);
    b.style(look::WELCOME_SHEET_HEAD);
    text(b, Role::Heading { level: 2 }, &title, None);
    text(b, Role::Paragraph, &w.say(Word::HomeAddSheetLead, &[]), None);
    b.close();
    let legend = w.say(Word::HomeAddLegend, &[]);
    b.open(Role::ChoiceGroup);
    b.style(look::WELCOME_START);
    b.name(&legend);
    text(b, Role::Heading { level: 3 }, &legend, Some(look::WELCOME_LEGEND));
    for (start, title, detail) in [
        (Start::Empty, Word::WelcomeForgejoEmpty, Word::WelcomeForgejoEmptyDetail),
        (Start::Copies, Word::WelcomeForgejoCopies, Word::WelcomeForgejoCopiesDetail),
    ] {
        b.open(Role::Slot);
        b.style(look::WELCOME_OPTION);
        b.open(Role::Choice);
        b.name(&w.say(title, &[]));
        if sheet.start == start {
            b.state(State::Selected);
        }
        if sheet.out.is_some() {
            b.state(State::Disabled);
        }
        b.bind(bindings.insert(Act::AddChoose { start }));
        b.close();
        text(b, Role::Paragraph, &w.say(detail, &[]), None);
        b.close();
        if sheet.start == start {
            add_fields(b, root, w, bindings, sheet);
        }
    }
    b.close();
    if let Some(refused) = &sheet.refused {
        add_alert(b, w, refused);
    }
    b.open(Role::Slot);
    b.style(look::WELCOME_SHEET_FOOT);
    b.open(Role::Button);
    b.name(&w.say(Word::Cancel, &[]));
    b.bind(bindings.insert(Act::AddClose));
    b.shortcut(Shortcut::ESCAPE);
    if sheet.out.is_some() {
        b.state(State::Disabled);
    }
    b.close();
    button(b, bindings, &w.say(Word::HomeAddIt, &[]), Act::AddSubmit, look::FLOW_PRIMARY, sheet.out.is_some());
    b.close();
    b.close();
}
fn add_fields(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, sheet: &AddSheet) {
    b.open(Role::Slot);
    b.style(look::WELCOME_OPTION_MORE);
    match sheet.start {
        Start::Empty => {
            for (field, word, value) in [
                (AddField::Org, Word::SetupOrganization, &sheet.org),
                (AddField::Name, Word::WelcomeForgejoName, &sheet.name),
            ] {
                b.open(Role::Slot);
                b.open(Role::TextField);
                b.state(State::SubmitsOnEnter);
                b.name(&w.say(word, &[]));
                b.value(value, temper_client_domain::pages::shown(root.pages()));
                b.bind(bindings.insert(Act::AddWrite { field, words: Box::default() }));
                if sheet.out.is_some() {
                    b.state(State::Disabled);
                }
                if let Some(AddRefused::Blank | AddRefused::Invalid(_)) = sheet.refused {
                    b.state(State::Invalid);
                }
                b.close();
                b.close();
            }
        }
        Start::Copies => {
            let addresses = w.say(Word::WelcomeForgejoAddresses, &[]);
            text(b, Role::Paragraph, &addresses, None);
            b.open(Role::TextArea);
            b.name(&addresses);
            b.placeholder(&w.say(Word::WelcomeForgejoAddressHint, &[]));
            b.value(&sheet.addresses, temper_client_domain::pages::shown(root.pages()));
            b.bind(bindings.insert(Act::AddWrite { field: AddField::Addresses, words: Box::default() }));
            if sheet.out.is_some() {
                b.state(State::Disabled);
            }
            if let Some(AddRefused::Copies(_)) = sheet.refused {
                b.state(State::Invalid);
            }
            b.close();
        }
        Start::Nothing => {}
    }
    b.close();
}
fn add_alert(b: &mut Builder, w: &Words, refused: &AddRefused) {
    use temper_client_protocol_wire::v1::MakeRefusal;
    let why: &[u8] = match refused {
        AddRefused::Blank => {
            text(b, Role::Alert, &w.say(Word::WelcomeSheetBlank, &[]), None);
            return;
        }
        AddRefused::Copies(refused) => {
            crate::forge::connect::copies_alert(b, w, refused);
            return;
        }
        AddRefused::Invalid(_) => b"invalid",
        AddRefused::Make(why) => match why {
            MakeRefusal::NoSuchForge => b"no such forge",
            MakeRefusal::NotOffered => b"not offered",
            MakeRefusal::NotOwner => b"not owner",
            MakeRefusal::Invalid => b"invalid",
            MakeRefusal::HomeHeld => b"The project has a home already.",
            MakeRefusal::Full => b"full",
            MakeRefusal::NotFailed => b"It is not failed.",
        },
        AddRefused::Unknown => b"no answer; try again",
    };
    text(b, Role::Alert, &w.say(Word::HomeAddRefused, &[Hole::Name(why.into())]), None);
}

fn missing(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::HOME);
    let me = root.pages().me(root.client());
    let name = crate::pages::project_name(me, project);
    let fallback = w.say(Word::Project, &[Hole::Number(u64::from(project))]);
    let missing = me.is_some() && name.is_none();
    let name = name.as_deref().unwrap_or(&fallback);
    b.open(Role::Heading { level: 1 });
    b.name(name);
    b.text(Run::Plain, name);
    b.close();
    if missing {
        b.open(Role::Paragraph);
        w.text(b, Word::ProjectNotYours, &[Hole::Number(u64::from(project))]);
        b.close();
        let chats = w.say(Word::Chats, &[]);
        crate::link(b, &chats, bindings.insert(Act::Go { address: Address::Chats { filter: ChatsFilter::All } }));
        b.text(Run::Plain, &chats);
        b.close();
    }
    b.close();
}

/// Home's need words (temper/docs/design/client/pages/README.md, Home, Needs you).
pub(crate) fn need_words(need: &needs::Need<'_>, w: &Words) -> (Box<[u8]>, Box<[u8]>, crate::shared::NeedKind) {
    match need {
        needs::Need::Card(card) => {
            let asked = super::ask::read(card, w);
            let title = if card.page.is_none() { super::ask::need_title(card, w) } else { asked.title };
            let kind = match asked.kind {
                super::ask::Kind::Question => crate::shared::NeedKind::Question,
                super::ask::Kind::Proposal
                | super::ask::Kind::Held
                | super::ask::Kind::Funds { .. }
                | super::ask::Kind::PersonTask => crate::shared::NeedKind::Other,
            };
            (title, asked.short, kind)
        }
        needs::Need::Approval(row) => {
            (row.title.clone(), super::approval::question(row, w), crate::shared::NeedKind::Other)
        }
    }
}
pub(crate) fn need_sentence(title: &[u8], short: &[u8], w: &Words) -> Box<[u8]> {
    w.say(Word::HomeAskLine, &[Hole::Name(title.into()), Hole::Name(short.into())])
}
