//! A standing role fills template 7 (domain/agent/standing.md, 6).
use super::{Call, Failed, Limits, Prepared, Request, c, empty, number, shapes, words};

pub(super) fn start(call: &Call, me: &c::Me, limits: &Limits) -> Prepared {
    let project = super::super::reads::project(call, me).ok_or(Failed::Fixed(b"unknown project"))?;
    let draft = shapes::GoalDraft {
        words: words(call, 2, limits)?,
        executor: c::Executor::Agent { charter: u32::try_from(number(call, 3).unwrap_or(13)).expect("charter fits") },
        budget: number(call, 1).expect("period budget"),
        deadline: None,
        priority: None,
        llm: None,
    };
    match shapes::set_goal_from(me, project, 7, draft, empty(limits)) {
        shapes::Judged::Create(ask) => Ok((Request::Ask(ask), None)),
        shapes::Judged::Propose { ask, to } => Ok((Request::Ask(ask), Some(to))),
        shapes::Judged::Refused(why) => Err(super::goal::refused(&why, limits)),
        shapes::Judged::NotOffered(why) => Err(Failed::Fixed(match why {
            shapes::NotOffered::Project => b"unknown project",
            shapes::NotOffered::Template => b"standing role template 7 not offered",
            shapes::NotOffered::ChipExecutor => b"role charter not offered",
            shapes::NotOffered::Ceiling => b"role budget or deadline exceeds the project's offered ceiling",
            shapes::NotOffered::Creating => b"project has no creating offer",
        })),
    }
}
