//! Memory measurements for the suite; heap stays a dev dependency.
use hearth_protocol_config::{
    json::{self, Decoder, Request},
    limits::Limits,
};
use skein_heap::Meter;
use skein_lib::{
    Env, Queue, Time, Wall,
    stream::{Down, Read, Up},
};

pub fn metered(document: &[u8], limits: &Limits, ceiling: u64, what: &str) -> u64 {
    measure(document, limits, ceiling, what, false)
}

pub fn metered_declaration(document: &[u8], limits: &Limits, ceiling: u64, what: &str) -> u64 {
    measure(document, limits, ceiling, what, true)
}
fn measure(document: &[u8], limits: &Limits, ceiling: u64, what: &str, declare: bool) -> u64 {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    let mut offset = 0;
    let meter = Meter::new();
    meter.start();
    let mut decoder = Decoder::new(limits);
    let mut most = meter.check(meter.end(), ceiling, &what);
    meter.start();
    json::down(&mut decoder, &env, Request::Decode, &mut above, &mut below);
    most = most.max(meter.check(meter.end(), ceiling, &what));
    for _ in 0..document.len() * 100 + 1000 {
        if let Some(event) = above.pop() {
            match event {
                json::Event::Decoded(value) if declare => {
                    meter.start();
                    let declared =
                        hearth_protocol_config::declaration::declaration(value, limits).expect("valid declaration");
                    most = most.max(meter.check(meter.end(), ceiling, &what));
                    drop(declared);
                }
                json::Event::Decoded(_) | json::Event::Failed(_) | json::Event::Closed => {}
            }
            meter.start();
            json::down(&mut decoder, &env, Request::Close, &mut above, &mut below);
            let step = meter.end();
            drop(above.pop());
            drop(below.pop());
            most = most.max(meter.check(step, ceiling, &what));
            return most;
        }
        let request = below.pop().expect("one demand");
        let read = match request {
            Down::Demand { read, .. } => read,
            Down::Send(_) | Down::Finish => panic!("only reads"),
        };
        let remaining = &document[offset..];
        let count = match read {
            Read::Fill(n) => usize::try_from(n).expect("u32 fits"),
            Read::Scan { until, max } => {
                let max = usize::try_from(max).expect("u32 fits");
                remaining[..remaining.len().min(max)]
                    .windows(until.as_bytes().len())
                    .position(|w| w == until.as_bytes())
                    .map_or(max, |n| n + until.as_bytes().len())
            }
            Read::Nothing | Read::Line { .. } | Read::Arrived { .. } => panic!("tokenizer framing"),
        };
        let event = if count <= remaining.len() {
            offset += count;
            Up::Bytes(remaining[..count].into())
        } else {
            Up::End
        };
        meter.start();
        json::up(&mut decoder, &env, event, &mut above, &mut below);
        most = most.max(meter.check(meter.end(), ceiling, &what));
    }
    panic!("memory run settles: {what}");
}
