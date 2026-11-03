use smith_agent_process_world::{World, bearer, charter, program, withdrawn};
use smith_channel::{InvalidStart, RunResult, StartRefusal};

fn refusal(world: &World) -> Option<InvalidStart> {
    world.answer().and_then(|answer| match answer.result() {
        RunResult::Refused(refused) => match refused.reason() {
            StartRefusal::Invalid(invalid) => Some(invalid.value().clone()),
            StartRefusal::Busy => None,
        },
        RunResult::Accepted(_) | RunResult::Parked | RunResult::Failed(_) => None,
    })
}

#[test]
fn an_uninstallable_grant_refuses_the_start_as_grants() {
    let charter = charter();
    let expected = (true, false, Some(InvalidStart::Grants), Some(skein_io::kernel::Exit::Code(0)));
    let rows = [
        (bearer(b""), false, expected.clone()),
        (bearer(b"not-for-anthropic"), true, expected.clone()),
        (withdrawn(b"account", b"value"), false, expected.clone()),
        (program(b"source"), false, expected),
    ];
    for (index, (value, anthropic, expected)) in rows.into_iter().enumerate() {
        let mut world = World::new(31, &charter);
        world.granting(value);
        if anthropic {
            world.anthropic();
        }
        let settled = world.settle();
        let admitted = world.observed().iter().any(|frame| frame.kind == 0x0106);
        assert_eq!((settled, admitted, refusal(&world), world.exit()), expected, "row {index}");
        world.assert_agent_clean();
    }
}

#[test]
fn a_later_uninstallable_grant_fails_the_run_as_credential() {
    for (value, anthropic) in [(bearer(b"not-for-anthropic"), true), (withdrawn(b"account", b"value"), false)] {
        let mut world = World::new(32, &charter());
        if anthropic {
            world.anthropic();
        }
        world.refreshing(value);
        assert!(world.settle());
        assert!(world.answer().is_none());
        let errors = String::from_utf8(world.errors()).expect("stderr UTF-8");
        assert_eq!(
            errors.lines().last(),
            Some("smith: the run could not answer: a host credential could not be installed")
        );
        assert_eq!(world.exit(), Some(skein_io::kernel::Exit::Code(1)));
        world.assert_agent_clean();
    }
}
