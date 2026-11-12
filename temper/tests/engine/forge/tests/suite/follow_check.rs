//! The independent forge runs the check, uploads its report and sees only the proved head land.
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::Duration;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as c;
use temper_engine_domain_forge_tiers as p;
use temper_engine_forge_world::{
    REPO, World,
    changes::{change_row, change_step},
    translate,
};
const STEPS: u64 = 120; // A 10-second workflow, fault-free polls, repair and the queue window.
const WORKFLOW: &[u8] = b".forgejo/workflows/follow.yml";
fn parity(world: &World) -> &top::FollowParity {
    let top::Stored::Follow(row) = world
        .stored()
        .get(&top::Key::Follow { project: 5, follow: 1 })
        .expect("the follow fixture has its saved verdict")
    else {
        panic!("follow row")
    };
    row.parity.as_ref().expect("the follow fixture has its saved verdict")
}
#[test]
#[expect(clippy::too_many_lines, reason = "one workflow story follows red through repair, restart and landing")]
fn the_follow_workflow_repairs_from_its_report_then_lands_and_keeps_green_parity() {
    let mut world = World::checking_follow(
        511,
        raw::Workflow {
            file: WORKFLOW.into(),
            jobs: Box::new([b"test".as_slice().into()]),
            latency_min: Duration::from_secs(10),
            latency_max: Duration::from_secs(10),
            silent: 0,
            passes: 1000,
            fails: Box::new([]),
            flaky: Box::new([]),
            select: None,
            report: Some(raw::Report {
                artifact: b"results".as_slice().into(),
                format: raw::ReportFormat::Lines,
                deflate: false,
            }),
            cue: Some(raw::Cue::Tests { path: b"failures".as_slice().into() }),
        },
    );
    world.keep_follow(
        1,
        Some(top::settings::Follow {
            source: top::settings::FollowSource {
                repository: api::Repository { forge: 1, repository: 3 },
                branch: b"main".as_slice().into(),
                paths: Box::new([]),
            },
            target: top::settings::FollowTarget { repository: REPO, branch: b"main".as_slice().into() },
            owner: top::settings::FollowOwner::Person(4),
            start: top::settings::FollowStart::Code,
            producer: None,
            check: top::settings::FollowCheck {
                workflow: WORKFLOW.into(),
                artifact: b"results".as_slice().into(),
                format: p::Format::Lines,
            },
        }),
    );
    world.produce_file(b"temper/51", b"failures", b"tests/handoff_1\ntests/handoff_2\n");
    world.status(b"temper/51", raw::Check::Passed);
    let pull = world.open_pull(b"temper/51", b"main");
    let head = translate::commit(world.branch(b"temper/51"));
    let mut row = change_row();
    row.pull = Some(pull);
    row.change.last_head = Some(head);
    row.change.state = c::State::Checking { head };
    row.follows = Some(top::ChangeFollows { follow: 1, commits: Box::new([[0xaa; 32], [0xbb; 32]]) });
    world.event(top::Event::Change { row });
    let mut repair = false;
    for entry in 10..STEPS {
        let decision = change_step(&mut world, entry, c::Status::Unknown, Box::new([]));
        if decision == c::Decision::Delegate(c::Delegate::Repair(c::Repair::Gate(c::FOLLOW_CHECK_GATE))) {
            repair = true;
            break;
        }
        assert!(
            !matches!(decision, c::Decision::Ready | c::Decision::Effect(c::Effect::Merge { .. })),
            "no landing before the report"
        );
    }
    assert!(repair, "failed source tests request repair");
    assert_eq!(parity(&world).verdict, top::FollowVerdict::Failing(2));
    let remarks = &world.change(51).gate_remarks[0].words;
    assert!(
        remarks
            .windows(b"tests/handoff_1\ntests/handoff_2\nRun: ".len())
            .any(|word| word == b"tests/handoff_1\ntests/handoff_2\nRun: ")
    );
    world.event(top::Event::Delegated {
        task: 51,
        child: 52,
        kind: c::Delegate::Repair(c::Repair::Gate(c::FOLLOW_CHECK_GATE)),
    });
    world.push_as(b"temper/51", b"failures", b"", 1);
    world.status(b"temper/51", raw::Check::Passed);
    let repaired = translate::commit(world.branch(b"temper/51"));
    world.event(top::Event::DelegateResult {
        task: 51,
        child: 52,
        status: c::Status::Passed,
        no_change: false,
        failed: false,
        cancelled: false,
        words: b"Fixed the port".as_slice().into(),
    });
    let mut landed = false;
    let mut restarted = false;
    for entry in STEPS..STEPS * 3 {
        change_step(&mut world, entry, c::Status::Unknown, Box::new([]));
        if !restarted
            && world.change(51).follow_check.as_ref().is_some_and(|check| {
                check.head == repaired && check.flight.as_ref().is_some_and(|flight| flight.start.is_some())
            })
        {
            world.restart();
            restarted = true;
        }
        if matches!(world.change(51).change.state, c::State::Landed { .. } | c::State::Cleared { .. }) {
            landed = true;
            break;
        }
    }
    assert!(landed, "{:?}", world.change(51).change);
    assert!(restarted, "the dispatched flight crosses restart");
    let parity = parity(&world);
    assert_eq!((parity.source, parity.head, &parity.verdict), ([0xbb; 32], repaired, &top::FollowVerdict::Passed));
    assert!(!parity.run.is_empty());
    let calls = world.take_calls();
    let dispatches: Vec<_> = calls
        .iter()
        .filter_map(|op| match op {
            api::Op::Write(api::Write::Dispatch { workflow, inputs, .. }) if workflow.as_ref() == WORKFLOW => {
                Some(inputs)
            }
            _ => None,
        })
        .collect();
    assert_eq!(dispatches.len(), 2, "red and repaired heads each dispatch once, including across restart");
    assert!(dispatches.iter().all(|inputs| {
        inputs.iter().any(|input| input.name.as_ref() == b"temper_follows" && input.value.as_ref() == [b'b'; 64])
    }));
}
