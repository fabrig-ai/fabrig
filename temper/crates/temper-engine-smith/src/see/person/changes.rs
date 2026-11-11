//! Forge facts in temper's words (temper/docs/design/domain/agent/ask-temper.md, 1).
use super::{join, project_name, words};
use alloc::boxed::Box;
use skein_lib::List;
use temper_engine_domain::SeeNames;
use temper_engine_domain_forge::{self as f, Document, PanelStep};
use temper_engine_domain_forge_change as c;

pub(super) fn change(number: u64, document: &Document, lines: &mut List<Box<[u8]>>) {
    let Document::Change(panel) = document else {
        lines.push(join(&[b"change ", &words::number(number), b": not read\n"])).expect("unread change");
        return;
    };
    // Panel has no task title or named gates/roles; use only the facts it holds.
    lines
        .push(join(&[
            b"change ",
            &words::number(number),
            " · ".as_bytes(),
            &words::short(&panel.owner, false),
            b"/",
            &words::short(&panel.name, false),
            b" ",
            &words::short(&panel.branch, false),
            b" into ",
            &words::short(&panel.landing, false),
            " · ".as_bytes(),
            step(&panel.step),
            b"\n",
        ]))
        .expect("change heading");
    match panel.pull {
        Some(pull) => lines.push(join(&[b"pull request ", &words::number(pull), b"\n"])).expect("pull request"),
        None => lines.push(b"no pull request yet\n".as_slice().into()).expect("no pull"),
    }
    if panel.ci_required {
        lines.push(join(&[b"CI: ", verdict(panel.ci), b"\n"])).expect("CI");
    }
    for gate in &panel.gates {
        lines.push(join(&[b"gate ", &words::number(gate.number), b": ", verdict(gate.verdict), b"\n"])).expect("gate");
    }
    approvals(panel, lines);
    lines
        .push(join(&[
            b"repairs: ",
            &words::number(u64::from(panel.repairs.used)),
            b" of ",
            &words::number(u64::from(panel.repairs.limit)),
            b"\n",
        ]))
        .expect("repairs");
    for question in &panel.questions.open {
        let stands = match question.stands {
            f::PanelStands::Taken => b"taken".as_slice(),
            f::PanelStands::Foreman(_) => b"with the foreman",
            f::PanelStands::Asked => b"waiting for a person",
            f::PanelStands::Held => b"held",
        };
        lines
            .push(join(&[
                b"question ",
                &words::number(question.number),
                b": ",
                &words::short(&question.line, false),
                " · ".as_bytes(),
                stands,
                b"\n",
            ]))
            .expect("question");
    }
    if let Some(why) = panel.hold {
        lines.push(join(&[b"hold: ", hold(why), b"\n"])).expect("hold");
    }
    match panel.way {
        c::Way::Turn => lines.push(b"landing by turn\n".as_slice().into()).expect("turn place"),
        c::Way::Queue => match &panel.step {
            PanelStep::First | PanelStep::Queued { place: 1, .. } => lines
                .push(join(&[b"first in the queue into ", &words::short(&panel.landing, false), b"\n"]))
                .expect("first place"),
            PanelStep::Queued { place, .. } => {
                lines.push(join(&[&ordinal(*place), b" in the queue\n"])).expect("queue place");
            }
            PanelStep::Ended
            | PanelStep::Ready { .. }
            | PanelStep::Preparing { .. }
            | PanelStep::Producing
            | PanelStep::Opening
            | PanelStep::Recreating
            | PanelStep::Reopening
            | PanelStep::Checking
            | PanelStep::Gating
            | PanelStep::Updating
            | PanelStep::Resolving
            | PanelStep::Repairing(_)
            | PanelStep::Landing
            | PanelStep::Landed { .. }
            | PanelStep::Verifying { .. }
            | PanelStep::Red { .. }
            | PanelStep::Cleared { .. } => {}
        },
    }
}

fn approvals(panel: &f::Panel, lines: &mut List<Box<[u8]>>) {
    for approval in &panel.approvals {
        lines
            .push(join(&[
                &words::number(skein_lib::len_u64(approval.counted.len())),
                b" of ",
                &words::number(u64::from(approval.required)),
                b" approvals by role ",
                &words::number(u64::from(approval.role)),
                b"\n",
            ]))
            .expect("approval");
        match approval.read {
            f::ReviewRead::Unread => {
                lines.push(b"approvals: reviews not read\n".as_slice().into()).expect("unread reviews");
            }
            f::ReviewRead::Incomplete => {
                lines.push(b"approvals: reviews incomplete\n".as_slice().into()).expect("incomplete reviews");
            }
            f::ReviewRead::Complete => {}
        }
        if approval.refused {
            lines.push(b"approvals: changes requested\n".as_slice().into()).expect("review refused");
        }
    }
}

fn ordinal(value: u32) -> Box<[u8]> {
    let suffix = match value.rem_euclid(100) {
        11..=13 => b"th".as_slice(),
        0..=10 | 14..=99 => match value.rem_euclid(10) {
            1 => b"st".as_slice(),
            2 => b"nd",
            3 => b"rd",
            0 | 4..=9 => b"th",
            10.. => unreachable!("remainder below ten"),
        },
        100.. => unreachable!("remainder below one hundred"),
    };
    join(&[&words::number(u64::from(value)), suffix])
}
fn verdict(status: c::Status) -> &'static [u8] {
    match status {
        c::Status::Unknown => b"waiting",
        c::Status::Pending => b"running",
        c::Status::Passed => b"passed",
        c::Status::Failed => b"failed",
    }
}
fn step(value: &PanelStep) -> &'static [u8] {
    match value {
        PanelStep::Ended => b"ended",
        PanelStep::Ready { .. } => b"ready",
        PanelStep::Preparing { .. } => b"preparing",
        PanelStep::Producing => b"producing",
        PanelStep::Opening => b"opening",
        PanelStep::Recreating => b"recreating",
        PanelStep::Reopening => b"reopening",
        PanelStep::Checking => b"checking",
        PanelStep::Gating => b"gating",
        PanelStep::Queued { .. } => b"queued",
        PanelStep::First => b"first",
        PanelStep::Updating => b"updating",
        PanelStep::Resolving => b"resolving",
        PanelStep::Repairing(_) => b"repairing",
        PanelStep::Landing => b"landing",
        PanelStep::Landed { .. } => b"landed",
        PanelStep::Verifying { .. } => b"verifying",
        PanelStep::Red { .. } => b"red",
        PanelStep::Cleared { .. } => b"cleared",
    }
}
fn hold(value: c::Hold) -> &'static [u8] {
    match value {
        c::Hold::Stalled => b"the forge stopped making progress",
        c::Hold::Repairs => b"the repair allowance was exhausted",
        c::Hold::Resolutions => b"the resolution allowance was exhausted",
        c::Hold::Updates => b"the update allowance was exhausted",
        c::Hold::BranchMoved => b"the branch moved outside its writer",
        c::Hold::BranchMissing => b"the branch disappeared",
        c::Hold::PullClosed => b"the pull request closed without a merge",
        c::Hold::Retargeted => b"the pull request was retargeted",
        c::Hold::Failed => b"a delegate or effect failed",
        c::Hold::Rejected => b"a required gate was rejected",
        c::Hold::Authority(_) => b"the next delegate needs more authority",
        c::Hold::Questions => b"the question allowance was exhausted",
    }
}

pub(super) fn queues(
    changes: Option<&Document>,
    landings: Option<&Document>,
    names: &SeeNames,
    lines: &mut List<Box<[u8]>>,
) {
    let Some(Document::Changes(changes)) = changes else {
        lines.push(join(&[b"changes of ", &project_name(names), b": not read\n"])).expect("unread changes");
        return;
    };
    let mut queued = u64::from(changes.more);
    for queue in &changes.queues {
        queued = queued.checked_add(skein_lib::len_u64(queue.changes.len())).expect("bounded queues");
    }
    let landings = match landings {
        Some(Document::Landings(value)) => Some(value),
        Some(
            Document::Change(_)
            | Document::Landing(_)
            | Document::Changes(_)
            | Document::Status(_)
            | Document::Devices(_)
            | Document::Aways(_)
            | Document::Decisions(_)
            | Document::Dismissals(_),
        )
        | None => None,
    };
    let landing_count = match landings {
        Some(value) => words::number(skein_lib::len_u64(value.landings.len())),
        None => b"not read".as_slice().into(),
    };
    lines
        .push(join(&[
            b"changes of ",
            &project_name(names),
            b": ",
            &words::number(queued),
            b" queued, ",
            &landing_count,
            b" landing by turn\n",
        ]))
        .expect("changes heading");
    for queue in &changes.queues {
        lines
            .push(join(&[
                b"into ",
                &words::short(&queue.repository, false),
                b" ",
                &words::short(&queue.branch, false),
                b": ",
                &words::number(skein_lib::len_u64(queue.changes.len())),
                b"\n",
            ]))
            .expect("queue heading");
        for row in &queue.changes {
            // Queue rows carry title and order, but no panel step or wait reason.
            lines
                .push(join(&[b"change ", &words::number(row.task), b", ", &words::short(&row.title, false), b"\n"]))
                .expect("queued change");
        }
    }
    if changes.more != 0 {
        lines
            .push(join(&[&words::number(u64::from(changes.more)), b" queued changes not read\n"]))
            .expect("queue remainder");
    }
    lines.push(join(&[b"landing by turn: ", &landing_count, b"\n"])).expect("landing heading");
    if let Some(landings) = landings {
        for landing in &landings.landings {
            let mut members = List::uncapped();
            for change in &landing.changes {
                members.push(join(&[b"change ", &words::number(*change)])).expect("landing member");
            }
            lines
                .push(join(&[b"landing task ", &words::number(landing.task), b": ", &member_names(&members), b"\n"]))
                .expect("landing row");
        }
    }
    lines
        .push(join(&[b"landed lately: ", &words::number(skein_lib::len_u64(changes.landed.len())), b"\n"]))
        .expect("landed heading");
    for row in &changes.landed {
        // A retained landed row carries no branch; ChangesView::landing names
        // only the newest landing, so cannot label every historical row.
        lines
            .push(join(&[
                b"change ",
                &words::number(row.task),
                b", ",
                &words::short(&row.title, false),
                " · ".as_bytes(),
                &words::time(row.at),
                b"\n",
            ]))
            .expect("landed change");
    }
    if changes.landed_more != 0 {
        lines
            .push(join(&[&words::number(u64::from(changes.landed_more)), b" landed changes not read\n"]))
            .expect("landed remainder");
    }
}
fn member_names(members: &List<Box<[u8]>>) -> Box<[u8]> {
    let mut parts = List::uncapped();
    for (index, member) in members.iter().enumerate() {
        if index != 0 {
            parts.push(b", ".as_slice().into()).expect("separator");
        }
        parts.push(member.clone()).expect("member name");
    }
    super::super::join_parts(&parts)
}
