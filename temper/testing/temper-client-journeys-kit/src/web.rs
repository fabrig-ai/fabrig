//! Native browser reports and managed-forge captures for journey tests.
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_client_shell_web::adapter::{Asked, Heard};
use skein_lib::{
    Duration, List, Time, Wall, Writer,
    base64::{self, Alphabet},
};
use skein_ui::{Act, Role};
use skein_ui_web::{mapping::Primary, page::Page};
use temper_client_shell_web::{
    host::{Host, Report},
    journey::{Facts, Recorder},
};

// Keep native reports as text lines, with no browser or box.
pub struct Capture {
    host: Host,
    page: Page<'static>,
    recorder: Recorder,
    now: Time,
}
impl Default for Capture {
    fn default() -> Self {
        Self::new()
    }
}
impl Capture {
    #[must_use]
    pub fn new() -> Self {
        let facts = Facts {
            seed: 11,
            now: Time::ZERO,
            wall: Wall::EPOCH,
            primary: Primary::Control,
            zone_minutes: 0,
            ways: None,
            forges: vec![],
            path: b"/forges/connect?forge=managed".as_slice().into(),
            secret: None,
            stored: vec![],
        };
        let (host, page) = facts.start();
        Self { host, page, recorder: Recorder::new(&facts), now: Time::ZERO }
    }
    pub fn report(&mut self, report: Report) -> Vec<Asked> {
        self.now = self.now.saturating_add(Duration::from_millis(10));
        self.host.clocks(self.now, Wall::EPOCH);
        self.recorder.turn(self.now, Wall::EPOCH);
        let duties = self.recorder.report(&mut self.host, &self.page, report);
        self.host.draw(&mut self.page, &mut Vec::new());
        self.recorder.deadline(self.host.deadline());
        duties.asked
    }
    fn stream(&mut self, event: &w::StreamEvent) -> Vec<Asked> {
        let bytes = event.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding");
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
#[must_use]
pub fn managed_capture(mut capture: Capture) -> Capture {
    capture.report(Report::Start);
    capture.report(Report::Heard(Heard::StreamOpened));
    let mut served = List::with_capacity(1);
    served.push(w::Served { owner: w::Owner::Jig, first: w::VERSION, last: w::VERSION }).expect("one owner");
    let mut me = jig_client_test_records::frame_two_projects_record();
    me.deployment_role = Some(w::HeldRole::Owner);
    capture.stream(&jig_client_test_records::opened(
        w::Hello { stream: [7; 16], served, heartbeat: Duration::from_secs(30) },
        me,
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
