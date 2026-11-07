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
use temper_client_tui::Scheme;

pub const ADDRESS: &[u8] = b"127.0.0.1:8367";

pub struct Tui {
    pub host: Host,
    pub who: ClientId,
    pub keeps: Vec<Option<Secret>>,
    pub opens: Vec<Box<[u8]>>,
    pub approval_asks: usize,
    pub address: Option<Box<[u8]>>,
    pub closed: bool,
}

impl Tui {
    /// Links `who` and starts the shipped host at 80 by 24 with no small-store records.
    pub fn start(world: &mut World<Temper, Joint>, who: ClientId, credential: Option<Secret>) -> Self {
        let limits = limits::tui();
        world.front().link_client(who, limits.native.client.client.domain);
        let now = world.now();
        let wall = world.front().engine.env.wall;
        let seed = world.front().seed;
        let host = Host::new(&Settings { limits }, seed, now, wall);
        let mut tui =
            Self { host, who, keeps: Vec::new(), opens: Vec::new(), approval_asks: 0, address: None, closed: false };
        tui.report(
            world,
            Report::Start(Start {
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
            self.host.frame();
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
        String::from_utf8(self.host.frame().text().into_vec()).expect("UTF-8 frame")
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
        self.carry(world, duties);
    }

    fn carry(&mut self, world: &mut World<Temper, Joint>, duties: Duties) {
        let now = world.now();
        for request in duties.asked {
            if let jig_client::Request::Exchange { message: jig_client::Message::ApprovalAsk(_), .. } = &request {
                self.approval_asks += 1;
            }
            world.front().carry(self.who, now, request);
        }
        if let Some(credential) = duties.keep {
            self.keeps.push(credential);
            self.report(world, Report::Kept { written: true });
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
