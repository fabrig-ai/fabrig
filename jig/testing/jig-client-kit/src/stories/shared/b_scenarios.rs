//! Starting conversations with delegated work (jig/docs/design/client/domain/testing.md, 4–5).
use super::b_seed::{Facts, inbox_source, install};
use crate::{Scripted, joint::engine::Driver};
use jig_client_domain::contract as c;
use jig_client_scripted_engine::Change;
use skein_lib::{Time, Wall};

pub fn core(driver: &mut Driver, now: Time, wall: Wall, facts: Facts) {
    use jig_fake_parties::{Act, Party, Role};
    driver.scripted_party(
        Party::new(
            1,
            Role::Owner,
            1_000_000,
            Box::new([Act::SignIn { provider: 0, subject: 7_u64.to_be_bytes().into() }]),
        ),
        now,
        wall,
    );
    let sign_in = driver
        .world
        .peers
        .as_ref()
        .expect("story peers")
        .parties
        .last()
        .expect("seed owner")
        .sign_in
        .expect("authenticated owner");
    let (configuration, limits, _) = super::b_config::setup(8510, &facts, driver.world.env().limits);
    let creating = jig_test_profile::fixtures::Creating::new(&configuration.core, limits.core.tasks.tree_tasks);
    let jig_domain_people::Ask::Create { project, mut batch } =
        jig_test_profile::fixtures::create_chat(&creating, 1, 0, b"Hello".as_slice().into())
    else {
        unreachable!("conversation recipe")
    };
    batch[0].authority.spend = 10;
    batch[0].authority.delegation.tasks = 2;
    batch[0].budget = jig_domain_people::Budget::Named;
    batch[0].contract = jig_domain_people::Contract::Report { words: 32 };
    jig_core_kit::fixtures::ask_as(&mut driver.world, sign_in, 80, jig_domain_people::Ask::Create { project, batch });
}
pub fn scripted(front: &mut Scripted, facts: Facts) {
    let mut chat = crate::stories::conversations::chat(1, b"Hello", Wall::EPOCH);
    chat.requester = c::Requester::Person(1);
    match facts {
        Facts::ProposedGoal => {
            let mut proposal = crate::stories::proposal();
            proposal.key.proposer = c::Proposer::Task(1);
            proposal.waiting.holder = c::HistoryHolder::Person(1);
            proposal.reason = c::DocumentText::complete(crate::stories::text(b"Resolve incident"));
            proposal.funding.clear();
            chat.proposals = crate::test::list([proposal.key]);
            install(front, c::ObjectKey::Proposal(proposal.key), c::ReadValue::Proposal(Box::new(proposal.clone())));
            let mut frame = crate::stories::frame(1);
            frame.needs.proposals = 1;
            frame.needs_total = 1;
            front.engine.act(Time::ZERO, Wall::EPOCH, Change::Me(frame)).expect("frame fits");
            inbox_source(
                front,
                1,
                c::InboxReadGroup::Needs,
                vec![crate::stories::inbox_entry(
                    c::ObjectKey::Proposal(proposal.key),
                    1,
                    c::InboxGroup::Proposals,
                    1,
                    Wall::EPOCH,
                    None,
                )],
            );
        }
        Facts::Delegates => {
            let mut cancelled = crate::stories::task();
            cancelled.chip.task = 2;
            cancelled.chip.title = c::DocumentText::complete(crate::stories::text(b"Cancelled delegate"));
            cancelled.requester = c::Requester::Task(1);
            cancelled.acts = crate::test::list([
                c::Offered { effect: None, act: c::ActKind::Cancel, admission: c::Admission::Allowed },
                c::Offered { effect: None, act: c::ActKind::Move, admission: c::Admission::Allowed },
            ]);
            let mut moved = cancelled.clone();
            moved.chip.task = 3;
            moved.chip.title = c::DocumentText::complete(crate::stories::text(b"Moved delegate"));
            chat.delegates = crate::test::list([
                c::LiveDelegate { chip: cancelled.chip.clone(), takeover_budget: 1 },
                c::LiveDelegate { chip: moved.chip.clone(), takeover_budget: 1 },
            ]);
            chat.chip.details.details.live_delegates = 2;
            chat.acts = crate::test::list([c::Offered {
                effect: None,
                act: c::ActKind::Cancel,
                admission: c::Admission::Allowed,
            }]);
            install(front, c::ObjectKey::Task(cancelled.chip.task), c::ReadValue::Task(Box::new(cancelled)));
            install(front, c::ObjectKey::Task(moved.chip.task), c::ReadValue::Task(Box::new(moved)));
        }
        Facts::OwnChats
        | Facts::InboxOrder
        | Facts::Goals
        | Facts::HeldConversation
        | Facts::FamilyResource
        | Facts::NoChatTemplate
        | Facts::GoalOffer { .. } => {
            unreachable!("conversation scenarios")
        }
    }
    install(front, c::ObjectKey::Task(chat.chip.task), c::ReadValue::Task(Box::new(chat)));
}
