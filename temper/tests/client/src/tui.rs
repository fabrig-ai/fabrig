//! The TUI's shipped host in the joint world (temper/docs/design/client/README.md, 6.13;
//! temper/docs/design/domain/testing.md, 6.3).
use crate::{app::Temper, front::Joint};
use jig_client_kit::harness::{ClientId, Until, World};
use skein_lib::{Duration, Secret};
use skein_ui_screen::Size;
use skein_ui_screen_terminal::Key;
use temper_client_shell_tui::{
    host::{Duties, Host, Report, Settings, Start},
    limits,
};
use temper_client_tui::{Note, Scheme};

pub const ADDRESS: &[u8] = b"127.0.0.1:8367";

/// The world's stand-in for the engine's link file (temper/docs/design/client/README.md, 6.14).
pub enum File {
    Absent,
    Bytes(Secret),
    Engine,
}

pub struct Tui {
    pub host: Host,
    pub who: ClientId,
    pub keeps: Vec<Option<Secret>>,
    pub opens: Vec<Box<[u8]>>,
    pub approval_asks: usize,
    pub address: Option<Box<[u8]>>,
    pub closed: bool,
    pub reads: usize,
    pub notes: Vec<Note>,
    pub frames: Vec<String>,
    pub sign_in_sends: usize,
    file: File,
}

impl Tui {
    /// Links `who` and starts the shipped host at 80 by 24 with no small-store records.
    pub fn start(world: &mut World<Temper, Joint>, who: ClientId, credential: Option<Secret>) -> Self {
        Self::with_file(world, who, credential, File::Absent, None)
    }

    /// Starts with the supplied file and the address's retained sign-in note.
    pub fn with_file(
        world: &mut World<Temper, Joint>,
        who: ClientId,
        credential: Option<Secret>,
        file: File,
        note: Option<Note>,
    ) -> Self {
        let limits = limits::tui();
        world.front().link_client(who, limits.native.client.client.domain);
        let now = world.now();
        let wall = world.front().engine.env.wall;
        let seed = world.front().seed;
        let host = Host::new(&Settings { limits }, seed, now, wall);
        let local = !matches!(&file, File::Absent);
        let mut tui = Self {
            host,
            who,
            keeps: Vec::new(),
            opens: Vec::new(),
            approval_asks: 0,
            address: None,
            closed: false,
            reads: 0,
            notes: Vec::new(),
            frames: Vec::new(),
            sign_in_sends: 0,
            file,
        };
        tui.report(
            world,
            Report::Start(Start {
                local,
                note,
                size: Size { columns: 80, rows: 24 },
                scheme: Scheme::Dark,
                address: ADDRESS.into(),
                kept: Vec::new(),
                credential,
            }),
        );
        tui
    }

    /// Each key acts on the frame laid before it.
    pub fn press(&mut self, world: &mut World<Temper, Joint>, keys: Vec<Key>) {
        for key in keys {
            self.clocks(world);
            self.text();
            self.report(world, Report::Key(key));
        }
    }

    /// Runs the world's own steps, carrying each report's duties at once.
    pub fn run(&mut self, world: &mut World<Temper, Joint>, span: Duration) {
        let until = world.now().saturating_add(span);
        while world.now() < until {
            world.run(Until::Steps(1));
            let now = world.now();
            let heard = world.front().heard(self.who, now);
            self.clocks(world);
            for event in heard {
                self.report(world, Report::Heard(event));
            }
            if self.host.deadline().is_some_and(|at| at <= now) {
                self.report(world, Report::Due);
            }
        }
    }

    pub fn text(&mut self) -> String {
        let text = String::from_utf8(self.host.frame().text().into_vec()).expect("UTF-8 frame");
        self.frames.push(text.clone());
        text
    }

    pub fn status(&mut self) -> String {
        // The bottom region pads the status row with an empty row below it.
        self.text().trim_end().lines().next_back().unwrap_or("").trim().to_owned()
    }

    #[must_use]
    pub fn credential(&self) -> Option<&Secret> {
        self.keeps.last().and_then(Option::as_ref)
    }

    fn clocks(&mut self, world: &mut World<Temper, Joint>) {
        self.host.clocks(world.now(), world.front().engine.env.wall);
    }

    fn report(&mut self, world: &mut World<Temper, Joint>, report: Report) {
        let duties = self.host.report(report);
        self.text();
        self.carry(world, duties);
    }

    fn carry(&mut self, world: &mut World<Temper, Joint>, duties: Duties) {
        let now = world.now();
        for request in duties.asked {
            if let jig_client::Request::Exchange { message: jig_client::Message::ApprovalAsk(_), .. } = &request {
                self.approval_asks += 1;
            }
            if let jig_client::Request::Exchange { message: jig_client::Message::SignInLink(_), .. } = &request {
                self.sign_in_sends += 1;
            }
            world.front().carry(self.who, now, request);
        }
        if let Some(credential) = duties.keep {
            self.keeps.push(credential);
            self.report(world, Report::Kept { written: true });
        }
        if let Some(note) = duties.note {
            self.notes.push(note);
        }
        if duties.local {
            self.reads += 1;
            let link = match &self.file {
                File::Absent => None,
                File::Bytes(link) => Some(link.clone()),
                File::Engine => world.front().latest_link().map(Secret::new),
            };
            self.report(world, Report::Local { link });
        }
        if let Some(address) = duties.address {
            self.address = Some(address);
        }
        if let Some(address) = duties.open {
            self.opens.push(address);
            self.report(world, Report::Opened { opened: true });
        }
        self.closed |= duties.close;
    }
}
