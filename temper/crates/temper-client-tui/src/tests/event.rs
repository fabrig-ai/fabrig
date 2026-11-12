use super::fixtures as f;
use crate::{Event, Tui};
use skein_ui::{Act, LogEdge, NodeId, Role};

#[test]
fn each_act_is_its_event() {
    let tui = Tui::new(f::ADDRESS);
    let mut sign_in = f::view();
    f::build(&mut sign_in, &tui, &f::signed_out());
    let field = f::find(sign_in.tree(), Role::TextField, b"Link").id;
    let unbound = f::find(sign_in.tree(), Role::Heading { level: 1 }, b"temper").id;
    let mut first = f::view();
    f::build(&mut first, &tui, &f::live(Some(f::me(&[]))).0);
    let mut goal_root = f::GoalCase::READY.root();
    f::event(
        &mut goal_root,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::MessageWords {
                words: b"/goal".as_slice().into(),
            }),
        },
    );
    let mut commands = f::view();
    f::build(&mut commands, &tui, &goal_root);
    let command = f::find(commands.tree(), Role::MenuItem, b"/goal").id;
    super::foot::open_goal(&mut goal_root);
    let mut goal = f::view();
    f::build(&mut goal, &tui, &goal_root);
    let way = f::find(goal.tree(), Role::MenuItem, b"1. Plan, tasks, land whole").id;
    let cancel = f::find(goal.tree(), Role::MenuItem, b"Cancel").id;
    let rows = [
        (
            &commands,
            command,
            Act::Press,
            Some(Event::Command { command: temper_client_domain::pages::terminal::Command::Goal }),
        ),
        (&goal, way, Act::Press, Some(Event::Way { template: 4 })),
        (&goal, cancel, Act::Press, Some(Event::Act { act: temper_client_domain::pages::Act::StartCancel })),
        (&goal, way, Act::Submit, None),
        (
            &sign_in,
            field,
            Act::Input { value: Box::from(*b"pasted") },
            Some(Event::Typed { value: Box::from(*b"pasted") }),
        ),
        (&sign_in, field, Act::Submit, Some(Event::SignIn)),
        (&sign_in, field, Act::Press, None),
        (&sign_in, field, Act::Toggle, None),
        (&sign_in, field, Act::Drawing { data: Box::from([]) }, None),
        (&sign_in, field, Act::Near { edge: LogEdge::Foot }, None),
        (&sign_in, unbound, Act::Press, None),
        (&sign_in, unbound, Act::Input { value: Box::from([]) }, None),
        (&sign_in, NodeId(0), Act::Submit, None),
        (&first, NodeId(u32::MAX), Act::Press, None),
    ];
    for (view, node, act, expected) in rows {
        assert_eq!(crate::event(view, node, act), expected);
    }
    // A field removed from the displayed tree keeps no event binding.
    f::build(&mut sign_in, &tui, &f::live(Some(f::me(&[]))).0);
    assert_eq!(crate::event(&sign_in, field, Act::Submit), None);
}

#[test]
fn arrows_leave_every_screen_to_the_key_reader() {
    use temper_client_domain::{Address, Event as DomainEvent, Place};
    let mut launchpad = f::live(Some(f::me(&[b"Hello"]))).0;
    f::event(&mut launchpad, DomainEvent::Went { place: Place::Address(Address::Home { project: 1 }) });
    let mut chat = f::live(Some(f::me(&[b"Hello"]))).0;
    f::event(&mut chat, DomainEvent::Went { place: Place::Address(Address::Chat { task: 9 }) });
    for root in [&f::signed_out(), &launchpad, &chat] {
        let mut view = f::view();
        f::build(&mut view, &Tui::new(f::ADDRESS), root);
        for arrow in [crate::Arrow::Up, crate::Arrow::Down] {
            assert_eq!(crate::arrow(&view, arrow), None);
        }
    }
}

#[test]
fn the_own_commands_have_their_names() {
    let mut failures = skein_lib::List::with_capacity(u32::try_from(crate::Own::ALL.len()).unwrap());
    for (command, name) in crate::Own::ALL.into_iter().zip([
        b"/draft".as_slice(),
        b"/change",
        b"/ask",
        b"/brief",
        b"/needs",
        b"/open",
        b"/tell",
        b"/theme",
    ]) {
        if command.name() != name {
            failures.push(command).unwrap();
        }
    }
    assert!(failures.is_empty(), "own command names: {failures:?}");
}
