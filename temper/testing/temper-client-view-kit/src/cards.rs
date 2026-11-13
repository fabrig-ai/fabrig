use super as fixtures;
use jig_client::contract as c;
use skein_lib::List;

pub const FUNDS: u64 = 40;

pub const LEFT: u64 = 120;

pub const AFTER: u64 = 80;

pub fn batch_proposal() -> c::ReadValue {
    let mut task = fixtures::task::document();
    task.authority.authority.spend = 6;
    task.authority.authority.delegation.tasks = 1;
    task.authority.authority.delegation.depth = 1;
    task.spec.specification.words =
        c::DocumentText::complete(c::Text::new(Box::from(*b"Ship update\nDetails"), 32).unwrap());
    let mut resources = List::with_capacity(1);
    resources
        .push(c::Part {
            connector: fixtures::FORGE,
            kind: c::PartKind::Resource,
            key: c::PartKey::Path(fixtures::path(&[b"forge", b"github.com", b"robin", b"tally", b"branch", b"main"])),
        })
        .unwrap();
    let mut tasks = List::with_capacity(1);
    tasks
        .push(c::ProposedTask {
            number: 1,
            spec: task.spec,
            executor: c::Executor::Agent { charter: 2 },
            contract: task.result_contract,
            authority: task.authority,
            deadline: c::NewDeadline::Named,
            budget: 6,
            funder: task.budget.funder,
            dependencies: List::with_capacity(0),
            resources,
            wake: task.wake,
            priority: Some(2),
            cut: false,
        })
        .unwrap();
    let root = fixtures::chat::proposal_root();
    let mut page = None;
    for card in root.pages().cards().unwrap() {
        if let Some(value) = &card.page {
            page = Some(value.clone());
            break;
        }
    }
    let mut page = page.unwrap();
    let c::ReadValue::Proposal(proposal) = &mut page else { panic!("proposal") };
    proposal.action = c::ProposalAction::Batch(tasks);
    proposal.funding = List::with_capacity(1);
    proposal
        .funding
        .push(c::FundingOffer {
            source: c::Funder::Pool { project: fixtures::PROJECT, person: 73, period: 1 },
            needed: FUNDS,
            left: Some(LEFT),
            after: Some(AFTER),
        })
        .unwrap();
    page
}
