//! The project goals' page (temper/docs/design/client/pages/README.md).
pub(crate) use crate::common::goals::way_name;
pub use crate::common::goals::{Way, ways};
use crate::{
    bindings::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{List, Map, bytes::copy_of};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain, GoalsFilter,
    pages::{
        Act,
        goals::{self, Goals, State, Wait},
        report::Report,
    },
};

/// Names held by a page, extensible with further task lines (client/pages/work.md, 1).
#[derive(Debug)]
pub struct Names<'a> {
    chips: Map<u64, &'a c::Chip>,
    goals: Option<&'a Goals>,
    me: Option<&'a c::Me>,
}
impl<'a> Names<'a> {
    #[must_use]
    #[expect(clippy::new_without_default, reason = "page-local names use an explicit constructor")]
    pub fn new() -> Self {
        Self { chips: Map::uncapped(), goals: None, me: None }
    }
    #[must_use]
    pub fn of(root: &'a Domain) -> Self {
        let mut names = Self::new();
        names.goals = root.pages().goals();
        if let Some(page) = names.goals {
            if let Some(live) = page.live(root.client()) {
                for line in &live.rows {
                    names.add(line);
                }
            }
            for line in page.ended() {
                names.add(line);
            }
        }
        if let Some(milestones) = root.pages().milestones() {
            for milestone in milestones.drawn() {
                for goal in &milestone.goals {
                    names.add(goal);
                }
            }
        }
        names
    }
    pub fn add(&mut self, line: &'a c::TaskLine) {
        self.chips.insert(line.chip.task, &line.chip).expect("held task names fit");
    }
    pub(crate) fn dependencies(task: &'a c::Task, me: Option<&'a c::Me>) -> Self {
        let mut names = Self::new();
        names.me = me;
        for link in &task.dependencies {
            match link {
                c::TaskLink::Live(chip) | c::TaskLink::Retained { chip, .. } => {
                    names.chips.insert(chip.task, chip).expect("document dependency names fit");
                }
                c::TaskLink::Stored { .. } => {}
            }
        }
        names
    }
    fn title(&self, task: u64) -> Option<Box<[u8]>> {
        let title = self.chips.get(&task)?.title.bytes();
        Some(match goals::id(title) {
            Some(id) => copy_of(id),
            None => crate::common::chat::cut_title(title, 40, false),
        })
    }
    fn yours(&self, task: u64) -> bool {
        if let Some(goals) = self.goals {
            return goals.needs_own(task);
        }
        if let Some(chip) = self.chips.get(&task)
            && let c::ChipPhase::Person { address, .. } = chip.details.phase
            && let Some(me) = self.me
        {
            let project = chip.details.details.project;
            let whom = match address {
                c::PersonAddress::Party(party) => c::Whom::Person(party),
                c::PersonAddress::Role(role) => c::Whom::Role { project, role },
            };
            return temper_client_domain::pages::task::reader(whom, project, me);
        }
        false
    }
    fn wait(&self, words: &Words, wait: Wait, chip: &c::Chip) -> Box<[u8]> {
        if let c::ChipPhase::Waiting { dependencies, .. } = &chip.details.phase {
            for task in dependencies {
                if self.yours(*task) {
                    return words.say(Word::GoalWaitingYou, &[]);
                }
            }
        }
        if !wait.more {
            if wait.count == 1 {
                return match self.title(wait.first) {
                    Some(title) => words.say(Word::GoalWaitsTitle, &[Hole::Name(title)]),
                    None => words.say(Word::GoalWaitsTask, &[Hole::Number(wait.first)]),
                };
            }
            if wait.count == 2
                && let Some(first) = self.title(wait.first)
                && let Some(second_task) = wait.second
                && let Some(second) = self.title(second_task)
            {
                return words.say(Word::GoalWaitingPair, &[Hole::Name(first), Hole::Name(second)]);
            }
        }
        words.say(
            if wait.more { Word::GoalWaitingMore } else { Word::GoalWaitsTasks },
            &[Hole::Number(u64::from(wait.count))],
        )
    }
}

pub fn main(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, project: u32, times: &mut Times) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::GOALS);
    super::work::head(b, root, words, bindings, project, times);
    intro(b, root, words, project);
    list(b, root, words, bindings, project, times);
    b.close();
}

fn intro(b: &mut Builder, root: &Domain, words: &Words, project: u32) {
    let me = root.pages().me(root.client());
    let page = root.pages().goals();
    b.open(Role::Slot);
    b.style(look::GOALS_INTRO);
    if me.is_some()
        && let Some(page) = page
        && let Some(live) = page.live(root.client())
    {
        let mut underway = 0_u32;
        for line in live.rows.iter().chain(page.ended()) {
            let state = goals::state(
                &line.chip,
                page.needs_person(line.chip.task),
                page.report(line.chip.task).is_some(),
                line.delegates_total,
            );
            match state {
                State::Needs { .. }
                | State::Running
                | State::Planning
                | State::Paused
                | State::Waits(_)
                | State::WaitsForRoom => {
                    underway = underway.checked_add(1).expect("goal rows fit");
                }
                State::Landed | State::Closed | State::Failed => {}
            }
        }
        let count = page.count(root.client());
        let word = if page.unread(root.client()) {
            if count == 1 { Word::GoalsMoreOne } else { Word::GoalsMoreMany }
        } else if count == 0 {
            Word::GoalsNone
        } else if count == 1 {
            Word::GoalsOne
        } else {
            Word::GoalsMany
        };
        let holes = [Hole::Number(u64::from(count)), Hole::Number(u64::from(underway))];
        if word != Word::GoalsNone || page.known(root.client()) {
            text(
                b,
                Role::Heading { level: 2 },
                look::GOALS_HEADLINE,
                &words.say(word, if word == Word::GoalsNone { &[] } else { &holes }),
            );
        }
    }
    b.open(Role::Paragraph);
    b.style(look::GOALS_LEDE);
    let ways = match me {
        Some(me) => ways(me, project, words),
        None => List::uncapped(),
    };
    if ways.is_empty() {
        words.text(b, Word::GoalsLede, &[]);
    } else {
        words.text(b, Word::GoalsWays, &[]);
        for (index, way) in ways.iter().enumerate() {
            if index != 0 {
                words.text(b, Word::GoalsWaySeparator, &[]);
            }
            b.text(Run::Plain, &lowercase(&way.name));
        }
        words.text(b, Word::GoalsWayEnd, &[]);
    }
    b.close();
    b.close();
}

#[expect(clippy::disallowed_types, reason = "Unicode lowercasing owns its temporary UTF-8 string")]
pub(crate) fn lowercase(name: &[u8]) -> Box<[u8]> {
    use alloc::string::String;
    String::from_utf8_lossy(name).to_lowercase().into_bytes().into_boxed_slice()
}

fn list(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, project: u32, times: &mut Times) {
    let me = root.pages().me(root.client());
    let page = root.pages().goals();
    if me.is_some()
        && let Some(page) = page
        && let Some(live) = page.live(root.client())
    {
        b.open(Role::Slot);
        b.style(look::GOALS_SECTION);
        b.open(Role::Slot);
        b.style(look::GOALS_SECTION_HEAD);
        b.open(Role::Slot);
        b.style(look::GOALS_FILTERS);
        for (filter, word) in [
            (GoalsFilter::All, Word::GoalsAll),
            (GoalsFilter::Needs, Word::GoalsNeeds),
            (GoalsFilter::Live, Word::GoalsLive),
            (GoalsFilter::Ended, Word::GoalsEnded),
        ] {
            b.open(Role::Button);
            b.name(&words.say(word, &[]));
            if page.filter == filter {
                b.state(skein_ui::State::Pressed);
            }
            b.bind(bindings.insert(Act::GoalsFilter { filter }));
            b.close();
        }
        b.close();
        if let Some(me) = me
            && !ways(me, project, words).is_empty()
        {
            let name = words.say(Word::GoalsStartDraft, &[]);
            b.open(Role::Button);
            b.name(&name);
            b.bind(bindings.insert(Act::OpenStart { draft: None, chat: None }));
            b.style(look::SHEET_LINK);
            b.close();
        }
        b.close();
        let names = Names::of(root);
        let mut shown = List::uncapped();
        for line in live.rows.iter().chain(page.ended()) {
            let report = page.report(line.chip.task);
            let state =
                goals::state(&line.chip, page.needs_person(line.chip.task), report.is_some(), line.delegates_total);
            if goals::shown(page.filter, state) {
                shown.push(line.clone()).expect("held goals fit");
            }
        }
        if !shown.is_empty() {
            b.open(Role::Slot);
            b.style(look::GOALS_ROWS);
            let milestones = root.pages().milestones();
            for group in &goals::groups(shown.as_slice(), milestones) {
                match *group {
                    goals::GroupRow::Goal(task) => {
                        if let Some(line) = goals::goal(shown.as_slice(), &[], task) {
                            held_row(b, words, bindings, times, page, line, &names);
                        }
                    }
                    goals::GroupRow::Milestone(task) => {
                        if let Some(milestones) = milestones
                            && let Some(line) = goals::goal(milestones.lines(), &[], task)
                            && let Some(milestone) = milestones.milestone(task)
                        {
                            b.open(Role::Slot);
                            b.style(look::GOALS_GROUP);
                            super::milestones::row(b, words, bindings, line, milestone, page, &names);
                            b.open(Role::Slot);
                            b.style(look::GOALS_GROUP_ROWS);
                            for goal in &shown {
                                if milestones.of(goal) == Some(task) {
                                    held_row(b, words, bindings, times, page, goal, &names);
                                }
                            }
                            b.close();
                            b.close();
                        }
                    }
                }
            }
            b.close();
        } else if page.known(root.client()) {
            text(b, Role::Paragraph, look::GOALS_EMPTY, &words.say(Word::GoalsEmpty, &[]));
        }
        if page.more() {
            b.open(Role::Button);
            b.style(look::GOALS_MORE);
            b.name(&words.say(Word::GoalsMore, &[]));
            b.bind(bindings.insert(Act::MoreGoals));
            b.close();
        }
        b.close();
    }
}

fn text(b: &mut Builder, role: Role, style: skein_ui::Style, value: &[u8]) {
    b.open(role);
    b.style(style);
    b.text(Run::Plain, value);
    b.close();
}

/// A held goal uses the shared read's report and needs.
pub(crate) fn held_row(
    b: &mut Builder,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    page: &Goals,
    line: &c::TaskLine,
    names: &Names<'_>,
) {
    let report = page.report(line.chip.task);
    let state = goals::state(&line.chip, page.needs_person(line.chip.task), report.is_some(), line.delegates_total);
    row(b, words, bindings, times, line, state, report, names);
}

/// A goal's state word and token (client/pages/README.md, client/pages/goal.md, 2).
pub(crate) fn state_word(
    words: &Words,
    state: State,
    names: &Names<'_>,
    chip: &c::Chip,
) -> (Box<[u8]>, skein_ui::Style) {
    let (word, token) = match state {
        State::Needs { funds: true } => (Word::GoalNeedsFunds, look::GOALS_STATE_NEEDS),
        State::Needs { funds: false } => (Word::GoalNeedsYou, look::GOALS_STATE_NEEDS),
        State::Paused => (Word::GoalPaused, look::GOALS_STATE_PAUSED),
        State::Waits(wait) => return (names.wait(words, wait, chip), look::GOALS_STATE_WAITS),
        State::WaitsForRoom => (Word::GoalWaitingResources, look::GOALS_STATE_WAITS),
        State::Planning => (Word::GoalPlanning, look::GOALS_STATE_PLANNING),
        State::Running => (Word::Running, look::GOALS_STATE_RUNNING),
        State::Landed => (Word::GoalLandedAlone, look::GOALS_STATE_LANDED),
        State::Closed => (Word::GoalClosed, look::GOALS_STATE_CLOSED),
        State::Failed => (Word::GoalFailed, look::GOALS_STATE_CLOSED),
    };
    (words.say(word, &[]), token)
}

pub(crate) fn wait_sentence(words: &Words, state: State, names: &Names<'_>, chip: &c::Chip) -> Box<[u8]> {
    let (mut word, _) = state_word(words, state, names, chip);
    if let Some(first) = word.first_mut() {
        first.make_ascii_uppercase();
    }
    words.say(Word::GoalWaitingSentence, &[Hole::Name(word)])
}

/// One goal links its title, state and report to its task (client/pages/README.md).
#[expect(clippy::too_many_arguments, reason = "the shared row takes its held names alongside the report and state")]
pub(crate) fn row(
    b: &mut Builder,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    line: &c::TaskLine,
    state: State,
    report: Option<&Report>,
    names: &Names<'_>,
) {
    if words.shell == Shell::Terminal {
        b.open(Role::Slot);
    }
    let (word, token) = state_word(words, state, names, &line.chip);
    let status = if state == State::Landed {
        match line.ended_at {
            Some(at) => words.say(Word::GoalLanded, &[Hole::Name(times.say(words, TimeForm::Clock, at))]),
            None => word,
        }
    } else {
        word
    };
    let count = if line.delegates_total == 0 {
        Box::default()
    } else {
        crate::accounts::words::join(&[
            &words.say(
                Word::GoalCount,
                &[Hole::Number(u64::from(line.delegates_done)), Hole::Number(u64::from(line.delegates_total))],
            ),
            &words.say(Word::ToolSeparator, &[]),
        ])
    };
    let title = crate::common::chat::row_title(line.chip.title.bytes());
    let name = crate::accounts::words::join(&[&title, &words.say(Word::ToolSeparator, &[]), &count, &status]);
    crate::bindings::link(b, &name, bindings.insert(Act::Go { address: Address::Task { task: line.chip.task } }));
    b.style(look::GOALS_ROW);
    match state {
        State::Landed | State::Closed | State::Failed => b.style(look::GOALS_ROW_ENDED),
        State::Needs { .. }
        | State::Paused
        | State::Waits(_)
        | State::WaitsForRoom
        | State::Planning
        | State::Running => {}
    }
    match words.shell {
        Shell::Browser => {
            b.open(Role::Text);
            b.style(look::GOALS_TOP);
            b.text(Run::Plain, &title);
            b.open(Role::Text);
            b.style(look::GOALS_PROGRESS);
            progress(b, words, line, state);
            b.open(Role::Text);
            b.run(Run::Plain, &count);
            b.open(Role::Text);
            b.style(token);
            b.run(Run::Plain, &status);
            b.close();
            b.close();
            b.close();
            b.close();
        }
        Shell::Terminal => {
            b.text(Run::Plain, &name);
            b.close();
        }
    }
    b.open(if words.shell == Shell::Browser { Role::Text } else { Role::Paragraph });
    b.style(look::GOALS_LINE);
    let headline = headline(words, line, state, report, names);
    match words.shell {
        Shell::Browser => b.run(Run::Plain, &headline),
        Shell::Terminal => b.text(Run::Plain, &headline),
    }
    b.close();
    b.close();
}

pub(crate) fn headline(
    words: &Words,
    line: &c::TaskLine,
    state: State,
    report: Option<&Report>,
    names: &Names<'_>,
) -> Box<[u8]> {
    if let Some(report) = report {
        return report.headline.clone();
    }
    let word = match state {
        State::Waits(_) | State::WaitsForRoom => return wait_sentence(words, state, names, &line.chip),
        State::Planning => Word::GoalPlanningLine,
        State::Needs { .. } => {
            if line.delegates_total == 0 {
                Word::GoalPlanningLine
            } else {
                Word::GoalTasksDone
            }
        }
        State::Paused | State::Running => Word::GoalTasksDone,
        State::Landed => Word::GoalLandedLine,
        State::Closed => Word::GoalClosedLine,
        State::Failed => Word::GoalFailedLine,
    };
    if word == Word::GoalTasksDone {
        tasks_done(words, line.delegates_done, line.delegates_total)
    } else {
        words.say(word, &[])
    }
}

/// The row counts tasks done, whatever they did (client/pages/work.md, 1).
fn progress(b: &mut Builder, words: &Words, line: &c::TaskLine, state: State) {
    if line.delegates_total == 0 {
        return;
    }
    let ongoing = match state {
        State::Landed | State::Closed | State::Failed => false,
        State::Needs { .. }
        | State::Paused
        | State::Waits(_)
        | State::WaitsForRoom
        | State::Planning
        | State::Running => true,
    };
    b.open(Role::Figure);
    b.style(look::GOAL_BAR);
    b.name(&words.say(
        Word::GoalsBarCount,
        &[Hole::Number(u64::from(line.delegates_done)), Hole::Number(u64::from(line.delegates_total))],
    ));
    for part in 0..line.delegates_total {
        b.open(Role::Text);
        if part < line.delegates_done {
            b.style(look::GOAL_BAR_LANDED);
        } else if part == line.delegates_done && ongoing {
            b.style(look::GOAL_BAR_RUNNING);
        }
        b.close();
    }
    b.close();
}

pub(crate) fn tasks_done(words: &Words, done: u32, total: u32) -> Box<[u8]> {
    let mut done = if done == 0 { words.say(Word::GoalTasksNone, &[]) } else { super::goal::count_words(words, done) };
    if let Some(first) = done.first_mut() {
        first.make_ascii_uppercase();
    }
    words.say(Word::GoalTasksDone, &[Hole::Name(done), Hole::Name(super::goal::count_words(words, total))])
}
