//! Held-task card projection (client/domain/view.md, section 10).
//! Typed occupied object keys share one residual Task-chip budget. Repeated
//! references still render repeatedly; embedded Person results remain whole.
use super::{Config, chip_nodes, frame_of_card, markdown};

fn value(l: &Config, client: &jig_client::DomainLimits, proposals: u32) -> Option<u32> {
    // Kept original proposals appear in Conversation, never with Inbox lineage.
    let proposal = if proposals == 0 {
        0
    } else {
        super::proposal_document_nodes(l, client)?.checked_sub(chip_nodes(l, client)?.checked_mul(client.rows)?)?
    };
    let verdict = markdown(l, client)?
        .checked_add(1)?
        .checked_add(chip_nodes(l, client)?)?
        .checked_mul(client.rows)?
        .checked_add(proposal)?
        .checked_add(markdown(l, client)?)?
        .checked_add(2)?;
    let change = markdown(l, client)?
        .checked_add(crate::tree::fragments::proposed_task::part_nodes(l, client)?)?
        .checked_add(2)?;
    Some(verdict.max(markdown(l, client)?.checked_add(3)?).max(change))
}

fn person(l: &Config, client: &jig_client::DomainLimits, facts: u32, frame: u32) -> Option<u32> {
    // Both source contracts retain all embedded results, links and controls.
    let old_facts = super::result_facts(l, client)?.checked_mul(client.rows)?;
    let new_facts = facts.checked_mul(client.rows)?;
    let original = super::person_task_nodes(l, client)?
        .checked_sub(chip_nodes(l, client)?.checked_mul(client.rows)?)?
        .checked_sub(old_facts)?
        .checked_add(new_facts)?;
    let owner = crate::tree::fragments::person_task::max_nodes(l, client)?
        .checked_sub(old_facts)?
        .checked_add(new_facts)?
        .checked_add(frame)?
        .checked_add(crate::tree::fragments::person_task::controls_nodes(l, client)?)?;
    Some(original.max(owner))
}

fn case(l: &Config, client: &jig_client::DomainLimits, results: u32, persons: u32, proposals: u32) -> Option<u32> {
    // The escalation itself and each present distinct typed key cannot also
    // be a stored Task. Other non-Task keys only reduce the actual capacity.
    let occupied = 1_u32.checked_add(results)?.checked_add(persons)?.checked_add(proposals)?;
    let Some(objects) = l.objects.checked_sub(occupied) else { return Some(0) };
    let l = &Config { objects, ..*l };
    let value = value(l, client, proposals)?;
    let ending = value.checked_add(markdown(l, client)?)?.checked_add(8)?;
    let facts = chip_nodes(l, client)?
        .checked_add(value)?
        .checked_add(crate::tree::fragments::result::nodes(l, client)?)?
        .checked_add(10)?;
    // Nested Result and Person cards use TaskPage, without Inbox lineage.
    // Their whole frames also cover absent-body loading alternatives.
    let frame = frame_of_card(l, client)?.checked_sub(chip_nodes(l, client)?.checked_mul(client.rows)?)?;
    let result = if results == 0 { 0 } else { frame.checked_add(facts)?.checked_mul(client.rows)? };
    let person = if persons == 0 { 0 } else { person(l, client, facts, frame)? };
    // Owner-chip and Task endings are passive values/facts, without a Result
    // card's outer acts. Keep all other supplied metadata and ending reasons.
    let owner_chip = crate::tree::fragments::chip::max_nodes(l, client)?
        .checked_sub(super::result_nodes(l, client)?)?
        .checked_add(ending)?;
    frame_of_card(l, client)?
        .checked_add(chip_nodes(l, client)?)?
        .checked_add(owner_chip)?
        .checked_add(client.rows.checked_add(20)?.checked_mul(client.rows)?)?
        .checked_add(facts.max(ending))?
        .checked_add(crate::tree::fragments::specification::nodes(l, client)?)?
        .checked_add(crate::tree::fragments::instructions::nodes(l, client)?)?
        .checked_add(crate::tree::fragments::proposed_task::contract_nodes(l, client)?)?
        .checked_add(crate::tree::fragments::wake::policy_nodes())?
        .checked_add(crate::tree::fragments::wake::subscription_nodes(client.rows)?)?
        .checked_add(chip_nodes(l, client)?.checked_add(4)?.checked_mul(client.rows)?.checked_mul(3)?)?
        .checked_add(result)?
        .checked_add(person)?
        .checked_add(client.rows.checked_mul(36)?)?
        .checked_add(crate::tree::fragments::authority::document_nodes(client.rows, client.segments)?)?
        .checked_add(crate::tree::fragments::budget::budget_nodes())?
        .checked_add(28)
}

pub(super) fn nodes(l: &Config, client: &jig_client::DomainLimits) -> Option<u32> {
    // Presence is existential: one Result/Proposal can appear on every row.
    // All eight feasible combinations use the same residual budget per case.
    Some(
        case(l, client, 0, 0, 0)?
            .max(case(l, client, 0, 0, 1)?)
            .max(case(l, client, 0, 1, 0)?)
            .max(case(l, client, 0, 1, 1)?)
            .max(case(l, client, 1, 0, 0)?)
            .max(case(l, client, 1, 0, 1)?)
            .max(case(l, client, 1, 1, 0)?)
            .max(case(l, client, 1, 1, 1)?),
    )
}
