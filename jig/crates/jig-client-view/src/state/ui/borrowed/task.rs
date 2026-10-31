use super::Borrowed;
use crate::state::page::task::{Plan, PlanFilter, TaskPage};
use jig_client::{
    Latest,
    contract::{Delivery, OwnerTree, Phase},
};
#[derive(Debug)]
pub struct Task<'a> {
    pub state: &'a TaskPage,
    pub plan: PlanView<'a>,
}
impl core::ops::Deref for Task<'_> {
    type Target = TaskPage;
    fn deref(&self) -> &TaskPage {
        self.state
    }
}
#[derive(Debug)]
pub struct PlanView<'a> {
    pub state: &'a Plan,
    pub owner: Option<&'a OwnerTree>,
}
impl core::ops::Deref for PlanView<'_> {
    type Target = Plan;
    fn deref(&self) -> &Plan {
        self.state
    }
}
impl PlanView<'_> {
    #[must_use]
    pub fn owner_visible(&self, row: &jig_client::contract::OwnerTreeRow) -> bool {
        match self.filter {
            PlanFilter::All => true,
            PlanFilter::Live => crate::state::page::conversations::live(row.chip.phase),
            PlanFilter::Held => {
                let mut held = row.chip.phase == Phase::Held;
                if let Some(owner) = self.owner {
                    for fold in &owner.folded {
                        held |= fold.parent == row.chip.task && fold.held_below > 0;
                    }
                }
                held
            }
        }
    }
}
impl<'a> Borrowed<'a> {
    #[must_use]
    pub fn task_page(&self) -> Task<'a> {
        let mut plan = PlanView { state: &self.ui.task_page.plan, owner: None };
        for (handle, _) in &self.ui.handles.watches {
            let Some(document) = self.client.document(*handle) else {
                continue;
            };
            match document.latest {
                Some(Latest::Jig(Delivery::OwnerTree(tree))) if Some(tree.root.task) == self.ui.shown_task() => {
                    plan.owner = Some(tree);
                }

                Some(Latest::Jig(_) | Latest::Family(_)) | None => {}
            }
        }
        Task { state: &self.ui.task_page, plan }
    }
}
