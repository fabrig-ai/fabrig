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
    ended: bool,
    bound: u64,
}
impl Recorder {
    #[must_use]
    pub fn new(facts: &Facts) -> Self {
        let mut this = Self {
            file: Vec::new(),
            turn: None,
            kept: false,
            ended: false,
            bound: u64::from(crate::limits::RECORDING_BYTES),
        };
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
    /// Set the recording's byte bound; the start's facts are always kept whole.
    #[must_use]
    pub fn with_bound(mut self, bytes: u64) -> Self {
        self.bound = bytes;
        self
    }
    pub fn turn(&mut self, now: Time, wall: Wall) {
        self.turn = Some((now, wall));
        self.kept = false;
    }
    /// Give one report and keep its duties in order.
    pub fn report(&mut self, host: &mut Host, page: &Page<'_>, report: Report) -> Duties {
        if self.ended {
            return host.report(report);
        }
        let mut lines = Vec::new();
        match &report {
            Report::Push(_) | Report::Screen(_) => {}
            Report::Start => lines.push(Line::Start),
            Report::Copied { token, written } => lines.push(Line::Copied { token: *token, written: *written }),
            Report::Due => lines.push(Line::Due),
            Report::TabRefused => lines.push(Line::TabRefused),
            Report::Place { path } => lines.push(Line::Place(path.clone())),
            Report::Heard(heard) => lines.push(Line::Heard(clone_heard(heard))),
            Report::Act { node, act } => {
                let Some(event) = super::event(host, *node, act) else {
                    let duties = host.report(report);
                    assert!(asked(&duties).is_empty(), "an act with no event unexpectedly has duties");
                    return duties;
                };
                lines.extend(acted(page, act, event, super::name(host.tree(), *node)));
            }
            Report::Carried { reports, writes } => {
                lines.push(Line::Carried { writes: *writes });
                for report in reports {
                    lines.push(match report {
                        Carried::Written { key } => Line::Written(key.clone()),
                        Carried::Refused { key } => Line::Unwritten(key.clone()),
                        Carried::Failed { exchange } => Line::Lost(*exchange),
                    });
                }
            }
        }
        let duties = host.report(report);
        lines.extend(asked(&duties));
        self.keep_call(page, &lines);
        duties
    }
    pub fn deadline(&mut self, deadline: Option<Time>) {
        if !self.ended && self.kept {
            Line::Deadline(deadline).write(&mut self.file);
            self.kept = false;
        }
        self.turn = None;
    }
    /// Mark the page as drawn, without drawing it again.
    pub fn mark(&mut self, page: &Page<'_>) {
        if !self.ended {
            self.keep_call(page, &[Line::Mark(Some(page.html().into_bytes().into()))]);
        }
    }
    /// The recording as it is now, closed by an end mark; recording goes on
    /// (client/web/journeys.md, 3.1).
    #[must_use]
    pub fn save(&self, page: &Page<'_>) -> Box<[u8]> {
        let mut file = self.file.clone();
        if !self.ended {
            Line::End(Some(page.html().into_bytes().into())).write(&mut file);
        }
        file.into()
    }
    /// Keep the end mark and consume the recorder.
    #[must_use]
    pub fn end(mut self, page: &Page<'_>) -> Box<[u8]> {
        if !self.ended {
            Line::End(Some(page.html().into_bytes().into())).write(&mut self.file);
        }
        self.file.into()
    }
    fn keep(&mut self, line: &Line) {
        line.write(&mut self.file);
    }
    fn keep_call(&mut self, page: &Page<'_>, lines: &[Line]) {
        if self.ended || lines.is_empty() {
            return;
        }
        let mut pending = Vec::new();
        if let Some((now, wall)) = self.turn {
            Line::Turn { now, wall }.write(&mut pending);
        }
        for line in lines {
            line.write(&mut pending);
        }
        let bytes = self.file.len().checked_add(pending.len()).expect("recording size fits usize");
        if u64::try_from(bytes).expect("recording size fits u64") > self.bound {
            Line::End(Some(page.html().into_bytes().into())).write(&mut self.file);
            self.ended = true;
        } else {
            if self.turn.take().is_some() {
                self.kept = true;
            }
            self.file.extend_from_slice(&pending);
        }
    }
    #[cfg(test)]
    pub(super) fn act(&mut self, page: &Page<'_>, act: &Act, event: Box<[u8]>, name: Box<[u8]>) {
        self.keep_call(page, &acted(page, act, event, name));
    }
}
fn acted(page: &Page<'_>, act: &Act, event: Box<[u8]>, name: Box<[u8]>) -> Vec<Line> {
    let mut lines = Vec::new();
    if !matches!(act, Act::Input { .. } | Act::Near { .. }) {
        lines.push(Line::Mark(Some(page.html().into_bytes().into())));
    }
    lines.push(Line::Act { act: clone_act(act), event, name });
    lines
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
