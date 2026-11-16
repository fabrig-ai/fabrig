//! Template 8 with jig's gathering (client/ctl/milestones.md, 2).
use super::{Call, Failed, Limits, Prepared, Request, c, empty, number, shapes, value, words};
use crate::jig::{call::Value, render::Text};

pub(super) fn new(call: &Call, me: &c::Me, limits: &Limits) -> Prepared {
    let project = super::super::reads::project(call, me).ok_or(Failed::Fixed(b"unknown project"))?;
    let mut offered = me.clone();
    for index in 0..offered.projects.len() {
        let membership = offered.projects.get_mut(index).expect("project index");
        if membership.project != project {
            continue;
        }
        for index in 0..membership.templates.len() {
            let template = membership.templates.get_mut(index).expect("template index");
            if template.number != 8 {
                continue;
            }
            let budget =
                template.budget == c::TemplateBudget::Given && number(call, 4).is_none() && value(call, 5).is_none();
            let deadline = template.deadline == c::TemplateDeadline::Given && value(call, 6).is_none();
            if budget || deadline {
                return Err(required(budget, deadline));
            }
            if value(call, 5).is_some() {
                template.budget = c::TemplateBudget::AsAllowed { cap: None };
            } else if number(call, 4).is_some() {
                template.budget = c::TemplateBudget::Given;
            }
            if value(call, 6).is_some() {
                template.deadline = c::TemplateDeadline::Given;
            }
        }
    }
    let name = words(call, 1, limits)?;
    let spec = if value(call, 2).is_some() {
        let words = words(call, 2, limits)?;
        let bytes = skein_lib::bytes::concat(&[name.bytes(), b"\n\n", words.bytes()]);
        c::Text::new(bytes, limits.words).or(Err(Failed::Fixed(b"text limit")))?
    } else {
        name
    };
    let draft = shapes::GoalDraft {
        words: spec,
        executor: c::Executor::Procedure { connector: shapes::RECURRING, code: shapes::GATHERING },
        budget: number(call, 4).unwrap_or(0),
        deadline: match value(call, 6) {
            Some(Value::Text(text)) => crate::jig::argv::deadline(text),
            _ => None,
        },
        priority: match number(call, 3) {
            Some(priority) => Some(u32::try_from(priority).expect("priority fits")),
            None => None,
        },
        llm: None,
    };
    match shapes::set_goal_from(&offered, project, 8, draft, empty(limits)) {
        shapes::Judged::Create(ask) => Ok((Request::Ask(ask), None)),
        shapes::Judged::Propose { ask, to } => Ok((Request::Ask(ask), Some(to))),
        shapes::Judged::Refused(why) => Err(super::goal::refused(&why, limits)),
        shapes::Judged::NotOffered(why) => Err(Failed::Fixed(match why {
            shapes::NotOffered::Project => b"unknown project",
            shapes::NotOffered::Template => b"milestone template 8 not offered",
            shapes::NotOffered::ChipExecutor => b"milestone gathering not offered",
            shapes::NotOffered::Ceiling => b"milestone budget or deadline exceeds the project's offered ceiling",
            shapes::NotOffered::Creating => b"project has no creating offer; check role and receiving bound documents.lines for charter and procedure lists",
        })),
    }
}
fn required(budget: bool, deadline: bool) -> Failed {
    let mut measure = Text::measure();
    required_words(budget, deadline, &mut measure);
    let mut out = Text::writing_exact(measure);
    required_words(budget, deadline, &mut out);
    Failed::Detail(out.finish())
}
fn required_words(budget: bool, deadline: bool, out: &mut Text) {
    out.put(b"template 8 needs: ");
    if budget {
        out.put(b"--budget AMOUNT or --as-allowed; ");
    }
    if deadline {
        out.put(b"--deadline TIME; ");
    }
    out.put(b"help milestone new");
}
