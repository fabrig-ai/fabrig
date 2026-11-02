//! Each restart-rebuilt document field is declared here (jig engine.md, 15).
use jig_domain_core::{documents as docs, reads::Answer};

pub(super) fn answer(mut answer: Answer) -> Answer {
    if let Answer::Document(document) = &mut answer {
        match document.as_mut() {
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
            | docs::Document::Accounts(_)
            | docs::Document::Deployment(_)
            | docs::Document::Settings(_) => {}
        }
    }
    answer
}

fn durable_frame(_frame: &mut docs::Frame) {
    // The frame is made entirely from retained identity, roles and read positions.
}

fn durable_task(task: &mut docs::Task) {
    // task_wait::waiting joins the current fleet and credential availability;
    // those peers have no live connection after a cold restart.
    task.run_wait = None;
    durable_chip(&mut task.chip);
    for i in 0..task.lineage.len() {
        durable_chip(task.lineage.get_mut(i).expect("lineage index"));
    }
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
    document
}
fn durable_panel(panel: &mut temper_engine_domain_forge::Panel) {
    use temper_engine_domain_forge::PanelStep;
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
/// (temper/docs/design/domain/forge.md, 4.1). Adopted rows persist.
#[must_use]
pub fn forges(mut document: temper_engine_domain_forge::Forges) -> temper_engine_domain_forge::Forges {
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
