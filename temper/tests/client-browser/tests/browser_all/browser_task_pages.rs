//! Step functions walked by `browser_pages.rs`: `plan_and_agent_steps` follows the CSV
//! change's plan, agent and task addresses; `reload_chats` checks Live, never Behind,
//! and the story's chat before and after a Chats reload
//! (temper/docs/design/client/browser.md, 4.6, item 11).
use skein_fake_person::{Find, Step as PersonStep, Target};
use skein_fake_person_browser::Step;
use skein_ui::Role;
use temper_client_browser_world::{ABSENT, FIRST_CHAT, Tier, WITHIN};

/// Follow the CSV change through its plan, agent and fresh addresses in the chat story.
pub(super) fn plan_and_agent_steps(tier: &Tier) -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |find| Step::Person(PersonStep::Press { find });
    let heading = |level, name: &[u8]| Find::exactly_named(Role::Heading { level }, name);
    let task = |number: &[u8]| Find::exactly_named(Role::Link, number);
    let change = b"Export widget's table as CSV";
    let agent = "T3 · Export widget's table as CSV".as_bytes();
    let plan_agent = || {
        task(b"Task 3")
            .within(Box::new([Target::ExactRole { role: Role::TreeItem, name: b"Task 3".as_slice().into() }]))
    };
    let mut steps = vec![
        // The card settles at its call once the turn's words show (client/README.md, 5.7).
        // A person reads the reply, then follows its card.
        see(Find::text("I’ll delegate this change in playground/widget. It will push a branch, open a pull request, check CI and land into main on Playground forge.".as_bytes())
            .within(Box::new([Target::ExactRole { role: Role::Log, name: b"Conversation".as_slice().into() }]))),
        see(Find::exactly_named(Role::Region, b"Task 2")),
        see(task(b"Task 2")
            .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Task 2".as_slice().into() }]))),
        press(
            task(b"Task 2")
                .within(Box::new([Target::ExactRole { role: Role::Region, name: b"Task 2".as_slice().into() }])),
        ),
        // A press finishes before its effects show; the title is also on the chat card (skein's browser.md, 6).
        see(heading(1, change)),
        see(heading(2, b"Its tasks")),
        Step::Address(b"/tasks/2".as_slice().into()),
        // 3: task::a_delegates_page_shows_its_brief_run_and_report follows a plan to its agent.
        see(Find::exactly_named(Role::TreeItem, b"Task 3")),
        see(plan_agent()),
        press(plan_agent()),
        see(heading(1, agent)),
        Step::Address(b"/tasks/3".as_slice().into()),
        see(Find::exactly_named(Role::Region, b"Task brief")),
        // 4: task::a_delegates_page_shows_its_brief_run_and_report and
        // b2::roadmap_b2_through_the_task_pages return through the lineage.
        press(task(b"Task 2")),
        see(heading(2, b"Its tasks")),
        Step::Address(b"/tasks/2".as_slice().into()),
        // 5: b2::roadmap_b2_through_the_task_pages starts a fresh client at a delegate's address.
        Step::Person(PersonStep::Go { address: b"/tasks/3".as_slice().into() }),
        see(heading(1, agent)),
        see(Find::exactly_named(Role::Region, b"Task brief")),
        Step::Address(b"/tasks/3".as_slice().into()),
    ];
    steps.extend(reload_chats(tier));
    steps
}

fn reload_chats(tier: &Tier) -> Vec<Step> {
    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let heading = |level, name: &[u8]| Find::exactly_named(Role::Heading { level }, name);
    let behind = || Find::exactly_named(Role::Status, b"Behind");
    let chat = || Find::exactly_named(Role::Link, FIRST_CHAT);
    vec![
        // 6: the playground's project made, the link stays up across a reload: the page's own
        // client bounds receive the box's Me (temper/docs/design/client/README.md, 2).
        Step::Person(PersonStep::Go { address: b"/chats".as_slice().into() }),
        see(heading(1, b"Talk to an agent about playground/widget.")),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        Step::Person(PersonStep::Gone { find: behind(), within: ABSENT }),
        see(chat()),
        Step::Person(PersonStep::Reload),
        see(heading(1, b"Talk to an agent about playground/widget.")),
        see(tier.states().live),
        temper_client_browser_world::connected(),
        Step::Person(PersonStep::Gone { find: behind(), within: ABSENT }),
        see(chat()),
    ]
}
