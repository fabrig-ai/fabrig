//! Each restart-rebuilt document field is declared here (jig jig/docs/design/domain/engine/world.md, 1).
use jig_domain_core::{WaitKind, documents as docs, reads::Answer};
use jig_domain_tasks::Active;

pub(super) fn answers(before: &mut Answer, after: &mut Answer) {
    if let (Answer::Document(left), Answer::Document(right)) = (&mut *before, &mut *after)
        && let (docs::Document::Task(left), docs::Document::Task(right)) = (left.as_mut(), right.as_mut())
    {
        let (left_wait, left_bound) = durable_waits(left);
        let (right_wait, right_bound) = durable_waits(right);
        // A discarded winner can expose a different wait on the other side. The
        // retained wait lines still compare; only the affected projection is masked.
        if left_wait || right_wait {
            left.wait = None;
            right.wait = None;
        }
        if left_bound || right_bound {
            left.nearest = None;
            right.nearest = None;
        }
    }
    for answer in [before, after] {
        if let Answer::Document(document) = answer {
            durable(document);
        }
    }
}

fn durable(document: &mut docs::Document) {
    match document {
        docs::Document::Frame(frame) => durable_frame(frame),
        docs::Document::Task(task) => durable_task(task),
        docs::Document::Chips(_)
        | docs::Document::Run(_)
        | docs::Document::Tree(_)
        | docs::Document::Lines(_)
        | docs::Document::Proposal(_)
        | docs::Document::Escalation(_)
        | docs::Document::Question(_)
        | docs::Document::PersonTask(_)
        | docs::Document::Result(_)
        | docs::Document::Project(_)
        | docs::Document::Note(_)
        | docs::Document::Inbox(_)
        | docs::Document::Terminal { .. }
        | docs::Document::Accounts(_)
        | docs::Document::Deployment(_)
        | docs::Document::Settings(_) => {}
    }
}

fn durable_frame(frame: &mut docs::Frame) {
    // Engine 6 step 1 restores accounts without runtime grants; step 3 awaits
    // reconnecting workers' logins. Engine 11.1's offers and inferred previews
    // observe those facts; a template's named choice is a retained setting.
    for project in frame.projects.iter_mut() {
        project.offer.clear();
        let mut defaults = skein_lib::List::with_capacity(project.defaults.capacity());
        for (number, choice) in &project.defaults {
            if project.templates.iter().any(|template| template.number == *number && template.llm.is_some()) {
                defaults.push((*number, choice.clone())).expect("retained preview subset");
            }
        }
        project.defaults = defaults;
    }
}

fn durable_task(task: &mut docs::Task) {
    // task_wait::waiting joins the current fleet and credential availability;
    // those peers have no live connection after a cold restart.
    task.run_wait = None;
    // An omitted choice is also inferred from current accounts and hosts.
    // The retained inventory compares every stored choice, including named ones.
    task.llm = None;
    durable_chip(&mut task.chip);
    for i in 0..task.lineage.len() {
        durable_chip(task.lineage.get_mut(i).expect("lineage index"));
    }
}

/// Keep durable wait episodes and report whether first or nearest projected a
/// volatile one (jig/docs/design/domain/engine/waits.md, 4).
fn durable_waits(task: &mut docs::Task) -> (bool, bool) {
    use docs::{ChipWork, PhaseLine, WaitsOn};
    let volatile: fn(WaitKind) -> bool = match task.chip.phase {
        PhaseLine::Active(Active::Due) if task.run_wait.is_some() => {
            |kind| matches!(kind, WaitKind::Account | WaitKind::Host)
        }
        PhaseLine::Active(Active::Claimed { .. } | Active::Running { .. })
            if task.chip.details.live_delegates == 0
                && task.chip.details.observation.as_ref().is_some_and(|value| value.work == ChipWork::Waiting) =>
        {
            |kind| kind == WaitKind::News
        }
        _ => return (false, false),
    };
    let mut durable = skein_lib::List::with_capacity(task.waits.capacity());
    for wait in &task.waits {
        if !volatile(wait.kind) {
            durable.push(wait.clone()).expect("durable waits are a subset of the document lines");
        }
    }
    task.waits = durable;
    if task.waits_on == WaitsOn::News && volatile(WaitKind::News) {
        task.waits_on = WaitsOn::Nothing;
    }
    // Each branch contributes one volatile episode, even if its line is cut.
    task.wait_count = task.wait_count.saturating_sub(1);
    (task.wait.as_ref().is_some_and(|wait| volatile(wait.kind)), task.nearest.is_some_and(|bound| volatile(bound.kind)))
}

fn durable_chip(chip: &mut docs::Chip) {
    // chip_phase::observe joins fenced host observations, absent on a cold engine.
    chip.details.observation = None;
}

/// Live CI is joined from `panel_ci`; terminal CI is retained in `ChangeRow.ci`
/// (temper-engine-domain-forge/src/panel.rs, panel's read selection).
#[must_use]
pub fn forge(
    mut document: Option<temper_engine_domain_forge::Document>,
) -> Option<temper_engine_domain_forge::Document> {
    if let Some(temper_engine_domain_forge::Document::Change(panel)) = &mut document {
        durable_panel(panel);
    }
    if let Some(temper_engine_domain_forge::Document::Landings(view)) = &mut document {
        // The document's clock is read anew; its event and turn times remain retained evidence.
        view.now = skein_lib::Wall::EPOCH;
    }
    document
}
fn durable_panel(panel: &mut temper_engine_domain_forge::Panel) {
    use temper_engine_domain_forge::{PanelStep, ReviewRead};
    // Last judged reviews are unsaved observations (domain/forge/change/gates.md, 2).
    panel.unmapped_reviews = Box::new([]);
    for approval in &mut panel.approvals {
        approval.read = ReviewRead::Unread;
        approval.counted = Box::new([]);
        approval.remaining = None;
        approval.refused = false;
    }
    if !matches!(
        panel.step,
        PanelStep::Ended
            | PanelStep::Landed { .. }
            | PanelStep::Verifying { .. }
            | PanelStep::Red { .. }
            | PanelStep::Cleared { .. }
    ) {
        panel.ci = temper_engine_domain_forge_change::Status::Unknown;
    }
}

/// Routing rebuilds connection, setup and installation observations at startup
/// (temper/docs/design/domain/forge/adoption-connection.md, 1). Adopted rows persist.
#[must_use]
pub fn forges(mut document: temper_engine_domain_forge::Forges) -> temper_engine_domain_forge::Forges {
    document.full = Box::new([]);
    for row in &mut document.rows {
        row.connection = temper_engine_domain_forge::Connection::NotConnected;
        row.app = None;
        row.failed = None;
        row.offer = temper_engine_domain_forge::Offer::NotOffered;
        row.first = None;
        row.installations = Box::new([]);
    }
    document
}
