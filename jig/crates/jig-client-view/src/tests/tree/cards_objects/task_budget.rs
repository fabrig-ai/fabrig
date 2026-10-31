use crate::state::address::{Address, TaskSection};
use crate::tests::kit::Goldens;
use crate::tests::tree::cards_objects::*;

fn budget_page(row: Task) -> Tree {
    let (mut domain, out) =
        crate::tests::tree::work_pages::entered(Address::Task { task: 8, section: TaskSection::Budget });
    let watch =
        out.iter()
            .find_map(|request| match request {
                Request::Client {
                    asked:
                        jig_client::Event::Watch {
                            handle: watch,
                            subject: jig_client::Watchable::Jig(Subject::Task { .. }),
                        },
                } => Some(*watch),
                _ => None,
            })
            .unwrap();
    crate::tests::tree::pages::deliver(&mut domain, watch, Delivery::Task(Box::new(row)));
    let tree = crate::tests::tree::pages::build(&domain, 3, words()).0;
    crate::tests::tree::cards_access::check(&tree);
    assert!(tree.len() <= crate::task_nodes(&CLIENT, &(CLIENT).client, 0).unwrap());
    tree
}

#[test]
fn task_budget_page_renders_all_owner_funders_numbers_and_deadline_presence_within_the_whole_bound() {
    let mut goldens = Goldens::new("tree/cards_objects/task_budget/task_budget_page");
    for funder in [
        Funder::Task(u64::MAX),
        Funder::Pool { project: u32::MAX, person: u64::MAX, period: u64::MAX },
        Funder::Recurring { project: u32::MAX, task: u64::MAX, period: u64::MAX },
        Funder::Period { project: u32::MAX, period: u64::MAX },
    ] {
        for deadline in [None, Some(Wall::EPOCH), Some(Wall::from_nanos(u64::MAX))] {
            let mut row = task_doc(&[]);
            row.budget = Budget {
                numbers: Numbers { budget: u64::MAX, spent: 4, spent_below: 7, reserved: 11 },
                left: 17,
                funder,
                deadline,
            };
            row.authority.authority.deadline = Some(Wall::from_nanos(123));
            let tree = budget_page(row);
            goldens.check(&tree);
        }
    }
}
