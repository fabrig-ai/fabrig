use super::fixtures;
use crate::{Kind, setup::Mode, steps};
use alloc::boxed::Box;
#[test]
fn startup_runs_only_serve_with_directory_flags() {
    let commands = steps(&flags(), Box::new([]));
    assert!(commands.asked.is_empty());
    assert_eq!(commands.always.len(), 1);
    let step = &commands.always[0];
    assert_eq!(step.name(), b"serve");
    assert_eq!(step.kind(), Kind::Long);
    let expected: &[&[u8]] =
        &[b"internal", b"serve", b"--config", b"/config", b"--data", b"/data", b"--cache", b"/cache", b"--box"];
    assert_eq!(step.arguments().len(), expected.len());
    for (argument, expected) in step.arguments().iter().zip(expected) {
        assert_eq!(argument.as_ref(), *expected);
    }
}
#[test]
fn serve_holds_only_fixed_document_ports_in_their_written_order() {
    let ephemeral = ([127, 0, 0, 1], 0).into();
    let clients = ([127, 0, 0, 1], 8367).into();
    let hooks = ([127, 0, 0, 1], 8371).into();
    let commands = steps(&flags(), Box::new([ephemeral, clients, ephemeral, hooks]));
    assert_eq!(commands.always[0].listeners(), &[clients, hooks]);
}
fn flags() -> Box<[Box<[u8]>]> {
    Box::new([
        Box::from(b"--config".as_slice()),
        Box::from(b"/config".as_slice()),
        Box::from(b"--data".as_slice()),
        Box::from(b"/data".as_slice()),
        Box::from(b"--cache".as_slice()),
        Box::from(b"/cache".as_slice()),
    ])
}

#[test]
fn owned_group_has_the_asked_groups_steps() {
    for (runner, fetch_forgejo, fetch_runner, listens) in [
        (true, false, false, false),
        (false, false, false, true),
        (true, true, false, true),
        (true, false, true, false),
    ] {
        let setup = fixtures::setup();
        let mut entry = setup.entries[1].clone();
        entry.forge = 7;
        entry.asked = true;
        let Mode::Owned(owned) = &mut entry.mode else { panic!("owned") };
        if !runner {
            owned.runner = None;
        }
        owned.fetch.forgejo = fetch_forgejo;
        owned.fetch.runner = fetch_runner;
        let group = crate::step::owned_group(7, owned, &flags(), listens);
        assert_eq!(group.forge, 7);
        assert_eq!(group.label.as_ref(), b"Forge 7 (Forgejo)");
        assert!(group.recorded);
        assert_eq!(group.before, 0);
        assert_eq!(group.steps.len(), 2 + usize::from(runner) + usize::from(fetch_forgejo || fetch_runner));
        let forgejo = &group.steps[1 + usize::from(fetch_forgejo || fetch_runner)];
        assert_eq!(forgejo.1.listeners(), if listens { core::slice::from_ref(&owned.listen) } else { &[] });
        if fetch_forgejo || fetch_runner {
            assert_eq!(group.steps[0].0, crate::Stage::Fetch);
            assert_eq!(group.steps[0].1.name(), b"fetch 7");
        }
    }
}
