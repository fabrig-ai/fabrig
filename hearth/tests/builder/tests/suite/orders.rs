//! Orders beside the service's room checks (hearth/docs/design/domain/render.md, 8.5).
use hearth_builder_world::{fixtures, simulated};
use hearth_lib::encode::Digest;
use skein_io::kernel::Exit;

#[path = "../../src/fixtures/bounds.rs"]
#[expect(clippy::duplicate_mod, reason = "the order and memory tests each use the task's shared bounds fixture")]
mod bounds;

// The task's smallest, intermediate and roomy service queues.
const QUEUES: [u32; 3] = [bounds::QUEUE, 32, 4_096];

fn laid() -> [fixtures::Laid; 2] {
    [1_u8, 2]
        .map(|byte| fixtures::Laid::Layer(Box::new(fixtures::build::config_over(Digest([byte; 32]), Digest([0; 32])))))
}

#[test]
fn what_a_call_prints_and_stores_does_not_depend_on_its_queues() {
    let laid = laid();
    let building = fixtures::Scenario { laid: &laid, ..fixtures::build::building() };
    for (name, scenario) in [
        ("thin", fixtures::thin()),
        ("queued", bounds::queued()),
        ("resolving", fixtures::resolving(fixtures::THIN)),
        ("building", building),
    ] {
        let mut baseline = None;
        for queue in QUEUES {
            let mut limits = bounds::limits();
            limits.queue = queue;
            let outcome = simulated::with_limits(&scenario, bounds::SEED, limits);
            let snapshot = (
                outcome.machine.contents(fixtures::OUTPUT),
                outcome.machine.contents(fixtures::ERROR),
                fixtures::client_exits(&outcome),
                fixtures::store_files(&outcome.machine),
            );
            if let Some(expected) = &baseline {
                assert_eq!(&snapshot, expected, "{name}: queue {queue} against queue {}", bounds::QUEUE);
            } else {
                baseline = Some(snapshot);
            }
        }
    }
}

#[test]
fn a_build_prints_its_layers_then_its_failure_then_its_reply() {
    let laid = laid();
    let scenario = fixtures::Scenario { laid: &laid, ..fixtures::build::building() };
    let outcome = simulated::with_limits(&scenario, bounds::SEED, bounds::limits());
    let output = String::from_utf8(outcome.machine.contents(fixtures::OUTPUT).expect("printed report")).expect("text");
    let mut lines = output.lines();
    for layer in ["packages", "software", "config"] {
        let line = lines.next().expect("each planned layer prints before the failure");
        let mut words = line.split_whitespace();
        assert_eq!(words.next(), Some(layer), "the plan's layer order: {output}");
        let _key = words.next().expect("the layer's key");
        assert_eq!(words.collect::<Vec<_>>(), ["to", "build"], "{output}");
    }
    assert_eq!(lines.next(), Some("failed       fetch packages   unavailable"), "{output}");
    assert_eq!(lines.next(), None, "nothing prints after the failure: {output}");
    assert_eq!(fixtures::client_exits(&outcome), [Exit::Code(69)], "the complete report precedes the reply");
}
