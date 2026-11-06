//! Orders beside the service's room checks (hearth/docs/design/domain/render.md, 8.5).
use hearth_builder_world::{fixtures, protocol, simulated};
use hearth_lib::encode::Digest;
use skein_io::kernel::{Done, Exit};
use skein_sim::{Event, Summary, Text};

use super::bounds;

// Kernel queue sizes and the independent watermark that paces each pass.
const QUEUES: [(u32, u32); 4] = [(1, 1), (3, 2), (32, 30), (4_096, 4_096)];

#[test]
fn a_client_s_output_has_one_demand_out_whatever_the_chunk() {
    let mut limits = hearth::limits();
    limits.protocol.printer.chunk = 16;
    for seed in 0..2 {
        let run = protocol::with_limits(&fixtures::thin(), seed, &limits);
        assert_eq!(run.stdout, fixtures::THIN_TEXT.as_bytes(), "seed {seed}: the records whole, at 16 bytes a piece");
        assert_eq!(run.code, Some(0), "seed {seed}: done");
    }
}

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
        for (queue, watermark) in QUEUES {
            let mut limits = bounds::limits();
            limits.queue = queue;
            limits.watermark = watermark;
            let outcome = simulated::with_limits(&scenario, bounds::SEED, limits);
            if name == "resolving" && (queue, watermark) == QUEUES[0] {
                a_store_answer_precedes_the_next_index(&outcome);
            }
            let snapshot = (
                outcome.machine.contents(fixtures::OUTPUT),
                outcome.machine.contents(fixtures::ERROR),
                fixtures::client_exits(&outcome),
                fixtures::store_files(&outcome.machine),
            );
            if let Some(expected) = &baseline {
                assert_eq!(&snapshot, expected, "{name}: queue {queue}, watermark {watermark} against {:?}", QUEUES[0]);
            } else {
                baseline = Some(snapshot);
            }
        }
    }
}

// A kept index's terminal store answer releases the next index, after the
// cache record and its directory close (hearth/docs/design/domain/render.md, 8.5).
fn a_store_answer_precedes_the_next_index(outcome: &skein_world::Outcome<simulated::Proc, fixtures::Shared>) {
    let records = fixtures::simulation_records(&outcome.machine);
    let resolution = fixtures::only(&records);
    let mut indices = resolution.sources.iter().flat_map(|source| &source.indices);
    let first = indices.next().expect("InRelease");
    let next = indices.next().expect("Packages");
    let record = Text::of(format!("{}.record", fixtures::hex_digest(first.sha256.0)).as_bytes());
    let next_digest = fixtures::hex_digest(next.sha256.0);
    let next_name = outcome
        .machine
        .borrow()
        .renames
        .iter()
        .find(|rename| rename.to.as_ref() == next_digest.as_bytes())
        .expect("the next index was kept")
        .from
        .clone();
    let next_path = Text::of(
        &[fixtures::hex_digest(resolution.key.0).as_bytes(), b"/root/var/lib/apt/lists/", &next_name].concat(),
    );
    let mut record_directory = None;
    let mut order = Vec::new();
    for entry in &outcome.trace {
        if entry.pid.raw() != fixtures::CLIENT_PID {
            continue;
        }
        match entry.event {
            Event::Complete { kind: Summary::Rename { to, to_dir, .. }, result: Ok(Done::Nothing), .. }
                if to == record =>
            {
                record_directory = Some(to_dir);
            }
            Event::Complete { kind: Summary::Close { fd }, result: Ok(Done::Nothing), .. }
                if record_directory == Some(fd) =>
            {
                order.push("the first index's store answer");
            }
            Event::Submit { kind: Summary::Open { path, .. }, .. } if path == next_path => {
                order.push("opening the next index");
            }
            _ => {}
        }
    }
    assert_eq!(order, ["the first index's store answer", "opening the next index"]);
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

#[test]
fn a_pass_stops_short_of_a_whole_render() {
    // The watermark bounds the number of resumes in a pass, even when render
    // retains the first side's specs without queue entries. The watermark holds
    // every target's final change and the reply, so only the pass bound can
    // leave work pending below it.
    let mut scenario = bounds::queued();
    scenario.call = hearth::Call::Diff {
        resolve: false,
        repository: fixtures::GIT_DIR.into(),
        from: b"HEAD".as_slice().into(),
        to: b"HEAD".as_slice().into(),
        targets: Box::default(),
    };
    scenario.form = hearth::Form::JsonLines;
    let mut limits = bounds::limits();
    limits.watermark = bounds::TARGETS + 1;
    let outcome = simulated::with_limits(&scenario, bounds::SEED, limits);
    let passes: u32 = outcome
        .procs
        .iter()
        .filter_map(|proc| match proc {
            simulated::Proc::Client(client) => Some(client.bounded_passes()),
            simulated::Proc::Git(_) | simulated::Proc::Mmdebstrap(_) => None,
        })
        .sum();
    assert!(passes > 0, "the pass bound leaves render pending below the watermark: {passes}");
    let output =
        String::from_utf8(outcome.machine.contents(fixtures::OUTPUT).expect("printed changes")).expect("JSON lines");
    let changes: Vec<_> = output.lines().filter(|line| line.starts_with(r#"{"record":"change","target":"#)).collect();
    let expected: Vec<_> = (0..bounds::TARGETS)
        .map(|target| format!(r#"{{"record":"change","target":"b{target}-prod","kind":"same"}}"#))
        .collect();
    assert_eq!(changes, expected, "every target prints once, in order");
    assert_eq!(fixtures::client_exits(&outcome), [Exit::Code(0)]);
}
