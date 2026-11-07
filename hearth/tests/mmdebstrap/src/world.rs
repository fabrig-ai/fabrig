use crate::reference::Decoded;
use hearth_protocol_mmdebstrap::{
    limits::Limits,
    output::{self, Reader, Seen},
    packages::{self, Found, Stanzas},
    tail::Cause,
};
use skein_heap::Meter;
use skein_lib::{
    Env, Queue, Time, Wall,
    stream::{Down, Fault, Up},
};
use skein_world::stream::Wire;
/// End normally, at a byte prefix, or with a pipe fault at a byte prefix.
#[derive(Clone, Copy, Debug)]
pub enum End {
    Whole,
    Cut(usize),
    Break(usize),
}
/// Observations made while reading, independent of fragmentation.
#[derive(Debug)]
pub struct Run {
    pub decoded: Decoded,
}
#[must_use]
pub fn end(rng: &mut skein_lib::Rng, bytes: &[u8]) -> End {
    crate::generate::end(rng, bytes)
}

/// What one entry point left in a machine's queues, before the driver took them.
/// Orders follow hearth/docs/design/domain/render.md, 8.5.
#[derive(Debug)]
pub struct Step<T> {
    pub above: Vec<T>,
    pub below: Vec<Down>,
}

fn snapshot<T>(above: &mut Queue<T>, below: &mut Queue<Down>) -> Step<T> {
    let mut step = Step { above: Vec::new(), below: Vec::new() };
    while let Some(item) = above.pop() {
        step.above.push(item);
    }
    while let Some(item) = below.pop() {
        step.below.push(item);
    }
    step
}

fn take_step<T>(wire: &mut Wire, step: &Step<T>) {
    for down in &step.below {
        wire.take(match down {
            Down::Demand { read, room } => Down::Demand { read: *read, room: *room },
            Down::Send(bytes) => Down::Send(bytes.clone()),
            Down::Finish => Down::Finish,
        });
    }
}

/// A pipe's reader at seeded cuts. The first step is `open`'s; the last three are strays past the
/// terminal event: `Up::End`, `Up::Failed(Fault::Reset)` and `Up::Bytes` of an `Inst` line.
#[must_use]
pub fn output_steps(bytes: &[u8], seed: u64, limits: &Limits, end: End) -> Vec<Step<Seen>> {
    let (bytes, broken) = match end {
        End::Whole => (bytes, false),
        End::Cut(at) => (&bytes[..at.min(bytes.len())], false),
        End::Break(at) => (&bytes[..at.min(bytes.len())], true),
    };
    let mut wire = Wire::new(seed);
    wire.write(bytes);
    wire.eof = true;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    let mut reader = Reader::new(limits);
    output::open(&mut reader, &env, &mut below);
    let mut steps = vec![snapshot(&mut above, &mut below)];
    take_step(&mut wire, &steps[0]);
    for _ in 0..bytes.len() * 10 + 100 {
        // Steps retain the observations; the wire's trace need not accumulate.
        wire.trace.clear();
        if let Some(mut event) = wire.answer() {
            if broken && matches!(event, Up::End) {
                event = Up::Failed(Fault::Reset);
            }
            output::up(&mut reader, &env, event, &mut above, &mut below);
            let step = snapshot(&mut above, &mut below);
            let ended = step.above.iter().any(|seen| matches!(seen, Seen::Ended | Seen::Failed(_)));
            take_step(&mut wire, &step);
            steps.push(step);
            if ended {
                for stray in
                    [Up::End, Up::Failed(Fault::Reset), Up::Bytes(Box::from(&b"Inst stray (1 Debian [amd64])\n"[..]))]
                {
                    output::up(&mut reader, &env, stray, &mut above, &mut below);
                    steps.push(snapshot(&mut above, &mut below));
                }
                return steps;
            }
        }
    }
    panic!("reader did not drain seed {seed}");
}

/// The same for a list's reader; its stray bytes are a `Package: ` line.
#[must_use]
pub fn index_steps(bytes: &[u8], seed: u64, limits: &Limits, end: End) -> Vec<Step<Found>> {
    let (bytes, broken) = match end {
        End::Whole => (bytes, false),
        End::Cut(at) => (&bytes[..at.min(bytes.len())], false),
        End::Break(at) => (&bytes[..at.min(bytes.len())], true),
    };
    let mut wire = Wire::new(seed);
    wire.write(bytes);
    wire.eof = true;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    let mut reader = Stanzas::new(limits);
    packages::open(&mut reader, &env, &mut below);
    let mut steps = vec![snapshot(&mut above, &mut below)];
    take_step(&mut wire, &steps[0]);
    for _ in 0..bytes.len() * 10 + 100 {
        // Steps retain the observations; the wire's trace need not accumulate.
        wire.trace.clear();
        if let Some(mut event) = wire.answer() {
            if broken && matches!(event, Up::End) {
                event = Up::Failed(Fault::Reset);
            }
            packages::up(&mut reader, &env, event, &mut above, &mut below);
            let step = snapshot(&mut above, &mut below);
            let ended = step.above.iter().any(|found| matches!(found, Found::Ended | Found::Failed(_)));
            take_step(&mut wire, &step);
            steps.push(step);
            if ended {
                for stray in [Up::End, Up::Failed(Fault::Reset), Up::Bytes(Box::from(&b"Package: stray\n"[..]))] {
                    packages::up(&mut reader, &env, stray, &mut above, &mut below);
                    steps.push(snapshot(&mut above, &mut below));
                }
                return steps;
            }
        }
    }
    panic!("list reader did not drain seed {seed}");
}
#[must_use]
pub fn read_output(bytes: &[u8], seed: u64, limits: &Limits, end: End) -> Run {
    let mut wire = Wire::new(seed);
    let (bytes, broken) = match end {
        End::Whole => (bytes, false),
        End::Cut(at) => (&bytes[..at.min(bytes.len())], false),
        End::Break(at) => (&bytes[..at.min(bytes.len())], true),
    };
    wire.write(bytes);
    wire.eof = true;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    let mut reader = Reader::new(limits);
    output::open(&mut reader, &env, &mut below);
    let mut decoded = Decoded { insts: Vec::new(), fetched: 0, error: None, tail: Vec::new(), cause: Cause::Other };
    for _ in 0..bytes.len() * 10 + 100 {
        if let Some(down) = below.pop() {
            wire.take(down);
        }
        if let Some(mut event) = wire.answer() {
            if broken && matches!(event, Up::End) {
                event = Up::Failed(Fault::Reset);
            }
            output::up(&mut reader, &env, event, &mut above, &mut below);
        }
        match above.pop() {
            Some(Seen::Inst(inst)) => decoded.insts.push(inst),
            Some(Seen::Fetched(bytes)) => decoded.fetched = bytes,
            Some(Seen::Failed(error)) => {
                decoded.error = Some(error);
                finish(&reader, &mut decoded);
                return Run { decoded };
            }
            Some(Seen::Ended) => {
                finish(&reader, &mut decoded);
                if let Some(withdraw) = below.pop() {
                    wire.take(withdraw);
                }
                return Run { decoded };
            }
            None => {}
        }
    }
    panic!("reader did not drain seed {seed}");
}
/// Check each entry point under a heap meter, handing emitted payloads out.
#[must_use]
pub fn read_output_metered(bytes: &[u8], limits: &Limits, ceiling: u64) -> u64 {
    use skein_lib::{
        Intake,
        stream::{Down, Read},
    };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    let mut intake = Intake::with_capacity(
        u32::try_from(bytes.len()).expect("bounded fixture value").max(output::largest_demand(limits)),
    );
    intake.append(bytes).expect("bounded fixture value");
    let meter = Meter::new();
    meter.start();
    let mut reader = Reader::new(limits);
    let mut most = meter.check(meter.end(), ceiling, &"new");
    meter.start();
    output::open(&mut reader, &env, &mut below);
    most = most.max(meter.check(meter.end(), ceiling, &"open"));
    while let Some(down) = below.pop() {
        let read = match down {
            Down::Demand { read: Read::Nothing, .. } => break,
            Down::Demand { read, .. } => read,
            other @ (Down::Send(_) | Down::Finish) => panic!("unexpected {other:?}"),
        };
        let event = intake.meet(read).map_or(Up::End, Up::Bytes);
        meter.start();
        output::up(&mut reader, &env, event, &mut above, &mut below);
        let step = meter.end();
        drop(above.pop());
        most = most.max(meter.check(step, ceiling, &"up"));
    }
    meter.start();
    output::close(&mut reader, &mut below);
    most.max(meter.check(meter.end(), ceiling, &"close"))
}

fn finish(reader: &Reader, decoded: &mut Decoded) {
    decoded.cause = reader.tail().cause().clone();
    decoded.tail = reader.tail().lines().iter().map(|line| line.to_vec()).collect();
}

/// Read a Packages list through the same seeded lower stream as output.
#[must_use]
pub fn read_index(bytes: &[u8], seed: u64, limits: &Limits, end: End) -> crate::reference::IndexDecoded {
    use hearth_protocol_mmdebstrap::packages::{self, Found, Stanzas};
    let (bytes, broken) = match end {
        End::Whole => (bytes, false),
        End::Cut(at) => (&bytes[..at.min(bytes.len())], false),
        End::Break(at) => (&bytes[..at.min(bytes.len())], true),
    };
    let mut wire = Wire::new(seed);
    wire.write(bytes);
    wire.eof = true;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    let mut reader = Stanzas::new(limits);
    packages::open(&mut reader, &env, &mut below);
    let mut decoded = crate::reference::IndexDecoded { stanzas: Vec::new(), error: None };
    for _ in 0..bytes.len() * 10 + 100 {
        if let Some(down) = below.pop() {
            wire.take(down);
        }
        if let Some(mut event) = wire.answer() {
            if broken && matches!(event, Up::End) {
                event = Up::Failed(Fault::Reset);
            }
            packages::up(&mut reader, &env, event, &mut above, &mut below);
        }
        while let Some(found) = above.pop() {
            match found {
                Found::Stanza(stanza) => decoded.stanzas.push(stanza),
                Found::Ended => return decoded,
                Found::Failed(error) => {
                    decoded.error = Some(error);
                    return decoded;
                }
            }
        }
    }
    panic!("list reader did not drain seed {seed}");
}
/// Meter each list entry point while excluding the world's input and queues.
#[must_use]
pub fn read_index_metered(bytes: &[u8], limits: &Limits, ceiling: u64) -> u64 {
    use hearth_protocol_mmdebstrap::packages::{self, Stanzas};
    use skein_lib::{
        Intake,
        stream::{Down, Read},
    };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    let mut intake = Intake::with_capacity(u32::try_from(bytes.len()).expect("fixture fits"));
    intake.append(bytes).expect("fixture fits");
    let meter = Meter::new();
    meter.start();
    let mut reader = Stanzas::new(limits);
    let mut most = meter.check(meter.end(), ceiling, &"new");
    meter.start();
    packages::open(&mut reader, &env, &mut below);
    most = most.max(meter.check(meter.end(), ceiling, &"open"));
    while let Some(down) = below.pop() {
        let read = match down {
            Down::Demand { read: Read::Nothing, .. } => break,
            Down::Demand { read, .. } => read,
            other @ (Down::Send(_) | Down::Finish) => panic!("unexpected {other:?}"),
        };
        let event = intake.meet(read).map_or(Up::End, Up::Bytes);
        meter.start();
        packages::up(&mut reader, &env, event, &mut above, &mut below);
        let step = meter.end();
        while let Some(found) = above.pop() {
            drop(found);
        }
        most = most.max(meter.check(step, ceiling, &"up"));
    }
    meter.start();
    packages::close(&mut reader, &mut below);
    most.max(meter.check(meter.end(), ceiling, &"close"))
}

/// Observe both pipes, preferring stderr's failure tail as the real owner does.
#[must_use]
pub fn read_recording(case: &crate::recordings::Case, seed: u64, limits: &Limits) -> Run {
    let mut stderr = read_output(&case.recording.stderr, seed, limits, End::Whole).decoded;
    let stdout = read_output(&case.recording.stdout, seed ^ 0x3802_0000, limits, End::Whole).decoded;
    stderr.insts.extend(stdout.insts);
    if stdout.fetched != 0 {
        stderr.fetched = stdout.fetched;
    }
    stderr.error = stderr.error.or(stdout.error);
    if case.recording.exit.as_ref() == "0" {
        stderr.tail.clear();
        stderr.cause = Cause::Other;
    } else if stderr.tail.is_empty() {
        stderr.tail = stdout.tail;
        stderr.cause = stdout.cause;
    }
    Run { decoded: stderr }
}
