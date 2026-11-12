//! A goal's header, needs, report and side (temper/docs/design/client/pages/goal.md, 2, temper/docs/design/client/pages/task.md, 4, temper/docs/design/client/view.md, 6).
use super::ask::{self, Kind};
use crate::{
    frame::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::{Latest, contract as c};
use skein_lib::{List, Map, Wall, bytes::copy_of};
use skein_ui::{Builder, Role, Run, State as UiState, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act, delegation,
        goal::{self, Counts, Fallback},
        goals::State,
        inbox::Card,
        needs,
        report::Report,
    },
};

pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    rows: u32,
    times: &mut Times,
    sign_in: &crate::forge_sign_in::Forges,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::GOAL);
    if let Some(page) = root.pages().task() {
        match page.document(root.client()) {
            Some(task) => {
                if let Some(goal) = &page.goal {
                    let ended = match &page.ended {
                        Some(ended) => ended.page.as_deref(),
                        None => None,
                    };
                    let bands = open_bands(root, words.shell);
                    let shown = Page {
                        task,
                        report: goal.report.as_ref(),
                        draft: &goal.draft,
                        counts: goal.counts(task, ended),
                        state: goal.state(task, ended, !bands.is_empty()),
                    };
                    shown.header(b, words, bindings, times);
                    super::task::amendment(b, root, words, bindings);
                    let mut question = false;
                    for card in &bands {
                        match card {
                            needs::Need::Card(card) => {
                                let first = !question;
                                question |= card.question_words().is_some();
                                band(b, root, task, card, words, bindings, first);
                            }
                            needs::Need::Approval(row) => super::approval::band(b, row, words, bindings, sign_in),
                        }
                    }
                    b.open(Role::Slot);
                    b.style(look::GOAL_TWO);
                    shown.report(b, words, times);
                    shown.side(b, root, words, bindings, times);
                    b.close();
                    sent(b, root, page, words);
                    composer(b, root, page, task, words, bindings);
                }
            }
            None => super::task::reading(b, page, words),
        }
    }
    crate::confirm::main(b, root, words, bindings, rows);
    b.close();
}

fn open_bands(root: &Domain, shell: Shell) -> Box<[needs::Need<'_>]> {
    let mut bands = List::uncapped();
    if let Some(inbox) = root.pages().inbox() {
        for need in &root.pages().needs().needs(inbox, root.forge().approvals()) {
            match need {
                needs::Need::Card(card) => {
                    if card.group != c::InboxGroup::Questions || card.page.is_some() {
                        bands.push(*need).expect("open inbox cards fit");
                    }
                }
                needs::Need::Approval(_) => {
                    if shell == Shell::Browser {
                        bands.push(*need).expect("approval rows fit");
                    }
                }
            }
        }
    }
    bands.into_boxed()
}

fn band(b: &mut Builder, root: &Domain, task: &c::Task, card: &Card, w: &Words, bindings: &mut Bindings, first: bool) {
    let mut asked = if first { ask::read(card, w) } else { ask::bounded(card, w) };
    let kind = match asked.kind {
        Kind::Funds { .. } => Word::NeedsKindFunds,
        Kind::Question | Kind::Proposal | Kind::Held | Kind::PersonTask => ask::card_kind(card),
    };
    let question = match asked.kind {
        Kind::Funds { spent, limit } => w.say(Word::NeedsFundsUsed, &[Hole::Number(spent), Hole::Number(limit)]),
        Kind::Question if first => asked.parsed.as_ref().expect("question parsed").question.clone(),
        Kind::Question | Kind::Proposal | Kind::Held | Kind::PersonTask => asked.short.clone(),
    };
    b.open(Role::Region);
    b.style(look::GOAL_BAND);
    b.name(&w.say(Word::NeedsYou, &[]));
    b.open(Role::Paragraph);
    b.open(Role::Text);
    b.style(look::GOAL_BAND_TAG);
    b.text(Run::Plain, &w.say(Word::NeedsYou, &[]));
    b.text(Run::Plain, &w.say(Word::ToolSeparator, &[]));
    b.text(Run::Plain, &w.say(kind, &[]));
    b.close();
    b.text(Run::Plain, &question);
    b.close();
    if first {
        ask::whole_parts(b, &asked, w, bindings);
    } else {
        asked.parsed = None;
    }
    band_controls(b, root, task, &asked, w, bindings);
    b.close();
}

fn band_controls(
    b: &mut Builder,
    root: &Domain,
    task: &c::Task,
    asked: &ask::Asked<'_>,
    w: &Words,
    bindings: &mut Bindings,
) {
    let card = asked.card;
    b.open(Role::Slot);
    b.style(look::GOAL_CHOICES);
    if let Some(parsed) = &asked.parsed
        && !parsed.choices.is_empty()
    {
        for (i, choice) in parsed.choices.iter().enumerate() {
            b.open(Role::Button);
            b.style(look::GOAL_BTN);
            b.name(&choice.words);
            b.bind(
                bindings.insert(Act::Needs(needs::Act::Choose { object: card.object, words: choice.words.clone() })),
            );
            if parsed.primary() == Some(i) {
                b.style(look::GOAL_PRIMARY);
            }
            if choice.recommended {
                if w.shell == Shell::Terminal {
                    b.text(Run::Plain, &choice.words);
                    b.text(Run::Plain, &w.say(Word::ToolSeparator, &[]));
                }
                ask::text(b, Role::Text, &w.say(Word::NeedsRecommended, &[]), look::GOAL_PRIMARY);
            }
            b.close();
        }
    } else if funds_offered(asked) {
        crate::confirm::needs_controls(b, root, card, w, bindings);
    } else {
        let word = match asked.kind {
            Kind::Question | Kind::PersonTask => Word::Answer,
            Kind::Proposal | Kind::Held | Kind::Funds { .. } => Word::HomeOpen,
        };
        let name = w.say(word, &[]);
        crate::link(
            b,
            &name,
            bindings.insert(Act::Go {
                address: Address::Inbox {
                    filter: temper_client_domain::address::Filter {
                        project: Some(task.project),
                        goal: Some(task.chip.task),
                        kind: None,
                    },
                },
            }),
        );
        b.style(look::GOAL_BTN);
        b.style(look::GOAL_PRIMARY);
        b.text(Run::Plain, &name);
        b.close();
    }
    b.close();
}

fn funds_offered(asked: &ask::Asked<'_>) -> bool {
    match asked.kind {
        Kind::Funds { .. } => {}
        Kind::Question | Kind::Proposal | Kind::Held | Kind::PersonTask => return false,
    }
    let Some(page) = &asked.card.page else { return false };
    let offers = temper_client_domain::acts::offers(page);
    jig_client::facts::offer(offers, c::ActKind::Release).is_some()
        || jig_client::facts::offer(offers, c::ActKind::LeaveHeld).is_some()
}

struct Page<'a> {
    task: &'a c::Task,
    report: Option<&'a Report>,
    draft: &'a (Option<Box<[u8]>>, Option<u64>),
    counts: Counts,
    state: State,
}
impl Page<'_> {
    fn header(&self, b: &mut Builder, words: &Words, bindings: &mut Bindings, times: &mut Times) {
        b.open(Role::Banner);
        b.style(look::GOAL_HEAD);
        self.meta(b, words, times);
        let headline = match self.report {
            Some(report) => report.headline.clone(),
            None => fallback(words, goal::fallback(self.state, self.task, self.counts)),
        };
        text(b, Role::Heading { level: 1 }, &headline);
        byline(b, self.report, Word::GoalWrittenAt, words, times);
        b.open(Role::Slot);
        b.style(look::GOAL_ACTS);
        let (draft, chat) = self.draft;
        if draft.is_some()
            && let Some(task) = chat
        {
            chat_link(b, *task, &words.say(Word::GoalDraft, &[]), bindings);
        }
        if let c::Requester::Task(task) = self.task.requester {
            chat_link(b, task, &words.say(Word::GoalChat, &[Hole::Number(task)]), bindings);
        }
        self.acts(b, words, bindings);
        b.close();
        b.close();
    }

    fn acts(&self, b: &mut Builder, words: &Words, bindings: &mut Bindings) {
        match self.task.chip.details.phase {
            c::ChipPhase::Ended => return,
            c::ChipPhase::Active(_)
            | c::ChipPhase::Waiting { .. }
            | c::ChipPhase::Person { .. }
            | c::ChipPhase::Closing(_)
            | c::ChipPhase::Held { .. } => {}
        }
        super::task::act(b, self.task, c::ActKind::Amend, Word::Amend, Act::Amend, words, bindings);
        if self.state == State::Paused {
            super::task::act(b, self.task, c::ActKind::Release, Word::Resume, Act::Resume, words, bindings);
        } else {
            super::task::act(b, self.task, c::ActKind::Stop, Word::Pause, Act::Pause, words, bindings);
        }
        super::task::act(b, self.task, c::ActKind::Cancel, Word::Close, Act::Close, words, bindings);
    }

    fn meta(&self, b: &mut Builder, words: &Words, times: &mut Times) {
        let (state_word, token) = super::goals::state_word(self.state);
        let mut items = List::with_capacity(5);
        items.push(copy_of(self.task.chip.title.bytes())).expect("five meta items");
        items.push(words.say(state_word, &[])).expect("five meta items");
        if self.counts.total != 0 {
            items
                .push(words.say(
                    Word::GoalMetaCount,
                    &[Hole::Number(u64::from(self.counts.done)), Hole::Number(u64::from(self.counts.total))],
                ))
                .expect("five meta items");
        }
        let numbers = self.task.budget.numbers;
        items
            .push(words.say(Word::GoalSpent, &[Hole::Number(numbers.spent), Hole::Number(numbers.budget)]))
            .expect("five meta items");
        items
            .push(words.say(Word::GoalSince, &[Hole::Name(times.say(words, TimeForm::Clock, self.task.made))]))
            .expect("five meta items");
        b.open(Role::Paragraph);
        b.style(look::GOAL_META);
        match words.shell {
            Shell::Browser => {
                for (index, item) in items.iter().enumerate() {
                    if index != 0 {
                        text(b, Role::Text, &words.say(Word::GoalSeparator, &[]));
                    }
                    b.open(Role::Text);
                    if index == 1 {
                        b.style(token);
                    }
                    b.run(if index == 0 { Run::Strong } else { Run::Plain }, item);
                    b.close();
                }
            }
            Shell::Terminal => {
                b.open(Role::Text);
                for (index, item) in items.iter().enumerate() {
                    if index != 0 {
                        b.run(Run::Plain, &words.say(Word::ToolSeparator, &[]));
                    }
                    b.run(Run::Plain, item);
                }
                b.close();
            }
        }
        b.close();
    }

    fn side(&self, b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, times: &mut Times) {
        let tasks = Delegates::of(root);
        b.open(Role::Complementary);
        b.style(look::GOAL_SIDE);
        section(b, Word::GoalSideTasks, words);
        tasks.draw(b, self.state, words, bindings, times);
        b.close();
        section(b, Word::GoalLandingTitle, words);
        let landing = goal::landing(self.state, self.task, self.task.chip.details.details.ended_at, self.counts);
        facts(b, &landing_words(words, times, landing));
        b.close();
        section(b, Word::GoalHowTitle, words);
        let way = if goal::holds_branch(self.task) { Word::GoalLandWhole } else { Word::GoalTaskByTask };
        let asks = if goal::always_asks(self.task.spec.specification.words.bytes.bytes()) {
            Word::GoalAlwaysAsks
        } else {
            Word::GoalFollowsAway
        };
        facts(b, &words.say(asks, &[Hole::Name(words.say(way, &[]))]));
        b.close();
        if let Some(report) = self.report
            && !report.decided.is_empty()
        {
            section(b, Word::GoalDecidedTitle, words);
            b.open(Role::Paragraph);
            b.style(look::GOAL_FACTS);
            for (index, item) in report.decided.iter().enumerate() {
                if index != 0 {
                    b.text(Run::Plain, &words.say(Word::ToolSeparator, &[]));
                }
                b.text(Run::Plain, item);
            }
            b.close();
            b.close();
        }
        b.close();
    }

    fn report(&self, b: &mut Builder, words: &Words, times: &mut Times) {
        b.open(Role::Region);
        b.name(&words.say(Word::GoalReportTitle, &[]));
        b.style(look::GOAL_REPORT);
        b.open(Role::Heading { level: 2 });
        b.style(look::GOAL_TITLE);
        b.name(&words.say(Word::GoalReportTitle, &[]));
        b.text(Run::Plain, &words.say(Word::GoalReportTitle, &[]));
        b.close();
        if let Some(report) = self.report {
            for paragraph in &report.paragraphs {
                text(b, Role::Paragraph, paragraph);
            }
        }
        match self.state {
            State::Planning => {
                let (draft, _) = goal::draft_of(self.task.spec.specification.words.bytes.bytes());
                let line = match draft {
                    Some(path) => words.say(Word::GoalReadingDraft, &[Hole::Name(copy_of(goal::file_of(&path)))]),
                    None => words.say(Word::GoalReadingAsked, &[]),
                };
                text(b, Role::Paragraph, &line);
                b.open(Role::Figure);
                b.name(&words.say(Word::GoalPlanningTitle, &[]));
                b.style(look::GOAL_TYPING);
                for _ in 0..3_u8 {
                    b.open(Role::Text);
                    if words.shell == Shell::Terminal {
                        b.run(Run::Plain, &words.say(Word::GoalsWayEnd, &[]));
                    }
                    b.close();
                }
                b.close();
            }
            State::Needs { .. } | State::Paused | State::Running | State::Landed | State::Closed | State::Failed => {
                if let Some(report) = self.report {
                    for (word, value) in [(Word::GoalLeft, &report.left), (Word::GoalRisk, &report.risk)] {
                        if let Some(value) = value {
                            b.open(Role::Paragraph);
                            let run = if words.shell == Shell::Browser { Run::Strong } else { Run::Plain };
                            b.open(Role::Text);
                            b.run(run, &words.say(word, &[]));
                            b.run(Run::Plain, value);
                            b.close();
                            b.close();
                        }
                    }
                } else {
                    match self.state {
                        State::Landed | State::Closed | State::Failed => match delegation::facts(self.task).said {
                            Some(delegation::Said::Words(value) | delegation::Said::Merge(value)) => {
                                skein_ui::markdown::read(value, 3, b);
                            }
                            None => {}
                        },
                        State::Needs { .. } | State::Paused | State::Planning | State::Running => {}
                    }
                }
            }
        }
        byline(b, self.report, Word::GoalByAt, words, times);
        b.close();
    }
}

fn fallback(words: &Words, value: Fallback) -> Box<[u8]> {
    match value {
        Fallback::Planning { draft: Some(draft) } => words.say(Word::GoalPlanningDraft, &[Hole::Name(draft)]),
        Fallback::Planning { draft: None } => words.say(Word::GoalPlanningAsked, &[]),
        Fallback::Closed => words.say(Word::GoalClosedHeading, &[]),
        Fallback::Landed { first_line: Some(line) } => line,
        Fallback::Landed { first_line: None } => words.say(Word::GoalLandedLine, &[]),
        Fallback::Failed { reason } => words.say(Word::GoalFailedReason, &[Hole::Name(reason)]),
        Fallback::Tasks(counts) => words
            .say(Word::GoalTasksDone, &[Hole::Number(u64::from(counts.done)), Hole::Number(u64::from(counts.total))]),
    }
}
fn text(b: &mut Builder, role: Role, value: &[u8]) {
    b.open(role);
    b.name(value);
    if role == Role::Text {
        b.run(Run::Plain, value);
    } else {
        b.text(Run::Plain, value);
    }
    b.close();
}
fn byline(b: &mut Builder, report: Option<&Report>, word: Word, words: &Words, times: &mut Times) {
    if let Some(report) = report
        && let Some(at) = report.at
    {
        b.open(Role::Paragraph);
        b.style(look::GOAL_BYLINE);
        b.text(Run::Plain, &words.say(word, &[Hole::Name(times.say(words, TimeForm::Clock, at))]));
        b.close();
    }
}
fn chat_link(b: &mut Builder, task: u64, label: &[u8], bindings: &mut Bindings) {
    crate::link(b, label, bindings.insert(Act::Go { address: Address::Chat { task } }));
    b.style(look::GOAL_BTN);
    b.style(look::GOAL_SMALL);
    b.text(Run::Plain, label);
    b.close();
}

fn section(b: &mut Builder, title: Word, words: &Words) {
    b.open(Role::Region);
    b.name(&words.say(title, &[]));
    b.style(look::GOAL_SECTION);
    b.open(Role::Heading { level: 2 });
    b.style(look::GOAL_TITLE);
    b.text(Run::Plain, &words.say(title, &[]));
    b.close();
}
fn facts(b: &mut Builder, value: &[u8]) {
    b.open(Role::Paragraph);
    b.style(look::GOAL_FACTS);
    b.text(Run::Plain, value);
    b.close();
}

#[derive(Debug)]
struct Delegate<'a> {
    chip: &'a c::Chip,
    ended: Option<Wall>,
    line: Option<goal::Line>,
}
struct Delegates<'a> {
    rows: Map<u64, Delegate<'a>>,
    more: bool,
}
impl<'a> Delegates<'a> {
    fn of(root: &'a Domain) -> Self {
        let open = match root.pages().inbox() {
            Some(inbox) => root.pages().needs().open(inbox),
            None => Box::default(),
        };
        let mut rows = Map::uncapped();
        let mut more = false;
        if let Some(page) = root.pages().task() {
            if let Some(ended) = &page.ended
                && let Some(page) = &ended.page
                && let c::EndedGroup::Rows { rows: ended, after, cut } = &page.ended
            {
                more = after.is_some() || *cut;
                for row in ended {
                    rows.insert(
                        row.chip.task,
                        Delegate { chip: &row.chip, ended: row.ended_at, line: delegate_line(&row.chip, &open) },
                    )
                    .expect("delivered delegate rows fit");
                }
            }
            if let Some(document) = root.client().document(page.tree)
                && let Some(Latest::Jig(c::Delivery::Tree(tree))) = document.latest
            {
                for row in &tree.rows {
                    if row.parent == page.number {
                        rows.insert(
                            row.chip.task,
                            Delegate {
                                chip: &row.chip,
                                ended: row.chip.details.details.ended_at,
                                line: delegate_line(&row.chip, &open),
                            },
                        )
                        .expect("delivered delegate rows fit");
                    }
                }
            }
        }
        Self { rows, more }
    }

    fn draw(&self, b: &mut Builder, state: State, words: &Words, bindings: &mut Bindings, times: &mut Times) {
        let mut total = 0_u64;
        let mut landed = 0_u64;
        for (_, row) in &self.rows {
            if row.line.is_some() {
                total = total.checked_add(1).expect("delegate count fits");
            }
            if row.line == Some(goal::Line::Landed) {
                landed = landed.checked_add(1).expect("delegate count fits");
            }
        }
        if total == 0 {
            let word = if state == State::Planning { Word::GoalProposing } else { Word::GoalNoTasks };
            facts(b, &words.say(word, &[]));
        } else {
            b.open(match words.shell {
                Shell::Browser => Role::Figure,
                Shell::Terminal => Role::Image,
            });
            b.style(look::GOAL_BAR);
            b.name(&words.say(Word::GoalBarCount, &[Hole::Number(landed), Hole::Number(total)]));
            for (_, row) in &self.rows {
                if let Some(line) = row.line {
                    b.open(Role::Text);
                    match line {
                        goal::Line::Landed => b.style(look::GOAL_BAR_LANDED),
                        goal::Line::Running => b.style(look::GOAL_BAR_RUNNING),
                        goal::Line::Done
                        | goal::Line::WaitsOn(_)
                        | goal::Line::WaitsOnTasks(_)
                        | goal::Line::WaitsOnYou
                        | goal::Line::WaitsOnResources
                        | goal::Line::Held
                        | goal::Line::Failed
                        | goal::Line::Waits => {}
                    }
                    b.close();
                }
            }
            b.close();
            for (_, row) in &self.rows {
                if let Some(line) = row.line
                    && line != goal::Line::Landed
                {
                    self.row(b, row, words, bindings, times);
                }
            }
            if landed != 0 {
                b.open(Role::Disclosure);
                b.style(look::GOAL_LANDED_FOLD);
                b.name(&words.say(Word::GoalFoldCount, &[Hole::Number(landed)]));
                for (_, row) in &self.rows {
                    if row.line == Some(goal::Line::Landed) {
                        self.row(b, row, words, bindings, times);
                    }
                }
                b.close();
            }
        }
        if self.more {
            facts(b, &words.say(Word::GoalMoreTasks, &[]));
        }
    }

    fn row(&self, b: &mut Builder, row: &Delegate<'_>, words: &Words, bindings: &mut Bindings, times: &mut Times) {
        b.open(Role::Paragraph);
        b.style(look::GOAL_LINE);
        if row.line == Some(goal::Line::Running) {
            b.style(look::GOAL_LINE_RUNNING);
        }
        let title = row.chip.title.bytes();
        crate::link(b, title, bindings.insert(Act::Go { address: Address::Task { task: row.chip.task } }));
        b.text(Run::Plain, title);
        b.close();
        let said = match row.line {
            Some(goal::Line::Landed) => match row.ended {
                Some(at) => Some(times.say(words, TimeForm::Clock, at)),
                None => None,
            },
            Some(line) => Some(self.line_words(words, line, row.ended, times)),
            None => None,
        };
        if let Some(said) = said {
            if words.shell == Shell::Terminal {
                text(b, Role::Text, &words.say(Word::ToolSeparator, &[]));
            }
            b.open(Role::Text);
            if row.line == Some(goal::Line::Held) {
                b.style(look::GOAL_LINE_NEEDS);
            }
            b.run(Run::Plain, &said);
            b.close();
        }
        b.close();
    }

    fn line_words(&self, words: &Words, line: goal::Line, ended: Option<Wall>, times: &mut Times) -> Box<[u8]> {
        match line {
            goal::Line::Running => words.say(Word::Running, &[]),
            goal::Line::WaitsOn(task) => match self.rows.get(&task) {
                Some(row) if row.line.is_some() => {
                    words.say(Word::GoalWaitsTitle, &[Hole::Name(copy_of(row.chip.title.bytes()))])
                }
                Some(_) | None => words.say(Word::GoalWaitsTask, &[Hole::Number(task)]),
            },
            goal::Line::WaitsOnTasks(count) => words.say(Word::GoalWaitsTasks, &[Hole::Number(u64::from(count))]),
            goal::Line::WaitsOnYou => words.say(Word::GoalWaitsYou, &[]),
            goal::Line::WaitsOnResources => words.say(Word::GoalWaitsResources, &[]),
            goal::Line::Held => words.say(Word::PhaseHeld, &[]),
            goal::Line::Failed => words.say(Word::GoalFailed, &[]),
            goal::Line::Waits => words.say(Word::GoalWaits, &[]),
            goal::Line::Landed => words.say(Word::GoalLandedAlone, &[]),
            goal::Line::Done => {
                let at = match ended {
                    Some(at) => times.say(words, TimeForm::Clock, at),
                    None => Box::default(),
                };
                words.say(Word::GoalDoneAt, &[Hole::Name(at)]).trim_ascii().into()
            }
        }
    }
}

fn landing_words(words: &Words, times: &mut Times, landing: goal::Landing) -> Box<[u8]> {
    match landing {
        goal::Landing::Landed(Some(at)) => {
            words.say(Word::GoalLandedAt, &[Hole::Name(times.say(words, TimeForm::Clock, at))])
        }
        goal::Landing::Landed(None) => words.say(Word::GoalLandedLine, &[]),
        goal::Landing::Closed => words.say(Word::GoalClosedBefore, &[]),
        goal::Landing::Failed => words.say(Word::GoalFailedBefore, &[]),
        goal::Landing::Whole { total: 0 } => words.say(Word::GoalWholeNoTasks, &[]),
        goal::Landing::Whole { total } => words.say(Word::GoalWholeCount, &[Hole::Name(count_words(words, total))]),
        goal::Landing::Each => words.say(Word::GoalEachChecked, &[]),
    }
}
fn count_words(words: &Words, total: u32) -> Box<[u8]> {
    let small = [
        Word::GoalOne,
        Word::GoalTwo,
        Word::GoalThree,
        Word::GoalFour,
        Word::GoalFive,
        Word::GoalSix,
        Word::GoalSeven,
        Word::GoalEight,
        Word::GoalNine,
    ];
    let word = match total.checked_sub(1) {
        Some(index) => small.get(usize::try_from(index).expect("count fits")),
        None => None,
    };
    match word {
        Some(word) => words.say(*word, &[]),
        None => words.say(Word::GoalCountDigits, &[Hole::Number(u64::from(total))]),
    }
}

fn delegate_line(chip: &c::Chip, open: &[&Card]) -> Option<goal::Line> {
    let mut yours = false;
    for card in open {
        if super::inbox::source_task(card.object) == Some(chip.task) {
            yours = true;
        }
    }
    goal::line(chip, yours)
}

/// The latest page is newest first; a goal keeps only the reader's words (5.8).
fn sent(b: &mut Builder, root: &Domain, page: &temper_client_domain::pages::task::Task, words: &Words) {
    let Some(me) = root.pages().me(root.client()) else { return };
    let mut lines = List::uncapped();
    if let Some(latest) = &page.latest {
        for row in latest.rows.iter().rev() {
            match &row.source.line {
                c::Activity::Words { by: c::Requester::Person(person), words, .. } if *person == me.party.number => {
                    lines.push((words.bytes(), false)).expect("latest activity words fit");
                }
                c::Activity::Words { .. }
                | c::Activity::Heard(_)
                | c::Activity::Notified { .. }
                | c::Activity::RunBegan { .. }
                | c::Activity::RunEnded { .. }
                | c::Activity::Turn { .. }
                | c::Activity::Decided { .. }
                | c::Activity::Milestone { .. }
                | c::Activity::NowWritten { .. }
                | c::Activity::Stood { .. }
                | c::Activity::TalkOpened { .. }
                | c::Activity::TalkEnded { .. }
                | c::Activity::SessionEnded { .. }
                | c::Activity::Kept { .. } => {}
            }
        }
    }
    for pending in &page.thread.words.rows {
        if pending.state == jig_client::thread::words::Words::BeingSent
            && let Some(by) = &pending.by
            && by.number == me.party.number
        {
            lines.push((pending.words.bytes(), true)).expect("pending words fit");
        }
    }
    if lines.is_empty() {
        return;
    }
    b.open(Role::Slot);
    b.style(look::GOAL_SENT);
    for (line, pending) in &lines {
        b.open(Role::Paragraph);
        b.style(look::GOAL_YOU);
        b.text(Run::Plain, line);
        b.close();
        if *pending {
            words.status(b, crate::cards::P::WordsBeingSent, &[]);
        }
    }
    b.close();
}

fn composer(
    b: &mut Builder,
    root: &Domain,
    page: &temper_client_domain::pages::task::Task,
    task: &c::Task,
    words: &Words,
    bindings: &mut Bindings,
) {
    if super::chats::ended(&task.chip) || task.result.is_some() {
        return;
    }
    let Some(admission) = jig_client::facts::offer(task.acts.as_slice(), c::ActKind::Message) else { return };
    match admission {
        c::Admission::Allowed | c::Admission::Bounded { .. } => {}
        c::Admission::Refused(c::Refusal::NotWaiting) => return,
        c::Admission::Refused(_) | c::Admission::Proposed { .. } => {
            super::task::act(b, task, c::ActKind::Message, Word::Message, Act::Send, words, bindings);
            return;
        }
    }
    let over = match root.pages().me(root.client()) {
        Some(me) => temper_client_domain::pages::composer::over(&page.words, page.beyond, me.words_bytes),
        None => 0,
    };
    let name = words.say(Word::GoalTell, &[Hole::Name(copy_of(task.chip.title.bytes()))]);
    b.open(Role::Slot);
    b.style(look::GOAL_COMPOSER_BOX);
    b.open(Role::Slot);
    b.style(look::GOAL_COMPOSER);
    b.open(Role::TextArea);
    b.name(&name);
    if page.composer_focus != 0 && root.pages().switcher().renaming().is_none() {
        b.focus(page.composer_focus);
    }
    b.placeholder(&name);
    if over == 0 && crate::choice::can_send(root) {
        b.state(UiState::SubmitsOnEnter);
    } else if over != 0 {
        bindings.block_submissions();
    }
    b.value(&page.words, temper_client_domain::pages::shown(root.pages()));
    b.bind(bindings.insert(Act::MessageWords { words: Box::default() }));
    b.close();
    b.open(Role::Button);
    b.name(&words.say(Word::Send, &[]));
    if page.words.is_empty() || over != 0 || !crate::choice::can_send(root) {
        b.state(UiState::Disabled);
    }
    b.bind(bindings.insert(Act::Send));
    b.close();
    b.close();
    super::session::over(b, words, over);
    b.open(Role::Paragraph);
    b.style(look::GOAL_HINT);
    b.text(Run::Plain, &words.say(Word::GoalHint, &[]));
    b.close();
    b.close();
}
