//! Home's launchpad (temper/docs/design/client/tui/pages/launchpad.md, 3–7).
use super::{Binds, foot, front, text};
use crate::{
    Own, Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::{Link, contract as c};
use skein_lib::{
    List, Wall,
    bytes::{concat, copy_of},
};
use skein_ui::{Builder, Role, Run, Style, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{all_home, chats, home, team},
};
use temper_client_view::{
    shared::{self, NeedMark},
    words::{TimeForm, Word as Shared},
};

#[derive(Debug)]
pub(crate) struct Launchpad {
    pub project: Option<Box<[u8]>>,
    pub drafts: bool,
    pub talk: bool,
    pub address: Box<[u8]>,
    pub playground: bool,
    pub lines: List<Line>,
    pub left_off: List<LeftOff>,
    pub selected: Option<u64>,
    pub needs: Box<[NeedRow]>,
    pub brief: List<Line>,
    pub team: List<TeamRow>,
    pub columns: u16,
    pub notes: foot::Notes,
    pub tick: Option<Wall>,
    pub wall: Wall,
}
#[derive(Debug)]
pub(crate) struct LeftOff {
    pub task: u64,
    pub project: Box<[u8]>,
    pub title: Box<[u8]>,
    pub kind: LeftKind,
    pub at: Box<[u8]>,
    pub state: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum LeftKind {
    Chat { number: u32, session: bool },
    Goal,
}
#[derive(Debug)]
pub(crate) struct NeedRow {
    pub mark: NeedMark,
    pub project: Box<[u8]>,
    pub words: Box<[u8]>,
    pub since: Option<Wall>,
}
#[derive(Debug)]
pub(crate) struct TeamRow {
    pub name: Box<[u8]>,
    pub areas: Box<[u8]>,
    pub line: Box<[u8]>,
}
#[derive(Debug)]
pub(crate) struct Line {
    pub role: Role,
    pub words: Box<[u8]>,
    pub style: Option<Style>,
}
fn push(lines: &mut List<Line>, role: Role, words: &[u8], style: Option<Style>) {
    lines.push(Line { role, words: copy_of(words), style }).expect("launchpad lines fit their header or brief bound");
}
fn project(root: &Domain, me: &c::Me) -> Option<u32> {
    match root.pages().address() {
        Address::Home { project } => Some(*project),
        Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Ask { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Task { .. }
        | Address::Chat { .. }
        | Address::Talk { .. }
        | Address::Work { .. }
        | Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Team { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Repositories { .. }
        | Address::Adopt { .. } => {
            let first = me.projects.get(0)?;
            Some(first.project)
        }
    }
}
fn name(me: &c::Me, number: u32) -> Option<Box<[u8]>> {
    for project in &me.projects {
        if project.project == number {
            return Some(copy_of(project.name.bytes()));
        }
    }
    None
}
fn left_off(root: &Domain, me: &c::Me, words: &Words, wall: Wall, tick: &mut Option<Wall>) -> List<LeftOff> {
    let mut lines = List::with_capacity(3);
    let Some(reads) = root.pages().left_off() else { return lines };
    let tasks = reads.chats(root.client());
    let mut goal: Option<&c::TaskLine> = None;
    if let Some(tasks) = reads.goals(root.client()) {
        for row in &tasks.rows {
            if row.requester != c::Requester::Person(me.party.number) || chats::ended(&row.chip) {
                continue;
            }
            if match goal {
                Some(held) => before(row, held),
                None => true,
            } {
                goal = Some(row);
            }
        }
    }
    // Three passes retain the next row in activity order, with only one goal.
    let mut previous: Option<&c::TaskLine> = None;
    let mut number = 0_u32;
    for _place in 0_u32..3 {
        let mut newest = None;
        let mut kind = LeftKind::Goal;
        if let Some(row) = goal
            && match previous {
                Some(previous) => before(previous, row),
                None => true,
            }
        {
            newest = Some(row);
        }
        if let Some(tasks) = tasks {
            for row in &tasks.rows {
                if chats::ended(&row.chip) {
                    continue;
                }
                if let Some(previous) = previous
                    && !before(previous, row)
                {
                    continue;
                }
                if match newest {
                    Some(held) => before(row, held),
                    None => true,
                } {
                    newest = Some(row);
                    kind = LeftKind::Chat {
                        number: number.checked_add(1).expect("three chat numbers"),
                        session: chats::drafting(me, row.project, &row.chip.executor),
                    };
                }
            }
        }
        let Some(row) = newest else { break };
        match kind {
            LeftKind::Chat { number: next, .. } => number = next,
            LeftKind::Goal => {}
        }
        let (at, next) = words.view().time(TimeForm::Dated, row.last_activity, wall);
        *tick = [*tick, next].into_iter().flatten().min();
        lines
            .push(LeftOff {
                task: row.chip.task,
                project: name(me, row.project).unwrap_or_default(),
                title: copy_of(row.chip.title.bytes()),
                kind,
                at,
                state: shared::chat_state(row, me, words.view()),
            })
            .expect("at most three left-off rows");
        previous = Some(row);
    }
    lines
}
fn before(line: &c::TaskLine, other: &c::TaskLine) -> bool {
    line.last_activity > other.last_activity
        || (line.last_activity == other.last_activity && line.chip.task < other.chip.task)
}
fn home(part: &mut Launchpad, root: &Domain, project: u32, me: &c::Me, words: &Words, wall: Wall, shown: bool) {
    if root.pages().address() != &(Address::Home { project }) {
        return;
    }
    let facts = all_home::facts(root, wall);
    if !part.playground && facts.needs != home::Needs::Unknown {
        let (headline, tick) = shared::all_home_headline(words.view(), &facts, wall);
        push(&mut part.lines, Role::Paragraph, &headline, None);
        part.tick = tick;
        if let Some(brief) = all_home::brief(&facts) {
            push(&mut part.lines, Role::Paragraph, &shared::joined_brief(words.view(), &brief), None);
            let (clock, tick) = words.view().time(TimeForm::Clock, brief.at, wall);
            part.tick = [part.tick, tick].into_iter().flatten().min();
            let byline = words.say(Word::Byline, &[Hole::Name(clock)]);
            push(
                &mut part.lines,
                Role::Paragraph,
                &concat(&[&byline, &words.say(Word::BriefMore, &[])]),
                Some(look::BYLINE),
            );
            if shown {
                for line in &brief.lines {
                    for project in &facts.projects {
                        if project.project == line.project
                            && let home::Status::Shown(document) = &project.status
                            && let Some(brief) = document.brief()
                        {
                            push(
                                &mut part.brief,
                                Role::Paragraph,
                                &concat(&[&line.name, b": ", brief.sentence()]),
                                Some(look::NOTE),
                            );
                            push(&mut part.brief, Role::Paragraph, brief.lede(), Some(look::NOTE));
                        }
                    }
                }
                push(&mut part.brief, Role::Paragraph, &byline, Some(look::NOTE));
            }
        }
    }
    part.team = team_rows(root, project, me, words, wall);
    part.left_off = left_off(root, me, words, wall, &mut part.tick);
    let mut needs = List::with_capacity(5);
    for prompt in shared::need_prompts(root, words.view()).into_iter().take(5) {
        let project = match prompt.project {
            Some(number) => name(me, number).unwrap_or_default(),
            None => Box::default(),
        };
        if let Some(since) = prompt.since {
            let (_, tick) = words.view().time(TimeForm::Ago, since, wall);
            part.tick = [part.tick, tick].into_iter().flatten().min();
        }
        needs
            .push(NeedRow { mark: prompt.mark, project, words: prompt.line.words, since: prompt.since })
            .expect("five launchpad needs");
    }
    part.needs = needs.into_boxed();
}
fn held_areas(areas: Option<&team::HeldAreas>, holder: &team::Holder, words: &Words) -> Box<[u8]> {
    let Some(areas) = areas else { return Box::default() };
    let mut held: Box<[u8]> = Box::default();
    for area in &areas.rows {
        if area.holder.as_ref() == Some(holder) {
            held = if held.is_empty() { copy_of(&area.name) } else { concat(&[&held, b", ", &area.name]) };
        }
    }
    if areas.triager.as_ref() == Some(holder) {
        let triage = words.say(Word::Triage, &[]);
        held = if held.is_empty() { triage } else { concat(&[&held, b", ", &triage]) };
    }
    held
}
fn team_rows(root: &Domain, project: u32, me: &c::Me, words: &Words, wall: Wall) -> List<TeamRow> {
    let mut rows = List::with_capacity(4);
    if team::standing_roles(root, project).is_none() {
        return rows;
    }
    let shown = team::shown_at(root, project, wall);
    let Some(facts) = shown.facts() else { return rows };
    let areas = team::areas(root, project);
    let yours = held_areas(areas.as_ref(), &team::Holder::Party(me.party.number), words);
    let limit = if yours.is_empty() { 4 } else { 3 };
    for role in facts.roles.iter().take(limit) {
        rows.push(TeamRow {
            name: copy_of(&role.name),
            areas: held_areas(areas.as_ref(), &team::Holder::Task(role.task), words),
            line: role.now.as_deref().unwrap_or_default().into(),
        })
        .expect("four team rows");
    }
    if !yours.is_empty() {
        let mut line = Box::default();
        for person in &facts.people {
            if person.you {
                line = shared::person_line(words.view(), person).unwrap_or_default();
            }
        }
        rows.push(TeamRow { name: words.say(Word::You, &[]), areas: yours, line }).expect("person keeps fourth place");
    }
    rows
}
pub(crate) fn read(root: &Domain, tui: &Tui, words: &Words, wall: Wall) -> Launchpad {
    let me = if root.client().link().state == Link::SignedOut { None } else { root.pages().me(root.client()) };
    let number = match me {
        Some(me) => project(root, me),
        None => None,
    };
    let project = match (me, number) {
        (Some(me), Some(number)) => name(me, number),
        (Some(_), None) | (None, Some(_) | None) => None,
    };
    let mut part = Launchpad {
        project,
        drafts: match number {
            Some(project) => temper_client_domain::pages::drafts_offered(root, project),
            None => false,
        },
        talk: match number {
            Some(project) => super::talk::available(root, project),
            None => false,
        },
        address: crate::place(tui.address(), tui.front()),
        playground: front::playground(root, number),
        lines: List::with_capacity(4),
        left_off: List::with_capacity(3),
        selected: tui.left_off(),
        needs: Box::default(),
        brief: List::with_capacity(9),
        team: List::with_capacity(4),
        columns: tui.columns(),
        notes: foot::read_notes(root, tui, words),
        tick: None,
        wall,
    };
    if root.client().link().state == Link::OutOfDate {
        push(&mut part.lines, Role::Paragraph, &words.view().say(Shared::OutOfDateMain, &[]), None);
    } else if let Some(me) = me {
        if let Some(number) = number {
            if part.project.is_some() {
                home(&mut part, root, number, me, words, wall, tui.brief_shown());
                if part.playground {
                    push(&mut part.lines, Role::Paragraph, &words.say(Word::PlaygroundBanner, &[]), None);
                }
            }
        } else {
            push(&mut part.lines, Role::Paragraph, &words.view().say(Shared::NoProjectYet, &[]), None);
            push(
                &mut part.lines,
                Role::Paragraph,
                &words.say(Word::NoProject, &[Hole::Name(copy_of(tui.address()))]),
                None,
            );
        }
    } else {
        push(
            &mut part.lines,
            Role::Paragraph,
            &words.say(Word::Connecting, &[Hole::Name(copy_of(tui.address()))]),
            None,
        );
    }
    if tui.unkept() {
        push(&mut part.lines, Role::Alert, &words.say(Word::NotKept, &[]), Some(look::REFUSED));
    }
    part
}
#[expect(
    clippy::disallowed_methods,
    reason = "view words are validated UTF-8, with a character per column as chat::cut"
)]
fn columns(bytes: &[u8]) -> usize {
    core::str::from_utf8(bytes).expect("view words are UTF-8").chars().count()
}
fn padded(bytes: &[u8], width: usize) -> Box<[u8]> {
    let mut spaces =
        List::with_capacity(u32::try_from(width.saturating_sub(columns(bytes))).expect("project padding fits"));
    for _column in columns(bytes)..width {
        spaces.push(b' ').expect("project padding fits");
    }
    concat(&[bytes, spaces.as_slice()])
}
#[expect(clippy::disallowed_methods, reason = "cutting validated UTF-8 view words at character and word boundaries")]
pub(super) fn cut(bytes: &[u8], room: usize, words: &Words) -> (Box<[u8]>, usize) {
    let line = core::str::from_utf8(bytes).expect("view words are UTF-8");
    if line.chars().count() <= room {
        return (copy_of(bytes), bytes.len());
    }
    if room == 0 {
        return (Box::default(), 0);
    }
    let mut end = 0;
    let mut word_end = None;
    let mut previous = 0;
    for (column, (at, character)) in line.char_indices().enumerate() {
        if character.is_whitespace() {
            word_end = Some(previous);
        }
        if column == room.saturating_sub(1) {
            end = at;
            break;
        }
        if !character.is_whitespace() {
            previous = at.checked_add(character.len_utf8()).expect("line fits memory");
        }
    }
    let end = word_end.unwrap_or(end);
    (concat(&[bytes.get(..end).expect("word boundary"), &words.say(Word::Dots, &[])]), end)
}
fn span(b: &mut Builder, bytes: &[u8], style: Option<Style>) {
    b.open(Role::Text);
    if let Some(style) = style {
        b.style(style);
    }
    b.run(Run::Plain, bytes);
    b.close();
}
#[expect(clippy::too_many_arguments, reason = "a row's styled words share one room calculation and hint placement")]
fn row(
    b: &mut Builder,
    part: &Launchpad,
    project: &[u8],
    title: &[u8],
    tail: &[u8],
    hint: &[u8],
    mark: usize,
    words: &Words,
) {
    let mut longest = 0;
    for left in &part.left_off {
        longest = longest.max(columns(&left.project));
    }
    for need in part.needs.iter().take(5) {
        longest = longest.max(columns(&need.project));
    }
    let project = padded(project, longest.saturating_add(3));
    // Sections have one column of padding on each side and two for the item mark.
    let room = if part.columns == 0 { 80 } else { usize::from(part.columns) }
        .saturating_sub(4)
        .saturating_sub(mark)
        .saturating_sub(columns(&project));
    let with_hint = room.saturating_sub(columns(hint)).saturating_sub(2);
    let hint = if hint.is_empty() || with_hint < 24 { b"" } else { hint };
    let room = if hint.is_empty() { room } else { with_hint };
    let (line, end) = cut(&concat(&[title, tail]), room, words);
    let split = title.len().min(end);
    span(b, &project, Some(look::NAME));
    span(b, line.get(..split).expect("title boundary"), None);
    span(b, line.get(split..).expect("tail boundary"), Some(look::HINT));
    if !hint.is_empty() {
        b.text(Run::Plain, &padded(b"", room.saturating_sub(columns(&line)).saturating_add(2)));
        span(b, hint, Some(look::HINT));
    }
}
fn draw_team(b: &mut Builder, part: &Launchpad, words: &Words) {
    if part.team.is_empty() {
        return;
    }
    let Some(project) = &part.project else { return };
    let mut names = 0;
    let mut areas = 0;
    for row in &part.team {
        names = names.max(columns(&row.name));
        areas = areas.max(columns(&row.areas));
    }
    let room = if part.columns == 0 { 80 } else { usize::from(part.columns) }.saturating_sub(4);
    b.open(Role::Region);
    text(b, Role::Paragraph, &words.say(Word::TeamHead, &[Hole::Name(copy_of(project))]), Some(look::HEAD));
    b.open(Role::List);
    b.style(look::LEFT_OFF);
    for row in &part.team {
        b.open(Role::Item);
        b.open(Role::Paragraph);
        let name = padded(&row.name, names.saturating_add(3));
        let areas = if areas == 0 { Box::default() } else { padded(&row.areas, areas.saturating_add(3)) };
        let (line, end) = cut(&concat(&[&name, &areas, &row.line]), room, words);
        let split = name.len().min(end);
        span(b, line.get(..split).expect("name boundary"), Some(look::NAME));
        span(b, line.get(split..).expect("line boundary"), Some(look::HINT));
        b.close();
        b.close();
    }
    b.close();
    b.close();
}
// The screen adds punctuation to Term and Heading. Paragraph heads keep
// the section labels plain; Description retains the two-column words indent.
#[expect(clippy::too_many_lines, reason = "the foot and launchpad retain their existing section layout")]
fn sections(b: &mut Builder, part: &Launchpad, words: &Words) {
    b.open(Role::Region);
    b.region(look::SECTIONS);
    if !part.left_off.is_empty()
        || !part.needs.is_empty()
        || part.project.is_some()
        || !part.notes.lines.is_empty()
        || part.notes.gave_up.is_some()
        || part.notes.log.is_some()
        || !part.brief.is_empty()
    {
        text(b, Role::Paragraph, b"", None);
    }
    b.open(Role::Group);
    b.style(look::SPACED);
    if !part.left_off.is_empty() {
        b.open(Role::Region);
        text(b, Role::Paragraph, &words.say(Word::LeftOff, &[]), Some(look::HEAD));
        b.open(Role::List);
        b.style(look::LEFT_OFF);
        for left in &part.left_off {
            b.open(Role::Item);
            if part.selected == Some(left.task) {
                b.style(look::PICKED);
            }
            b.open(Role::Paragraph);
            let kind = match left.kind {
                LeftKind::Chat { session: true, .. } => Word::KindSession,
                LeftKind::Chat { session: false, .. } => Word::KindChat,
                LeftKind::Goal => Word::KindGoal,
            };
            let tail = concat(&[
                &words.say(kind, &[]),
                &words.say(Word::Separator, &[]),
                &left.at,
                &words.say(Word::Separator, &[]),
                &left.state,
            ]);
            let hint = match left.kind {
                LeftKind::Chat { number, .. } => words.say(Word::ChatNumber, &[Hole::Number(u64::from(number))]),
                LeftKind::Goal => Box::default(),
            };
            row(b, part, &left.project, &left.title, &tail, &hint, 0, words);
            b.close();
            b.close();
        }
        b.close();
        b.close();
    }
    if !part.needs.is_empty() {
        b.open(Role::Region);
        text(b, Role::Paragraph, &words.say(Word::NeedsYou, &[]), Some(look::HEAD));
        for (index, need) in part.needs.iter().take(5).enumerate() {
            b.open(Role::Description);
            b.open(Role::Paragraph);
            let marker = match need.mark {
                NeedMark::Question => Word::Question,
                NeedMark::Money => Word::MoneyNeed,
                NeedMark::SignOff => Word::SignOffNeed,
                NeedMark::Other => Word::OtherNeed,
            };
            text(b, Role::Text, &words.say(marker, &[]), Some(look::SIGNAL));
            b.text(Run::Plain, b" ");
            let hint = if index.checked_add(1) == Some(part.needs.len().min(5)) {
                words.say(Word::WalksThem, &[])
            } else {
                Box::default()
            };
            let tail = match need.since {
                Some(since) => {
                    let (ago, _) = words.view().time(TimeForm::Ago, since, part.wall);
                    concat(&[&words.say(Word::Separator, &[]), &words.say(Word::Asked, &[Hole::Name(ago)])])
                }
                None => Box::default(),
            };
            row(b, part, &need.project, &need.words, &tail, &hint, 2, words);
            b.close();
            b.close();
        }
        b.close();
    }
    draw_team(b, part, words);
    if let Some(project) = &part.project {
        b.open(Role::Region);
        text(b, Role::Paragraph, &words.say(Word::StartHead, &[]), Some(look::HEAD));
        for (command, description) in [
            (Own::Draft, words.say(Word::DraftStart, &[])),
            (Own::Change, words.say(Word::ChangeStart, &[Hole::Name(copy_of(project))])),
            (Own::Talk, words.say(Word::TalkStart, &[])),
            (Own::Ask, words.say(Word::AskTemper, &[])),
        ] {
            if (command == Own::Draft && !part.drafts) || (command == Own::Talk && !part.talk) {
                continue;
            }
            b.open(Role::Description);
            b.open(Role::Paragraph);
            text(
                b,
                Role::Text,
                &padded(command.name(), columns(Own::Change.name()).saturating_add(3)),
                Some(look::NAME),
            );
            text(b, Role::Text, &description, Some(look::HINT));
            b.close();
            b.close();
        }
        b.close();
    }
    if part.notes.follows_end() {
        b.open(Role::Group);
    } else {
        b.close();
    }
    foot::draw_notes(b, &part.notes, words);
    for line in &part.brief {
        text(b, line.role, &line.words, line.style);
    }
    if part.notes.follows_end() {
        b.close();
        b.close();
    }
    b.close();
}
pub(crate) fn draw(b: &mut Builder, part: &Launchpad, words: &Words, _binds: &mut Binds) {
    b.open(Role::Main);
    b.open(Role::Region);
    b.region(look::TOP);
    b.open(Role::Heading { level: 1 });
    text(b, Role::Text, &words.say(Word::Title, &[]), Some(look::TITLE));
    if let Some(project) = &part.project {
        text(b, Role::Text, &words.say(Word::Separator, &[]), Some(look::FAINT));
        text(b, Role::Text, project, Some(look::NAME));
        if part.playground {
            text(b, Role::Text, &words.say(Word::Playground, &[]), Some(look::PLAYGROUND));
            text(b, Role::Text, &words.say(Word::Separator, &[]), Some(look::END_FAINT));
        }
        text(b, Role::Text, &part.address, Some(look::ADDRESS));
    }
    b.close();
    for line in &part.lines {
        text(b, line.role, &line.words, line.style);
    }
    b.close();
    sections(b, part, words);
    b.close();
}
