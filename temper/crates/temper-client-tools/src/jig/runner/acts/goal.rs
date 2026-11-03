//! Template controls are filled before jig judges authority (client/tools.md, 5.5).
use super::{Call, Failed, Limits, Prepared, Request, c, choice, empty, number, shapes, value, words};
use crate::jig::call::Value;
use crate::jig::render::Text;

/// Apply ctl controls, then use the offered template and jig judgement.
pub(super) fn start(call: &Call, me: &c::Me, limits: &Limits) -> Prepared {
    let project = super::super::reads::project(call, me).ok_or(Failed::Fixed(b"unknown project"))?;
    let number = u32::try_from(number(call, 1).unwrap_or(u64::from(shapes::GOAL))).expect("template fits");
    let template = shapes::template(me, project, number).ok_or(Failed::Fixed(b"goal template not offered"))?;
    let executor = executor(call, template)?;
    let llm = choice::selection(call, 8, 9, 10, me, project, template.llm.as_ref())?;
    let mut offered = me.clone();
    controls(call, project, number, &mut offered)?;
    #[expect(clippy::manual_map, reason = "step code matches the optional priority without closures")]
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
    match shapes::set_goal_from(&offered, project, number, draft, empty(limits)) {
        shapes::Judged::Create(ask) => Ok((Request::Ask(ask), None)),
        shapes::Judged::Propose { ask, to } => Ok((Request::Ask(ask), Some(to))),
        shapes::Judged::Refused(_) => Err(Failed::Fixed(b"goal exceeds offered bounds")),
        shapes::Judged::NotOffered(_) => Err(Failed::Fixed(b"goal not offered or exceeds offered bounds")),
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
            if value(call, 6).is_some() {
                template.budget = c::TemplateBudget::AsAllowed { cap: None };
            } else if super::number(call, 5).is_some() {
                template.budget = c::TemplateBudget::Given;
            } else if template.budget == c::TemplateBudget::Given {
                return Err(Failed::Fixed(b"this template needs --budget AMOUNT or --as-allowed"));
            }
            if value(call, 7).is_some() {
                template.deadline = c::TemplateDeadline::Given;
            } else if template.deadline == c::TemplateDeadline::Given {
                return Err(Failed::Fixed(b"this template needs --deadline TIME"));
            }
        }
    }
    Ok(())
}
