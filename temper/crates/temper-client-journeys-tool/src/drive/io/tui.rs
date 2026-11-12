//! Terminal and inode observation only; the session machine owns every bound.
use super::super::{Event, Program};
use crate::tui_driver::{key_bytes, optional_file, replaced, terminal::Terminal};
use skein_lib::{Duration, Time};
use skein_ui_screen_terminal::Key;
use std::{
    fs::{self, File},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};
struct Saving {
    earlier: Option<File>,
}
pub(super) struct Tui {
    session: PathBuf,
    terminal: Option<Terminal>,
    ready: bool,
    writing: bool,
    save: Option<Saving>,
}
impl Tui {
    pub(super) fn new(session: PathBuf) -> Self {
        Self { session, terminal: None, ready: false, writing: false, save: None }
    }
    pub(super) fn start(&mut self, program: &Program, rows: u16, columns: u16) -> Result<(), String> {
        assert_eq!((rows, columns), (24, 80), "the shared terminal's size");
        self.stop();
        // Restarting walks approval again, never reusing the previous run's
        // credential or recording. No credential is read by the driver.
        let state = self.session.join("state/temper/tui");
        match fs::remove_dir_all(&state) {
            Ok(()) => {}
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
            Err(why) => return Err(format!("cannot reset TUI state: {why}")),
        }
        let address = program.args.first().ok_or("TUI program has no origin")?;
        self.terminal = Some(
            Terminal::start(&program.directory, &program.program, address.as_bytes(), &program.environment)
                .map_err(str::to_owned)?,
        );
        Ok(())
    }
    pub(super) fn write(&mut self, bytes: Vec<u8>) {
        if let Some(terminal) = &mut self.terminal {
            terminal.write(bytes);
            self.writing = true;
        }
    }
    pub(super) fn save(&mut self) -> Result<(), String> {
        self.save = Some(Saving {
            earlier: optional_file(&self.session.join("state/temper/tui/recording"))
                .map_err(|why| format!("cannot observe TUI save: {why}"))?,
        });
        if let Some(terminal) = &mut self.terminal {
            terminal.write(key_bytes(&Key::Alt('S')));
        }
        Ok(())
    }
    pub(super) fn stop(&mut self) {
        drop(self.terminal.take());
        self.ready = false;
        self.writing = false;
        self.save = None;
    }
    pub(super) fn deadline(&self, now: Time, deadline: Option<Time>) -> Option<Time> {
        if self.terminal.is_some() {
            Some(deadline.map_or(now.saturating_add(Duration::from_millis(50)), |at| {
                at.min(now.saturating_add(Duration::from_millis(50)))
            }))
        } else {
            deadline
        }
    }
    pub(super) fn tick(&mut self) -> Vec<Event> {
        let Some(terminal) = &mut self.terminal else {
            return vec![];
        };
        if let Err(why) = terminal.tick() {
            return vec![Event::TuiFailure(why.into())];
        }
        let mut events = vec![];
        if !self.ready && terminal.ready() {
            self.ready = true;
            events.push(Event::TuiReady);
        }
        if self.writing && terminal.written() {
            self.writing = false;
            events.push(Event::TuiWritten);
        }
        if let Some(earlier) = &self.save {
            match replaced(&self.session.join("state/temper/tui/recording"), earlier.earlier.as_ref()) {
                Ok(Some(bytes)) => {
                    self.save = None;
                    events.push(Event::TuiSaved(bytes.into()));
                }
                Ok(None) => {}
                Err(why) => {
                    self.save = None;
                    events.push(Event::TuiFailure(format!("cannot read TUI save: {why}")));
                }
            }
        }
        events
    }
}
