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
    let quit = f::find(first.tree(), Role::Button, b"q to quit").id;
    let rows = [
        (
            &sign_in,
            field,
            Act::Input { value: Box::from(*b"pasted") },
            Some(Event::Typed { value: Box::from(*b"pasted") }),
        ),
        (&sign_in, field, Act::Submit, Some(Event::SignIn)),
        (&first, quit, Act::Press, Some(Event::Quit)),
        (&sign_in, field, Act::Press, None),
        (&sign_in, field, Act::Toggle, None),
        (&sign_in, field, Act::Drawing { data: Box::from([]) }, None),
        (&sign_in, field, Act::Near { edge: LogEdge::Foot }, None),
        (&first, quit, Act::Input { value: Box::from(*b"q") }, None),
        (&first, quit, Act::Submit, None),
        (&first, quit, Act::Toggle, None),
        (&first, quit, Act::Drawing { data: Box::from([]) }, None),
        (&first, quit, Act::Near { edge: LogEdge::Head }, None),
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
