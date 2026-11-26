//! A milestone's page (temper/docs/design/client/pages/milestone.md, 3).
use super::{
    goal,
    goals::{self, Names},
    milestones,
};
use crate::{
    bindings::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use core::cmp::Reverse;
use jig_client::contract as c;
use skein_lib::{List, Map, bytes::copy_of};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act,
        milestone::Page,
        milestones::{self as ms, Count},
        needs,
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
    if let Some(task_page) = root.pages().task()
        && let Some(task) = task_page.document(root.client())
        && let Some(page) = &task_page.milestone
    {
        let bands = goal::open_bands(root, words.shell);
        let mut names = Names::dependencies(task, root.pages().me(root.client()));
        for line in &page.milestone.goals {
            names.add(line);
        }
        let shown = Shown { root, task, page, names, needs: !bands.is_empty(), needing: needing(root, page) };
        shown.header(b, words, bindings, times);
        let mut question = false;
        for need in &bands {
            match need {
                needs::Need::Card(card) => {
                    let first = !question;
                    question |= card.question_words().is_some();
                    goal::band_named(b, root, task, card, words, bindings, first, band_goal(root, page, card.object));
                }
                needs::Need::Approval(row) => {
                    super::approval::band(b, row, super::approval::viewer(root), words, bindings, sign_in);
                }
            }
        }
        b.open(Role::Slot);
        b.style(look::GOAL_TWO);
        shown.goals(b, words, bindings, times);
        shown.side(b, words);
        b.close();
    }
    crate::confirm::main(b, root, words, bindings, rows);
    b.close();
}

fn band_goal<'a>(root: &Domain, page: &'a Page, object: c::ObjectKey) -> Option<&'a [u8]> {
    let source = super::inbox::source_task(object)?;
    let selection = root.pages().inbox()?.selection.as_ref()?;
    for selected in selection {
        let task = match selected.cursor {
            c::InboxCursor::Inbox { task, .. } => Some(task),
            c::InboxCursor::Number { .. } => None,
        };
        if task == Some(source) {
            for line in &page.milestone.goals {
                if selected.goals.as_slice().contains(&line.chip.task) {
                    return Some(line.chip.title.bytes());
                }
            }
        }
    }
    None
}
fn needing(root: &Domain, page: &Page) -> List<u64> {
    let mut tasks = List::uncapped();
    if let Some(inbox) = root.pages().inbox()
        && let Some(selection) = &inbox.selection
    {
        for line in &page.milestone.goals {
            for selected in selection {
                if selected.goals.as_slice().contains(&line.chip.task) {
                    tasks.push(line.chip.task).expect("held goals fit");
                    break;
                }
            }
        }
    }
    tasks
}

struct Shown<'a> {
    root: &'a Domain,
    task: &'a c::Task,
    page: &'a Page,
    names: Names<'a>,
    needs: bool,
    needing: List<u64>,
}
impl Shown<'_> {
    fn count(&self) -> Option<Count> {
        if self.page.line.is_some() && self.page.milestone.known(self.root.client()) {
            Some(ms::count(self.page.milestone.goals.as_slice(), self.page.milestone.totals.as_ref()))
        } else {
            None
        }
    }
    fn header(&self, b: &mut Builder, words: &Words, bindings: &mut Bindings, times: &mut Times) {
        b.open(Role::Banner);
        b.style(look::GOAL_HEAD);
        self.meta(b, words, times);
        if self.count().is_some()
            && let Some(line) = &self.page.line
        {
            let headline = ms::headline(
                line,
                self.page.milestone.goals.as_slice(),
                self.page.milestone.totals.as_ref(),
                self.needs,
            );
            let report = match headline.case {
                ms::HeadlineCase::Running { goal, .. } => self.page.headline(goal),
                ms::HeadlineCase::Count(_)
                | ms::HeadlineCase::Complete(_)
                | ms::HeadlineCase::Closed(_)
                | ms::HeadlineCase::Open(_)
                | ms::HeadlineCase::Ended { .. } => None,
            };
            goal::text(
                b,
                Role::Heading { level: 1 },
                &milestones::headline(
                    words,
                    headline,
                    self.page.milestone.goals.as_slice(),
                    report.as_deref(),
                    &self.names,
                ),
            );
        }
        self.acts(b, words, bindings);
        b.close();
    }
    fn meta(&self, b: &mut Builder, words: &Words, times: &mut Times) {
        let mut items = List::uncapped();
        let name = match &self.page.milestone.facts {
            Some(facts) => facts.name.as_ref(),
            None => self.task.chip.title.bytes(),
        };
        items.push(copy_of(name)).expect("meta fits");
        let mut token = None;
        if let Some(count) = self.count()
            && let Some(line) = &self.page.line
        {
            let (word, style) = milestones::state_word(
                words,
                ms::state(line, self.page.milestone.goals.as_slice(), self.page.milestone.totals.as_ref(), self.needs),
            );
            token = Some(style);
            items.push(word).expect("meta fits");
            items
                .push(words.say(Word::MsMetaCount, &[Hole::Number(count.done), Hole::Number(count.total)]))
                .expect("meta fits");
        }
        if let Some(line) = &self.page.line
            && let Some(totals) = &self.page.milestone.totals
            && line.budget.numbers.budget != 0
        {
            items
                .push(words.say(Word::MsSpent, &[Hole::Number(totals.spent), Hole::Number(line.budget.numbers.budget)]))
                .expect("meta fits");
        }
        items
            .push(words.say(Word::GoalSince, &[Hole::Name(times.say(words, TimeForm::Clock, self.task.made))]))
            .expect("meta fits");
        goal::meta(b, items.as_slice(), token, words);
    }
    fn acts(&self, b: &mut Builder, words: &Words, bindings: &mut Bindings) {
        if self.task.chip.details.phase != c::ChipPhase::Ended {
            b.open(Role::Slot);
            b.style(look::GOAL_ACTS);
            if temper_client_domain::pages::goals::state(&self.task.chip, self.needs, false, 0)
                == temper_client_domain::pages::goals::State::Paused
            {
                super::task::act(b, self.task, c::ActKind::Release, Word::Resume, Act::Resume, words, bindings);
            } else {
                super::task::act(b, self.task, c::ActKind::Stop, Word::Pause, Act::Pause, words, bindings);
            }
            super::task::act(b, self.task, c::ActKind::Cancel, Word::Close, Act::Close, words, bindings);
            b.close();
        }
    }
    fn goals(&self, b: &mut Builder, words: &Words, bindings: &mut Bindings, times: &mut Times) {
        b.open(Role::Region);
        b.name(&words.say(Word::Goals, &[]));
        b.style(look::GOAL_REPORT);
        title(b, &words.say(Word::Goals, &[]));
        let mut ended = Map::uncapped();
        let mut counts = [0_u64; 3];
        for line in &self.page.milestone.goals {
            if line.chip.details.phase == c::ChipPhase::Ended {
                let group = match &line.chip.details.details.ending {
                    Some(c::ChipEnding::Done(_)) => 0,
                    Some(c::ChipEnding::Failed(_)) | None => 1,
                    Some(c::ChipEnding::Cancelled { .. }) => 2,
                };
                let count = counts.get_mut(group).expect("three endings");
                *count = count.checked_add(1).expect("held goals fit");
                ended.insert((Reverse(line.ended_at), line.chip.task), (group, line)).expect("held goals fit");
            } else {
                self.row(b, line, words, bindings, times, true);
            }
        }
        for (group, word) in [Word::MsFoldDone, Word::MsFoldFailed, Word::MsFoldClosed].iter().enumerate() {
            let count = *counts.get(group).expect("three endings");
            if count != 0 {
                b.open(Role::Disclosure);
                b.style(look::GOAL_LANDED_FOLD);
                b.name(&words.say(*word, &[Hole::Number(count)]));
                for (_, (ending, line)) in &ended {
                    if *ending == group {
                        self.row(b, line, words, bindings, times, false);
                    }
                }
                b.close();
            }
        }
        if self.page.milestone.goals.is_empty()
            && !self.page.milestone.more
            && let Some(Count { total: 0, .. }) = self.count()
        {
            let whole = match &self.page.milestone.facts {
                Some(facts) => facts.whole.is_some(),
                None => false,
            };
            goal::facts(b, &words.say(if whole { Word::MsNoGoalsWhole } else { Word::MsNoGoals }, &[]));
        }
        if self.page.milestone.more {
            goal::facts(b, &words.say(Word::MsMoreGoals, &[]));
        }
        b.close();
    }
    fn row(
        &self,
        b: &mut Builder,
        line: &c::TaskLine,
        words: &Words,
        bindings: &mut Bindings,
        times: &mut Times,
        active: bool,
    ) {
        let report = self.page.headline(line.chip.task);
        let state = temper_client_domain::pages::goals::state(
            &line.chip,
            self.needing.as_slice().contains(&line.chip.task),
            report.is_some(),
            line.delegates_total,
        );
        b.open(Role::Slot);
        b.style(look::MS_GOAL);
        b.open(Role::Paragraph);
        b.style(look::GOAL_LINE);
        let title = line.chip.title.bytes();
        crate::bindings::link(b, title, bindings.insert(Act::Go { address: Address::Task { task: line.chip.task } }));
        b.text(Run::Plain, title);
        b.close();
        let (word, token) = goals::state_word(words, state, &self.names, &line.chip);
        let word = if !active && let Some(at) = line.ended_at {
            words.say(Word::MsEndedAt, &[Hole::Name(word), Hole::Name(times.say(words, TimeForm::Clock, at))])
        } else {
            word
        };
        if words.shell == Shell::Terminal {
            goal::text(b, Role::Text, &words.say(Word::ToolSeparator, &[]));
        }
        b.open(Role::Text);
        b.style(token);
        b.run(Run::Plain, &word);
        b.close();
        b.close();
        if active {
            let headline = match report {
                Some(headline) => headline,
                None => goals::headline(words, line, state, None, &self.names),
            };
            goal::facts(b, &headline);
            if let Some(wait) = &line.wait {
                self.goal_wait(b, words, times, line.chip.task, wait);
            }
        }
        b.close();
    }
    fn goal_wait(&self, b: &mut Builder, words: &Words, times: &mut Times, goal: u64, wait: &c::Wait) {
        let reader = match self.root.pages().me(self.root.client()) {
            Some(me) => Some(me.party.number),
            None => None,
        };
        let on_box = match self.root.forge().deployment() {
            Some(answer) => answer.owned,
            None => false,
        };
        let text = words.wait(wait, times, reader, &[], on_box, false);
        b.open(Role::Paragraph);
        b.style(look::GOAL_WAIT);
        if wait.task == goal {
            b.text(Run::Plain, &text);
        } else {
            let prefix = words.say(Word::WaitInDescendant, &[Hole::Number(wait.task)]);
            b.text(Run::Plain, &skein_lib::bytes::concat(&[&prefix, &text]));
        }
        b.close();
    }
    fn side(&self, b: &mut Builder, words: &Words) {
        b.open(Role::Complementary);
        b.style(look::GOAL_SIDE);
        if words.shell == Shell::Browser {
            self.strip(b, words);
        }
        if let Some(facts) = &self.page.milestone.facts {
            if let Some(value) = &facts.words {
                goal::section(b, Word::MsPurpose, words);
                goal::facts(b, value);
                b.close();
            }
            goal::section(b, Word::GoalLandingTitle, words);
            goal::facts(b, &words.say(Word::MsLanding, &[]));
            b.close();
            goal::section(b, Word::MsItsGoals, words);
            goal::facts(b, &words.say(if facts.whole.is_some() { Word::MsWholeGoals } else { Word::MsOpenGoals }, &[]));
            let maker = match facts.maker {
                c::Requester::Person(person) => {
                    let reader = self.root.pages().me(self.root.client());
                    if match reader {
                        Some(me) => me.party.number == person,
                        None => false,
                    } {
                        words.say(Word::MsMakerYou, &[])
                    } else {
                        words.say(Word::MsMakerPerson, &[Hole::Number(person)])
                    }
                }
                c::Requester::Task(task) => words.say(Word::MsMakerTask, &[Hole::Number(task)]),
                c::Requester::Deployment { .. } => words.say(Word::MsMakerDeployment, &[]),
            };
            goal::facts(b, &words.say(Word::MsMadeBy, &[Hole::Name(maker)]));
            b.close();
        }
        if words.shell == Shell::Terminal {
            self.strip(b, words);
        }
        b.close();
    }
    fn strip(&self, b: &mut Builder, words: &Words) {
        if let Some(count) = self.count() {
            b.open(Role::Region);
            b.style(look::GOAL_SECTION);
            title(b, &words.say(Word::GoalsBarCount, &[Hole::Number(count.done), Hole::Number(count.total)]));
            milestones::strip(b, words, self.page.milestone.goals.as_slice(), self.needing.as_slice(), count);
            b.close();
        }
    }
}
fn title(b: &mut Builder, value: &[u8]) {
    b.open(Role::Heading { level: 2 });
    b.style(look::GOAL_TITLE);
    b.text(Run::Plain, value);
    b.close();
}
