use super::*;
use crate::host::{Duties, Host, Report};
use crate::journey::{Facts, Recorder};
use jig_client::{Message, Request};
use skein_lib::{Time, Token, Wall};
use skein_ui_screen::Size;
use temper_client_tui::Scheme;

struct World {
    host: Host,
    recorder: Recorder,
    now: u64,
}
impl World {
    fn new() -> (Self, Duties) {
        let facts = Facts {
            set_up: false,
            box_log: Box::default(),
            seed: 7,
            now: Time::ZERO,
            wall: Wall::EPOCH,
            size: Size { rows: 24, columns: 80 },
            scheme: Some(Scheme::Dark),
            background: None,
            plain: false,
            version: Box::default(),
            boxed: false,
            address: temper_client_tui::Origin::Http("127.0.0.1:8367".parse().expect("fixture socket")),
            front: false,
            local: false,
            note: None,
            kept: Vec::new(),
            credential: None,
        };
        let (host, start) = facts.start();
        let mut world = Self { host, recorder: Recorder::new(&facts), now: 1 };
        let duties = world.report(start);
        (world, duties)
    }
    fn turn(&mut self) {
        self.host.clocks(Time::from_nanos(self.now), Wall::from_nanos(self.now));
        self.recorder.turn(Time::from_nanos(self.now), Wall::from_nanos(self.now));
        self.now += 1;
    }
    fn report(&mut self, report: Report) -> Duties {
        self.turn();
        let duties = self.recorder.report(&mut self.host, report);
        self.finish();
        duties
    }
    fn finish(&mut self) {
        self.recorder.deadline(self.host.deadline());
        self.host.frame();
    }
    fn mark(&mut self) {
        self.turn();
        self.recorder.mark(&mut self.host);
        self.finish();
    }
    fn save(&mut self) -> Box<[u8]> {
        self.recorder.save(&mut self.host)
    }
}

fn exchange(duties: &Duties, approval_ask: bool) -> Token {
    duties
        .asked
        .iter()
        .find_map(|request| match request {
            Request::Exchange { exchange, message: Message::ApprovalAsk(_), .. } if approval_ask => Some(*exchange),
            Request::Exchange { exchange, message: Message::SignInApproval(_), .. } if !approval_ask => Some(*exchange),
            Request::Exchange { .. }
            | Request::OpenStream { .. }
            | Request::CloseStream
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Told { .. }
            | Request::Authorized(_)
            | Request::AskedLink(_)
            | Request::Approval(_)
            | Request::ServiceCredential { .. } => None,
        })
        .expect("approval exchange")
}

fn approval_saves() -> [Box<[u8]>; 3] {
    let (mut world, duties) = World::new();
    let initial = world.save();
    // Jig's v21 ApprovalAskAnswer::Asked: tag, secret length and bytes, code, expiry.
    let mut asked = vec![0];
    asked.extend_from_slice(&32_u32.to_be_bytes());
    asked.extend_from_slice(&[7; 32]);
    asked.extend_from_slice(b"BCDFGHJK");
    asked.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    let duties = world
        .report(Report::Heard(jig_client::Event::Reply { exchange: exchange(&duties, true), message: asked.into() }));
    let take = exchange(&duties, false);
    world.report(Report::Opened { opened: true });
    world.mark();
    let waiting = world.save();
    // SignedIn: tag, present credential, length, sign-in id and fake secret, party, expiry.
    let mut signed_in = vec![0, 1];
    signed_in.extend_from_slice(&40_u32.to_be_bytes());
    signed_in.extend_from_slice(&7_u64.to_be_bytes());
    signed_in.extend_from_slice(&[7; 32]);
    signed_in.extend_from_slice(&7_u64.to_be_bytes());
    signed_in.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    world.report(Report::Heard(jig_client::Event::Reply { exchange: take, message: signed_in.into() }));
    world.report(Report::Kept { written: true });
    connect(&mut world);
    [initial, waiting, world.save()]
}

fn connect(world: &mut World) {
    let duties = world.report(Report::Heard(jig_client::Event::Streamed {
        message: jig_client::Streamed::Record(crate::host::tests::hello_bytes()),
    }));
    world.report(Report::Heard(jig_client::Event::Streamed {
        message: jig_client::Streamed::Record(crate::host::tests::me_bytes(&duties)),
    }));
}

fn written(records: &[Line]) -> Box<[u8]> {
    let mut bytes = Vec::new();
    for record in records {
        record.write(&mut bytes);
    }
    bytes.into()
}
#[test]
fn saves_keep_only_the_last_browse() {
    let [initial, waiting, approved] = approval_saves();
    for (save, address) in [
        (&initial, None),
        (&waiting, Some(b"http://127.0.0.1:8367/terminal?code=BCDF-GHJK".as_slice())),
        (&approved, Some(b"http://127.0.0.1:8367/terminal?code=BCDF-GHJK".as_slice())),
    ] {
        assert_eq!(browsed(save).expect("save's last browse").as_deref(), address);
    }
    let mut malformed = approved.to_vec();
    malformed.extend_from_slice(b"unknown\n");
    assert!(browsed(&malformed).is_err(), "the whole save must be readable");
    let mut records = lines(&approved).expect("records");
    let last = b"http://localhost/approve?code=JKLM-NPQR".as_slice();
    records.insert(records.len() - 1, Line::Browse(last.into()));
    assert_eq!(browsed(&written(&records)).expect("last browse").as_deref(), Some(last));
}
