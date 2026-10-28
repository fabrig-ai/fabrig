use super::*;
use crate::PlanFilter;
use jig_client::contract as c;
const HELD: u64 = 12;
const OUTPUTS: u32 = 16;
const FIRST: u64 = chat::FIRST;
pub fn held_pages() -> crate::pages::Pages {
    use crate::pages::{self, inbox::Card};
    let mut pages = pages::Pages::new();
    let limits = limits();
    let client = crate::unlinked(&limits, SEED);
    pages::step(
        &mut pages,
        &env(&limits),
        client.client(),
        pages::Event::Went { address: Address::Task { task: chat::TASK, filter: PlanFilter::All } },
        &mut Queue::with_capacity(OUTPUTS),
    );
    let c::ReadValue::Escalation(held) = acts::held(c::ActKind::Release) else { panic!("hold") };
    let mut task = chat::task();
    task.chip.task = HELD;
    task.hold = Some(c::TaskHold::Document(Box::new(held.hold)));
    task.acts = held.acts;
    pages
        .cards_mut()
        .expect("task cards")
        .push(Card {
            object: c::ObjectKey::Task(HELD),
            group: c::InboxGroup::Escalations,
            project: Some(PROJECT),
            position: None,
            unread: None,
            handle: jig_client::Handle { asker: crate::owner::Owner::Pages.asker(), number: FIRST },
            page: Some(c::ReadValue::Task(Box::new(task))),
            words: None,
            key: None,
            outcome: None,
        })
        .expect("one held card");
    pages
}
