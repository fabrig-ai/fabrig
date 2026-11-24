//! The drive session's terminal and save observer (temper/docs/design/client/tui/journeys.md, 8.2).
use super::super::{Event, Program};
use crate::tui_driver::{key_bytes, optional_file, replaced, terminal::Terminal};
use skein_lib::{Civil, Duration, Time};
use skein_ui_screen_terminal::Key;
use std::{
    fs::{self, File},
    io,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
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
    fn state(&self) -> PathBuf {
        self.session.join("state/temper/tui")
    }
    pub(super) fn start(&mut self, program: &Program, rows: u16, columns: u16) -> Result<(), String> {
        assert_eq!((rows, columns), (24, 80), "the shared terminal's size");
        self.stop();
        // Restarting walks approval again, never reusing the previous run's
        // credential or recording. No credential is read by the driver.
        match fs::remove_dir_all(self.state()) {
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
        let latest = latest_recording(&self.state()).and_then(|path| match path {
            Some(path) => optional_file(&path),
            None => Ok(None),
        });
        self.save = Some(Saving { earlier: latest.map_err(|why| format!("cannot observe TUI save: {why}"))? });
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
        events.extend(self.poll_save());
        events
    }
    fn poll_save(&mut self) -> Option<Event> {
        let saving = self.save.as_ref()?;
        let result = latest_recording(&self.state()).and_then(|path| match path {
            Some(path) => replaced(&path, saving.earlier.as_ref()),
            None => Ok(None),
        });
        let event = match result {
            Ok(Some(bytes)) => Event::TuiSaved(bytes.into()),
            Ok(None) => return None,
            Err(why) => Event::TuiFailure(format!("cannot read TUI save: {why}")),
        };
        self.save = None;
        Some(event)
    }
}

// Basic UTC run names sort in start order.
fn latest_recording(directory: &Path) -> io::Result<Option<PathBuf>> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(why) if why.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(why) => return Err(why),
    };
    let mut latest = None;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        if recording_name(name.as_bytes())
            && latest.as_ref().is_none_or(|latest| name > *latest)
            && entry.file_type()?.is_file()
        {
            latest = Some(name);
        }
    }
    Ok(latest.map(|name| directory.join(name)))
}

fn recording_name(name: &[u8]) -> bool {
    let Ok(name) = std::str::from_utf8(name) else { return false };
    let Some(time) = name.strip_prefix("recording-").and_then(|suffix| suffix.strip_suffix('Z')) else { return false };
    let Some((date, clock)) = time.split_once('T') else { return false };
    if date.len() != 8 || clock.len() != 6 || !date.bytes().chain(clock.bytes()).all(|byte| byte.is_ascii_digit()) {
        return false;
    }
    let number = |part: &str| part.parse::<u8>().expect("two ASCII digits");
    let date = Civil {
        year: date[..4].parse().expect("four ASCII digits"),
        month: number(&date[4..6]),
        day: number(&date[6..8]),
    };
    date.valid() && number(&clock[..2]) < 24 && number(&clock[2..4]) < 60 && number(&clock[4..]) < 60
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_observe_the_newest_tui_run_and_its_replacements() {
        let scratch = skein_scratch::Scratch::new("tui-save-observer");
        let mut tui = Tui::new(scratch.path().join("session"));
        tui.save().expect("request a save before the TUI creates its directory");
        assert!(tui.poll_save().is_none());

        let directory = tui.state();
        fs::create_dir_all(&directory).expect("TUI state");
        fs::write(directory.join("recording"), b"obsolete name").expect("legacy file");
        assert!(tui.poll_save().is_none(), "a save is only written to a run's file");
        let latest = directory.join("recording-20261116T000000Z");
        let earlier = directory.join("recording-20261115T235959Z");
        fs::write(&latest, b"latest").expect("newest run's first save");
        fs::write(&earlier, b"older").expect("older run, written out of order");
        fs::write(directory.join("recording-20261116T000000Z.skein-draft"), b"unfinished").expect("temporary file");
        fs::write(directory.join("recording-20261131T000000Z"), b"invalid date").expect("unrelated file");
        fs::create_dir(directory.join("recording-20261117T000000Z")).expect("not a file");
        assert!(matches!(tui.poll_save(), Some(Event::TuiSaved(bytes)) if bytes.as_ref() == b"latest"));

        tui.save().expect("request another save");
        let temporary = directory.join("next");
        fs::write(&temporary, b"updated older").expect("older recording bytes");
        fs::rename(&temporary, earlier).expect("replace older run's file");
        assert!(tui.poll_save().is_none(), "only the newest run's replacement counts");
        fs::write(&temporary, b"latest").expect("identical recording bytes");
        fs::rename(temporary, latest).expect("replace newest run's file");
        assert!(matches!(tui.poll_save(), Some(Event::TuiSaved(bytes)) if bytes.as_ref() == b"latest"));
        assert!(tui.poll_save().is_none(), "one event per request");
    }
}
