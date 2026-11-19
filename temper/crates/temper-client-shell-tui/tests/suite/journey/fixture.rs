//! Journey fixtures for this binary (temper/docs/design/client/tui/journeys.md, 7).
//! Library tests and the suite keep separate copies of their own types (tools/crates/design.md, 3.2).
mod client;
pub(super) use client::{delivery_report, terminal_document, watch};
use jig_client::{Message, Request};
use skein_lib::{Secret, Time, Wall};
use skein_ui_screen::Size;
use skein_ui_screen_terminal::Key;
use temper_client_shell_tui::{
    host::{Duties, Host, Report},
    journey::{Facts, Line, Recorder, Replay, Stale, lines, prove},
};
use temper_client_tui::Scheme;

#[must_use]
pub(super) fn facts() -> Facts {
    Facts {
        set_up: false,
        box_log: Box::default(),
        seed: 7,
        now: Time::ZERO,
        wall: Wall::EPOCH,
        size: Size { rows: 12, columns: 48 },
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
    }
}

pub(super) const SETUPS: [(bool, &[u8]); 5] = [
    (false, b""),
    (true, b""),
    (false, b"/data/box/logs/box.log"),
    (true, b"/data/box/logs/box.log"),
    (true, b"/data/\"\xff/box/logs/box.log"),
];

#[must_use]
pub(super) fn setup(set_up: bool, log: &[u8]) -> Facts {
    let mut given = facts();
    given.front = true;
    given.boxed = true;
    given.set_up = set_up;
    given.box_log = log.into();
    given
}

#[must_use]
pub(super) fn written(records: &[Line]) -> Box<[u8]> {
    let mut bytes = Vec::new();
    for record in records {
        record.write(&mut bytes);
    }
    bytes.into()
}

pub(super) struct Walk {
    host: Host,
    recorder: Recorder,
    now: u64,
    marks: Vec<(Box<[u8]>, Option<Key>)>,
}
impl Walk {
    pub(super) fn with(facts: &Facts) -> Self {
        let (host, start) = facts.start();
        let mut walk = Self { host, recorder: Recorder::new(facts), now: 1, marks: Vec::new() };
        walk.reports(vec![start]);
        walk
    }
    fn begin(&mut self) {
        let now = Time::from_nanos(self.now);
        let wall = Wall::from_nanos(self.now);
        self.host.clocks(now, wall);
        self.recorder.turn(now, wall);
        self.now += 1;
    }
    fn finish(&mut self) {
        self.recorder.deadline(self.host.deadline());
        self.host.frame();
    }
    pub(super) fn reports(&mut self, reports: Vec<Report>) -> Vec<Duties> {
        self.begin();
        let duties = reports.into_iter().map(|report| self.recorder.report(&mut self.host, report)).collect();
        self.finish();
        duties
    }
    pub(super) fn mark(&mut self) {
        self.begin();
        self.marks.push((self.host.frame().print(), None));
        self.recorder.mark(&mut self.host);
        self.finish();
    }
    pub(super) fn save(&mut self) -> Box<[u8]> {
        self.recorder.save(&mut self.host)
    }
}

fn credential() -> Secret {
    let mut bytes = 7_u64.to_be_bytes().to_vec();
    bytes.extend_from_slice(&[7; 32]);
    Secret::new(bytes.into())
}

fn signed_in() -> Box<[u8]> {
    let credential = credential();
    let mut bytes = vec![0, 1];
    bytes.extend_from_slice(&u32::try_from(credential.len()).expect("credential size").to_be_bytes());
    bytes.extend_from_slice(credential.expose());
    bytes.extend_from_slice(&7_u64.to_be_bytes());
    bytes.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    bytes.into()
}

pub(super) fn linked_walk_duties(front: bool) -> (Walk, Duties) {
    let mut facts = facts();
    facts.local = true;
    facts.front = front;
    facts.note = Some(temper_client_tui::Note::Until(Wall::from_nanos(1)));
    let mut walk = Walk::with(&facts);
    let duties = walk.reports(vec![Report::Local { link: Some(Secret::new(printed_link())) }]);
    let exchange = duties
        .iter()
        .flat_map(|duty| &duty.asked)
        .find_map(|request| {
            if let Request::Exchange { exchange, message: Message::SignInLink(_), .. } = request {
                Some(*exchange)
            } else {
                None
            }
        })
        .expect("engine link exchange");
    walk.reports(vec![Report::Heard(jig_client::Event::Reply { exchange, message: signed_in() })]);
    let duties = walk.reports(vec![Report::Heard(jig_client::Event::Streamed {
        message: jig_client::Streamed::Record(client::hello_bytes()),
    })]);
    let mut duties = duties.into_iter().next().expect("hello's duties");
    let next = walk.reports(vec![Report::Heard(jig_client::Event::Streamed {
        message: jig_client::Streamed::Record(client::me_bytes(&duties)),
    })]);
    duties.asked.extend(next.into_iter().flat_map(|duties| duties.asked));
    walk.mark();
    (walk, duties)
}

pub(super) fn replay_stale(recording: &[u8]) -> Stale {
    let mut replay = Replay::new(recording).expect("readable recording");
    loop {
        match replay.next() {
            Err(stale) => return stale,
            Ok(Some(_)) => {}
            Ok(None) => panic!("changed rule was accepted"),
        }
    }
}

pub(super) fn same_frames(mut walk: Walk) {
    let file = walk.save();
    let records = lines(&file).expect("readable records");
    assert!(written(&records) == file, "line codec preserves the recording");
    assert!(prove(&file).is_ok(), "the whole walk proves its live frames");
    let mut replay = Replay::new(&file).expect("facts");
    walk.marks.push((walk.host.frame().print(), None));
    for (print, key) in walk.marks {
        let mark = replay.next().expect("same duties").expect("mark");
        assert!(mark.frame.print() == print, "replay yields the live frame");
        assert!(mark.live == Some(print), "mark retains its live print");
        assert_eq!(mark.key, key);
    }
    assert!(replay.next().expect("end").is_none());
}

impl Walk {
    pub(super) fn admit(&mut self, mut duties: Duties) -> Duties {
        let mut pending = std::collections::VecDeque::from(core::mem::take(&mut duties.asked));
        while let Some(request) = pending.pop_front() {
            for report in client::background_reply(&request) {
                for next in self.reports(vec![report]) {
                    pending.extend(next.asked);
                }
            }
            duties.asked.push(request);
        }
        duties
    }
}

pub(super) struct World {
    pub host: Host,
    pub recorder: Recorder,
    pub(super) now: u64,
}
impl World {
    pub(super) fn with_front(address: &[u8], front: bool) -> Self {
        let mut facts = facts();
        facts.size = Size { rows: 16, columns: 100 };
        facts.address = temper_client_tui::Origin::Http(
            core::str::from_utf8(address).expect("fixture authority").parse().expect("fixture socket"),
        );
        facts.front = front;
        facts.local = front;
        let (host, start) = facts.start();
        let mut world = Self { host, recorder: Recorder::new(&facts), now: 1 };
        world.report(start);
        world
    }
    pub fn report(&mut self, report: Report) -> Duties {
        self.reports(vec![report]).pop().expect("one report")
    }
    pub fn reports(&mut self, reports: Vec<Report>) -> Vec<Duties> {
        let now = Time::from_nanos(self.now);
        let wall = Wall::from_nanos(self.now);
        self.now += 1;
        self.host.clocks(now, wall);
        self.recorder.turn(now, wall);
        let duties = reports.into_iter().map(|report| self.recorder.report(&mut self.host, report)).collect();
        self.host.frame();
        self.recorder.deadline(self.host.deadline());
        duties
    }
    pub fn save(&mut self) -> Box<[u8]> {
        self.recorder.save(&mut self.host)
    }
}

pub(super) fn printed_link() -> Box<[u8]> {
    format!("http://127.0.0.1:8367/jig/v1/sign-in/link?secret={}", "A".repeat(43)).into_bytes().into()
}

pub(super) fn url_base64(bytes: &[u8]) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(
        usize::try_from(
            skein_lib::base64::encoded_len(
                skein_lib::base64::Alphabet::Url,
                u32::try_from(bytes.len()).expect("secret size"),
            )
            .expect("base64 size"),
        )
        .expect("base64 size"),
    );
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Url, bytes, &mut writer).expect("base64 secret");
    writer.finish()
}
