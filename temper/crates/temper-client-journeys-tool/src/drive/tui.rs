//! Lazy TUI startup, approval, saves and teardown (client/drive.md, 3.3).
use super::{Action, Event, Machine, Phase, LOOK, POLL};
use crate::{line::{Line, Record, Tui}, tui_driver::key_bytes};
use skein_lib::{Duration, Time};
use skein_fake_person::Find;
use skein_ui::Role;
use temper_client_shell_tui::journey;
const BOUND: Duration = Duration::from_secs(30);
impl Machine {
    pub(super) fn tui(&mut self, now: Time, out: &mut Vec<Action>) {
        if matches!(self.current.as_ref().map(Line::record), Some(Record::Tui(Tui::Stop))) {
            if self.tui_running {
                self.wait(Phase::TuiStopping, now, LOOK);
                out.push(Action::StopTui);
            } else { self.answer(None, out); }
        } else if self.tui_running {
            self.tui_line(now, out);
        } else {
            match self.plan.tui(&self.origin) {
                Ok(program) => {
                    self.tui_running = true;
                    self.frame = Box::new([]);
                    self.wait(Phase::TuiStarting, now, BOUND);
                    out.push(Action::StartTui { program, rows: 24, columns: 80 });
                }
                Err(why) => self.answer(Some(why), out),
            }
        }
    }
    fn tui_line(&mut self, now: Time, out: &mut Vec<Action>) {
        match self.current.as_ref().map(Line::record) {
            Some(Record::Tui(Tui::Key(key))) => {
                let bytes = key_bytes(key);
                self.wait(Phase::TuiKey, now, LOOK);
                out.push(Action::WriteTui(bytes));
            }
            Some(Record::Tui(Tui::See(_))) => {
                self.wait(Phase::TuiSeeing, now, BOUND);
                self.tui_save(now, out);
            }
            Some(Record::Tui(Tui::Look)) => {
                self.wait(Phase::TuiLook, now, LOOK);
                self.tui_save(now, out);
            }
            _ => unreachable!("a running TUI line"),
        }
    }
    fn tui_save(&mut self, now: Time, out: &mut Vec<Action>) {
        self.save_until = Some(now.saturating_add(LOOK));
        out.push(Action::SaveTui);
    }
    pub(super) fn tui_event(&mut self, event: Event, now: Time, out: &mut Vec<Action>) {
        match event {
            Event::TuiReady if self.phase == Phase::TuiStarting => {
                self.phase = Phase::TuiBrowse;
                self.tui_save(now, out);
            }
            Event::TuiSaved(bytes) if matches!(self.phase, Phase::TuiBrowse | Phase::TuiSeeing | Phase::TuiLook) => {
                self.save_until = None;
                let result = self.tui_saved(&bytes, now, out);
                if let Err(why) = result { self.tui_fail(why, now, out); }
            }
            Event::BrowserAddress(address) if self.phase == Phase::TuiAddress => {
                self.return_address = Some(address);
                self.phase = Phase::TuiOpening;
                out.push(Action::Open(std::mem::take(&mut self.frame)));
            }
            Event::TuiWritten if self.phase == Phase::TuiKey => self.answer(None, out),
            Event::TuiStopped => {
                self.tui_running = false;
                if self.phase == Phase::TuiStopping { self.answer(None, out); }
                else if self.phase == Phase::ClosingTui {
                    self.wait(Phase::ClosingBrowser, now, super::STEP);
                    out.push(Action::CloseBrowser);
                }
            }
            Event::TuiFailure(why) => {
                self.tui_running = false;
                out.push(Action::StopTui);
                if self.tui_active() { self.tui_fail(why, now, out); }
                else if self.phase == Phase::ClosingTui {
                    self.wait(Phase::ClosingBrowser, now, super::STEP);
                    out.push(Action::CloseBrowser);
                }
            }
            _ => {}
        }
    }
    fn tui_saved(&mut self, bytes: &[u8], now: Time, out: &mut Vec<Action>) -> Result<(), String> {
        if self.phase == Phase::TuiBrowse {
            if let Some(address) = journey::browsed(bytes).map_err(|why| format!("unreadable TUI save: {why:?}"))? {
                self.frame = address;
                self.phase = Phase::TuiAddress;
                out.push(Action::BrowserAddress);
            } else { self.poll = Some(now.saturating_add(POLL)); }
            return Ok(());
        }
        let mut replay = journey::Replay::new(bytes).map_err(|why| format!("unreadable TUI save: {why:?}"))?;
        while replay.next().map_err(|why| format!("stale TUI save: {why:?}"))?.is_some() {}
        self.frame = replay.host().frame().text();
        if self.phase == Phase::TuiLook {
            self.shown.extend(String::from_utf8_lossy(&self.frame).lines().map(|row| row.trim_end_matches(' ').to_owned()));
            self.answer(None, out);
        } else if let Some(Record::Tui(Tui::See(text))) = self.current.as_ref().map(Line::record) {
            if skein_lib::bytes::find(&self.frame, text).is_some() { self.answer(None, out); }
            else { self.poll = Some(now.saturating_add(POLL)); }
        }
        Ok(())
    }
    fn tui_active(&self) -> bool {
        matches!(self.phase, Phase::TuiStarting | Phase::TuiBrowse | Phase::TuiAddress | Phase::TuiOpening
            | Phase::TuiApprove | Phase::TuiPressed | Phase::TuiReturning | Phase::TuiRecover
            | Phase::TuiSeeing | Phase::TuiLook | Phase::TuiKey | Phase::TuiStopping)
    }
    fn tui_fail(&mut self, why: String, now: Time, out: &mut Vec<Action>) {
        if matches!(self.phase, Phase::TuiStarting | Phase::TuiBrowse | Phase::TuiAddress | Phase::TuiOpening
            | Phase::TuiApprove | Phase::TuiPressed | Phase::TuiReturning) {
            self.tui_running = false;
            out.push(Action::StopTui);
        }
        self.save_until = None;
        out.push(Action::CancelFace);
        if let Some(address) = self.return_address.take() {
            self.diagnostic = why;
            self.wait(Phase::TuiRecover, now, LOOK);
            out.push(Action::Open(address));
        } else { self.answer(Some(why), out); }
    }
    pub(super) fn tui_face_done(&mut self, result: &Result<(), String>, now: Time, out: &mut Vec<Action>) -> bool {
        if !self.tui_active() { return false; }
        if self.phase == Phase::TuiRecover {
            self.answer(Some(self.diagnostic.clone()), out);
            return true;
        }
        if let Err(why) = result { self.tui_fail(why.clone(), now, out); return true; }
        match self.phase {
            Phase::TuiOpening => {
                self.phase = Phase::TuiApprove;
                out.push(Action::Find(Find::exactly_named(Role::Button, b"Approve")));
            }
            Phase::TuiPressed => {
                self.phase = Phase::TuiReturning;
                out.push(Action::Open(self.return_address.take().expect("browser address before approval")));
            }
            Phase::TuiReturning => self.tui_line(now, out),
            _ => {}
        }
        true
    }
    pub(super) fn tui_deadline(&mut self, now: Time, out: &mut Vec<Action>) -> bool {
        if self.phase == Phase::ClosingTui {
            if self.until.is_some_and(|at| now >= at) {
                out.push(Action::StopTui);
                self.tui_running = false;
                self.wait(Phase::ClosingBrowser, now, super::STEP);
                out.push(Action::CloseBrowser);
            }
            return true;
        }
        if !self.tui_active() { return false; }
        if self.save_until.is_some_and(|at| now >= at) {
            self.tui_fail("no recording within 10 s; the session needs the client's recording build".into(), now, out);
        } else if self.until.is_some_and(|at| now >= at) {
            let rows = String::from_utf8_lossy(self.frame.get(..4096).unwrap_or(&self.frame));
            let why = format!("{}: deadline passed; frame's rows: {rows}", self.sought());
            if self.phase == Phase::TuiRecover { self.answer(Some(self.diagnostic.clone()), out); }
            else { self.tui_fail(why, now, out); }
        } else if self.poll.is_some_and(|at| now >= at) {
            self.poll = None;
            if self.phase == Phase::TuiApprove { out.push(Action::Find(Find::exactly_named(Role::Button, b"Approve"))); }
            else { self.tui_save(now, out); }
        }
        true
    }
}
