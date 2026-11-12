//! Keep the host's seam (temper/docs/design/client/tui/journeys.md, 3.1–3.4).
use super::{Line, Sent, Streamed};
use crate::{
    host::{Duties, Host, Report, Settings, Start},
    limits,
};
use jig_client::{Event, Request};
use skein_lib::{Secret, Time, Wall};
use skein_ui_screen::{Rgb, Size};
use skein_ui_screen_terminal::Key;
use temper_client_tui::{Note, Scheme};

/// The start's facts, in the file's order.
pub struct Facts {
    pub seed: u64,
    pub now: Time,
    pub wall: Wall,
    pub size: Size,
    pub scheme: Option<Scheme>,
    pub background: Option<Rgb>,
    pub address: Box<[u8]>,
    pub local: bool,
    pub front: bool,
    pub boxed: bool,
    pub note: Option<Note>,
    pub kept: Vec<jig_client::Record>,
    pub credential: Option<Secret>,
}
impl Facts {
    /// Make the host as `Run::new` does, leaving Start to its first turn.
    #[must_use]
    pub fn start(&self) -> (Host, Report) {
        let host = Host::new(&Settings { limits: limits::tui() }, self.seed, self.now, self.wall);
        let report = Report::Start(Start {
            size: self.size,
            scheme: self.scheme,
            background: self.background,
            boxed: self.boxed,
            address: self.address.clone(),
            local: self.local,
            note: self.note,
            front: self.front,
            kept: self.kept.clone(),
            credential: self.credential.clone(),
        });
        (host, report)
    }
}

/// The view's event before giving the host its key, in compact Debug form.
#[must_use]
pub fn event(host: &Host, key: &Key) -> Option<Box<[u8]>> {
    host.event_of(key).map(|event| format!("{event:?}").into_bytes().into())
}

pub struct Recorder {
    file: Vec<u8>,
    turn: Option<(Time, Wall)>,
    kept: bool,
}
impl Recorder {
    #[must_use]
    pub fn new(facts: &Facts) -> Self {
        let mut this = Self { file: Vec::new(), turn: None, kept: false };
        for line in [
            Line::Journey { version: 2 },
            Line::Seed(facts.seed),
            Line::Clocks { now: facts.now, wall: facts.wall },
            Line::Screen { size: facts.size, scheme: facts.scheme },
        ] {
            this.keep(&line);
        }
        if let Some(background) = facts.background {
            this.keep(&Line::Background(background));
        }
        this.keep(&Line::Address(facts.address.clone()));
        if facts.local {
            this.keep(&Line::Local);
        }
        if facts.front {
            this.keep(&Line::Front);
        }
        if facts.boxed {
            this.keep(&Line::Box);
        }
        for record in &facts.kept {
            this.keep(&Line::Stored { key: record.key.clone(), bytes: record.bytes.clone() });
        }
        if let Some(credential) = &facts.credential {
            this.keep(&Line::Credential(credential.expose().into()));
        }
        if let Some(note) = facts.note {
            this.keep(&Line::Noted(note));
        }
        this
    }

    pub fn turn(&mut self, now: Time, wall: Wall) {
        assert!(!self.kept, "a kept turn ends with its deadline");
        self.turn = Some((now, wall));
    }

    pub fn report(&mut self, host: &mut Host, report: Report) -> Duties {
        let line = match &report {
            Report::Lent { number, exit } => Line::Lent { number: *number, exit: *exit },
            Report::Front(line) => Line::BoxLine(line.print()),
            Report::Stage(stage) => Line::BoxStage(stage.print()),
            Report::Local { link } => Line::Found(link.as_ref().map(|link| link.expose().into())),
            Report::Start(_) => Line::Start,
            Report::Due => Line::Due,
            Report::Key(key) => {
                let event = super::event(host, key);
                if host.event_of(key).is_some_and(|event| !matches!(event, temper_client_tui::Event::Typed { .. })) {
                    self.mark(host);
                }
                Line::Key { key: key.clone(), event }
            }
            Report::Resize(size) => Line::Resize(*size),
            Report::Heard(event) => heard(event),
            Report::Kept { written } => Line::Kept { written: *written },
            Report::Opened { opened } => Line::Opened { opened: *opened },
        };
        self.keep(&line);
        let duties = host.report(report);
        for line in asked(&duties) {
            self.keep(&line);
        }
        duties
    }

    pub fn deadline(&mut self, deadline: Option<Time>) {
        if self.kept {
            Line::Deadline(deadline).write(&mut self.file);
            self.kept = false;
        }
        self.turn = None;
    }

    pub fn mark(&mut self, host: &mut Host) {
        self.keep(&Line::Mark(Some(host.frame().print())));
    }

    #[must_use]
    pub fn save(&self, host: &mut Host) -> Box<[u8]> {
        let mut file = self.file.clone();
        Line::End(Some(host.frame().print())).write(&mut file);
        file.into()
    }

    fn keep(&mut self, line: &Line) {
        if let Some((now, wall)) = self.turn.take() {
            Line::Turn { now, wall }.write(&mut self.file);
            self.kept = true;
        }
        line.write(&mut self.file);
    }
}

fn heard(event: &Event) -> Line {
    match event {
        Event::Reply { exchange, message } => Line::Reply { exchange: exchange.raw(), bytes: message.clone() },
        Event::Lost { exchange } => Line::Unanswered(exchange.raw()),
        Event::StreamGone => Line::Gone,
        Event::Stored => Line::Saved,
        Event::NotStored => Line::Unsaved,
        Event::Streamed { message } => Line::Streamed(match message {
            jig_client::Streamed::Record(bytes) => Streamed::Record(bytes.clone()),
            jig_client::Streamed::Heartbeat => Streamed::Heartbeat,
            jig_client::Streamed::Full(full) => {
                Streamed::Full { transport: full.bound == jig_client::StreamBound::Transport, size: full.size }
            }
            jig_client::Streamed::SignedOut => Streamed::SignedOut,
        }),
        Event::Start { .. }
        | Event::StartUnlinked { .. }
        | Event::SignIn { .. }
        | Event::SignOut
        | Event::Authorize { .. }
        | Event::AskApproval
        | Event::ReadApproval { .. }
        | Event::DecideApproval { .. }
        | Event::ApprovalAnswer { .. }
        | Event::SignOutLost
        | Event::Watch { .. }
        | Event::Unwatch { .. }
        | Event::Read { .. }
        | Event::Send { .. }
        | Event::Carry { .. }
        | Event::Up { .. } => {
            panic!("a report outside the native shell's seam")
        }
    }
}

pub(super) fn asked(duties: &Duties) -> Vec<Line> {
    let mut lines = Vec::new();
    for request in &duties.asked {
        match request {
            Request::Exchange { exchange, credential, message } => lines.push(Line::Send {
                exchange: exchange.raw(),
                message: Sent::of(message),
                credential: credential.as_ref().map(|secret| secret.expose().into()),
            }),
            Request::OpenStream { credential } => {
                lines.push(Line::Stream(credential.as_ref().map(|secret| secret.expose().into())));
            }
            Request::CloseStream => lines.push(Line::Unstream),
            Request::Save { records } => {
                for record in records {
                    lines.push(Line::Save { key: record.key.clone(), bytes: record.bytes.clone() });
                }
            }
            Request::Erase { records } => {
                for key in records {
                    lines.push(Line::Erase(key.clone()));
                }
            }
            Request::Told { .. }
            | Request::Authorized(_)
            | Request::Approval(_)
            | Request::ServiceCredential { .. } => {
                panic!("a duty outside the native shell's seam")
            }
        }
    }
    if let Some(credential) = &duties.keep {
        lines.push(Line::Keep(credential.as_ref().map(|secret| secret.expose().into())));
    }
    if let Some(note) = duties.note {
        lines.push(Line::Note(note));
    }
    if let Some(address) = &duties.address {
        lines.push(Line::Signed(address.clone()));
    }
    if duties.local {
        lines.push(Line::Read);
    }
    if let Some(address) = &duties.open {
        lines.push(Line::Browse(address.clone()));
    }
    if let Some(stop) = duties.stop {
        lines.push(match stop {
            crate::host::Stop::First => Line::StopBox,
            crate::host::Stop::Second => Line::StopBoxAgain,
        });
    }
    if let Some(lend) = &duties.lend {
        lines.push(Line::Lend { number: lend.number, name: lend.name.clone() });
    }
    if duties.unlend {
        lines.push(Line::Unlend);
    }
    if duties.close {
        lines.push(Line::Quit);
    }
    lines
}

pub(super) fn is_ask(line: &Line) -> bool {
    duty_order(line).is_some()
}

pub(super) fn duty_order(line: &Line) -> Option<u8> {
    match line {
        Line::Send { .. } | Line::Stream(_) | Line::Unstream | Line::Save { .. } | Line::Erase(_) => Some(0),
        Line::Keep(_) => Some(1),
        Line::Note(_) => Some(2),
        Line::Signed(_) => Some(3),
        Line::Read => Some(4),
        Line::Browse(_) => Some(5),
        Line::StopBox | Line::StopBoxAgain => Some(6),
        Line::Lend { .. } => Some(7),
        Line::Unlend => Some(8),
        Line::Quit => Some(9),
        Line::Journey { .. }
        | Line::Seed(_)
        | Line::Clocks { .. }
        | Line::Screen { .. }
        | Line::Background(_)
        | Line::Box
        | Line::Address(_)
        | Line::Front
        | Line::BoxLine(_)
        | Line::BoxStage(_)
        | Line::Lent { .. }
        | Line::Local
        | Line::Stored { .. }
        | Line::Credential(_)
        | Line::Noted(_)
        | Line::Turn { .. }
        | Line::Start
        | Line::Due
        | Line::Key { .. }
        | Line::Resize(_)
        | Line::Reply { .. }
        | Line::Unanswered(_)
        | Line::Streamed(_)
        | Line::Gone
        | Line::Saved
        | Line::Unsaved
        | Line::Kept { .. }
        | Line::Opened { .. }
        | Line::Found(_)
        | Line::Deadline(_)
        | Line::Mark(_)
        | Line::End(_) => None,
    }
}
