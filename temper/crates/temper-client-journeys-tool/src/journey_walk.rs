//! A journey's chain on one drive session (client/drive.md, 2; draft/journey-walks.md, 4.2).
use crate::{
    EnvironmentEntry, drive,
    line::{Record, Tui, Web},
};
use std::{collections::VecDeque, fs, io::Write, path::Path};
use temper_client_walk_lines::walk::{self, Client, Roots};

/// Walk only the supplied roots, writing the session beneath `options.scratch`.
/// The caller supplies the captured environment, including Chromium and PATH.
pub fn walk(
    roots: &Roots,
    name: &str,
    options: drive::Options,
    environment: &[EnvironmentEntry],
    out: &mut impl Write,
) -> drive::Exit {
    match execute(roots, name, options, environment, out) {
        Ok(exit) => exit,
        Err(why) => drive::Exit::own(why),
    }
}
fn execute(
    roots: &Roots,
    name: &str,
    options: drive::Options,
    environment: &[EnvironmentEntry],
    out: &mut impl Write,
) -> Result<drive::Exit, String> {
    let (options, mut walker, first) = prepare(roots, name, options)?;
    let mut session = match drive::Session::start(options, environment) {
        Ok(session) => Some(session),
        Err(exit) => return Ok(exit),
    };
    let mut actions: VecDeque<_> = vec![first].into();
    while let Some(action) = actions.pop_front() {
        match action {
            Action::Line(line) => match session.as_mut().expect("live session").line(&line.bytes) {
                Ok(answer) => actions.extend(walker.answer(answer)),
                Err(_) => actions.push_back(Action::End),
            },
            Action::Print(text) => {
                if let Err(why) = writeln!(out, "{text}").and_then(|()| out.flush()) {
                    // Even a broken output must stop the box before returning.
                    let _exit = session.take().expect("live session").end();
                    return Err(format!("output: {why}"));
                }
            }
            Action::End => {
                let exit = session.take().expect("end once").end();
                return Ok(walker.ended(exit));
            }
        }
    }
    unreachable!("walk ends its session")
}

#[derive(Debug)]
struct WalkLine {
    source: String,
    bytes: Box<[u8]>,
}
#[derive(Debug)]
enum Action {
    Line(WalkLine),
    Print(String),
    End,
}
struct Walker {
    name: String,
    lines: VecDeque<WalkLine>,
    source: Option<String>,
    stopped: Option<String>,
}
impl Walker {
    fn next(&mut self) -> Action {
        match self.lines.pop_front() {
            Some(line) => {
                self.source = Some(line.source.clone());
                Action::Line(line)
            }
            None => Action::End,
        }
    }
    fn answer(&mut self, answer: drive::Answer) -> Vec<Action> {
        let source = self.source.take().expect("one answer per line");
        let mut actions: Vec<_> =
            answer.print().lines().map(|line| Action::Print(format!("{source} {line}"))).collect();
        if let Some(why) = answer.failed {
            let why =
                why.strip_prefix("line ").and_then(|why| why.split_once(": ")).map_or(why.as_str(), |(_, why)| why);
            self.stopped = Some(format!("stopped at {source}: {}", why.replace(['\n', '\r'], " ")));
            self.lines.clear();
            actions.push(Action::End);
        } else {
            actions.push(self.next());
        }
        actions
    }
    fn ended(&self, mut exit: drive::Exit) -> drive::Exit {
        if exit.status == 0 {
            if let Some(stopped) = &self.stopped {
                exit.status = 5;
                exit.shown.push(stopped.clone());
            } else {
                exit.shown.push(format!("walked {}", self.name));
            }
        }
        exit
    }
}
fn prepare(roots: &Roots, name: &str, mut options: drive::Options) -> Result<(drive::Options, Walker, Action), String> {
    let chain = walk::read(roots, name).map_err(|why| format!("{}:{}: {}", why.journey, why.line, why.why))?;
    for directory in ["config", "data"] {
        let path = options.scratch.join(directory);
        match fs::symlink_metadata(&path) {
            Ok(_) => return Err(format!("{}: fresh walk requires no config/ and no data/ in scratch", path.display())),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
            Err(why) => return Err(format!("{}: {why}", path.display())),
        }
    }
    check_roots(roots, &options.scratch)?;
    options.journey = true;
    options.scripts.clear();
    let mut lines = VecDeque::new();
    for journey in chain.chain {
        if journey.scripts.is_some() {
            let root = match journey.client {
                Client::Web => &roots.web,
                Client::Tui => &roots.tui,
            };
            options.scripts.push(root.join(&journey.name).join("scripts.json"));
        }
        for numbered in journey.lines {
            if journey.name != name && matches!(numbered.line.record(), Record::Web(Web::Mark) | Record::Tui(Tui::Mark))
            {
                continue;
            }
            lines.push_back(WalkLine {
                source: format!("{}:{}", journey.name, numbered.number),
                bytes: numbered.line.print().into(),
            });
        }
    }
    let mut walker = Walker { name: name.into(), lines, source: None, stopped: None };
    let first = walker.next();
    Ok((options, walker, first))
}

fn check_roots(roots: &Roots, scratch: &Path) -> Result<(), String> {
    let scratch = fs::canonicalize(scratch).map_err(|why| format!("{}: {why}", scratch.display()))?;
    for root in [&roots.web, &roots.tui] {
        let root = match fs::canonicalize(root) {
            Ok(root) => root,
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => continue,
            Err(why) => return Err(format!("{}: {why}", root.display())),
        };
        if scratch.starts_with(&root)
            || ["session", "config", "data", "cache"].iter().any(|directory| root.starts_with(scratch.join(directory)))
        {
            return Err(format!("{}: session writes would overlap journey root {}", scratch.display(), root.display()));
        }
    }
    Ok(())
}

pub(crate) fn run(arguments: &[Box<[u8]>], environment: &[EnvironmentEntry]) -> drive::Exit {
    let parsed = arguments_for_walk(arguments);
    match parsed {
        Ok((name, options)) => walk(
            &Roots { web: Path::new(crate::JOURNEYS).into(), tui: Path::new(crate::TUI_JOURNEYS).into() },
            &name,
            options,
            environment,
            &mut std::io::stdout(),
        ),
        Err(why) => drive::Exit::own(why),
    }
}
fn arguments_for_walk(arguments: &[Box<[u8]>]) -> Result<(String, drive::Options), String> {
    let mut name = None;
    let mut session = vec![b"--journey".as_slice().into()];
    let mut args = arguments.iter();
    while let Some(argument) = args.next() {
        match argument.as_ref() {
            b"--bin" | b"--bundle" | b"--scratch" | b"--ports" | b"--widths" | b"--deployment" => {
                session.push(argument.clone());
                let value = args
                    .next()
                    .filter(|arg| !arg.is_empty() && !arg.starts_with(b"--"))
                    .ok_or_else(|| format!("missing {} value", String::from_utf8_lossy(argument)))?;
                session.push(value.clone());
            }
            b"--strict" => session.push(argument.clone()),
            flag if flag.starts_with(b"--") => {
                return Err(format!("unknown journey walk option {}", String::from_utf8_lossy(flag)));
            }
            bytes if name.is_none() => {
                name = Some(std::str::from_utf8(bytes).map_err(|_| "journey name is not UTF-8")?.to_owned());
            }
            _ => return Err("unexpected journey walk argument".into()),
        }
    }
    Ok((name.ok_or("missing journey NAME")?, drive::Options::parse(&session)?))
}

#[cfg(test)]
mod tests;
