//! Goal board, factual funding and typed form controls (pages.md, 6.2).
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use crate::{JigBindings, JigMeaning, tree::cards};
use alloc::boxed::Box;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, State};

use crate::{
    Borrowed,
    state::{action::Act, fields::Purpose, page::project::goals::GoalForm},
};
use jig_client::contract::{Admission, ObjectKey, OwnerExecutor, Project, TaskLine};
/// The one typed form property changed by an input's complete value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// Whole funding amount.
    Budget,
    /// Optional wall deadline as UTC year-month-day and hour-minute.
    Deadline,
    /// Lower numbers place the goal earlier.
    Priority,
}
/// Decode one form field without deriving admission or changing other choices.
#[must_use]
pub fn edit(form: GoalForm, field: Field, value: &[u8]) -> Option<Act> {
    let mut form = form;
    match field {
        Field::Budget => form.budget = skein_ui::number(value)?,
        Field::Priority => form.priority = u32::try_from(skein_ui::number(value)?).ok()?,
        Field::Deadline => {
            form.deadline = if value.is_empty() { None } else { Some(crate::state::clock::deadline(value)?) }
        }
    }
    Some(Act::GoalForm { form })
}
pub(super) fn goals(b: &mut Builder, client: &Borrowed<'_>, doc: &Project, words: &Words, bindings: &mut JigBindings) {
    if !available(b, client, doc.number, words) {
        return;
    }
    let project_page = client.project_page();
    words.text(b, JigPhrase::PriorityMeaning, &[]);
    b.open(Role::List);
    for goal in &project_page.listed_goals {
        line(b, client, goal, words, bindings);
    }
    b.close();
    if project_page.goals_more {
        words.status(b, JigPhrase::MoreRows, &[]);
    }
    crate::tree::pages::cut(b, words, project_page.ended_cut);
    crate::tree::pages::work::cursors(
        b,
        words,
        bindings,
        project_page.ended_before.as_ref(),
        project_page.ended_after.as_ref(),
        crate::state::page::project::GOALS_WINDOW,
    );
    form(b, client, doc.number, words, bindings);
}
pub(super) fn form(b: &mut Builder, client: &Borrowed<'_>, project: u32, words: &Words, bindings: &mut JigBindings) {
    let text_limit = bindings.text_limit();
    b.open(Role::Form);
    b.region(crate::regions::JigRegion::NewGoal.region());
    b.name(&words.say(JigPhrase::GoalWords, &[]));
    let state = &client.project_page().form;
    let mut offered = false;
    if let Some(me) = client.frame().document {
        for membership in &me.projects {
            if membership.project == project
                && let Some(creating) = &membership.creating
            {
                offered = true;
                let left = creating.within.pool_left;
                words.text(b, JigPhrase::PoolLeft, &[Hole::Amount(left)]);
                cards::common::field(b, client, Purpose::NewGoal, JigPhrase::GoalWords, words, bindings);
                for charter in &creating.within.charters {
                    choice(b, client, words, bindings, project, OwnerExecutor::Agent { charter: *charter }, left);
                }
                for procedure in &creating.within.procedures {
                    choice(
                        b,
                        client,
                        words,
                        bindings,
                        project,
                        OwnerExecutor::Procedure { connector: procedure.connector, code: procedure.code },
                        left,
                    );
                }
                inputs(b, client, project, words, bindings, left);
            }
        }
    }
    if !offered {
        words.refusal(b, &jig_client::contract::Refusal::NotOffered, text_limit);
    }
    if let Some(submission) = &state.pending {
        b.open(Role::Status);
        b.name(&words.say(JigPhrase::ActSetGoal, &[]));
        b.state(State::Busy);
        if let Some(intent) = client.intents().get(skein_lib::Id::from_token(submission.request)) {
            words.progress(b, intent.progress);
        }
        b.close();
    }
    if let Some(why) = &state.refusal {
        words.refusal(b, why, text_limit);
    }
    if let Some(answer) = &state.answer {
        words.outcome(b, answer);
    }
    b.close();
}
pub(super) fn choice(
    b: &mut Builder,
    client: &Borrowed<'_>,
    words: &Words,
    bindings: &mut JigBindings,
    project: u32,
    executor: OwnerExecutor,
    pool_left: u64,
) {
    let form = match client.project_page().form.form {
        Some(form) if form.project == project => GoalForm { executor, ..form },
        Some(_) | None => GoalForm { project, executor, budget: pool_left, deadline: None, priority: 0 },
    };
    match executor {
        OwnerExecutor::Agent { charter: number } => crate::tree::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::Charter,
            &[Hole::Number(u64::from(number))],
            Act::GoalForm { form },
        ),
        OwnerExecutor::Procedure { connector, code } => crate::tree::frame::control(
            b,
            words,
            bindings,
            Role::Button,
            JigPhrase::CreateProcedure,
            &[Hole::Number(u64::from(connector)), Hole::Number(u64::from(code))],
            Act::GoalForm { form },
        ),
        OwnerExecutor::Person(_) => {}
    }
}

fn inputs(
    b: &mut Builder,
    client: &Borrowed<'_>,
    project: u32,
    words: &Words,
    bindings: &mut JigBindings,
    pool_left: u64,
) {
    let text_limit = bindings.text_limit();
    let state = &client.project_page().form;
    if let Some(form) = state.form {
        if form.project != project {
            return;
        }
        for (field, phrase) in [
            (Field::Budget, JigPhrase::GoalBudget),
            (Field::Deadline, JigPhrase::GoalDeadline),
            (Field::Priority, JigPhrase::GoalPriority),
        ] {
            b.open(Role::TextField);
            b.name(&words.say(phrase, &[]));
            match field {
                Field::Budget => b.value(skein_lib::Decimal::of(form.budget).as_bytes(), state.writes),
                Field::Priority => {
                    b.value(skein_lib::Decimal::of(u64::from(form.priority)).as_bytes(), state.writes);
                }
                Field::Deadline => match form.deadline {
                    Some(at) => b.value(&date(at), state.writes),
                    None => b.value(&[], state.writes),
                },
            }
            b.bind(bindings.insert(JigMeaning::GoalField { form, field }));
            b.close();
        }
        words.text(b, JigPhrase::Budget, &[Hole::Amount(form.budget)]);
        words.text(b, JigPhrase::Priority, &[Hole::Number(u64::from(form.priority))]);
        if let Some(deadline) = form.deadline {
            words.deadline(b, deadline);
        }
        if let Some(me) = client.frame().document {
            for membership in &me.projects {
                if membership.project == project
                    && let Some(creating) = &membership.creating
                    && let Some(template) = crate::state::queries::goal_template(me, project)
                {
                    match crate::state::page::project::goals::fall(
                        creating,
                        template,
                        form,
                        jig_client::contract::Text::new(Box::from([]), text_limit).expect("empty draft fits"),
                    ) {
                        jig_client::shapes::Fall::Proposable(to) => {
                            words.text(
                                b,
                                if form.budget > pool_left { JigPhrase::GoalPropose } else { JigPhrase::WillPropose },
                                &[],
                            );
                            cards::common::recipient(b, &to, words);
                        }
                        jig_client::shapes::Fall::Refused(why) => words.refusal(b, &why, text_limit),
                        jig_client::shapes::Fall::NotOffered(_) => {
                            words.refusal(b, &jig_client::contract::Refusal::NotOffered, text_limit);
                        }
                        jig_client::shapes::Fall::Within => {}
                    }
                }
            }
        }
        crate::tree::pages::work::control(
            b,
            words,
            bindings,
            JigPhrase::ActSetGoal,
            Act::Submit { field: Purpose::NewGoal },
            false,
        );
    }
}
pub(super) fn line(b: &mut Builder, client: &Borrowed<'_>, goal: &TaskLine, words: &Words, bindings: &mut JigBindings) {
    b.open(Role::Item);
    b.key(crate::tree::pages::object_key(ObjectKey::Task(goal.chip.task)));
    cards::chip::chip(b, client, &goal.chip, words, bindings);
    if let Some(id) = client.objects().find(ObjectKey::Task(goal.chip.task)) {
        cards::card(b, client, id, &cards::Place::Elsewhere, words, bindings);
    }
    if let Some(priority) = goal.tracked {
        words.text(b, JigPhrase::Priority, &[Hole::Number(u64::from(priority))]);
    }
    crate::tree::pages::work::owner_counts(b, goal.phases, words);
    arrows(b, client, goal, words, bindings);
    words.text(b, JigPhrase::HeldBelow, &[Hole::Number(u64::from(goal.held_below))]);
    crate::tree::fragments::budget::render(b, &goal.budget, words);
    b.close();
}

pub(super) fn date(at: skein_lib::Wall) -> [u8; 16] {
    let seconds = at.as_secs();
    let date = skein_ui::date::format(skein_ui::date::civil(at).expect("wall dates fit the calendar"));
    let mut bytes = *b"0000-00-00 00:00";
    for (index, byte) in date.iter().enumerate() {
        *bytes.get_mut(index).expect("fixed date byte") = *byte;
    }
    for (number, start, width) in [((seconds % 86400) / 3600, 11_usize, 2_usize), ((seconds % 3600) / 60, 14, 2)] {
        let decimal = skein_lib::Decimal::of(number);
        let value = decimal.as_bytes();
        let end = start.checked_add(width).expect("fixed field");
        let begin = end.checked_sub(value.len()).expect("date field fits");
        for (offset, byte) in value.iter().enumerate() {
            let index = begin.checked_add(offset).expect("fixed date offset");
            *bytes.get_mut(index).expect("fixed date byte") = *byte;
        }
    }
    bytes
}

fn arrows(b: &mut Builder, client: &Borrowed<'_>, goal: &TaskLine, words: &Words, bindings: &mut JigBindings) {
    if goal.ended_at.is_some() {
        return;
    }
    match crate::state::page::project::moving::admission(client.ui) {
        Admission::Refused(_) => return,
        Admission::Allowed | Admission::Bounded { .. } | Admission::Proposed { .. } => {}
    }
    let rows = client.project_page().goals;
    let Some(index) = crate::state::page::project::moving::index(&rows, goal.chip.task) else { return };
    for (up, phrase, offered) in [
        (true, JigPhrase::MoveGoalUp, index > 0),
        (false, JigPhrase::MoveGoalDown, index.saturating_add(1) < rows.len()),
    ] {
        if offered {
            crate::tree::frame::control(
                b,
                words,
                bindings,
                Role::Button,
                phrase,
                &[],
                Act::MoveGoal { task: goal.chip.task, up },
            );
        }
    }
}

pub(super) fn available(b: &mut Builder, client: &Borrowed<'_>, project: u32, words: &Words) -> bool {
    if let Some(me) = client.frame().document
        && crate::state::queries::goal_template(me, project).is_some()
    {
        return true;
    }
    words.text(b, JigPhrase::NoGoalTemplate, &[]);
    false
}
