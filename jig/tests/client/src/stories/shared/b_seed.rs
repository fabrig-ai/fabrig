//! Starting facts for project and conversation stories (jig/docs/design/client/domain/testing.md, 4–5).
use crate::{Scripted, joint::engine::Driver};
use jig_client_domain::contract as c;
use jig_client_scripted_engine::Change;
use skein_lib::{Time, Wall};

#[derive(Clone, Copy, Debug)]
pub enum Facts {
    Goals,
    HeldConversation,
    FamilyResource,
    NoChatTemplate,
    GoalOffer { procedure: bool },
    ProposedGoal,
    Delegates,
}

pub fn scripted(front: &mut Scripted, facts: Facts) {
    match facts {
        Facts::ProposedGoal | Facts::Delegates => super::b_scenarios::scripted(front, facts),
        Facts::FamilyResource => super::b_family::scripted(front),
        Facts::HeldConversation => super::b_held::scripted(front),
        Facts::Goals => {
            for (priority, words) in [(0, b"First goal".as_slice()), (1, b"Second goal".as_slice())] {
                let number = u64::from(priority) + 1;
                let mut task = crate::stories::conversations::chat(number, words, Wall::EPOCH);
                task.flags.conversation = false;
                task.flags.listed = true;
                task.requester = c::OwnerRequester::Person(1);
                task.label = 2;
                task.priority = Some(priority);
                front
                    .engine
                    .act(
                        Time::ZERO,
                        Wall::EPOCH,
                        Change::Object {
                            project: 1,
                            key: c::ObjectKey::Task(number),
                            value: c::ReadValue::Task(Box::new(task)),
                        },
                    )
                    .expect("goal fact fits");
            }
            let mut project = crate::stories::project();
            project.acts = crate::test::list([c::Offered {
                act: c::ActKind::Reorder,
                admission: c::Admission::Allowed,
                effect: None,
            }]);
            front
                .engine
                .act(Time::ZERO, Wall::EPOCH, Change::OwnerProject(Box::new(project)))
                .expect("project fact fits");
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
                .act(Time::ZERO, Wall::EPOCH, Change::OwnerProject(Box::new(crate::stories::project())))
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
        Facts::ProposedGoal | Facts::Delegates => super::b_scenarios::core(driver, now, wall, facts),
        Facts::FamilyResource => super::b_family::core(driver, now, wall),
        Facts::HeldConversation => super::b_held::core(driver, now, wall),
        Facts::Goals => {
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
                        Act::Request {
                            key: [72; 16],
                            ask: Ask::Goal {
                                words: b"Second goal".as_slice().into(),
                                budget: 10,
                                priority: 1,
                                deadline: None,
                            },
                        },
                    ]),
                ),
                now,
                wall,
            );
        }
        Facts::GoalOffer { .. } | Facts::NoChatTemplate => {}
    }
}
