//! Shared recorded worlds and file checks for journey-tool tests (tools/crates/design.md, 3.2).

pub mod web;

use skein_lib::{Time, Wall};
use skein_ui_web::mapping::Primary;
use std::{fs, path::Path};
use temper_client_shell_web::journey;

#[must_use]
pub fn fixture_recording() -> Box<[u8]> {
    fixture_with_secret(None)
}

#[must_use]
pub fn fixture_with_secret(secret: Option<Box<[u8]>>) -> Box<[u8]> {
    fixture(secret, false)
}

#[must_use]
pub fn fixture_with_marks_and_secret(secret: Option<Box<[u8]>>) -> Box<[u8]> {
    fixture(secret, true)
}

fn fixture(secret: Option<Box<[u8]>>, marked: bool) -> Box<[u8]> {
    let facts = journey::Facts {
        fixed: false,
        seed: 11,
        now: Time::ZERO,
        wall: Wall::EPOCH,
        primary: Primary::Control,
        zone_minutes: 0,
        ways: Some((b"google".as_slice().into(), b"http://127.0.0.1:34073".as_slice().into())),
        forges: Vec::new(),
        path: b"/".as_slice().into(),
        secret,
        stored: Vec::new(),
    };
    let (mut host, mut page) = facts.start();
    let mut recorder = journey::Recorder::new(&facts);
    if marked {
        recorder.mark(&page);
    }
    recorder.turn(facts.now, facts.wall);
    recorder.report(&mut host, &page, temper_client_shell_web::host::Report::Start);
    if facts.secret.is_none() {
        recorder.report(
            &mut host,
            &page,
            temper_client_shell_web::host::Report::Heard(jig_client_shell_web::adapter::Heard::StreamRefused {
                status: 401,
            }),
        );
    }
    host.draw(&mut page, &mut Vec::new());
    recorder.deadline(host.deadline());
    if marked {
        recorder.mark(&page);
    }
    recorder.save(&page)
}

#[must_use]
pub fn stale_recording(recording: &[u8]) -> Box<[u8]> {
    let mut lines = journey::lines(recording).expect("fixture records");
    let mut changed = 0;
    for line in &mut lines {
        if let journey::Line::Deadline(deadline) = line {
            *deadline = Some(Time::ZERO);
            changed += 1;
        }
    }
    assert_eq!(changed, 1, "the start turn's deadline");
    let mut out = Vec::new();
    for line in lines {
        line.write(&mut out);
    }
    out.into()
}

pub fn absent(bytes: &[u8], private: &[u8], place: &str) {
    assert!(skein_lib::bytes::find(bytes, private).is_none(), "original absent from {place}");
}

pub struct TuiWorld {
    host: temper_client_shell_tui::host::Host,
    recorder: temper_client_shell_tui::journey::Recorder,
    now: u64,
}
impl TuiWorld {
    #[must_use]
    pub fn sized(address: &[u8], size: skein_ui_screen::Size) -> Self {
        Self::with_local(address, size, false)
    }
    fn with_local(address: &[u8], size: skein_ui_screen::Size, local: bool) -> Self {
        Self::with_facts(&Self::facts(address, size, local))
    }
    fn facts(address: &[u8], size: skein_ui_screen::Size, local: bool) -> temper_client_shell_tui::journey::Facts {
        temper_client_shell_tui::journey::Facts {
            set_up: false,
            box_log: Box::default(),
            seed: 7,
            now: Time::ZERO,
            wall: Wall::EPOCH,
            size,
            scheme: Some(temper_client_tui::Scheme::Dark),
            background: None,
            plain: false,
            version: b"1.2.3 (abcdef123456-dirty)".as_slice().into(),
            boxed: false,
            address: temper_client_tui::Origin::Http(
                core::str::from_utf8(address).expect("fixture authority").parse().expect("fixture socket"),
            ),
            front: false,
            local,
            note: None,
            kept: Vec::new(),
            credential: None,
        }
    }
    fn with_facts(facts: &temper_client_shell_tui::journey::Facts) -> Self {
        let (host, start) = facts.start();
        let mut world = Self { host, recorder: temper_client_shell_tui::journey::Recorder::new(facts), now: 1 };
        world.report(start);
        world
    }
    #[must_use]
    pub fn with_box_log(log: &[u8]) -> Self {
        use temper_client_shell_tui::host::Report;
        let mut facts = Self::facts(b"127.0.0.1:8367", skein_ui_screen::Size { rows: 16, columns: 100 }, true);
        facts.front = true;
        facts.set_up = true;
        facts.box_log = log.into();
        let mut world = Self::with_facts(&facts);
        world.report(Report::Local {
            link: Some(skein_lib::Secret::new(
                format!("http://127.0.0.1:8367/sign-in?secret={}", "A".repeat(43)).into_bytes().into(),
            )),
        });
        world.report(Report::Stage(temper_client_tui::BoxStage::Running));
        world
    }
    pub fn report(&mut self, report: temper_client_shell_tui::host::Report) -> temper_client_shell_tui::host::Duties {
        let now = Time::from_nanos(self.now);
        let wall = Wall::from_nanos(self.now);
        self.now += 1;
        self.host.clocks(now, wall);
        self.recorder.turn(now, wall);
        let duties = self.recorder.report(&mut self.host, report);
        self.host.frame();
        self.recorder.deadline(self.host.deadline());
        duties
    }
    pub fn mark(&mut self) {
        let now = Time::from_nanos(self.now);
        let wall = Wall::from_nanos(self.now);
        self.now += 1;
        self.host.clocks(now, wall);
        self.recorder.turn(now, wall);
        self.recorder.mark(&mut self.host);
        self.recorder.deadline(self.host.deadline());
    }
    pub fn save(&mut self) -> Box<[u8]> {
        self.recorder.save(&mut self.host)
    }
    pub fn exchange(&mut self, approval: bool) -> skein_lib::Token {
        Self::saved_exchange(&self.save(), approval)
    }
    #[must_use]
    pub fn saved_exchange(file: &[u8], approval: bool) -> skein_lib::Token {
        use temper_client_shell_tui::journey::{Line, Sent, lines};
        let records = lines(file).expect("records");
        records
            .iter()
            .rev()
            .find_map(|line| match line {
                Line::Send { exchange, message: Sent::SignInApproval(_), .. } if approval => {
                    Some(skein_lib::Token::new(*exchange))
                }
                Line::Send { exchange, message: Sent::ApprovalAsk(_), .. } if !approval => {
                    Some(skein_lib::Token::new(*exchange))
                }
                _ => None,
            })
            .expect("sign-in exchange")
    }
}

#[must_use]
pub fn tui_recording() -> Box<[u8]> {
    use skein_ui_screen_terminal::Key;
    use temper_client_shell_tui::host::Report;
    let mut world = TuiWorld::with_local(b"127.0.0.1:34073", skein_ui_screen::Size { rows: 16, columns: 100 }, true);
    world.mark();
    world.report(Report::Local { link: None });
    world.report(Report::Key(Key::Paste(
        format!("http://127.0.0.1:34073/jig/v1/sign-in/link?secret={}", "A".repeat(43)).into_bytes().into(),
    )));
    world.report(Report::Key(Key::Enter));
    world.save()
}

#[must_use]
pub fn tui_approval_recording() -> Box<[u8]> {
    use temper_client_shell_tui::host::Report;
    let mut world = TuiWorld::with_local(b"127.0.0.1:34073", skein_ui_screen::Size { rows: 16, columns: 100 }, true);
    world.mark();
    world.report(Report::Local { link: None });
    let exchange = world.exchange(false);
    world.report(Report::Heard(jig_client::Event::Reply { exchange, message: tui_approval_reply() }));
    world.report(Report::Opened { opened: false });
    world.mark();
    let exchange = world.exchange(true);
    let mut credential = 7_u64.to_be_bytes().to_vec();
    credential.extend_from_slice(&[8; 32]);
    let mut reply = vec![0, 1];
    reply.extend_from_slice(&u32::try_from(credential.len()).expect("credential length").to_be_bytes());
    reply.extend_from_slice(&credential);
    reply.extend_from_slice(&7_u64.to_be_bytes());
    reply.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    world.report(Report::Heard(jig_client::Event::Reply { exchange, message: reply.into() }));
    world.report(Report::Kept { written: true });
    world.save()
}

#[must_use]
pub fn tui_approval_reply() -> Box<[u8]> {
    let mut reply = vec![0];
    reply.extend_from_slice(&32_u32.to_be_bytes());
    reply.extend_from_slice(&[7; 32]);
    reply.extend_from_slice(b"WDJBMJHT");
    reply.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    reply.into()
}

#[must_use]
pub fn tui_written(lines: &[temper_client_shell_tui::journey::Line]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for line in lines {
        line.write(&mut bytes);
    }
    bytes
}

#[must_use]
pub fn tree_bytes(root: &Path) -> std::collections::BTreeMap<std::path::PathBuf, Option<Vec<u8>>> {
    let mut files = std::collections::BTreeMap::new();
    let mut directories = vec![root.to_owned()];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(directory).expect("tree directory") {
            let entry = entry.expect("entry");
            let path = entry.path();
            let relative = path.strip_prefix(root).expect("relative path").to_owned();
            if entry.file_type().expect("file type").is_dir() {
                files.insert(relative, None);
                directories.push(path);
            } else {
                files.insert(relative, Some(fs::read(path).expect("tree file")));
            }
        }
    }
    files
}

pub fn assert_normalised(root: &Path, tui: bool) {
    for entry in fs::read_dir(root).expect("journeys directory") {
        let entry = entry.expect("journey entry");
        let name = entry.file_name();
        if !entry.file_type().expect("journey type").is_dir()
            || !entry.path().join("recording").try_exists().expect("recording path")
        {
            continue;
        }
        let recording = fs::read(root.join(&name).join("recording")).expect("committed recording");
        let normalised = if tui {
            temper_client_shell_tui::journey::normalise(&recording).expect("normalise TUI")
        } else {
            journey::normalise(&recording).expect("normalise browser")
        };
        assert!(
            recording == normalised.recording.as_ref(),
            "{} is not its own normalisation; run cargo run -p temper-client-journeys -- normalise",
            root.join(name).display()
        );
    }
}
