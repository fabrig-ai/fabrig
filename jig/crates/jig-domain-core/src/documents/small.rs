//! Small live documents and the shared bounded chip construction.
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).

use crate::{
    Core, Limits,
    documents::{Chip, Count, Frame, FrameProjectRole, Missing, Run},
};
pub use jig_domain_core_vocabulary::documents::small::*;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::List;

pub(super) fn visible(core: &Core, party: u64, number: u64) -> Result<&tasks::TaskRecord, Missing> {
    let row = core.tasks.task(number).ok_or(Missing::Unknown)?;
    if !core.open_project(row.project) || core.people.role(party, row.project).is_none() {
        return Err(Missing::NotAllowed);
    }
    match row.phase {
        tasks::Phase::Ended(_) => Err(Missing::Ended),
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            Ok(row)
        }
    }
}

pub(super) fn chip(core: &Core, limits: &Limits, party: u64, number: u64) -> Result<Chip, Missing> {
    let row = visible(core, party, number)?;
    chip_with_core(core, limits, party, row)
}

pub(super) fn chip_with_core(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
) -> Result<Chip, Missing> {
    let mut chip = chip_from_row(limits, row)?;
    if let Some(hold) = &mut chip.details.hold {
        hold.placed = super::task_tree::placed(core, limits, party, row.number)?;
    }
    super::chip_phase::observe(core, limits, row, &mut chip.details)?;
    Ok(chip)
}

#[expect(
    clippy::too_many_lines,
    reason = "the frame assembles project offers, chat bounds and template previews together"
)]
pub(super) fn frame(core: &Core, limits: &Limits, party: u64) -> Result<Frame, Missing> {
    let identity = core.people.identity(party).ok_or(Missing::Unknown)?;
    let roles = core.people.project_roles(party).ok_or(Missing::Unknown)?;
    let list_left_out = super::accounts::left_out(core, party);
    let mut projects = List::with_capacity(limits.documents.lines);
    let mut cut = false;
    for role in &roles {
        if !core.open_project(role.project) {
            continue;
        }
        if projects.room() == 0 {
            cut = true;
            continue;
        }
        let pool = match core.tasks.funding(tasks::Funder::Pool {
            project: role.project,
            person: party,
            period: core.period(role.project),
        }) {
            Some(record) => Some(super::project_note::ledger(record)),
            None => None,
        };
        let models =
            core.authority.models(role.project, role.role.number()).unwrap_or(jig_domain_authority::Models::NONE);
        let allowed = tasks::Models::from_parts(models.bits(), models.is_all());
        let payer = tasks::Payer::Party(party);
        let offer = core.offer(payer, role.project, allowed);
        let merged = core.project_templates(role.project);
        let mut left_out = 0;
        let mut defaults = List::with_capacity(limits.templates);
        let mut templates = List::with_capacity(limits.templates);
        for template in merged {
            let chat = match template.executor {
                people::TemplateExecutor::Charter(number) if template.number == 1 => core.current.charter(number),
                people::TemplateExecutor::Charter(_) | people::TemplateExecutor::Chosen => None,
            };
            if let Some(charter) = chat {
                left_out = crate::settings::charters::left_out(
                    charter,
                    &crate::settings::catalogue::entries(&core.current),
                    limits,
                );
            }
            let preview = match template.executor {
                people::TemplateExecutor::Chosen => None,
                people::TemplateExecutor::Charter(charter) => match &template.llm {
                    Some(choice) => Some(choice.clone()),
                    None => core.preview_llm(charter, core.current.newest(), allowed, payer, role.project),
                },
            };
            if let Some(mut choice) = preview {
                choice.account = None;
                defaults.push((template.number, choice)).expect("one preview per template");
            }
            templates.push(template).expect("configured template bound");
        }
        left_out = left_out.checked_add(list_left_out).ok_or(Missing::TooLarge)?;
        projects
            .push(FrameProjectRole {
                left_out,
                public: core.public_project(role.project),
                offer,
                defaults,
                templates,
                project: role.project,
                role: role.role,
                name: text(
                    core.names.get(&role.project).expect("project role has a kept name").as_bytes(),
                    limits.documents.text_bytes,
                ),
                pool,
                creating: match crate::acts::bounds::creating(core, limits, party, role.project, Some(role.role)) {
                    crate::acts::Admission::Bounded { within, beyond } => {
                        Some(crate::acts::Creating { within, beyond })
                    }
                    crate::acts::Admission::Refused(_) => None,
                    crate::acts::Admission::Allowed | crate::acts::Admission::Proposed { .. } => {
                        unreachable!("creating is bounded or refused")
                    }
                },
                models,
            })
            .expect("bounded project list");
    }
    let counts = core.people.counts(&limits.people, party).ok_or(Missing::Unknown)?;
    let updates = if counts.more_unread { Count::AtLeast(counts.unread) } else { Count::Known(counts.unread) };
    let (providers, providers_cut) = super::catalogue::offers(core, &limits.documents)?;
    Ok(Frame {
        providers,
        providers_cut,
        key_bytes: core.settings.key_bytes,
        redirect_bytes: core.settings.redirect_bytes,
        words_bytes: limits.people.words,
        party,
        kind: core.people.kind(party).ok_or(Missing::Unknown)?,
        login: text(&identity.login, limits.documents.text_bytes),
        name: text(&identity.name, limits.documents.text_bytes),
        deployment_role: core.people.role(party, people::DEPLOYMENT),
        cut,
        projects,
        needs: counts.needs.checked_add(core.settings_needs(party)).ok_or(Missing::TooLarge)?,
        need_counts: core.party_need_counts(party).ok_or(Missing::Unknown)?,
        updates,
        results_seen: core.people.results_seen(party).ok_or(Missing::Unknown)?,
        retention: limits.people.request_retention,
    })
}

pub(super) fn run(core: &Core, limits: &Limits, party: u64, number: u64) -> Result<Run, Missing> {
    visible(core, party, number)?;
    let proof = match core.proofs.get(&number) {
        Some(proof) if proof.terminal.is_none() => proof,
        Some(_) | None => return Ok(Run { task: number, attempt: None, committed_turn: 0, writing: None, call: None }),
    };
    let committed_turn = match proof.turn {
        Some(turn) => turn.turn,
        None => 0,
    };
    let mut writing =
        match jig_domain_views::writing(&core.views, skein_lib::Token::new(number), limits.documents.text_bytes) {
            Some(writing) if writing.turn.attempt == proof.attempt && writing.turn.turn > committed_turn => {
                Some(writing)
            }
            Some(_) | None => None,
        };
    let mut call = jig_domain_views::current_call(
        &core.views,
        skein_lib::Token::new(number),
        proof.attempt,
        limits.documents.text_bytes,
    );
    crate::reads::withheld::snapshot(core, party, number, &mut call, &mut writing);
    Ok(Run { task: number, attempt: Some(proof.attempt), committed_turn, writing, call })
}
