//! Keep the reports and duties, never patches (client/web/journeys.md, 3.1).
use super::{Facts, Line, line::clone_act, line::clone_heard};
use crate::host::{Duties, Host, Report, TabAct};
use jig_client_shell_web::adapter::Asked;
use skein_lib::{Time, Wall};
use skein_ui::Act;
use skein_ui_web::{page::Page, transport::Carried};

/// What crosses the host's seam, kept as recording lines (client/web/journeys.md, 3.1).
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
            Line::Primary(facts.primary),
        ] {
            this.keep(&line);
        }
        if facts.zone_minutes != 0 {
            this.keep(&Line::Zone(facts.zone_minutes));
        }
        if let Some((content, origin)) = &facts.ways {
            this.keep(&Line::Ways { content: content.clone(), origin: origin.clone() });
        }
        for forge in &facts.forges {
            this.keep(&Line::Forge(forge.clone()));
        }
        this.keep(&Line::Path(facts.path.clone()));
        if let Some(secret) = &facts.secret {
            this.keep(&Line::Secret(secret.clone()));
        }
        for record in &facts.stored {
            this.keep(&Line::Stored { key: record.key.clone(), value: record.value.clone() });
        }
        this
    }
    pub fn turn(&mut self, now: Time, wall: Wall) {
        self.turn = Some((now, wall));
        self.kept = false;
    }
    /// Give one report and keep its duties in order.
    pub fn report(&mut self, host: &mut Host, page: &Page<'_>, report: Report) -> Duties {
        match &report {
            Report::Push(_) | Report::Screen(_) => {}
            Report::Start => self.keep(&Line::Start),
            Report::Copied { token, written } => self.keep(&Line::Copied { token: *token, written: *written }),
            Report::Due => self.keep(&Line::Due),
            Report::TabRefused => self.keep(&Line::TabRefused),
            Report::Place { path } => self.keep(&Line::Place(path.clone())),
            Report::Heard(heard) => self.keep(&Line::Heard(clone_heard(heard))),
            Report::Act { node, act } => {
                let Some(event) = super::event(host, *node, act) else {
                    let duties = host.report(report);
                    assert!(asked(&duties).is_empty(), "an act with no event unexpectedly has duties");
                    return duties;
                };
                self.act(page, act, event, super::name(host.tree(), *node));
            }
            Report::Carried { reports, writes } => {
                self.keep(&Line::Carried { writes: *writes });
                for report in reports {
                    self.keep(&match report {
                        Carried::Written { key } => Line::Written(key.clone()),
                        Carried::Refused { key } => Line::Unwritten(key.clone()),
                        Carried::Failed { exchange } => Line::Lost(*exchange),
                    });
                }
            }
        }
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
    /// Mark the page as drawn, without drawing it again.
    pub fn mark(&mut self, page: &Page<'_>) {
        self.keep(&Line::Mark(Some(page.html().into_bytes().into())));
    }
    /// The recording as it is now, closed by an end mark; recording goes on
    /// (client/web/journeys.md, 3.1).
    #[must_use]
    pub fn save(&self, page: &Page<'_>) -> Box<[u8]> {
        let mut file = self.file.clone();
        Line::End(Some(page.html().into_bytes().into())).write(&mut file);
        file.into()
    }
    /// Keep the end mark and consume the recorder.
    #[must_use]
    pub fn end(mut self, page: &Page<'_>) -> Box<[u8]> {
        Line::End(Some(page.html().into_bytes().into())).write(&mut self.file);
        self.file.into()
    }
    fn keep(&mut self, line: &Line) {
        if let Some((now, wall)) = self.turn.take() {
            Line::Turn { now, wall }.write(&mut self.file);
            self.kept = true;
        }
        line.write(&mut self.file);
    }
    pub(super) fn act(&mut self, page: &Page<'_>, act: &Act, event: Box<[u8]>, name: Box<[u8]>) {
        if !matches!(act, Act::Input { .. } | Act::Near { .. }) {
            self.mark(page);
        }
        self.keep(&Line::Act { act: clone_act(act), event, name });
    }
}
pub(super) fn asked(duties: &Duties) -> Vec<Line> {
    let mut lines = Vec::new();
    for tab in &duties.tab {
        lines.push(match tab {
            TabAct::Open { words } => Line::TabOpen(words.clone()),
            TabAct::Point { to } => Line::TabPoint(to.clone()),
            TabAct::Close => Line::TabClose,
        });
    }
    if let Some((token, words)) = &duties.copy {
        lines.push(Line::Copy { token: *token, words: words.clone() });
    }
    for ask in &duties.asked {
        lines.push(match ask {
            Asked::Exchange { id, call, bound, cache: _, keep: _ } => Line::Exchange {
                id: *id,
                bound: *bound,
                method: call.method.as_bytes().into(),
                path: call.path.clone(),
                body: call.body.clone(),
                headers: call.headers.as_slice().iter().map(|h| (h.name.clone(), h.value.clone())).collect(),
            },
            Asked::OpenStream { call, piece } => Line::Open {
                piece: *piece,
                path: call.path.clone(),
                headers: call.headers.as_slice().iter().map(|h| (h.name.clone(), h.value.clone())).collect(),
            },
            Asked::CloseStream => Line::Close,
            Asked::Save { key, value } => Line::Save { key: key.clone(), value: value.clone() },
            Asked::Erase { key } => Line::Erase(key.clone()),
        });
    }
    if duties.writes != 0 {
        lines.push(Line::Writes(duties.writes));
    }
    if let Some(path) = &duties.navigate {
        lines.push(Line::Navigate(path.clone()));
    }
    if let Some(path) = &duties.replace {
        lines.push(Line::Replace(path.clone()));
    }
    if let Some((path, replace)) = &duties.show {
        lines.push(Line::Show { path: path.clone(), replace: *replace });
    }
    lines
}
pub(super) fn is_ask(line: &Line) -> bool {
    matches!(
        line,
        Line::TabOpen(_)
            | Line::TabPoint(_)
            | Line::TabClose
            | Line::Copy { .. }
            | Line::Exchange { .. }
            | Line::Open { .. }
            | Line::Close
            | Line::Save { .. }
            | Line::Erase(_)
            | Line::Writes(_)
            | Line::Navigate(_)
            | Line::Replace(_)
            | Line::Show { .. }
    )
}
