//! Random check lifetimes, engine replies and ancestry on a small history.
use crate::tests::{reds::{prepared, notify, ended}, fixtures, harness::Harness};
use crate::{Event, Request, agent, host, wire};
use alloc::boxed::Box;
use skein_lib::{Duration, Queue, List, Rng, Set, Time, Writer};
use temper_worker_protocol_git::{invocation::{self, Command}, reds::CheckCommand, sequence::{self, Next}};
use temper_worker_domain_checkout::git::{self, Commit, Done, Op};
struct Ask { number: u64, generation: u64, reds: Box<[git::Red]> }
struct World {
    parents: [u8; 8], base: u8, asks: Queue<Ask>, generation: u64, deadline: Time, active: bool,
    expected: Option<Box<[u8]>>, file: Option<Box<[u8]>>, rng: Rng,
}
fn contains(parents: &[u8], head: u8, mut base: u8) -> bool {
    if usize::from(head) >= parents.len() { return false; }
    for _ in 0..8_u32 { if head == base { return true; } if base == 0 { return false; } base = *parents.get(usize::from(base)).expect("history node"); }
    panic!("acyclic history");
}
fn snapshot(world: &mut World) -> Box<[git::Red]> {
    let mut reds = List::with_capacity(16);
    for _ in 0..16_u32 {
        let head = u8::try_from(world.rng.below(9)).unwrap();
        let key = b'a'.checked_add(u8::try_from(world.rng.below(4)).unwrap()).unwrap();
        let mut name = *b"test a::t x"; *name.get_mut(5).unwrap() = key;
        reds.push(git::Red { key: Box::new(name), head: Box::new([head; 32]) }).unwrap();
    }
    reds.into_boxed()
}
fn expected(world: &World, reds: &[git::Red]) -> Box<[u8]> {
    let mut keys = Set::with_capacity(16);
    for red in reds { if contains(&world.parents, *red.head.first().unwrap(), world.base) { keys.insert(red.key.clone()).unwrap(); } }
    let mut size = b"temper-reds 1\n".len(); for key in &keys { size = size.checked_add(key.len()).unwrap().checked_add(3).unwrap(); }
    let mut out = Writer::new(size); out.put(b"temper-reds 1\n").unwrap();
    for key in &keys { out.put(b"- ").unwrap(); out.put(key).unwrap(); out.put(b"\n").unwrap(); } out.finish()
}
fn render(world: &World, op: &Op) -> Option<Box<[u8]>> {
    let mut next = sequence::first(op);
    for _ in 0..128_u32 {
        next = match next {
            Next::Run(Command::Reds(command)) => {
                let (code, stdout): (u8, Box<[u8]>) = match &command {
                    CheckCommand::Pin => (0, Box::new([])), CheckCommand::Merge => (1, Box::new([])),
                    CheckCommand::Base { .. } => {
                        let hex = invocation::hex(Commit::new([world.base; 32])); let mut out = Writer::new(hex.len().checked_add(1).unwrap());
                        out.put(&hex).unwrap(); out.put(b"\n").unwrap(); (0, out.finish())
                    }
                    CheckCommand::Ancestor { expected, .. } => {
                        let head = *expected.first().unwrap(); let code = if contains(&world.parents, head, world.base) { 0 } else if head < 8 { 1 } else { 128 }; (code, Box::new([]))
                    }
                    CheckCommand::Path { .. } => (0, Box::from(b".git/temper\n".as_slice())),
                };
                sequence::after(op, &Command::Reds(command), code, &stdout, b"", 0, &fixtures::limits().checkout)
            }
            Next::Reds { content, .. } => return content,
            Next::End(_) | Next::Run(_) | Next::Snapshot { .. } | Next::Make | Next::Store | Next::InstallStore { .. } | Next::Alternates | Next::Scan { .. } => panic!("only check operations"),
        };
    }
    panic!("bounded operation");
}
fn pump(h: &mut Harness, world: &mut World, rows: Box<[Request]>, answer: Option<wire::Answer>) {
    let mut answer = answer; let mut pending = Queue::uncapped(); for row in rows { pending.push(row); }
    for _ in 0..256_u32 {
        let Some(row) = pending.pop() else { return };
        let event = if let Request::RedsAsk { owner, check, .. } = row {
            world.generation = check; world.deadline = h.env.now.saturating_add(Duration::from_millis(5000));
            Event::RedsAsk { owner, check, bytes: Box::from(b"reds".as_slice()) }
        } else if let Request::Link(host::LinkUp::Ask { number, .. }) = row {
            let reds = snapshot(world); world.asks.push(Ask { number, generation: world.generation, reds }); continue;
        } else if let Request::RedsAnswer { owner, check, .. } = row { Event::RedsAnswer { owner, check, answer: answer.take() } }
        else if let Request::Io { owner, op: Op::Reds { at, action }, .. } = row {
            world.file = render(world, &Op::Reds { at, action }); Event::Done { owner, done: Done::Succeeded }
        } else { continue };
        for row in h.step(event) { pending.push(row); }
    }
    panic!("root settles");
}
#[test]
fn random_checks_use_only_their_answers_contained_heads_and_leave_no_file_at_end() {
    for seed in 0..64_u64 {
        let mut h = prepared();
        let mut world = World { parents: [0; 8], base: 6, asks: Queue::uncapped(), generation: 0, deadline: Time::ZERO, active: false, expected: None, file: None, rng: Rng::new(seed) };
        for node in 1..8_usize { *world.parents.get_mut(node).unwrap() = u8::try_from(world.rng.below(u64::try_from(node).unwrap())).unwrap(); }
        for _ in 0..128_u32 {
            match world.rng.below(5) {
                0 | 1 => { world.active = true; world.expected = None; let rows = notify(&mut h, agent::Operation::Checks); pump(&mut h, &mut world, rows, None); }
                2 => {
                    if let Some(ask) = world.asks.pop() {
                        let refused = world.rng.chance(250);
                        if world.active && ask.generation == world.generation && h.env.now <= world.deadline && !refused { world.expected = Some(expected(&world, &ask.reds)); }
                        let mut reds = List::with_capacity(16); for red in ask.reds { reds.push(wire::Red { key: red.key, head: red.head }).unwrap(); }
                        let reply = if refused { host::AskAnswer::Refused } else { host::AskAnswer::Bytes(Box::from(b"reply".as_slice())) };
                        let rows = h.step(Event::Link(host::LinkDown::Asked { run: fixtures::RUN, attempt: fixtures::ATTEMPT, number: ask.number, answer: reply }));
                        pump(&mut h, &mut world, rows, Some(wire::Answer::Reds { reds: reds.into_boxed() }));
                    }
                }
                3 => {
                    world.active = false; world.expected = None; ended(&mut h); let mut rows = List::uncapped();
                    while let Some(row) = h.out.pop() { rows.push(row).unwrap(); } pump(&mut h, &mut world, rows.into_boxed(), None);
                }
                4 => { h.env.now = h.env.now.saturating_add(Duration::from_millis(5001)); crate::reds::fire(&mut h.domain, h.env.now); }
                _ => unreachable!(),
            }
            assert_eq!(world.file, world.expected, "seed {seed}");
        }
        ended(&mut h); let mut rows = List::uncapped(); while let Some(row) = h.out.pop() { rows.push(row).unwrap(); }
        pump(&mut h, &mut world, rows.into_boxed(), None); assert!(world.file.is_none(), "seed {seed}: no file survives the end");
    }
}
