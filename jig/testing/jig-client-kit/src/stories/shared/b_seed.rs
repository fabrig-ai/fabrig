//! Starting facts for project and conversation stories (jig/docs/design/client/domain/testing.md, 4–5).
use crate::{Scripted, joint::engine::Driver};
use jig_client_domain::contract as c;
use jig_client_scripted_engine::{Change, Rows, Source};
use skein_lib::{Time, Wall};

#[derive(Clone, Copy, Debug)]
pub enum Facts {
    /// Equal priorities make moving the second goal up amend only the first.
    Goals,
    HeldConversation,
    FamilyResource,
    NoChatTemplate,
    GoalOffer {
        procedure: bool,
    },
    ProposedGoal,
    Delegates,
    InboxOrder,
    OwnChats,
}

/// One object of project 1, as every person story's scripted seed writes it.
pub(super) fn install(front: &mut Scripted, key: c::ObjectKey, value: c::ReadValue) {
    front.engine.act(Time::ZERO, Wall::EPOCH, Change::Object { project: 1, key, value }).expect("object fits");
}

/// The rows of one inbox group, for one party.
pub(super) fn inbox_source(front: &mut Scripted, party: u64, group: c::InboxReadGroup, rows: Vec<c::InboxEntry>) {
    front
        .engine
        .act(
            Time::ZERO,
            Wall::EPOCH,
            Change::Source(Source {
                party,
                read: c::Read::Inbox { group, project: None, kind: None, goal: None, after: None },
                rows: Rows::Inbox(rows),
            }),
        )
        .expect("inbox fits");
}

pub fn scripted(front: &mut Scripted, facts: Facts) {
    match facts {
        Facts::InboxOrder => super::b_inbox::scripted(front),
        Facts::OwnChats => {}
        Facts::ProposedGoal | Facts::Delegates => {
            super::b_scenarios::scripted(front, facts);
        }
        Facts::FamilyResource => super::b_family::scripted(front),
        Facts::HeldConversation => super::b_held::scripted(front),
        Facts::Goals => {
            for (number, words) in [(1, b"First goal".as_slice()), (2, b"Second goal".as_slice())] {
                let mut task = crate::stories::conversations::chat(number, words, Wall::EPOCH);
                task.flags.conversation = false;
                task.flags.listed = true;
                task.requester = c::Requester::Person(1);
                task.label = 2;
                task.priority = Some(0);
                task.changed = Wall::from_nanos(number);
                install(front, c::ObjectKey::Task(number), c::ReadValue::Task(Box::new(task)));
            }
            let mut project = crate::stories::project();
            project.acts = crate::test::list([c::Offered {
                act: c::ActKind::Reorder,
                admission: c::Admission::Allowed,
                effect: None,
            }]);
            front.engine.act(Time::ZERO, Wall::EPOCH, Change::Project(Box::new(project))).expect("project fact fits");
        }
        Facts::GoalOffer { procedure } => {
            let mut frame = crate::stories::frame(1);
            if procedure {
                let bounds = &mut frame
                    .projects
                    .get_mut(0)
                    .expect("seed project")
                    .creating
                    .as_mut()
                    .expect("creating offer")
                    .within;
                bounds.charters.clear();
                bounds.procedures = crate::test::list([c::CreateProcedure { connector: 2, code: 7 }]);
            }
            front.engine.act(Time::ZERO, Wall::EPOCH, Change::Me(frame)).expect("goal offer fits");
            front
                .engine
                .act(Time::ZERO, Wall::EPOCH, Change::Project(Box::new(crate::stories::project())))
                .expect("project fits");
        }
        Facts::NoChatTemplate => {
            let mut frame = crate::stories::frame(1);
            frame.projects.get_mut(0).expect("seed project").templates.clear();
            front.engine.act(Time::ZERO, Wall::EPOCH, Change::Me(frame)).expect("frame fits");
        }
    }
}
pub fn core(driver: &mut Driver, now: Time, wall: Wall, facts: Facts) {
    use jig_fake_parties::{Act, Ask, Party, Role};
    match facts {
        Facts::InboxOrder => super::b_inbox::core(driver, now, wall),
        Facts::ProposedGoal | Facts::Delegates => {
            super::b_scenarios::core(driver, now, wall, facts);
        }
        Facts::FamilyResource => super::b_family::core(driver, now, wall),
        Facts::HeldConversation => super::b_held::core(driver, now, wall),
        Facts::Goals => {
            const SPACING: skein_lib::Duration = skein_lib::Duration::from_millis(20);
            driver.scripted_party(
                Party::new(
                    1,
                    Role::Owner,
                    1_000_000,
                    Box::new([
                        Act::SignIn { provider: 0, subject: 7_u64.to_be_bytes().into() },
                        Act::Request {
                            key: [71; 16],
                            ask: Ask::Goal {
                                words: b"First goal".as_slice().into(),
                                budget: 10,
                                priority: 0,
                                deadline: None,
                            },
                        },
                        Act::Pause { for_: SPACING },
                        Act::Request {
                            key: [72; 16],
                            ask: Ask::Goal {
                                words: b"Second goal".as_slice().into(),
                                budget: 10,
                                priority: 0,
                                deadline: None,
                            },
                        },
                    ]),
                ),
                now,
                wall,
            );
        }
        Facts::GoalOffer { .. } | Facts::NoChatTemplate | Facts::OwnChats => {}
    }
}
