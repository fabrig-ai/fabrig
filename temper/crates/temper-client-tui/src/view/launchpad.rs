//! Home's launchpad (temper/docs/design/client/tui/pages/launchpad.md, 1 and temper/docs/design/client/tui/box.md, The launchpad).
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
    pages::{chats, home},
};
use temper_client_view::{
    shared::{self, NeedMark},
    words::{TimeForm, Word as Shared},
};

#[derive(Debug)]
pub(crate) struct Launchpad {
    pub project: Option<Box<[u8]>>,
    pub drafts: bool,
    pub address: Box<[u8]>,
    pub playground: bool,
    pub lines: List<Line>,
    pub left_off: List<LeftOff>,
    pub selected: Option<u64>,
    pub needs: Box<[NeedRow]>,
    pub brief: List<Line>,
    pub columns: u16,
    pub notes: foot::Notes,
    pub tick: Option<Wall>,
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
}
#[derive(Debug)]
pub(crate) struct Line {
    pub role: Role,
    pub words: Box<[u8]>,
    pub style: Option<Style>,
}
fn push(lines: &mut List<Line>, role: Role, words: &[u8], style: Option<Style>) {
    lines.push(Line { role, words: copy_of(words), style }).expect("header has at most three lines");
}
fn project(root: &Domain, me: &c::Me) -> Option<u32> {
    match root.pages().address() {
        Address::Home { project } => Some(*project),
        Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Ask { .. }
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
fn left_off(root: &Domain, project: u32, me: &c::Me, words: &Words, wall: Wall) -> List<LeftOff> {
    let mut lines = List::with_capacity(3);
    let tasks = match root.pages().chats() {
        Some(chats) => chats.live(root.client()),
        None => None,
    };
    let mut goal: Option<&c::TaskLine> = None;
    if let Some(goals) = root.pages().goals()
        && let Some(tasks) = goals.live(root.client())
    {
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
                if row.project != project || chats::ended(&row.chip) {
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
        lines
            .push(LeftOff {
                task: row.chip.task,
                project: name(me, row.project).unwrap_or_default(),
                title: copy_of(row.chip.title.bytes()),
                kind,
                at: words.view().time(TimeForm::Clock, row.last_activity, wall).0,
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
    let facts = home::facts(root, project, wall);
    if !part.playground {
        if let Some(brief) = facts.brief() {
            push(&mut part.lines, Role::Paragraph, brief.sentence(), None);
            let (clock, _) = words.view().time(TimeForm::Clock, Wall::from_nanos(brief.at()), wall);
            let byline = words.say(Word::Byline, &[Hole::Name(clock)]);
            let more = if brief.lede().is_empty() { Box::default() } else { words.say(Word::BriefMore, &[]) };
            push(&mut part.lines, Role::Paragraph, &concat(&[&byline, &more]), Some(look::BYLINE));
            if shown {
                for line in [brief.sentence(), brief.lede(), &byline] {
                    push(&mut part.brief, Role::Paragraph, line, Some(look::NOTE));
                }
            }
        } else {
            match facts.needs {
                home::Needs::Unknown => {}
                home::Needs::Count { .. } => {
                    let (line, tick) = shared::home_headline(
                        words.view(),
                        part.project.as_deref().expect("project has a name"),
                        &facts,
                        wall,
                    );
                    push(&mut part.lines, Role::Paragraph, &line, None);
                    part.tick = tick;
                }
            }
        }
    }
    part.left_off = left_off(root, project, me, words, wall);
    let mut needs = List::with_capacity(5);
    for prompt in shared::need_prompts(root, words.view()).into_iter().take(5) {
        let project = match prompt.project {
            Some(number) => name(me, number).unwrap_or_default(),
            None => Box::default(),
        };
        needs.push(NeedRow { mark: prompt.mark, project, words: prompt.line.words }).expect("five launchpad needs");
    }
    part.needs = needs.into_boxed();
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
        address: crate::place(tui.address(), tui.front()),
        playground: front::playground(root, number),
        lines: List::with_capacity(3),
        left_off: List::with_capacity(3),
        selected: tui.left_off(),
        needs: Box::default(),
        brief: List::with_capacity(3),
        columns: tui.columns(),
        notes: foot::read_notes(root, tui, words),
        tick: None,
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
fn cut(bytes: &[u8], room: usize, words: &Words) -> (Box<[u8]>, usize) {
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
// The screen adds punctuation to Term and Heading. Paragraph heads keep
// the section labels plain; Description retains the two-column words indent.
#[expect(clippy::too_many_lines, reason = "the foot and launchpad retain their existing section layout")]
fn sections(b: &mut Builder, part: &Launchpad, words: &Words) {
    b.open(Role::Region);
    b.region(look::SECTIONS);
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
            row(b, part, &need.project, &need.words, b"", &hint, 2, words);
            b.close();
            b.close();
        }
        b.close();
    }
    if let Some(project) = &part.project {
        b.open(Role::Region);
        text(b, Role::Paragraph, &words.say(Word::StartHead, &[]), Some(look::HEAD));
        for (command, description) in [
            (Own::Draft, words.say(Word::DraftStart, &[])),
            (Own::Change, words.say(Word::ChangeStart, &[Hole::Name(copy_of(project))])),
            (Own::Ask, words.say(Word::AskTemper, &[])),
        ] {
            if command == Own::Draft && !part.drafts {
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
    if part.notes.log.is_some() {
        b.open(Role::Group);
    } else {
        b.close();
    }
    foot::draw_notes(b, &part.notes, words);
    for line in &part.brief {
        text(b, line.role, &line.words, line.style);
    }
    if part.notes.log.is_some() {
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
