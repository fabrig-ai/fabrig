//! Bounded iterative tree presentation, with held counts on folds (pages.md, 5.2).
use crate::JigBindings;
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use skein_ui::Builder;

use crate::{
    Borrowed,
    state::{action::Act, page::task::PlanFilter},
};
pub(super) fn plan(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) {
    let task = client.task_page();
    let plan = &task.plan;
    for (filter, phrase) in [
        (PlanFilter::All, JigPhrase::AllPlan),
        (PlanFilter::Held, JigPhrase::HeldPlan),
        (PlanFilter::Live, JigPhrase::LivePlan),
    ] {
        crate::tree::pages::work::control(
            b,
            words,
            bindings,
            phrase,
            Act::FilterPlan { filter },
            plan.filter == filter,
        );
    }
    if let Some(owner) = &plan.owner {
        crate::tree::fragments::tree::render(b, owner, plan, client, words, bindings);
        crate::tree::pages::cut(b, words, task.ended_cut);
        crate::tree::pages::work::cursors(
            b,
            words,
            bindings,
            task.ended_before.as_ref(),
            task.ended_after.as_ref(),
            crate::state::page::task::ENDED_WINDOW,
        );
        owner_ended(b, client, words, bindings);
        return;
    }
    owner_ended(b, client, words, bindings);
}

fn owner_ended(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) {
    if let Some(page) = &client.task_page().owner_ended {
        crate::tree::fragments::historical_pages::ended_render(b, client, page, words, bindings);
    }
}
