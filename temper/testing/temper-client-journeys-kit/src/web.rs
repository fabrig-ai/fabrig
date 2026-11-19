//! Native browser reports and managed-forge captures for journey tests.
use jig_client_protocol_wire as w;
use jig_client_shell_web::adapter::{Asked, Heard};
use skein_lib::{
    Duration, FixedClock, FixedStart, List, Overflow, Time, Wall, Writer,
    base64::{self, Alphabet},
};
use skein_ui::{Act, Role};
use skein_ui_web::{mapping::Primary, page::Page};
use temper_client_shell_web::{
    host::{Host, Report},
    journey::{Facts, Recorder},
};

// The scripted walk's start (temper/docs/design/client/journey-clock.md, 2).
const FIXED_START: &str = "1767690000,7";

// Keep native reports as text lines, with no browser or box.
pub struct Capture {
    host: Host,
    page: Page<'static>,
    recorder: Recorder,
    now: Time,
    clock: Option<FixedClock>,
}
impl Capture {
    #[must_use]
    pub fn unfixed() -> Self {
        Self::starting(None)
    }
    #[must_use]
    pub fn fixed() -> Self {
        Self::starting(Some(FixedStart::read(FIXED_START.as_bytes()).expect("walk start")))
    }
    fn starting(start: Option<FixedStart>) -> Self {
        let facts = Facts {
            fixed: start.is_some(),
            seed: start.map_or(11, |start| start.seed),
            now: start.map_or(Time::ZERO, |_| FixedStart::NOW),
            wall: start.map_or(Wall::EPOCH, |start| start.wall),
            primary: Primary::Control,
            zone_minutes: 0,
            ways: None,
            forges: vec![],
            path: b"/forges/connect?forge=managed".as_slice().into(),
            secret: None,
            stored: vec![],
        };
        let (host, page) = facts.start();
        Self {
            host,
            page,
            recorder: Recorder::new(&facts),
            now: Time::ZERO,
            clock: start.map(|start| FixedClock::new(&start, Time::ZERO)),
        }
    }
    pub fn report(&mut self, report: Report) -> Vec<Asked> {
        self.now = self.now.saturating_add(Duration::from_millis(10));
        let (now, wall) = self.clock.as_mut().map_or((self.now, Wall::EPOCH), |clock| clock.read(self.now));
        self.host.clocks(now, wall);
        self.recorder.turn(now, wall);
        let duties = self.recorder.report(&mut self.host, &self.page, report);
        self.host.draw(&mut self.page, &mut Vec::new());
        self.recorder.deadline(self.host.deadline());
        if let Some(clock) = &mut self.clock {
            let _next = clock.until(self.host.deadline(), self.now);
        }
        duties.asked
    }
    fn stream(&mut self, event: &w::StreamEvent) -> Vec<Asked> {
        let bytes = encoded(event.measure(), |writer| event.encode(writer));
        let size = base64::encoded_len(Alphabet::Standard, u32::try_from(bytes.len()).expect("fixture size"))
            .expect("base64 size");
        let mut writer = Writer::new(usize::try_from(size).expect("fixture size"));
        base64::encode(Alphabet::Standard, &bytes, &mut writer).expect("sized base64");
        self.report(Report::Heard(Heard::Piece {
            bytes: [b"data: ".as_slice(), &writer.finish(), b"\n\n"].concat().into(),
        }))
    }
    pub fn act(&mut self, role: Role, name: &[u8], act: Act) {
        let node = self
            .host
            .tree()
            .nodes()
            .iter()
            .find(|node| node.role == role && node.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("missing {role:?} {}: {}", String::from_utf8_lossy(name), self.page.html()))
            .id;
        self.report(Report::Act { node, act });
    }
    #[must_use]
    pub fn save(&self) -> Box<[u8]> {
        self.recorder.save(&self.page)
    }
}
fn encoded(size: u32, write: impl FnOnce(&mut Writer) -> Result<(), Overflow>) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(size).expect("fixture size"));
    write(&mut writer).expect("sized fixture");
    writer.finish()
}
#[must_use]
pub fn managed_capture(mut capture: Capture) -> Capture {
    capture.report(Report::Start);
    capture.report(Report::Heard(Heard::StreamOpened));
    let mut served = List::with_capacity(1);
    served
        .push(
            w::Served::new(&w::CEILINGS, w::ServedParts { owner: w::Owner::Jig, first: w::VERSION, last: w::VERSION })
                .expect("served jig"),
        )
        .expect("one owner");
    let mut me = jig_client_test_records::frame_two_projects_record().into_parts();
    me.deployment_role = Some(w::HeldRole::Owner);
    capture.stream(&jig_client_test_records::opened(
        w::Hello::new(&w::CEILINGS, w::HelloParts { stream: [7; 16], served, heartbeat: Duration::from_secs(30) })
            .expect("hello"),
        w::Me::new(&w::CEILINGS, me).expect("deployment owner"),
    ));
    capture.report(Report::Place { path: b"/forges/connect?forge=managed".as_slice().into() });
    capture.act(Role::TextField, b"Its address", Act::Input { value: b"https://forge.example".as_slice().into() });
    capture.act(
        Role::TextField,
        b"Its web address",
        Act::Input { value: b"http://127.0.0.1:41234/web".as_slice().into() },
    );
    capture
}
