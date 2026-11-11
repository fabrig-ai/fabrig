//! The browser half of recording again, over the native walk (client/web/journeys.md, 8.1–8.2).
//! Like skein's harness (skein/docs/design/ui/web/testing.md, 2), this is ordinary Rust: the clock is
//! supplied by the harness, and only completed downloads are read here.
use crate::Refusal;
use skein_browser::boundary::Key;
use skein_fake_person::{Doing, Face, Find, Found, Next};
use skein_fake_person_browser::{
    BrowserFace,
    harness::{Harness, Names, Outcome, Output, Play, Printed, Sent, Setup},
};
use skein_lib::{Duration, Time};
use skein_ui::{Act, Role};
use std::{
    fs,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use temper_client_shell_web::journey::{self, Control, SetupStep, Step};

const HORIZON: Duration = Duration::from_secs(600);
const POLL: Duration = Duration::from_millis(50);

/// What `again` starts from, found by `main` (client/web/journeys.md, 8.1).
pub struct Paths {
    pub temper: PathBuf,
    pub bundle: PathBuf,
    pub chromium: PathBuf,
    pub scratch: PathBuf,
}

/// Record the journey NAME again: walk it, check it, add it.
/// # Errors
/// Refuses an unwalkable journey, a page that stays moving, a missing event,
/// a different walk, an unproved save, or a browser, box or filesystem failure.
pub fn again(
    journeys: &Path,
    name: &str,
    paths: &Paths,
    environment: &[crate::EnvironmentEntry],
) -> Result<(), Refusal> {
    let steps = starting(journeys, name)?;
    let prepared = prepared(journeys, name)?;
    let downloads = paths.scratch.join("downloads");
    fs::create_dir(&downloads).map_err(|_| crate::io_refusal(&downloads))?;
    // The harness gives its server an empty environment. Forward PATH alone
    // through env so a fresh fake box can find the worker's git at init.
    let path = std::ffi::OsStr::from_bytes(crate::table::PATH.value(environment).unwrap_or_default());
    let launcher = std::env::split_paths(&path)
        .filter(|directory| directory.is_absolute())
        .map(|directory| directory.join("env"))
        .find(|program| program.is_file())
        .ok_or_else(|| Refusal::Harness { name: name.to_owned(), step: 1, report: "cannot find env on PATH".into() })?;
    let mut path_argument = std::ffi::OsString::from("PATH=");
    path_argument.push(path);
    let mut args = vec![
        path_argument,
        paths.temper.clone().into_os_string(),
        "box".into(),
        "--fake-backends".into(),
        "--bundle".into(),
        paths.bundle.clone().into_os_string(),
    ];
    for (flag, directory) in [("--config", "config"), ("--data", "data"), ("--cache", "cache")] {
        args.push(flag.into());
        args.push(paths.scratch.join(directory).into_os_string());
    }
    let mut harness = Harness::new(Setup {
        server: launcher,
        restart: args.clone(),
        args,
        printed: Printed { sent: Sent::Page, prefix: b"http://127.0.0.1:", on: Output::Stdout, names: Names::Url },
        chromium: paths.chromium.clone(),
        downloads: Some(downloads),
        expected: vec![],
        origins: vec![],
    })
    .horizon(HORIZON);
    let mut walker = Walker::new(name, steps, prepared);
    let outcome = harness.run(&mut [&mut walker]);
    if let Some(refusal) = walker.refusal {
        return Err(refusal);
    }
    if let Outcome::Failed { step, report, .. } = outcome {
        return Err(Refusal::Harness { name: name.to_owned(), step: step.saturating_add(1), report });
    }
    let recording = walker.recording.expect("completed walker keeps its end save");
    let new =
        journey::walk(&recording).map_err(|unwalkable| Refusal::Unwalkable { name: name.to_owned(), unwalkable })?;
    if let Some(step) = journey::differs(&walker.steps, &new) {
        return Err(Refusal::WalkDiffers { name: name.to_owned(), step });
    }
    crate::add(journeys, name, &recording, &[]).map(|_| ())
}

/// Check the old walk before building or starting anything.
/// # Errors
/// Refuses an invalid name, a missing recording or an unwalkable recording.
pub fn starting(journeys: &Path, name: &str) -> Result<Vec<Step>, Refusal> {
    if !crate::named(name) {
        return Err(Refusal::Name(name.to_owned()));
    }
    let path = journeys.join(name).join("recording");
    let recording = fs::read(&path).map_err(|_| crate::io_refusal(&path))?;
    journey::walk(&recording).map_err(|unwalkable| Refusal::Unwalkable { name: name.to_owned(), unwalkable })
}

/// The journey's setup, its `after` joined, or None.
/// # Errors
/// Refuses malformed files and missing, chained or self-referencing setups.
pub fn setup(journeys: &Path, name: &str) -> Result<Option<journey::Setup>, Refusal> {
    Ok(prepared(journeys, name)?.map(|prepared| prepared.setup))
}
#[derive(Debug)]
pub(crate) struct Prepared {
    pub(crate) setup: journey::Setup,
    pub(crate) sources: Vec<(String, u32)>,
}
pub(crate) fn read_setup(journeys: &Path, name: &str) -> Result<Option<Prepared>, Refusal> {
    if !crate::named(name) {
        return Err(Refusal::Name(name.to_owned()));
    }
    let path = journeys.join(name).join("setup");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(crate::io_refusal(&path)),
    };
    let setup =
        journey::setup(&bytes).map_err(|unreadable| Refusal::SetupUnreadable { name: name.to_owned(), unreadable })?;
    let first = if setup.after.is_some() { 3 } else { 2 };
    let sources = (0..setup.steps.len())
        .map(|at| (name.to_owned(), first + u32::try_from(at).expect("setup fits u32")))
        .collect();
    Ok(Some(Prepared { setup, sources }))
}
fn prepared(journeys: &Path, name: &str) -> Result<Option<Prepared>, Refusal> {
    let Some(child) = read_setup(journeys, name)? else {
        return Ok(None);
    };
    join_setup(child, journeys, name).map(Some)
}
pub(crate) fn join_setup(mut child: Prepared, after_journeys: &Path, name: &str) -> Result<Prepared, Refusal> {
    if let Some(after) = &child.setup.after {
        let after =
            std::str::from_utf8(after).ok().filter(|after| crate::named(after)).ok_or_else(|| Refusal::SetupAfter {
                name: name.to_owned(),
                after: String::from_utf8_lossy(after).into_owned(),
                why: "invalid setup name",
            })?;
        let refuse = |why| Refusal::SetupAfter { name: name.to_owned(), after: after.to_owned(), why };
        if after == name {
            return Err(refuse("names the journey itself"));
        }
        let Some(mut parent) = read_setup(after_journeys, after)? else {
            return Err(refuse("has no setup"));
        };
        if parent.setup.after.is_some() {
            return Err(refuse("a chain of after is refused"));
        }
        parent.setup.steps.append(&mut child.setup.steps);
        parent.sources.append(&mut child.sources);
        child.setup.steps = parent.setup.steps;
        child.sources = parent.sources;
    }
    Ok(child)
}

enum Phase {
    SetupStep { at: usize },
    SetupSee { at: usize },
    Load,
    MarkWait { mark: u32, text: Box<[u8]> },
    Opening,
    Start,
    Download { after: usize, earlier: Option<Vec<u8>> },
    Gap { earlier: Vec<u8>, until: Time },
    Still { save: Vec<u8> },
    AfterMark { until: Time },
    Find { control: Control },
    Finished,
}
struct Walker {
    name: String,
    setup: Option<Prepared>,
    mark: u32,
    waited_mark: Option<u32>,
    steps: Vec<Step>,
    at: usize,
    phase: Phase,
    since: Option<Time>,
    missing_since: Option<Time>,
    refusal: Option<Refusal>,
    recording: Option<Vec<u8>>,
}
impl Walker {
    fn new(name: &str, steps: Vec<Step>, setup: Option<Prepared>) -> Self {
        Self {
            name: name.to_owned(),
            steps,
            at: 0,
            phase: if setup.is_some() { Phase::SetupStep { at: 0 } } else { Phase::Opening },
            setup,
            mark: 0,
            waited_mark: None,
            since: None,
            missing_since: None,
            refusal: None,
            recording: None,
        }
    }
    fn step(&self) -> u32 {
        u32::try_from(self.at + 1).expect("walk length fits u32")
    }
    fn save_refusal(&self, refusal: journey::Refusal) -> Refusal {
        Refusal::Save { name: self.name.clone(), step: self.step(), refusal }
    }
    fn save(&mut self, face: &mut BrowserFace<'_>, earlier: Option<Vec<u8>>) {
        self.phase = Phase::Download { after: face.downloads().len(), earlier };
        face.key(Key::Letter { letter: b's', alt: true }, true);
    }
    fn advance(&mut self) {
        self.at += 1;
        self.since = None;
        self.missing_since = None;
        self.phase = Phase::Start;
    }
    fn deadline(&self, now: Time) -> Result<(), Refusal> {
        match &self.phase {
            Phase::SetupStep { at } | Phase::SetupSee { at } => {
                if self.since.is_some_and(|since| now >= since.saturating_add(Duration::from_secs(60))) {
                    let prepared = self.setup.as_ref().expect("setup phase has a setup");
                    let (setup, line) = &prepared.sources[*at];
                    let sought = match &prepared.setup.steps[*at] {
                        SetupStep::Act { role, name, order, .. } => Find::exactly_named(*role, name).nth(*order),
                        SetupStep::See { text } => Find::text(text),
                    };
                    return Err(Refusal::SetupTimeout {
                        name: self.name.clone(),
                        setup: setup.clone(),
                        line: *line,
                        sought: sought.to_string(),
                    });
                }
                return Ok(());
            }
            Phase::MarkWait { mark, text } => {
                if self.since.is_some_and(|since| now >= since.saturating_add(Duration::from_secs(30))) {
                    return Err(Refusal::MarkTimeout { name: self.name.clone(), mark: *mark, text: text.clone() });
                }
                return Ok(());
            }
            _ => {}
        }
        if let Some(since) = self.missing_since {
            if now >= since.saturating_add(Duration::from_secs(10)) {
                return Err(Refusal::NoControl { name: self.name.clone(), step: self.step() });
            }
        } else if self.since.is_some_and(|since| now >= since.saturating_add(Duration::from_secs(30))) {
            return Err(Refusal::NotStill { name: self.name.clone(), step: self.step() });
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one phase dispatch keeps setup, reload and recorded walk transitions together"
    )]
    fn progress(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Result<Next<Box<[u8]>>, Refusal> {
        self.deadline(now)?;
        match std::mem::replace(&mut self.phase, Phase::Finished) {
            Phase::SetupStep { at } => {
                let prepared = self.setup.as_ref().expect("setup phase has a setup");
                match prepared.setup.steps.get(at) {
                    Some(SetupStep::Act { role, name, order, act }) => {
                        self.since.get_or_insert(now);
                        let find = Find::exactly_named(*role, name).nth(*order);
                        match face.find(&find) {
                            Found::Node(node) if face.allows(node, act) => {
                                let act = copy_act(act);
                                self.since = None;
                                self.phase = Phase::SetupStep { at: at + 1 };
                                return Ok(Next::Do(Doing::Act { node, act }));
                            }
                            Found::Node(_) | Found::Pending | Found::Absent => self.phase = Phase::SetupStep { at },
                        }
                    }
                    Some(SetupStep::See { .. }) => self.phase = Phase::SetupSee { at },
                    None => self.phase = Phase::Load,
                }
            }
            Phase::SetupSee { at } => {
                self.since.get_or_insert(now);
                let SetupStep::See { text } = &self.setup.as_ref().expect("setup exists").setup.steps[at] else {
                    unreachable!("see phase")
                };
                match face.find(&Find::text(text)) {
                    Found::Node(_) => {
                        self.since = None;
                        self.phase = Phase::SetupStep { at: at + 1 };
                    }
                    Found::Pending | Found::Absent => self.phase = Phase::SetupSee { at },
                }
            }
            Phase::Load => {
                let address = self.setup.as_ref().expect("setup exists").setup.reload.clone();
                self.since = None;
                // The face completes Go after the load, accepting its closing warning.
                // Opening then waits for that new page's asynchronous client start.
                self.phase = Phase::Opening;
                return Ok(Next::Do(Doing::Go { address }));
            }
            Phase::MarkWait { mark, text } => {
                self.since.get_or_insert(now);
                match face.find(&Find::text(&text)) {
                    Found::Node(_) => {
                        self.since = None;
                        self.waited_mark = Some(mark);
                        self.phase = Phase::Start;
                    }
                    Found::Pending | Found::Absent => self.phase = Phase::MarkWait { mark, text },
                }
            }
            Phase::Opening => {
                self.since.get_or_insert(now);
                // A module's load event can precede its asynchronous wasm start.
                // The first main region is drawn after the recorder's keys are installed.
                match face.find(&Find::named(Role::Main, b"")) {
                    Found::Node(_) => {
                        self.since = None;
                        self.phase = Phase::Start;
                    }
                    Found::Pending | Found::Absent => self.phase = Phase::Opening,
                }
            }
            Phase::Start => {
                let mark = self.mark + 1;
                if matches!(self.steps[self.at], Step::Mark)
                    && self.waited_mark != Some(mark)
                    && let Some(wait) = self
                        .setup
                        .as_ref()
                        .and_then(|prepared| prepared.setup.marks.iter().find(|wait| wait.mark == mark))
                {
                    self.since = None;
                    self.phase = Phase::MarkWait { mark, text: wait.text.clone() };
                    return Ok(Next::Wait { until: Some(now.saturating_add(POLL)) });
                }
                self.since.get_or_insert(now);
                self.save(face, None);
            }
            Phase::Download { after, earlier } => {
                if face.downloads().len() > after {
                    let path = &face.downloads().last().expect("newest completed download").path;
                    let save = fs::read(path).map_err(|error| Refusal::Harness {
                        name: self.name.clone(),
                        step: self.step(),
                        report: format!("cannot read download {}: {error}", path.display()),
                    })?;
                    if let Some(earlier) = earlier {
                        if journey::still(&earlier, &save)
                            .map_err(|unreadable| self.save_refusal(journey::Refusal::Unreadable(unreadable)))?
                        {
                            self.since = None;
                            self.phase = Phase::Still { save };
                        } else {
                            // Each failed comparison starts a fresh pair.
                            self.save(face, None);
                        }
                    } else {
                        let gap = match &self.steps[self.at] {
                            Step::Mark => Duration::from_secs(6),
                            Step::Act { .. } | Step::End => Duration::from_millis(500),
                        };
                        self.phase = Phase::Gap { earlier: save, until: now.saturating_add(gap) };
                    }
                } else {
                    self.phase = Phase::Download { after, earlier };
                }
            }
            Phase::Gap { earlier, until } => {
                if now >= until {
                    self.save(face, Some(earlier));
                } else {
                    self.phase = Phase::Gap { earlier, until };
                    return Ok(Next::Wait { until: Some(until) });
                }
            }
            Phase::Still { save } => match &self.steps[self.at] {
                Step::Act { act, event } => {
                    if let Some(control) = journey::control(&save, act, event).map_err(|why| self.save_refusal(why))? {
                        self.phase = Phase::Find { control };
                    } else {
                        self.missing_since.get_or_insert(now);
                        self.save(face, None);
                    }
                }
                Step::Mark => {
                    self.mark += 1;
                    face.key(Key::Letter { letter: b'm', alt: true }, true);
                    self.advance();
                    // A key mark's half-second window ends before another control is acted on.
                    self.phase = Phase::AfterMark { until: now.saturating_add(Duration::from_secs(1)) };
                }
                Step::End => {
                    self.recording = Some(save);
                    return Ok(Next::Done);
                }
            },
            Phase::AfterMark { until } => {
                if now >= until {
                    self.phase = Phase::Start;
                } else {
                    self.phase = Phase::AfterMark { until };
                    return Ok(Next::Wait { until: Some(until) });
                }
            }
            Phase::Find { control } => {
                let find = Find::exactly_named(control.role, &control.name).nth(control.order);
                match face.find(&find) {
                    Found::Node(node) => {
                        let Step::Act { act, .. } = &self.steps[self.at] else {
                            unreachable!("finding an act's control")
                        };
                        let act = copy_act(act);
                        self.advance();
                        return Ok(Next::Do(Doing::Act { node, act }));
                    }
                    Found::Pending => self.phase = Phase::Find { control },
                    Found::Absent => {
                        return Err(Refusal::Harness {
                            name: self.name.clone(),
                            step: self.step(),
                            report: format!("the live page has no {find}"),
                        });
                    }
                }
            }
            Phase::Finished => return Ok(Next::Done),
        }
        Ok(Next::Wait { until: Some(now.saturating_add(POLL)) })
    }
}
impl Play for Walker {
    fn at(&self) -> u32 {
        u32::try_from(self.at).expect("walk length fits u32")
    }
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>> {
        match self.progress(face, now) {
            Ok(next) => next,
            Err(why) => {
                // Done lets the harness close both children before returning the refusal.
                self.refusal = Some(why);
                Next::Done
            }
        }
    }
}
pub(crate) fn copy_act(act: &Act) -> Act {
    match act {
        Act::Near { edge } => Act::Near { edge: *edge },
        Act::Press => Act::Press,
        Act::Submit => Act::Submit,
        Act::Toggle => Act::Toggle,
        Act::Input { value } => Act::Input { value: value.clone() },
        Act::Drawing { .. } => unreachable!("walk refuses drawings before starting the harness"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::Token;

    fn walker() -> Walker {
        let setup = journey::setup(
            b"setup 1\npress button \"Create\"\nsee \"Ready\"\nreload \"/projects/1\"\nmark 1 \"Settled\"\n",
        )
        .expect("setup");
        Walker::new(
            "child",
            vec![Step::Mark, Step::End],
            Some(Prepared { setup, sources: vec![("parent".into(), 7), ("child".into(), 3)] }),
        )
    }
    #[test]
    fn a_setup_step_has_sixty_seconds_and_names_its_source() {
        for (at, phase) in [(0, Phase::SetupStep { at: 0 }), (1, Phase::SetupSee { at: 1 })] {
            let mut walker = walker();
            walker.phase = phase;
            walker.since = Some(Time::ZERO);
            assert!(walker.deadline(Time::from_nanos(59_999_999_999)).is_ok());
            let why = walker.deadline(Time::from_nanos(60_000_000_000)).expect_err("setup deadline");
            let Refusal::SetupTimeout { name, setup, line, sought } = why else { panic!("setup refusal: {why:?}") };
            assert_eq!(name, "child");
            assert_eq!((setup.as_str(), line), if at == 0 { ("parent", 7) } else { ("child", 3) });
            assert!(sought.contains(if at == 0 { "Create" } else { "Ready" }));
        }
    }
    #[test]
    fn a_mark_has_thirty_seconds_and_names_its_text() {
        let mut walker = walker();
        walker.phase = Phase::MarkWait { mark: 1, text: b"Settled".as_slice().into() };
        walker.since = Some(Time::ZERO);
        assert!(walker.deadline(Time::from_nanos(29_999_999_999)).is_ok());
        assert!(
            matches!(walker.deadline(Time::from_nanos(30_000_000_000)), Err(Refusal::MarkTimeout { name, mark: 1, text }) if name == "child" && text.as_ref() == b"Settled")
        );
    }
    #[test]
    fn setup_and_mark_queries_poll_and_reload_opens_the_new_client() {
        for phase in [
            Phase::SetupStep { at: 0 },
            Phase::SetupSee { at: 1 },
            Phase::MarkWait { mark: 1, text: b"Settled".as_slice().into() },
        ] {
            let mut walker = walker();
            walker.phase = phase;
            let mut face = BrowserFace::new(Token::new(1), "127.0.0.1:1");
            assert_eq!(
                walker.progress(&mut face, Time::ZERO).expect("pending query"),
                Next::Wait { until: Some(Time::from_nanos(50_000_000)) }
            );
        }
        let mut walker = walker();
        let mut face = BrowserFace::new(Token::new(1), "127.0.0.1:1");
        walker.phase = Phase::Load;
        assert_eq!(
            walker.progress(&mut face, Time::ZERO).expect("load"),
            Next::Do(Doing::Go { address: b"/projects/1".as_slice().into() })
        );
        assert!(matches!(walker.phase, Phase::Opening));
        walker.phase = Phase::Start;
        walker.progress(&mut face, Time::ZERO).expect("first mark wait");
        assert!(matches!(walker.phase, Phase::MarkWait { mark: 1, .. }));
    }
}
