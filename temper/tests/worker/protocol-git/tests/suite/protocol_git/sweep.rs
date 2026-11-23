use super::{Node, fixtures, world};

pub(crate) fn sweep_past_a_stuck_file(mut seed: u64) {
    use skein_io::kernel::Error;
    use skein_lib::{Duration, Time};
    use temper_worker_protocol_git::{Component, Event, Request};
    use temper_worker_protocol_git_world::io::{Below, Invocation, Script};

    let mut below = Below::new(Script { invocations: [Invocation::captured("version")].into(), ..Script::default() });
    for path in [b"trash/1", b"trash/2", b"trash/3"] {
        below.tree.insert(path.to_vec(), Node::File(Box::new([])));
    }
    below.remove_failure = Some((b"trash/1".to_vec(), Error::Permission));
    let mut component =
        Component::new(&fixtures::limits(), &fixtures::checkout(), fixtures::settings()).expect("valid limits");
    component.start(Time::ZERO, &mut below.out);
    world::settle(&mut component, &mut below, Time::ZERO, &mut seed);
    assert_eq!(below.out.events.pop(), Some(Event::Started));
    assert_eq!(
        below.out.events.pop(),
        Some(Event::Unswept { path: b"trash/1".as_slice().into(), error: Error::Permission })
    );
    assert!(below.out.events.is_empty(), "no repeat report on the immediate second pass");
    assert_eq!(
        below.tree.keys().filter(|path| path.starts_with(b"trash/")).map(Vec::as_slice).collect::<Vec<_>>(),
        [b"trash/1"]
    );

    below.remove_failure = None;
    let due = component.next_deadline().expect("a retry is armed");
    assert_eq!(due, Time::ZERO.saturating_add(Duration::from_secs(60)));
    component.fire(due, &mut below.out);
    world::settle(&mut component, &mut below, due, &mut seed);
    assert!(!below.tree.keys().any(|path| path.starts_with(b"trash/")));
    assert!(below.out.events.is_empty());
    assert_eq!(component.next_deadline(), None);
    component.down(due, Request::Shutdown, &mut below.out);
    world::settle(&mut component, &mut below, due, &mut seed);
    assert!(component.is_empty() && below.settled());
    skein_checks::count("seed", 1);
}
