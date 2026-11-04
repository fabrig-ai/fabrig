//! Native heap measurements (temper/docs/design/playground.md, 8 and 11).
use skein_heap::{Grown, Span};
use skein_lib::Wall;
use skein_sim_world::Meter;
use temper_playground::{Fakes, Start, machine, programs::Process, stories::A, story};

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

const SEED: u64 = 7;
const STARTUP_STEPS: u64 = 10_000;

#[derive(Clone, Copy, Default)]
struct Held {
    now: i64,
    peak: i64,
}

// Story peers are built before admission. Charge their retained allocations to
// their process once; the shared forge stays outside the process spans.
struct Heap {
    prebuilt: [i64; 3],
    processes: Vec<Held>,
}
impl Heap {
    fn record(&mut self, at: usize, grown: Grown) {
        let held = &mut self.processes[at];
        held.peak = held.peak.max(held.now + grown.peak);
        held.now += grown.net;
    }
}
impl Meter for Heap {
    fn admit<T>(&mut self, make: impl FnOnce() -> T) -> T {
        let span = Span::start();
        let process = make();
        let grown = span.end();
        let at = self.processes.len();
        let now = self.prebuilt.get(at).copied().unwrap_or(0);
        self.processes.push(Held { now, peak: now });
        self.record(at, grown);
        process
    }
    fn around<T>(&mut self, at: usize, call: impl FnOnce() -> T) -> T {
        let span = Span::start();
        let result = call();
        self.record(at, span.end());
        result
    }
    fn release<T>(&mut self, at: usize, process: T) -> Option<u64> {
        let span = Span::start();
        drop(process);
        self.record(at, span.end());
        let held = self.processes.remove(at);
        Some(u64::try_from(held.peak).expect("a nonnegative peak"))
    }
    fn restart<T>(&mut self, at: usize, process: T, make: impl FnOnce() -> T) -> T {
        self.around(at, || {
            drop(process);
            make()
        })
    }
}

fn freed<T>(value: T) -> i64 {
    let span = Span::start();
    drop(value);
    -span.end().net
}

fn prebuilt() -> ([i64; 3], i64) {
    let Start { fakes: Fakes { llm, peer, forge }, .. } = A.start(SEED, Wall::EPOCH);
    let llm = freed(llm);
    let peer = freed(peer);
    let shared = freed(forge);
    ([0, llm, peer], shared)
}

fn name(process: &Process<()>) -> &'static str {
    match process {
        Process::Service(_) => "service (engine + worker)",
        Process::Peer(_) => "fake LLM",
        Process::Forge(_) => "fake forge HTTP",
        Process::Git(_) | Process::GitRoute(_) => "git",
        Process::Agent(_) => "agent",
        Process::Shell(_) => "shell",
        Process::Visitor(_) => "visitor",
        Process::Other(never) => match *never {},
    }
}

#[test]
#[ignore = "a measurement, run by hand: temper/docs/development/playground-memory.md"]
fn story_as_world_at_its_first_step_and_start() {
    let (prebuilt, shared) = prebuilt();
    println!("shared forge before start: {shared} bytes (outside process spans)");
    for after_start in [false, true] {
        let mut world = story::world_metered(A.start(SEED, Wall::EPOCH), Heap { prebuilt, processes: Vec::new() });
        story::step(&mut world);
        if after_start {
            for step in 1..STARTUP_STEPS {
                if !world.busy() && world.next_due().is_none_or(|due| due > world.now()) {
                    break;
                }
                if !world.busy()
                    && let Some(due) = world.next_due()
                {
                    world.advance(due);
                }
                story::step(&mut world);
                assert!(step < STARTUP_STEPS - 1, "story A did not wait idle after start");
            }
        }
        let phase = if after_start { "start idle" } else { "first step" };
        println!("{phase}: {} world steps; bytes", world.steps());
        println!("process | held | peak");
        for (at, (process, held)) in world.processes().iter().zip(&world.meter().processes).enumerate() {
            println!("{at}: {} | {} | {}", name(process), held.now, held.peak);
        }
        assert!(world.meter().processes.iter().any(|held| held.peak > 0), "the allocator measured heap");
        let Process::Service(service) = &world.processes()[0] else { panic!("service is process zero") };
        let disk = service.parameters.disk.clone();
        // The disk's final owner is held outside the world so destroying it
        // measures its retained heap, including metadata and durable copies.
        drop(world);
        println!("{phase}: fake disk retained heap | {}", freed(disk));
        // Reset the root bindings outside the next world's process spans.
        drop(machine::fresh());
    }
}
