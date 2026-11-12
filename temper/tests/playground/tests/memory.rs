//! Native heap regression bounds (temper/docs/design/playground.md, 9).
use skein_heap::{Grown, Span};
use skein_lib::Wall;
use skein_sim_world::Meter;
use temper_playground::{Fakes, Start, machine, programs::Process, stories::A, story};

#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

const SEED: u64 = 7;
const STARTUP_STEPS: u64 = 10_000;

// The 2026-11-08 table in temper/docs/development/playground-memory.md,
// each figure plus a quarter. Bounds follow process names, not admission order.
const SERVICE_FIRST_STEP_HELD: i64 = 144_717 + 144_717 / 4;
const SERVICE_FIRST_STEP_PEAK: i64 = 226_963 + 226_963 / 4;
const LLM_FIRST_STEP_HELD: i64 = 27_449 + 27_449 / 4;
const LLM_FIRST_STEP_PEAK: i64 = 27_449 + 27_449 / 4;
const FORGE_FIRST_STEP_HELD: i64 = 30_002 + 30_002 / 4;
const FORGE_FIRST_STEP_PEAK: i64 = 30_002 + 30_002 / 4;
const VISITOR_FIRST_STEP_HELD: i64 = 1_054 + 1_054 / 4;
const VISITOR_FIRST_STEP_PEAK: i64 = 1_068 + 1_068 / 4;
const SERVICE_START_HELD: i64 = 239_146 + 239_146 / 4;
const SERVICE_START_PEAK: i64 = 285_982 + 285_982 / 4;
const LLM_START_HELD: i64 = 28_185 + 28_185 / 4;
const LLM_START_PEAK: i64 = 28_185 + 28_185 / 4;
const FORGE_START_HELD: i64 = 30_743 + 30_743 / 4;
const FORGE_START_PEAK: i64 = 30_787 + 30_787 / 4;
const VISITOR_START_HELD: i64 = 1_054 + 1_054 / 4;
const VISITOR_START_PEAK: i64 = 1_068 + 1_068 / 4;
const GIT_START_HELD: i64 = 3_410 + 3_410 / 4;
const GIT_START_PEAK: i64 = 5_952 + 5_952 / 4;
const DISK_FIRST_STEP_RETAINED: i64 = 9_393 + 9_393 / 4;
const DISK_START_RETAINED: i64 = 155_267 + 155_267 / 4;
const SHARED_FORGE_BEFORE_START_RETAINED: i64 = 6_981 + 6_981 / 4;

fn within(process: &str, phase: &str, figure: &str, measured: i64, bound: i64) {
    assert!(measured <= bound, "{process}, {phase}: {figure} {measured} bytes exceeds bound {bound} bytes");
}

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
        Process::Agent(_) | Process::List(_) => "agent",
        Process::Shell(_) => "shell",
        Process::Visitor(_) => "visitor",
        Process::Other(never) => match *never {},
    }
}

#[test]
fn story_as_world_at_its_first_step_and_start() {
    let (prebuilt, shared) = prebuilt();
    println!("shared forge before start: {shared} bytes (outside process spans)");
    within("shared forge", "before start", "retained", shared, SHARED_FORGE_BEFORE_START_RETAINED);
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
        let bounds = if after_start {
            vec![
                ("service (engine + worker)", SERVICE_START_HELD, SERVICE_START_PEAK),
                ("fake LLM", LLM_START_HELD, LLM_START_PEAK),
                ("fake forge HTTP", FORGE_START_HELD, FORGE_START_PEAK),
                ("visitor", VISITOR_START_HELD, VISITOR_START_PEAK),
                ("git", GIT_START_HELD, GIT_START_PEAK),
            ]
        } else {
            vec![
                ("service (engine + worker)", SERVICE_FIRST_STEP_HELD, SERVICE_FIRST_STEP_PEAK),
                ("fake LLM", LLM_FIRST_STEP_HELD, LLM_FIRST_STEP_PEAK),
                ("fake forge HTTP", FORGE_FIRST_STEP_HELD, FORGE_FIRST_STEP_PEAK),
                ("visitor", VISITOR_FIRST_STEP_HELD, VISITOR_FIRST_STEP_PEAK),
            ]
        };
        let mut names: Vec<_> = world.processes().iter().map(name).collect();
        let mut expected: Vec<_> = bounds.iter().map(|(name, _, _)| *name).collect();
        names.sort_unstable();
        expected.sort_unstable();
        assert_eq!(names, expected, "{phase}: measured processes");
        assert_eq!(world.processes().len(), world.meter().processes.len(), "{phase}: every process is metered");
        println!("{phase}: {} world steps; bytes", world.steps());
        println!("process | held | peak");
        for (at, (process, held)) in world.processes().iter().zip(&world.meter().processes).enumerate() {
            println!("{at}: {} | {} | {}", name(process), held.now, held.peak);
            let (_, held_bound, peak_bound) =
                bounds.iter().find(|(expected, _, _)| *expected == name(process)).expect("named process bound");
            within(name(process), phase, "held", held.now, *held_bound);
            within(name(process), phase, "peak", held.peak, *peak_bound);
        }
        assert!(world.meter().processes.iter().any(|held| held.peak > 0), "the allocator measured heap");
        let Process::Service(service) = &world.processes()[0] else { panic!("service is process zero") };
        let disk = service.parameters.disk.clone();
        // The disk's final owner is held outside the world so destroying it
        // measures its retained heap, including metadata and durable copies.
        drop(world);
        let retained = freed(disk);
        println!("{phase}: fake disk retained heap | {retained}");
        let bound = if after_start { DISK_START_RETAINED } else { DISK_FIRST_STEP_RETAINED };
        within("fake disk", phase, "retained", retained, bound);
        // Reset the root bindings outside the next world's process spans.
        drop(machine::fresh());
    }
}
