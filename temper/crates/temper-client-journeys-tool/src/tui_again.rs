//! Recording a TUI walk over skein's browser harness and terminal startup
//! (temper/docs/design/client/tui/journeys.md, 8.1–8.2).
use crate::{
    Refusal,
    again::{Paths, Prepared},
};
use skein_fake_person::{Doing, Face, Find, Found, Next};
use skein_fake_person_browser::{
    BrowserFace,
    harness::{Harness, Names, Outcome, Output, Play, Printed, Sent, Setup},
};
use skein_lib::{Duration, Time};
use skein_ui::{Act, Role};
use skein_ui_screen_terminal::Key;
use std::{
    ffi::{OsStr, OsString},
    fs::{self, File},
    os::unix::{ffi::OsStrExt, fs::DirBuilderExt},
    path::Path,
};
use temper_client_shell_tui::journey::{self, WalkStep};
use temper_client_shell_web::journey::SetupStep;

use crate::tui_driver::{environment, key_bytes, optional_file, replaced, terminal::Terminal};
#[cfg(test)]
mod tests;

const HORIZON: Duration = Duration::from_secs(600);
const BOUND: Duration = Duration::from_secs(30);
const POLL: Duration = Duration::from_millis(50);

/// A complete TUI walk and its browser setup, validated before starting children.
#[derive(Debug)]
pub struct Starting {
    steps: Vec<WalkStep>,
    setup: Option<Prepared>,
}

/// Read the complete walk and setup before a build, terminal, browser or box starts.
pub fn starting(journeys: &Path, browser_journeys: &Path, name: &str) -> Result<Starting, Refusal> {
    if !crate::named(name) {
        return Err(Refusal::Name(name.to_owned()));
    }
    let path = journeys.join(name).join("walk");
    let bytes = fs::read(&path).map_err(|_| crate::io_refusal(&path))?;
    let steps = journey::walk(&bytes).map_err(|unreadable| Refusal::TuiWalk { name: name.to_owned(), unreadable })?;
    let setup = crate::again::read_setup(journeys, name)?;
    let setup = if let Some(prepared) = setup {
        if !prepared.setup.marks.is_empty() {
            let first = if prepared.setup.after.is_some() { 3 } else { 2 };
            return Err(Refusal::SetupUnreadable {
                name: name.to_owned(),
                unreadable: temper_client_shell_web::journey::Unreadable {
                    line: first + u32::try_from(prepared.setup.steps.len()).expect("setup fits u32") + 1,
                    why: "a TUI setup cannot hold a mark",
                },
            });
        }
        Some(crate::again::join_setup(prepared, browser_journeys, name)?)
    } else {
        None
    };
    Ok(Starting { steps, setup })
}

/// Follow an already validated walk, check its recording and add it.
pub fn again(
    journeys: &Path,
    tui_journeys: &Path,
    name: &str,
    starting: &Starting,
    paths: &Paths,
    given: &[crate::EnvironmentEntry],
) -> Result<(), Refusal> {
    for attempt in 1..=3 {
        let scratch = paths.scratch.join(format!("attempt-{attempt}"));
        fs::DirBuilder::new().mode(0o700).create(&scratch).map_err(|_| crate::io_refusal(&scratch))?;
        let attempt_paths = Paths {
            temper: paths.temper.clone(),
            bundle: paths.bundle.clone(),
            cache: paths.cache.clone(),
            chromium: paths.chromium.clone(),
            scratch,
        };
        match record(journeys, tui_journeys, name, starting, &attempt_paths, given) {
            Err(Refusal::TuiProof { refusal: journey::Refusal::Differs { .. }, .. }) if attempt < 3 => {}
            result => return result,
        }
    }
    unreachable!("the third attempt returns its result")
}

fn record(
    journeys: &Path,
    tui_journeys: &Path,
    name: &str,
    starting: &Starting,
    paths: &Paths,
    given: &[crate::EnvironmentEntry],
) -> Result<(), Refusal> {
    let fail = |report: String| Refusal::TuiAgain { name: name.to_owned(), step: 1, report };
    let environment =
        environment(given, &paths.scratch).map_err(|why| fail(format!("cannot build TUI environment: {why}")))?;
    for directory in ["home", "state", "bin", "tmp", "config", "data", "cache", "runtime", "tui-runtime"] {
        let path = paths.scratch.join(directory);
        fs::DirBuilder::new().mode(0o700).create(&path).map_err(|_| crate::io_refusal(&path))?;
    }
    let true_program = program("true", given).ok_or_else(|| fail("cannot find true on PATH".into()))?;
    for opener in ["xdg-open", "open"] {
        let path = paths.scratch.join("bin").join(opener);
        std::os::unix::fs::symlink(&true_program, &path).map_err(|_| crate::io_refusal(&path))?;
    }
    // Like browser again, env gives the harness's empty server environment
    // a PATH. HOME and temporary files also stay within this fresh run.
    let launcher = program("env", given).ok_or_else(|| fail("cannot find env on PATH".into()))?;
    let mut args: Vec<OsString> = environment
        .iter()
        .filter(|(name, _)| {
            [
                "PATH",
                "HOME",
                "TMPDIR",
                "XDG_STATE_HOME",
                "XDG_CONFIG_HOME",
                "XDG_DATA_HOME",
                "XDG_CACHE_HOME",
                "XDG_RUNTIME_DIR",
            ]
            .iter()
            .any(|forward| name == forward)
        })
        .map(|(name, value)| {
            let mut entry = name.clone();
            entry.push("=");
            if name == "XDG_RUNTIME_DIR" {
                entry.push(paths.scratch.join("runtime"));
            } else {
                entry.push(value);
            }
            entry
        })
        .collect();
    args.extend([
        paths.temper.clone().into_os_string(),
        "--fake-backends".into(),
        "--bundle".into(),
        paths.bundle.clone().into_os_string(),
    ]);
    for (flag, directory) in [("--config", "config"), ("--data", "data"), ("--cache", "cache")] {
        args.push(flag.into());
        let path = if directory == "cache" {
            paths.cache.clone().unwrap_or_else(|| paths.scratch.join(directory))
        } else {
            paths.scratch.join(directory)
        };
        args.push(path.into_os_string());
    }
    let mut harness = Harness::new(Setup {
        server: launcher,
        restart: args.clone(),
        args,
        printed: Printed { sent: Sent::Page, prefix: b"http://127.0.0.1:", on: Output::Stdout, names: Names::Url },
        chromium: paths.chromium.clone(),
        downloads: None,
        expected: vec![],
        origins: vec![],
    })
    .page_query(b"internal-loopback-navigation=1")
    .horizon(HORIZON);
    let mut walker = Walker::new(name, paths, environment, starting);
    let outcome = harness.run(&mut [&mut walker]);
    // Stop and reap the TUI even when the browser or box failed before next().
    drop(walker.terminal.take());
    if let Some(refusal) = walker.refusal {
        return Err(refusal);
    }
    if let Outcome::Failed { .. } = outcome {
        // A browser failure's snapshot or URL can contain an approval or link.
        return Err(walker.fail("browser or box failed; scratch kept for inspection"));
    }
    let recording = walker.recording.take().ok_or_else(|| walker.fail("the walk ended without a save"))?;
    let lines = journey::lines(&recording).map_err(|why| walker.save_refusal(journey::Refusal::Unreadable(why)))?;
    if let Some(step) = journey::follows(&walker.steps, &lines) {
        return Err(Refusal::WalkDiffers { name: name.to_owned(), step });
    }
    crate::add_to(journeys, tui_journeys, name, &recording, &crate::again::private_paths(paths)).map(|_| ())
}

fn program(name: &str, given: &[crate::EnvironmentEntry]) -> Option<std::path::PathBuf> {
    let path = OsStr::from_bytes(crate::table::PATH.value(given).unwrap_or_default());
    std::env::split_paths(path)
        .filter(|path| path.is_absolute())
        .map(|path| path.join(name))
        .find(|path| path.is_file())
}

enum Phase {
    SetupStep { at: usize },
    SetupSee { at: usize },
    Load,
    Opening,
    Address,
    Ready,
    Step,
    Save { earlier: Option<File> },
    Approve,
    Pressed,
    Finished,
}
struct Walker<'a> {
    name: String,
    paths: &'a Paths,
    environment: Vec<(OsString, OsString)>,
    steps: Vec<WalkStep>,
    setup: Option<&'a Prepared>,
    at: usize,
    phase: Phase,
    since: Option<Time>,
    terminal: Option<Terminal>,
    refusal: Option<Refusal>,
    recording: Option<Vec<u8>>,
}
impl<'a> Walker<'a> {
    fn new(name: &str, paths: &'a Paths, environment: Vec<(OsString, OsString)>, starting: &'a Starting) -> Self {
        Self {
            name: name.to_owned(),
            paths,
            environment,
            steps: starting.steps.clone(),
            setup: starting.setup.as_ref(),
            at: 0,
            phase: if starting.setup.is_some() { Phase::SetupStep { at: 0 } } else { Phase::Opening },
            since: None,
            terminal: None,
            refusal: None,
            recording: None,
        }
    }
    fn step(&self) -> u32 {
        u32::try_from(self.at + 1).expect("walk length fits u32")
    }
    fn fail(&self, report: &str) -> Refusal {
        Refusal::TuiAgain { name: self.name.clone(), step: self.step(), report: report.into() }
    }
    fn save_refusal(&self, refusal: journey::Refusal) -> Refusal {
        crate::tui::proof_refusal(&format!("{} at step {}", self.name, self.step()), refusal)
    }
    fn advance(&mut self) {
        self.at += 1;
        self.since = None;
        self.phase = Phase::Step;
    }
    fn terminal(&mut self) -> &mut Terminal {
        self.terminal.as_mut().expect("TUI started before walking")
    }
    fn save(&mut self) -> Result<(), Refusal> {
        let path = self.paths.scratch.join("state/temper/tui/recording");
        let earlier = optional_file(&path).map_err(|_| self.fail("cannot observe recording before save"))?;
        self.terminal().write(key_bytes(&Key::Alt('S')));
        self.phase = Phase::Save { earlier };
        Ok(())
    }
    fn deadline(&self, now: Time) -> Result<(), Refusal> {
        if let Phase::SetupStep { at } | Phase::SetupSee { at } = &self.phase {
            if self.since.is_some_and(|since| now >= since.saturating_add(Duration::from_secs(60))) {
                let prepared = self.setup.expect("setup phase has a setup");
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
        if self.since.is_some_and(|since| now >= since.saturating_add(BOUND)) {
            return Err(self.fail("step not completed within 30 s"));
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one phase dispatch keeps browser setup and terminal walk transitions together"
    )]
    fn progress(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Result<Next<Box<[u8]>>, Refusal> {
        self.deadline(now)?;
        if !matches!(self.phase, Phase::SetupStep { .. } | Phase::SetupSee { .. } | Phase::Load) {
            self.since.get_or_insert(now);
        }
        match std::mem::replace(&mut self.phase, Phase::Finished) {
            Phase::SetupStep { at } => {
                let prepared = self.setup.expect("setup phase has a setup");
                match prepared.setup.steps.get(at) {
                    Some(SetupStep::Act { role, name, order, act }) => {
                        self.since.get_or_insert(now);
                        let find = Find::exactly_named(*role, name).nth(*order);
                        match face.find(&find) {
                            Found::Node(node) if face.allows(node, act) => {
                                let act = crate::again::copy_act(act);
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
                let SetupStep::See { text } = &self.setup.expect("setup exists").setup.steps[at] else {
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
                let address = self.setup.expect("setup exists").setup.reload.clone();
                self.since = None;
                self.phase = Phase::Opening;
                return Ok(Next::Do(Doing::Go {
                    address: skein_fake_person_browser::harness::with_query(
                        &address,
                        b"internal-loopback-navigation=1",
                    ),
                }));
            }
            Phase::Opening => {
                // The load precedes the client's asynchronous sign-in. Its main
                // region is the same readiness seam browser again waits for.
                if let Found::Node(_) = face.find(&Find::named(Role::Main, b"")) {
                    face.address();
                    self.phase = Phase::Address;
                } else {
                    self.phase = Phase::Opening;
                }
            }
            Phase::Address => {
                let address = face.take_address().ok_or_else(|| self.fail("browser supplied no box address"))?;
                let address = origin(&address).ok_or_else(|| self.fail("browser supplied no loopback box address"))?;
                self.terminal = Some(
                    Terminal::start(
                        &self.paths.scratch,
                        &self.paths.temper.with_file_name("temper"),
                        &address,
                        &self.environment,
                    )
                    .map_err(|why| self.fail(why))?,
                );
                self.phase = Phase::Ready;
            }
            Phase::Ready => {
                if self.terminal().ready() {
                    self.phase = Phase::Step;
                    self.since = None;
                } else {
                    self.phase = Phase::Ready;
                }
            }
            Phase::Step => match self.steps.get(self.at).expect("validated walk ends once") {
                WalkStep::Key(key) => {
                    let bytes = key_bytes(key);
                    self.terminal().write(bytes);
                    self.advance();
                }
                WalkStep::Mark => {
                    self.terminal().write(key_bytes(&Key::Alt('M')));
                    self.advance();
                }
                WalkStep::See(_) | WalkStep::Approve | WalkStep::End => self.save()?,
            },
            Phase::Save { earlier } => {
                let path = self.paths.scratch.join("state/temper/tui/recording");
                if let Some(save) =
                    replaced(&path, earlier.as_ref()).map_err(|_| self.fail("cannot read replaced recording"))?
                {
                    match self.steps.get(self.at).expect("saving a walk step") {
                        WalkStep::See(text) => {
                            if journey::shows(&save, text).map_err(|why| self.save_refusal(why))? {
                                self.advance();
                            } else {
                                self.save()?;
                            }
                        }
                        WalkStep::Approve => {
                            let address = journey::browsed(&save)
                                .map_err(|why| self.save_refusal(journey::Refusal::Unreadable(why)))?;
                            if let Some(address) = address {
                                face.go_to(address);
                                self.phase = Phase::Approve;
                            } else {
                                self.save()?;
                            }
                        }
                        WalkStep::End => {
                            self.recording = Some(save);
                            return Ok(Next::Done);
                        }
                        WalkStep::Key(_) | WalkStep::Mark => unreachable!("keys do not request saves"),
                    }
                } else {
                    self.phase = Phase::Save { earlier };
                }
            }
            Phase::Approve => {
                let find = Find::exactly_named(Role::Button, b"Approve");
                match face.find(&find) {
                    Found::Node(node) if face.allows(node, &Act::Press) => {
                        self.phase = Phase::Pressed;
                        return Ok(Next::Do(Doing::Act { node, act: Act::Press }));
                    }
                    Found::Node(_) | Found::Absent | Found::Pending => self.phase = Phase::Approve,
                }
            }
            Phase::Pressed => self.advance(),
            Phase::Finished => return Ok(Next::Done),
        }
        Ok(Next::Wait { until: Some(now.saturating_add(POLL)) })
    }
}
impl Play for Walker<'_> {
    fn at(&self) -> u32 {
        u32::try_from(self.at).expect("walk length fits u32")
    }
    fn check_deadline(&mut self, _face: &mut BrowserFace<'_>, now: Time) {
        // The harness calls this even while a page's navigation is pending.
        let progress = self.terminal.as_mut().map_or(Ok(()), Terminal::tick);
        if self.refusal.is_none() {
            self.refusal = progress.map_err(|why| self.fail(why)).and_then(|()| self.deadline(now)).err();
        }
    }
    fn next(&mut self, face: &mut BrowserFace<'_>, now: Time) -> Next<Box<[u8]>> {
        if self.refusal.is_some() {
            return Next::Done;
        }
        match self.progress(face, now) {
            Ok(next) => next,
            Err(why) => {
                self.refusal = Some(why);
                Next::Done
            }
        }
    }
}

fn origin(url: &[u8]) -> Option<Vec<u8>> {
    let address = url.strip_prefix(b"http://127.0.0.1:")?;
    let port = address.split(|byte| *byte == b'/' || *byte == b'?' || *byte == b'#').next()?;
    std::str::from_utf8(port).ok()?.parse::<u16>().ok()?;
    let mut origin = b"http://127.0.0.1:".to_vec();
    origin.extend_from_slice(port);
    Some(origin)
}
