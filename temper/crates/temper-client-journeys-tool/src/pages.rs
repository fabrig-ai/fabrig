//! The writer's stale walks, reprint and orphan-page removal
//! (temper/docs/design/client/journey-writer.md, 3).
use crate::{Refusal, io_refusal, names, printed, printed_page, run, tui};
use run::Destination;
use std::{
    fmt::Write,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

// The harness reserves four seconds inside its horizon for closing
// (skein/docs/design/ui/web/testing.md, 2).
pub(crate) const CLOSE_WINDOW: Duration = Duration::from_secs(4);

#[derive(Debug)]
pub(crate) struct Bound {
    pub word: String,
    pub duration: Duration,
}

impl Bound {
    pub fn rest(&self, elapsed: Duration) -> Duration {
        self.duration.saturating_sub(elapsed)
    }

    pub fn not_walked(&self) -> Stopped {
        Stopped::from(format!("not walked, past --within {}", self.word))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Stopped {
    pub why: String,
    pub scratch: Option<PathBuf>,
}

impl From<String> for Stopped {
    fn from(why: String) -> Self {
        Self { why, scratch: None }
    }
}

impl From<&str> for Stopped {
    fn from(why: &str) -> Self {
        Self::from(why.to_owned())
    }
}

impl Stopped {
    fn line(&self, name: &str) -> String {
        let mut line = format!("stopped {name}: {}", self.why.replace('\n', "; "));
        if let Some(scratch) = &self.scratch {
            write!(line, "; scratch kept at {}", scratch.display()).expect("write to String");
        }
        line
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Client {
    Web,
    Tui,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Journey {
    pub client: Client,
    pub name: String,
    pub stale: bool,
}

/// Replay through the print's current hosts, without writing or starting children.
pub(crate) fn stale(web: &Path, tui: &Path) -> Result<Vec<Journey>, Refusal> {
    let mut journeys = Vec::new();
    for (root, client) in [(web, Client::Web), (tui, Client::Tui)] {
        fs::read_dir(root).map_err(|_| io_refusal(root))?;
        for name in names(root)? {
            let path = root.join(&name).join("recording");
            let recording = fs::read(&path).map_err(|_| io_refusal(&path))?;
            let stopped = match client {
                Client::Web => printed(&name, &recording)?.is_err(),
                Client::Tui => tui::printed(&name, &recording)?.is_err(),
            };
            journeys.push(Journey { client, name, stale: stopped });
        }
    }
    Ok(journeys)
}

pub(crate) fn check(web: &Path, tui: &Path, mut show: impl FnMut(&str)) -> Result<bool, String> {
    let journeys = stale(web, tui).map_err(run::refusal)?;
    for journey in &journeys {
        show(&format!("{} {}", if journey.stale { "stale" } else { "fresh" }, journey.name));
    }
    Ok(journeys.iter().any(|journey| journey.stale))
}

/// The command's decisions; the shell supplies `again`, tests supply no process.
pub(crate) fn write(
    web: &Path,
    tui: &Path,
    journeys: &[Journey],
    bound: Option<&Bound>,
    mut clock: impl FnMut() -> Duration,
    mut walk: impl FnMut(&Journey, Option<Duration>) -> Result<(), Stopped>,
    mut show: impl FnMut(Destination, &str),
) -> Result<bool, String> {
    let mut outcomes = Vec::new();
    for journey in journeys {
        let result = if journey.stale {
            let rest = bound.map(|bound| bound.rest(clock()));
            let horizon = match journey.client {
                Client::Web => crate::again::HORIZON,
                Client::Tui => crate::tui_again::HORIZON,
            };
            let cut = rest.is_some_and(|rest| rest <= Duration::from_nanos(horizon.as_nanos()));
            let past = rest.is_some_and(|rest| rest < CLOSE_WINDOW);
            let mut result =
                if past { Err(bound.expect("rest has a bound").not_walked()) } else { walk(journey, rest) };
            if cut
                && !past
                && let (Some(bound), Err(stopped)) = (bound, &mut result)
                && bound.rest(clock()).is_zero()
            {
                stopped.why = format!("cut at --within {}", bound.word);
            }
            Some(result)
        } else {
            None
        };
        outcomes.push(result);
    }
    let unprinted = reprint(web, tui, &mut show)?;
    let mut stopped = false;
    for (journey, outcome) in journeys.iter().zip(outcomes) {
        let line = match outcome {
            Some(Err(why)) => {
                stopped = true;
                why.line(&journey.name)
            }
            outcome => {
                if unprinted.contains(&journey.name) {
                    stopped = true;
                    Stopped::from("still stale after its walk").line(&journey.name)
                } else {
                    format!("{} {}", if outcome.is_some() { "walked again" } else { "fresh" }, journey.name)
                }
            }
        };
        show(Destination::Stdout, &line);
    }
    Ok(stopped)
}

/// Print both clients, keeping stale pages, then remove only numbered orphan pages.
pub(crate) fn reprint(web: &Path, tui: &Path, show: impl FnMut(Destination, &str)) -> Result<Vec<String>, String> {
    let unprinted = run::reprint_roots(web, tui, Destination::Stderr, show)?;
    for (root, client) in [(web, Client::Web), (tui, Client::Tui)] {
        remove_orphans(root, client).map_err(run::refusal)?;
    }
    Ok(unprinted)
}

fn remove_orphans(root: &Path, client: Client) -> Result<(), Refusal> {
    for entry in fs::read_dir(root).map_err(|_| io_refusal(root))? {
        let entry = entry.map_err(|_| io_refusal(root))?;
        let directory = entry.path();
        if !entry.file_type().map_err(|_| io_refusal(&directory))?.is_dir() {
            continue;
        }
        let recording = directory.join("recording");
        if recording.try_exists().map_err(|_| io_refusal(&recording))? {
            continue;
        }
        for entry in fs::read_dir(&directory).map_err(|_| io_refusal(&directory))? {
            let entry = entry.map_err(|_| io_refusal(&directory))?;
            let path = entry.path();
            let page = match client {
                Client::Web => printed_page(&path),
                Client::Tui => tui::printed_file(&path),
            };
            if page && entry.file_type().map_err(|_| io_refusal(&path))?.is_file() {
                fs::remove_file(&path).map_err(|_| io_refusal(&path))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
