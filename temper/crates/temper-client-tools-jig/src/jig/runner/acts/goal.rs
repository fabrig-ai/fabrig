//! Template controls precede jig's admission (client/ctl/areas.md and client/ctl/milestones.md, 3).
use super::{Call, Failed, Limits, Prepared, Request, c, choice, empty, number, shapes, value, words};
use crate::jig::call::{Value, Values};
use crate::jig::render::Text;
use skein_lib::{List, Wall};

/// Apply ctl controls, then use the offered template and jig judgement.
pub(super) fn start(call: &Call, me: &c::Me, limits: &Limits) -> Prepared {
    prepare(call, me, limits, None)
}

pub(super) fn beneath(call: &Call, document: &c::ReadValue, me: &c::Me, limits: &Limits, now: Wall) -> Prepared {
    let project = super::super::reads::project(call, me).ok_or(Failed::Fixed(b"unknown project"))?;
    let number = number(call, 11).expect("milestone argument");
    let milestone = match super::render::read_task(document) {
        Some((task, _))
            if task.chip.task == number
                && task.project == project
                && temper_client_domain_forge::milestones::is(&task.chip) =>
        {
            task
        }
        Some(_) | None => return Err(milestone_error(number, project, None, false)),
    };
    if let Some(concluded) = milestone.concluded {
        return Err(milestone_error(number, project, Some(concluded.at), false));
    }
    if value(call, 13).is_some()
        && let Some(deadline) = milestone.budget.deadline
    {
        return Err(milestone_error(number, project, Some(deadline), true));
    }
    prepare(call, me, limits, Some((milestone, now)))
}

fn milestone_error(number: u64, project: u32, at: Option<Wall>, deadline: bool) -> Failed {
    let mut measure = Text::measure();
    milestone_words(number, project, at, deadline, &mut measure);
    let mut out = Text::writing_exact(measure);
    milestone_words(number, project, at, deadline, &mut out);
    Failed::Detail(out.finish())
}

fn milestone_words(number: u64, project: u32, at: Option<Wall>, deadline: bool, out: &mut Text) {
    if deadline {
        out.put(b"goal start: --no-deadline: ");
    }
    match at {
        None => {
            out.put(b"task ");
            out.number(number);
            out.put(b" is not a milestone of project ");
            out.number(u64::from(project));
        }
        Some(at) => {
            out.put(b"milestone ");
            out.number(number);
            out.put(if deadline { b" ends by " } else { b" is whole since " });
            out.number(at.as_nanos());
            if deadline {
                out.put(b"; give --deadline");
            }
        }
    }
}

fn prepare(call: &Call, me: &c::Me, limits: &Limits, milestone: Option<(&c::Task, Wall)>) -> Prepared {
    let project = super::super::reads::project(call, me).ok_or(Failed::Fixed(b"unknown project"))?;
    let number = u32::try_from(number(call, 1).unwrap_or(u64::from(shapes::GOAL))).expect("template fits");
    let template = shapes::template(me, project, number).ok_or(Failed::Fixed(b"goal template not offered"))?;
    let mut offered = me.clone();
    controls(call, project, number, &mut offered)?;
    if value(call, 13).is_some() {
        for membership in &me.projects {
            if membership.project == project
                && let Some(creating) = &membership.creating
                && creating.within.maximum_deadline.is_some()
            {
                return Err(Failed::Fixed(
                    b"goal start: --no-deadline: the project's role ends every task by its ceiling; give --deadline",
                ));
            }
        }
    }
    let executor = executor(call, template)?;
    let llm = choice::selection(call, crate::jig::rows::GOAL_CHOICE, me, project, template.llm.as_ref())?;
    let priority = match super::number(call, 4) {
        Some(priority) => Some(u32::try_from(priority).expect("priority fits")),
        None => None,
    };
    let draft = shapes::GoalDraft {
        words: words(call, 2, limits)?,
        executor,
        budget: super::number(call, 5).unwrap_or(0),
        deadline: match value(call, 7) {
            Some(Value::Text(text)) => crate::jig::argv::deadline(text),
            _ => None,
        },
        priority,
        llm,
    };
    if let Some((milestone, now)) = milestone {
        let mut task = match shapes::fill_goal_from(&offered, project, number, &draft) {
            Ok(task) => task,
            Err(why) => return Err(not_offered(why, me, project, number, executor)),
        };
        narrow(&mut task, milestone, now);
        waits(call, &mut task);
        let mut tasks = List::with_capacity(1);
        tasks.push(task).expect("one goal");
        return Ok((Request::Ask(c::Ask::Delegate { project, task: milestone.chip.task, tasks }), None));
    }
    match shapes::set_goal_from(&offered, project, number, draft, empty(limits)) {
        shapes::Judged::Create(ask) => Ok((Request::Ask(with_waits(call, ask)), None)),
        shapes::Judged::Propose { ask, to } => Ok((Request::Ask(with_waits(call, ask)), Some(to))),
        shapes::Judged::Refused(why) => Err(refused(&why, limits)),
        shapes::Judged::NotOffered(why) => Err(not_offered(why, me, project, number, executor)),
    }
}

fn narrow(task: &mut c::NewTask, milestone: &c::Task, now: Wall) {
    let allowance = &milestone.authority.authority.delegation;
    task.authority.delegation.depth = task.authority.delegation.depth.min(allowance.depth.saturating_sub(1));
    task.authority.delegation.tasks = if task.authority.delegation.depth == 0 {
        0
    } else {
        task.authority.delegation.tasks.min(milestone.tasks_left.saturating_sub(1))
    };
    if let Some(latest) = milestone.budget.deadline {
        let deadline = match task.deadline {
            c::NewDeadline::Named => task.authority.deadline,
            c::NewDeadline::Span(span) => Some(Wall::from_nanos(now.as_nanos().saturating_add(span.as_nanos()))),
        };
        task.authority.deadline = Some(match deadline {
            Some(deadline) => deadline.min(latest),
            None => latest,
        });
        task.deadline = c::NewDeadline::Named;
    }
}

fn with_waits(call: &Call, mut ask: c::Ask) -> c::Ask {
    let (c::Ask::Create { tasks, .. } | c::Ask::Propose { action: c::ProposeAction::Batch { tasks }, .. }) = &mut ask
    else {
        unreachable!("goal creates or proposes a batch")
    };
    waits(call, tasks.get_mut(0).expect("one goal"));
    ask
}

fn waits(call: &Call, task: &mut c::NewTask) {
    let Some(Some(Values::Many(values))) = call.values.get(12) else { return };
    let mut dependencies = List::with_capacity(u32::try_from(values.len()).expect("bounded waits"));
    for value in values {
        let Value::Number(number) = value else { unreachable!("after task number") };
        let dependency = c::TaskRef::Task(*number);
        if !dependencies.as_slice().contains(&dependency) {
            dependencies.push(dependency).expect("one dependency per argument");
        }
    }
    task.dependencies = dependencies;
}

pub(super) fn refused(why: &c::Refusal, limits: &Limits) -> Failed {
    let book = crate::jig::render::document::phrasebook(limits.text);
    let mut measure = Text::measure();
    crate::jig::render::acts::refusal(why, &book, &mut measure);
    let mut out = Text::writing_exact(measure);
    crate::jig::render::acts::refusal(why, &book, &mut out);
    Failed::Detail(out.finish())
}

fn not_offered(why: shapes::NotOffered, me: &c::Me, project: u32, template: u32, executor: c::Executor) -> Failed {
    match why {
        shapes::NotOffered::Project => Failed::Fixed(b"unknown project"),
        shapes::NotOffered::Template => Failed::Fixed(b"goal template not offered"),
        shapes::NotOffered::Ceiling => Failed::Fixed(b"goal budget or deadline exceeds the project's offered ceiling"),
        shapes::NotOffered::Creating => Failed::Fixed(
            b"project has no creating offer; check role and receiving bound documents.lines for charter and procedure lists",
        ),
        shapes::NotOffered::ChipExecutor => {
            let c::Executor::Agent { charter } = executor else { unreachable!("goal templates name charters") };
            let mut charters = &[][..];
            for membership in &me.projects {
                if membership.project == project
                    && let Some(creating) = &membership.creating
                {
                    charters = creating.within.charters.as_slice();
                }
            }
            let mut measure = Text::measure();
            missing_executor_words(template, charter, charters, &mut measure);
            let mut out = Text::writing_exact(measure);
            missing_executor_words(template, charter, charters, &mut out);
            Failed::Detail(out.finish())
        }
    }
}

fn missing_executor_words(template: u32, charter: u32, charters: &[u32], out: &mut Text) {
    out.put(b"template ");
    out.number(u64::from(template));
    out.put(b" requires charter ");
    out.number(u64::from(charter));
    out.put(b", which is not offered; offered charters:");
    if charters.is_empty() {
        out.put(b" none");
    }
    for charter in charters {
        out.put(b" ");
        out.number(u64::from(*charter));
    }
}

fn executor(call: &Call, template: &c::Template) -> Result<c::Executor, Failed> {
    match template.executor {
        c::TemplateExecutor::Chosen => {
            let Some(charter) = number(call, 3) else {
                return Err(executor_refusal(template.number, None));
            };
            Ok(c::Executor::Agent { charter: u32::try_from(charter).expect("charter fits") })
        }
        c::TemplateExecutor::Charter(charter) => {
            if number(call, 3).is_some() {
                return Err(executor_refusal(template.number, Some(charter)));
            }
            Ok(c::Executor::Agent { charter })
        }
    }
}

fn executor_refusal(template: u32, charter: Option<u32>) -> Failed {
    let mut measure = Text::measure();
    executor_words(template, charter, &mut measure);
    let mut out = Text::writing_exact(measure);
    executor_words(template, charter, &mut out);
    Failed::Detail(out.finish())
}

fn executor_words(template: u32, charter: Option<u32>, out: &mut Text) {
    out.put(b"template ");
    out.number(u64::from(template));
    match charter {
        None => out.put(b" lets its creator choose the executor; name it: --charter CHARTER"),
        Some(charter) => {
            out.put(b"'s executor is charter ");
            out.number(u64::from(charter));
            out.put(b"; leave out --charter");
        }
    }
}

fn controls(call: &Call, project: u32, number: u32, me: &mut c::Me) -> Result<(), Failed> {
    for index in 0..me.projects.len() {
        let membership = me.projects.get_mut(index).expect("project index");
        if membership.project != project {
            continue;
        }
        for index in 0..membership.templates.len() {
            let template = membership.templates.get_mut(index).expect("template index");
            if template.number != number {
                continue;
            }
            required_controls(call, template)?;
            if value(call, 6).is_some() {
                template.budget = c::TemplateBudget::AsAllowed { cap: None };
            } else if super::number(call, 5).is_some() {
                template.budget = c::TemplateBudget::Given;
            }
            if value(call, 13).is_some() {
                template.deadline = c::TemplateDeadline::None;
            } else if value(call, 7).is_some() {
                template.deadline = c::TemplateDeadline::Given;
            }
        }
    }
    Ok(())
}

fn required_controls(call: &Call, template: &c::Template) -> Result<(), Failed> {
    let charter = template.executor == c::TemplateExecutor::Chosen && number(call, 3).is_none();
    let budget = template.budget == c::TemplateBudget::Given && number(call, 5).is_none() && value(call, 6).is_none();
    let deadline =
        template.deadline == c::TemplateDeadline::Given && value(call, 7).is_none() && value(call, 13).is_none();
    if !charter && !budget && !deadline {
        return Ok(());
    }
    let mut measure = Text::measure();
    required_words(template.number, charter, budget, deadline, &mut measure);
    let mut out = Text::writing_exact(measure);
    required_words(template.number, charter, budget, deadline, &mut out);
    Err(Failed::Detail(out.finish()))
}

fn required_words(template: u32, charter: bool, budget: bool, deadline: bool, out: &mut Text) {
    out.put(b"template ");
    out.number(u64::from(template));
    out.put(b" needs: ");
    if charter {
        out.put(b"--charter CHARTER; ");
    }
    if budget {
        out.put(b"--budget AMOUNT or --as-allowed; ");
    }
    if deadline {
        out.put(b"--deadline TIME; ");
    }
    out.put(b"help goal start");
}
