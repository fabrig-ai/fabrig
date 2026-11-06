use super::fixtures;
use crate::{
    form::{self, Form, Refusal},
    limits::{self, Limits},
    outcome::Outcome,
    printer::{self, Event, Printer, Request, Waiting},
    records::{Record, Terminal},
};
use alloc::boxed::Box;
use skein_lib::{
    Env, List, Queue, Time, Wall, Writer,
    stream::{Down, Fault, Read, Up},
};
struct Machine {
    printer: Printer,
    env: Env<Limits>,
    above: Queue<Event>,
    below: Queue<Down>,
}
struct Printed {
    bytes: Box<[u8]>,
    events: Box<[Event]>,
    largest: usize,
}
impl Machine {
    fn new(form: Form, limits: Limits) -> Self {
        Self {
            printer: Printer::new(form, &limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            above: Queue::with_capacity(printer::UP_MAX_OUT.above.max(printer::DOWN_MAX_OUT.above)),
            below: Queue::with_capacity(printer::UP_MAX_OUT.below.max(printer::DOWN_MAX_OUT.below)),
        }
    }
    fn down(&mut self, request: Request) {
        printer::down(&mut self.printer, &self.env, request, &mut self.above, &mut self.below);
        assert!(self.above.len() <= printer::DOWN_MAX_OUT.above, "down MAX_OUT above");
        assert!(self.below.len() <= printer::DOWN_MAX_OUT.below, "down MAX_OUT below");
    }
    fn up(&mut self, event: Up) {
        printer::up(&mut self.printer, &self.env, event, &mut self.above, &mut self.below);
        assert!(self.above.len() <= printer::UP_MAX_OUT.above, "up MAX_OUT above");
        assert!(self.below.len() <= printer::UP_MAX_OUT.below, "up MAX_OUT below");
    }
    fn print(&mut self, record: Record) -> Printed {
        self.down(Request::Print(record));
        let mut pieces = List::with_capacity(4096);
        let mut events = List::with_capacity(1);
        let mut largest = 0;
        let mut total = 0_usize;
        let mut granted = None;
        for _ in 0..4096_u32 {
            let mut wanted = None;
            while let Some(request) = self.below.pop() {
                match request {
                    Down::Demand { read, room } => {
                        assert_eq!(read, Read::Nothing, "write half only");
                        assert!(room > 0 && room <= self.env.limits.chunk, "room within chunk");
                        wanted = Some(room);
                    }
                    Down::Send(piece) => {
                        assert!(
                            piece.len() <= usize::try_from(granted.take().expect("one Send per grant")).unwrap(),
                            "Send within grant"
                        );
                        largest = largest.max(piece.len());
                        total = total.checked_add(piece.len()).unwrap();
                        pieces.push(piece).unwrap();
                    }
                    Down::Finish => panic!("a record never finishes the stream"),
                }
            }
            while let Some(event) = self.above.pop() {
                events.push(event).unwrap();
            }
            if !events.is_empty() {
                assert_eq!(wanted, None, "no demand after Printed");
                let mut writer = Writer::new(total);
                for piece in pieces.as_slice() {
                    writer.put(piece).unwrap();
                }
                return Printed { bytes: writer.finish(), events: events.into_boxed(), largest };
            }
            granted = Some(wanted.expect("printing demands room"));
            self.up(Up::Room);
        }
        panic!("a fixture finishes within 4096 pieces");
    }
}
fn terminal() -> Record {
    Record::Terminal(Terminal { outcome: Outcome::Done, diff: None, left_out: 0 })
}

/// What one entry point left in a printer's queues, before the driver took them.
#[derive(Debug)]
struct Step {
    above: Box<[Event]>,
    below: Box<[Down]>,
}

/// What the driver tells the printer around one record.
#[derive(Clone, Copy, Debug)]
enum Told {
    /// `Up::Room` for each demand.
    Room,
    /// `Up::Failed(Fault::Other)` before the record.
    FaultBefore,
    /// `Up::Failed(Fault::Other)` in place of the `Room` after this many pieces.
    FaultAfter(u32),
}

/// One record, from `Print` until its answer: the first step is `Print`'s, each next one a `Room`'s or a fault's.
fn steps(form: Form, limits: Limits, record: Record, told: Told) -> Box<[Step]> {
    let length = match form::print(form, &record, None, &limits) {
        Ok(bytes) => bytes.len(),
        Err(Refusal::TooLong | Refusal::Text | Refusal::JsonLines) => 0,
    };
    let mut printer = Printer::new(form, &limits);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    if let Told::FaultBefore = told {
        printer::up(&mut printer, &env, Up::Failed(Fault::Other), &mut above, &mut below);
        assert!(above.is_empty() && below.is_empty(), "a fault before the record waits for it");
    }
    printer::down(&mut printer, &env, Request::Print(record), &mut above, &mut below);
    let mut steps = List::uncapped();
    let mut pieces = 0_u32;
    for _ in 0..length.checked_add(2).unwrap() {
        let mut events = List::uncapped();
        let mut requests = List::uncapped();
        let mut demanded = false;
        while let Some(event) = above.pop() {
            events.push(event).unwrap();
        }
        while let Some(request) = below.pop() {
            match &request {
                Down::Demand { .. } => demanded = true,
                Down::Send(_) => pieces = pieces.checked_add(1).unwrap(),
                Down::Finish => {}
            }
            requests.push(request).unwrap();
        }
        let answered = !events.is_empty();
        steps.push(Step { above: events.into_boxed(), below: requests.into_boxed() }).unwrap();
        if answered {
            return steps.into_boxed();
        }
        assert!(demanded, "a record without an answer demands room");
        let event = match told {
            Told::FaultAfter(after) if pieces == after => Up::Failed(Fault::Other),
            Told::Room | Told::FaultBefore | Told::FaultAfter(_) => Up::Room,
        };
        printer::up(&mut printer, &env, event, &mut above, &mut below);
    }
    panic!("a record answers within its length and two steps");
}

const CHUNKS: [u32; 6] = [1, 2, 3, 16, 64, 65_536];

// One write at a time (hearth/docs/design/domain/render.md, 8.5).
#[test]
fn a_printer_has_one_demand_out_whatever_the_chunk() {
    for chunk in CHUNKS {
        for (form, record) in [(Form::Text, Record::Spec(fixtures::x1_prod())), (Form::JsonLines, terminal())] {
            let limits = Limits { chunk, ..limits::SHIPPED };
            let expected = form::print(form, &record, None, &limits).unwrap();
            let steps = steps(form, limits, record, Told::Room);
            let mut bytes = Writer::new(expected.len());
            let mut demanded = None;
            let mut granted = None;
            for (index, step) in steps.iter().enumerate() {
                let mut pieces = 0;
                for request in &step.below {
                    match request {
                        Down::Demand { read, room } => {
                            assert_eq!(*read, Read::Nothing, "write half only");
                            assert!(*room > 0 && *room <= chunk, "room within chunk {chunk}");
                            assert!(demanded.is_none(), "one demand out: {step:?}");
                            assert!(granted.is_none(), "the last room is spent before the next demand: {step:?}");
                            demanded = Some(*room);
                        }
                        Down::Send(piece) => {
                            let room = granted.take().expect("a piece answers a demand");
                            assert!(piece.len() <= usize::try_from(room).unwrap(), "a piece within its room");
                            bytes.put(piece).unwrap();
                            pieces += 1;
                        }
                        Down::Finish => panic!("a record never finishes the stream"),
                    }
                }
                assert_eq!(pieces, usize::from(index > 0), "each Room gives one piece: {step:?}");
                granted = demanded.take();
            }
            assert!(granted.is_none(), "no demand after the last piece");
            assert_eq!(bytes.finish(), expected, "the record whole, chunk {chunk}");
        }
    }
}

fn answered_once(steps: &[Step], expected: Event, no_bytes: bool) {
    let last = steps.len().checked_sub(1).expect("Print's step");
    for (index, step) in steps.iter().enumerate() {
        if index == last {
            assert_eq!(&*step.above, &[expected], "one answer in the last step: {steps:?}");
        } else {
            assert!(step.above.is_empty(), "no answer before the last step: {steps:?}");
        }
        for request in &step.below {
            match request {
                Down::Demand { .. } => assert_ne!(index, last, "no demand beside the answer: {step:?}"),
                Down::Send(_) => assert!(!no_bytes, "an empty or refused record sends no bytes: {step:?}"),
                Down::Finish => panic!("a record never finishes the stream"),
            }
        }
    }
}

// A record is answered once (hearth/docs/design/domain/render.md, 8.5).
#[test]
fn a_record_is_answered_once_whatever_the_chunk() {
    for chunk in CHUNKS {
        let limits = Limits { chunk, ..limits::SHIPPED };
        let record = Record::Spec(fixtures::x1_prod());
        for (told, expected) in [
            (Told::Room, Event::Printed),
            (Told::FaultBefore, Event::Failed(Fault::Other)),
            (Told::FaultAfter(0), Event::Failed(Fault::Other)),
        ] {
            answered_once(&steps(Form::Text, limits, record.clone(), told), expected, false);
        }
        if form::print(Form::Text, &record, None, &limits).unwrap().len() > usize::try_from(chunk).unwrap() {
            answered_once(
                &steps(Form::Text, limits, record.clone(), Told::FaultAfter(1)),
                Event::Failed(Fault::Other),
                false,
            );
        }
        answered_once(
            &steps(Form::Text, Limits { record: 1, chunk }, record, Told::Room),
            Event::Refused(Refusal::TooLong),
            true,
        );
        answered_once(&steps(Form::Text, limits, terminal(), Told::Room), Event::Printed, true);
    }
}
#[test]
fn valid_requires_a_nonzero_chunk() {
    for (limits, expected) in [
        (limits::SHIPPED, true),
        (Limits { chunk: 0, ..limits::SHIPPED }, false),
        (Limits { chunk: 1, record: 0 }, true),
        (Limits { chunk: u32::MAX, ..limits::SHIPPED }, true),
    ] {
        assert_eq!(printer::valid(&limits), expected, "{limits:?}");
    }
}
#[test]
fn a_record_goes_down_in_pieces_of_at_most_a_chunk() {
    let mut machine = Machine::new(Form::Text, Limits { chunk: 16, ..limits::SHIPPED });
    let printed = machine.print(Record::Spec(fixtures::x1_prod()));
    assert_eq!(&*printed.bytes, fixtures::X1_PROD_TEXT.as_bytes(), "spec lines whole and ordered");
    assert!(printed.largest <= 16, "no piece past the chunk");
    assert_eq!(&*printed.events, &[Event::Printed], "one Printed answers the record");
}
const VERSION_LINES: [(&[u8], &[u8]); 1] = [(
    b"{\"record\":\"version\",\"client\":1}\n{\"record\":\"outcome\",\"outcome\":\"done\",\"left_out\":0}\n",
    b"{\"record\":\"outcome\",\"outcome\":\"done\",\"left_out\":0}\n",
)];
#[test]
fn only_the_answers_first_record_follows_the_version_line() {
    for (expected_first, expected_second) in VERSION_LINES {
        let mut machine = Machine::new(Form::JsonLines, limits::SHIPPED);
        let first = machine.print(terminal());
        let second = machine.print(terminal());
        assert_eq!(&*first.bytes, expected_first, "version first");
        assert_eq!(&*second.bytes, expected_second, "version only once");
    }
}
#[test]
fn a_record_that_prints_nothing_is_printed_at_once() {
    let mut machine = Machine::new(Form::Text, limits::SHIPPED);
    machine.down(Request::Print(terminal()));
    assert_eq!(machine.above.pop(), Some(Event::Printed), "immediate terminal");
    assert_eq!(machine.below.pop(), None, "no room needed");
    assert_eq!(machine.printer.waiting(), Waiting::Above, "ready for next record");
}
#[test]
fn finish_ends_the_stream_once_nothing_is_printing() {
    let mut machine = Machine::new(Form::Text, limits::SHIPPED);
    machine.print(terminal());
    machine.down(Request::Finish);
    assert_eq!(machine.below.pop(), Some(Down::Finish), "one stream finish");
    assert_eq!(machine.below.pop(), None, "nothing else");
    assert_eq!(machine.printer.waiting(), Waiting::Close, "finish awaits close");
}
#[test]
fn a_failed_stream_fails_the_record_being_printed() {
    let mut machine = Machine::new(Form::Text, limits::SHIPPED);
    machine.down(Request::Print(Record::Spec(fixtures::x1_prod())));
    machine.below.pop().expect("outstanding demand");
    machine.up(Up::Failed(Fault::Other));
    assert_eq!(machine.above.pop(), Some(Event::Failed(Fault::Other)), "one record failure");
    assert_eq!(machine.printer.waiting(), Waiting::Close, "only close follows failure");
}
#[test]
fn close_withdraws_the_demand_and_answers_closed() {
    let mut machine = Machine::new(Form::Text, limits::SHIPPED);
    machine.down(Request::Print(Record::Spec(fixtures::x1_prod())));
    machine.below.pop().expect("outstanding demand");
    machine.down(Request::Close);
    assert_eq!(machine.below.pop(), Some(Down::Demand { read: Read::Nothing, room: 0 }), "withdraw room");
    assert_eq!(machine.above.pop(), Some(Event::Closed), "one close answer");
    machine.up(Up::Room);
    assert_eq!(machine.printer.waiting(), Waiting::Nothing, "late room discarded");
}
