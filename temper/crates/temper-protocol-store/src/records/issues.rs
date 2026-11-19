use super::{names, w};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use temper_engine_domain_forge_issues as d;
use temper_engine_domain_forge_vocabulary as forge;

fn encode_phase(value: d::Phase) -> w::Phase {
    match value {
        d::Phase::Waiting => w::Phase::Waiting,
        d::Phase::Active => w::Phase::Active,
        d::Phase::Settling => w::Phase::Settling,
        d::Phase::Held => w::Phase::Held,
        d::Phase::Done => w::Phase::Done,
        d::Phase::Failed => w::Phase::Failed,
        d::Phase::Cancelled => w::Phase::Cancelled,
    }
}
fn decode_phase(value: w::Phase) -> d::Phase {
    match value {
        w::Phase::Waiting => d::Phase::Waiting,
        w::Phase::Active => d::Phase::Active,
        w::Phase::Settling => d::Phase::Settling,
        w::Phase::Held => d::Phase::Held,
        w::Phase::Done => d::Phase::Done,
        w::Phase::Failed => d::Phase::Failed,
        w::Phase::Cancelled => d::Phase::Cancelled,
    }
}
fn encode_key(value: d::MilestoneKey) -> w::MilestoneKey {
    match value {
        d::MilestoneKey::Lifecycle { task, position } => w::MilestoneKey::Lifecycle(w::Lifecycle { task, position }),
        d::MilestoneKey::TaskRevision { task, revision } => {
            w::MilestoneKey::TaskRevision(w::TaskRevision { task, revision })
        }
        d::MilestoneKey::PlanAccepted => w::MilestoneKey::PlanAccepted,
        d::MilestoneKey::Revision(number) => w::MilestoneKey::Revision(number_record(number)),
        d::MilestoneKey::ChangeLanded(number) => w::MilestoneKey::ChangeLanded(number_record(number)),
        d::MilestoneKey::ChangeHeld(number) => w::MilestoneKey::ChangeHeld(number_record(number)),
        d::MilestoneKey::Report(number) => w::MilestoneKey::Report(number_record(number)),
        d::MilestoneKey::Finished => w::MilestoneKey::Finished,
    }
}
fn number_record(number: u64) -> w::Number {
    w::Number { number }
}
fn decode_key(value: w::MilestoneKey) -> d::MilestoneKey {
    match value {
        w::MilestoneKey::Lifecycle(value) => {
            let w::Lifecycle { task, position } = value;
            d::MilestoneKey::Lifecycle { task, position }
        }
        w::MilestoneKey::TaskRevision(value) => {
            let w::TaskRevision { task, revision } = value;
            d::MilestoneKey::TaskRevision { task, revision }
        }
        w::MilestoneKey::PlanAccepted => d::MilestoneKey::PlanAccepted,
        w::MilestoneKey::Revision(value) => d::MilestoneKey::Revision(value.number),
        w::MilestoneKey::ChangeLanded(value) => d::MilestoneKey::ChangeLanded(value.number),
        w::MilestoneKey::ChangeHeld(value) => d::MilestoneKey::ChangeHeld(value.number),
        w::MilestoneKey::Report(value) => d::MilestoneKey::Report(value.number),
        w::MilestoneKey::Finished => d::MilestoneKey::Finished,
    }
}
fn encode_projected(value: &d::Projected) -> w::Projected {
    let d::Projected { opened, digest, last_body, body_revision, milestones, closed } = value;
    let mut keys = List::with_capacity(u32::try_from(milestones.len()).expect("admitted count"));
    for key in milestones {
        keys.push(encode_key(*key)).expect("same count");
    }
    let last_body = match last_body {
        Some(value) => Some(value.as_nanos()),
        None => None,
    };
    w::Projected {
        opened: *opened,
        digest: *digest,
        last_body,
        body_revision: *body_revision,
        milestones: keys,
        closed: *closed,
    }
}
fn decode_projected(value: w::Projected) -> d::Projected {
    let w::Projected { opened, digest, last_body, body_revision, milestones, closed } = value;
    let mut keys = List::with_capacity(milestones.len());
    for key in milestones.into_boxed() {
        keys.push(decode_key(key)).expect("same count");
    }
    let last_body = match last_body {
        Some(value) => Some(Wall::from_nanos(value)),
        None => None,
    };
    d::Projected { opened, digest, last_body, body_revision, milestones: keys.into_boxed(), closed }
}
fn encode_view(value: &d::GoalView) -> w::GoalView {
    let d::GoalView { phase, goal, repository, title, goal_text, plan, milestones, finished } = value;
    let mut items = List::with_capacity(u32::try_from(plan.len()).expect("admitted count"));
    for d::PlanItem { text, phase } in plan {
        let item = w::PlanItem { text: text.as_bytes().into(), phase: encode_phase(*phase) };
        items.push(item).expect("same count");
    }
    let mut comments = List::with_capacity(u32::try_from(milestones.len()).expect("admitted count"));
    for d::Milestone { key, text } in milestones {
        let item = w::Milestone { key: encode_key(*key), text: text.as_bytes().into() };
        comments.push(item).expect("same count");
    }
    w::GoalView {
        phase: encode_phase(*phase),
        goal: *goal,
        repository: *repository,
        title: title.as_bytes().into(),
        goal_text: goal_text.as_bytes().into(),
        plan: items,
        milestones: comments,
        finished: match finished {
            Some(value) => Some(value.as_bytes().into()),
            None => None,
        },
    }
}
fn decode_view(value: w::GoalView) -> d::GoalView {
    let w::GoalView { phase, goal, repository, title, goal_text, plan, milestones, finished } = value;
    let mut items = List::with_capacity(plan.len());
    for item in plan.into_boxed() {
        let w::PlanItem { text, phase } = item;
        items.push(d::PlanItem { text: decode_text(text), phase: decode_phase(phase) }).expect("same count");
    }
    let mut comments = List::with_capacity(milestones.len());
    for item in milestones.into_boxed() {
        let w::Milestone { key, text } = item;
        comments.push(d::Milestone { key: decode_key(key), text: decode_text(text) }).expect("same count");
    }
    d::GoalView {
        phase: decode_phase(phase),
        goal,
        repository,
        title: decode_text(title),
        goal_text: decode_text(goal_text),
        plan: items.into_boxed(),
        milestones: comments.into_boxed(),
        finished: match finished {
            Some(value) => Some(decode_text(value)),
            None => None,
        },
    }
}
pub(super) fn encode(value: &forge::IssueRow) -> w::IssueRow {
    let forge::IssueRow { goal, repository, state, number, pending, before, failed, desired } = value;
    let before = match before {
        Some(value) => Some(encode_projected(value)),
        None => None,
    };
    let desired = match desired {
        Some(value) => Some(encode_view(value)),
        None => None,
    };
    w::IssueRow {
        goal: *goal,
        repository: names::encode_repository(*repository),
        state: encode_projected(state),
        number: *number,
        pending: *pending,
        before,
        failed: *failed,
        desired,
    }
}
pub(super) fn decode(value: w::IssueRow) -> forge::IssueRow {
    let w::IssueRow { goal, repository, state, number, pending, before, failed, desired } = value;
    let before = match before {
        Some(value) => Some(decode_projected(value)),
        None => None,
    };
    let desired = match desired {
        Some(value) => Some(decode_view(value)),
        None => None,
    };
    forge::IssueRow {
        goal,
        repository: names::decode_repository(repository),
        state: decode_projected(state),
        number,
        pending,
        before,
        failed,
        desired,
    }
}
#[expect(clippy::disallowed_methods, reason = "the generated protocol decoder already validated this text")]
fn decode_text(bytes: Box<[u8]>) -> Box<str> {
    core::str::from_utf8(&bytes).expect("generated text validated UTF-8").into()
}
